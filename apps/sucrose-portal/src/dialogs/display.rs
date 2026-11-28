//! The Display Preferences dialog (UI-SPEC "Display Preferences dialog", E5, D-02, D-07, D-08): the monitor canvas with
//! every connected monitor drawn to scale, the Placement Mode radios, the list of monitors and the assignments of monitors
//! that are not connected. The dialog changes nothing itself (D-11): a tile click, a radio and Remove Wallpaper are
//! requests to the service, and the view shows only what the service broadcast.
//!
//! The model (what the dialog shows for a runtime state) is plain data built by [DisplayModel::build], so every rule of
//! the dialog is tested without a window; [DisplayPreferences] draws it.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use gpui_kit::assets::IconName;
use gpui_kit::base::actions::Confirm;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, StyledExt as _, WindowExt as _,
    button::Button,
    h_flex,
    radio::{Radio, RadioGroup},
    scroll::ScrollableElement as _,
    skeleton::Skeleton,
    spinner::Spinner,
    tooltip::Tooltip,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use sucrose_core::settings::DisplayScreenType;
use sucrose_ipc::portal::{DisplayView, PortalRequest, PortalSettings, RuleState, RuntimeState, SettingChange};

use super::{body, close_footer};
use crate::{
    atlas,
    i18n::{tr, tr_with},
    ipc::ServiceLink,
    library::{
        LibraryView, ThemeBrief,
        thumbs::{Bucket, decode_thumbnail},
    },
};

/// Width of the dialog: a 680 px canvas plus 24 px padding on each side (UI-SPEC).
pub const WIDTH: f32 = 728.0;
/// Width of the monitor canvas.
pub const CANVAS_WIDTH: f32 = 680.0;
/// Height of the monitor canvas.
pub const CANVAS_HEIGHT: f32 = 232.0;
/// The space between the canvas edge and the tiles.
pub const CANVAS_INSET: f32 = 16.0;
/// The smallest a tile gets, however many monitors share the canvas.
pub const MIN_TILE: (f32, f32) = (48.0, 28.0);
/// A tile at least this wide also shows its resolution and, when empty, its caption.
pub const DETAIL_WIDTH: f32 = 120.0;
/// The gap between the sections of the dialog (UI-SPEC: lg).
const SECTION_GAP: f32 = 24.0;
/// The tile corner radius.
const TILE_RADIUS: f32 = 8.0;
/// The height of a row of the monitor list (UI-SPEC).
const ROW_HEIGHT: f32 = 48.0;
/// The most the monitor list takes before it scrolls (UI-SPEC).
const LIST_MAX_HEIGHT: f32 = 200.0;

/// The three placement modes in the order the radios list them.
pub const MODES: [DisplayScreenType; 3] = [
    DisplayScreenType::PerDisplay,
    DisplayScreenType::SpanAcross,
    DisplayScreenType::SameDuplicate,
];

/// The keys of the title and the description of a placement mode.
pub fn mode_keys(mode: DisplayScreenType) -> (&'static str, &'static str) {
    match mode {
        DisplayScreenType::PerDisplay => (
            "Portal.DisplayPreferences.PerDisplay",
            "Portal.DisplayPreferences.PerDisplay.Description",
        ),
        DisplayScreenType::SpanAcross => (
            "Portal.DisplayPreferences.Expand",
            "Portal.DisplayPreferences.Expand.Description",
        ),
        DisplayScreenType::SameDuplicate => (
            "Portal.DisplayPreferences.Duplicate",
            "Portal.DisplayPreferences.Duplicate.Description",
        ),
    }
}

/// A rectangle on the canvas, in canvas pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileRect {
    /// Distance from the left edge of the canvas.
    pub left: f32,
    /// Distance from the top edge of the canvas.
    pub top: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl TileRect {
    /// The right edge.
    pub fn right(&self) -> f32 {
        self.left + self.width
    }

    /// The bottom edge.
    pub fn bottom(&self) -> f32 {
        self.top + self.height
    }
}

/// Draws the monitor rectangles ([left, top, right, bottom] in virtual-desktop pixels) on a canvas of the given size: one
/// uniform factor scales the union of all rectangles to fit inside the inset, and the result is centred. No monitor is
/// special-cased. A tile never gets smaller than [MIN_TILE]; it grows around its centre.
#[allow(
    clippy::cast_precision_loss,
    reason = "monitor sizes are far below the range f32 holds exactly"
)]
pub fn canvas_layout(rects: &[[i32; 4]], width: f32, height: f32, inset: f32) -> Vec<TileRect> {
    if rects.is_empty() {
        return Vec::new();
    }
    let left = rects.iter().map(|rect| rect[0]).min().unwrap_or(0);
    let top = rects.iter().map(|rect| rect[1]).min().unwrap_or(0);
    let right = rects.iter().map(|rect| rect[2]).max().unwrap_or(0);
    let bottom = rects.iter().map(|rect| rect[3]).max().unwrap_or(0);
    let union_width = (right - left) as f32;
    let union_height = (bottom - top) as f32;
    let room_width = (width - 2.0 * inset).max(1.0);
    let room_height = (height - 2.0 * inset).max(1.0);
    let scale = if union_width > 0.0 && union_height > 0.0 {
        (room_width / union_width).min(room_height / union_height)
    } else {
        1.0
    };
    let origin_x = inset + (room_width - union_width * scale) / 2.0;
    let origin_y = inset + (room_height - union_height * scale) / 2.0;
    rects
        .iter()
        .map(|rect| {
            let natural_width = ((rect[2] - rect[0]) as f32) * scale;
            let natural_height = ((rect[3] - rect[1]) as f32) * scale;
            let tile_width = natural_width.max(MIN_TILE.0);
            let tile_height = natural_height.max(MIN_TILE.1);
            TileRect {
                left: origin_x + (rect[0] - left) as f32 * scale - (tile_width - natural_width) / 2.0,
                top: origin_y + (rect[1] - top) as f32 * scale - (tile_height - natural_height) / 2.0,
                width: tile_width,
                height: tile_height,
            }
        })
        .collect()
}

/// The mode the radios show, and whether a change waits for the service (UI-SPEC "Placement mode change").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModeState {
    /// The mode the service last broadcast.
    stored: Option<DisplayScreenType>,
    /// The mode the user asked for and the service has not confirmed.
    pending: Option<DisplayScreenType>,
}

impl ModeState {
    /// The mode the service reported (Welcome or Changed).
    pub fn set_stored(&mut self, mode: DisplayScreenType) {
        self.stored = Some(mode);
        if self.pending == Some(mode) {
            self.pending = None;
        }
    }

    /// The mode the radios show: the one asked for while it waits, else the broadcast one.
    pub fn shown(&self) -> Option<DisplayScreenType> {
        self.pending.or(self.stored)
    }

    /// Whether a change waits for the service (Applying).
    pub fn applying(&self) -> bool {
        self.pending.is_some()
    }

    /// A radio was chosen: whether a request must be sent. The mode already shown asks for nothing.
    pub fn choose(&mut self, mode: DisplayScreenType) -> bool {
        if self.shown() == Some(mode) {
            return false;
        }
        self.pending = Some(mode);
        true
    }

    /// The service refused a mode change: the radios show the broadcast mode again. True when one was waiting.
    pub fn rejected(&mut self) -> bool {
        self.pending.take().is_some()
    }
}

/// What a performance rule did to a wallpaper, as the tile shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleIcon {
    /// A rule paused the wallpaper.
    Paused,
    /// A rule closed the wallpaper.
    Closed,
}

/// One connected monitor on the canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct TileModel {
    /// The stable id of the monitor.
    pub monitor_id: String,
    /// The device name, which the target is stored under.
    pub device_name: String,
    /// The number the monitor shows.
    pub number: u8,
    /// Where it is drawn.
    pub rect: TileRect,
    /// Its size in pixels.
    pub size: (i32, i32),
    /// The folder of the theme assigned to it.
    pub theme: Option<String>,
    /// Whether it is the target of the next Use (Per display only).
    pub selected: bool,
    /// Whether a click sets it as the target (Per display only).
    pub clickable: bool,
    /// The mark of a performance rule on its wallpaper.
    pub rule: Option<RuleIcon>,
}

impl TileModel {
    /// The request a click sends.
    pub fn select_request(&self) -> PortalRequest {
        PortalRequest::Change(SettingChange::ScreenDevice(self.device_name.clone()))
    }
}

/// What a row of the list stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowKey {
    /// One connected monitor, by its stable id.
    Monitor(String),
    /// The single wallpaper of Span and Duplicate.
    All,
}

/// One row of the monitor list.
#[derive(Debug, Clone, PartialEq)]
pub struct RowModel {
    /// What the row stands for.
    pub key: RowKey,
    /// The monitor number; none for the All screens row.
    pub number: Option<u8>,
    /// The device name; empty for the All screens row.
    pub device_name: String,
    /// The monitor size; none for the All screens row.
    pub size: Option<(i32, i32)>,
    /// The folder of the theme on it.
    pub theme: Option<String>,
    /// Whether it is the target of the next Use (Per display only).
    pub selected: bool,
    /// Whether a click sets it as the target (Per display only).
    pub selectable: bool,
    /// Whether Remove Wallpaper is enabled: a wallpaper plays or is assigned there.
    pub can_remove: bool,
}

