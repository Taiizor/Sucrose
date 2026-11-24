//! The library page (LIB-04, UI-07): the library folder is scanned on a background thread, sorted like the legacy app and
//! shown one page at a time as theme cards. Thumbnails are decoded at card size on the background executor into a bounded
//! cache; what the cache pushes out is dropped from GPUI's image atlas.

mod card;
mod grid;
pub mod layout;
pub mod thumbs;

use std::{
    collections::{HashMap, HashSet},
    io,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::{Duration, SystemTime},
};

use gpui_kit::*;
use sucrose_core::{
    library::{self, LibraryTheme},
    paths,
    settings::{SortKind, SortMode},
    version::app_version,
};
use sucrose_ipc::portal::{PortalSettings, RuntimeState, UseOutcome};

use card::{CardData, Thumb};

use crate::ipc::ServiceLink;
use thumbs::{Bucket, CACHE_BYTES, CACHE_IMAGES, DECODE_SLOTS, DecodeQueue, ThumbCache, ThumbKey, decode_thumbnail};

/// How long a scan may take before the skeleton grid shows (UI-SPEC E1 loading row).
const SKELETON_DELAY: Duration = Duration::from_millis(150);

/// The settings the page reads, taken from what the service reported.
#[derive(Debug, Clone, PartialEq)]
struct LibrarySettings {
    /// The folder that holds the theme folders.
    location: PathBuf,
    /// Themes per page, 1..=100.
    pagination: usize,
    /// The most columns, 0 for no cap.
    adaptive_layout: usize,
    /// The gap between cards.
    margin: f32,
    sort_mode: SortMode,
    sort_kind: SortKind,
}

impl LibrarySettings {
    fn from_portal(settings: &PortalSettings) -> Self {
        // An empty location is the default folder of the classic app's data folder (the service resolves it the same way).
        let location = if settings.library_location.is_empty() {
            paths::legacy_data_dir()
                .map(|root| root.join("Library"))
                .unwrap_or_default()
        } else {
            PathBuf::from(&settings.library_location)
        };
        Self {
            location,
            pagination: usize::from(settings.library_pagination).clamp(1, 100),
            adaptive_layout: usize::from(settings.adaptive_layout),
            margin: f32::from(settings.adaptive_margin),
            sort_mode: settings.library_sort_mode,
            sort_kind: settings.library_sort_kind,
        }
    }
}

/// Where the scan stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// The service has not reported the settings yet.
    Idle,
    /// A scan runs; `skeleton` is set once it took longer than [`SKELETON_DELAY`].
    Scanning { skeleton: bool },
    /// The themes are shown.
    Ready,
    /// The library folder cannot be read.
    Failed,
}

/// What the background scan returns.
struct Scanned {
    themes: Vec<LibraryTheme>,
    corrupt: usize,
    ignored: usize,
    /// Last write of every thumbnail, for the cache keys (read here so the GPUI thread never touches the disk).
    thumb_times: HashMap<PathBuf, Option<SystemTime>>,
}

/// Scans `location` and sorts the themes; runs on the background executor.
fn scan_library(location: &Path, mode: SortMode, kind: SortKind) -> io::Result<Scanned> {
    let mut scan = library::scan(location, &app_version())?;
    library::sort(&mut scan.themes, mode, kind);
    let thumb_times = scan
        .themes
        .iter()
        .map(|theme| {
            let modified = std::fs::metadata(&theme.thumbnail).and_then(|m| m.modified()).ok();
            (theme.thumbnail.clone(), modified)
        })
        .collect();
    Ok(Scanned {
        themes: scan.themes,
        corrupt: scan.corrupt.len(),
        ignored: scan.ignored,
        thumb_times,
    })
}

/// The cache key of `theme`'s thumbnail.
fn thumb_key(times: &HashMap<PathBuf, Option<SystemTime>>, theme: &LibraryTheme, bucket: Bucket) -> ThumbKey {
    ThumbKey {
        path: theme.thumbnail.clone(),
        modified: times.get(&theme.thumbnail).copied().flatten(),
        bucket,
    }
}

