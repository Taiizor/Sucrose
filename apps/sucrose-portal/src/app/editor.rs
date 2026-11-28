//! The properties editor window (PROP-01, PROP-02, D-18, D-19): a second window of the Portal process that edits the
//! properties of one wallpaper while it plays.
//!
//! At most one editor window exists. The card's Customize icon (or the measurement-only switch) opens it, or, when it is
//! open, replaces its content and brings it to the front; closing it drops the document and the controls and tells the
//! service, which writes the working copy once more. The editor never reads or writes a property file: the service sends
//! the document as blob chunks, and every change goes back as one ChangeProperty request (Pattern 6).
//!
//! The window is placed once, when it opens, and never moves again. It goes to the corner of the work area that is next to
//! the taskbar of the monitor that holds the Portal window, inset 16 px, at that monitor's DPI. No taskbar API is asked:
//! the taskbar edge is the side where the work area is smaller than the monitor, and a taskbar that hides itself leaves no
//! such side, which gives the bottom corner. [`editor_origin`] and [`taskbar_edge`] are pure functions of physical pixels, so every edge and the
//! mirrored right-to-left corners are unit tests; the window itself is given the origin divided by the monitor's scale,
//! because GPUI places a window in logical pixels of the display it is opened on.
//!
//! Measured by plan 48's spike on Windows 11 (DISPLAY1, bottom taskbar, 100 %): the window opens at the computed corner in x
//! exactly, and 4 px higher than computed in y. The client origin GPUI reads back lies half of the invisible resize frame
//! above the requested top, because a window with the title bar drawn by the app has that frame at the bottom only
//! (`FRAME_SHIFT`); the first window of a run logs the difference as `editor placed ... agree=false` and the next window
//! of the run asks for the shifted origin. A window is placed in logical pixels of the display named in its options:
//! without `display_id` GPUI converts with the primary display's scale.

use std::time::{Duration, Instant};

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Icon, Sizable, TitleBar, h_flex,
    slider::{Slider, SliderEvent, SliderState},
    tooltip::Tooltip,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use sucrose_core::properties::{Control, ControlKind, SucroseProperties, parse_properties};
use sucrose_ipc::{
    blob::{Outcome, Reassembler},
    portal::{PortalEvent, PortalRequest, PropertiesFailure, PropertiesView},
};
use sucrose_platform::{MonitorInfo, RectPx};

use crate::{
    i18n::{tr, tr_with},
    ipc::ServiceLink,
};

/// The window's width, in logical pixels.
pub const EDITOR_WIDTH: f32 = 400.0;
/// The smallest height, in logical pixels.
pub const EDITOR_MIN_HEIGHT: f32 = 320.0;
/// The height the window opens with before the document is known, in logical pixels.
const EDITOR_START_HEIGHT: f32 = 480.0;
/// The distance to the work area's edges, in physical pixels of a 100 % screen and scaled with the monitor.
const EDITOR_INSET: f32 = 16.0;
/// A slider drag sends at most one change per this long.
pub const SEND_INTERVAL: Duration = Duration::from_millis(50);

/// The side of a monitor the taskbar is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskbarEdge {
    /// The work area is shorter at the bottom.
    Bottom,
    /// The work area starts right of the monitor's left edge.
    Left,
    /// The work area ends left of the monitor's right edge.
    Right,
    /// The work area starts below the monitor's top edge.
    Top,
    /// The work area is the whole monitor: no taskbar, or one that hides itself.
    None,
}

/// The taskbar edge of a monitor: the side on which its work area is smaller than the monitor rectangle.
#[must_use]
pub fn taskbar_edge(rect: RectPx, work: RectPx) -> TaskbarEdge {
    if work.bottom < rect.bottom {
        TaskbarEdge::Bottom
    } else if work.left > rect.left {
        TaskbarEdge::Left
    } else if work.right < rect.right {
        TaskbarEdge::Right
    } else if work.top > rect.top {
        TaskbarEdge::Top
    } else {
        TaskbarEdge::None
    }
}

/// The top-left corner, in physical pixels, of a window of \`size\` (width, height) next to the taskbar: the corner of
/// \`work\` that touches the taskbar's side, inset 16 px. A taskbar at the bottom gives the bottom-right corner (bottom-left
/// when \`rtl\`), one at the left the bottom-left and one at the right the bottom-right (always next to the bar), one at
/// the top the top-right (top-left when \`rtl\`); no taskbar gives the bottom corner.
#[must_use]
pub fn editor_origin(work: RectPx, edge: TaskbarEdge, size: (i32, i32), rtl: bool, inset: i32) -> (i32, i32) {
    let (width, height) = size;
    let right = work.right - inset - width;
    let left = work.left + inset;
    let bottom = work.bottom - inset - height;
    let top = work.top + inset;
    match edge {
        TaskbarEdge::Bottom | TaskbarEdge::None => (if rtl { left } else { right }, bottom),
        TaskbarEdge::Left => (left, bottom),
        TaskbarEdge::Right => (right, bottom),
        TaskbarEdge::Top => (if rtl { left } else { right }, top),
    }
}

/// The window height in logical pixels for a content of \`content\` pixels on a work area \`work_height\` tall: at least the
/// minimum and at most the work area minus 32.
#[must_use]
pub fn editor_height(content: f32, work_height: f32) -> f32 {
    content.clamp(EDITOR_MIN_HEIGHT, (work_height - 32.0).max(EDITOR_MIN_HEIGHT))
}

/// The monitor that holds the physical point (\`x\`, \`y\`), else the primary one, else the first.
#[must_use]
pub fn monitor_holding(monitors: &[MonitorInfo], x: i32, y: i32) -> Option<&MonitorInfo> {
    monitors
        .iter()
        .find(|monitor| {
            monitor.rect.left <= x && x < monitor.rect.right && monitor.rect.top <= y && y < monitor.rect.bottom
        })
        .or_else(|| monitors.iter().find(|monitor| monitor.primary))
        .or_else(|| monitors.first())
}

