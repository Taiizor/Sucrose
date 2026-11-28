//! One theme card (UI-SPEC E2): the cover-fit thumbnail, the title strip and the newer-version pill.

use std::{rc::Rc, sync::Arc};

use gpui_kit::assets::IconName;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, skeleton::Skeleton, spinner::Spinner, tooltip::Tooltip};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use sucrose_core::{
    library::{Compatibility, LibraryTheme},
    settings::DisplayScreenType,
};
use sucrose_ipc::portal::{DisplayView, EngineReportView, PortalRequest, RuleState, RuntimeState};

use super::layout::{Arrow, CARD_HEIGHT};
use crate::{
    i18n::{tr, tr_count_with, tr_with},
    ipc::ServiceLink,
};

/// Height of the strip with the title and the description.
const STRIP_HEIGHT: f32 = 56.0;

/// What the card shows where the thumbnail goes.
pub enum Thumb {
    /// The decoded thumbnail.
    Ready(Arc<RenderImage>),
    /// Wanted and waiting for its decode: the animated skeleton.
    Loading,
    /// Not requested: the card is not near the view; a static fill, so it asks for no animation frames.
    Idle,
    /// It cannot be decoded.
    Failed,
}

/// What a click on a card does.
pub type UseHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// What the pointer entering (`true`) or leaving (`false`) a card does.
pub type HoverHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// What an arrow key on a focused card does.
pub type ArrowHandler = Rc<dyn Fn(Arrow, &mut Window, &mut App)>;

/// Everything one card needs to draw itself.
pub struct CardData {
    /// Position in the sorted list, for element ids.
    pub index: usize,
    /// The title to show.
    pub title: SharedString,
    /// The description to show, when the theme has one.
    pub description: Option<SharedString>,
    /// The thumbnail state.
    pub thumb: Thumb,
    /// Whether the theme was made for a newer Sucrose.
    pub needs_newer_app: bool,
    /// Whether this theme is the selected one and the wallpaper plays (the "Selected" pill).
    pub selected: bool,
    /// Where the theme plays now, one pill each right of the Selected pill (UI-SPEC E6); decoration, never a button.
    pub badges: Vec<Badge>,
    /// Whether a Use request for this theme is in flight (the spinner).
    pub pending: bool,
    /// Whether Use works on this card, and when it does not, why (the hand cursor, the hover ring and the tooltip).
    pub state: CardUse,
    /// The card's focus handle: a tab stop, so the keyboard reaches disabled cards too.
    pub focus: FocusHandle,
    /// What a click, or Enter and Space on the focused card, does; it checks again whether Use is allowed.
    pub on_use: UseHandler,
    /// What an arrow key does: moves focus to the neighboring card.
    pub on_arrow: ArrowHandler,
    /// The animated preview that plays on this card, shown above the thumbnail. It is painted with the element id
    /// `("library-card-preview", index)`: the id is what makes GPUI advance the frames of a multi-frame image.
    pub preview: Option<Arc<RenderImage>>,
    /// Whether the strip hides while the preview plays (`LibraryPreviewHide`).
    pub preview_hide: bool,
    /// What the pointer entering or leaving the card does (the hover preview).
    pub on_hover: HoverHandler,
}

/// The most monitor badges a card draws before the rest collapse into one "+N" badge (UI-SPEC Library card).
pub const MAX_MONITOR_BADGES: usize = 2;

/// One badge on a card: where the theme plays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Badge {
    /// The theme plays on the monitor with this number.
    Monitor(u8),
    /// The theme plays on every monitor (Span and Duplicate).
    All,
    /// The numbers of the monitors beyond [`MAX_MONITOR_BADGES`], behind one "+N" badge.
    More(Vec<u8>),
}

/// The badges of the card of `folder`: one per connected monitor that plays it now, at most [`MAX_MONITOR_BADGES`], then
/// `More`; one `All` in Span and Duplicate, where the single wallpaper covers every monitor; none when it does not play.
pub fn monitor_badges(folder: &str, displays: &[DisplayView], mode: DisplayScreenType) -> Vec<Badge> {
    let mut numbers = displays
        .iter()
        .filter(|display| display.connected && display.running && display.theme.as_deref() == Some(folder))
        .map(|display| display.number);
    let Some(first) = numbers.next() else {
        return Vec::new();
    };
    if mode != DisplayScreenType::PerDisplay {
        return vec![Badge::All];
    }
    let mut badges = vec![Badge::Monitor(first)];
    let mut rest = Vec::new();
    for number in numbers {
        if badges.len() < MAX_MONITOR_BADGES {
            badges.push(Badge::Monitor(number));
        } else {
            rest.push(number);
        }
    }
    if !rest.is_empty() {
        badges.push(Badge::More(rest));
    }
    badges
}

impl Badge {
    /// What the pill shows next to the Monitor icon: the monitor number, "All" or "+N".
    pub fn text(&self) -> SharedString {
        match self {
            Self::Monitor(number) => SharedString::from(number.to_string()),
            Self::All => tr("Portal.LibraryCard.MonitorBadgeAllLabel"),
            Self::More(rest) => SharedString::from(format!("+{}", rest.len())),
        }
    }

