//! The virtual origin of web wallpapers (D-14) and the one function that maps a request path to a file (SEC-02).
//!
//! Every theme page is served from [`ORIGIN`]. A request is answered from the theme folder first and the
//! generated-content folder second (the legacy lookup order). [`resolve`] is pure in the sense that matters for
//! security: it holds no state and opens no file, so parallel requests resolve independently.

use std::path::{Path, PathBuf};

/// The one origin every web wallpaper is served from; its storage (localStorage, IndexedDB, cookies) is bound to it.
pub const ORIGIN: &str = "https://sucrose.wallpaper.engine/";
/// The host of [`ORIGIN`]: the only host Sucrose handles; every other address keeps the engine's own networking.
pub const HOST: &str = "sucrose.wallpaper.engine";
/// The longest URL path (bytes, before decoding) the resolver looks at; a longer one is refused.
pub const MAX_PATH_BYTES: usize = 2048;

/// Why a request path was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Reject {
    /// No file name at all (the root).
    #[error("empty path")]
    Empty,
    /// A NUL character (also encoded as %00).
    #[error("NUL in path")]
    Nul,
    /// A backslash (also encoded as %5c).
    #[error("backslash in path")]
    Backslash,
    /// A drive letter, a stream name or any other colon.
    #[error("drive letter or colon in path")]
    DriveOrColon,
    /// A leading double slash (UNC).
    #[error("UNC path")]
    Unc,
    /// A `..` segment.
    #[error("parent directory segment")]
    Dots,
    /// An escape that decodes to another escape (%252e): never decoded twice, so refused.
    #[error("double-encoded path")]
    DoubleEncoded,
    /// The path decodes to bytes that are not UTF-8.
    #[error("path is not valid UTF-8")]
    BadUtf8,
    /// The path is longer than [`MAX_PATH_BYTES`].
    #[error("path is too long")]
    TooLong,
    /// The file lies outside the root (a junction or link leads out).
    #[error("outside the served folders")]
    Outside,
    /// Nothing there, or a folder.
    #[error("not a file")]
    NotAFile,
    /// The file system refused the lookup.
    #[error("file system error")]
    Io,
}

impl Reject {
    /// Whether the reason is a traversal attempt rather than a page asking for a file that is not there.
    #[must_use]
    pub fn is_traversal(self) -> bool {
        !matches!(self, Self::Empty | Self::NotAFile | Self::Io)
    }
}

/// A file the origin serves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFile {
    /// The canonical path of the file.
    pub path: PathBuf,
    /// Its length in bytes.
    pub len: u64,
    /// Its MIME type.
    pub mime: &'static str,
    /// The index in the roots slice of the folder it was found in.
    pub root: usize,
}

/// What a `Range` header asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteRange {
    /// The whole file (no range, several ranges, or a header that cannot be read).
    Full,
    /// Bytes `start..=end`.
    Partial {
        /// First byte.
        start: u64,
        /// Last byte (inclusive).
        end: u64,
    },
    /// The range starts past the end of the file.
    Unsatisfiable,
}

/// Maps a request path to a file under one of `roots` (the theme folder, then the generated-content folder).
pub fn resolve(url_path: &str, roots: &[&Path]) -> Result<ResolvedFile, Reject> {
    let _ = (url_path, roots);
    Err(Reject::Io)
}

/// The path (with query and fragment) of an address on the virtual host, or `None` for any other address.
#[must_use]
pub fn url_path(url: &str) -> Option<&str> {
    let _ = url;
    None
}

/// The address of a theme's page: the origin plus the path of `source` (absolute under `theme_dir`, or already
/// relative) with every segment percent-encoded.
#[must_use]
pub fn page_url(theme_dir: &Path, source: &str) -> String {
    let _ = (theme_dir, source);
    String::new()
}