/// Where the editor window goes: all numbers physical pixels except \`bounds\`, which is what GPUI is given.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    /// The monitor's device name.
    pub monitor: String,
    /// The taskbar edge that decided the corner.
    pub edge: TaskbarEdge,
    /// The monitor's scale (1.0 at 100 %).
    pub scale: f32,
    /// The top-left corner of the window, physical pixels.
    pub origin: (i32, i32),
    /// The window's size, physical pixels.
    pub size: (i32, i32),
}

/// The placement of the editor on \`monitor\`.
#[must_use]
pub fn placement(monitor: &MonitorInfo, rtl: bool) -> Placement {
    let scale = (monitor.dpi.max(1) as f32 / 96.0).max(0.5);
    let height = editor_height(EDITOR_START_HEIGHT, monitor.work.height() as f32 / scale);
    let size = ((EDITOR_WIDTH * scale).round() as i32, (height * scale).round() as i32);
    let edge = taskbar_edge(monitor.rect, monitor.work);
    let origin = editor_origin(monitor.work, edge, size, rtl, (EDITOR_INSET * scale).round() as i32);
    Placement {
        monitor: monitor.id.clone(),
        edge,
        scale,
        origin,
        size,
    }
}

impl Placement {
    /// The bounds GPUI is given: the origin moved by `shift` (physical pixels) and the size, divided by the monitor's
    /// scale (logical pixels). The shift is how far GPUI's client origin lay from the requested one on a window of this
    /// kind before (see [`FRAME_SHIFT`]).
    #[must_use]
    pub fn bounds(&self, shift: (i32, i32)) -> Bounds<Pixels> {
        Bounds {
            origin: point(
                px((self.origin.0 + shift.0) as f32 / self.scale),
                px((self.origin.1 + shift.1) as f32 / self.scale),
            ),
            size: size(px(self.size.0 as f32 / self.scale), px(self.size.1 as f32 / self.scale)),
        }
    }
}

/// How far, in physical pixels, the client origin GPUI reports for an editor window lay from the origin it was asked for,
/// learned from the first window of this run. GPUI asks Windows for a window rectangle that is the requested client
/// rectangle plus half of the invisible frame on every side; a window with the title bar drawn by the app has that frame at
/// the bottom only, so its client area starts half a frame (4 px at 100 %) above the requested top. The next editor window
/// of the run asks for the origin plus this shift, so it lands where [`placement`] says. Measured by plan 48's spike.
static FRAME_SHIFT: std::sync::Mutex<(i32, i32)> = std::sync::Mutex::new((0, 0));

/// Whether the UI language is written right to left (Arabic, Hebrew, Persian, Urdu): the editor then goes to the left corner.
fn is_rtl() -> bool {
    let locale = rust_i18n::locale().to_ascii_lowercase();
    ["ar", "he", "fa", "ur"].iter().any(|prefix| locale.starts_with(prefix))
}

/// What a value that changes fast is told: send it now, or wait for the interval to pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    /// Send it now.
    Send,
    /// The last send is too recent: send the newest value after this long.
    Wait(Duration),
}

/// Keeps a drag to one change per [`SEND_INTERVAL`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SendGate {
    last: Option<Instant>,
}

impl SendGate {
    /// A value arrived at \`now\`: either it goes out (and counts as the last send) or it has to wait.
    pub fn offer(&mut self, now: Instant) -> Gate {
        match self.last {
            Some(last) if now.saturating_duration_since(last) < SEND_INTERVAL => {
                Gate::Wait(SEND_INTERVAL - now.saturating_duration_since(last))
            }
            _ => {
                self.last = Some(now);
                Gate::Send
            }
        }
    }

    /// A value was sent at \`now\` outside [`SendGate::offer`] (the release of a drag, or the trailing send).
    pub fn sent(&mut self, now: Instant) {
        self.last = Some(now);
    }
}

/// The number of decimals of \`step\`, at most 6: the precision of a slider's readout.
#[must_use]
pub fn decimals(step: f64) -> usize {
    let text = format!("{step}");
    text.split_once('.').map_or(0, |(_, fraction)| fraction.len().min(6))
}

/// \`value\` held inside \`min\` and \`max\` and snapped to the grid of \`step\` that starts at \`min\`.
#[must_use]
pub fn snapped(value: f64, min: f64, max: f64, step: f64) -> f64 {
    let (low, high) = if min <= max { (min, max) } else { (max, min) };
    let held = value.clamp(low, high);
    if step <= 0.0 {
        return held;
    }
    let snapped = ((held - low) / step).round().mul_add(step, low).clamp(low, high);
    // Round away the noise of the multiplication (0.1 steps).
    let factor = 10f64.powi(i32::try_from(decimals(step)).unwrap_or(6));
    (snapped * factor).round() / factor
}

/// The text of a label or help: a key of the file's localization table for \`culture\`, else of the first table that has
/// it, else the text itself (UI-SPEC partial state).
#[must_use]
pub fn localized(properties: &SucroseProperties, culture: Option<&str>, text: &str) -> String {
    let in_table = |table: &(String, Vec<(String, String)>)| {
        table
            .1
            .iter()
            .find(|(key, _)| key == text)
            .map(|(_, value)| value.clone())
    };
    culture
        .and_then(|culture| {
            properties
                .localization
                .iter()
                .find(|table| table.0.eq_ignore_ascii_case(culture))
                .and_then(in_table)
        })
        .or_else(|| properties.localization.first().and_then(in_table))
        .unwrap_or_else(|| text.to_owned())
}

/// The number of a monitor from its device name (\`\\.\DISPLAY9\` is "9").
fn monitor_number(device: &str) -> String {
    let digits: String = device
        .chars()
        .rev()
        .take_while(char::is_ascii_digit)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if digits.is_empty() { device.to_owned() } else { digits }
}

