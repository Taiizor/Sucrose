//! The media fields of the Audio group: the legacy `GetAudioInfo` keys that come from the current media session (title,
//! artist, album, track numbers, media type, playback state, rate, shuffle, repeat, the timeline times in milliseconds,
//! the source app) and the album art as the legacy `ThumbnailString` (WAPI-06).
//!
//! The legacy source was the Playhead package, which wraps the Windows now-playing session. The documented Windows API for
//! the same data is `GlobalSystemMediaTransportControlsSessionManager` (GSMTC), and that is what this module reads. The
//! session exists only while a running page asks for the Audio group (D-24, D-008): the data hub starts it with the
//! capture and drops it with the last such page. What the user listens to is never written to disk or logged beyond
//! counts and sizes (T-04-64).
//!
//! Keys GSMTC cannot give keep their legacy default: `PropsValid` (a Playhead capability flag), `LastPlayingFileTime` (the
//! time the file last played, which a session does not report). The enum-valued keys carry the GSMTC member names; the
//! Playhead enums are not documented, so their member names beyond the shared `Unknown` default cannot be compared.

use std::{
    sync::{Arc, Mutex, PoisonError, mpsc},
    thread::JoinHandle,
};

use serde_json::{Map, Value, json};

use crate::SysInfoError;

/// How long the session is asked again when no change event arrived: the legacy "session init re-checked every 2 s".
#[cfg_attr(not(windows), allow(dead_code))]
const RECHECK: std::time::Duration = std::time::Duration::from_secs(2);
/// The windows build from which the legacy code treats the system as "Cobalt" (Windows 11), and then never crops.
#[cfg_attr(not(windows), allow(dead_code))]
const COBALT_BUILD: u32 = 22000;
/// The crop of the legacy thumbnail: left, top, width, height.
const CROP: (u32, u32, u32, u32) = (34, 1, 233, 233);
/// The largest album art that is read: bigger is ignored, so a misbehaving player cannot make the service hold megabytes.
#[cfg_attr(not(windows), allow(dead_code))]
const MAX_ART: u64 = 8 * 1024 * 1024;
/// How long a track without art is asked again: players publish the art a moment after the title.
#[cfg_attr(not(windows), allow(dead_code))]
const ART_RETRY: std::time::Duration = std::time::Duration::from_secs(10);
/// The longest `ThumbnailString` the media session gives. The service sends the art to a renderer as one IPC blob, the data
/// hub checks at compile time that this is not above that blob cap, and art that would be longer is made smaller to fit.
pub const THUMBNAIL_TEXT_MAX: usize = 2 * 1024 * 1024;
/// The JPEG quality of opaque art that is made smaller.
const ART_JPEG_QUALITY: u8 = 90;
/// Art is never scaled down below this many pixels on its longer side.
const ART_MIN_SIDE: u32 = 64;
/// How many times art is encoded while it is made to fit.
const ART_ATTEMPTS: u32 = 8;
/// The most pixels a side of the art may have for it to be decoded.
const ART_DECODE_SIDE: u32 = 8192;
/// The most memory the decoder may allocate for one art.
const ART_DECODE_ALLOC: u64 = 256 * 1024 * 1024;

/// What the worker thread is told.
#[cfg_attr(not(windows), allow(dead_code))]
enum Msg {
    /// A session event arrived: read the session again.
    Refresh,
    /// The session is dropped: leave.
    Stop,
}

/// One reading of the current session, in the units of GSMTC (times in 100 ns ticks) and with the enums as their names.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) struct SessionRecord {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub subtitle: String,
    pub track_number: Option<u32>,
    pub track_count: Option<u32>,
    /// The GSMTC media type name: Unknown, Music, Video or Image.
    pub media_type: &'static str,
    /// The GSMTC playback status name: Closed, Opened, Changing, Stopped, Playing or Paused.
    pub playback_state: &'static str,
    pub rate: Option<f64>,
    pub shuffle: bool,
    /// The GSMTC repeat mode name: None, Track or List; Unknown when the player gives none.
    pub repeat: &'static str,
    pub start_ticks: i64,
    pub end_ticks: i64,
    pub position_ticks: i64,
    pub min_seek_ticks: i64,
    pub max_seek_ticks: i64,
    /// When the position was last set, in 100 ns ticks since 1601-01-01 UTC.
    pub position_set_filetime: Option<i64>,
    pub source_app_id: String,
}

const DEFAULT_TIME: &str = "0001-01-01T00:00:00";

/// A time span in ticks as the legacy `TimeSpan.TotalMilliseconds`.
fn milliseconds(ticks: i64) -> Value {
    json!(ticks as f64 / 10_000.0)
}

/// The text Newtonsoft writes for a UTC `DateTime` given as ticks since 1601-01-01: `yyyy-MM-ddTHH:mm:ss[.fffffff]Z`, the
/// fraction without trailing zeros. The legacy default (no time) is `0001-01-01T00:00:00`.
fn file_time_text(ticks: Option<i64>) -> String {
    let Some(ticks) = ticks.filter(|ticks| *ticks > 0) else {
        return DEFAULT_TIME.to_owned();
    };
    let seconds = ticks / 10_000_000;
    let fraction = ticks % 10_000_000;
    // Days between 1601-01-01 and 1970-01-01.
    let days = seconds / 86_400 - 134_774;
    let in_day = seconds % 86_400;
    // Civil date from a day count (Howard Hinnant's algorithm).
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    let mut text = format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}",
        in_day / 3600,
        in_day % 3600 / 60,
        in_day % 60
    );
    if fraction > 0 {
        text.push('.');
        text.push_str(format!("{fraction:07}").trim_end_matches('0'));
    }
    text.push('Z');
    text
}