impl RowModel {
    /// The request Remove Wallpaper sends: for one monitor, or for the single wallpaper.
    pub fn remove_request(&self) -> PortalRequest {
        PortalRequest::RemoveWallpaper {
            monitor_id: match &self.key {
                RowKey::Monitor(id) => Some(id.clone()),
                RowKey::All => None,
            },
        }
    }

    /// The request a click on the row sends: it selects the monitor as the target, unless it already is.
    pub fn select_request(&self) -> Option<PortalRequest> {
        (self.selectable && !self.selected)
            .then(|| PortalRequest::Change(SettingChange::ScreenDevice(self.device_name.clone())))
    }
}

/// The stored assignment of a monitor that is not connected (D-07, D-08).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingModel {
    /// The stable id the assignment is stored under.
    pub monitor_id: String,
    /// The number the monitor showed.
    pub number: u8,
    /// The folder of the theme assigned to it.
    pub theme: Option<String>,
}

impl MissingModel {
    /// The request Remove sends.
    pub fn remove_request(&self) -> PortalRequest {
        PortalRequest::RemoveWallpaper {
            monitor_id: Some(self.monitor_id.clone()),
        }
    }
}

/// Everything the dialog shows, for one runtime state.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayModel {
    /// Nothing has arrived from the service yet: the canvas is a skeleton and the radios are disabled.
    pub loading: bool,
    /// The mode the radios show.
    pub mode: Option<DisplayScreenType>,
    /// Whether a mode change waits for the service.
    pub applying: bool,
    /// The connected monitors, left to right.
    pub tiles: Vec<TileModel>,
    /// The one outline around all tiles in Span.
    pub outline: Option<TileRect>,
    /// The rows of the list.
    pub rows: Vec<RowModel>,
    /// The assignments of monitors that are not connected.
    pub missing: Vec<MissingModel>,
}

impl DisplayModel {
    /// The model of the dialog before the service has reported.
    fn loading(mode: Option<DisplayScreenType>, applying: bool) -> Self {
        Self {
            loading: true,
            mode,
            applying,
            tiles: Vec::new(),
            outline: None,
            rows: Vec::new(),
            missing: Vec::new(),
        }
    }

    /// The model for the runtime state, the stored target device and the mode the radios show.
    pub fn build(
        runtime: Option<&RuntimeState>,
        device: &str,
        mode: Option<DisplayScreenType>,
        applying: bool,
    ) -> Self {
        let Some((runtime, mode)) = runtime.zip(mode) else {
            return Self::loading(mode, applying);
        };
        let per_display = mode == DisplayScreenType::PerDisplay;
        let connected: Vec<&DisplayView> = runtime.displays.iter().filter(|display| display.connected).collect();
        let target = target_index(&connected, device);
        let rects: Vec<[i32; 4]> = connected.iter().map(|display| display.rect).collect();
        let layout = canvas_layout(&rects, CANVAS_WIDTH, CANVAS_HEIGHT, CANVAS_INSET);
        let rule = match runtime.rule {
            RuleState::None => None,
            RuleState::Paused => Some(RuleIcon::Paused),
            RuleState::Closed => Some(RuleIcon::Closed),
        };
        let tiles: Vec<TileModel> = connected
            .iter()
            .zip(&layout)
            .enumerate()
            .map(|(index, (display, rect))| TileModel {
                monitor_id: display.monitor_id.clone(),
                device_name: display.device_name.clone(),
                number: display.number,
                rect: *rect,
                size: (display.rect[2] - display.rect[0], display.rect[3] - display.rect[1]),
                theme: display.theme.clone(),
                selected: per_display && target == Some(index),
                clickable: per_display,
                rule: rule.filter(|_| display.theme.is_some()),
            })
            .collect();
        let outline = (mode == DisplayScreenType::SpanAcross)
            .then(|| union(&layout))
            .flatten();
        let rows = if per_display {
            connected
                .iter()
                .enumerate()
                .map(|(index, display)| RowModel {
                    key: RowKey::Monitor(display.monitor_id.clone()),
                    number: Some(display.number),
                    device_name: display.device_name.clone(),
                    size: Some((display.rect[2] - display.rect[0], display.rect[3] - display.rect[1])),
                    theme: display.theme.clone(),
                    selected: target == Some(index),
                    selectable: true,
                    can_remove: display.theme.is_some(),
                })
                .collect()
        } else if connected.is_empty() {
            Vec::new()
        } else {
            let theme = connected.iter().find_map(|display| display.theme.clone());
            vec![RowModel {
                key: RowKey::All,
                number: None,
                device_name: String::new(),
                size: None,
                can_remove: theme.is_some(),
                theme,
                selected: false,
                selectable: false,
            }]
        };
        let missing = runtime
            .displays
            .iter()
            .filter(|display| !display.connected)
            .map(|display| MissingModel {
                monitor_id: display.monitor_id.clone(),
                number: display.number,
                theme: display.theme.clone(),
            })
            .collect();
        Self {
            loading: false,
            mode: Some(mode),
            applying,
            tiles,
            outline,
            rows,
            missing,
        }
    }
}

/// The index of the target monitor among the connected ones: the one whose device name is stored, else the first (an
/// empty or unknown value auto-chooses the first screen, D-02).
fn target_index(connected: &[&DisplayView], device: &str) -> Option<usize> {
    if connected.is_empty() {
        return None;
    }
    connected
        .iter()
        .position(|display| display.device_name.eq_ignore_ascii_case(device))
        .or(Some(0))
}

/// The rectangle that holds all of the given ones.
fn union(rects: &[TileRect]) -> Option<TileRect> {
    let first = rects.first()?;
    let (mut left, mut top, mut right, mut bottom) = (first.left, first.top, first.right(), first.bottom());
    for rect in rects {
        left = left.min(rect.left);
        top = top.min(rect.top);
        right = right.max(rect.right());
        bottom = bottom.max(rect.bottom());
    }
    Some(TileRect {
        left,
        top,
        width: right - left,
        height: bottom - top,
    })
}

/// How a lookup finds the title and the thumbnail of the theme in a folder.
type ThemeLookup = Rc<dyn Fn(&str, &App) -> Option<ThemeBrief>>;

/// Where the decode of a tile thumbnail stands.
enum ThumbState {
    /// The thumbnail is being decoded on a background thread.
    Loading,
    /// The decoded thumbnail, at tile size.
    Ready(Arc<RenderImage>),
    /// It cannot be decoded: the tile shows the card fallback.
    Failed,
}

/// What a tile shows where its thumbnail goes.
enum TileThumb {
    /// The monitor has no wallpaper.
    Empty,
    /// The decoded thumbnail.
    Ready(Arc<RenderImage>),
    /// Waiting for the decode.
    Loading,
    /// The theme is unknown or its thumbnail cannot be decoded.
    Failed,
}

/// Held by the open dialog: when the dialog is dropped, the thumbnails it decoded are released.
struct CloseGuard;

/// The dialog's state: what the service reported, the thumbnails of the tiles and the focus handles of its parts.
pub struct DisplayPreferences {
    /// Sends the requests.
    link: ServiceLink,
    /// Finds the title and the thumbnail of a theme in the library.
    lookup: ThemeLookup,
    /// The placement mode the radios show.
    mode: ModeState,
    /// The device name of the target monitor, as the service stored it.
    device: String,
    /// What every monitor shows, from the latest runtime state; none before the first report.
    runtime: Option<RuntimeState>,
    /// The tile thumbnails by file; only while the dialog is open.
    thumbs: HashMap<PathBuf, ThumbState>,
    /// The running decodes; dropped with the dialog.
    tasks: Vec<Task<()>>,
    /// Whether the dialog is open.
    open: bool,
    /// The width the thumbnails are decoded at, from the window scale factor.
    bucket: Bucket,
    /// One focus handle per tile, for the arrow keys.
    tile_focus: Vec<FocusHandle>,
    /// One focus handle per row of the list.
    row_focus: Vec<FocusHandle>,
    /// The group of radios, which has the focus when the dialog opens.
    radio_focus: FocusHandle,
    /// The focus moves to the first radio once the dialog has been drawn.
    focus_radios: bool,
    /// Asks for the thumbnails again when the library finds a theme; dropped with the model.
    _library: Option<Subscription>,
}

impl DisplayPreferences {
    /// An empty dialog model that finds themes in the library: it shows the loading state until the service reports.
    pub fn new(link: ServiceLink, library: Entity<LibraryView>, cx: &mut Context<Self>) -> Self {
        let lookup: ThemeLookup = {
            let library = library.clone();
            Rc::new(move |folder, cx| library.read(cx).theme_brief(folder))
        };
        let mut model = Self::with_lookup(link, lookup, cx);
        model._library = Some(cx.observe(&library, |this, _, cx| this.request_thumbnails(cx)));
        model
    }