    /// The accessible description of the badge; the tooltip of the "+N" badge, which lists the monitors it stands for.
    pub fn description(&self) -> SharedString {
        match self {
            Self::Monitor(number) => tr_with("Portal.LibraryCard.MonitorBadge", &[("number", &number.to_string())]),
            Self::All => tr("Portal.LibraryCard.MonitorBadgeAll"),
            Self::More(rest) => {
                let list = rest.iter().map(u8::to_string).collect::<Vec<_>>().join(", ");
                tr_count_with("Portal.LibraryCard.MonitorBadgeMore", rest.len(), &[("list", &list)])
            }
        }
    }

    /// Whether the pill starts with the Monitor icon ("+N" is text only).
    fn has_icon(&self) -> bool {
        !matches!(self, Self::More(_))
    }
}

/// Whether the title strip shows: it hides only while a preview plays and `LibraryPreviewHide` is on (LIB-06).
pub fn strip_visible(preview_playing: bool, preview_hide: bool) -> bool {
    !(preview_playing && preview_hide)
}

/// Why Use is unavailable on a card (UI-SPEC: Use disabled reasons).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisabledReason {
    /// The service is not connected.
    Unavailable,
    /// The theme was made for a newer Sucrose (LIB-08, D-18).
    NewerVersion,
    /// No slice of this build plays the theme's type yet (D-31): the service's report does not list it as supported.
    UnsupportedType,
    /// The type is supported but no installed engine can play it now (UI-SPEC reason 4).
    NoEngine,
    /// The theme is the selected one and the wallpaper plays.
    Selected,
    /// A fullscreen rule paused the wallpaper (D-23).
    Paused,
    /// A fullscreen rule closed the wallpaper (D-23).
    Closed,
    /// The library is being moved to another folder (LIB-09); the service refuses Use until it ends.
    Moving,
}

impl DisabledReason {
    /// The tooltip and the accessible description of a disabled card.
    pub fn tooltip(self) -> SharedString {
        match self {
            Self::Unavailable => use_with("Portal.Connection.Unavailable"),
            Self::NewerVersion => tr("Portal.LibraryCard.VersionTip"),
            Self::UnsupportedType => tr("Portal.LibraryCard.UnsupportedType"),
            Self::NoEngine => tr("Portal.LibraryCard.NoEngine"),
            Self::Selected => use_with("Portal.LibraryCard.Selected"),
            Self::Paused => use_with("Portal.LibraryCard.Paused"),
            Self::Closed => use_with("Portal.LibraryCard.Closed"),
            Self::Moving => tr("Portal.GeneralSettingPage.PrivateLibrary.LibraryLocation.Move"),
        }
    }
}

/// "Use (<reason>)" from the text of `reason_key`.
fn use_with(reason_key: &str) -> SharedString {
    SharedString::from(format!("{} ({})", tr("Portal.LibraryCard.MenuUse"), tr(reason_key)))
}

/// Whether a click on a card applies its theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardUse {
    /// A click sends Use.
    Enabled,
    /// A click does nothing, for this reason.
    Disabled(DisabledReason),
}

