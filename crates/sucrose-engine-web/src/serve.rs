//! The engine-neutral reply planner of the virtual origin (D-14): which file a request for
//! `https://sucrose.wallpaper.engine/` gets, with which status and headers, and what is logged.
//!
//! The decisions live in [`plan`], which is plain Rust over [`origin::resolve`]. Each web engine's adapter (CEF's scheme
//! handler, WebView2's WebResourceRequested handler) only streams what the plan says and decides nothing itself, so the
//! containment rules of SEC-02 exist once.

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use crate::origin::{self, ByteRange};

/// The legacy CORS headers of the Watson server, kept for parity (every page is on the same origin, so they add nothing).
const CORS: [(&str, &str); 3] = [
    ("Access-Control-Allow-Origin", "*"),
    ("Access-Control-Allow-Headers", "Content-Type"),
    ("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS, PATCH"),
];

/// The body of every refusal.
const NOT_FOUND_BODY: &str = "File not found.";

/// The longest request path written to the log.
const LOGGED_PATH_CHARS: usize = 200;

/// What a reply carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    /// A short text.
    Text(&'static str),
    /// `len` bytes of a file, from byte `start`.
    File {
        /// The canonical path.
        path: PathBuf,
        /// The first byte to send.
        start: u64,
        /// How many bytes to send.
        len: u64,
    },
}

/// The answer to one request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    /// The HTTP status.
    pub status: u16,
    /// The reason phrase.
    pub status_text: &'static str,
    /// The MIME type.
    pub mime: &'static str,
    /// The headers besides the type and the length.
    pub headers: Vec<(&'static str, String)>,
    /// The body.
    pub content: Content,
}

impl Reply {
    /// The refusal for everything that is not a served file.
    #[must_use]
    pub fn not_found() -> Self {
        Self {
            status: 404,
            status_text: "Not Found",
            mime: "text/plain",
            headers: cors(),
            content: Content::Text(NOT_FOUND_BODY),
        }
    }
}

fn cors() -> Vec<(&'static str, String)> {
    CORS.iter().map(|(name, value)| (*name, (*value).to_owned())).collect()
}

/// A request path as it is written to the log: cut, so a long attack path does not flood it.
fn shown(path: &str) -> String {
    path.chars().take(LOGGED_PATH_CHARS).collect()
}

/// Decides the answer to a request for `url` (with its `Range` header, if any) over the `roots` (the theme folder,
/// then the generated-content folder), and logs it: a served file at debug, a traversal-class refusal at info, a file
/// that is just not there at debug. A single range is answered with 206, a range past the end with 416, anything else
/// with the whole file.
#[must_use]
pub fn plan(url: &str, range: Option<&str>, roots: &[&Path]) -> Reply {
    let Some(path) = origin::url_path(url) else {
        tracing::info!(url = %shown(url), "origin request rejected: not an address of the virtual host");
        return Reply::not_found();
    };
    match origin::resolve(path, roots) {
        Ok(file) => {
            let mut headers = cors();
            headers.push(("Accept-Ranges", "bytes".to_owned()));
            let (status, status_text, start, len) = match range
                .map_or(ByteRange::Full, |header| origin::byte_range(header, file.len))
            {
                ByteRange::Full => (200, "OK", 0, file.len),
                ByteRange::Partial { start, end } => {
                    headers.push(("Content-Range", format!("bytes {start}-{end}/{}", file.len)));
                    (206, "Partial Content", start, end - start + 1)
                }
                ByteRange::Unsatisfiable => {
                    headers.push(("Content-Range", format!("bytes */{}", file.len)));
                    tracing::debug!(path = %shown(path), root = file.root, status = 416, "origin request not satisfiable");
                    return Reply {
                        status: 416,
                        status_text: "Range Not Satisfiable",
                        mime: "text/plain",
                        headers,
                        content: Content::Text(""),
                    };
                }
            };
            match headers.iter().find(|(name, _)| *name == "Content-Range") {
                Some((_, content_range)) => tracing::debug!(
                    path = %shown(path),
                    root = file.root,
                    status,
                    content_range = %content_range,
                    "origin request served"
                ),
                None => tracing::debug!(path = %shown(path), root = file.root, status, "origin request served"),
            }
            Reply {
                status,
                status_text,
                mime: file.mime,
                headers,
                content: Content::File {
                    path: file.path,
                    start,
                    len,
                },
            }
        }
        Err(reason) => {
            if reason.is_traversal() {
                tracing::info!(path = %shown(path), %reason, "origin request rejected");
            } else {
                tracing::debug!(path = %shown(path), %reason, "origin request rejected");
            }
            Reply::not_found()
        }
    }
}