    /// An empty dialog model with its own way to find themes.
    fn with_lookup(link: ServiceLink, lookup: ThemeLookup, cx: &mut Context<Self>) -> Self {
        Self {
            link,
            lookup,
            mode: ModeState::default(),
            device: String::new(),
            runtime: None,
            thumbs: HashMap::new(),
            tasks: Vec::new(),
            open: false,
            bucket: Bucket::W360,
            tile_focus: Vec::new(),
            row_focus: Vec::new(),
            radio_focus: cx.focus_handle(),
            focus_radios: false,
            _library: None,
        }
    }

    /// The Welcome of the service: the placement settings and what the monitors show.
    pub fn load(&mut self, settings: &PortalSettings, runtime: &RuntimeState, cx: &mut Context<Self>) {
        self.set_placement(settings.display_screen_type, settings.screen_device.clone(), cx);
        self.set_runtime(runtime.clone(), cx);
    }

    /// The placement settings as the service stored them.
    pub fn set_placement(&mut self, mode: DisplayScreenType, device: String, cx: &mut Context<Self>) {
        self.mode.set_stored(mode);
        self.device = device;
        cx.notify();
    }

    /// What every monitor shows changed.
    pub fn set_runtime(&mut self, runtime: RuntimeState, cx: &mut Context<Self>) {
        self.runtime = Some(runtime);
        self.request_thumbnails(cx);
        cx.notify();
    }

    /// The service applied a setting.
    pub fn on_setting(&mut self, change: &SettingChange, cx: &mut Context<Self>) {
        match change {
            SettingChange::DisplayScreenType(mode) => {
                self.mode.set_stored(*mode);
                cx.notify();
            }
            SettingChange::ScreenDevice(device) => {
                self.device.clone_from(device);
                cx.notify();
            }
            _ => {}
        }
    }

    /// The service refused a setting. A refused placement change (mode or target) is the dialog's: the radios show the
    /// broadcast mode again, and the result is true so the caller tells the user. Any other change is not the dialog's.
    pub fn on_rejected(&mut self, change: &SettingChange, cx: &mut Context<Self>) -> bool {
        match change {
            SettingChange::DisplayScreenType(_) => {
                self.mode.rejected();
                cx.notify();
                true
            }
            SettingChange::ScreenDevice(_) => true,
            _ => false,
        }
    }

    /// The model of what the dialog shows now.
    pub fn model(&self) -> DisplayModel {
        DisplayModel::build(
            self.runtime.as_ref(),
            &self.device,
            self.mode.shown(),
            self.mode.applying(),
        )
    }

    /// A radio was chosen: the request goes to the service and the radios wait for its broadcast.
    fn choose_mode(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(mode) = MODES.get(index).copied() else {
            return;
        };
        if self.mode.choose(mode) {
            self.link
                .send(PortalRequest::Change(SettingChange::DisplayScreenType(mode)));
            cx.notify();
        }
    }

    /// The dialog opened: thumbnails are decoded for the tiles from now until it closes.
    fn on_opened(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.open = true;
        self.focus_radios = true;
        self.bucket = Bucket::for_scale(window.scale_factor());
        self.request_thumbnails(cx);
        cx.notify();
    }

    /// The dialog closed: the decodes stop and every thumbnail leaves the atlas (docs/GPUI.md).
    fn on_closed(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        self.tasks.clear();
        let images = self.thumbs.drain().filter_map(|(_, state)| match state {
            ThumbState::Ready(image) => Some(image),
            ThumbState::Loading | ThumbState::Failed => None,
        });
        atlas::release_images(cx, images);
    }

    /// The thumbnail files of the connected monitors' themes.
    fn wanted_thumbnails(&self, cx: &App) -> Vec<PathBuf> {
        let Some(runtime) = &self.runtime else {
            return Vec::new();
        };
        let mut wanted: Vec<PathBuf> = Vec::new();
        for display in runtime.displays.iter().filter(|display| display.connected) {
            let brief = display.theme.as_deref().and_then(|folder| (self.lookup)(folder, cx));
            if let Some(brief) = brief
                && !wanted.contains(&brief.thumbnail)
            {
                wanted.push(brief.thumbnail);
            }
        }
        wanted
    }

    /// Starts the decode of every tile thumbnail that is not decoded yet, off the UI thread, and lets go of the ones no
    /// tile needs any more. Nothing happens while the dialog is closed.
    fn request_thumbnails(&mut self, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        let wanted = self.wanted_thumbnails(cx);
        let stale: Vec<PathBuf> = self
            .thumbs
            .keys()
            .filter(|path| !wanted.contains(path))
            .cloned()
            .collect();
        let released = stale.into_iter().filter_map(|path| match self.thumbs.remove(&path) {
            Some(ThumbState::Ready(image)) => Some(image),
            _ => None,
        });
        atlas::release_images(cx, released.collect::<Vec<_>>());
        for path in wanted {
            if self.thumbs.contains_key(&path) {
                continue;
            }
            self.thumbs.insert(path.clone(), ThumbState::Loading);
            let bucket = self.bucket;
            let task = cx.spawn(async move |this, cx| {
                let file = path.clone();
                let decoded = cx
                    .background_executor()
                    .spawn(async move { decode_thumbnail(&file, bucket) })
                    .await;
                this.update(cx, |this, cx| this.on_decoded(&path, decoded, cx)).ok();
            });
            self.tasks.push(task);
        }
    }

    /// A background decode finished. A result nobody waits for any more (the dialog closed, the tile changed) is released
    /// at once.
    fn on_decoded(&mut self, path: &Path, decoded: anyhow::Result<Arc<RenderImage>>, cx: &mut Context<Self>) {
        let waiting = self.open && matches!(self.thumbs.get(path), Some(ThumbState::Loading));
        match decoded {
            Ok(image) if waiting => {
                self.thumbs.insert(path.to_owned(), ThumbState::Ready(image));
            }
            Ok(image) => atlas::release_image(cx, image),
            Err(error) => {
                tracing::warn!(%error, "a monitor thumbnail cannot be decoded");
                if waiting {
                    self.thumbs.insert(path.to_owned(), ThumbState::Failed);
                }
            }
        }
        cx.notify();
    }

    /// What the tile of the monitor that plays folder shows.
    fn tile_thumb(&self, folder: Option<&str>, cx: &App) -> TileThumb {
        let Some(folder) = folder else {
            return TileThumb::Empty;
        };
        let Some(brief) = (self.lookup)(folder, cx) else {
            return TileThumb::Failed;
        };
        match self.thumbs.get(&brief.thumbnail) {
            Some(ThumbState::Ready(image)) => TileThumb::Ready(image.clone()),
            Some(ThumbState::Failed) => TileThumb::Failed,
            Some(ThumbState::Loading) | None => TileThumb::Loading,
        }
    }

    /// The title of the theme in folder: its title from the library, else the folder name.
    fn theme_title(&self, folder: &str, cx: &App) -> String {
        (self.lookup)(folder, cx).map_or_else(|| folder.to_owned(), |brief| brief.title)
    }

    /// Keeps one focus handle per tile and per row.
    fn sync_focus(&mut self, tiles: usize, rows: usize, cx: &mut Context<Self>) {
        while self.tile_focus.len() < tiles {
            self.tile_focus.push(cx.focus_handle());
        }
        self.tile_focus.truncate(tiles);
        while self.row_focus.len() < rows {
            self.row_focus.push(cx.focus_handle());
        }
        self.row_focus.truncate(rows);
    }

    /// The first thumbnail that is decoded, for the tests.
    #[cfg(test)]
    fn ready_images(&self) -> Vec<Arc<RenderImage>> {
        self.thumbs
            .values()
            .filter_map(|state| match state {
                ThumbState::Ready(image) => Some(image.clone()),
                ThumbState::Loading | ThumbState::Failed => None,
            })
            .collect()
    }
}

/// Opens Display Preferences over the window. The thumbnails of the tiles are decoded while it is open and released when
/// it closes; the focus lands on the Placement Mode radios, and the dialog layer gives it back to the element that had it.
pub fn open_display_preferences(display: &Entity<DisplayPreferences>, window: &mut Window, cx: &mut App) {
    display.update(cx, |display, cx| display.on_opened(window, cx));
    let guard = watch_close(display, cx);
    let content = display.clone();
    window.open_dialog(cx, move |dialog, _, _| {
        // The guard lives as long as the dialog's builder does.
        let _ = &guard;
        dialog
            .title(tr("Portal.DisplayPreferences.Title"))
            .w(px(WIDTH))
            .child(content.clone())
            .footer(close_footer(tr("Portal.DisplayPreferences.Close")))
    });
    let radios = display.read(cx).radio_focus.clone();
    window.focus(&radios, cx);
}

/// A guard whose release tells the model that its dialog is gone, however the dialog was closed (Close, Escape, a click
/// outside).
fn watch_close(display: &Entity<DisplayPreferences>, cx: &mut App) -> Entity<CloseGuard> {
    let display = display.downgrade();
    cx.new(move |cx| {
        cx.on_release(move |_, cx| {
            display.update(cx, |display, cx| display.on_closed(cx)).ok();
        })
        .detach();
        CloseGuard
    })
}