/// The library page: owned by the Portal's root entity, which feeds it the settings the service reports.
pub struct LibraryView {
    settings: Option<LibrarySettings>,
    phase: Phase,
    /// The scanned themes in display order.
    themes: Vec<LibraryTheme>,
    thumb_times: HashMap<PathBuf, Option<SystemTime>>,
    /// The page shown, counted from 1.
    page: usize,
    cache: ThumbCache<Arc<RenderImage>>,
    queue: DecodeQueue,
    /// Thumbnails that could not be decoded.
    failed: HashSet<ThumbKey>,
    bucket: Bucket,
    scroll: ScrollHandle,
    /// Bumped by every scan so an older result is ignored.
    generation: u64,
    scan_task: Option<Task<()>>,
    skeleton_task: Option<Task<()>>,
    /// Sends Use requests to the service.
    link: ServiceLink,
    /// What the service is doing, as it last reported.
    runtime: Option<RuntimeState>,
    /// Whether the Portal is registered with the service.
    connected: bool,
    /// The folder of the last Use click that the service has not answered yet; its card shows the spinner.
    pending_use: Option<String>,
}

impl LibraryView {
    /// An empty page; `scale_factor` is the window's, which picks the thumbnail bucket.
    pub fn new(scale_factor: f32, link: ServiceLink) -> Self {
        Self {
            settings: None,
            phase: Phase::Idle,
            themes: Vec::new(),
            thumb_times: HashMap::new(),
            page: 1,
            cache: ThumbCache::new(CACHE_IMAGES, CACHE_BYTES),
            queue: DecodeQueue::new(DECODE_SLOTS),
            failed: HashSet::new(),
            bucket: Bucket::for_scale(scale_factor),
            scroll: ScrollHandle::new(),
            generation: 0,
            scan_task: None,
            skeleton_task: None,
            link,
            runtime: None,
            connected: false,
            pending_use: None,
        }
    }

    /// The service reported what it is doing: the Selected pill and the disabled reasons follow it, never the clicks.
    pub fn set_runtime(&mut self, runtime: RuntimeState, cx: &mut Context<Self>) {
        self.runtime = Some(runtime);
        cx.notify();
    }

    /// The connection to the service came up or went down. A request that was in flight is lost with the connection.
    pub fn set_connected(&mut self, connected: bool, cx: &mut Context<Self>) {
        if self.connected == connected {
            return;
        }
        self.connected = connected;
        if !connected {
            self.pending_use = None;
        }
        cx.notify();
    }

    /// Use was clicked on the card of `folder`: ask the service, show the spinner, and wait for what it broadcasts.
    fn use_theme(&mut self, folder: &str, cx: &mut Context<Self>) {
        let Some(theme) = self.themes.iter().find(|theme| theme.folder == folder) else {
            return;
        };
        if !card::can_use(theme, self.runtime.as_ref(), self.connected) {
            return;
        }
        // The request of this folder is in flight already; a click on another card replaces it (the last click wins).
        if self.pending_use.as_deref() == Some(folder) {
            return;
        }
        tracing::debug!("use requested");
        self.pending_use = Some(folder.to_owned());
        card::send_use(&self.link, folder);
        cx.notify();
    }

    /// The service answered a Use request: only the spinner of the folder it names goes away.
    pub fn on_use_result(&mut self, folder: &str, outcome: &UseOutcome, cx: &mut Context<Self>) {
        if self.pending_use.as_deref() == Some(folder) {
            self.pending_use = None;
        }
        match outcome {
            UseOutcome::Applied => tracing::debug!("use applied"),
            UseOutcome::Refused(refusal) => tracing::warn!(?refusal, "the service refused to use the theme"),
            UseOutcome::Failed { reason } => tracing::warn!(%reason, "the service could not apply the theme"),
        }
        cx.notify();
    }

    /// The service reported its settings (Welcome): remember them and scan the library.
    pub fn load(&mut self, settings: &PortalSettings, cx: &mut Context<Self>) {
        self.settings = Some(LibrarySettings::from_portal(settings));
        self.reload(cx);
    }