impl CardUse {
    /// Whether a click sends Use.
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

/// Whether Use works on `theme` now, in the priority of the UI-SPEC: no connection, a library move, a newer theme, a type the
/// service's `engines` report does not support, a supported type no installed engine plays, the theme that plays already,
/// then a fullscreen rule. Until the report arrives nothing can be used. A selected theme that does not run can be used again.
pub fn use_state(
    theme: &LibraryTheme,
    runtime: Option<&RuntimeState>,
    engines: Option<&EngineReportView>,
    connected: bool,
) -> CardUse {
    if !connected {
        return CardUse::Disabled(DisabledReason::Unavailable);
    }
    // The library is on its way to another folder: nothing in it can be used until the move ends.
    if runtime.is_some_and(|runtime| runtime.library_moving) {
        return CardUse::Disabled(DisabledReason::Moving);
    }
    if matches!(theme.compatibility, Compatibility::NeedsNewerApp { .. }) {
        return CardUse::Disabled(DisabledReason::NewerVersion);
    }
    // Which types play is the service's word, never the Portal's (UI-SPEC: never hard-codes what is playable).
    let Some(engines) = engines else {
        return CardUse::Disabled(DisabledReason::Unavailable);
    };
    if !engines.supported.contains(&theme.info.kind) {
        return CardUse::Disabled(DisabledReason::UnsupportedType);
    }
    if !engines.playable.contains(&theme.info.kind) {
        return CardUse::Disabled(DisabledReason::NoEngine);
    }
    let Some(runtime) = runtime else {
        return CardUse::Enabled;
    };
    if runtime.running && runtime.selected.as_deref() == Some(&theme.folder) {
        return CardUse::Disabled(DisabledReason::Selected);
    }
    match runtime.rule {
        RuleState::None => CardUse::Enabled,
        RuleState::Paused => CardUse::Disabled(DisabledReason::Paused),
        RuleState::Closed => CardUse::Disabled(DisabledReason::Closed),
    }
}

/// Asks the service to make `folder` the wallpaper (D-11): the Portal changes nothing itself and waits for the broadcast.
pub fn send_use(link: &ServiceLink, folder: &str) {
    link.send(PortalRequest::Use {
        folder: folder.to_owned(),
    });
}

/// The card for `data`, `width` wide.
pub fn render_card(data: CardData, width: Pixels, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let index = data.index;
    let on_use = Rc::clone(&data.on_use);
    let on_arrow = Rc::clone(&data.on_arrow);
    let on_hover = Rc::clone(&data.on_hover);
    let show_strip = strip_visible(data.preview.is_some(), data.preview_hide);
    let selected = data.selected;
    let badge_pills: Vec<Stateful<Div>> = data
        .badges
        .into_iter()
        .enumerate()
        .map(|(position, badge)| render_badge(index, position, badge))
        .collect();
    let label = data.title.clone();
    let enabled = data.state.is_enabled();
    let disabled_reason = match data.state {
        CardUse::Enabled => None,
        CardUse::Disabled(reason) => Some(reason.tooltip()),
    };
    let ring = theme.primary.opacity(0.55);
    let focus_ring = theme.primary;
    let layer = match data.thumb {
        Thumb::Ready(image) => img(image)
            .absolute()
            .inset_0()
            .size_full()
            .object_fit(ObjectFit::Cover)
            .into_any_element(),
        Thumb::Loading => Skeleton::new().absolute().inset_0().size_full().into_any_element(),
        Thumb::Idle => div()
            .absolute()
            .inset_0()
            .size_full()
            .bg(theme.skeleton)
            .into_any_element(),
        Thumb::Failed => div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.secondary)
            .child(
                Icon::new(IconName::ImageOff)
                    .with_size(px(28.))
                    .text_color(theme.muted_foreground),
            )
            .into_any_element(),
    };
    // The strip sits on the image: its colors are the overlay exceptions of the UI-SPEC.
    let strip = div()
        .absolute()
        .left_0()
        .right_0()
        .bottom_0()
        .h(px(STRIP_HEIGHT))
        .flex()
        .flex_col()
        .justify_end()
        .overflow_hidden()
        .px_3()
        .py_2()
        .bg(linear_gradient(
            180.,
            linear_color_stop(black().opacity(0.30), 0.),
            linear_color_stop(black().opacity(0.75), 1.),
        ))
        .child(
            div()
                .w_full()
                .text_size(px(16.))
                .line_height(px(20.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(white())
                .truncate()
                .child(data.title),
        )
        .when_some(data.description, |strip, description| {
            strip.child(
                div()
                    .w_full()
                    .text_size(px(12.))
                    .line_height(px(16.))
                    .text_color(white().opacity(0.82))
                    .truncate()
                    .child(description),
            )
        });
    div()
        .id(("library-card", index))
        .relative()
        .flex_none()
        .w(width)
        .h(px(CARD_HEIGHT))
        .rounded(px(8.))
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .overflow_hidden()
        .track_focus(&data.focus)
        .focus_visible(move |style| style.border_2().border_color(focus_ring))
        .role(Role::Button)
        .aria_label(label)
        .when(enabled, |card| {
            // The 2 px accent ring at 55 % and the hand cursor only where Use is possible (UI-SPEC E2).
            card.cursor_pointer()
                .hover(move |style| style.border_2().border_color(ring))
        })
        .when_some(disabled_reason, |card, reason| {
            // A disabled card stays focusable and says why, as its tooltip and as its accessible description.
            card.aria_description(reason.clone())
                .tooltip(move |window, cx| Tooltip::new(reason.clone()).build(window, cx))
        })
        // A click, and Enter or Space on the focused card (GPUI turns those into a click), are Use.
        .on_click(move |_, window, cx| on_use(window, cx))
        .on_hover(move |hovered, window, cx| on_hover(*hovered, window, cx))
        .on_key_down(move |event, window, cx| {
            if let Some(arrow) = Arrow::from_key(&event.keystroke.key) {
                cx.stop_propagation();
                on_arrow(arrow, window, cx);
            }
        })
        .child(layer)
        // The preview plays above the static thumbnail, which stays underneath.
        .when_some(data.preview, |card, image| {
            // The element id is what makes GPUI keep the frame index of a multi-frame image and request animation frames.
            card.child(
                img(image)
                    .id(("library-card-preview", index))
                    .absolute()
                    .inset_0()
                    .size_full()
                    .object_fit(ObjectFit::Cover),
            )
        })
        .when(show_strip, |card| card.child(strip))
        // The Selected pill and, right of it with an 8 px gap, the monitor badges (UI-SPEC E6).
        .when(selected || !badge_pills.is_empty(), |card| {
            card.child(
                div()
                    .id(("library-card-pills", index))
                    .absolute()
                    .top_2()
                    .left_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(selected, |row| {
                        row.child(
                            div()
                                .id(("library-card-selected", index))
                                .flex()
                                .items_center()
                                .gap_1()
                                .h(px(24.))
                                .px_2()
                                .rounded_full()
                                .bg(theme.primary)
                                .text_color(theme.primary_foreground)
                                .text_size(px(12.))
                                .child(Icon::new(IconName::Check).with_size(px(14.)))
                                .child(tr("Portal.LibraryCard.Selected")),
                        )
                    })
                    .children(badge_pills),
            )
        })
        .when(data.pending, |card| {
            card.child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(black().opacity(0.35))
                    .child(Spinner::new().with_size(px(28.)).color(theme.primary)),
            )
        })
        .when(data.needs_newer_app, |card| {
            card.child(
                div()
                    .id(("library-card-newer", index))
                    .absolute()
                    .top_2()
                    .right_2()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(24.))
                    .px_2()
                    .rounded_full()
                    .bg(theme.danger)
                    .text_color(theme.danger_foreground)
                    .child(Icon::new(IconName::PackageX).with_size(px(14.)))
                    .tooltip(|window, cx| Tooltip::new(tr("Portal.LibraryCard.VersionTip")).build(window, cx)),
            )
        })
}