impl Render for DisplayPreferences {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model();
        self.sync_focus(model.tiles.len(), model.rows.len(), cx);
        if self.focus_radios && !model.loading {
            // The radios are tab stops once this frame is drawn: the focus moves from the group to its first radio then.
            self.focus_radios = false;
            let group = self.radio_focus.clone();
            window.on_next_frame(move |window, cx| {
                window.focus(&group, cx);
                window.focus_next(cx);
            });
        }
        let mut sections: Vec<AnyElement> = vec![self.canvas(&model, cx), self.placement(&model, cx)];
        if !model.rows.is_empty() {
            sections.push(self.list(&model, cx));
        }
        if !model.missing.is_empty() {
            sections.push(self.not_connected(&model, cx));
        }
        body(
            window,
            [v_flex().gap(px(SECTION_GAP)).children(sections).into_any_element()],
        )
    }
}

impl DisplayPreferences {
    /// The monitor canvas: one tile per connected monitor, or one skeleton while the service has not reported.
    fn canvas(&self, model: &DisplayModel, cx: &mut Context<Self>) -> AnyElement {
        let surface = cx.theme().secondary.opacity(0.55);
        let frame = div()
            .id("display-canvas")
            .relative()
            .flex_none()
            .w(px(CANVAS_WIDTH))
            .h(px(CANVAS_HEIGHT))
            .rounded(px(TILE_RADIUS))
            .bg(surface)
            .overflow_hidden();
        if model.loading {
            return frame
                .child(Skeleton::new().size_full().rounded(px(TILE_RADIUS)))
                .into_any_element();
        }
        let accent = cx.theme().primary;
        let tiles: Vec<AnyElement> = model
            .tiles
            .iter()
            .enumerate()
            .map(|(index, tile)| self.tile(index, tile, cx))
            .collect();
        frame
            .children(tiles)
            .when_some(model.outline, |frame, rect| {
                frame.child(
                    div()
                        .absolute()
                        .left(px(rect.left - 2.))
                        .top(px(rect.top - 2.))
                        .w(px(rect.width + 4.))
                        .h(px(rect.height + 4.))
                        .rounded(px(TILE_RADIUS + 2.))
                        .border_2()
                        .border_color(accent),
                )
            })
            .into_any_element()
    }

    /// One monitor tile.
    fn tile(&self, index: usize, tile: &TileModel, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let (border, secondary, muted, accent) = (theme.border, theme.secondary, theme.muted_foreground, theme.primary);
        let wide = tile.rect.width >= DETAIL_WIDTH;
        let label = tr_with(
            "Portal.DisplayPreferences.Monitor",
            &[("number", &tile.number.to_string()), ("device", &tile.device_name)],
        );
        let ring = accent.opacity(0.55);
        let handle = self.tile_focus.get(index).cloned();
        let handles = self.tile_focus.clone();
        let link = self.link.clone();
        let request = (tile.clickable && !tile.selected).then(|| tile.select_request());
        let layer = match self.tile_thumb(tile.theme.as_deref(), cx) {
            TileThumb::Ready(image) => img(image)
                .absolute()
                .inset_0()
                .size_full()
                .object_fit(ObjectFit::Cover)
                .into_any_element(),
            TileThumb::Loading => Skeleton::new().absolute().inset_0().size_full().into_any_element(),
            TileThumb::Failed => div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .bg(secondary)
                .child(Icon::new(IconName::ImageOff).with_size(px(24.)).text_color(muted))
                .into_any_element(),
            TileThumb::Empty => v_flex()
                .absolute()
                .inset_0()
                .items_center()
                .justify_center()
                .gap_1()
                .bg(secondary)
                .text_color(muted)
                .child(Icon::new(IconName::Monitor).with_size(px(24.)))
                .when(wide, |column| {
                    column.child(div().text_xs().child(tr("Portal.DisplayPreferences.WindowsWallpaper")))
                })
                .into_any_element(),
        };
        let number = h_flex()
            .absolute()
            .top_1()
            .left_1()
            .h(px(20.))
            .px_2()
            .gap_1()
            .items_center()
            .rounded_full()
            .bg(black().opacity(0.55))
            .text_color(white())
            .text_size(px(12.))
            .when(wide, |badge| {
                badge.child(Icon::new(IconName::Monitor).with_size(px(12.)))
            })
            .child(tile.number.to_string());
        let resolution = div()
            .absolute()
            .left_0()
            .right_0()
            .bottom_0()
            .h(px(24.))
            .flex()
            .items_end()
            .px_2()
            .pb_1()
            .bg(linear_gradient(
                180.,
                linear_color_stop(black().opacity(0.0), 0.),
                linear_color_stop(black().opacity(0.75), 1.),
            ))
            .text_color(white())
            .text_size(px(12.))
            .child(resolution_text(tile.size));
        let rule = tile.rule.map(|rule| {
            div()
                .absolute()
                .top_1()
                .right_1()
                .size(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(black().opacity(0.55))
                .text_color(white())
                .child(
                    Icon::new(match rule {
                        RuleIcon::Paused => IconName::Pause,
                        RuleIcon::Closed => IconName::Power,
                    })
                    .with_size(px(12.)),
                )
        });
        div()
            .id(("display-tile", index))
            .debug_selector(move || format!("display-tile-{index}"))
            .absolute()
            .left(px(tile.rect.left))
            .top(px(tile.rect.top))
            .w(px(tile.rect.width))
            .h(px(tile.rect.height))
            .rounded(px(TILE_RADIUS))
            .border_1()
            .border_color(border)
            .overflow_hidden()
            .when(tile.clickable, |tile_div| {
                tile_div
                    .when_some(handle, |tile_div, handle| tile_div.track_focus(&handle))
                    .focus_visible(move |style| style.border_2().border_color(accent))
                    .role(Role::Button)
                    .aria_label(label)
                    .cursor_pointer()
                    .hover(move |style| style.border_2().border_color(ring))
                    // A click, and Space on the focused tile (GPUI turns it into a click), set the target.
                    .on_click({
                        let (link, request) = (link.clone(), request.clone());
                        move |_, _, _| {
                            if let Some(request) = &request {
                                link.send(request.clone());
                            }
                        }
                    })
                    // The dialog binds Enter to its own confirmation, which closes it: the focused tile takes Enter first.
                    .on_action(move |_: &Confirm, _, _| {
                        if let Some(request) = &request {
                            link.send(request.clone());
                        }
                    })
                    // Left and right move between tiles in left-to-right order.
                    .on_key_down(move |event, window, cx| {
                        let next = match event.keystroke.key.as_str() {
                            "left" => index.checked_sub(1),
                            "right" => Some(index + 1).filter(|next| *next < handles.len()),
                            _ => return,
                        };
                        cx.stop_propagation();
                        if let Some(handle) = next.and_then(|next| handles.get(next)) {
                            window.focus(handle, cx);
                        }
                    })
            })
            .child(layer)
            .child(number)
            .when(wide, |tile_div| tile_div.child(resolution))
            .when_some(rule, |tile_div, rule| tile_div.child(rule))
            .when(tile.selected, |tile_div| {
                tile_div.child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(px(TILE_RADIUS))
                        .border_2()
                        .border_color(accent),
                )
            })
            .into_any_element()
    }