    /// The service reported a change: apply what concerns the library.
    pub fn update_settings(&mut self, settings: &PortalSettings, cx: &mut Context<Self>) {
        let new = LibrarySettings::from_portal(settings);
        let Some(old) = self.settings.replace(new.clone()) else {
            return;
        };
        if old.location != new.location {
            self.reload(cx);
            return;
        }
        let resorted = (old.sort_mode, old.sort_kind) != (new.sort_mode, new.sort_kind);
        if resorted {
            library::sort(&mut self.themes, new.sort_mode, new.sort_kind);
            self.page = 1;
        }
        self.page = self
            .page
            .clamp(1, layout::page_count(self.themes.len(), new.pagination).max(1));
        if resorted || old.pagination != new.pagination {
            self.request_page(cx);
        }
        cx.notify();
    }

    /// Scans the library folder again.
    #[allow(dead_code, reason = "the retry button of the error state calls it")]
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(settings) = self.settings.clone() else {
            return;
        };
        self.generation += 1;
        let generation = self.generation;
        if self.phase != Phase::Ready {
            self.phase = Phase::Scanning { skeleton: false };
        }
        let (location, mode, kind) = (settings.location, settings.sort_mode, settings.sort_kind);
        let scan = cx
            .background_executor()
            .spawn(async move { scan_library(&location, mode, kind) });
        self.scan_task = Some(cx.spawn(async move |this, cx| {
            let result = scan.await;
            this.update(cx, |this, cx| this.on_scanned(generation, (mode, kind), result, cx))
                .ok();
        }));
        let timer = cx.background_executor().timer(SKELETON_DELAY);
        self.skeleton_task = Some(cx.spawn(async move |this, cx| {
            timer.await;
            this.update(cx, |this, cx| {
                if this.generation == generation
                    && let Phase::Scanning { skeleton } = &mut this.phase
                {
                    *skeleton = true;
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    fn on_scanned(
        &mut self,
        generation: u64,
        sorted_by: (SortMode, SortKind),
        result: io::Result<Scanned>,
        cx: &mut Context<Self>,
    ) {
        if generation != self.generation {
            return;
        }
        self.skeleton_task = None;
        match result {
            Ok(scanned) => {
                tracing::info!(
                    themes = scanned.themes.len(),
                    corrupt = scanned.corrupt,
                    ignored = scanned.ignored,
                    "library loaded"
                );
                self.themes = scanned.themes;
                self.thumb_times = scanned.thumb_times;
                if let Some(settings) = &self.settings {
                    // The sort settings changed while the scan ran.
                    if (settings.sort_mode, settings.sort_kind) != sorted_by {
                        library::sort(&mut self.themes, settings.sort_mode, settings.sort_kind);
                    }
                    let pages = layout::page_count(self.themes.len(), settings.pagination).max(1);
                    self.page = self.page.clamp(1, pages);
                }
                self.phase = Phase::Ready;
                self.request_page(cx);
            }
            Err(error) => {
                tracing::warn!(%error, "the library folder cannot be read");
                self.themes.clear();
                self.phase = Phase::Failed;
            }
        }
        cx.notify();
    }

    /// Shows page `page` (counted from 1; out of range values are clamped) and scrolls to its top.
    #[allow(dead_code, reason = "the pagination bar of the search and paging plan calls it")]
    pub fn set_page(&mut self, page: usize, cx: &mut Context<Self>) {
        let Some(settings) = &self.settings else {
            return;
        };
        let page = page.clamp(1, layout::page_count(self.themes.len(), settings.pagination).max(1));
        if page == self.page {
            return;
        }
        self.page = page;
        self.scroll.set_offset(point(px(0.), px(0.)));
        self.request_page(cx);
        cx.notify();
    }

    /// Queues the thumbnails of the page that is shown, first card first.
    fn request_page(&mut self, cx: &mut Context<Self>) {
        let Some(settings) = &self.settings else {
            return;
        };
        let range = layout::page_slice(self.themes.len(), self.page, settings.pagination);
        let keys: Vec<ThumbKey> = self.themes[range]
            .iter()
            .take(CACHE_IMAGES)
            .map(|theme| thumb_key(&self.thumb_times, theme, self.bucket))
            .filter(|key| !self.cache.contains(key) && !self.failed.contains(key))
            .collect();
        self.queue.set_wanted(keys);
        self.pump(cx);
    }

    /// Starts decodes while a slot is free.
    fn pump(&mut self, cx: &mut Context<Self>) {
        while let Some(key) = self.queue.next() {
            let (path, bucket) = (key.path.clone(), key.bucket);
            let task = cx
                .background_executor()
                .spawn(async move { decode_thumbnail(&path, bucket) });
            cx.spawn(async move |this, cx| {
                let result = task.await;
                this.update(cx, |this, cx| this.on_decoded(key, result, cx)).ok();
            })
            .detach();
        }
    }

    fn on_decoded(&mut self, key: ThumbKey, result: anyhow::Result<Arc<RenderImage>>, cx: &mut Context<Self>) {
        self.queue.finish(&key);
        match result {
            Ok(image) => {
                for evicted in self.cache.insert(key, image) {
                    // No window is being updated here, so GPUI finds every window itself.
                    cx.drop_image(evicted, None);
                }
            }
            Err(error) => {
                tracing::warn!(%error, "a thumbnail could not be decoded");
                tracing::debug!(path = %key.path.display(), "thumbnail not decoded");
                self.failed.insert(key);
            }
        }
        if self.queue.is_idle() {
            tracing::debug!(
                images = self.cache.len(),
                bytes = self.cache.total_bytes(),
                "thumbnail cache"
            );
        }
        self.pump(cx);
        cx.notify();
    }

    /// The cards of the page that is shown.
    fn cards(&mut self, settings: &LibrarySettings, cx: &mut Context<Self>) -> Vec<CardData> {
        let this = cx.entity().downgrade();
        let range = layout::page_slice(self.themes.len(), self.page, settings.pagination);
        let mut cards = Vec::with_capacity(range.len());
        for index in range {
            let theme = &self.themes[index];
            let key = thumb_key(&self.thumb_times, theme, self.bucket);
            let thumb = match self.cache.get(&key) {
                Some(image) => Thumb::Ready(image),
                None if self.failed.contains(&key) => Thumb::Failed,
                None => Thumb::Loading,
            };
            let folder = theme.folder.clone();
            let on_use_view = this.clone();
            let on_use: Rc<dyn Fn(&mut Window, &mut App)> = Rc::new(move |_, cx| {
                on_use_view.update(cx, |view, cx| view.use_theme(&folder, cx)).ok();
            });
            let selected = self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.running && runtime.selected.as_deref() == Some(&theme.folder));
            cards.push(CardData {
                index,
                title: SharedString::from(library::display_title(&theme.info, &theme.folder).to_owned()),
                description: library::display_description(&theme.info).map(|text| SharedString::from(text.to_owned())),
                thumb,
                needs_newer_app: matches!(theme.compatibility, library::Compatibility::NeedsNewerApp { .. }),
                selected,
                pending: self.pending_use.as_deref() == Some(&theme.folder),
                enabled: card::can_use(theme, self.runtime.as_ref(), self.connected),
                on_use,
            });
        }
        cards
    }
}

impl Render for LibraryView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bucket = Bucket::for_scale(window.scale_factor());
        if bucket != self.bucket {
            self.bucket = bucket;
            self.request_page(cx);
        }
        let Some(settings) = self.settings.clone() else {
            return div().size_full().into_any_element();
        };
        let content_width = (f32::from(window.viewport_size().width) - 2.0 * grid::CONTENT_PADDING).max(0.0);
        match self.phase {
            Phase::Scanning { skeleton: true } => {
                grid::skeleton(content_width, settings.margin, settings.adaptive_layout, &self.scroll)
            }
            Phase::Ready if !self.themes.is_empty() => {
                let cards = self.cards(&settings, cx);
                grid::page(
                    cards,
                    content_width,
                    settings.margin,
                    settings.adaptive_layout,
                    &self.scroll,
                    cx,
                )
            }
            // The empty and error states, with their buttons, come with the search and paging plan.
            _ => div().size_full().into_any_element(),
        }
    }
}