/// One monitor badge: a neutral pill (black at 55 % with white text, an image-overlay exception), 20 px high, with the
/// Monitor icon and the number, "All" or "+N". It is decoration with an accessible description and no click of its own,
/// so a click on it reaches the card; only the "+N" badge has a tooltip, which lists the monitors it stands for.
fn render_badge(card: usize, position: usize, badge: Badge) -> Stateful<Div> {
    let description = badge.description();
    let list = matches!(badge, Badge::More(_)).then(|| description.clone());
    div()
        // Up to MAX_MONITOR_BADGES + 1 badges per card, so card * 4 + position never repeats on a page.
        .id(("library-card-badge", card * 4 + position))
        .flex()
        .items_center()
        .gap_1()
        .h(px(20.))
        .px_2()
        .rounded_full()
        .bg(black().opacity(0.55))
        .text_color(white())
        .text_size(px(12.))
        .role(Role::Image)
        .aria_label(description)
        .when(badge.has_icon(), |pill| {
            pill.child(Icon::new(IconName::Monitor).with_size(px(12.)))
        })
        .child(badge.text())
        .when_some(list, |pill, text| {
            pill.tooltip(move |window, cx| Tooltip::new(text.clone()).build(window, cx))
        })
}

/// A card-sized placeholder for the grid that shows while the library is scanned.
pub fn render_skeleton_card(width: Pixels) -> impl IntoElement {
    Skeleton::new().flex_none().w(width).h(px(CARD_HEIGHT)).rounded(px(8.))
}

#[cfg(test)]
mod tests {
    // Not `use super::*`: the crate root glob-imports GPUI, whose `test` macro would shadow the built-in attribute.
    use super::{
        Badge, CardData, CardUse, DisabledReason, Thumb, monitor_badges, render_card, strip_visible, use_state,
    };
    use gpui_kit::{
        Context, FocusHandle, ImageSource, IntoElement, Modifiers, Render, RenderImage, Styled as _, TestAppContext,
        VisualTestContext, Window, img, point, px,
    };
    use image::{Delay, Frame, Rgba, RgbaImage};
    use std::{cell::Cell, path::PathBuf, rc::Rc, sync::Arc, time::Duration};
    use sucrose_core::settings::DisplayScreenType;
    use sucrose_core::{
        library::{Compatibility, LibraryTheme},
        theme::{SucroseInfo, WallpaperType},
        version::LegacyVersion,
    };
    use sucrose_ipc::portal::{DisplayView, EngineReportView, RuleState, RuntimeState};

    fn theme(kind: WallpaperType, compatibility: Compatibility) -> LibraryTheme {
        LibraryTheme {
            folder: "Rain".to_owned(),
            dir: PathBuf::from("Rain"),
            info: SucroseInfo {
                app_version: LegacyVersion::default(),
                version: LegacyVersion::default(),
                title: "Rain".to_owned(),
                thumbnail: String::new(),
                preview: String::new(),
                description: String::new(),
                author: String::new(),
                license: None,
                contact: String::new(),
                kind,
                source: String::new(),
                tags: None,
                arguments: None,
                localization: None,
                extra: Default::default(),
            },
            thumbnail: PathBuf::new(),
            preview: PathBuf::new(),
            compatibility,
            created: None,
            modified: None,
            index: Vec::new(),
        }
    }

    fn video() -> LibraryTheme {
        theme(WallpaperType::Video, Compatibility::Compatible)
    }

    fn newer(kind: WallpaperType) -> LibraryTheme {
        theme(
            kind,
            Compatibility::NeedsNewerApp {
                required: LegacyVersion::new(99, 0, -1, -1),
            },
        )
    }