/// The most bytes of one partial reply an adapter that has to hold the body in memory (WebView2) sends: 4 MiB. A
/// browser asks for `bytes=0-` of a long video and takes a shorter 206, then asks again from where it ended, so the cap
/// costs round trips and never correctness. CEF streams a range from the file and does not use it.
pub const MAX_RANGE_BODY: u64 = 4 * 1024 * 1024;

/// The part of a partial reply an in-memory adapter sends: where to read, how much, and the `Content-Range` that names
/// exactly those bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeBody {
    /// The first byte to read.
    pub offset: u64,
    /// How many bytes to read: the planned length, at most [`MAX_RANGE_BODY`].
    pub len: u64,
    /// The length of the whole file.
    pub total: u64,
    /// The `Content-Range` value for these bytes.
    pub content_range: String,
}

/// The `Content-Range` value for `len` (at least one) bytes from `start` of a file of `total` bytes.
#[must_use]
pub fn content_range_text(start: u64, len: u64, total: u64) -> String {
    format!("bytes {start}-{}/{total}", start + len.max(1) - 1)
}

/// What an adapter that holds a range in memory sends for a 206 `reply`: the planned range cut to [`MAX_RANGE_BODY`]
/// and its `Content-Range`. `None` for anything that is not a partial file reply.
#[must_use]
pub fn range_body(reply: &Reply) -> Option<RangeBody> {
    if reply.status != 206 {
        return None;
    }
    let Content::File { start, len, .. } = &reply.content else {
        return None;
    };
    let total: u64 = reply
        .headers
        .iter()
        .find(|(name, _)| *name == "Content-Range")
        .and_then(|(_, value)| value.rsplit_once('/'))
        .and_then(|(_, total)| total.parse().ok())?;
    let len = (*len).min(MAX_RANGE_BODY);
    if len == 0 {
        return None;
    }
    Some(RangeBody {
        offset: *start,
        len,
        total,
        content_range: content_range_text(*start, len, total),
    })
}