/// Reads a `Range` request header against a file of `len` bytes.
#[must_use]
pub fn byte_range(header: &str, len: u64) -> ByteRange {
    let _ = (header, len);
    ByteRange::Full
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicU32, Ordering},
    };

    use super::*;

    /// A scratch folder under the temp folder, removed on drop.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            static NEXT: AtomicU32 = AtomicU32::new(0);
            let dir = std::env::temp_dir().join(format!(
                "sucrose-origin-{name}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("create scratch folder");
            Self(dir)
        }

        fn file(&self, rel: &str, body: &str) {
            let path = self.0.join(rel);
            fs::create_dir_all(path.parent().expect("parent")).expect("create folders");
            fs::write(path, body).expect("write file");
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn origin_finds_a_theme_file() {
        let theme = Scratch::new("theme");
        let content = Scratch::new("content");
        theme.file("index.html", "<html>");

        let file = resolve("/index.html", &[&theme.0, &content.0]).expect("resolves");

        assert_eq!(file.len, 6);
        assert_eq!(file.mime, "text/html");
        assert_eq!(file.root, 0);
        assert!(file.path.ends_with("index.html"));
    }

    #[test]
    fn origin_finds_a_name_present_only_in_content() {
        let theme = Scratch::new("theme");
        let content = Scratch::new("content");
        content.file("generated.html", "<p>");

        let file = resolve("/generated.html", &[&theme.0, &content.0]).expect("resolves");

        assert_eq!(file.root, 1);
    }

    #[test]
    fn origin_prefers_the_theme_when_both_have_the_name() {
        let theme = Scratch::new("theme");
        let content = Scratch::new("content");
        theme.file("page.html", "theme");
        content.file("page.html", "content!");

        let file = resolve("/page.html", &[&theme.0, &content.0]).expect("resolves");

        assert_eq!(file.root, 0);
        assert_eq!(file.len, 5);
    }

    #[test]
    fn origin_ignores_query_and_fragment() {
        let theme = Scratch::new("theme");
        theme.file("sub/page.html", "x");

        assert!(resolve("/sub/page.html?x=1#y", &[&theme.0]).is_ok());
        assert!(resolve("/sub/page.html#y", &[&theme.0]).is_ok());
    }

    #[test]
    fn origin_decodes_a_name_once() {
        let theme = Scratch::new("theme");
        theme.file("my page.html", "x");
        theme.file("100%.txt", "x");

        assert!(resolve("/my%20page.html", &[&theme.0]).is_ok());
        assert!(resolve("/100%25.txt", &[&theme.0]).is_ok());
    }

    #[test]
    fn origin_skips_a_root_that_does_not_exist() {
        let theme = Scratch::new("theme");
        theme.file("a.js", "x");
        let missing = theme.0.join("no-such-folder");

        assert!(resolve("/a.js", &[&missing, &theme.0]).is_ok());
        assert_eq!(resolve("/b.js", &[&missing]), Err(Reject::NotAFile));
    }

    #[test]
    fn origin_url_path_takes_only_the_virtual_host() {
        assert_eq!(url_path("https://sucrose.wallpaper.engine/a/b?x=1"), Some("/a/b?x=1"));
        assert_eq!(url_path("https://sucrose.wallpaper.engine"), Some(""));
        assert_eq!(url_path("HTTPS://Sucrose.Wallpaper.Engine/x"), Some("/x"));
        assert_eq!(url_path("https://sucrose.wallpaper.engine.evil.example/x"), None);
        assert_eq!(url_path("https://other.example/x"), None);
        assert_eq!(url_path("http://sucrose.wallpaper.engine/x"), None);
        assert_eq!(url_path("sucrose.wallpaper.engine/x"), None);
    }

    #[test]
    fn origin_page_url_encodes_each_segment() {
        let theme = Path::new("C:\\t\\a b");
        assert_eq!(
            page_url(theme, "pages/start.html"),
            "https://sucrose.wallpaper.engine/pages/start.html"
        );
        assert_eq!(
            page_url(theme, "C:\\t\\a b\\pages\\x y.html"),
            "https://sucrose.wallpaper.engine/pages/x%20y.html"
        );
        assert_eq!(
            page_url(theme, "\\\\?\\c:\\T\\A B\\\u{fc}#.html"),
            "https://sucrose.wallpaper.engine/%C3%BC%23.html"
        );
    }
}