    fn runtime(selected: Option<&str>, running: bool, rule: RuleState) -> RuntimeState {
        RuntimeState {
            selected: selected.map(str::to_owned),
            running,
            rule,
            library_moving: false,
            tray_visible: true,
            running_engines: Vec::new(),
            displays: Vec::new(),
        }
    }

    /// The report of a machine where mpv and CEF are installed: Video and Web play.
    fn ready() -> EngineReportView {
        EngineReportView {
            engines: Vec::new(),
            supported: vec![WallpaperType::Video, WallpaperType::Web],
            playable: vec![WallpaperType::Video, WallpaperType::Web],
            offered: Vec::new(),
            fallbacks: Vec::new(),
        }
    }

    fn state(theme: &LibraryTheme, runtime: &RuntimeState, connected: bool) -> CardUse {
        use_state(theme, Some(runtime), Some(&ready()), connected)
    }

    #[test]
    fn a_compatible_video_theme_can_be_used() {
        let idle = runtime(None, false, RuleState::None);
        assert_eq!(state(&video(), &idle, true), CardUse::Enabled);
        assert_eq!(
            use_state(&video(), None, Some(&ready()), true),
            CardUse::Enabled,
            "no runtime report yet is not a reason"
        );
    }

    #[test]
    fn a_web_theme_can_be_used_when_the_service_reports_web_as_playable() {
        let idle = runtime(None, false, RuleState::None);
        let web = theme(WallpaperType::Web, Compatibility::Compatible);
        assert_eq!(state(&web, &idle, true), CardUse::Enabled);
    }

    #[test]
    fn a_supported_type_no_installed_engine_plays_has_its_own_reason() {
        let idle = runtime(None, false, RuleState::None);
        let web = theme(WallpaperType::Web, Compatibility::Compatible);
        let no_cef = EngineReportView {
            playable: vec![WallpaperType::Video],
            ..ready()
        };
        assert_eq!(
            use_state(&web, Some(&idle), Some(&no_cef), true),
            CardUse::Disabled(DisabledReason::NoEngine)
        );
        assert_eq!(
            use_state(&video(), Some(&idle), Some(&no_cef), true),
            CardUse::Enabled,
            "the type that plays is not affected"
        );
    }

    #[test]
    fn gif_url_and_youtube_themes_are_usable_once_the_service_reports_them_and_application_themes_are_not() {
        let idle = runtime(None, false, RuleState::None);
        let round_b = EngineReportView {
            supported: vec![
                WallpaperType::Gif,
                WallpaperType::Video,
                WallpaperType::Url,
                WallpaperType::Web,
                WallpaperType::YouTube,
            ],
            playable: vec![
                WallpaperType::Gif,
                WallpaperType::Video,
                WallpaperType::Url,
                WallpaperType::Web,
                WallpaperType::YouTube,
            ],
            ..ready()
        };
        for kind in [WallpaperType::Gif, WallpaperType::Url, WallpaperType::YouTube] {
            assert_eq!(
                use_state(
                    &theme(kind, Compatibility::Compatible),
                    Some(&idle),
                    Some(&round_b),
                    true
                ),
                CardUse::Enabled,
                "{kind:?}"
            );
        }
        assert_eq!(
            use_state(
                &theme(WallpaperType::Application, Compatibility::Compatible),
                Some(&idle),
                Some(&round_b),
                true
            ),
            CardUse::Disabled(DisabledReason::UnsupportedType),
            "Application waits for Round E"
        );
    }

    #[test]
    fn the_card_waits_for_the_engine_report_before_it_allows_use() {
        let idle = runtime(None, false, RuleState::None);
        assert_eq!(
            use_state(&video(), Some(&idle), None, true),
            CardUse::Disabled(DisabledReason::Unavailable)
        );
    }

    #[test]
    fn the_engine_reasons_come_after_the_newer_theme_and_before_the_selection_and_the_rule() {
        let paused = runtime(Some("Rain"), true, RuleState::Paused);
        let none = EngineReportView {
            playable: Vec::new(),
            ..ready()
        };
        let web = theme(WallpaperType::Web, Compatibility::Compatible);
        assert_eq!(
            use_state(&newer(WallpaperType::Web), Some(&paused), Some(&none), true),
            CardUse::Disabled(DisabledReason::NewerVersion)
        );
        assert_eq!(
            use_state(&web, Some(&paused), Some(&none), true),
            CardUse::Disabled(DisabledReason::NoEngine)
        );
    }

    #[test]
    fn no_connection_wins_over_every_other_reason() {
        let paused = runtime(Some("Rain"), true, RuleState::Paused);
        let reason = CardUse::Disabled(DisabledReason::Unavailable);
        assert_eq!(state(&newer(WallpaperType::Web), &paused, false), reason);
        assert_eq!(state(&video(), &paused, false), reason);
    }

    #[test]
    fn a_newer_theme_is_refused_before_its_type_is_looked_at() {
        let idle = runtime(None, false, RuleState::None);
        assert_eq!(
            state(&newer(WallpaperType::Web), &idle, true),
            CardUse::Disabled(DisabledReason::NewerVersion)
        );
    }

