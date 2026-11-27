//! CEF's side of the virtual origin (D-14): a scheme handler for `https://sucrose.wallpaper.engine/` and nothing else.
//!
//! The decisions live in [`sucrose_engine_web::serve::plan`], the planner both web engines share: which file a request
//! gets, with which status and headers, and what is logged. The CEF part below only streams what the plan says. Every
//! request gets its own handler and its own open file, so requests from several renderers and subresources run side
//! by side and share nothing.
//!
//! CEF's loader reads the `Range` header of a request itself: before it reads a range that starts past byte 0 it asks
//! the handler to skip to that byte, and fails the request when the handler cannot. The planner has already opened the
//! file at the range start, so the skip only confirms the position (or moves forward when the stream starts earlier).

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

    /// The net error a failed skip reports (`ERR_REQUEST_RANGE_NOT_SATISFIABLE`), as CEF's own loader does.
    const RANGE_NOT_SATISFIABLE: i64 = cef::sys::cef_errorcode_t::ERR_REQUEST_RANGE_NOT_SATISFIABLE as i64;

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

        /// The bytes still to send.
        fn length(&self) -> u64 {
            match self {
                Self::Idle => 0,
                Self::Text { reply, sent } => match &reply.content {
                    Content::Text(text) => text.len().saturating_sub(*sent) as u64,
                    Content::File { .. } => 0,
                },
                Self::File { remaining, .. } => *remaining,
            }
        }

        /// Moves the stream to byte `target` of the served file, as CEF's loader asks before it reads a range that starts
        /// there. A file opened at the range start is already there; one that starts earlier (a reply the planner made
        /// for the whole file) is moved forward. A text reply (a 404, a 416) is not a part of the file, so there is
        /// nothing to move and it is sent as it is. `false` for a target behind the stream or past its end, and for a
        /// file that cannot move.
        fn skip_to(&mut self, target: u64) -> bool {
            match self {
                Self::Idle => false,
                Self::Text { .. } => true,
                Self::File { reply, file, remaining } => {
                    let Content::File { start, len, .. } = &reply.content else {
                        return false;
                    };
                    let position = start + (len - *remaining);
                    let Some(ahead) = target.checked_sub(position).filter(|ahead| *ahead <= *remaining) else {
                        return false;
                    };
                    if ahead > 0 {
                        let Ok(offset) = i64::try_from(ahead) else {
                            return false;
                        };
                        if let Err(error) = file.seek(SeekFrom::Current(offset)) {
                            tracing::debug!(%error, "skipping in a served file failed");
                            return false;
                        }
                        *remaining -= ahead;
                    }
                    true
                }
            }
        }

        /// Copies the next bytes into `buffer` and returns how many; 0 at the end.
        fn read_into(&mut self, buffer: &mut [u8]) -> usize {
            match self {
                Self::Idle => 0,
                Self::Text { reply, sent } => {
                    let Content::Text(text) = &reply.content else {
                        return 0;
                    };
                    let rest = &text.as_bytes()[(*sent).min(text.len())..];
                    let count = rest.len().min(buffer.len());
                    buffer[..count].copy_from_slice(&rest[..count]);
                    *sent += count;
                    count
                }
                Self::File { file, remaining, .. } => {
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
                // CEF asks for the headers after the skip, so the length is what is left to send.
                if let Some(response_length) = response_length {
                    *response_length = match stream.reply() {
                        Some(_) => i64::try_from(stream.length()).unwrap_or(-1),
                        None => -1,
                    };
                }
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
            }

            // CEF's loader calls this with the first byte of a `Range` that starts past 0, before any read; without it
            // every such request fails (a video whose index is at the end of its file never plays, and no seek works).
            fn skip(
                &self,
                bytes_to_skip: i64,
                bytes_skipped: Option<&mut i64>,
                _callback: Option<&mut ResourceSkipCallback>,
            ) -> ::std::os::raw::c_int {
                let Some(bytes_skipped) = bytes_skipped else {
                    return 0;
                };
                let moved = u64::try_from(bytes_to_skip)
                    .ok()
                    .filter(|target| *target > 0)
                    .is_some_and(|target| self.stream.lock().unwrap_or_else(PoisonError::into_inner).skip_to(target));
                if moved {
                    // Done at once: CEF goes on to the headers and reads from there.
                    *bytes_skipped = bytes_to_skip;
                    1
                } else {
                    tracing::debug!(bytes_to_skip, "origin request cannot skip to the asked byte");
                    *bytes_skipped = RANGE_NOT_SATISFIABLE;
                    0
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
                let count = self
                    .stream
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .read_into(buffer);
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

    #[cfg(test)]
    mod tests {
        use std::{
            fs,
            sync::atomic::{AtomicU32, Ordering},
        };

        use super::*;

        const CLIP: &str = "https://sucrose.wallpaper.engine/clip.mp4";

        /// A theme folder under the temp folder with a 1000-byte `clip.mp4`, removed on drop.
        struct Theme {
            dir: PathBuf,
            data: Vec<u8>,
        }

        impl Theme {
            fn new(name: &str) -> Self {
                static NEXT: AtomicU32 = AtomicU32::new(0);
                let dir = std::env::temp_dir().join(format!(
                    "sucrose-cef-scheme-{name}-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::SeqCst)
                ));
                let _ = fs::remove_dir_all(&dir);
                fs::create_dir_all(&dir).expect("create the theme folder");
                let data: Vec<u8> = (0..=255u8).cycle().take(1000).collect();
                fs::write(dir.join("clip.mp4"), &data).expect("write the clip");
                Self { dir, data }
            }

            fn open(&self, url: &str, range: Option<&str>) -> Stream {
                open_stream(url, range, std::slice::from_ref(&self.dir))
            }
        }

        impl Drop for Theme {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.dir);
            }
        }

        fn read_all(stream: &mut Stream) -> Vec<u8> {
            let mut bytes = Vec::new();
            let mut buffer = [0u8; 64];
            loop {
                let count = stream.read_into(&mut buffer);
                if count == 0 {
                    return bytes;
                }
                bytes.extend_from_slice(&buffer[..count]);
            }
        }

        #[test]
        fn a_range_past_zero_is_already_at_its_start_when_cef_skips_to_it() {
            let theme = Theme::new("past-zero");
            let mut stream = theme.open(CLIP, Some("bytes=100-199"));

            assert!(stream.skip_to(100));
            assert_eq!(stream.length(), 100);
            assert_eq!(read_all(&mut stream), &theme.data[100..200]);
        }

        #[test]
        fn a_whole_file_reply_moves_forward_to_the_skip_target() {
            let theme = Theme::new("forward");
            let mut stream = theme.open(CLIP, None);

            assert!(stream.skip_to(10));
            assert_eq!(stream.length(), 990, "the length CEF is told is what is left");
            assert_eq!(read_all(&mut stream), &theme.data[10..]);
        }

        #[test]
        fn a_skip_behind_the_stream_or_past_its_end_fails() {
            let theme = Theme::new("outside");
            let mut stream = theme.open(CLIP, Some("bytes=100-199"));

            assert!(!stream.skip_to(50), "the stream cannot go back");
            assert!(!stream.skip_to(201), "nor past the end of the range");
            assert!(stream.skip_to(200), "the end itself is reachable");
            assert_eq!(stream.length(), 0);
            assert!(!Stream::Idle.skip_to(1), "a cancelled request skips nothing");
        }

        #[test]
        fn a_refusal_is_sent_whole_after_a_skip() {
            let theme = Theme::new("refusal");

            let mut past_end = theme.open(CLIP, Some("bytes=1000-"));
            assert_eq!(past_end.reply().map(|reply| reply.status), Some(416));
            assert!(past_end.skip_to(1000), "CEF goes on to the 416 headers");
            assert_eq!(past_end.length(), 0);

            let mut missing = theme.open("https://sucrose.wallpaper.engine/missing.mp4", Some("bytes=10-"));
            assert_eq!(missing.reply().map(|reply| reply.status), Some(404));
            assert!(missing.skip_to(10));
            assert_eq!(read_all(&mut missing), b"File not found.");
        }

        // Called through the handler's C function table, as CEF's loader calls it; no CEF code runs.
        #[test]
        fn the_handler_answers_cef_skip_through_its_function_table() {
            let theme = Theme::new("handler");
            let stream = theme.open(CLIP, Some("bytes=500-"));
            let handler = OriginHandler::new(Arc::new(vec![theme.dir.clone()]), Arc::new(Mutex::new(stream)));

            let mut skipped = 0i64;
            assert_eq!(handler.skip(500, Some(&mut skipped), None), 1);
            assert_eq!(skipped, 500, "done at once, so CEF reads from byte 500");

            let mut length = 0i64;
            handler.response_headers(None, Some(&mut length), None);
            assert_eq!(length, 500);

            let mut buffer = vec![0u8; 1000];
            let mut read = 0;
            assert_eq!(handler.read(buffer.as_mut_ptr(), 1000, Some(&mut read), None), 1);
            assert_eq!(&buffer[..usize::try_from(read).expect("a count")], &theme.data[500..]);

            let mut failed = 0i64;
            assert_eq!(handler.skip(100, Some(&mut failed), None), 0);
            assert_eq!(failed, RANGE_NOT_SATISFIABLE, "the error CEF's own loader reports");
        }
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