    /// The Placement Mode card: its title and description, the Applying indicator and the three radios.
    fn placement(&self, model: &DisplayModel, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let (border, surface, muted, accent) = (
            theme.border,
            theme.secondary.opacity(0.55),
            theme.muted_foreground,
            theme.primary,
        );
        let this = cx.entity();
        let selected = model
            .mode
            .and_then(|mode| MODES.iter().position(|known| *known == mode));
        let radios = MODES.iter().enumerate().map(|(index, mode)| {
            let (title, description) = mode_keys(*mode);
            Radio::new(("placement-mode", index))
                .debug_selector(move || format!("display-radio-{index}"))
                .accessibility_label(tr(title))
                .h(px(56.))
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().text_sm().font_semibold().child(tr(title)))
                        .child(div().text_xs().text_color(muted).child(tr(description))),
                )
        });
        let applying = model.applying.then(|| {
            h_flex()
                .flex_none()
                .gap_2()
                .items_center()
                .text_xs()
                .text_color(muted)
                .child(Spinner::new().with_size(px(16.)).color(accent))
                .child(tr("Portal.DisplayPreferences.Applying"))
        });
        v_flex()
            .gap_3()
            .p_4()
            .rounded(px(TILE_RADIUS))
            .border_1()
            .border_color(border)
            .bg(surface)
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .child(tr("Portal.DisplayPreferences.Expander")),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted)
                                    .child(tr("Portal.DisplayPreferences.Expander.Description")),
                            ),
                    )
                    .children(applying),
            )
            .child(
                div().id("placement-radios").track_focus(&self.radio_focus).child(
                    RadioGroup::new("placement-mode")
                        .children(radios)
                        .selected_index(selected)
                        .disabled(model.loading)
                        .w_full()
                        .on_click(move |index, _, cx| {
                            this.update(cx, |this, cx| this.choose_mode(*index, cx));
                        }),
                ),
            )
            .into_any_element()
    }

    /// The monitor list: one row per connected monitor in Per display, one All screens row in Span and Duplicate.
    fn list(&self, model: &DisplayModel, cx: &App) -> AnyElement {
        let rows: Vec<AnyElement> = model
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| self.row(index, row, cx))
            .collect();
        v_flex()
            .id("display-list")
            .gap_1()
            .max_h(px(LIST_MAX_HEIGHT))
            .children(rows)
            .overflow_y_scrollbar()
            .into_any_element()
    }

    /// One row of the list.
    fn row(&self, index: usize, row: &RowModel, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let (muted, accent, hover, secondary) = (
            theme.muted_foreground,
            theme.primary,
            theme.secondary.opacity(0.55),
            theme.secondary,
        );
        let name = match row.number {
            Some(number) => tr_with(
                "Portal.DisplayPreferences.Monitor",
                &[("number", &number.to_string()), ("device", &row.device_name)],
            ),
            None => tr("Portal.DisplayPreferences.AllScreens"),
        };
        let wallpaper = match row.theme.as_deref() {
            Some(folder) => self.theme_title(folder, cx),
            None => tr("Portal.DisplayPreferences.NoWallpaper").to_string(),
        };
        let detail = match row.size {
            Some(size) => format!("{}, {wallpaper}", resolution_text(size)),
            None => wallpaper,
        };
        let link = self.link.clone();
        let remove = row.remove_request();
        let select = row.select_request();
        let handle = self.row_focus.get(index).cloned();
        let tip = SharedString::from(format!("{name}: {detail}"));
        let text = v_flex()
            .id(("display-row-text", index))
            .flex_1()
            .min_w_0()
            .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
            .child(div().text_sm().font_semibold().truncate().child(name.clone()))
            .child(div().text_xs().text_color(muted).truncate().child(detail));
        let badge = row.number.map(|number| {
            div()
                .flex_none()
                .h(px(20.))
                .min_w(px(20.))
                .px_2()
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(secondary)
                .text_size(px(12.))
                .child(number.to_string())
        });
        let can_remove = row.can_remove;
        h_flex()
            .id(("display-row", index))
            .debug_selector(move || format!("display-row-{index}"))
            .w_full()
            .h(px(ROW_HEIGHT))
            .flex_none()
            .gap_3()
            .items_center()
            .rounded(px(TILE_RADIUS))
            .when(select.is_some(), |row_div| {
                row_div
                    .when_some(handle, |row_div, handle| row_div.track_focus(&handle))
                    .focus_visible(move |style| style.bg(hover))
                    .hover(move |style| style.bg(hover))
                    .cursor_pointer()
                    .role(Role::Button)
                    .aria_label(name)
                    .on_click({
                        let (link, select) = (link.clone(), select.clone());
                        move |_, _, _| {
                            if let Some(request) = &select {
                                link.send(request.clone());
                            }
                        }
                    })
                    // Enter on the focused row selects it instead of confirming (and closing) the dialog.
                    .on_action({
                        let link = link.clone();
                        move |_: &Confirm, _, _| {
                            if let Some(request) = &select {
                                link.send(request.clone());
                            }
                        }
                    })
            })
            // The 3 px accent indicator at the start edge marks the target row.
            .child(
                div()
                    .flex_none()
                    .w(px(3.))
                    .h(px(32.))
                    .rounded_full()
                    .when(row.selected, |indicator| indicator.bg(accent)),
            )
            .children(badge)
            .child(text)
            .child(
                // The button is its own click: it must not also select the row.
                div()
                    .flex_none()
                    .pr_2()
                    .debug_selector(move || format!("display-remove-{index}"))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        Button::new(("display-remove", index))
                            .outline()
                            .label(tr("Portal.DisplayPreferences.Remove"))
                            .disabled(!can_remove)
                            .on_click(move |_, _, _| link.send(remove.clone())),
                    ),
            )
            .into_any_element()
    }

    /// The Not connected section: the stored assignments of monitors that are absent, each with a Remove button.
    fn not_connected(&self, model: &DisplayModel, cx: &App) -> AnyElement {
        let muted = cx.theme().muted_foreground;
        let rows: Vec<AnyElement> = model
            .missing
            .iter()
            .enumerate()
            .map(|(index, missing)| {
                let wallpaper = missing
                    .theme
                    .as_deref()
                    .map(|folder| self.theme_title(folder, cx))
                    .unwrap_or_default();
                let link = self.link.clone();
                let remove = missing.remove_request();
                h_flex()
                    .id(("display-missing", index))
                    .w_full()
                    .h(px(ROW_HEIGHT))
                    .flex_none()
                    .gap_3()
                    .items_center()
                    .child(div().flex_none().w(px(3.)).h(px(32.)))
                    .child(Icon::new(IconName::MonitorOff).text_color(muted))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(div().text_sm().truncate().child(tr_with(
                                "Portal.DisplayPreferences.NotConnected",
                                &[("number", &missing.number.to_string())],
                            )))
                            .child(div().text_xs().text_color(muted).truncate().child(wallpaper)),
                    )
                    .child(
                        div().flex_none().pr_2().child(
                            Button::new(("display-missing-remove", index))
                                .debug_selector(move || format!("display-missing-remove-{index}"))
                                .outline()
                                .label(tr("Portal.DisplayPreferences.Remove"))
                                .on_click(move |_, _, _| link.send(remove.clone())),
                        ),
                    )
                    .into_any_element()
            })
            .collect();
        v_flex()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(tr("Portal.DisplayPreferences.NotConnectedTitle")),
            )
            .child(v_flex().gap_1().children(rows))
            .into_any_element()
    }
}