/// The media keys of the Audio group, the keys in the order of `GetAudioInfo` in `Data.cs` (the spectrum and the thumbnail keys
/// are not here). Without a session they are the legacy defaults, with `State` false; with one `State` is true, as the legacy code
/// set it when it had a session.
pub(crate) fn map(record: Option<&SessionRecord>) -> Map<String, Value> {
    let mut map = Map::new();
    let mut put = |key: &str, value: Value| {
        map.insert(key.to_owned(), value);
    };
    match record {
        Some(record) => {
            put("State", json!(true));
            put("Title", json!(record.title));
            put("Artist", json!(record.artist));
            put("Subtitle", json!(record.subtitle));
            put("AlbumTitle", json!(record.album));
            put("AlbumArtist", json!(record.album_artist));
            put("SourceAppId", json!(record.source_app_id));
            put("TrackNumber", json!(record.track_number.unwrap_or(0)));
            put("MediaType", json!(record.media_type));
            put("PlaybackRate", json!(record.rate.unwrap_or(0.0)));
            put("PropsValid", json!("Capabilities"));
            put("RepeatMode", json!(record.repeat));
            put("ShuffleEnabled", json!(record.shuffle));
            put("PlaybackMode", json!(record.media_type));
            put("AlbumTrackCount", json!(record.track_count.unwrap_or(0)));
            put("PlaybackState", json!(record.playback_state));
            put("EndTime", milliseconds(record.end_ticks));
            put("Position", milliseconds(record.position_ticks));
            put("StartTime", milliseconds(record.start_ticks));
            put("LastPlayingFileTime", json!(DEFAULT_TIME));
            put(
                "PositionSetFileTime",
                json!(file_time_text(record.position_set_filetime)),
            );
            put("MaxSeekTime", milliseconds(record.max_seek_ticks));
            put("MinSeekTime", milliseconds(record.min_seek_ticks));
        }
        None => {
            put("State", json!(false));
            put("Title", json!(""));
            put("Artist", json!(""));
            put("Subtitle", json!(""));
            put("AlbumTitle", json!(""));
            put("AlbumArtist", json!(""));
            put("SourceAppId", json!(""));
            put("TrackNumber", json!(0));
            put("MediaType", json!("Unknown"));
            put("PlaybackRate", json!(0.0));
            put("PropsValid", json!("Capabilities"));
            put("RepeatMode", json!("Unknown"));
            put("ShuffleEnabled", json!(false));
            put("PlaybackMode", json!("Unknown"));
            put("AlbumTrackCount", json!(0));
            put("PlaybackState", json!("Unknown"));
            put("EndTime", json!(0.0));
            put("Position", json!(0.0));
            put("StartTime", json!(0.0));
            put("LastPlayingFileTime", json!(DEFAULT_TIME));
            put("PositionSetFileTime", json!(DEFAULT_TIME));
            put("MaxSeekTime", json!(0.0));
            put("MinSeekTime", json!(0.0));
        }
    }
    map
}

/// The media keys of the legacy default, which a page sees while there is no media session.
#[must_use]
pub fn media_unavailable() -> Map<String, Value> {
    map(None)
}

/// Standard base64 with padding, as .NET's `Convert.ToBase64String`.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let [a, b, c] = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let group = (u32::from(a) << 16) | (u32::from(b) << 8) | u32::from(c);
        for (index, shift) in [18_u32, 12, 6, 0].into_iter().enumerate() {
            if index <= chunk.len() {
                text.push(char::from(ALPHABET[((group >> shift) & 63) as usize]));
            } else {
                text.push('=');
            }
        }
    }
    text
}

/// The crop of the legacy `Thumbnail.CropImage`: the rectangle of `CROP` as a PNG of exactly that size, transparent where the
/// source does not reach (what GDI+ draws).
fn crop_png(source: &image::RgbaImage) -> Option<Vec<u8>> {
    let (left, top, width, height) = CROP;
    let mut canvas = image::RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            if x + left < source.width() && y + top < source.height() {
                canvas.put_pixel(x, y, *source.get_pixel(x + left, y + top));
            }
        }
    }
    let mut png = Vec::new();
    canvas
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .ok()?;
    Some(png)
}

/// Decodes untrusted art within `ART_DECODE_SIDE` and `ART_DECODE_ALLOC`; art beyond them, or that cannot be read, gives `None`.
fn decode_art(bytes: &[u8]) -> Option<image::DynamicImage> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(ART_DECODE_SIDE);
    limits.max_image_height = Some(ART_DECODE_SIDE);
    limits.max_alloc = Some(ART_DECODE_ALLOC);
    reader.limits(limits);
    reader.decode().ok()
}

/// The length of the base64 text of `bytes` encoded bytes.
fn text_len(bytes: usize) -> usize {
    bytes.div_ceil(3) * 4
}

/// Art made to fit: the encoded bytes, the sides and how many encodes it took.
struct Fitted {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    attempts: u32,
}

/// Encodes `image` as a JPEG of `ART_JPEG_QUALITY`.
fn encode_jpeg(image: &image::DynamicImage) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, ART_JPEG_QUALITY);
    image.to_rgb8().write_with_encoder(encoder).ok()?;
    Some(bytes)
}

/// Encodes `image` as a PNG that keeps its alpha, as small as the encoder can make it.
fn encode_png(image: &image::DynamicImage) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut bytes,
        image::codecs::png::CompressionType::Best,
        image::codecs::png::FilterType::Adaptive,
    );
    image.to_rgba8().write_with_encoder(encoder).ok()?;
    Some(bytes)
}

