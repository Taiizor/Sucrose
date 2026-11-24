//! The library page (LIB-04, UI-07): the library folder is scanned on a background thread, sorted like the legacy app and
//! shown one page at a time as theme cards. Thumbnails are decoded at card size on the background executor into a bounded
//! cache; what the cache pushes out is dropped from GPUI's image atlas.

mod card;
mod grid;
pub mod layout;
mod pagination;
pub mod preview;
pub mod thumbs;

use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    io,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::{Duration, SystemTime},
};

use gpui_kit::component::{
    input::{InputEvent, InputState},
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use sucrose_core::{
    library::{self, LibraryTheme},
    paths,
    settings::{SortKind, SortMode},
    version::app_version,
};
use sucrose_ipc::portal::{PortalSettings, RuntimeState, UseOutcome};

use card::{CardData, Thumb};
use preview::{Finish, HOVER_DWELL, PreviewState, load_preview};

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
    /// Whether a hovered card plays its preview (LIB-06).
    preview: bool,
    /// Whether the title strip hides while a preview plays.
    preview_hide: bool,
}

/// The library folder the service reported: its configured location, or the default folder when none is set.
pub fn location_of(settings: &PortalSettings) -> PathBuf {
    LibrarySettings::from_portal(settings).location
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
            preview: settings.library_preview,
            preview_hide: settings.library_preview_hide,
        }
    }
}

/// A preview decode that waits for a free decode slot.
struct PreviewJob {
    folder: String,
    path: PathBuf,
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
    /// The normalized search text; empty shows the whole library.
    query: String,
    /// Indexes into `themes` of what the search lets through, in the order shown across the pages.
    visible: Vec<usize>,
    thumb_times: HashMap<PathBuf, Option<SystemTime>>,
    /// The page shown, counted from 1.
    page: usize,
    cache: ThumbCache<Arc<RenderImage>>,
    queue: DecodeQueue,
    /// Thumbnails that could not be decoded.
    failed: HashSet<ThumbKey>,
    /// The single live hover preview; its frames never enter `cache` (they are not thumbnails).
    preview: PreviewState<Arc<RenderImage>>,
    /// The dwell timer of the card the pointer rests on; dropping it cancels the wait.
    preview_timer: Option<Task<()>>,
    /// The preview decode that waits for a decode slot.
    preview_job: Option<PreviewJob>,
    /// Preview files that could not be decoded, so a card is not tried (and logged) again.
    preview_failed: HashSet<PathBuf>,
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
    /// One focus handle per card position of the page, kept between renders so focus survives a re-render.
    focus: Vec<FocusHandle>,
    /// The jump field of the pagination bar.
    jump: Entity<InputState>,
    /// The page count the jump field limits its digits by.
    jump_total: Rc<Cell<usize>>,
    /// The page the jump field was last set to; the field shows the page that is shown until the user types.
    jump_synced: usize,
    _jump_subscription: Subscription,
    _visibility: Subscription,
}

