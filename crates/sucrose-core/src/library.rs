//! The library as the legacy app reads it: every folder of the library location is classified as a theme, a theme that
//! needs a newer app, or a corrupt folder (`docs/legacy/portal.md` section 2). Everything here only reads; deleting or
//! moving a folder is the service's job.

use std::{
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{
    settings::{SortKind, SortMode},
    theme::{INFO_FILE, SucroseInfo, ThemeError, read_info},
    version::LegacyVersion,
};

/// Whether this build can use a theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compatibility {
    /// The theme was made for this app version or an older one.
    Compatible,
    /// The theme was made for a newer app; the card stays visible but the theme cannot be used.
    NeedsNewerApp {
        /// The `AppVersion` of the theme.
        required: LegacyVersion,
    },
}

/// A folder of the library that is a usable theme (or one that needs a newer app).
#[derive(Debug, Clone)]
pub struct LibraryTheme {
    /// Folder name exactly as the file system returned it.
    pub folder: String,
    /// The theme folder.
    pub dir: PathBuf,
    /// Its `SucroseInfo.json`.
    pub info: SucroseInfo,
    /// Thumbnail file, verified to exist inside `dir`.
    pub thumbnail: PathBuf,
    /// Preview file, verified to exist inside `dir`.
    pub preview: PathBuf,
    /// Whether this build can use the theme.
    pub compatibility: Compatibility,
    /// Creation time of the folder, or its last write when the file system has no creation time.
    pub created: Option<SystemTime>,
    /// Last write of `SucroseInfo.json`.
    pub modified: Option<SystemTime>,
    /// Search tokens: title, description and tags, lowercased, without punctuation, de-duplicated.
    pub index: Vec<String>,
}

/// Why a folder of the library is not a theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorruptReason {
    /// The folder has no `SucroseInfo.json`.
    InfoMissing,
    /// `SucroseInfo.json` is not a valid theme description.
    InfoInvalid,
    /// `Preview` is empty or names no file.
    PreviewMissing,
    /// `Thumbnail` is empty or names no file.
    ThumbnailMissing,
    /// `Source` is neither an address nor an existing file.
    SourceMissing,
    /// `Preview`, `Thumbnail` or `Source` leaves the theme folder.
    PathEscapes,
    /// The folder could not be read; never a reason to delete it.
    Unreadable,
}

/// A folder of the library that is not a theme.
#[derive(Debug, Clone)]
pub struct CorruptTheme {
    /// Folder name exactly as the file system returned it.
    pub folder: String,
    /// The folder.
    pub dir: PathBuf,
    /// What is wrong with it.
    pub reason: CorruptReason,
}

/// The result of scanning a library location.
#[derive(Debug, Clone, Default)]
pub struct Scan {
    /// Themes, in folder-name order.
    pub themes: Vec<LibraryTheme>,
    /// Folders that are not themes, in folder-name order.
    pub corrupt: Vec<CorruptTheme>,
    /// Entries that are not folders (loose files); the legacy scan lists folders only.
    pub ignored: usize,
}

/// Characters that may appear in the tail of an address (the legacy regex class `[\w.,@?^=%&:/~+#-]`).
fn is_tail_char(c: char) -> bool {
    is_word_char(c)
        || matches!(
            c,
            '.' | ',' | '@' | '?' | '^' | '=' | '%' | '&' | ':' | '/' | '~' | '+' | '#' | '-'
        )
}

/// Characters an address may end with (the tail class without `.`, `,` and `:`).
fn is_tail_end(c: char) -> bool {
    is_tail_char(c) && !matches!(c, '.' | ',' | ':')
}

/// Legacy `\w`: letters, digits and the underscore.
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Characters of one host label (`[\w-]`).
fn is_label_char(c: char) -> bool {
    is_word_char(c) || c == '-'
}

/// Whether `text` is an address the legacy theme check accepts: the scheme `http` or `https` (any case), `://`, a host
/// with at least two dot-separated labels, then an optional path, query or fragment and an optional final dot. It is
/// the legacy `IsUrl` expression written as code so no regex crate is needed; unlike .NET's `$` it does not accept a
/// trailing line feed.
pub fn is_url(text: &str) -> bool {
    let Some((scheme, rest)) = text.split_once("://") else {
        return false;
    };
    if !(scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")) {
        return false;
    }
    // The first label is the whole run of label characters; a dot and one more label character must follow.
    let label_len: usize = rest.chars().take_while(|c| is_label_char(*c)).map(char::len_utf8).sum();
    if label_len == 0 {
        return false;
    }
    let mut after = rest[label_len..].chars();
    if after.next() != Some('.') || !after.next().is_some_and(is_label_char) {
        return false;
    }
    // Whatever follows is the tail: characters of the tail class ending in a tail-end character, or nothing, and
    // then at most one final dot.
    let tail = after.as_str();
    let tail = tail.strip_suffix('.').unwrap_or(tail);
    tail.is_empty() || (tail.chars().all(is_tail_char) && tail.chars().next_back().is_some_and(is_tail_end))
}

/// Why a path named by `SucroseInfo.json` cannot be used.
enum Problem {
    /// Empty, or no regular file there.
    Missing,
    /// Absolute, prefixed, containing `..`, or resolving outside the theme folder.
    Escapes,
    /// The file system could not answer.
    Unreadable,
}

/// Judges a path on its text: it must be relative, free of drive and device prefixes and of `..` components.
fn text_escapes(path: &str) -> bool {
    path.starts_with(['/', '\\']) || path.contains([':', '\0']) || path.split(['/', '\\']).any(|part| part == "..")
}

