//! The library as the legacy app reads it: every folder of the library location is classified as a theme, a theme that
//! needs a newer app, or a corrupt folder (`docs/legacy/portal.md` section 2). Everything here only reads; deleting or
//! moving a folder is the service's job.

use std::{
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{theme::SucroseInfo, version::LegacyVersion};

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

/// Scans `location` (placeholder, implemented next).
pub fn scan(_location: &Path, _app: &LegacyVersion) -> io::Result<Scan> {
    Ok(Scan::default())
}

/// Whether `text` is an address the legacy theme check accepts (placeholder, implemented next).
pub fn is_url(_text: &str) -> bool {
    false
}