/// Re-encodes the art until its base64 text is at most `max_text` long: first at its own size, then with both sides scaled
/// down together (always from the decoded original, so the aspect ratio holds). `None` when the art cannot be decoded, or does
/// not fit within `ART_ATTEMPTS` encodes and `ART_MIN_SIDE`.
fn fit_art(payload: &[u8], max_text: usize) -> Option<Fitted> {
    use image::GenericImageView;

    let original = decode_art(payload)?;
    let (width, height) = original.dimensions();
    if width == 0 || height == 0 {
        return None;
    }
    let opaque = !original.color().has_alpha() || original.pixels().all(|(_, _, pixel)| pixel.0[3] == 255);
    // Opaque art becomes a JPEG; art with a transparent pixel stays a PNG so its alpha survives.
    let encode = if opaque { encode_jpeg } else { encode_png };
    let mut scale = 1.0_f64;
    let (mut new_width, mut new_height) = (width, height);
    let mut attempts = 0;
    loop {
        let bytes = if attempts == 0 {
            encode(&original)?
        } else {
            let resized = original.resize_exact(new_width, new_height, image::imageops::FilterType::CatmullRom);
            encode(&resized)?
        };
        attempts += 1;
        let length = text_len(bytes.len());
        if length <= max_text {
            return Some(Fitted {
                bytes,
                width: new_width,
                height: new_height,
                attempts,
            });
        }
        if attempts >= ART_ATTEMPTS {
            return None;
        }
        let factor = ((max_text as f64 / length as f64).sqrt() * 0.9).clamp(0.25, 0.75);
        scale *= factor;
        let side = |original: u32| ((f64::from(original) * scale).round() as u32).max(1);
        (new_width, new_height) = (side(width), side(height));
        if new_width.max(new_height) < ART_MIN_SIDE {
            return None;
        }
    }
}

/// The legacy `Thumbnail.Create`: the base64 of the art. Unless the system is Cobalt, art whose pixel (0, 0) is transparent is
/// cropped to 233 x 233 at (34, 1) and re-encoded as PNG first. Art that cannot be decoded goes out as it is. Art whose text
/// would be over `THUMBNAIL_TEXT_MAX` is made smaller to fit.
pub(crate) fn thumbnail_text(bytes: &[u8], cobalt: bool) -> String {
    thumbnail_text_within(bytes, cobalt, THUMBNAIL_TEXT_MAX)
}

/// `thumbnail_text` with its own text limit: the legacy text when it fits, the art made to fit when it does not, and an
/// empty text (with a warning) when it cannot be made to fit.
pub(crate) fn thumbnail_text_within(bytes: &[u8], cobalt: bool, max_text: usize) -> String {
    let mut cropped = None;
    if !cobalt && let Some(decoded) = decode_art(bytes) {
        let rgba = decoded.to_rgba8();
        if rgba.width() > 0 && rgba.height() > 0 && rgba.get_pixel(0, 0).0[3] == 0 {
            cropped = crop_png(&rgba);
        }
    }
    let payload = cropped.as_deref().unwrap_or(bytes);
    let length = text_len(payload.len());
    if length <= max_text {
        return base64(payload);
    }
    let started = std::time::Instant::now();
    match fit_art(payload, max_text) {
        Some(fitted) => {
            let text = base64(&fitted.bytes);
            tracing::debug!(
                text = length,
                fitted = text.len(),
                width = fitted.width,
                height = fitted.height,
                attempts = fitted.attempts,
                elapsed_ms = started.elapsed().as_millis() as u64,
                "the album art was made smaller to fit the renderer"
            );
            text
        }
        None => {
            tracing::warn!(
                size = length,
                "the album art cannot be made small enough for the renderer, the page gets none"
            );
            String::new()
        }
    }
}

/// What the worker and the reader share.
#[derive(Default)]
struct Shared {
    record: Option<SessionRecord>,
    /// The art text of the track that changed, until the reader takes it.
    thumbnail: Option<String>,
}

/// The current media session, followed by a worker thread of its own while it exists.
pub struct MediaSession {
    shared: Arc<Mutex<Shared>>,
    /// Told to stop when the session is dropped.
    stop: Option<mpsc::Sender<Msg>>,
    worker: Option<JoinHandle<()>>,
}

impl MediaSession {
    /// Asks Windows for the session manager and starts the worker that follows the current session.
    ///
    /// # Errors
    /// [`SysInfoError::Os`] when the session manager cannot be created, [`SysInfoError::Unsupported`] on an OS without a
    /// provider.
    pub fn start() -> Result<Self, SysInfoError> {
        let shared = Arc::new(Mutex::new(Shared::default()));
        let (stop, messages) = mpsc::channel::<Msg>();
        let (ready, started) = mpsc::sync_channel::<Result<(), SysInfoError>>(1);
        let worker = platform::spawn(Arc::clone(&shared), stop.clone(), messages, ready)?;
        match started.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(Ok(())) => Ok(Self {
                shared,
                stop: Some(stop),
                worker: Some(worker),
            }),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(_) => {
                let _ = stop.send(Msg::Stop);
                let _ = worker.join();
                Err(SysInfoError::Os {
                    call: "media session",
                    message: "the media session worker did not start".to_owned(),
                })
            }
        }
    }

    /// The media keys of the Audio group for the current session (the legacy defaults when there is none).
    #[must_use]
    pub fn fields(&self) -> Map<String, Value> {
        let shared = self.shared.lock().unwrap_or_else(PoisonError::into_inner);
        map(shared.record.as_ref())
    }

    /// The `ThumbnailString` text of the current track when the track changed since the last call (empty when the track has
    /// no art), and `None` otherwise. The art is read by the worker when the track changes, never on every call.
    #[must_use]
    pub fn thumbnail_if_changed(&self) -> Option<String> {
        self.shared
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .thumbnail
            .take()
    }
}