/// What the window shows in its content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Content {
    /// The document is on its way.
    Loading,
    /// The document has no controls.
    Empty,
    /// The service could not give the document.
    Failed(PropertiesFailure),
    /// The controls are shown.
    Ready,
}

/// One slider of the document with the state it drags.
struct SliderRow {
    /// The property name.
    key: String,
    state: Entity<SliderState>,
    min: f64,
    max: f64,
    step: f64,
    /// The value shown.
    value: f64,
    /// The newest value not yet sent.
    pending: Option<f64>,
    gate: SendGate,
    /// The trailing send that waits for the interval; replacing it cancels the one before.
    trailing: Option<Task<()>>,
}

/// The editor window's view.
pub struct PropertiesEditor {
    folder: String,
    title: SharedString,
    link: ServiceLink,
    content: Content,
    view: Option<PropertiesView>,
    reassembler: Reassembler,
    properties: Option<SucroseProperties>,
    sliders: Vec<SliderRow>,
    subscriptions: Vec<Subscription>,
    focus: FocusHandle,
}

impl PropertiesEditor {
    /// An editor that asks the service for the properties of \`folder\`.
    pub fn new(folder: &str, title: &str, link: ServiceLink, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let mut editor = Self {
            folder: String::new(),
            title: SharedString::default(),
            link,
            content: Content::Loading,
            view: None,
            reassembler: Reassembler::new(),
            properties: None,
            sliders: Vec::new(),
            subscriptions: Vec::new(),
            focus,
        };
        editor.show(folder, title, window, cx);
        editor
    }

    /// Shows the properties of \`folder\`: the document of the theme shown before is dropped (and the service writes its
    /// working copy), and the new one is asked for. The same theme again asks again, which refreshes the controls.
    pub fn show(&mut self, folder: &str, title: &str, window: &mut Window, cx: &mut Context<Self>) {
        let _ = window;
        if !self.folder.is_empty() {
            self.link.send(PortalRequest::CloseProperties {
                folder: self.folder.clone(),
            });
        }
        folder.clone_into(&mut self.folder);
        self.title = SharedString::from(title.to_owned());
        self.content = Content::Loading;
        self.view = None;
        self.reassembler = Reassembler::new();
        self.properties = None;
        self.sliders.clear();
        self.subscriptions.clear();
        self.link.send(PortalRequest::OpenProperties {
            folder: self.folder.clone(),
        });
        cx.notify();
    }

    /// The folder of the theme the window shows.
    #[cfg(test)]
    fn folder(&self) -> &str {
        &self.folder
    }