/// Reads up to `len` bytes of `path` from `offset`: fewer when the file ended (or shrank) in between.
///
/// # Errors
///
/// The file cannot be opened or read.
pub fn read_range(path: &Path, offset: u64, len: u64) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = Vec::new();
    file.take(len).read_to_end(&mut bytes)?;
    Ok(bytes)
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
                "sucrose-serve-{name}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("create scratch folder");
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_theme_file_is_served_with_its_type_and_the_legacy_cors_headers() {
        let theme = Scratch::new("theme");
        fs::write(theme.0.join("index.html"), "<html>").expect("write");

        let reply = plan("https://sucrose.wallpaper.engine/index.html?x=1", None, &[&theme.0]);

        assert_eq!((reply.status, reply.mime), (200, "text/html"));
        assert!(matches!(reply.content, Content::File { start: 0, len: 6, .. }));
        assert!(reply.headers.contains(&("Accept-Ranges", "bytes".to_owned())));
        assert!(reply.headers.contains(&("Access-Control-Allow-Origin", "*".to_owned())));
    }

    #[test]
    fn a_traversal_a_missing_file_and_a_foreign_address_are_all_404() {
        let theme = Scratch::new("theme");

        for url in [
            "https://sucrose.wallpaper.engine/%252e%252e/x",
            "https://sucrose.wallpaper.engine/missing.html",
            "https://sucrose.wallpaper.engine/",
            "https://example.com/index.html",
        ] {
            let reply = plan(url, None, &[&theme.0]);
            assert_eq!(reply.status, 404, "{url}");
            assert_eq!(reply.content, Content::Text(NOT_FOUND_BODY), "{url}");
        }
    }

    #[test]
    fn a_single_range_is_a_206_with_content_range() {
        let theme = Scratch::new("range");
        fs::write(theme.0.join("clip.mp4"), vec![7u8; 1000]).expect("write");
        let url = "https://sucrose.wallpaper.engine/clip.mp4";

        let reply = plan(url, Some("bytes=100-199"), &[&theme.0]);

        assert_eq!((reply.status, reply.status_text), (206, "Partial Content"));
        assert!(matches!(
            reply.content,
            Content::File {
                start: 100,
                len: 100,
                ..
            }
        ));
        assert!(
            reply
                .headers
                .contains(&("Content-Range", "bytes 100-199/1000".to_owned()))
        );
        assert!(reply.headers.contains(&("Accept-Ranges", "bytes".to_owned())));

        let tail = plan(url, Some("bytes=900-"), &[&theme.0]);
        assert!(matches!(
            tail.content,
            Content::File {
                start: 900,
                len: 100,
                ..
            }
        ));
        assert!(
            tail.headers
                .contains(&("Content-Range", "bytes 900-999/1000".to_owned()))
        );
    }

    #[test]
    fn a_range_past_the_end_is_a_416_and_several_ranges_are_the_whole_file() {
        let theme = Scratch::new("range416");
        fs::write(theme.0.join("clip.mp4"), vec![7u8; 1000]).expect("write");
        let url = "https://sucrose.wallpaper.engine/clip.mp4";

        let unsatisfiable = plan(url, Some("bytes=1000-"), &[&theme.0]);
        assert_eq!(unsatisfiable.status, 416);
        assert_eq!(unsatisfiable.content, Content::Text(""));
        assert!(
            unsatisfiable
                .headers
                .contains(&("Content-Range", "bytes */1000".to_owned()))
        );

        let several = plan(url, Some("bytes=0-9,20-29"), &[&theme.0]);
        assert_eq!(several.status, 200);
        assert!(matches!(
            several.content,
            Content::File {
                start: 0,
                len: 1000,
                ..
            }
        ));
    }

    /// A file of `len` zero bytes (sparse where the file system allows it).
    fn sized_file(dir: &Path, name: &str, len: u64) -> PathBuf {
        let path = dir.join(name);
        let file = fs::File::create(&path).expect("create");
        file.set_len(len).expect("set length");
        path
    }

    #[test]
    fn an_open_range_of_a_long_file_is_cut_to_the_cap_with_a_content_range_that_names_the_cut() {
        let theme = Scratch::new("cap");
        sized_file(&theme.0, "long.mp4", 10 * 1024 * 1024);
        let url = "https://sucrose.wallpaper.engine/long.mp4";

        let from_start = range_body(&plan(url, Some("bytes=0-"), &[&theme.0])).expect("a partial reply");
        assert_eq!(from_start.offset, 0);
        assert_eq!(from_start.len, MAX_RANGE_BODY);
        assert_eq!(from_start.total, 10_485_760);
        assert_eq!(from_start.content_range, "bytes 0-4194303/10485760");

        let from_middle = range_body(&plan(url, Some("bytes=5242880-"), &[&theme.0])).expect("a partial reply");
        assert_eq!((from_middle.offset, from_middle.len), (5_242_880, MAX_RANGE_BODY));
        assert_eq!(from_middle.content_range, "bytes 5242880-9437183/10485760");

        let tail = range_body(&plan(url, Some("bytes=-100"), &[&theme.0])).expect("a partial reply");
        assert_eq!((tail.offset, tail.len), (10_485_660, 100));
        assert_eq!(tail.content_range, "bytes 10485660-10485759/10485760");
    }

    #[test]
    fn a_range_under_the_cap_keeps_its_bytes() {
        let theme = Scratch::new("small");
        sized_file(&theme.0, "clip.mp4", 12_701);

        let reply = plan(
            "https://sucrose.wallpaper.engine/clip.mp4",
            Some("bytes=0-9"),
            &[&theme.0],
        );
        let body = range_body(&reply).expect("a partial reply");

        assert_eq!((body.offset, body.len, body.total), (0, 10, 12_701));
        assert_eq!(body.content_range, "bytes 0-9/12701");
    }

    #[test]
    fn only_a_partial_file_reply_has_a_range_body() {
        let theme = Scratch::new("none");
        sized_file(&theme.0, "clip.mp4", 1000);
        let url = "https://sucrose.wallpaper.engine/clip.mp4";

        assert_eq!(range_body(&plan(url, None, &[&theme.0])), None, "a whole file");
        assert_eq!(range_body(&plan(url, Some("bytes=1000-"), &[&theme.0])), None, "a 416");
        assert_eq!(
            range_body(&plan(url, Some("bytes=0-1,5-6"), &[&theme.0])),
            None,
            "several ranges"
        );
        assert_eq!(range_body(&Reply::not_found()), None, "a refusal");
    }

    #[test]
    fn content_range_text_names_the_inclusive_end() {
        assert_eq!(content_range_text(0, 1, 1), "bytes 0-0/1");
        assert_eq!(content_range_text(100, 100, 1000), "bytes 100-199/1000");
    }

    #[test]
    fn read_range_reads_exactly_the_bytes_asked_and_fewer_at_the_end() {
        let theme = Scratch::new("read");
        let data: Vec<u8> = (0..=255).collect();
        fs::write(theme.0.join("data.bin"), &data).expect("write");
        let path = theme.0.join("data.bin");

        assert_eq!(read_range(&path, 10, 5).expect("read"), &data[10..15]);
        assert_eq!(read_range(&path, 250, 100).expect("read"), &data[250..]);
        assert_eq!(read_range(&path, 256, 10).expect("read"), Vec::<u8>::new());
        assert!(read_range(&theme.0.join("missing.bin"), 0, 1).is_err());
    }

    #[test]
    fn a_long_path_is_cut_in_the_log() {
        assert_eq!(shown(&"a".repeat(5000)).len(), LOGGED_PATH_CHARS);
        assert_eq!(shown("/short"), "/short");
    }
}