impl Drop for MediaSession {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(Msg::Stop);
        }
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            tracing::error!("the media session worker panicked");
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use std::{
        sync::{Arc, Mutex, mpsc},
        thread::JoinHandle,
    };

    use super::{Msg, Shared};
    use crate::SysInfoError;

    pub(super) fn spawn(
        _shared: Arc<Mutex<Shared>>,
        _wake: mpsc::Sender<Msg>,
        _messages: mpsc::Receiver<Msg>,
        _ready: mpsc::SyncSender<Result<(), SysInfoError>>,
    ) -> Result<JoinHandle<()>, SysInfoError> {
        Err(SysInfoError::Unsupported("the media session"))
    }
}

#[cfg(windows)]
mod platform {
    use std::{
        sync::{Arc, Mutex, PoisonError, mpsc},
        thread::{self, JoinHandle},
        time::Instant,
    };

    use windows::{
        Foundation::TypedEventHandler,
        Media::{
            Control::{
                GlobalSystemMediaTransportControlsSession as Session,
                GlobalSystemMediaTransportControlsSessionManager as Manager,
                GlobalSystemMediaTransportControlsSessionMediaProperties as Properties,
                GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
            },
            MediaPlaybackAutoRepeatMode, MediaPlaybackType,
        },
        Storage::Streams::DataReader,
        Wdk::System::SystemServices::RtlGetVersion,
        Win32::System::{
            Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
            SystemInformation::OSVERSIONINFOW,
        },
    };

    use super::{ART_RETRY, COBALT_BUILD, MAX_ART, Msg, RECHECK, SessionRecord, Shared, thumbnail_text};
    use crate::SysInfoError;

    fn os(call: &'static str) -> impl Fn(windows::core::Error) -> SysInfoError {
        move |error| SysInfoError::Os {
            call,
            message: error.to_string(),
        }
    }

    pub(super) fn spawn(
        shared: Arc<Mutex<Shared>>,
        wake: mpsc::Sender<Msg>,
        messages: mpsc::Receiver<Msg>,
        ready: mpsc::SyncSender<Result<(), SysInfoError>>,
    ) -> Result<JoinHandle<()>, SysInfoError> {
        thread::Builder::new()
            .name("media-session".to_owned())
            .spawn(move || run(&shared, &wake, &messages, &ready))
            .map_err(|error| SysInfoError::Os {
                call: "spawn the media session thread",
                message: error.to_string(),
            })
    }

    /// COM (and so WinRT) in a multithreaded apartment on the worker thread, left again when the thread ends.
    struct Com;