    /// One event of the service that may concern this window: the answer to the open request.
    pub fn on_event(&mut self, event: PortalEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            PortalEvent::Properties(view) if view.folder == self.folder => {
                self.reassembler = Reassembler::new();
                self.view = Some(view);
            }
            PortalEvent::PropertiesBlob { id, seq, total, bytes } => {
                if self.view.as_ref().map(|view| view.blob_id) != Some(id) {
                    return;
                }
                match self.reassembler.push(id, seq, total, &bytes) {
                    Outcome::Done(_, document) => self.load(&document, window, cx),
                    Outcome::Pending | Outcome::Ignored => {}
                    Outcome::Dropped(_, reason) => {
                        tracing::warn!(?reason, "the properties document was dropped");
                        self.content = Content::Failed(PropertiesFailure::Unreadable);
                    }
                }
            }
            PortalEvent::PropertiesFailed { folder, reason } if folder == self.folder => {
                self.content = Content::Failed(reason);
            }
            _ => return,
        }
        cx.notify();
    }

    /// Builds the controls from the document text.
    fn load(&mut self, document: &[u8], window: &mut Window, cx: &mut Context<Self>) {
        let parsed = std::str::from_utf8(document)
            .map_err(|_| ())
            .and_then(|text| parse_properties(text).map_err(|_| ()));
        let Ok(properties) = parsed else {
            self.content = Content::Failed(PropertiesFailure::Unreadable);
            return;
        };
        self.sliders.clear();
        self.subscriptions.clear();
        for (key, control) in &properties.list {
            let ControlKind::Slider { min, max, step, value } = &control.kind else {
                continue;
            };
            let step = if *step > 0.0 { *step } else { 1.0 };
            let state = cx.new(|_| {
                SliderState::new()
                    .min(*min as f32)
                    .max(*max as f32)
                    .step(step as f32)
                    .default_value(*value as f32)
            });
            let index = self.sliders.len();
            self.subscriptions.push(cx.subscribe_in(
                &state,
                window,
                move |this, _, event: &SliderEvent, window, cx| {
                    this.on_slider(index, event, window, cx);
                },
            ));
            self.sliders.push(SliderRow {
                key: key.clone(),
                state,
                min: *min,
                max: *max,
                step,
                value: snapped(*value, *min, *max, step),
                pending: None,
                gate: SendGate::default(),
                trailing: None,
            });
        }
        self.content = if properties.list.is_empty() {
            Content::Empty
        } else {
            Content::Ready
        };
        self.properties = Some(properties);
    }

    /// A slider moved: a drag sends at most every [`SEND_INTERVAL`], the release always sends.
    fn on_slider(&mut self, index: usize, event: &SliderEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(row) = self.sliders.get_mut(index) else {
            return;
        };
        let (raw, release) = match event {
            SliderEvent::Change(value) => (f64::from(value.start()), false),
            SliderEvent::Release(value) => (f64::from(value.start()), true),
        };
        let value = snapped(raw, row.min, row.max, row.step);
        row.value = value;
        row.pending = Some(value);
        let now = Instant::now();
        if release {
            row.trailing = None;
            row.gate.sent(now);
            self.send_pending(index);
        } else {
            match row.gate.offer(now) {
                Gate::Send => self.send_pending(index),
                Gate::Wait(wait) => {
                    if row.trailing.is_none() {
                        row.trailing = Some(cx.spawn_in(window, async move |this, cx| {
                            cx.background_executor().timer(wait).await;
                            this.update_in(cx, |this, _, cx| this.send_trailing(index, cx)).ok();
                        }));
                    }
                }
            }
        }
        cx.notify();
    }

    /// The wait of a drag is over: the newest value goes out.
    fn send_trailing(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(row) = self.sliders.get_mut(index) else {
            return;
        };
        row.trailing = None;
        row.gate.sent(Instant::now());
        self.send_pending(index);
        cx.notify();
    }

    /// Sends the newest value of a slider, if one waits: the control as the file holds it with the new value.
    fn send_pending(&mut self, index: usize) {
        let Some(row) = self.sliders.get_mut(index) else {
            return;
        };
        let Some(value) = row.pending.take() else {
            return;
        };
        let key = row.key.clone();
        let Some(properties) = &mut self.properties else {
            return;
        };
        let Some((_, control)) = properties.list.iter_mut().find(|(name, _)| *name == key) else {
            return;
        };
        if let ControlKind::Slider { value: stored, .. } = &mut control.kind {
            *stored = value;
        }
        let text = control.to_json_value().to_string();
        self.link.send(PortalRequest::ChangeProperty {
            folder: self.folder.clone(),
            key,
            control: text,
        });
    }

    /// The label of a control: its text through the localization tables, else its key.
    fn label_of(&self, key: &str, control: &Control) -> String {
        let culture = self.view.as_ref().and_then(|view| view.culture.as_deref());
        match (&self.properties, control.text.as_deref()) {
            (Some(properties), Some(text)) if !text.is_empty() => localized(properties, culture, text),
            _ => key.to_owned(),
        }
    }

    /// The chip under the title and the status line: where the wallpaper plays.
    fn live_monitors(&self) -> Option<String> {
        let view = self.view.as_ref()?;
        if view.live_on.is_empty() {
            return None;
        }
        Some(
            view.live_on
                .iter()
                .map(|device| monitor_number(device))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }

    fn render_control(&self, key: &str, control: &Control, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let label = self.label_of(key, control);
        match &control.kind {
            ControlKind::Label { value } => {
                let text = self.properties.as_ref().map_or_else(
                    || value.clone(),
                    |properties| localized(properties, self.culture(), value),
                );
                div().w_full().text_sm().child(text).into_any_element()
            }
            ControlKind::Slider { .. } => {
                let Some((position, row)) = self.sliders.iter().enumerate().find(|(_, row)| row.key == key) else {
                    return div().into_any_element();
                };
                let help = control.help.as_deref().filter(|help| !help.is_empty()).map(|help| {
                    self.properties
                        .as_ref()
                        .map_or_else(|| help.to_owned(), |p| localized(p, self.culture(), help))
                });
                v_flex()
                    .id(("property-slider", position))
                    .w_full()
                    .gap_1()
                    .child(
                        h_flex()
                            .gap_1()
                            .items_center()
                            .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child(label))
                            .when_some(help, |row, help| {
                                row.child(
                                    div()
                                        .id(("property-help", position))
                                        .child(
                                            Icon::new(IconName::Info)
                                                .with_size(px(12.))
                                                .text_color(theme.muted_foreground),
                                        )
                                        .tooltip(move |window, cx| Tooltip::new(help.clone()).build(window, cx)),
                                )
                            }),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Slider::new(&row.state).w(px(312.)))
                            .child(
                                div()
                                    .w(px(48.))
                                    .flex_none()
                                    .text_right()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("{:.*}", decimals(row.step), row.value)),
                            ),
                    )
                    .into_any_element()
            }
            // The other types follow in later plans; until then the line of an unknown type stands in for them.
            other => div()
                .w_full()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(tr_with("Property.Type.NotSupport", &[("type", other.type_name())]))
                .into_any_element(),
        }
    }

    /// The culture the file's texts are looked up for.
    fn culture(&self) -> Option<&str> {
        self.view.as_ref().and_then(|view| view.culture.as_deref())
    }
}

impl Drop for PropertiesEditor {
    /// Closing the window tells the service, which writes the working copy now instead of waiting for its timer.
    fn drop(&mut self) {
        if !self.folder.is_empty() {
            self.link.send(PortalRequest::CloseProperties {
                folder: std::mem::take(&mut self.folder),
            });
        }
    }
}

impl Render for PropertiesEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let monitors = self.live_monitors();
        let chip = monitors.as_ref().map_or_else(
            || tr("Property.Chip.NotRunning"),
            |monitors| tr_with("Property.Chip.Playing", &[("monitors", monitors)]),
        );
        let status = monitors.as_ref().map_or_else(
            || tr("Property.Status.NotRunning"),
            |monitors| tr_with("Property.Status.Live", &[("monitors", monitors)]),
        );
        let controls: Vec<AnyElement> = match (&self.properties, self.content) {
            (Some(properties), Content::Ready) => properties
                .list
                .iter()
                .map(|(key, control)| self.render_control(key, control, cx))
                .collect(),
            _ => Vec::new(),
        };
        let message = match self.content {
            Content::Loading => None,
            Content::Ready => None,
            Content::Empty => Some((tr("Property.Empty.Title"), Some(tr("Property.Empty.Body")))),
            Content::Failed(PropertiesFailure::NoProperties) => {
                Some((tr("Property.Empty.Title"), Some(tr("Property.Empty.Body"))))
            }
            Content::Failed(_) => Some((tr("Property.Unreadable"), None)),
        };
        v_flex()
            .id("properties-editor")
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .key_context("PropertiesEditor")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, window, _| {
                // Escape closes the window; no field of this build holds the keyboard.
                if event.keystroke.key == "escape" {
                    window.remove_window();
                }
            }))
            .child(TitleBar::new().child(div().text_xs().child(tr("Property.WindowTitle"))))
            .child(
                v_flex()
                    .h(px(56.))
                    .flex_none()
                    .justify_center()
                    .px_4()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .w_full()
                            .truncate()
                            .text_size(px(16.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .w_full()
                            .truncate()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(chip),
                    ),
            )
            .child(
                div()
                    .id("property-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p_4()
                    .child(
                        v_flex()
                            .gap_4()
                            .children(controls)
                            .when_some(message, |column, (title, body)| {
                                column
                                    .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child(title))
                                    .when_some(body, |column, body| {
                                        column.child(div().text_xs().text_color(theme.muted_foreground).child(body))
                                    })
                            }),
                    ),
            )
            .child(
                h_flex()
                    .h(px(32.))
                    .flex_none()
                    .px_4()
                    .gap_2()
                    .items_center()
                    .border_t_1()
                    .border_color(theme.border)
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(div().size(px(8.)).rounded_full().bg(if monitors.is_some() {
                        theme.primary
                    } else {
                        theme.muted_foreground
                    }))
                    .child(status),
            )
    }
}

