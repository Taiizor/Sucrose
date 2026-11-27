//! The Audio group: the legacy `GetAudioInfo` object, whose `Data`, `DataLeft` and `DataRight` are the spectrum of what the default
//! render device plays. The spectrum comes from a WASAPI loopback capture of that device, fed to the legacy visualizer
//! ([`crate::visualizer`]). The capture exists only while a running page asks for the Audio group (D-24, D-008): the data hub
//! starts it with the first such page and drops it with the last.
//!
//! Captured samples never leave the capture thread: they live in the visualizer's in-memory ring, and only the 128-band
//! magnitudes are published (T-04-62). The media-session fields of the object are at their legacy defaults unless the caller
//! passes the keys of a [`crate::MediaSession`] (`snapshot_with`, `audio_unavailable_with`).

use std::{
    sync::{Arc, Mutex, PoisonError, mpsc},
    thread::JoinHandle,
};

use serde_json::{Map, Value, json};

use crate::{SysInfoError, visualizer::Bands};

/// How the samples of a captured packet are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) enum SampleFormat {
    /// 32-bit IEEE float, what a shared-mode loopback normally gives.
    Float32,
    /// 16-bit signed integer.
    Pcm16,
    /// 32-bit signed integer.
    Pcm32,
}

impl SampleFormat {
    /// The size of one sample in bytes.
    #[cfg_attr(not(windows), allow(dead_code))]
    pub(crate) fn bytes(self) -> usize {
        match self {
            Self::Pcm16 => 2,
            Self::Float32 | Self::Pcm32 => 4,
        }
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    fn name(self) -> &'static str {
        match self {
            Self::Float32 => "float32",
            Self::Pcm16 => "pcm16",
            Self::Pcm32 => "pcm32",
        }
    }
}

/// Converts the little-endian samples of `bytes` to floats in [-1, 1] and appends them to `out`; a trailing partial
/// sample is ignored.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn convert(bytes: &[u8], format: SampleFormat, out: &mut Vec<f32>) {
    match format {
        SampleFormat::Float32 => out.extend(bytes.as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b))),
        SampleFormat::Pcm16 => out.extend(
            bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| f32::from(i16::from_le_bytes(*b)) / 32768.0),
        ),
        SampleFormat::Pcm32 => out.extend(
            bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| i32::from_le_bytes(*b) as f32 / 2_147_483_648.0),
        ),
    }
}

/// The Audio object, the keys in the order of `GetAudioInfo` in `Data.cs`. `bands` fills the three spectrum arrays (mono is
/// `Data`); without them they are empty, as in the legacy default. The media fields are at their legacy default unless
/// `media` holds their keys, which then replace them in place (the key order of `GetAudioInfo` stays).
fn payload(bands: Option<&Bands>, media: Option<&Map<String, Value>>) -> Value {
    let (mono, left, right) = bands.map_or((Vec::new(), Vec::new(), Vec::new()), |bands| {
        (bands.mono.clone(), bands.left.clone(), bands.right.clone())
    });
    let mut object = json!({
        "State": false,
        "Title": "",
        "Artist": "",
        "Subtitle": "",
        "AlbumTitle": "",
        "Data": mono,
        "AlbumArtist": "",
        "SourceAppId": "",
        "TrackNumber": 0,
        "MediaType": "Unknown",
        "PlaybackRate": 0.0,
        "PropsValid": "Capabilities",
        "RepeatMode": "Unknown",
        "ShuffleEnabled": false,
        "DataLeft": left,
        "PlaybackMode": "Unknown",
        "AlbumTrackCount": 0,
        "DataRight": right,
        "ThumbnailString": "",
        "PlaybackState": "Unknown",
        "EndTime": 0.0,
        "Position": 0.0,
        "StartTime": 0.0,
        "LastPlayingFileTime": "0001-01-01T00:00:00",
        "PositionSetFileTime": "0001-01-01T00:00:00",
        "MaxSeekTime": 0.0,
        "MinSeekTime": 0.0
    });
    if let (Some(media), Some(fields)) = (media, object.as_object_mut()) {
        for (key, value) in media {
            // Only keys the object has: the spectrum and the thumbnail are not the media session's to set here.
            if !matches!(key.as_str(), "Data" | "DataLeft" | "DataRight" | "ThumbnailString")
                && let Some(slot) = fields.get_mut(key)
            {
                *slot = value.clone();
            }
        }
    }
    object
}