    impl Com {
        fn init() -> Result<Self, SysInfoError> {
            // SAFETY: called once on this thread before any other COM call; the matching CoUninitialize is in Drop.
            unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok() }.map_err(os("CoInitializeEx"))?;
            Ok(Self)
        }
    }

    impl Drop for Com {
        fn drop(&mut self) {
            // SAFETY: pairs the successful CoInitializeEx of Com::init on this thread; every WinRT object of the thread is
            // dropped before, because Com is the first local of run.
            unsafe { CoUninitialize() };
        }
    }

    /// The build number of this Windows, 0 when it does not answer. The legacy `IsCobalt` is "build 22000 or later".
    fn windows_build() -> u32 {
        let mut info = OSVERSIONINFOW {
            dwOSVersionInfoSize: u32::try_from(size_of::<OSVERSIONINFOW>()).unwrap_or(0),
            ..OSVERSIONINFOW::default()
        };
        // SAFETY: `info` is a live, writable OSVERSIONINFOW whose size field is its real size, which is all RtlGetVersion
        // requires; the call only fills the struct and keeps no pointer.
        let status = unsafe { RtlGetVersion(&mut info) };
        if status.is_ok() { info.dwBuildNumber } else { 0 }
    }

    /// The session being followed and the event registrations that go with it.
    struct Followed {
        app_id: String,
        session: Session,
        tokens: [i64; 3],
    }

    impl Followed {
        fn open(session: Session, app_id: String, wake: &mpsc::Sender<Msg>) -> windows::core::Result<Self> {
            let handler = |wake: &mpsc::Sender<Msg>| {
                let wake = wake.clone();
                move || {
                    let _ = wake.send(Msg::Refresh);
                }
            };
            let (a, b, c) = (handler(wake), handler(wake), handler(wake));
            let properties = session.MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| {
                a();
                Ok(())
            }))?;
            let playback = session.PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| {
                b();
                Ok(())
            }))?;
            let timeline = session.TimelinePropertiesChanged(&TypedEventHandler::new(move |_, _| {
                c();
                Ok(())
            }))?;
            Ok(Self {
                app_id,
                session,
                tokens: [properties, playback, timeline],
            })
        }
    }

    impl Drop for Followed {
        fn drop(&mut self) {
            // A session that is already gone refuses the removal, which is the state wanted.
            let _ = self.session.RemoveMediaPropertiesChanged(self.tokens[0]);
            let _ = self.session.RemovePlaybackInfoChanged(self.tokens[1]);
            let _ = self.session.RemoveTimelinePropertiesChanged(self.tokens[2]);
        }
    }

    /// What the worker remembers between readings: the track whose art was read and how that went.
    #[derive(Default)]
    struct Track {
        identity: Option<(String, String, String, String)>,
        art_found: bool,
        since: Option<Instant>,
    }

    fn status_name(status: Status) -> &'static str {
        match status {
            Status::Closed => "Closed",
            Status::Opened => "Opened",
            Status::Changing => "Changing",
            Status::Stopped => "Stopped",
            Status::Playing => "Playing",
            Status::Paused => "Paused",
            _ => "Unknown",
        }
    }

    fn type_name(kind: MediaPlaybackType) -> &'static str {
        match kind {
            MediaPlaybackType::Music => "Music",
            MediaPlaybackType::Video => "Video",
            MediaPlaybackType::Image => "Image",
            _ => "Unknown",
        }
    }

    fn repeat_name(mode: MediaPlaybackAutoRepeatMode) -> &'static str {
        match mode {
            MediaPlaybackAutoRepeatMode::None => "None",
            MediaPlaybackAutoRepeatMode::Track => "Track",
            MediaPlaybackAutoRepeatMode::List => "List",
            _ => "Unknown",
        }
    }

    /// Reads everything of the session the Audio group needs.
    fn read(session: &Session, app_id: &str) -> windows::core::Result<(SessionRecord, Properties)> {
        let properties = session.TryGetMediaPropertiesAsync()?.join()?;
        let playback = session.GetPlaybackInfo()?;
        let timeline = session.GetTimelineProperties()?;
        let text = |value: windows::core::Result<windows::core::HSTRING>| {
            value.map(|text| text.to_string()).unwrap_or_default()
        };
        let number = |value: windows::core::Result<i32>| value.ok().and_then(|number| u32::try_from(number).ok());
        let record = SessionRecord {
            title: text(properties.Title()),
            artist: text(properties.Artist()),
            album: text(properties.AlbumTitle()),
            album_artist: text(properties.AlbumArtist()),
            subtitle: text(properties.Subtitle()),
            track_number: number(properties.TrackNumber()),
            track_count: number(properties.AlbumTrackCount()),
            media_type: playback
                .PlaybackType()
                .and_then(|kind| kind.Value())
                .map_or("Unknown", type_name),
            playback_state: playback.PlaybackStatus().map_or("Unknown", status_name),
            rate: playback.PlaybackRate().and_then(|rate| rate.Value()).ok(),
            shuffle: playback
                .IsShuffleActive()
                .and_then(|shuffle| shuffle.Value())
                .unwrap_or(false),
            repeat: playback
                .AutoRepeatMode()
                .and_then(|mode| mode.Value())
                .map_or("Unknown", repeat_name),
            start_ticks: timeline.StartTime().map_or(0, |time| time.Duration),
            end_ticks: timeline.EndTime().map_or(0, |time| time.Duration),
            position_ticks: timeline.Position().map_or(0, |time| time.Duration),
            min_seek_ticks: timeline.MinSeekTime().map_or(0, |time| time.Duration),
            max_seek_ticks: timeline.MaxSeekTime().map_or(0, |time| time.Duration),
            position_set_filetime: timeline.LastUpdatedTime().ok().map(|time| time.UniversalTime),
            source_app_id: app_id.to_owned(),
        };
        Ok((record, properties))
    }

    /// The bytes of the album art of `properties`, when there is some and it is not absurdly large.
    fn read_art(properties: &Properties) -> Option<Vec<u8>> {
        let stream = properties.Thumbnail().ok()?.OpenReadAsync().ok()?.join().ok()?;
        let size = stream.Size().ok()?;
        if size == 0 || size > MAX_ART {
            return None;
        }
        let reader = DataReader::CreateDataReader(&stream).ok()?;
        reader.LoadAsync(u32::try_from(size).ok()?).ok()?.join().ok()?;
        let mut bytes = vec![0_u8; usize::try_from(size).ok()?];
        reader.ReadBytes(&mut bytes).ok()?;
        Some(bytes)
    }

    /// Looks at the current session once: follows a new session, updates the record and, when the track changed (or its
    /// art arrived late), the pending thumbnail.
    fn refresh(
        manager: &Manager,
        followed: &mut Option<Followed>,
        track: &mut Track,
        shared: &Mutex<Shared>,
        wake: &mpsc::Sender<Msg>,
        cobalt: bool,
    ) {
        let session = manager.GetCurrentSession().ok();
        let Some(session) = session else {
            *followed = None;
            let had_track = track.identity.take().is_some();
            *track = Track::default();
            let mut shared = shared.lock().unwrap_or_else(PoisonError::into_inner);
            shared.record = None;
            if had_track {
                shared.thumbnail = Some(String::new());
            }
            return;
        };
        let app_id = session
            .SourceAppUserModelId()
            .map(|id| id.to_string())
            .unwrap_or_default();
        if followed.as_ref().is_none_or(|followed| followed.app_id != app_id) {
            *followed = None;
            match Followed::open(session.clone(), app_id.clone(), wake) {
                Ok(opened) => *followed = Some(opened),
                Err(error) => {
                    tracing::debug!(%error, "the media session events cannot be followed, it is read every 2 s")
                }
            }
        }
        let (record, properties) = match read(&session, &app_id) {
            Ok(read) => read,
            Err(error) => {
                tracing::debug!(%error, "the media session cannot be read this time");
                return;
            }
        };
        let identity = (
            record.title.clone(),
            record.artist.clone(),
            record.album.clone(),
            app_id,
        );
        let mut thumbnail = None;
        if track.identity.as_ref() != Some(&identity) {
            track.identity = Some(identity);
            track.since = Some(Instant::now());
            let art = read_art(&properties);
            track.art_found = art.is_some();
            thumbnail = Some(art.map_or_else(String::new, |bytes| thumbnail_text(&bytes, cobalt)));
            tracing::debug!(
                art_bytes = thumbnail.as_ref().map_or(0, String::len),
                "the media track changed"
            );
        } else if !track.art_found && track.since.is_some_and(|since| since.elapsed() < ART_RETRY) {
            // A player publishes the art a moment after the title: ask again for a while.
            if let Some(bytes) = read_art(&properties) {
                track.art_found = true;
                thumbnail = Some(thumbnail_text(&bytes, cobalt));
            }
        }
        let mut shared = shared.lock().unwrap_or_else(PoisonError::into_inner);
        shared.record = Some(record);
        if thumbnail.is_some() {
            shared.thumbnail = thumbnail;
        }
    }

    /// The worker: asks for the manager, then follows the current session until told to stop.
    fn run(
        shared: &Mutex<Shared>,
        wake: &mpsc::Sender<Msg>,
        messages: &mpsc::Receiver<Msg>,
        ready: &mpsc::SyncSender<Result<(), SysInfoError>>,
    ) {
        // Declared first so it is dropped last: the WinRT objects below go before CoUninitialize.
        let _com = match Com::init() {
            Ok(com) => com,
            Err(error) => {
                let _ = ready.send(Err(error));
                return;
            }
        };
        let manager = match Manager::RequestAsync()
            .and_then(|operation| operation.join())
            .map_err(os("GlobalSystemMediaTransportControlsSessionManager::RequestAsync"))
        {
            Ok(manager) => manager,
            Err(error) => {
                let _ = ready.send(Err(error));
                return;
            }
        };
        let token = {
            let wake = wake.clone();
            manager.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
                let _ = wake.send(Msg::Refresh);
                Ok(())
            }))
        };
        let token = match token.map_err(os("CurrentSessionChanged")) {
            Ok(token) => token,
            Err(error) => {
                let _ = ready.send(Err(error));
                return;
            }
        };
        let cobalt = windows_build() >= COBALT_BUILD;
        // The start is reported after the first reading, so the first snapshot a page gets already shows what plays.
        let mut ready = Some(ready);
        let mut followed: Option<Followed> = None;
        let mut track = Track::default();
        loop {
            refresh(&manager, &mut followed, &mut track, shared, wake, cobalt);
            if let Some(ready) = ready.take() {
                let _ = ready.send(Ok(()));
            }
            match messages.recv_timeout(RECHECK) {
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) | Ok(Msg::Stop) => break,
                Ok(Msg::Refresh) => {
                    // A burst of events is one reading.
                    let mut stop = false;
                    while let Ok(message) = messages.try_recv() {
                        stop |= matches!(message, Msg::Stop);
                    }
                    if stop {
                        break;
                    }
                }
            }
        }
        drop(followed);
        let _ = manager.RemoveCurrentSessionChanged(token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYS: [&str; 23] = [
        "State",
        "Title",
        "Artist",
        "Subtitle",
        "AlbumTitle",
        "AlbumArtist",
        "SourceAppId",
        "TrackNumber",
        "MediaType",
        "PlaybackRate",
        "PropsValid",
        "RepeatMode",
        "ShuffleEnabled",
        "PlaybackMode",
        "AlbumTrackCount",
        "PlaybackState",
        "EndTime",
        "Position",
        "StartTime",
        "LastPlayingFileTime",
        "PositionSetFileTime",
        "MaxSeekTime",
        "MinSeekTime",
    ];

    fn record() -> SessionRecord {
        SessionRecord {
            title: "Song".to_owned(),
            artist: "Band".to_owned(),
            album: "Record".to_owned(),
            album_artist: "Various".to_owned(),
            subtitle: "Live".to_owned(),
            track_number: Some(3),
            track_count: Some(12),
            media_type: "Music",
            playback_state: "Playing",
            rate: Some(1.0),
            shuffle: true,
            repeat: "List",
            start_ticks: 0,
            end_ticks: 2_405_000_000,
            position_ticks: 125_000_000,
            min_seek_ticks: 10_000,
            max_seek_ticks: 2_405_000_000,
            position_set_filetime: Some(133_800_000_000_000_000),
            source_app_id: "Spotify.exe".to_owned(),
        }
    }

    fn keys(map: &Map<String, Value>) -> Vec<&str> {
        map.keys().map(String::as_str).collect()
    }

    #[test]
    fn the_media_keys_are_those_of_get_audio_info_in_that_order() {
        assert_eq!(keys(&map(None)), KEYS);
        assert_eq!(keys(&map(Some(&record()))), KEYS);
    }

    #[test]
    fn a_session_fills_the_legacy_keys_with_the_legacy_types_and_the_times_in_milliseconds() {
        let fields = map(Some(&record()));

        assert_eq!(fields["State"], true);
        assert_eq!(fields["Title"], "Song");
        assert_eq!(fields["Artist"], "Band");
        assert_eq!(fields["Subtitle"], "Live");
        assert_eq!(fields["AlbumTitle"], "Record");
        assert_eq!(fields["AlbumArtist"], "Various");
        assert_eq!(fields["SourceAppId"], "Spotify.exe");
        assert_eq!(fields["TrackNumber"], 3);
        assert_eq!(fields["AlbumTrackCount"], 12);
        assert_eq!(fields["MediaType"], "Music");
        assert_eq!(fields["PlaybackMode"], "Music");
        assert_eq!(fields["PlaybackState"], "Playing");
        assert_eq!(fields["RepeatMode"], "List");
        assert_eq!(fields["ShuffleEnabled"], true);
        assert_eq!(fields["PlaybackRate"].as_f64(), Some(1.0));
        assert_eq!(fields["EndTime"].as_f64(), Some(240_500.0));
        assert_eq!(fields["Position"].as_f64(), Some(12_500.0));
        assert_eq!(fields["StartTime"].as_f64(), Some(0.0));
        assert_eq!(fields["MinSeekTime"].as_f64(), Some(1.0));
        assert_eq!(fields["MaxSeekTime"].as_f64(), Some(240_500.0));
        assert_eq!(
            fields["PropsValid"], "Capabilities",
            "GSMTC has no equivalent: the legacy default stays"
        );
        assert_eq!(fields["LastPlayingFileTime"], DEFAULT_TIME);
    }

    #[test]
    fn a_player_that_gives_no_numbers_gets_the_legacy_zeros() {
        let mut record = record();
        record.track_number = None;
        record.track_count = None;
        record.rate = None;

        let fields = map(Some(&record));

        assert_eq!(fields["TrackNumber"], 0);
        assert_eq!(fields["AlbumTrackCount"], 0);
        assert_eq!(fields["PlaybackRate"].as_f64(), Some(0.0));
    }

    #[test]
    fn with_no_session_the_media_keys_are_the_legacy_defaults_and_state_is_false() {
        let fields = media_unavailable();

        assert_eq!(fields["State"], false);
        assert_eq!(fields["Title"], "");
        assert_eq!(fields["MediaType"], "Unknown");
        assert_eq!(fields["PlaybackState"], "Unknown");
        assert_eq!(fields["RepeatMode"], "Unknown");
        assert_eq!(fields["PropsValid"], "Capabilities");
        assert_eq!(fields["PositionSetFileTime"], DEFAULT_TIME);
        let text = serde_json::to_string(&fields).expect("JSON");
        assert!(
            text.contains("\"EndTime\":0.0"),
            "doubles keep their decimal point: {text}"
        );
    }

    #[test]
    fn a_file_time_is_written_as_newtonsoft_writes_a_utc_date_time() {
        // 2020-01-02T03:04:05.5Z is 132224078455000000 ticks since 1601.
        assert_eq!(file_time_text(Some(132_224_078_455_000_000)), "2020-01-02T03:04:05.5Z");
        assert_eq!(file_time_text(Some(132_224_078_450_000_000)), "2020-01-02T03:04:05Z");
        assert_eq!(file_time_text(None), DEFAULT_TIME);
        assert_eq!(file_time_text(Some(0)), DEFAULT_TIME);
    }

    #[test]
    fn base64_matches_the_standard_alphabet_with_padding() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    fn png_of(image: &image::RgbaImage) -> Vec<u8> {
        let mut png = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .expect("encode");
        png
    }

    /// A 300 x 300 picture: red everywhere, with pixel (0, 0) as given and a marker pixel at (34, 1).
    fn picture(corner_alpha: u8) -> image::RgbaImage {
        let mut image = image::RgbaImage::from_pixel(300, 300, image::Rgba([255, 0, 0, 255]));
        image.put_pixel(0, 0, image::Rgba([255, 0, 0, corner_alpha]));
        image.put_pixel(34, 1, image::Rgba([0, 0, 255, 255]));
        image
    }

    fn decode(text: &str) -> image::RgbaImage {
        image::load_from_memory(&unbase64(text)).expect("a PNG").to_rgba8()
    }

    /// The reverse of base64 for the test only.
    fn unbase64(text: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut bits = 0_u32;
        let mut count = 0;
        for character in text.bytes().filter(|byte| *byte != b'=') {
            let value = match character {
                b'A'..=b'Z' => character - b'A',
                b'a'..=b'z' => character - b'a' + 26,
                b'0'..=b'9' => character - b'0' + 52,
                b'+' => 62,
                _ => 63,
            };
            bits = (bits << 6) | u32::from(value);
            count += 6;
            if count >= 8 {
                count -= 8;
                bytes.push((bits >> count) as u8);
                bits &= (1 << count) - 1;
            }
        }
        bytes
    }

    /// A fixed-seed linear congruential generator: the same noise on every run, with no extra crate.
    fn noise(seed: &mut u64) -> u8 {
        *seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (*seed >> 56) as u8
    }

    /// An opaque picture of noise: it compresses badly, so it is a stand-in for large cover art.
    fn noisy(width: u32, height: u32) -> image::RgbaImage {
        let mut seed = 7;
        image::RgbaImage::from_fn(width, height, |_, _| {
            image::Rgba([noise(&mut seed), noise(&mut seed), noise(&mut seed), 255])
        })
    }

    #[test]
    fn a_3_mib_art_text_fits_the_media_session_limit_as_a_smaller_jpeg() {
        let png = png_of(&noisy(900, 900));
        assert!(base64(&png).len() > THUMBNAIL_TEXT_MAX);

        let text = thumbnail_text(&png, true);

        assert!(!text.is_empty() && text.len() <= THUMBNAIL_TEXT_MAX);
        let bytes = unbase64(&text);
        assert_eq!(image::guess_format(&bytes).expect("a format"), image::ImageFormat::Jpeg);
        let decoded = image::load_from_memory(&bytes).expect("a JPEG");
        assert_eq!(decoded.width(), decoded.height());
        assert!(decoded.width() <= 900);
    }

    #[test]
    fn oversized_opaque_art_is_reencoded_at_its_own_size_when_that_fits() {
        let mut seed = 11;
        let photo = image::RgbaImage::from_fn(512, 512, |x, y| {
            let wobble = |seed: &mut u64| i32::from(noise(seed) % 17) - 8;
            let channel = |base: u32, seed: &mut u64| (base as i32 + wobble(seed)).clamp(0, 255) as u8;
            image::Rgba([
                channel(x / 2, &mut seed),
                channel(y / 2, &mut seed),
                channel((x + y) / 4, &mut seed),
                255,
            ])
        });
        let png = png_of(&photo);
        let budget = base64(&png).len() / 2;

        let text = thumbnail_text_within(&png, true, budget);

        assert!(!text.is_empty() && text.len() <= budget);
        let bytes = unbase64(&text);
        assert_eq!(image::guess_format(&bytes).expect("a format"), image::ImageFormat::Jpeg);
        let decoded = image::load_from_memory(&bytes).expect("a JPEG");
        assert_eq!((decoded.width(), decoded.height()), (512, 512));
    }

    #[test]
    fn art_that_still_does_not_fit_is_downscaled_keeping_its_aspect_ratio() {
        let png = png_of(&noisy(600, 400));

        let text = thumbnail_text_within(&png, true, 40_000);

        assert!(!text.is_empty() && text.len() <= 40_000);
        let decoded = image::load_from_memory(&unbase64(&text)).expect("a picture");
        assert!(decoded.width() < 600);
        assert!(i64::from(decoded.width()) * 400 - i64::from(decoded.height()) * 600 <= 600);
        assert!(i64::from(decoded.height()) * 600 - i64::from(decoded.width()) * 400 <= 600);
        assert!(decoded.width().max(decoded.height()) >= ART_MIN_SIDE);
    }

    #[test]
    fn transparent_art_stays_a_png_with_its_alpha_when_it_is_made_smaller() {
        let mut seed = 5;
        let art = image::RgbaImage::from_fn(300, 300, |x, y| {
            let alpha = if (x, y) == (0, 0) {
                255
            } else {
                64 + noise(&mut seed) / 2
            };
            image::Rgba([noise(&mut seed), noise(&mut seed), noise(&mut seed), alpha])
        });
        let png = png_of(&art);
        let budget = base64(&png).len() / 2;

        let text = thumbnail_text_within(&png, true, budget);

        assert!(!text.is_empty() && text.len() <= budget);
        let bytes = unbase64(&text);
        assert_eq!(image::guess_format(&bytes).expect("a format"), image::ImageFormat::Png);
        let decoded = image::load_from_memory(&bytes).expect("a PNG").to_rgba8();
        assert!(decoded.pixels().any(|pixel| pixel.0[3] < 255));
    }

    #[test]
    fn art_that_fits_goes_out_unchanged_up_to_the_last_character() {
        let png = png_of(&noisy(40, 40));
        let whole = base64(&png);

        assert_eq!(thumbnail_text_within(&png, true, whole.len()), whole);
        assert_ne!(thumbnail_text_within(&png, true, whole.len() - 1), whole);
    }

    #[test]
    fn art_that_cannot_be_decoded_and_does_not_fit_gives_an_empty_text() {
        let junk = b"not an image".repeat(100);

        assert_eq!(thumbnail_text_within(&junk, true, 64), "");
    }

    #[test]
    fn art_that_cannot_fit_above_the_smallest_side_gives_an_empty_text() {
        let png = png_of(&noisy(300, 300));

        assert_eq!(thumbnail_text_within(&png, true, 16), "");
    }

    #[test]
    fn art_with_a_transparent_corner_is_cropped_to_233_by_233_from_34_1_unless_the_system_is_cobalt() {
        let png = png_of(&picture(0));

        let cropped = decode(&thumbnail_text(&png, false));

        assert_eq!((cropped.width(), cropped.height()), (233, 233));
        assert_eq!(
            cropped.get_pixel(0, 0).0,
            [0, 0, 255, 255],
            "the marker at (34, 1) is the new origin"
        );
        assert_eq!(cropped.get_pixel(232, 232).0, [255, 0, 0, 255]);
    }

    #[test]
    fn art_is_left_whole_on_cobalt_and_when_the_corner_is_opaque() {
        let transparent = png_of(&picture(0));
        let opaque = png_of(&picture(255));

        assert_eq!(
            thumbnail_text(&transparent, true),
            base64(&transparent),
            "Cobalt never crops"
        );
        assert_eq!(
            thumbnail_text(&opaque, false),
            base64(&opaque),
            "an opaque corner is not cropped"
        );
    }

    #[test]
    fn art_that_cannot_be_decoded_goes_out_as_it_is() {
        assert_eq!(thumbnail_text(b"not an image", false), base64(b"not an image"));
    }

    #[test]
    fn a_crop_of_a_small_picture_keeps_the_size_and_is_transparent_beyond_the_source() {
        let mut small = image::RgbaImage::from_pixel(100, 100, image::Rgba([9, 9, 9, 255]));
        small.put_pixel(0, 0, image::Rgba([9, 9, 9, 0]));

        let cropped = decode(&thumbnail_text(&png_of(&small), false));

        assert_eq!((cropped.width(), cropped.height()), (233, 233));
        assert_eq!(cropped.get_pixel(0, 0).0, [9, 9, 9, 255]);
        assert_eq!(cropped.get_pixel(232, 232).0[3], 0);
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "reads the media session of the machine it runs on"]
    #[allow(clippy::print_stdout)]
    fn now_playing_reads_the_current_session() {
        let session = MediaSession::start().expect("the session manager is created");
        // The worker reads the session before it answers the first poll; give its first reading a moment.
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let fields = session.fields();
        let art = session.thumbnail_if_changed();

        let text = serde_json::to_string_pretty(&Value::Object(fields.clone())).expect("JSON");
        match fields["State"] {
            Value::Bool(true) => {
                println!("now playing session found:\n{text}");
                println!("thumbnail text length: {:?}", art.as_ref().map(String::len));
            }
            _ => println!("no session:\n{text}"),
        }
        assert_eq!(keys(&fields), KEYS);
        assert!(
            session.thumbnail_if_changed().is_none(),
            "the art is taken once per track"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn an_os_without_a_provider_reports_unsupported() {
        assert!(matches!(MediaSession::start(), Err(SysInfoError::Unsupported(_))));
    }
}
