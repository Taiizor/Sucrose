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
    let index = build_index(&info);
    Ok(LibraryTheme {
        folder: folder.to_owned(),
        dir: dir.to_path_buf(),
        info,
        thumbnail,
        preview,
        compatibility,
        created,
        modified: info_meta.modified().ok(),
        index,
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

/// Language of the Portal in Phase 3; the language setting arrives with UI-06. Matched like the legacy culture, ignoring
/// letter case.
const CURRENT_LANGUAGE: &str = "EN";

/// The title to show for a theme: the text of the current language, then the base title, then the folder name.
pub fn display_title<'a>(info: &'a SucroseInfo, folder: &'a str) -> &'a str {
    let localized = info
        .localization
        .as_ref()
        .and_then(|texts| {
            texts
                .iter()
                .find(|(language, _)| language.eq_ignore_ascii_case(CURRENT_LANGUAGE))
        })
        .map(|(_, text)| text.title.as_str())
        .filter(|title| !title.trim().is_empty());
    localized
        .or(Some(info.title.as_str()).filter(|title| !title.trim().is_empty()))
        .unwrap_or(folder)
}

/// The description to show for a theme: the text of the current language, then the base description; `None` when both
/// are empty.
pub fn display_description(info: &SucroseInfo) -> Option<&str> {
    let localized = info
        .localization
        .as_ref()
        .and_then(|texts| {
            texts
                .iter()
                .find(|(language, _)| language.eq_ignore_ascii_case(CURRENT_LANGUAGE))
        })
        .map(|(_, text)| text.description.as_str())
        .filter(|description| !description.trim().is_empty());
    localized.or(Some(info.description.as_str()).filter(|description| !description.trim().is_empty()))
}

/// Whether `c` is punctuation in the sense of the legacy `\p{P}` (symbols such as `$` and `+` are not). Characters
/// outside ASCII are covered by the common blocks (general punctuation, CJK and full-width forms), not by the full
/// Unicode table the standard library does not carry.
fn is_punctuation(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_punctuation() && !matches!(c, '$' | '+' | '<' | '=' | '>' | '^' | '`' | '|' | '~')
    } else {
        matches!(
            c,
            '\u{A1}' | '\u{A7}' | '\u{AB}' | '\u{B6}' | '\u{B7}' | '\u{BB}' | '\u{BF}'
                | '\u{2010}'..='\u{2027}'
                | '\u{2030}'..='\u{2043}'
                | '\u{2045}'..='\u{2051}'
                | '\u{2053}'..='\u{205E}'
                | '\u{3001}'..='\u{3003}'
                | '\u{3008}'..='\u{3011}'
                | '\u{FF01}'..='\u{FF03}'
                | '\u{FF05}'..='\u{FF0A}'
                | '\u{FF0C}'..='\u{FF0F}'
                | '\u{FF1A}'..='\u{FF1B}'
                | '\u{FF1F}'..='\u{FF20}'
        )
    }
}

/// A search word: punctuation removed, then lowercased with `str::to_lowercase` only. No Turkish-specific folding is
/// done; the legacy culture-dependent lowercase differs only for Turkish text and is accepted until UI-06.
fn token(word: &str) -> String {
    word.chars()
        .filter(|c| !is_punctuation(*c))
        .collect::<String>()
        .to_lowercase()
}

/// Search tokens of a theme: the base title and description, the texts of every complete localization and the tags,
/// split on whitespace, de-duplicated, in that order.
fn build_index(info: &SucroseInfo) -> Vec<String> {
    let mut texts: Vec<&str> = vec![&info.title, &info.description];
    if let Some(localization) = &info.localization {
        for text in localization.values() {
            if !text.title.trim().is_empty() && !text.description.trim().is_empty() {
                texts.push(&text.title);
                texts.push(&text.description);
            }
        }
    }
    if let Some(tags) = &info.tags {
        texts.extend(tags.iter().map(String::as_str));
    }
    let mut index: Vec<String> = Vec::new();
    for word in texts.iter().flat_map(|text| text.split_whitespace()) {
        let word = token(word);
        if !word.is_empty() && !index.contains(&word) {
            index.push(word);
        }
    }
    index
}

/// Sorts themes by the chosen key. The sort is stable by construction: themes with equal keys keep folder-name order,
/// in both directions. `Name` compares titles lowercased, as the legacy culture comparison does for ordinary text.
pub fn sort(themes: &mut [LibraryTheme], mode: SortMode, kind: SortKind) {
    themes.sort_by(|a, b| {
        let order = match mode {
            SortMode::Name => display_title(&a.info, &a.folder)
                .to_lowercase()
                .cmp(&display_title(&b.info, &b.folder).to_lowercase()),
            SortMode::Creation => a.created.cmp(&b.created),
            SortMode::Modification => a.modified.cmp(&b.modified),
        };
        let order = if kind == SortKind::Descending {
            order.reverse()
        } else {
            order
        };
        order.then_with(|| a.folder.cmp(&b.folder))
    });
}

/// Indexes into `themes` of the themes that match `query`, best first. A theme matches when at least one query word is
/// a substring of one of its index tokens; themes are ordered by the number of query words matched, descending, and
/// ties keep the order of `themes`. An empty query (or one that is only punctuation) returns every index in order.
pub fn search(themes: &[LibraryTheme], query: &str) -> Vec<usize> {
    let mut words: Vec<String> = Vec::new();
    for word in query.split_whitespace() {
        let word = token(word);
        if !word.is_empty() && !words.contains(&word) {
            words.push(word);
        }
    }
    if words.is_empty() {
        return (0..themes.len()).collect();
    }
    let mut hits: Vec<(usize, usize)> = themes
        .iter()
        .enumerate()
        .filter_map(|(position, theme)| {
            let matched = words
                .iter()
                .filter(|word| theme.index.iter().any(|token| token.contains(word.as_str())))
                .count();
            (matched > 0).then_some((position, matched))
        })
        .collect();
    // `sort_by` is stable, so equal counts keep the input order.
    hits.sort_by_key(|hit| std::cmp::Reverse(hit.1));
    hits.into_iter().map(|(position, _)| position).collect()
}
