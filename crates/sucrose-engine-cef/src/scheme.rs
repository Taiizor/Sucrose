//! CEF's side of the virtual origin (D-14): a scheme handler for `https://sucrose.wallpaper.engine/` and nothing else.
//!
//! The decisions live in [`plan`], which is plain Rust over `sucrose_engine_web::origin::resolve`: which file a request
//! gets, with which status and headers, and what is logged. The CEF part below only streams what the plan says. Every
//! request gets its own handler and its own open file, so requests from several renderers and subresources run side
//! by side and share nothing.

use std::path::{Path, PathBuf};

use sucrose_engine_web::origin;

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
    /// `len` bytes of a file.
    File {
        /// The canonical path.
        path: PathBuf,
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
    fn not_found() -> Self {
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

/// Decides the answer to a request for `url` over the `roots` (the theme folder, then the generated-content folder),
/// and logs it: a served file at debug, a traversal-class refusal at info, a file that is just not there at debug.
#[must_use]
pub fn plan(url: &str, roots: &[&Path]) -> Reply {
    let Some(path) = origin::url_path(url) else {
        tracing::info!(url = %shown(url), "origin request rejected: not an address of the virtual host");
        return Reply::not_found();
    };
    match origin::resolve(path, roots) {
        Ok(file) => {
            tracing::debug!(path = %shown(path), root = file.root, status = 200, "origin request served");
            Reply {
                status: 200,
                status_text: "OK",
                mime: file.mime,
                headers: cors(),
                content: Content::File {
                    path: file.path,
                    len: file.len,
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

#[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
mod imp {
    use std::{
        fs::File,
        io::Read,
        path::{Path, PathBuf},
        sync::{Arc, Mutex, PoisonError},
    };

    use cef::*;

    use super::{Content, Reply, plan};

    /// What one handler streams after `open`.
    enum Stream {
        /// `open` has not run (or the request was cancelled).
        Idle,
        /// A text body, with the bytes sent so far.
        Text { reply: Reply, sent: usize },
        /// A file, with the bytes still to send.
        File { reply: Reply, file: File, remaining: u64 },
    }

    impl Stream {
        fn reply(&self) -> Option<&Reply> {
            match self {
                Self::Idle => None,
                Self::Text { reply, .. } | Self::File { reply, .. } => Some(reply),
            }
        }
    }

    /// Plans the request and opens its file; a file that cannot be opened is a 404.
    fn open_stream(url: &str, roots: &[PathBuf]) -> Stream {
        let roots: Vec<&Path> = roots.iter().map(PathBuf::as_path).collect();
        let reply = plan(url, &roots);
        match &reply.content {
            Content::Text(_) => Stream::Text { reply, sent: 0 },
            Content::File { path, len } => match File::open(path) {
                Ok(file) => {
                    let remaining = *len;
                    Stream::File { reply, file, remaining }
                }
                Err(error) => {
                    tracing::info!(%error, "origin request rejected: the file cannot be opened");
                    Stream::Text {
                        reply: Reply::not_found(),
                        sent: 0,
                    }
                }
            },
        }
    }

    wrap_resource_handler! {
        struct OriginHandler {
            roots: Arc<Vec<PathBuf>>,
            stream: Arc<Mutex<Stream>>,
        }

        impl ResourceHandler {
            fn open(
                &self,
                request: Option<&mut Request>,
                handle_request: Option<&mut ::std::os::raw::c_int>,
                _callback: Option<&mut Callback>,
            ) -> ::std::os::raw::c_int {
                let url = request
                    .map(|request| CefString::from(&request.url()).to_string())
                    .unwrap_or_default();
                *self.stream.lock().unwrap_or_else(PoisonError::into_inner) = open_stream(&url, &self.roots);
                // Answer now: opening a file does not wait for anything.
                if let Some(handle_request) = handle_request {
                    *handle_request = 1;
                }
                1
            }

            fn response_headers(
                &self,
                response: Option<&mut Response>,
                response_length: Option<&mut i64>,
                _redirect_url: Option<&mut CefString>,
            ) {
                let stream = self.stream.lock().unwrap_or_else(PoisonError::into_inner);
                let (Some(response), Some(reply)) = (response, stream.reply()) else {
                    return;
                };
                response.set_status(i32::from(reply.status));
                response.set_status_text(Some(&CefString::from(reply.status_text)));
                response.set_mime_type(Some(&CefString::from(reply.mime)));
                for (name, value) in &reply.headers {
                    response.set_header_by_name(
                        Some(&CefString::from(*name)),
                        Some(&CefString::from(value.as_str())),
                        1,
                    );
                }
                if let Some(response_length) = response_length {
                    *response_length = match &reply.content {
                        Content::Text(text) => i64::try_from(text.len()).unwrap_or(-1),
                        Content::File { len, .. } => i64::try_from(*len).unwrap_or(-1),
                    };
                }
            }

            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            fn read(
                &self,
                data_out: *mut u8,
                bytes_to_read: ::std::os::raw::c_int,
                bytes_read: Option<&mut ::std::os::raw::c_int>,
                _callback: Option<&mut ResourceReadCallback>,
            ) -> ::std::os::raw::c_int {
                let Some(bytes_read) = bytes_read else {
                    return 0;
                };
                *bytes_read = 0;
                let Ok(capacity) = usize::try_from(bytes_to_read) else {
                    return 0;
                };
                if capacity == 0 || data_out.is_null() {
                    return 0;
                }
                // SAFETY: CEF hands over a buffer of `bytes_to_read` bytes that is valid and exclusively ours for
                // the duration of this call, and `capacity` is that count.
                let buffer = unsafe { std::slice::from_raw_parts_mut(data_out, capacity) };
                let mut stream = self.stream.lock().unwrap_or_else(PoisonError::into_inner);
                let count = match &mut *stream {
                    Stream::Idle => 0,
                    Stream::Text { reply, sent } => {
                        let Content::Text(text) = &reply.content else {
                            return 0;
                        };
                        let rest = &text.as_bytes()[(*sent).min(text.len())..];
                        let count = rest.len().min(buffer.len());
                        buffer[..count].copy_from_slice(&rest[..count]);
                        *sent += count;
                        count
                    }
                    Stream::File { file, remaining, .. } => {
                        let want = usize::try_from(*remaining).unwrap_or(usize::MAX).min(buffer.len());
                        match file.read(&mut buffer[..want]) {
                            Ok(count) => {
                                *remaining -= count as u64;
                                count
                            }
                            Err(error) => {
                                tracing::warn!(%error, "reading a served file failed");
                                0
                            }
                        }
                    }
                };
                if count == 0 {
                    return 0;
                }
                // The count is at most `bytes_to_read`, an i32.
                *bytes_read = i32::try_from(count).unwrap_or(0);
                1
            }

            fn cancel(&self) {
                *self.stream.lock().unwrap_or_else(PoisonError::into_inner) = Stream::Idle;
            }
        }
    }

    wrap_scheme_handler_factory! {
        struct OriginFactory {
            roots: Arc<Vec<PathBuf>>,
        }

        impl SchemeHandlerFactory {
            fn create(
                &self,
                _browser: Option<&mut Browser>,
                _frame: Option<&mut Frame>,
                _scheme_name: Option<&CefString>,
                _request: Option<&mut Request>,
            ) -> Option<ResourceHandler> {
                Some(OriginHandler::new(
                    Arc::clone(&self.roots),
                    Arc::new(Mutex::new(Stream::Idle)),
                ))
            }
        }
    }

    /// Registers the factory for `https` on the host `sucrose.wallpaper.engine` only, so every other https request
    /// keeps CEF's own networking and certificate checks. Call it after CEF is initialised and before the first
    /// navigation.
    pub fn register(roots: Vec<PathBuf>) -> bool {
        let mut factory = OriginFactory::new(Arc::new(roots));
        register_scheme_handler_factory(
            Some(&CefString::from("https")),
            Some(&CefString::from(sucrose_engine_web::origin::HOST)),
            Some(&mut factory),
        ) == 1
    }
}

/// Targets without a CEF build have no browser to register with.
#[cfg(not(all(windows, any(target_arch = "x86_64", target_arch = "aarch64"))))]
mod imp {
    use std::path::PathBuf;

    /// Nothing to register on this target.
    pub fn register(_roots: Vec<PathBuf>) -> bool {
        true
    }
}

pub use imp::register;

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
                "sucrose-scheme-{name}-{}-{}",
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

        let reply = plan("https://sucrose.wallpaper.engine/index.html?x=1", &[&theme.0]);

        assert_eq!((reply.status, reply.mime), (200, "text/html"));
        assert!(matches!(reply.content, Content::File { len: 6, .. }));
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
            let reply = plan(url, &[&theme.0]);
            assert_eq!(reply.status, 404, "{url}");
            assert_eq!(reply.content, Content::Text(NOT_FOUND_BODY), "{url}");
        }
    }

    #[test]
    fn a_long_path_is_cut_in_the_log() {
        assert_eq!(shown(&"a".repeat(5000)).len(), LOGGED_PATH_CHARS);
        assert_eq!(shown("/short"), "/short");
    }
}