/// The Audio group of the legacy default, which a page sees while no device can be captured: `State` false, empty
/// arrays.
#[must_use]
pub fn audio_unavailable() -> Value {
    payload(None, None)
}

/// [`audio_unavailable`] with the media keys of a media session: a page on a machine whose render device cannot be
/// captured still sees what is playing.
#[must_use]
pub fn audio_unavailable_with(media: &Map<String, Value>) -> Value {
    payload(None, Some(media))
}

/// A running loopback capture of the default render device: a worker thread that keeps the latest bands.
pub struct AudioCapture {
    latest: Arc<Mutex<Bands>>,
    format: String,
    /// Dropping it ends the worker's wait.
    stop: Option<mpsc::Sender<()>>,
    worker: Option<JoinHandle<()>>,
}

impl AudioCapture {
    /// Opens the default render device in shared loopback mode and starts the worker. The first bands are 128 zeros, so a
    /// page sees a silent device as silence.
    ///
    /// # Errors
    /// [`SysInfoError::Os`] when there is no render device or it cannot be captured,
    /// [`SysInfoError::Unsupported`] on an OS without a provider.
    pub fn start() -> Result<Self, SysInfoError> {
        let latest = Arc::new(Mutex::new(Bands::silent()));
        let (stop, stopped) = mpsc::channel::<()>();
        let (ready, started) = mpsc::sync_channel::<Result<String, SysInfoError>>(1);
        let worker = {
            let latest = Arc::clone(&latest);
            platform::spawn(latest, stopped, ready)?
        };
        match started.recv() {
            Ok(Ok(format)) => Ok(Self {
                latest,
                format,
                stop: Some(stop),
                worker: Some(worker),
            }),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(_) => {
                let _ = worker.join();
                Err(SysInfoError::Os {
                    call: "audio capture",
                    message: "the capture thread ended before it started".to_owned(),
                })
            }
        }
    }

    /// What the capture opened, for the log: the sample rate, the channels and the sample format.
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }

    /// The Audio object with the latest bands.
    #[must_use]
    pub fn snapshot(&self) -> Value {
        let bands = self.latest.lock().unwrap_or_else(PoisonError::into_inner);
        payload(Some(&bands), None)
    }

    /// The Audio object with the latest bands and the media keys of a media session.
    #[must_use]
    pub fn snapshot_with(&self, media: &Map<String, Value>) -> Value {
        let bands = self.latest.lock().unwrap_or_else(PoisonError::into_inner);
        payload(Some(&bands), Some(media))
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        // Closing the channel wakes the worker at once; the join ends the capture before the device is let go.
        drop(self.stop.take());
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            tracing::error!("the audio capture worker panicked");
        }
    }
}

/// The text of a capture format for the log.
#[cfg_attr(not(windows), allow(dead_code))]
fn describe(rate: u32, channels: usize, format: SampleFormat) -> String {
    format!("{rate} Hz, {channels} channels, {}", format.name())
}

#[cfg(not(windows))]
mod platform {
    use std::{
        sync::{Arc, Mutex, mpsc},
        thread::JoinHandle,
    };

    use crate::{SysInfoError, visualizer::Bands};

    pub(super) fn spawn(
        _latest: Arc<Mutex<Bands>>,
        _stopped: mpsc::Receiver<()>,
        _ready: mpsc::SyncSender<Result<String, SysInfoError>>,
    ) -> Result<JoinHandle<()>, SysInfoError> {
        Err(SysInfoError::Unsupported("the audio capture"))
    }
}