/// The one editor window, if it is open.
#[derive(Default)]
struct EditorSlot(Option<(AnyWindowHandle, WeakEntity<PropertiesEditor>)>);

impl Global for EditorSlot {}

/// Whether the editor is the only window left: closing the Portal then ends the process.
#[must_use]
pub fn only_editor_left(cx: &App) -> bool {
    let Some(EditorSlot(Some((handle, _)))) = cx.try_global::<EditorSlot>() else {
        return false;
    };
    let windows = cx.windows();
    !windows.is_empty() && windows.iter().all(|window| window.window_id() == handle.window_id())
}

/// Hands an event of the service to the editor window, if it is open.
pub fn forward(event: &PortalEvent, cx: &mut App) {
    let Some(EditorSlot(Some((handle, view)))) = cx.try_global::<EditorSlot>() else {
        return;
    };
    let (handle, Some(view)) = (*handle, view.upgrade()) else {
        return;
    };
    let event = event.clone();
    handle
        .update(cx, move |_, window, cx| {
            view.update(cx, |editor, cx| editor.on_event(event, window, cx));
        })
        .ok();
}

/// Opens the editor window for \`folder\`, or, when it is open, shows \`folder\` in it and brings it to the front. The new
/// window goes to the corner next to the taskbar of the monitor that holds \`portal\`, the Portal's own window, and takes the
/// keyboard only when the Portal has it (a Portal started without focus opens an editor without focus too).
pub fn open_editor(folder: &str, title: &str, link: &ServiceLink, portal: &Window, cx: &mut App) {
    let focus = portal.is_window_active();
    if let Some(EditorSlot(Some((handle, view)))) = cx.try_global::<EditorSlot>() {
        let (handle, view) = (*handle, view.upgrade());
        if let Some(view) = view {
            let (folder_text, title_text) = (folder.to_owned(), title.to_owned());
            let shown = handle.update(cx, move |_, window, cx| {
                view.update(cx, |editor, cx| editor.show(&folder_text, &title_text, window, cx));
                if focus {
                    window.activate_window();
                }
            });
            if shown.is_ok() {
                return;
            }
        }
    }
    let monitors = sucrose_platform::current().monitors();
    let scale = portal.scale_factor();
    let center = portal.bounds().center();
    let held = monitor_holding(
        &monitors,
        (center.x.as_f32() * scale) as i32,
        (center.y.as_f32() * scale) as i32,
    );
    let placed = held.map(|monitor| (monitor, placement(monitor, is_rtl())));
    let display_id = placed.as_ref().and_then(|(monitor, placed)| {
        cx.displays().into_iter().find_map(|display| {
            let origin = display.bounds().origin;
            let near =
                |logical: Pixels, physical: i32| (logical.as_f32() * placed.scale - physical as f32).abs() <= 1.5;
            (near(origin.x, monitor.rect.left) && near(origin.y, monitor.rect.top)).then(|| display.id())
        })
    });
    let mut options = WindowOptions {
        window_min_size: Some(size(px(EDITOR_WIDTH), px(EDITOR_MIN_HEIGHT))),
        focus,
        is_resizable: false,
        display_id,
        titlebar: Some(TitlebarOptions {
            title: Some(tr("Property.WindowTitle")),
            ..TitleBar::title_bar_options()
        }),
        ..TitleBar::window_options()
    };
    if let Some((_, placed)) = &placed {
        let shift = *FRAME_SHIFT.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        options.window_bounds = Some(WindowBounds::Windowed(placed.bounds(shift)));
    }
    let (folder, title, link) = (folder.to_owned(), title.to_owned(), link.clone());
    let opened = gpui_kit::open_window(options, cx, move |window, cx| {
        cx.new(|cx| PropertiesEditor::new(&folder, &title, link, window, cx))
    });
    let (handle, view) = match opened {
        Ok(opened) => opened,
        Err(error) => {
            tracing::error!(%error, "cannot open the properties editor");
            return;
        }
    };
    cx.set_global(EditorSlot(Some((handle, view.downgrade()))));
    let Some((_, placed)) = placed else {
        tracing::info!("editor placed by the OS: the monitors are not known");
        return;
    };
    // The window reports its bounds once the OS has placed it; the placement is checked against them a moment later.
    cx.spawn(async move |cx| {
        cx.background_executor().timer(Duration::from_millis(300)).await;
        let read = handle.update(cx, |_, window, _| (window.bounds(), window.scale_factor()));
        if let Ok((bounds, scale)) = read {
            let read_back = (
                (bounds.origin.x.as_f32() * scale).round() as i32,
                (bounds.origin.y.as_f32() * scale).round() as i32,
            );
            let off = (placed.origin.0 - read_back.0, placed.origin.1 - read_back.1);
            let agree = off.0.abs() <= 2 && off.1.abs() <= 2;
            if !agree {
                // The next window of this run asks for the origin moved by what this one missed.
                let mut shift = FRAME_SHIFT.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                if *shift == (0, 0) {
                    *shift = off;
                }
            }
            tracing::info!(
                origin = ?placed.origin,
                size = ?placed.size,
                bounds = ?(read_back, (bounds.size.width.as_f32() * scale).round() as i32, (bounds.size.height.as_f32() * scale).round() as i32),
                edge = ?placed.edge,
                scale = placed.scale,
                monitor = %placed.monitor,
                agree,
                "editor placed"
            );
        }
    })
    .detach();
}