/// A regular file named `relative` that stays inside the theme folder once links are resolved (T-03-04). `root` is
/// the canonical theme folder.
fn contained_file(dir: &Path, root: &Path, relative: &str) -> Result<PathBuf, Problem> {
    if relative.is_empty() {
        return Err(Problem::Missing);
    }
    if text_escapes(relative) {
        return Err(Problem::Escapes);
    }
    let path = dir.join(relative);
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return Err(Problem::Missing),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(Problem::Missing),
        Err(_) => return Err(Problem::Unreadable),
    }
    let real = fs::canonicalize(&path).map_err(|_| Problem::Unreadable)?;
    if real.starts_with(root) {
        Ok(path)
    } else {
        Err(Problem::Escapes)
    }
}

/// The reason a folder is corrupt when one of its three files is not usable: a file system problem outranks an
/// escape, which outranks a plain missing file, so a transient error never reads as "delete me".
fn worst(problems: [Option<(Problem, CorruptReason)>; 3]) -> Option<CorruptReason> {
    let mut missing = None;
    let mut escapes = false;
    for (problem, reason) in problems.into_iter().flatten() {
        match problem {
            Problem::Unreadable => return Some(CorruptReason::Unreadable),
            Problem::Escapes => escapes = true,
            Problem::Missing => missing = missing.or(Some(reason)),
        }
    }
    if escapes {
        Some(CorruptReason::PathEscapes)
    } else {
        missing
    }
}

/// Classifies one folder of the library.
fn classify(dir: &Path, folder: &str, app: &LegacyVersion) -> Result<LibraryTheme, CorruptReason> {
    let info_path = dir.join(INFO_FILE);
    let info_meta = match fs::metadata(&info_path) {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => return Err(CorruptReason::InfoMissing),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(CorruptReason::InfoMissing),
        Err(_) => return Err(CorruptReason::Unreadable),
    };
    let info = read_info(dir).map_err(|error| match error {
        ThemeError::Missing(_) => CorruptReason::InfoMissing,
        ThemeError::Unreadable { .. } => CorruptReason::Unreadable,
        ThemeError::Invalid { .. } => CorruptReason::InfoInvalid,
    })?;
    let root = fs::canonicalize(dir).map_err(|_| CorruptReason::Unreadable)?;
    let preview = contained_file(dir, &root, &info.preview);
    let thumbnail = contained_file(dir, &root, &info.thumbnail);
    // A Source is an address or a file inside the folder; the file itself is the theme's business, only its
    // existence and containment decide validity here.
    let source = if is_url(&info.source) {
        Ok(())
    } else if info.source.contains("://") {
        Err(Problem::Missing)
    } else {
        contained_file(dir, &root, &info.source).map(|_| ())
    };
    let (preview, thumbnail) = match (preview, thumbnail, source) {
        (Ok(preview), Ok(thumbnail), Ok(())) => (preview, thumbnail),
        (preview, thumbnail, source) => {
            let reason = worst([
                preview.err().map(|problem| (problem, CorruptReason::PreviewMissing)),
                thumbnail
                    .err()
                    .map(|problem| (problem, CorruptReason::ThumbnailMissing)),
                source.err().map(|problem| (problem, CorruptReason::SourceMissing)),
            ]);
            return Err(reason.unwrap_or(CorruptReason::Unreadable));
        }
    };
    let compatibility = if info.app_version > *app {
        Compatibility::NeedsNewerApp {
            required: info.app_version,
        }
    } else {
        Compatibility::Compatible
    };
    let created = fs::metadata(dir)
        .ok()
        .and_then(|metadata| metadata.created().or_else(|_| metadata.modified()).ok());
    Ok(LibraryTheme {
        folder: folder.to_owned(),
        dir: dir.to_path_buf(),
        info,
        thumbnail,
        preview,
        compatibility,
        created,
        modified: info_meta.modified().ok(),
        // Filled in by the search index step.
        index: Vec::new(),
    })
}

/// Scans `location` read-only. Every folder is a theme (also one that needs a newer app) or a corrupt folder; entries
/// that are not folders are only counted. An error means the location itself is missing or unreadable, in which case
/// nothing may be concluded about its contents.
pub fn scan(location: &Path, app: &LegacyVersion) -> io::Result<Scan> {
    let mut entries = fs::read_dir(location)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(DirEntry::file_name);
    let mut scan = Scan::default();
    for entry in entries {
        let dir = entry.path();
        // Names stay as the file system returns them (FMT-06); `dir` is always the real path.
        let folder = entry.file_name().to_string_lossy().into_owned();
        match fs::metadata(&dir) {
            Ok(metadata) if metadata.is_dir() => match classify(&dir, &folder, app) {
                Ok(theme) => scan.themes.push(theme),
                Err(reason) => scan.corrupt.push(CorruptTheme { folder, dir, reason }),
            },
            Ok(_) => scan.ignored += 1,
            // A link to nothing is no folder; anything else the file system cannot say is never a deletion reason.
            Err(error) if error.kind() == io::ErrorKind::NotFound => scan.ignored += 1,
            Err(_) => scan.corrupt.push(CorruptTheme {
                folder,
                dir,
                reason: CorruptReason::Unreadable,
            }),
        }
    }
    Ok(scan)
}

/// Title shown for a theme (placeholder, implemented next).
pub fn display_title<'a>(_info: &'a SucroseInfo, folder: &'a str) -> &'a str {
    folder
}

/// Sorts themes (placeholder, implemented next).
pub fn sort(_themes: &mut [LibraryTheme], _mode: SortMode, _kind: SortKind) {}

/// Searches themes (placeholder, implemented next).
pub fn search(_themes: &[LibraryTheme], _query: &str) -> Vec<usize> {
    Vec::new()
}