#[cfg(windows)]
mod platform {
    use std::{
        ptr,
        sync::{Arc, Mutex, PoisonError, mpsc},
        thread::{self, JoinHandle},
        time::{Duration, Instant},
    };

    use windows::Win32::{
        Media::Audio::{
            AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, IAudioCaptureClient,
            IAudioClient, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, WAVEFORMATEX, WAVEFORMATEXTENSIBLE,
            eMultimedia, eRender,
        },
        System::Com::{
            CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
        },
    };

    use super::{SampleFormat, convert, describe};
    use crate::{
        SysInfoError,
        visualizer::{Bands, Visualizer},
    };

    /// How often the worker looks for new packets. The device buffer holds 200 ms, so a slow wake-up loses nothing.
    const POLL: Duration = Duration::from_millis(10);
    /// How long a device that delivers no packets is quiet before the visualizer is fed silence for the time that passed:
    /// a loopback of a device that plays nothing delivers nothing, and the bars must fall to 0 by the release envelope.
    const SILENCE_AFTER: Duration = Duration::from_millis(40);
    /// The most silence one feed adds, in frames: the length of the FFT window.
    const MAX_SILENCE_FRAMES: f64 = 4096.0;
    /// How long a failed re-creation (after a device change or an invalidated device) waits for the next try.
    const RETRY_AFTER: Duration = Duration::from_secs(2);
    /// How often the default render device is compared with the one being captured. A poll of the device id instead of an
    /// IMMNotificationClient: implementing that COM interface needs a direct dependency on windows-core, which is
    /// only transitive here and has no package audit (T-04-SC).
    const DEVICE_CHECK: Duration = Duration::from_secs(1);
    /// The requested device buffer, in 100 ns units.
    const BUFFER_DURATION: i64 = 2_000_000;
    const WAVE_FORMAT_PCM: u16 = 1;
    const WAVE_FORMAT_IEEE_FLOAT: u16 = 3;
    const WAVE_FORMAT_EXTENSIBLE: u16 = 0xFFFE;

    fn os(call: &'static str) -> impl Fn(windows::core::Error) -> SysInfoError {
        move |error| SysInfoError::Os {
            call,
            message: error.to_string(),
        }
    }

    pub(super) fn spawn(
        latest: Arc<Mutex<Bands>>,
        stopped: mpsc::Receiver<()>,
        ready: mpsc::SyncSender<Result<String, SysInfoError>>,
    ) -> Result<JoinHandle<()>, SysInfoError> {
        thread::Builder::new()
            .name("audio-capture".to_owned())
            .spawn(move || run(&latest, &stopped, &ready))
            .map_err(|error| SysInfoError::Os {
                call: "spawn the audio capture thread",
                message: error.to_string(),
            })
    }