#[cfg(test)]
mod tests {
    // Not `use super::*`: the module glob-imports GPUI, whose `test` macro would shadow the built-in attribute.
    use super::{
        Content, Gate, MonitorInfo, PortalEvent, PortalRequest, PropertiesEditor, PropertiesFailure, PropertiesView,
        RectPx, SEND_INTERVAL, SendGate, ServiceLink, SliderEvent, TaskbarEdge, decimals, editor_height, editor_origin,
        localized, monitor_holding, monitor_number, parse_properties, placement, snapped, taskbar_edge,
    };
    use gpui_kit::{Entity, TestAppContext, VisualTestContext};
    use std::time::{Duration, Instant};

    const RECT: RectPx = RectPx {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };

    fn work(left: i32, top: i32, right: i32, bottom: i32) -> RectPx {
        RectPx {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn the_taskbar_edge_is_the_side_where_the_work_area_is_smaller() {
        assert_eq!(taskbar_edge(RECT, work(0, 0, 1920, 1032)), TaskbarEdge::Bottom);
        assert_eq!(taskbar_edge(RECT, work(48, 0, 1920, 1080)), TaskbarEdge::Left);
        assert_eq!(taskbar_edge(RECT, work(0, 0, 1872, 1080)), TaskbarEdge::Right);
        assert_eq!(taskbar_edge(RECT, work(0, 48, 1920, 1080)), TaskbarEdge::Top);
        assert_eq!(taskbar_edge(RECT, RECT), TaskbarEdge::None);
    }

    #[test]
    fn the_editor_goes_to_the_corner_next_to_the_taskbar() {
        let bottom = work(0, 0, 1920, 1032);
        assert_eq!(
            editor_origin(bottom, TaskbarEdge::Bottom, (400, 600), false, 16),
            (1504, 416)
        );
        assert_eq!(
            editor_origin(bottom, TaskbarEdge::Bottom, (400, 600), true, 16),
            (16, 416)
        );
        let left = work(48, 0, 1920, 1080);
        assert_eq!(editor_origin(left, TaskbarEdge::Left, (400, 600), false, 16), (64, 464));
        assert_eq!(
            editor_origin(left, TaskbarEdge::Left, (400, 600), true, 16),
            (64, 464),
            "next to the bar in RTL too"
        );
        let right = work(0, 0, 1872, 1080);
        assert_eq!(
            editor_origin(right, TaskbarEdge::Right, (400, 600), false, 16),
            (1456, 464)
        );
        assert_eq!(
            editor_origin(right, TaskbarEdge::Right, (400, 600), true, 16),
            (1456, 464)
        );
        let top = work(0, 48, 1920, 1080);
        assert_eq!(editor_origin(top, TaskbarEdge::Top, (400, 600), false, 16), (1504, 64));
        assert_eq!(editor_origin(top, TaskbarEdge::Top, (400, 600), true, 16), (16, 64));
        assert_eq!(
            editor_origin(RECT, TaskbarEdge::None, (400, 600), false, 16),
            (1504, 464)
        );
        assert_eq!(editor_origin(RECT, TaskbarEdge::None, (400, 600), true, 16), (16, 464));
    }

    #[test]
    fn the_editor_height_stays_between_the_minimum_and_the_work_area_minus_32() {
        assert!((editor_height(100.0, 1000.0) - 320.0).abs() < f32::EPSILON);
        assert!((editor_height(600.0, 1000.0) - 600.0).abs() < f32::EPSILON);
        assert!((editor_height(2000.0, 1000.0) - 968.0).abs() < f32::EPSILON);
        assert!(
            (editor_height(2000.0, 300.0) - 320.0).abs() < f32::EPSILON,
            "a tiny work area keeps the minimum"
        );
    }

    fn monitor(id: &str, rect: RectPx, work: RectPx, dpi: u32, primary: bool) -> MonitorInfo {
        MonitorInfo {
            id: id.to_owned(),
            device_path: None,
            edid: None,
            friendly: None,
            rect,
            work,
            dpi,
            primary,
        }
    }

    #[test]
    fn the_monitor_that_holds_the_portal_is_found_by_its_centre_and_the_primary_stands_in() {
        let first = monitor("\\\\.\\DISPLAY1", RECT, work(0, 0, 1920, 1032), 96, true);
        let second = monitor(
            "\\\\.\\DISPLAY9",
            work(1920, 0, 3840, 1080),
            work(1920, 0, 3840, 1080),
            96,
            false,
        );
        let both = [first.clone(), second.clone()];

        assert_eq!(monitor_holding(&both, 100, 100), Some(&first));
        assert_eq!(monitor_holding(&both, 2500, 500), Some(&second));
        assert_eq!(
            monitor_holding(&both, -50, 0),
            Some(&first),
            "outside every monitor: the primary"
        );
        assert_eq!(monitor_holding(&[], 0, 0), None);
    }

    #[test]
    fn the_placement_follows_the_monitor_dpi_and_the_taskbar() {
        let at_100 = monitor("\\\\.\\DISPLAY1", RECT, work(0, 0, 1920, 1032), 96, true);
        let placed = placement(&at_100, false);
        assert_eq!(placed.edge, TaskbarEdge::Bottom);
        assert_eq!(placed.size, (400, 480));
        assert_eq!(placed.origin, (1920 - 16 - 400, 1032 - 16 - 480));
        let bounds = placed.bounds((0, 0));
        assert!((bounds.origin.x.as_f32() - 1504.0).abs() < 0.01);
        let shifted = placed.bounds((0, 4));
        assert!((shifted.origin.y.as_f32() - (placed.origin.1 + 4) as f32).abs() < 0.01);
        assert!((bounds.size.width.as_f32() - 400.0).abs() < 0.01);

        // At 150 % the monitor has 3840 x 2160 physical pixels: the window is 600 x 720 physical, 400 x 480 logical.
        let at_150 = monitor(
            "\\\\.\\DISPLAY2",
            work(0, 0, 3840, 2160),
            work(0, 0, 3840, 2064),
            144,
            false,
        );
        let placed = placement(&at_150, false);
        assert_eq!(placed.size, (600, 720));
        assert_eq!(
            placed.origin,
            (3840 - 24 - 600, 2064 - 24 - 720),
            "the inset scales too"
        );
        let bounds = placed.bounds((0, 0));
        assert!((bounds.size.width.as_f32() - 400.0).abs() < 0.01);
        assert!((bounds.size.height.as_f32() - 480.0).abs() < 0.01);
        assert!((bounds.origin.x.as_f32() * placed.scale - placed.origin.0 as f32).abs() < 0.01);
    }

    #[test]
    fn the_placement_of_a_small_work_area_keeps_the_window_inside_it() {
        let small = monitor("\\\\.\\DISPLAY3", work(0, 0, 800, 440), work(0, 0, 800, 400), 96, true);

        let placed = placement(&small, false);

        assert_eq!(placed.size.1, 368, "work height minus 32");
        assert_eq!(placed.origin, (800 - 16 - 400, 400 - 16 - 368));
    }

    #[test]
    fn a_drag_sends_at_most_one_change_per_interval_and_the_wait_ends_at_the_interval() {
        let start = Instant::now();
        let mut gate = SendGate::default();
        let mut sent = vec![];
        for millis in (0..=300).step_by(10) {
            let now = start + Duration::from_millis(millis);
            match gate.offer(now) {
                Gate::Send => sent.push(millis),
                Gate::Wait(wait) => assert!(wait <= SEND_INTERVAL && !wait.is_zero(), "{wait:?}"),
            }
        }

        assert_eq!(sent, [0, 50, 100, 150, 200, 250, 300]);
        assert_eq!(
            gate.offer(start + Duration::from_millis(310)),
            Gate::Wait(Duration::from_millis(40))
        );
        gate.sent(start + Duration::from_millis(350));
        assert_eq!(
            gate.offer(start + Duration::from_millis(360)),
            Gate::Wait(Duration::from_millis(40))
        );
    }

    #[test]
    fn a_value_is_held_inside_its_bounds_and_snapped_to_the_step() {
        assert!((snapped(2.3, -5.0, 5.0, 0.5) - 2.5).abs() < 1e-9);
        assert!((snapped(2.2, -5.0, 5.0, 0.5) - 2.0).abs() < 1e-9);
        assert!((snapped(99.0, -5.0, 5.0, 0.5) - 5.0).abs() < 1e-9);
        assert!((snapped(-99.0, -5.0, 5.0, 0.5) + 5.0).abs() < 1e-9);
        assert!((snapped(0.30000000000000004, 0.0, 1.0, 0.1) - 0.3).abs() < 1e-12);
        assert!(
            (snapped(3.7, 0.0, 10.0, 0.0) - 3.7).abs() < 1e-9,
            "no step: only the bounds"
        );
        assert!(
            (snapped(1.0, 5.0, -5.0, 1.0) - 1.0).abs() < 1e-9,
            "reversed bounds are read the other way round"
        );
    }

    #[test]
    fn the_readout_has_the_decimals_of_the_step() {
        assert_eq!(decimals(1.0), 0);
        assert_eq!(decimals(0.5), 1);
        assert_eq!(decimals(0.25), 2);
        assert_eq!(decimals(0.001), 3);
    }

    #[test]
    fn a_text_is_looked_up_for_the_culture_then_the_first_table_and_otherwise_stays() {
        let properties = parse_properties(
            r#"{"PropertyListener":"L({0},{1})","PropertyList":{},"PropertyLocalization":{
                "en":{"amount":"Amount","only-en":"Only English"},
                "tr":{"amount":"Miktar"}}}"#,
        )
        .expect("parse");

        assert_eq!(localized(&properties, Some("TR"), "amount"), "Miktar");
        assert_eq!(
            localized(&properties, Some("de"), "amount"),
            "Amount",
            "the first table"
        );
        assert_eq!(
            localized(&properties, Some("tr"), "only-en"),
            "Only English",
            "the culture's table lacks it: the first table has it"
        );
        assert_eq!(localized(&properties, Some("tr"), "unknown"), "unknown");
        assert_eq!(localized(&properties, None, "plain text"), "plain text");
    }

