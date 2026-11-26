//! CEF's side of the virtual origin (D-14): a scheme handler for `https://sucrose.wallpaper.engine/` and nothing else.
//!
//! The decisions live in [`sucrose_engine_web::serve::plan`], the planner both web engines share: which file a request
//! gets, with which status and headers, and what is logged. The CEF part below only streams what the plan says. Every
//! request gets its own handler and its own open file, so requests from several renderers and subresources run side
//! by side and share nothing.

#[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
mod imp {
    use std::{
        fs::File,
        io::{Read, Seek, SeekFrom},
        path::{Path, PathBuf},
        sync::{Arc, Mutex, PoisonError},
    };

    use cef::*;

    use sucrose_engine_web::serve::{Content, Reply, plan};

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
    fn open_stream(url: &str, range: Option<&str>, roots: &[PathBuf]) -> Stream {
        let roots: Vec<&Path> = roots.iter().map(PathBuf::as_path).collect();
        let reply = plan(url, range, &roots);
        match &reply.content {
            Content::Text(_) => Stream::Text { reply, sent: 0 },
            Content::File { path, start, len } => match File::open(path).and_then(|mut file| {
                file.seek(SeekFrom::Start(*start))?;
                Ok(file)
            }) {
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
                let (url, range) = request.map_or_else(Default::default, |request| {
                    let range = CefString::from(&request.header_by_name(Some(&CefString::from("Range")))).to_string();
                    (
                        CefString::from(&request.url()).to_string(),
                        Some(range).filter(|range| !range.is_empty()),
                    )
                });
                *self.stream.lock().unwrap_or_else(PoisonError::into_inner) =
                    open_stream(&url, range.as_deref(), &self.roots);
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