impl LibraryView {
    /// An empty page; the window's scale factor picks the thumbnail bucket.
    pub fn new(link: ServiceLink, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let jump_total = Rc::new(Cell::new(1));
        let jump = pagination::jump_state(&jump_total, window, cx);
        // Enter in the jump field is Jump.
        let jump_subscription = cx.subscribe_in(&jump, window, |this, _, event: &InputEvent, window, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                this.jump_to_typed(window, cx);
            }
        });
        // A window that is no longer visible shows no preview.
        let this = cx.weak_entity();
        let visibility = window.observe_window_visibility(move |visibility, _, cx| {
            if !visibility.is_visible() {
                this.update(cx, |this, cx| this.stop_preview(cx)).ok();
            }
        });
        Self {
            settings: None,
            phase: Phase::Idle,
            themes: Vec::new(),
            query: String::new(),
            visible: Vec::new(),
            thumb_times: HashMap::new(),
            page: 1,
            cache: ThumbCache::new(CACHE_IMAGES, CACHE_BYTES),
            queue: DecodeQueue::new(DECODE_SLOTS),
            failed: HashSet::new(),
            preview: PreviewState::Idle,
            preview_timer: None,
            preview_job: None,
            preview_failed: HashSet::new(),
            bucket: Bucket::for_scale(window.scale_factor()),
            scroll: ScrollHandle::new(),
            generation: 0,
            scan_task: None,
            skeleton_task: None,
            link,
            runtime: None,
            connected: false,
            pending_use: None,
            focus: Vec::new(),
            jump,
            jump_total,
            jump_synced: 0,
            _jump_subscription: jump_subscription,
            _visibility: visibility,
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
        if !card::use_state(theme, self.runtime.as_ref(), self.connected).is_enabled() {
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
        // Turning Library Preview off ends a playing preview at once; Hide Details is read when the card is drawn, so it
        // applies to the current preview and to the next ones.
        if preview_stopped_by(&old, &new) {
            self.stop_preview(cx);
        }
        if old.location != new.location {
            self.reload(cx);
            return;
        }
        let resorted = (old.sort_mode, old.sort_kind) != (new.sort_mode, new.sort_kind);
        if resorted {
            library::sort(&mut self.themes, new.sort_mode, new.sort_kind);
            self.refilter();
        }
        let page = page_after_settings_change(&old, &new, self.page, self.visible.len());
        let paged_again = resorted || old.pagination != new.pagination;
        if paged_again {
            self.scroll.set_offset(point(px(0.), px(0.)));
        }
        if paged_again || page != self.page {
            self.stop_preview(cx);
            self.page = page;
            self.request_page(cx);
        }
        cx.notify();
    }

    /// The search text changed (already trimmed, lowercased and cut to its length by the header): show what matches, from
    /// page 1. An empty text shows the whole library again.
    pub fn set_query(&mut self, query: &str, cx: &mut Context<Self>) {
        if self.query == query {
            return;
        }
        self.query = query.to_owned();
        self.stop_preview(cx);
        self.refilter();
        self.page = 1;
        self.scroll.set_offset(point(px(0.), px(0.)));
        self.request_page(cx);
        cx.notify();
    }

    /// Lists what the search lets through: the whole library, or the matches in match-count order.
    fn refilter(&mut self) {
        self.visible = library::search(&self.themes, &self.query);
    }

    /// The number of pages the visible themes fill.
    fn pages(&self) -> usize {
        self.settings.as_ref().map_or(0, |settings| {
            layout::page_count(self.visible.len(), settings.pagination)
        })
    }

    /// The jump field was confirmed (Jump or Enter): go to the page it holds. An empty field does nothing.
    fn jump_to_typed(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let text = self.jump.read(cx).value();
        let Some(page) = pagination::clamp_jump(&text, self.pages()) else {
            return;
        };
        // Whatever was typed, the field shows the page that is shown afterwards.
        self.jump_synced = 0;
        self.set_page(page, cx);
        cx.notify();
    }

    /// Scans the library folder again.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(settings) = self.settings.clone() else {
            return;
        };
        self.generation += 1;
        let generation = self.generation;
        self.stop_preview(cx);
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
                // The cards below are new; whatever plays belongs to the old list.
                self.stop_preview(cx);
                self.themes = scanned.themes;
                self.thumb_times = scanned.thumb_times;
                self.preview_failed.clear();
                if let Some(settings) = &self.settings
                    && (settings.sort_mode, settings.sort_kind) != sorted_by
                {
                    // The sort settings changed while the scan ran.
                    library::sort(&mut self.themes, settings.sort_mode, settings.sort_kind);
                }
                self.refilter();
                self.page = self.page.clamp(1, self.pages().max(1));
                self.phase = Phase::Ready;
                self.request_page(cx);
            }
            Err(error) => {
                tracing::warn!(%error, "the library folder cannot be read");
                self.themes.clear();
                self.refilter();
                self.phase = Phase::Failed;
            }
        }
        cx.notify();
    }

    /// Shows page `page` (counted from 1; out of range values are clamped) and scrolls to its top.
    pub fn set_page(&mut self, page: usize, cx: &mut Context<Self>) {
        let page = page.clamp(1, self.pages().max(1));
        if page == self.page {
            return;
        }
        self.stop_preview(cx);
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
        let range = layout::page_slice(self.visible.len(), self.page, settings.pagination);
        tracing::debug!(
            page = self.page,
            pages = layout::page_count(self.visible.len(), settings.pagination),
            "page shown"
        );
        let keys: Vec<ThumbKey> = self.visible[range]
            .iter()
            .take(CACHE_IMAGES)
            .map(|&index| thumb_key(&self.thumb_times, &self.themes[index], self.bucket))
            .filter(|key| !self.cache.contains(key) && !self.failed.contains(key))
            .collect();
        self.queue.set_wanted(keys);
        self.pump(cx);
    }

    /// The pointer entered (`true`) or left (`false`) the card of `folder`: with Library Preview on, a card the pointer
    /// rests on for [`HOVER_DWELL`] plays its preview; leaving releases it.
    fn hover_card(&mut self, folder: &str, hovered: bool, cx: &mut Context<Self>) {
        if !hovered {
            let released = self.preview.leave(folder);
            self.after_preview_change(released, cx);
            return;
        }
        if !self.settings.as_ref().is_some_and(|settings| settings.preview) || self.preview.folder() == Some(folder) {
            return;
        }
        let Some(theme) = self.themes.iter().find(|theme| theme.folder == folder) else {
            return;
        };
        if theme.preview.as_os_str().is_empty() || self.preview_failed.contains(&theme.preview) {
            return;
        }
        // Another card's preview is released before this one starts.
        let released = self.preview.hover(folder);
        let timer = cx.background_executor().timer(HOVER_DWELL);
        let folder = folder.to_owned();
        self.preview_timer = Some(cx.spawn(async move |this, cx| {
            timer.await;
            this.update(cx, |this, cx| this.on_dwell(&folder, cx)).ok();
        }));
        self.after_preview_change(released, cx);
    }

    /// The Portal leaves the library for the settings: a hovered card cannot report that the pointer left, so the preview
    /// stops here. The page, the scroll position and the search text stay as they are.
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.stop_preview(cx);
    }

    /// Stops the preview wherever it stands (waiting, decoding or playing) and releases its frames from GPUI's atlas. A
    /// decode that is still running ends as a stale result and is released too.
    fn stop_preview(&mut self, cx: &mut Context<Self>) {
        let released = self.preview.stop();
        self.after_preview_change(released, cx);
    }

    /// Releases `released` from GPUI's atlas, drops a decode request nothing waits for any more and redraws.
    fn after_preview_change(&mut self, released: Option<Arc<RenderImage>>, cx: &mut Context<Self>) {
        if let Some(image) = released {
            cx.drop_image(image, None);
        }
        if !matches!(self.preview, PreviewState::Waiting { .. }) {
            self.preview_timer = None;
        }
        if !matches!(self.preview, PreviewState::Decoding { .. }) {
            self.preview_job = None;
        }
        cx.notify();
    }

    /// The pointer rested on the card of `folder` for the whole dwell: decode its preview when a slot is free.
    fn on_dwell(&mut self, folder: &str, cx: &mut Context<Self>) {
        if !self.preview.dwell_elapsed(folder) {
            return;
        }
        match self.themes.iter().find(|theme| theme.folder == folder) {
            Some(theme) => {
                self.preview_job = Some(PreviewJob {
                    folder: folder.to_owned(),
                    path: theme.preview.clone(),
                });
                self.pump(cx);
            }
            // The theme went away during the dwell.
            None => {
                let released = self.preview.leave(folder);
                self.after_preview_change(released, cx);
            }
        }
    }

    /// The preview decode of `folder` ended. A result the pointer has moved away from is released at once.
    fn on_preview_decoded(
        &mut self,
        folder: &str,
        path: PathBuf,
        result: anyhow::Result<Arc<RenderImage>>,
        cx: &mut Context<Self>,
    ) {
        self.queue.unreserve();
        let image = match result {
            Ok(image) => Some(image),
            Err(error) => {
                // The card keeps its thumbnail and shows no message (UI-SPEC E2 error row); the path stays out of info.
                tracing::warn!(%error, "a preview could not be decoded");
                tracing::debug!(path = %path.display(), "preview not decoded");
                self.preview_failed.insert(path);
                None
            }
        };
        let stale = match self.preview.finished(folder, image) {
            Finish::Stale(image) => image,
            Finish::Playing | Finish::Failed => None,
        };
        self.after_preview_change(stale, cx);
        self.pump(cx);
    }

    /// Starts decodes while a slot is free: the preview of the hovered card first, then the thumbnails.
    fn pump(&mut self, cx: &mut Context<Self>) {
        if self.preview_job.is_some()
            && self.queue.reserve()
            && let Some(PreviewJob { folder, path }) = self.preview_job.take()
        {
            let bucket = self.bucket;
            let decode_path = path.clone();
            let task = cx
                .background_executor()
                .spawn(async move { load_preview(&decode_path, bucket) });
            cx.spawn(async move |this, cx| {
                let result = task.await;
                this.update(cx, |this, cx| this.on_preview_decoded(&folder, path, result, cx))
                    .ok();
            })
            .detach();
        }
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
    fn cards(&mut self, settings: &LibrarySettings, columns: usize, cx: &mut Context<Self>) -> Vec<CardData> {
        let this = cx.entity().downgrade();
        let range = layout::page_slice(self.visible.len(), self.page, settings.pagination);
        while self.focus.len() < range.len() {
            self.focus.push(cx.focus_handle().tab_stop(true));
        }
        let handles: Rc<[FocusHandle]> = self.focus[..range.len()].iter().cloned().collect();
        let count = range.len();
        let mut cards = Vec::with_capacity(count);
        for (position, slot) in range.enumerate() {
            let index = self.visible[slot];
            let theme = &self.themes[index];
            let key = thumb_key(&self.thumb_times, theme, self.bucket);
            let thumb = match self.cache.get(&key) {
                Some(image) => Thumb::Ready(image),
                None if self.failed.contains(&key) => Thumb::Failed,
                None => Thumb::Loading,
            };
            let folder = theme.folder.clone();
            let hover_folder = theme.folder.clone();
            let on_hover_view = this.clone();
            let on_hover: card::HoverHandler = Rc::new(move |hovered, _, cx| {
                on_hover_view
                    .update(cx, |view, cx| view.hover_card(&hover_folder, hovered, cx))
                    .ok();
            });
            let on_use_view = this.clone();
            let on_use: card::UseHandler = Rc::new(move |_, cx| {
                on_use_view.update(cx, |view, cx| view.use_theme(&folder, cx)).ok();
            });
            let on_arrow: card::ArrowHandler = {
                let handles = Rc::clone(&handles);
                Rc::new(move |arrow, window, cx| {
                    if let Some(next) = layout::neighbor(position, count, columns, arrow) {
                        window.focus(&handles[next], cx);
                    }
                })
            };
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
                state: card::use_state(theme, self.runtime.as_ref(), self.connected),
                focus: handles[position].clone(),
                on_use,
                on_arrow,
                preview: self.preview.image_of(&theme.folder).cloned(),
                preview_hide: settings.preview_hide,
                on_hover,
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
        let content = match self.phase {
            Phase::Scanning { skeleton: true } => {
                grid::skeleton(content_width, settings.margin, settings.adaptive_layout, &self.scroll)
            }
            Phase::Ready | Phase::Failed => {
                let state = grid::library_state(
                    self.phase == Phase::Failed,
                    self.themes.len(),
                    &self.query,
                    self.visible.len(),
                );
                match state {
                    grid::LibraryState::LoadError => {
                        let this = cx.entity().downgrade();
                        grid::load_error(move |_, _, cx| {
                            this.update(cx, |view, cx| view.reload(cx)).ok();
                        })
                    }
                    grid::LibraryState::Empty => grid::empty(),
                    grid::LibraryState::SearchEmpty => grid::search_empty(),
                    grid::LibraryState::Grid => {
                        let columns = layout::columns(content_width, settings.margin, settings.adaptive_layout);
                        let cards = self.cards(&settings, columns, cx);
                        grid::page(
                            cards,
                            content_width,
                            settings.margin,
                            settings.adaptive_layout,
                            &self.scroll,
                            cx,
                        )
                    }
                }
            }
            Phase::Idle | Phase::Scanning { skeleton: false } => div().size_full().into_any_element(),
        };

        // The pagination bar sits under the scrolling grid and shows from two pages on.
        let pages = self.pages();
        self.jump_total.set(pages.max(1));
        let bar = (self.phase == Phase::Ready && pages > 1).then(|| {
            if self.jump_synced != self.page {
                self.jump_synced = self.page;
                let text = SharedString::from(self.page.to_string());
                self.jump.update(cx, |state, cx| state.set_value(text, window, cx));
            }
            let this = cx.entity().downgrade();
            let on_page: pagination::PageHandler = Rc::new({
                let this = this.clone();
                move |page, _, cx| {
                    this.update(cx, |view, cx| view.set_page(page, cx)).ok();
                }
            });
            let on_jump: pagination::JumpHandler = Rc::new(move |window, cx| {
                this.update(cx, |view, cx| view.jump_to_typed(window, cx)).ok();
            });
            pagination::bar(self.page, pages, &self.jump, on_page, on_jump)
        });
        v_flex()
            .size_full()
            .child(div().flex_1().min_h_0().w_full().child(content))
            .when_some(bar, |column, bar| column.child(bar))
            .into_any_element()
    }
}

/// Whether the settings change from `old` to `new` ends the live preview: `Changed(LibraryPreview(false))`. `LibraryPreviewHide`
/// only changes how the strip is drawn.
fn preview_stopped_by(old: &LibrarySettings, new: &LibrarySettings) -> bool {
    old.preview && !new.preview
}

/// The page to show after the settings changed from `old` to `new` with `count` themes loaded: page 1 when the order or the
/// page size changed, otherwise the current page kept inside the page range.
fn page_after_settings_change(old: &LibrarySettings, new: &LibrarySettings, page: usize, count: usize) -> usize {
    let reordered = (old.sort_mode, old.sort_kind) != (new.sort_mode, new.sort_kind);
    if reordered || old.pagination != new.pagination {
        return 1;
    }
    page.clamp(1, layout::page_count(count, new.pagination).max(1))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use sucrose_core::settings::{SortKind, SortMode};

    use super::{LibrarySettings, layout, page_after_settings_change, preview_stopped_by};

    fn settings() -> LibrarySettings {
        LibrarySettings {
            location: PathBuf::from("Library"),
            pagination: 30,
            adaptive_layout: 0,
            margin: 10.0,
            sort_mode: SortMode::Creation,
            sort_kind: SortKind::Descending,
            preview: false,
            preview_hide: false,
        }
    }

    #[test]
    fn preview_turned_off_stops_the_live_preview_and_hide_details_does_not() {
        let on = LibrarySettings {
            preview: true,
            ..settings()
        };
        let off = settings();
        assert!(preview_stopped_by(&on, &off));
        assert!(!preview_stopped_by(&off, &on), "turning it on stops nothing");
        assert!(!preview_stopped_by(&on, &on));
        let hidden = LibrarySettings {
            preview_hide: true,
            ..on.clone()
        };
        assert!(!preview_stopped_by(&on, &hidden));
        assert_ne!(
            on, hidden,
            "a Hide Details change is still a settings change the cards redraw for"
        );
    }

    #[test]
    fn a_new_page_size_returns_to_page_one_with_the_new_page_count() {
        let old = settings();
        let new = LibrarySettings {
            pagination: 10,
            ..settings()
        };
        assert_eq!(layout::page_count(40, old.pagination), 2);
        assert_eq!(layout::page_count(40, new.pagination), 4);
        assert_eq!(page_after_settings_change(&old, &new, 2, 40), 1);
    }

    #[test]
    fn a_new_sort_order_returns_to_page_one() {
        let old = settings();
        for new in [
            LibrarySettings {
                sort_mode: SortMode::Name,
                ..settings()
            },
            LibrarySettings {
                sort_kind: SortKind::Ascending,
                ..settings()
            },
        ] {
            assert_eq!(page_after_settings_change(&old, &new, 2, 40), 1);
        }
    }

    #[test]
    fn a_new_layout_keeps_the_page() {
        let old = settings();
        for new in [
            LibrarySettings {
                adaptive_layout: 3,
                ..settings()
            },
            LibrarySettings {
                margin: 25.0,
                ..settings()
            },
        ] {
            assert_eq!(page_after_settings_change(&old, &new, 2, 40), 2);
        }
    }

    #[test]
    fn an_unchanged_page_is_clamped_into_the_range() {
        let settings = settings();
        assert_eq!(page_after_settings_change(&settings, &settings, 5, 40), 2);
        assert_eq!(page_after_settings_change(&settings, &settings, 1, 0), 1);
    }
}