    #[test]
    fn a_monitor_number_is_the_digits_at_the_end_of_its_device_name() {
        assert_eq!(monitor_number("\\\\.\\DISPLAY9"), "9");
        assert_eq!(monitor_number("\\\\.\\DISPLAY12"), "12");
        assert_eq!(monitor_number("eDP"), "eDP");
    }

    const DOCUMENT: &str = r#"{
      "PropertyListener": "SucrosePropertyListener('{0}', {1});",
      "PropertyList": {
        "title": { "type": "label", "text": "Calls", "value": "Calls" },
        "amount": { "type": "slider", "text": "Amount", "help": "How much", "min": -5, "max": 5, "step": 0.5, "value": 1 },
        "name": { "type": "textbox", "text": "Name", "value": "x" }
      }
    }"#;

    fn view(folder: &str, blob_id: u32) -> PropertiesView {
        PropertiesView {
            folder: folder.to_owned(),
            kind: "Base".to_owned(),
            blob_id,
            culture: None,
            live_on: vec!["\\\\.\\DISPLAY1".to_owned()],
            engine: None,
        }
    }

    /// Gives the editor the answer of the service for \`folder\`: the view and the document in chunks of \`chunk\` bytes.
    fn answer(editor: &Entity<PropertiesEditor>, folder: &str, blob: u32, text: &str, window: &mut VisualTestContext) {
        let parts: Vec<&[u8]> = text.as_bytes().chunks(60).collect();
        let total = u16::try_from(parts.len()).expect("a few chunks");
        editor.update_in(window, |editor, window, cx| {
            editor.on_event(PortalEvent::Properties(view(folder, blob)), window, cx);
            for (seq, part) in (0..).zip(parts) {
                editor.on_event(
                    PortalEvent::PropertiesBlob {
                        id: blob,
                        seq,
                        total,
                        bytes: part.to_vec(),
                    },
                    window,
                    cx,
                );
            }
        });
    }

    fn requests(queue: &crossbeam_channel::Receiver<PortalRequest>) -> Vec<PortalRequest> {
        queue.try_iter().collect()
    }

    #[gpui_kit::test]
    fn the_editor_asks_for_its_document_builds_the_slider_and_shows_a_second_theme_in_the_same_window(
        cx: &mut TestAppContext,
    ) {
        cx.update(gpui_kit::init);
        let (link, queue) = ServiceLink::recording();
        let (editor, window) = {
            let link = link.clone();
            cx.add_window_view(move |window, cx| PropertiesEditor::new("calls", "Calls", link, window, cx))
        };
        let editor = editor;
        assert_eq!(
            requests(&queue),
            [PortalRequest::OpenProperties {
                folder: "calls".to_owned()
            }]
        );

        answer(&editor, "calls", 1, DOCUMENT, window);
        window.run_until_parked();

        editor.read_with(window, |editor, _| {
            assert_eq!(editor.content, Content::Ready);
            assert_eq!(
                editor.sliders.len(),
                1,
                "one slider; the textbox waits for a later plan"
            );
            assert_eq!(editor.sliders[0].key, "amount");
            assert_eq!(editor.folder(), "calls");
        });
        assert!(requests(&queue).is_empty(), "opening sends nothing else");

        // Another theme replaces the content: the first is closed (the service saves it) and the second asked for.
        editor.update_in(window, |editor, window, cx| editor.show("rain", "Rain", window, cx));
        assert_eq!(
            requests(&queue),
            [
                PortalRequest::CloseProperties {
                    folder: "calls".to_owned()
                },
                PortalRequest::OpenProperties {
                    folder: "rain".to_owned()
                }
            ]
        );
        editor.read_with(window, |editor, _| {
            assert_eq!(editor.content, Content::Loading);
            assert!(
                editor.sliders.is_empty() && editor.properties.is_none(),
                "the controls are dropped"
            );
        });
        // A late chunk of the first document is ignored.
        answer(&editor, "calls", 1, DOCUMENT, window);
        editor.read_with(window, |editor, _| assert_eq!(editor.content, Content::Loading));

        drop(editor);
    }

    #[gpui_kit::test]
    fn the_release_of_a_slider_sends_the_snapped_value_as_a_whole_control(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (link, queue) = ServiceLink::recording();
        let (editor, window) = {
            let link = link.clone();
            cx.add_window_view(move |window, cx| PropertiesEditor::new("calls", "Calls", link, window, cx))
        };
        answer(&editor, "calls", 1, DOCUMENT, window);
        window.run_until_parked();
        let _ = requests(&queue);
        let state = editor.read_with(window, |editor, _| editor.sliders[0].state.clone());

        state.update_in(window, |state, window, cx| {
            state.set_value(2.3, window, cx);
        });
        editor.update_in(window, |editor, window, cx| {
            editor.on_slider(0, &SliderEvent::Release(2.3.into()), window, cx);
        });

        let sent = requests(&queue);
        assert_eq!(sent.len(), 1, "{sent:?}");
        let PortalRequest::ChangeProperty { folder, key, control } = &sent[0] else {
            panic!("not a change: {sent:?}");
        };
        assert_eq!((folder.as_str(), key.as_str()), ("calls", "amount"));
        let control: serde_json::Value = serde_json::from_str(control).expect("the control is JSON");
        assert_eq!(control["type"], "slider");
        assert_eq!(control["value"], 2.5, "snapped to the step of 0.5");
        assert_eq!(control["min"], -5.0);
        // A release with no new value sends nothing twice.
        editor.update_in(window, |editor, window, cx| {
            editor.on_slider(0, &SliderEvent::Release(2.3.into()), window, cx);
        });
        assert_eq!(requests(&queue).len(), 1, "the same value is the same change");
    }

    #[gpui_kit::test]
    fn a_failed_document_and_an_unreadable_one_show_their_message(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (link, _queue) = ServiceLink::recording();
        let (editor, window) =
            cx.add_window_view(move |window, cx| PropertiesEditor::new("calls", "Calls", link, window, cx));

        editor.update_in(window, |editor, window, cx| {
            editor.on_event(
                PortalEvent::PropertiesFailed {
                    folder: "calls".to_owned(),
                    reason: PropertiesFailure::NoProperties,
                },
                window,
                cx,
            );
        });
        editor.read_with(window, |editor, _| {
            assert_eq!(editor.content, Content::Failed(PropertiesFailure::NoProperties));
        });
        window.run_until_parked();
        answer(&editor, "calls", 2, "{ not json", window);
        editor.read_with(window, |editor, _| {
            assert_eq!(editor.content, Content::Failed(PropertiesFailure::Unreadable));
        });
        window.run_until_parked();
    }
}