    /// COM on the capture thread, left again when the thread ends.
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
            // SAFETY: pairs the successful CoInitializeEx of Com::init on this thread; every COM object of the thread was
            // dropped before, because Com is the first local of run.
            unsafe { CoUninitialize() };
        }
    }

    /// The layout of the mix format of a device.
    struct MixFormat {
        rate: u32,
        channels: usize,
        sample: SampleFormat,
    }

    /// Reads the format at `format`: float32, PCM16 and PCM32 are supported, as in the legacy code.
    ///
    /// # Safety
    /// `format` points to a valid WAVEFORMATEX, which is a WAVEFORMATEXTENSIBLE when its tag and `cbSize` say so.
    unsafe fn read_format(format: *const WAVEFORMATEX) -> Result<MixFormat, SysInfoError> {
        // SAFETY: the caller guarantees a valid WAVEFORMATEX; the struct is packed, so it is read unaligned.
        let base = unsafe { ptr::read_unaligned(format) };
        let (tag, bits, channels, rate, size) = (
            base.wFormatTag,
            base.wBitsPerSample,
            base.nChannels,
            base.nSamplesPerSec,
            base.cbSize,
        );
        let effective_tag = if tag == WAVE_FORMAT_EXTENSIBLE && usize::from(size) >= 22 {
            // SAFETY: the tag and cbSize say the format is a WAVEFORMATEXTENSIBLE; the struct is packed.
            let extended = unsafe { ptr::read_unaligned(format.cast::<WAVEFORMATEXTENSIBLE>()) };
            // The sub-format GUID is {0000000N-0000-0010-8000-00AA00389B71} with N the format tag.
            let sub = extended.SubFormat;
            u16::try_from(sub.data1).unwrap_or(0)
        } else {
            tag
        };
        let sample = match (effective_tag, bits) {
            (WAVE_FORMAT_IEEE_FLOAT, 32) => SampleFormat::Float32,
            (WAVE_FORMAT_PCM, 16) => SampleFormat::Pcm16,
            (WAVE_FORMAT_PCM, 32) => SampleFormat::Pcm32,
            _ => {
                return Err(SysInfoError::Os {
                    call: "IAudioClient::GetMixFormat",
                    message: format!("the sample format (tag {effective_tag}, {bits} bits) is not supported"),
                });
            }
        };
        Ok(MixFormat {
            rate,
            channels: usize::from(channels.max(1)),
            sample,
        })
    }

    /// The loopback capture of the default render device, stopped when dropped.
    struct Session {
        /// The id of the device being captured.
        device_id: String,
        client: IAudioClient,
        capture: IAudioCaptureClient,
        format: MixFormat,
        visualizer: Visualizer,
        frames: Vec<f32>,
        last_activity: Instant,
    }

    /// The id of the default render device.
    fn default_device_id(enumerator: &IMMDeviceEnumerator) -> Result<String, SysInfoError> {
        // SAFETY: the enumerator is a live COM object of this thread's apartment.
        let device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) }
            .map_err(os("IMMDeviceEnumerator::GetDefaultAudioEndpoint"))?;
        device_id(&device)
    }

    fn device_id(device: &IMMDevice) -> Result<String, SysInfoError> {
        // SAFETY: the device is live; GetId returns a CoTaskMemAlloc'd, NUL-terminated string.
        let id = unsafe { device.GetId() }.map_err(os("IMMDevice::GetId"))?;
        // SAFETY: id is the valid string GetId just returned.
        let text = unsafe { id.to_string() }.unwrap_or_default();
        // SAFETY: GetId documents CoTaskMemFree for the string, and id is not used after this.
        unsafe { CoTaskMemFree(Some(id.as_ptr().cast_const().cast())) };
        Ok(text)
    }

    impl Session {
        fn open(enumerator: &IMMDeviceEnumerator) -> Result<Self, SysInfoError> {
            // SAFETY: the enumerator is a live COM object of this thread's apartment.
            let device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) }
                .map_err(os("IMMDeviceEnumerator::GetDefaultAudioEndpoint"))?;
            let device_id = device_id(&device)?;
            // SAFETY: the device is live; Activate fills a new IAudioClient.
            let client: IAudioClient =
                unsafe { device.Activate(CLSCTX_ALL, None) }.map_err(os("IMMDevice::Activate"))?;
            // SAFETY: GetMixFormat returns a CoTaskMemAlloc'd format that is valid until it is freed below.
            let mix = unsafe { client.GetMixFormat() }.map_err(os("IAudioClient::GetMixFormat"))?;
            // SAFETY: mix is the valid pointer GetMixFormat just returned; the format is initialised before it is freed,
            // and it is freed on every path afterwards.
            let initialised = unsafe { read_format(mix) }.and_then(|format| {
                // SAFETY: mix is still valid; Initialize copies what it needs from it.
                unsafe {
                    client.Initialize(
                        AUDCLNT_SHAREMODE_SHARED,
                        AUDCLNT_STREAMFLAGS_LOOPBACK,
                        BUFFER_DURATION,
                        0,
                        mix,
                        None,
                    )
                }
                .map_err(os("IAudioClient::Initialize"))?;
                Ok(format)
            });
            // SAFETY: mix came from GetMixFormat, which documents CoTaskMemFree for it, and is not used after this.
            unsafe { CoTaskMemFree(Some(mix.cast_const().cast())) };
            let format = initialised?;
            // SAFETY: the client is initialised, so the capture service can be asked for.
            let capture: IAudioCaptureClient =
                unsafe { client.GetService() }.map_err(os("IAudioClient::GetService"))?;
            // SAFETY: the client is initialised and not started yet.
            unsafe { client.Start() }.map_err(os("IAudioClient::Start"))?;
            Ok(Self {
                visualizer: Visualizer::new(format.rate),
                device_id,
                client,
                capture,
                format,
                frames: Vec::new(),
                last_activity: Instant::now(),
            })
        }

        fn describe(&self) -> String {
            describe(self.format.rate, self.format.channels, self.format.sample)
        }

        /// Reads every packet that is waiting into the visualizer and publishes the next result. A device that plays
        /// nothing delivers no packets; its silence is fed in by the time that passes, so the bars fall to 0.
        fn pump(&mut self, latest: &Mutex<Bands>) -> windows::core::Result<()> {
            let mut got_packet = false;
            loop {
                // SAFETY: the capture client belongs to the started client of this session.
                if unsafe { self.capture.GetNextPacketSize() }? == 0 {
                    break;
                }
                let mut data: *mut u8 = ptr::null_mut();
                let (mut frames, mut flags) = (0_u32, 0_u32);
                // SAFETY: the out parameters are valid locals; the buffer is held until ReleaseBuffer below.
                unsafe {
                    self.capture
                        .GetBuffer(&raw mut data, &raw mut frames, &raw mut flags, None, None)
                }?;
                self.frames.clear();
                let count = frames as usize;
                if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 || data.is_null() {
                    self.frames.resize(count * self.format.channels, 0.0);
                } else {
                    let length = count * self.format.channels * self.format.sample.bytes();
                    // SAFETY: GetBuffer returned a buffer of `frames` frames of the mix format, valid until ReleaseBuffer.
                    let bytes = unsafe { std::slice::from_raw_parts(data, length) };
                    convert(bytes, self.format.sample, &mut self.frames);
                }
                // SAFETY: releases the buffer taken by the GetBuffer above with the frame count it returned.
                unsafe { self.capture.ReleaseBuffer(frames) }?;
                self.visualizer.push(&self.frames, self.format.channels);
                got_packet = true;
            }
            let now = Instant::now();
            if got_packet || self.visualizer.is_idle() {
                self.last_activity = now;
            } else if now.duration_since(self.last_activity) >= SILENCE_AFTER {
                let silent = now.duration_since(self.last_activity).as_secs_f64() * f64::from(self.format.rate);
                self.frames.clear();
                self.frames
                    .resize(silent.min(MAX_SILENCE_FRAMES) as usize * self.format.channels, 0.0);
                self.visualizer.push(&self.frames, self.format.channels);
                self.last_activity = now;
            }
            if let Some(bands) = self.visualizer.take() {
                *latest.lock().unwrap_or_else(PoisonError::into_inner) = bands;
            }
            Ok(())
        }
    }

    impl Drop for Session {
        fn drop(&mut self) {
            // SAFETY: stops the client started in open; a failure only means it was already stopped (device gone).
            let _ = unsafe { self.client.Stop() };
        }
    }

    /// The capture thread: opens the default render device, reads it until the stop channel closes, and opens the new
    /// default device when the default changes or the device goes away.
    fn run(
        latest: &Mutex<Bands>,
        stopped: &mpsc::Receiver<()>,
        ready: &mpsc::SyncSender<Result<String, SysInfoError>>,
    ) {
        // Declared first so it is dropped last: the COM objects below go before CoUninitialize.
        let com = match Com::init() {
            Ok(com) => com,
            Err(error) => {
                let _ = ready.send(Err(error));
                return;
            }
        };
        let enumerator: IMMDeviceEnumerator =
            // SAFETY: COM is initialised on this thread; MMDeviceEnumerator is the documented class of the interface.
            match unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }.map_err(os("CoCreateInstance")) {
                Ok(enumerator) => enumerator,
                Err(error) => {
                    let _ = ready.send(Err(error));
                    drop(com);
                    return;
                }
            };
        let mut ready = Some(ready);
        let mut session: Option<Session> = None;
        let mut retry_at = Instant::now();
        let mut next_check = Instant::now() + DEVICE_CHECK;
        let mut warned = false;
        loop {
            if session.is_some() && Instant::now() >= next_check {
                next_check = Instant::now() + DEVICE_CHECK;
                let current = default_device_id(&enumerator);
                if session
                    .as_ref()
                    .is_some_and(|active| current.as_ref().ok() != Some(&active.device_id))
                {
                    tracing::info!("the default audio device changed, the capture is opened again");
                    session = None;
                    retry_at = Instant::now();
                }
            }
            if session.is_none() && Instant::now() >= retry_at {
                match Session::open(&enumerator) {
                    Ok(opened) => {
                        if let Some(ready) = ready.take() {
                            let _ = ready.send(Ok(opened.describe()));
                        } else {
                            tracing::info!(format = %opened.describe(), "audio capture reopened");
                        }
                        warned = false;
                        next_check = Instant::now() + DEVICE_CHECK;
                        session = Some(opened);
                    }
                    Err(error) => {
                        if let Some(ready) = ready.take() {
                            let _ = ready.send(Err(error));
                            break;
                        }
                        if !warned {
                            warned = true;
                            tracing::warn!(%error, "the audio capture cannot be reopened, it is tried again every few seconds");
                        }
                        retry_at = Instant::now() + RETRY_AFTER;
                    }
                }
            }
            if let Some(active) = &mut session
                && let Err(error) = active.pump(latest)
            {
                tracing::warn!(%error, "the audio capture failed (the device may be gone), it is opened again");
                session = None;
                retry_at = Instant::now() + RETRY_AFTER;
            }
            match stopped.recv_timeout(POLL) {
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        drop(session);
        drop(enumerator);
        drop(com);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualizer::BAND_COUNT;

    const KEYS: [&str; 27] = [
        "State",
        "Title",
        "Artist",
        "Subtitle",
        "AlbumTitle",
        "Data",
        "AlbumArtist",
        "SourceAppId",
        "TrackNumber",
        "MediaType",
        "PlaybackRate",
        "PropsValid",
        "RepeatMode",
        "ShuffleEnabled",
        "DataLeft",
        "PlaybackMode",
        "AlbumTrackCount",
        "DataRight",
        "ThumbnailString",
        "PlaybackState",
        "EndTime",
        "Position",
        "StartTime",
        "LastPlayingFileTime",
        "PositionSetFileTime",
        "MaxSeekTime",
        "MinSeekTime",
    ];

    fn keys(value: &Value) -> Vec<&str> {
        value
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect()
    }

    #[test]
    fn the_object_has_the_get_audio_info_keys_in_that_order() {
        assert_eq!(keys(&payload(None, None)), KEYS);
        assert_eq!(keys(&payload(Some(&Bands::silent()), None)), KEYS);
    }

    #[test]
    fn the_legacy_default_media_keys_are_those_of_the_media_provider() {
        let plain = payload(None, None);
        let with_defaults = payload(None, Some(&crate::media_unavailable()));

        assert_eq!(plain, with_defaults);
    }

    #[test]
    fn media_keys_replace_their_defaults_in_place_and_never_touch_the_spectrum_or_the_thumbnail() {
        let mut media = Map::new();
        media.insert("Title".to_owned(), json!("Song"));
        media.insert("State".to_owned(), json!(true));
        media.insert("Data".to_owned(), json!([1.0]));
        media.insert("ThumbnailString".to_owned(), json!("abc"));
        media.insert("Unknown".to_owned(), json!(1));

        let value = payload(Some(&Bands::silent()), Some(&media));

        assert_eq!(keys(&value), KEYS, "no key added or moved");
        assert_eq!(value["Title"], "Song");
        assert_eq!(value["State"], true);
        assert_eq!(value["Data"].as_array().map(Vec::len), Some(BAND_COUNT));
        assert_eq!(value["ThumbnailString"], "");
    }

    #[test]
    fn the_spectrum_arrays_hold_128_numbers_each_in_the_legacy_assignment() {
        let mut bands = Bands::silent();
        bands.mono[3] = 0.25;
        bands.left[4] = 0.5;
        bands.right[5] = 0.75;

        let value = payload(Some(&bands), None);

        for key in ["Data", "DataLeft", "DataRight"] {
            let array = value[key].as_array().expect("an array");
            assert_eq!(array.len(), BAND_COUNT, "{key}");
            assert!(
                array
                    .iter()
                    .all(|v| v.as_f64().is_some_and(|v| (0.0..=1.0).contains(&v))),
                "{key}"
            );
        }
        assert_eq!(value["Data"][3].as_f64(), Some(0.25), "Data is the mono spectrum");
        assert_eq!(value["DataLeft"][4].as_f64(), Some(0.5));
        assert_eq!(value["DataRight"][5].as_f64(), Some(0.75));
    }

    #[test]
    fn with_no_device_the_group_is_the_legacy_default_with_state_false_and_empty_arrays() {
        let value = audio_unavailable();

        assert_eq!(value["State"], false);
        for key in ["Data", "DataLeft", "DataRight"] {
            assert_eq!(value[key].as_array().map(Vec::len), Some(0), "{key}");
        }
        assert_eq!(value["MediaType"], "Unknown");
        assert_eq!(value["PropsValid"], "Capabilities");
        assert_eq!(value["TrackNumber"], 0);
        assert_eq!(value["LastPlayingFileTime"], "0001-01-01T00:00:00");
    }

    #[test]
    fn the_double_fields_are_written_with_a_decimal_point_like_newtonsoft_does() {
        let text = serde_json::to_string(&payload(None, None)).expect("JSON");

        assert!(text.contains("\"PlaybackRate\":0.0"), "{text}");
        assert!(text.contains("\"EndTime\":0.0"), "{text}");
    }

    #[test]
    fn samples_are_converted_from_float32_pcm16_and_pcm32() {
        let mut out = Vec::new();

        convert(&0.5_f32.to_le_bytes(), SampleFormat::Float32, &mut out);
        convert(&(-16384_i16).to_le_bytes(), SampleFormat::Pcm16, &mut out);
        convert(&i32::MIN.to_le_bytes(), SampleFormat::Pcm32, &mut out);
        convert(&[1, 2, 3], SampleFormat::Float32, &mut out);

        assert_eq!(out, [0.5, -0.5, -1.0], "a trailing partial sample is ignored");
    }

    #[cfg(windows)]
    #[test]
    fn the_capture_of_the_default_device_starts_and_stops_or_reports_why_not() {
        match AudioCapture::start() {
            Ok(capture) => {
                assert!(capture.format().contains("Hz"), "{}", capture.format());
                let value = capture.snapshot();
                assert_eq!(value["Data"].as_array().map(Vec::len), Some(BAND_COUNT));
                // Dropping joins the worker and lets the device go.
                drop(capture);
            }
            Err(error) => assert!(matches!(error, SysInfoError::Os { .. }), "{error:?}"),
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn an_os_without_a_provider_reports_unsupported() {
        assert!(matches!(AudioCapture::start(), Err(SysInfoError::Unsupported(_))));
    }
}