    #[test]
    fn a_type_the_service_does_not_report_as_supported_is_not_supported_yet() {
        let idle = runtime(None, false, RuleState::None);
        for kind in [
            WallpaperType::Gif,
            WallpaperType::Url,
            WallpaperType::YouTube,
            WallpaperType::Application,
        ] {
            assert_eq!(
                state(&theme(kind, Compatibility::Compatible), &idle, true),
                CardUse::Disabled(DisabledReason::UnsupportedType),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn a_type_that_is_not_supported_beats_the_rule_and_the_selection() {
        let paused = runtime(Some("Rain"), true, RuleState::Paused);
        assert_eq!(
            state(&theme(WallpaperType::Gif, Compatibility::Compatible), &paused, true),
            CardUse::Disabled(DisabledReason::UnsupportedType)
        );
    }

    #[test]
    fn the_theme_that_plays_is_selected_and_one_that_does_not_run_can_be_used_again() {
        let playing = runtime(Some("Rain"), true, RuleState::None);
        assert_eq!(
            state(&video(), &playing, true),
            CardUse::Disabled(DisabledReason::Selected)
        );
        let stopped = runtime(Some("Rain"), false, RuleState::None);
        assert_eq!(state(&video(), &stopped, true), CardUse::Enabled);
        let other = runtime(Some("Snow"), true, RuleState::None);
        assert_eq!(state(&video(), &other, true), CardUse::Enabled);
    }

    #[test]
    fn a_fullscreen_rule_disables_use_until_it_lifts() {
        let paused = runtime(Some("Snow"), true, RuleState::Paused);
        assert_eq!(
            state(&video(), &paused, true),
            CardUse::Disabled(DisabledReason::Paused)
        );
        let closed = runtime(None, false, RuleState::Closed);
        assert_eq!(
            state(&video(), &closed, true),
            CardUse::Disabled(DisabledReason::Closed)
        );
        let selected = runtime(Some("Rain"), true, RuleState::Paused);
        assert_eq!(
            state(&video(), &selected, true),
            CardUse::Disabled(DisabledReason::Selected),
            "selected and running comes before the rule"
        );
    }

    #[test]
    fn card_preview_hides_the_strip_only_while_it_plays_and_hide_is_on() {
        assert!(strip_visible(false, false));
        assert!(strip_visible(false, true), "no preview, the strip stays");
        assert!(strip_visible(true, false), "hide is off, the strip stays");
        assert!(!strip_visible(true, true));
    }

    #[test]
    fn the_tooltips_are_the_english_texts_of_the_ui_spec() {
        assert_eq!(&*DisabledReason::Unavailable.tooltip(), "Use (Unavailable)");
        assert_eq!(
            &*DisabledReason::NewerVersion.tooltip(),
            "Update Sucrose to use this theme"
        );
        assert_eq!(
            &*DisabledReason::UnsupportedType.tooltip(),
            "This wallpaper type is not supported yet."
        );
        assert_eq!(
            &*DisabledReason::NoEngine.tooltip(),
            "No installed engine can play this wallpaper. Check Wallpaper settings."
        );
        assert_eq!(&*DisabledReason::Selected.tooltip(), "Use (Selected)");
        assert_eq!(&*DisabledReason::Paused.tooltip(), "Use (Paused)");
        assert_eq!(&*DisabledReason::Closed.tooltip(), "Use (Closed)");
        assert_eq!(
            &*DisabledReason::Moving.tooltip(),
            "Please wait while the location is being changed"
        );
    }

    #[test]
    fn the_selected_pill_follows_the_theme_on_the_target_monitor() {
        // The service reports the theme of the target monitor as selected (RuntimeState.selected): the card of that
        // theme says Selected, the card of a theme that plays only on another monitor can be used there.
        let on_target = runtime(Some("Rain"), true, RuleState::None);
        assert_eq!(
            state(&video(), &on_target, true),
            CardUse::Disabled(DisabledReason::Selected)
        );
        let mut elsewhere = video();
        "Snow".clone_into(&mut elsewhere.folder);
        assert_eq!(
            state(&elsewhere, &on_target, true),
            CardUse::Enabled,
            "Use goes to the target monitor, where this theme does not play"
        );
        let nothing_on_target = runtime(None, true, RuleState::None);
        assert_eq!(state(&video(), &nothing_on_target, true), CardUse::Enabled);
    }

    #[test]
    fn a_library_move_disables_use_before_any_other_reason_but_the_connection() {
        let moving = RuntimeState {
            library_moving: true,
            ..runtime(Some("Rain"), true, RuleState::Paused)
        };
        let reason = CardUse::Disabled(DisabledReason::Moving);
        assert_eq!(state(&video(), &moving, true), reason);
        assert_eq!(state(&newer(WallpaperType::Web), &moving, true), reason);
        assert_eq!(
            state(&video(), &moving, false),
            CardUse::Disabled(DisabledReason::Unavailable)
        );
    }

    fn display(number: u8, theme: Option<&str>, running: bool, connected: bool) -> DisplayView {
        DisplayView {
            monitor_id: format!("monitor-{number}"),
            device_name: format!("\\\\.\\DISPLAY{number}"),
            number,
            rect: [0, 0, 1920, 1080],
            primary: number == 1,
            connected,
            theme: theme.map(str::to_owned),
            running,
            engine: None,
        }
    }

    fn badges(displays: &[DisplayView], mode: DisplayScreenType) -> Vec<Badge> {
        monitor_badges("Rain", displays, mode)
    }

    #[test]
    fn badges_a_theme_on_two_monitors_has_one_badge_per_monitor() {
        let displays = [
            display(1, Some("Rain"), true, true),
            display(9, Some("Rain"), true, true),
        ];
        assert_eq!(
            badges(&displays, DisplayScreenType::PerDisplay),
            [Badge::Monitor(1), Badge::Monitor(9)]
        );
    }

    #[test]
    fn badges_a_theme_on_one_monitor_has_one_badge_and_the_other_monitor_none() {
        let displays = [
            display(1, Some("Rain"), true, true),
            display(9, Some("Snow"), true, true),
        ];
        assert_eq!(badges(&displays, DisplayScreenType::PerDisplay), [Badge::Monitor(1)]);
        assert_eq!(
            monitor_badges("Snow", &displays, DisplayScreenType::PerDisplay),
            [Badge::Monitor(9)]
        );
    }

    #[test]
    fn badges_the_third_and_later_monitors_collapse_into_more() {
        let three = [
            display(1, Some("Rain"), true, true),
            display(2, Some("Rain"), true, true),
            display(9, Some("Rain"), true, true),
        ];
        assert_eq!(
            badges(&three, DisplayScreenType::PerDisplay),
            [Badge::Monitor(1), Badge::Monitor(2), Badge::More(vec![9])]
        );
        let five: Vec<DisplayView> = [1, 2, 3, 4, 9]
            .into_iter()
            .map(|number| display(number, Some("Rain"), true, true))
            .collect();
        assert_eq!(
            badges(&five, DisplayScreenType::PerDisplay),
            [Badge::Monitor(1), Badge::Monitor(2), Badge::More(vec![3, 4, 9])]
        );
    }

    #[test]
    fn badges_span_and_duplicate_show_one_all_badge_however_many_monitors_play() {
        let displays = [
            display(1, Some("Rain"), true, true),
            display(9, Some("Rain"), true, true),
        ];
        for mode in [DisplayScreenType::SpanAcross, DisplayScreenType::SameDuplicate] {
            assert_eq!(badges(&displays, mode), [Badge::All], "{mode:?}");
        }
        let one = [display(1, Some("Rain"), true, true)];
        assert_eq!(badges(&one, DisplayScreenType::SameDuplicate), [Badge::All]);
    }

    #[test]
    fn badges_a_theme_that_does_not_run_has_none() {
        let assigned_only = [display(1, Some("Rain"), false, true)];
        assert!(badges(&assigned_only, DisplayScreenType::PerDisplay).is_empty());
        assert!(badges(&assigned_only, DisplayScreenType::SameDuplicate).is_empty());
        let other = [display(1, Some("Snow"), true, true)];
        assert!(badges(&other, DisplayScreenType::PerDisplay).is_empty());
        assert!(badges(&[], DisplayScreenType::SpanAcross).is_empty());
        let empty = [display(1, None, false, true)];
        assert!(badges(&empty, DisplayScreenType::PerDisplay).is_empty());
    }

    #[test]
    fn badges_a_monitor_that_is_not_connected_never_gives_one() {
        let displays = [
            display(1, Some("Rain"), true, true),
            display(9, Some("Rain"), true, false),
        ];
        assert_eq!(badges(&displays, DisplayScreenType::PerDisplay), [Badge::Monitor(1)]);
        let gone = [display(9, Some("Rain"), true, false)];
        assert!(badges(&gone, DisplayScreenType::PerDisplay).is_empty());
        assert!(badges(&gone, DisplayScreenType::SameDuplicate).is_empty());
    }

    #[test]
    fn badges_say_where_in_the_words_of_the_ui_spec() {
        assert_eq!(&*Badge::Monitor(1).text(), "1");
        assert_eq!(&*Badge::Monitor(1).description(), "Playing on monitor 1");
        assert_eq!(&*Badge::All.text(), "All");
        assert_eq!(&*Badge::All.description(), "Playing on all monitors");
        assert_eq!(&*Badge::More(vec![9]).text(), "+1");
        assert_eq!(&*Badge::More(vec![9]).description(), "Also playing on monitor 9");
        assert_eq!(&*Badge::More(vec![3, 4, 9]).text(), "+3");
        assert_eq!(
            &*Badge::More(vec![3, 4, 9]).description(),
            "Also playing on monitors 3, 4, 9"
        );
    }

    /// A window root that shows one card with `badges` (and the Selected pill when `selected`); a click counts as Use.
    struct BadgeHost {
        badges: Vec<Badge>,
        selected: bool,
        used: Rc<Cell<usize>>,
        focus: FocusHandle,
    }

    impl Render for BadgeHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let used = self.used.clone();
            let data = CardData {
                index: 0,
                title: "Rain".into(),
                description: None,
                thumb: Thumb::Failed,
                needs_newer_app: false,
                selected: self.selected,
                badges: self.badges.clone(),
                pending: false,
                state: CardUse::Enabled,
                focus: self.focus.clone(),
                on_use: Rc::new(move |_, _| used.set(used.get() + 1)),
                on_arrow: Rc::new(|_, _, _| {}),
                preview: None,
                preview_hide: false,
                on_hover: Rc::new(|_, _, _| {}),
            };
            render_card(data, px(240.), cx)
        }
    }

    #[gpui_kit::test]
    fn a_click_on_a_badge_is_a_click_on_the_card_and_nothing_else(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let used = Rc::new(Cell::new(0));
        let shared = used.clone();
        let (_, window) = cx.add_window_view(|_, cx| BadgeHost {
            badges: vec![Badge::Monitor(1), Badge::Monitor(9), Badge::More(vec![3, 4])],
            selected: true,
            used: shared,
            focus: cx.focus_handle(),
        });
        window.run_until_parked();
        // The Selected pill is about 90 px wide at the card's top-left corner: 8 px inset, 8 px gap, then the badges.
        window.simulate_click(point(px(130.), px(18.)), Modifiers::default());
        window.run_until_parked();
        assert_eq!(used.get(), 1, "the card got the click");
    }

    #[gpui_kit::test]
    fn badges_render_without_the_selected_pill_and_with_it(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        for selected in [false, true] {
            for badges in [vec![], vec![Badge::All], vec![Badge::Monitor(1), Badge::Monitor(9)]] {
                let (_, window) = cx.add_window_view(|_, cx| BadgeHost {
                    badges,
                    selected,
                    used: Rc::new(Cell::new(0)),
                    focus: cx.focus_handle(),
                });
                window.run_until_parked();
            }
        }
    }

    /// A two-frame image whose frames last 20 ms each.
    fn two_frames() -> Arc<RenderImage> {
        let frame = || {
            Frame::from_parts(
                RgbaImage::from_pixel(4, 4, Rgba([1, 2, 3, 255])),
                0,
                0,
                Delay::from_numer_denom_ms(20, 1),
            )
        };
        Arc::new(RenderImage::new(vec![frame(), frame()]))
    }

    /// A window root that shows one card whose preview is `preview`, with handlers that do nothing.
    struct PreviewHost {
        preview: Arc<RenderImage>,
        focus: FocusHandle,
    }

    impl Render for PreviewHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let data = CardData {
                index: 0,
                title: "Rain".into(),
                description: None,
                thumb: Thumb::Failed,
                needs_newer_app: false,
                selected: false,
                badges: Vec::new(),
                pending: false,
                state: CardUse::Enabled,
                focus: self.focus.clone(),
                on_use: Rc::new(|_, _| {}),
                on_arrow: Rc::new(|_, _, _| {}),
                preview: Some(self.preview.clone()),
                preview_hide: true,
                on_hover: Rc::new(|_, _, _| {}),
            };
            render_card(data, px(240.), cx)
        }
    }

    /// A window root that shows `preview` as a bare `img()` without an element id.
    struct BareHost {
        preview: Arc<RenderImage>,
    }

    impl Render for BareHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            img(ImageSource::Render(self.preview.clone())).size_full()
        }
    }

    /// Activates `window`, redraws it twice with a real pause longer than a frame between the draws, and says whether
    /// every frame of `image` was painted (GPUI times the frames with the real clock).
    fn paints_every_frame(window: &mut VisualTestContext, image: &Arc<RenderImage>) -> bool {
        window.update(|window, _| window.activate_window());
        window.run_until_parked();
        window.update(|window, _| assert!(window.is_window_active(), "the test window is active"));
        for round in 0..2 {
            window.update(|window, _| window.refresh());
            window.run_until_parked();
            if round == 0 {
                std::thread::sleep(Duration::from_millis(60));
            }
        }
        window.update(|window, _| window.has_image_atlas_entry(image))
    }

    #[gpui_kit::test]
    fn card_preview_paints_its_second_frame(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let image = two_frames();
        let shown = image.clone();
        let (_, window) = cx.add_window_view(|_, cx| PreviewHost {
            preview: shown,
            focus: cx.focus_handle(),
        });
        assert!(
            paints_every_frame(window, &image),
            "the card's preview advanced to frame 1"
        );
    }

    #[gpui_kit::test]
    fn card_preview_without_an_id_stays_on_frame_zero(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let image = two_frames();
        let shown = image.clone();
        let (_, window) = cx.add_window_view(|_, _| BareHost { preview: shown });
        assert!(
            !paints_every_frame(window, &image),
            "an img() without an element id keeps no frame state, so it never leaves frame 0"
        );
    }
}