/// The size of a monitor as the dialog writes it: "1920 x 1080".
fn resolution_text(size: (i32, i32)) -> SharedString {
    tr_with(
        "Portal.DisplayPreferences.Resolution",
        &[("width", &size.0.to_string()), ("height", &size.1.to_string())],
    )
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

    use gpui_kit::base::Root;
    use gpui_kit::component::{WindowExt as _, v_flex};
    use gpui_kit::{
        AppContext as _, Context, Entity, IntoElement, Modifiers, ParentElement as _, Render, Styled as _,
        TestAppContext, VisualTestContext, Window, div, px, size,
    };
    use sucrose_core::settings::{DisplayScreenType, EngineType};
    use sucrose_ipc::portal::{DisplayView, PortalRequest, RuleState, RuntimeState, SettingChange};

    use super::{
        CANVAS_HEIGHT, CANVAS_INSET, CANVAS_WIDTH, DisplayModel, DisplayPreferences, MIN_TILE, ModeState, RowKey,
        ThemeLookup, TileThumb, canvas_layout, open_display_preferences,
    };
    use crate::{
        header::Header,
        ipc::ServiceLink,
        library::{LibraryView, ThemeBrief},
        settings::Mode,
    };

    const DISPLAY1: &str = "\\\\.\\DISPLAY1";
    const DISPLAY9: &str = "\\\\.\\DISPLAY9";

    fn display(device: &str, number: u8, rect: [i32; 4], theme: Option<&str>) -> DisplayView {
        DisplayView {
            monitor_id: format!("id-{number}"),
            device_name: device.to_owned(),
            number,
            rect,
            primary: number == 1,
            connected: true,
            theme: theme.map(str::to_owned),
            running: theme.is_some(),
            engine: theme.map(|_| EngineType::MpvPlayerLive),
        }
    }

    fn missing(number: u8, theme: &str) -> DisplayView {
        DisplayView {
            connected: false,
            rect: [0; 4],
            running: false,
            engine: None,
            ..display("\\\\.\\DISPLAY3", number, [0; 4], Some(theme))
        }
    }

    fn runtime(displays: Vec<DisplayView>) -> RuntimeState {
        RuntimeState {
            selected: None,
            running: false,
            rule: RuleState::None,
            library_moving: false,
            tray_visible: false,
            running_engines: Vec::new(),
            displays,
        }
    }

    /// DISPLAY1 playing "a" and DISPLAY9 empty, side by side.
    fn two() -> RuntimeState {
        runtime(vec![
            display(DISPLAY1, 1, [0, 0, 1920, 1080], Some("a")),
            display(DISPLAY9, 9, [1920, 0, 3840, 1080], None),
        ])
    }

    #[test]
    fn display_canvas_two_equal_monitors_give_two_equal_tiles_side_by_side_and_centred() {
        let tiles = canvas_layout(
            &[[0, 0, 1920, 1080], [1920, 0, 3840, 1080]],
            CANVAS_WIDTH,
            CANVAS_HEIGHT,
            CANVAS_INSET,
        );
        assert_eq!(tiles.len(), 2);
        assert!((tiles[0].width - tiles[1].width).abs() < 0.001);
        assert!((tiles[0].height - tiles[1].height).abs() < 0.001);
        assert!((tiles[0].top - tiles[1].top).abs() < 0.001);
        assert!(
            (tiles[0].right() - tiles[1].left).abs() < 0.001,
            "no gap between touching monitors"
        );
        // One factor for both axes: the tile keeps the 16:9 proportion of the monitor.
        assert!((tiles[0].width / tiles[0].height - 1920.0 / 1080.0).abs() < 0.001);
        // Centred: the same room on both sides and above and below.
        let left = tiles[0].left;
        let right = CANVAS_WIDTH - tiles[1].right();
        assert!((left - right).abs() < 0.001, "{left} against {right}");
        let top = tiles[0].top;
        let bottom = CANVAS_HEIGHT - tiles[0].bottom();
        assert!((top - bottom).abs() < 0.001, "{top} against {bottom}");
        assert!(left >= CANVAS_INSET - 0.001 && top >= CANVAS_INSET - 0.001);
    }

    #[test]
    fn display_canvas_scales_all_monitors_by_one_factor_whatever_their_size_and_position() {
        // A 1080p monitor, a 4K monitor above it and a portrait one to the right.
        let rects = [[0, 1080, 1920, 2160], [0, 0, 3840, 1080], [3840, 0, 4920, 1920]];
        let tiles = canvas_layout(&rects, CANVAS_WIDTH, CANVAS_HEIGHT, CANVAS_INSET);
        let factor = tiles[0].width / 1920.0;
        for (tile, rect) in tiles.iter().zip(rects) {
            assert!((tile.width - (rect[2] - rect[0]) as f32 * factor).abs() < 0.01);
            assert!((tile.height - (rect[3] - rect[1]) as f32 * factor).abs() < 0.01);
            assert!(tile.left >= CANVAS_INSET - 0.01 && tile.right() <= CANVAS_WIDTH - CANVAS_INSET + 0.01);
            assert!(tile.top >= CANVAS_INSET - 0.01 && tile.bottom() <= CANVAS_HEIGHT - CANVAS_INSET + 0.01);
        }
    }

    #[test]
    fn display_canvas_one_monitor_fills_the_room_and_no_monitor_gives_no_tile() {
        let one = canvas_layout(&[[0, 0, 1920, 1080]], CANVAS_WIDTH, CANVAS_HEIGHT, CANVAS_INSET);
        assert_eq!(one.len(), 1);
        assert!(
            (one[0].height - (CANVAS_HEIGHT - 2.0 * CANVAS_INSET)).abs() < 0.001,
            "the height limits it"
        );
        assert!((one[0].left + one[0].width / 2.0 - CANVAS_WIDTH / 2.0).abs() < 0.001);
        assert!(canvas_layout(&[], CANVAS_WIDTH, CANVAS_HEIGHT, CANVAS_INSET).is_empty());
    }

    #[test]
    fn display_canvas_a_tiny_monitor_keeps_the_least_tile_size_around_its_centre() {
        let tiles = canvas_layout(
            &[[0, 0, 7680, 4320], [7680, 0, 7700, 10]],
            CANVAS_WIDTH,
            CANVAS_HEIGHT,
            CANVAS_INSET,
        );
        assert!(tiles[1].width >= MIN_TILE.0 && tiles[1].height >= MIN_TILE.1);
        assert!(tiles[0].width > tiles[1].width);
    }

    #[test]
    fn display_model_two_monitors_in_per_display_select_the_stored_target_and_disable_remove_without_a_wallpaper() {
        let model = DisplayModel::build(Some(&two()), DISPLAY9, Some(DisplayScreenType::PerDisplay), false);
        assert!(!model.loading);
        assert_eq!(model.tiles.len(), 2);
        assert!(
            !model.tiles[0].selected && model.tiles[1].selected,
            "DISPLAY9 is the target"
        );
        assert!(model.tiles.iter().all(|tile| tile.clickable));
        assert_eq!(model.rows.len(), 2);
        assert!(model.rows[0].can_remove, "DISPLAY1 plays a wallpaper");
        assert!(!model.rows[1].can_remove, "DISPLAY9 has none");
        assert!(model.rows[1].selected && !model.rows[0].selected);
        assert!(model.missing.is_empty(), "the Not connected section is omitted");
        assert_eq!(model.outline, None);
    }

    #[test]
    fn display_model_an_empty_or_unknown_target_is_the_first_monitor() {
        for device in ["", "\\\\.\\DISPLAY7"] {
            let model = DisplayModel::build(Some(&two()), device, Some(DisplayScreenType::PerDisplay), false);
            assert!(model.tiles[0].selected && !model.tiles[1].selected, "{device:?}");
        }
        let lower = DisplayModel::build(
            Some(&two()),
            "\\\\.\\display9",
            Some(DisplayScreenType::PerDisplay),
            false,
        );
        assert!(lower.tiles[1].selected, "device names compare without case");
    }

    #[test]
    fn display_model_span_and_duplicate_give_one_all_screens_row_and_no_target() {
        for mode in [DisplayScreenType::SpanAcross, DisplayScreenType::SameDuplicate] {
            let runtime = runtime(vec![
                display(DISPLAY1, 1, [0, 0, 1920, 1080], Some("a")),
                display(DISPLAY9, 9, [1920, 0, 3840, 1080], Some("a")),
            ]);
            let model = DisplayModel::build(Some(&runtime), DISPLAY1, Some(mode), false);
            assert!(
                model.tiles.iter().all(|tile| !tile.selected && !tile.clickable),
                "{mode:?}"
            );
            assert_eq!(model.rows.len(), 1, "{mode:?}");
            assert_eq!(model.rows[0].key, RowKey::All);
            assert!(model.rows[0].can_remove);
            assert_eq!(
                model.rows[0].remove_request(),
                PortalRequest::RemoveWallpaper { monitor_id: None }
            );
            assert_eq!(model.rows[0].select_request(), None);
            assert_eq!(model.outline.is_some(), mode == DisplayScreenType::SpanAcross);
        }
    }

    #[test]
    fn display_model_span_outline_surrounds_every_tile() {
        let model = DisplayModel::build(Some(&two()), DISPLAY1, Some(DisplayScreenType::SpanAcross), false);
        let outline = model.outline.expect("one outline in Span");
        for tile in &model.tiles {
            assert!(outline.left <= tile.rect.left && outline.top <= tile.rect.top);
            assert!(outline.right() >= tile.rect.right() && outline.bottom() >= tile.rect.bottom());
        }
    }

    #[test]
    fn display_model_a_tile_and_a_row_request_the_device_name_and_a_removal_the_monitor_id() {
        let model = DisplayModel::build(Some(&two()), DISPLAY1, Some(DisplayScreenType::PerDisplay), false);
        assert_eq!(
            model.tiles[1].select_request(),
            PortalRequest::Change(SettingChange::ScreenDevice(DISPLAY9.to_owned()))
        );
        assert_eq!(
            model.rows[1].select_request(),
            Some(PortalRequest::Change(SettingChange::ScreenDevice(DISPLAY9.to_owned())))
        );
        assert_eq!(
            model.rows[0].remove_request(),
            PortalRequest::RemoveWallpaper {
                monitor_id: Some("id-1".to_owned())
            }
        );
    }

    #[test]
    fn display_model_not_connected_assignments_are_listed_apart_and_removed_by_their_id() {
        let mut state = two();
        state.displays.push(missing(3, "b"));
        let model = DisplayModel::build(Some(&state), DISPLAY1, Some(DisplayScreenType::PerDisplay), false);
        assert_eq!(model.tiles.len(), 2, "a monitor that is not connected has no tile");
        assert_eq!(model.rows.len(), 2, "and no row among the connected ones");
        assert_eq!(model.missing.len(), 1);
        assert_eq!(model.missing[0].theme.as_deref(), Some("b"));
        assert_eq!(
            model.missing[0].remove_request(),
            PortalRequest::RemoveWallpaper {
                monitor_id: Some("id-3".to_owned())
            }
        );
    }

    #[test]
    fn display_model_is_loading_until_the_service_has_reported_the_runtime_and_the_mode() {
        let before = DisplayModel::build(None, "", None, false);
        assert!(before.loading && before.tiles.is_empty() && before.rows.is_empty());
        let no_mode = DisplayModel::build(Some(&two()), "", None, false);
        assert!(no_mode.loading);
        let no_runtime = DisplayModel::build(None, "", Some(DisplayScreenType::PerDisplay), false);
        assert!(no_runtime.loading);
    }

    #[test]
    fn display_model_a_rule_marks_only_the_tiles_that_have_a_wallpaper() {
        let mut state = two();
        state.rule = RuleState::Paused;
        let model = DisplayModel::build(Some(&state), DISPLAY1, Some(DisplayScreenType::PerDisplay), false);
        assert_eq!(model.tiles[0].rule, Some(super::RuleIcon::Paused));
        assert_eq!(model.tiles[1].rule, None);
        state.rule = RuleState::Closed;
        let model = DisplayModel::build(Some(&state), DISPLAY1, Some(DisplayScreenType::PerDisplay), false);
        assert_eq!(model.tiles[0].rule, Some(super::RuleIcon::Closed));
    }

    #[test]
    fn display_mode_a_choice_waits_for_the_broadcast_and_a_rejection_shows_the_stored_mode_again() {
        let mut mode = ModeState::default();
        assert_eq!(mode.shown(), None);
        mode.set_stored(DisplayScreenType::PerDisplay);
        assert!(
            !mode.choose(DisplayScreenType::PerDisplay),
            "the shown mode asks for nothing"
        );
        assert!(mode.choose(DisplayScreenType::SpanAcross));
        assert!(mode.applying());
        assert_eq!(mode.shown(), Some(DisplayScreenType::SpanAcross));
        mode.set_stored(DisplayScreenType::SpanAcross);
        assert!(!mode.applying(), "the broadcast of the asked mode ends the wait");
        assert!(mode.choose(DisplayScreenType::SameDuplicate));
        assert!(mode.rejected());
        assert_eq!(mode.shown(), Some(DisplayScreenType::SpanAcross), "the radios revert");
        assert!(!mode.rejected(), "nothing waits any more");
    }

    #[test]
    fn display_mode_a_broadcast_of_another_mode_does_not_end_the_wait_for_the_asked_one() {
        let mut mode = ModeState::default();
        mode.set_stored(DisplayScreenType::PerDisplay);
        assert!(mode.choose(DisplayScreenType::SpanAcross));
        mode.set_stored(DisplayScreenType::SameDuplicate);
        assert!(mode.applying());
        assert_eq!(mode.shown(), Some(DisplayScreenType::SpanAcross));
    }

    /// The header with the dialog behind it, inside a Root (the dialog layer lives there).
    struct Host {
        header: Entity<Header>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            v_flex().size_full().child(self.header.clone())
        }
    }

    #[gpui_kit::test]
    fn display_header_button_opens_the_dialog_and_a_tile_click_sets_the_target(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (link, requests) = ServiceLink::recording();
        let slot: Rc<RefCell<Option<Entity<DisplayPreferences>>>> = Rc::default();
        let handed = slot.clone();
        let (_, window) = cx.add_window_view(move |window, cx| {
            let library = cx.new(|cx| LibraryView::new(link.clone(), window, cx));
            let display = cx.new(|cx| DisplayPreferences::new(link.clone(), library.clone(), cx));
            *handed.borrow_mut() = Some(display.clone());
            let header = cx.new(|cx| Header::new(library, display, link, Mode::Library, window, cx));
            let host = cx.new(|_| Host { header });
            Root::new(host, window, cx)
        });
        let display = slot
            .borrow()
            .clone()
            .expect("the display model was built with the window");
        window.simulate_resize(size(px(1000.), px(800.)));
        display.update(window, |display, cx| {
            display.set_placement(DisplayScreenType::PerDisplay, DISPLAY9.to_owned(), cx);
            display.set_runtime(two(), cx);
        });
        window.run_until_parked();
        assert!(!window.update(|window, cx| window.has_active_dialog(cx)));

        let button = window
            .debug_bounds("display-preferences-button")
            .expect("the header has the Display Preferences button");
        window.simulate_click(button.center(), Modifiers::none());
        window.run_until_parked();
        assert!(
            window.update(|window, cx| window.has_active_dialog(cx)),
            "the button opened the dialog"
        );

        let first = window.debug_bounds("display-tile-0").expect("DISPLAY1 has a tile");
        let second = window.debug_bounds("display-tile-1").expect("DISPLAY9 has a tile");
        assert!(
            first.right() <= second.left() + px(0.5),
            "the tiles follow the monitors left to right"
        );
        window.simulate_click(first.center(), Modifiers::none());
        window.run_until_parked();

        let sent: Vec<PortalRequest> = requests.try_iter().collect();
        assert_eq!(
            sent,
            vec![PortalRequest::Change(SettingChange::ScreenDevice(DISPLAY1.to_owned()))],
            "exactly one request, for DISPLAY1"
        );
        // The dialog shows what the service broadcast: the target moves only when the service says so.
        let model = display.read_with(window, |display, _| display.model());
        assert!(model.tiles[1].selected && !model.tiles[0].selected);
        display.update(window, |display, cx| {
            display.on_setting(&SettingChange::ScreenDevice(DISPLAY1.to_owned()), cx);
        });
        let model = display.read_with(window, |display, _| display.model());
        assert!(model.tiles[0].selected && !model.tiles[1].selected);
    }

    /// A view with nothing in it; the dialog opens over it.
    struct Blank;

    impl Render for Blank {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    /// A lookup that knows no theme.
    fn no_themes() -> ThemeLookup {
        Rc::new(|_, _| None)
    }

    /// A window with Display Preferences open over it in the given mode, fed with the runtime state.
    fn open_window(
        cx: &mut TestAppContext,
        lookup: ThemeLookup,
        mode: DisplayScreenType,
        state: RuntimeState,
    ) -> (
        Entity<DisplayPreferences>,
        &mut VisualTestContext,
        crossbeam_channel::Receiver<PortalRequest>,
    ) {
        cx.update(gpui_kit::init);
        let (link, requests) = ServiceLink::recording();
        let slot: Rc<RefCell<Option<Entity<DisplayPreferences>>>> = Rc::default();
        let handed = slot.clone();
        let (_, window) = cx.add_window_view(move |window, cx| {
            let display = cx.new(|cx| DisplayPreferences::with_lookup(link, lookup, cx));
            *handed.borrow_mut() = Some(display);
            let blank = cx.new(|_| Blank);
            Root::new(blank, window, cx)
        });
        let display = slot
            .borrow()
            .clone()
            .expect("the display model was built with the window");
        // Tall enough for the whole dialog: a click outside the scrolled body would reach nothing.
        window.simulate_resize(size(px(1000.), px(1300.)));
        display.update(window, |display, cx| {
            display.set_placement(mode, DISPLAY1.to_owned(), cx);
            display.set_runtime(state, cx);
        });
        window.update(|window, cx| open_display_preferences(&display, window, cx));
        window.run_until_parked();
        window.update(|window, _| window.refresh());
        window.run_until_parked();
        (display, window, requests)
    }

    fn click(window: &mut VisualTestContext, selector: &'static str) {
        let bounds = window
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("no element tagged {selector}"));
        window.simulate_click(bounds.center(), Modifiers::none());
        window.run_until_parked();
    }

    fn sent(requests: &crossbeam_channel::Receiver<PortalRequest>) -> Vec<PortalRequest> {
        requests.try_iter().collect()
    }

    fn mode_request(mode: DisplayScreenType) -> PortalRequest {
        PortalRequest::Change(SettingChange::DisplayScreenType(mode))
    }

    /// Both monitors play a theme.
    fn both_play() -> RuntimeState {
        runtime(vec![
            display(DISPLAY1, 1, [0, 0, 1920, 1080], Some("a")),
            display(DISPLAY9, 9, [1920, 0, 3840, 1080], Some("b")),
        ])
    }

    #[gpui_kit::test]
    fn display_placement_a_radio_sends_one_change_and_waits_for_the_broadcast(cx: &mut TestAppContext) {
        let (display, window, requests) = open_window(cx, no_themes(), DisplayScreenType::PerDisplay, two());
        assert!(!display.read_with(window, |display, _| display.model().applying));

        click(window, "display-radio-1");
        assert_eq!(sent(&requests), vec![mode_request(DisplayScreenType::SpanAcross)]);
        let model = display.read_with(window, |display, _| display.model());
        assert!(model.applying, "Applying shows until the service confirms");
        assert_eq!(model.mode, Some(DisplayScreenType::SpanAcross));

        click(window, "display-radio-1");
        assert!(
            sent(&requests).is_empty(),
            "the mode already asked for is not asked again"
        );

        display.update(window, |display, cx| {
            display.on_setting(&SettingChange::DisplayScreenType(DisplayScreenType::SpanAcross), cx);
        });
        assert!(!display.read_with(window, |display, _| display.model().applying));
    }

    #[gpui_kit::test]
    fn display_placement_a_rejected_change_reverts_the_radios_and_asks_for_the_toast(cx: &mut TestAppContext) {
        let (display, window, requests) = open_window(cx, no_themes(), DisplayScreenType::PerDisplay, two());
        click(window, "display-radio-2");
        assert_eq!(sent(&requests), vec![mode_request(DisplayScreenType::SameDuplicate)]);

        let refused = SettingChange::DisplayScreenType(DisplayScreenType::SameDuplicate);
        let toast = display.update(window, |display, cx| display.on_rejected(&refused, cx));
        assert!(toast, "a refused placement change raises the placement toast");
        let model = display.read_with(window, |display, _| display.model());
        assert_eq!(
            model.mode,
            Some(DisplayScreenType::PerDisplay),
            "the radios show the stored mode"
        );
        assert!(!model.applying);

        let other = SettingChange::WallpaperLoop(true);
        assert!(!display.update(window, |display, cx| display.on_rejected(&other, cx)));
        let target = SettingChange::ScreenDevice(DISPLAY9.to_owned());
        assert!(display.update(window, |display, cx| display.on_rejected(&target, cx)));
    }

    #[gpui_kit::test]
    fn display_placement_the_radios_are_disabled_until_the_service_has_reported(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (link, requests) = ServiceLink::recording();
        let slot: Rc<RefCell<Option<Entity<DisplayPreferences>>>> = Rc::default();
        let handed = slot.clone();
        let (_, window) = cx.add_window_view(move |window, cx| {
            let display = cx.new(|cx| DisplayPreferences::with_lookup(link, no_themes(), cx));
            *handed.borrow_mut() = Some(display);
            let blank = cx.new(|_| Blank);
            Root::new(blank, window, cx)
        });
        let display = slot
            .borrow()
            .clone()
            .expect("the display model was built with the window");
        window.simulate_resize(size(px(1000.), px(900.)));
        window.update(|window, cx| open_display_preferences(&display, window, cx));
        window.run_until_parked();

        assert!(display.read_with(window, |display, _| display.model().loading));
        assert!(
            window.debug_bounds("display-tile-0").is_none(),
            "the canvas is one skeleton"
        );
        click(window, "display-radio-1");
        assert!(sent(&requests).is_empty(), "a disabled radio asks for nothing");
    }

    #[gpui_kit::test]
    fn display_list_remove_sends_the_monitor_and_does_not_also_select_the_row(cx: &mut TestAppContext) {
        let (_, window, requests) = open_window(cx, no_themes(), DisplayScreenType::PerDisplay, both_play());
        click(window, "display-remove-1");
        assert_eq!(
            sent(&requests),
            vec![PortalRequest::RemoveWallpaper {
                monitor_id: Some("id-9".to_owned())
            }],
            "only the removal, not a ScreenDevice for the row"
        );
        click(window, "display-row-1");
        assert_eq!(
            sent(&requests),
            vec![PortalRequest::Change(SettingChange::ScreenDevice(DISPLAY9.to_owned()))],
            "a click on the row selects the monitor as the target"
        );
        click(window, "display-row-0");
        assert!(
            sent(&requests).is_empty(),
            "the row that is the target asks for nothing"
        );
    }

    #[gpui_kit::test]
    fn display_list_remove_is_disabled_on_a_monitor_without_a_wallpaper(cx: &mut TestAppContext) {
        let (_, window, requests) = open_window(cx, no_themes(), DisplayScreenType::PerDisplay, two());
        click(window, "display-remove-1");
        assert!(sent(&requests).is_empty());
        click(window, "display-remove-0");
        assert_eq!(
            sent(&requests),
            vec![PortalRequest::RemoveWallpaper {
                monitor_id: Some("id-1".to_owned())
            }]
        );
    }

    #[gpui_kit::test]
    fn display_list_span_has_one_all_screens_row_that_removes_the_single_wallpaper(cx: &mut TestAppContext) {
        let (_, window, requests) = open_window(cx, no_themes(), DisplayScreenType::SpanAcross, both_play());
        assert!(window.debug_bounds("display-row-0").is_some());
        assert!(window.debug_bounds("display-row-1").is_none(), "one row in Span");
        click(window, "display-tile-0");
        assert!(sent(&requests).is_empty(), "tiles are not clickable in Span");
        click(window, "display-remove-0");
        assert_eq!(
            sent(&requests),
            vec![PortalRequest::RemoveWallpaper { monitor_id: None }]
        );
    }

    #[gpui_kit::test]
    fn display_not_connected_assignments_have_their_own_section_with_a_remove_button(cx: &mut TestAppContext) {
        let mut state = two();
        state.displays.push(missing(3, "c"));
        let (display, window, requests) = open_window(cx, no_themes(), DisplayScreenType::PerDisplay, state);
        click(window, "display-missing-remove-0");
        assert_eq!(
            sent(&requests),
            vec![PortalRequest::RemoveWallpaper {
                monitor_id: Some("id-3".to_owned())
            }]
        );
        // Without such an assignment the section is gone.
        display.update(window, |display, cx| display.set_runtime(two(), cx));
        window.run_until_parked();
        assert!(window.debug_bounds("display-missing-remove-0").is_none());
    }

    /// A lookup that knows the themes a and b, both with the thumbnail at the same file.
    fn lookup_of(file: PathBuf) -> ThemeLookup {
        Rc::new(move |folder, _| {
            matches!(folder, "a" | "b").then(|| ThemeBrief {
                title: format!("Theme {folder}"),
                thumbnail: file.clone(),
            })
        })
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sucrose-display-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a scratch folder");
        dir
    }

    #[gpui_kit::test]
    fn display_thumbnails_are_decoded_in_the_background_drawn_and_released_when_the_dialog_closes(
        cx: &mut TestAppContext,
    ) {
        let dir = scratch_dir("thumb");
        let file = dir.join("thumb.png");
        image::RgbaImage::from_pixel(64, 36, image::Rgba([200, 30, 30, 255]))
            .save(&file)
            .expect("a thumbnail file");
        let (display, window, _requests) = open_window(cx, lookup_of(file), DisplayScreenType::PerDisplay, both_play());

        let images = display.read_with(window, |display, _| display.ready_images());
        assert_eq!(images.len(), 1, "both monitors share one thumbnail file, decoded once");
        let image = images[0].clone();
        assert!(
            window.update(|window, _| window.has_image_atlas_entry(&image)),
            "the tiles draw it"
        );

        window.update(|window, cx| window.close_dialog(cx));
        window.run_until_parked();
        window.executor().advance_clock(std::time::Duration::from_secs(1));
        window.run_until_parked();
        assert!(!window.update(|window, cx| window.has_active_dialog(cx)));
        assert!(
            display.read_with(window, |display, _| display.ready_images().is_empty()),
            "the model lets go of the thumbnails"
        );
        assert!(
            !window.update(|window, _| window.has_image_atlas_entry(&image)),
            "and the atlas of the window does not keep them"
        );
        std::fs::remove_dir_all(dir).ok();
    }

    #[gpui_kit::test]
    fn display_thumbnails_a_file_that_is_no_image_gives_the_tile_the_fallback(cx: &mut TestAppContext) {
        let dir = scratch_dir("bad");
        let file = dir.join("thumb.png");
        std::fs::write(&file, b"not an image").expect("a bad thumbnail file");
        let (display, window, _requests) = open_window(cx, lookup_of(file), DisplayScreenType::PerDisplay, both_play());
        assert!(display.read_with(window, |display, _| display.ready_images().is_empty()));
        assert!(display.read_with(window, |display, cx| matches!(
            display.tile_thumb(Some("a"), cx),
            TileThumb::Failed
        )));
        assert!(display.read_with(window, |display, cx| matches!(
            display.tile_thumb(None, cx),
            TileThumb::Empty
        )));
        std::fs::remove_dir_all(dir).ok();
    }

    #[gpui_kit::test]
    fn display_keyboard_arrows_move_between_tiles_and_enter_selects(cx: &mut TestAppContext) {
        let (display, window, requests) = open_window(cx, no_themes(), DisplayScreenType::PerDisplay, two());
        let (first, second) = display.read_with(window, |display, _| {
            (display.tile_focus[0].clone(), display.tile_focus[1].clone())
        });
        window.update(|window, cx| window.focus(&first, cx));
        window.simulate_keystrokes("right");
        assert!(
            window.update(|window, _| second.is_focused(window)),
            "right moves to the next tile"
        );
        window.simulate_keystrokes("right");
        assert!(
            window.update(|window, _| second.is_focused(window)),
            "the last tile stays"
        );
        window.simulate_keystrokes("left");
        assert!(window.update(|window, _| first.is_focused(window)), "left moves back");
        window.simulate_keystrokes("right");
        window.simulate_keystrokes("enter");
        window.run_until_parked();
        assert_eq!(
            sent(&requests),
            vec![PortalRequest::Change(SettingChange::ScreenDevice(DISPLAY9.to_owned()))],
            "Enter on the focused tile selects it"
        );
        assert!(
            window.update(|window, cx| window.has_active_dialog(cx)),
            "and does not confirm (close) the dialog"
        );
    }

    #[gpui_kit::test]
    fn display_keyboard_focus_lands_on_the_radios_and_escape_closes_the_dialog(cx: &mut TestAppContext) {
        let (display, window, _requests) = open_window(cx, no_themes(), DisplayScreenType::PerDisplay, two());
        window.run_until_parked();
        window.update(|window, _| window.refresh());
        window.run_until_parked();
        let radios = display.read_with(window, |display, _| display.radio_focus.clone());
        assert!(
            window.update(|window, cx| radios.contains_focused(window, cx)),
            "the focus is in the radio group"
        );
        window.simulate_keystrokes("escape");
        window.run_until_parked();
        window.executor().advance_clock(std::time::Duration::from_secs(1));
        window.run_until_parked();
        assert!(
            !window.update(|window, cx| window.has_active_dialog(cx)),
            "Escape closes it"
        );
    }
}
