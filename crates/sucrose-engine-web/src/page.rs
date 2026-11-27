//! The page logic both web engines share: what a page is told after its main frame loaded (the property calls, the
//! compatible calls, the in-page pause) and with every system data push, built from plain values so CEF and WebView2 call
//! the same functions and send the same scripts in the same order (WAPI-01, WAPI-02, WAPI-03, WAPI-04, FMT-07). No OS
//! code and no IPC type: each engine builds [`PageState`] from its own assignment (`sucrose-ipc` depends on this crate).

use std::{
    path::Path,
    sync::{
        Mutex, PoisonError,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};

use serde_json::Value;
use sucrose_core::{
    compatible::{COMPATIBLE_FILE, CompatibleKey, DataGroup, read_compatible},
    properties::{PropertiesError, SucroseProperties},
    settings::Stretch,
    theme::WallpaperType,
};

use crate::{
    args::WebEngine,
    js::{self, CompatibleState, PageCall},
    pages::{self, GeneratedPage},
};

/// What a web engine shows: a theme's own page, or a page Sucrose generated for a Gif, Video or YouTube theme together
/// with the built-in property set that page uses. The page decides which calls the page gets after it loaded.
#[derive(Debug, Clone, Copy)]
pub enum PageKind {
    /// The page of a Web theme: its own `SucroseProperties.json` and `SucroseCompatible.json`, and system data on request.
    Theme,
    /// A generated page with its built-in set: the set's property calls and the current engine values, and no
    /// compatible calls and no system data.
    Generated(GeneratedPage, &'static SucroseProperties),
    /// The page of a Url theme: a site Sucrose does not own. It gets no property calls, no compatible calls and no
    /// system data (legacy `engines.md` §8); only the in-page pause, which is no part of the wallpaper API.
    Url,
}

impl PageKind {
    /// What `engine` shows for a wallpaper of `kind`: a generated page for Gif, Video and YouTube, the address itself for Url,
    /// the theme's own page for everything else a web engine plays.
    #[must_use]
    pub fn for_wallpaper(kind: WallpaperType, engine: WebEngine) -> Self {
        match (kind, GeneratedPage::for_kind(kind)) {
            (_, Some(page)) => Self::Generated(page, pages::builtin_set(engine)),
            (WallpaperType::Url, None) => Self::Url,
            (_, None) => Self::Theme,
        }
    }

    /// Whether the page is a Web theme's own page (the only one with theme files for calls and system data).
    #[must_use]
    pub fn is_theme(&self) -> bool {
        matches!(self, Self::Theme)
    }

    /// Whether the renderer forwards mouse and keyboard input to the page: Web and Url pages do, the generated Gif,
    /// Video and YouTube pages do not (legacy `engines.md` §5).
    #[must_use]
    pub fn takes_input(&self) -> bool {
        matches!(self, Self::Theme | Self::Url)
    }

    /// The generated page, when this is one.
    #[must_use]
    pub fn generated(&self) -> Option<GeneratedPage> {
        match self {
            Self::Theme | Self::Url => None,
            Self::Generated(page, _) => Some(*page),
        }
    }
}

/// The newest value of each setting and of each system data group, shared by the thread-safe handle of an engine and the
/// surface thread that runs the page calls.
#[derive(Debug)]
pub struct PageState {
    paused: AtomicBool,
    /// Whether this renderer plays the sound (D-02): a follower does not.
    audio: AtomicBool,
    volume: AtomicU8,
    looping: AtomicBool,
    shuffle: AtomicBool,
    stretch: Mutex<Stretch>,
    /// The legacy `ThemeType` name the `ThemeType` and `SystemTheme` calls carry, fixed for the life of the renderer.
    theme_type: String,
    /// The newest snapshot of each system data group the service pushed.
    data: Mutex<Vec<(DataGroup, Value)>>,
}

impl PageState {
    /// The state a renderer starts with. The volume is capped at 100.
    #[must_use]
    pub fn new(
        paused: bool,
        volume: u8,
        looping: bool,
        shuffle: bool,
        stretch: Stretch,
        theme_type: impl Into<String>,
    ) -> Self {
        Self {
            paused: AtomicBool::new(paused),
            audio: AtomicBool::new(true),
            volume: AtomicU8::new(volume.min(100)),
            looping: AtomicBool::new(looping),
            shuffle: AtomicBool::new(shuffle),
            stretch: Mutex::new(stretch),
            theme_type: theme_type.into(),
            data: Mutex::new(Vec::new()),
        }
    }

    /// Whether the renderer was last told to pause.
    #[must_use]
    pub fn paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    /// Keeps whether the renderer is paused.
    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::SeqCst);
    }

    /// Whether this renderer plays the sound, as last set; the legacy default and the start state is on.
    #[must_use]
    pub fn audio(&self) -> bool {
        self.audio.load(Ordering::SeqCst)
    }

    /// Keeps whether this renderer plays the sound.
    pub fn set_audio(&self, audio: bool) {
        self.audio.store(audio, Ordering::SeqCst);
    }

    /// The last volume, 0..=100.
    #[must_use]
    pub fn volume(&self) -> u8 {
        self.volume.load(Ordering::SeqCst)
    }

    /// Keeps the volume, capped at 100.
    pub fn set_volume(&self, volume: u8) {
        self.volume.store(volume.min(100), Ordering::SeqCst);
    }

    /// Whether the wallpaper loops, as last set.
    #[must_use]
    pub fn looping(&self) -> bool {
        self.looping.load(Ordering::SeqCst)
    }

    /// Keeps whether the wallpaper loops.
    pub fn set_looping(&self, looping: bool) {
        self.looping.store(looping, Ordering::SeqCst);
    }

    /// The value of the `ShuffleMode` call, as last set.
    #[must_use]
    pub fn shuffle(&self) -> bool {
        self.shuffle.load(Ordering::SeqCst)
    }

    /// Keeps the value of the `ShuffleMode` call.
    pub fn set_shuffle(&self, shuffle: bool) {
        self.shuffle.store(shuffle, Ordering::SeqCst);
    }

    /// How the wallpaper is fitted, as last set.
    #[must_use]
    pub fn stretch(&self) -> Stretch {
        *self.stretch.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Keeps how the wallpaper is fitted.
    pub fn set_stretch(&self, stretch: Stretch) {
        *self.stretch.lock().unwrap_or_else(PoisonError::into_inner) = stretch;
    }

    /// What the non-data compatible calls are built from, as the engine was last told (WAPI-02).
    #[must_use]
    pub fn compatible(&self) -> CompatibleState {
        CompatibleState {
            looping: self.looping(),
            shuffle: self.shuffle(),
            stretch: self.stretch(),
            theme_type: self.theme_type.clone(),
            volume: self.volume(),
        }
    }

    /// Keeps the newest snapshot of each group. A text that is not JSON is dropped with a log line and the group keeps its
    /// last value.
    pub fn store_data(&self, data: Vec<(DataGroup, String)>) {
        let mut stored = self.data.lock().unwrap_or_else(PoisonError::into_inner);
        for (group, text) in data {
            let value = match serde_json::from_str::<Value>(&text) {
                Ok(value) => value,
                Err(error) => {
                    tracing::warn!(?group, %error, "a data snapshot is not valid JSON and is dropped");
                    continue;
                }
            };
            match stored.iter_mut().find(|(known, _)| *known == group) {
                Some(slot) => slot.1 = value,
                None => stored.push((group, value)),
            }
        }
    }

    /// The newest snapshot of each group that has one.
    #[must_use]
    pub fn data(&self) -> Vec<(DataGroup, Value)> {
        self.data.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

/// Which theme files were already reported as unusable, so each is logged once per renderer.
#[derive(Debug, Default)]
pub struct Complaints {
    /// `SucroseProperties.json` was reported.
    pub properties: AtomicBool,
    /// `SucroseCompatible.json` was reported.
    pub compatible: AtomicBool,
}

/// A reason for a log line that names what is wrong with a theme file without its path or its text (D-08).
#[must_use]
pub fn describe(error: &PropertiesError) -> String {
    match error {
        PropertiesError::Missing(_) => "the file is missing".to_owned(),
        PropertiesError::Unreadable { source, .. } => format!("the file cannot be read ({:?})", source.kind()),
        PropertiesError::Invalid { reason, .. } => reason.clone(),
    }
}

/// The calls that tell a generated page what the engine holds: the volume level (0 for a renderer that does not play
/// the sound), whether the media loops, whether a playlist shuffles (the YouTube page only) and how it is fitted. They
/// are the same scripts the engine runs when one of these changes while the page plays.
#[must_use]
pub fn engine_calls(page: GeneratedPage, state: &PageState) -> Vec<PageCall> {
    let mut calls = vec![
        volume_call(state),
        PageCall {
            kind: js::KIND_ENGINE,
            key: "loop".to_owned(),
            script: pages::loop_script(page, state.looping()),
        },
    ];
    calls.extend(shuffle_call(page, state.shuffle()));
    calls.push(PageCall {
        kind: js::KIND_ENGINE,
        key: "stretch".to_owned(),
        script: pages::stretch_script(page, state.stretch()),
    });
    calls
}

/// The shuffle call of a generated page: only the YouTube page has one (the legacy `setShuffle`); `None` for the others.
#[must_use]
pub fn shuffle_call(page: GeneratedPage, on: bool) -> Option<PageCall> {
    pages::shuffle_script(page, on).map(|script| PageCall {
        kind: js::KIND_ENGINE,
        key: "shuffle".to_owned(),
        script,
    })
}

/// The volume call of a generated page: the engine's volume, or 0 for a renderer that does not play the sound (D-06, D-02).
#[must_use]
pub fn volume_call(state: &PageState) -> PageCall {
    PageCall {
        kind: js::KIND_ENGINE,
        key: "volume".to_owned(),
        script: pages::volume_script(pages::engine_volume_level(state.volume(), state.audio())),
    }
}

/// The scripts a page gets after its main frame loaded, in the order they must run (WAPI-01, WAPI-02): the property
/// calls of `SucroseProperties.json`, one per control in file order; then the non-data compatible calls of
/// `SucroseCompatible.json` in the legacy run order, so they come after every property call and before the first data
/// push; then, when the renderer was paused before the page finished loading, the pause. Only a loaded page (HTTP 200)
/// gets them; a theme without a file or with nothing in it gets none, and a file that cannot be used is logged once
/// per renderer (`complaints`) while the page still plays. A reload runs this again, so every key is sent again.
///
/// A generated page (`PageKind::Generated`) gets the property calls of its built-in set instead (never the theme's
/// own file, and no compatible calls), then the current engine values once (volume, loop, stretch), then the pause.
#[must_use]
pub fn page_calls(
    kind: PageKind,
    theme_dir: &Path,
    status: i32,
    state: &PageState,
    complaints: &Complaints,
) -> Vec<PageCall> {
    if status != 200 {
        tracing::debug!(status, "no page calls: the main frame did not load with status 200");
        return Vec::new();
    }
    if let PageKind::Url = kind {
        // Not a wallpaper API call: the Light pause of a page that was paused before it finished loading.
        return if state.paused() {
            vec![js::pause_call(true)]
        } else {
            Vec::new()
        };
    }
    if let PageKind::Generated(page, set) = kind {
        let mut calls = js::property_calls_of(set, theme_dir);
        calls.extend(engine_calls(page, state));
        if state.paused() {
            calls.push(js::pause_call(true));
        }
        return calls;
    }
    let mut calls = match js::property_calls(theme_dir) {
        Ok(calls) => calls,
        Err(error) => {
            if !complaints.properties.swap(true, Ordering::SeqCst) {
                tracing::warn!(reason = %describe(&error), "SucroseProperties.json cannot be used, the page gets no property calls");
            }
            Vec::new()
        }
    };
    match js::compatible_calls(theme_dir, &state.compatible()) {
        Ok(compatible) => calls.extend(compatible),
        Err(error) => {
            if !complaints.compatible.swap(true, Ordering::SeqCst) {
                tracing::warn!(reason = %describe(&error), "SucroseCompatible.json cannot be used, the page gets no compatible calls");
            }
        }
    }
    if state.paused() {
        calls.push(js::pause_call(true));
    }
    calls
}

/// The system data groups the theme's `SucroseCompatible.json` asks for, in the legacy payload order; empty for a theme
/// without the file, with nothing in it or with a file that cannot be used (`page_calls` reports that), and for a page
/// that did not load with status 200.
#[must_use]
pub fn wanted_groups(theme_dir: &Path, status: i32) -> Vec<DataGroup> {
    if status != 200 {
        return Vec::new();
    }
    read_compatible(&theme_dir.join(COMPATIBLE_FILE)).map_or_else(|_| Vec::new(), |compatible| compatible.data_groups())
}

/// The non-empty templates of the theme's `SucroseCompatible.json`; none when the file is missing or unusable.
#[must_use]
pub fn load_templates(theme_dir: &Path) -> Vec<(CompatibleKey, String)> {
    read_compatible(&theme_dir.join(COMPATIBLE_FILE)).map_or_else(
        |_| Vec::new(),
        |compatible| {
            compatible
                .templates()
                .into_iter()
                .map(|(key, template)| (key, template.to_owned()))
                .collect()
        },
    )
}

/// The calls of one data push: the templates run in the legacy order with the newest value of everything (the legacy
/// engine runs all of them for every message it receives).
#[must_use]
pub fn push_calls(templates: &[(CompatibleKey, String)], state: &PageState) -> Vec<PageCall> {
    let templates: Vec<(CompatibleKey, &str)> = templates.iter().map(|(key, text)| (*key, text.as_str())).collect();
    js::push_calls(&templates, &state.compatible(), &state.data())
}
