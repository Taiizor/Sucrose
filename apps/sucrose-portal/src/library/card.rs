//! One theme card (UI-SPEC E2): the cover-fit thumbnail, the title strip and the newer-version pill.

use std::{rc::Rc, sync::Arc};

use gpui_kit::assets::IconName;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, skeleton::Skeleton, spinner::Spinner, tooltip::Tooltip};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use sucrose_core::{
    library::{Compatibility, LibraryTheme},
    theme::WallpaperType,
};
use sucrose_ipc::portal::{PortalRequest, RuleState, RuntimeState};

use super::layout::{Arrow, CARD_HEIGHT};
use crate::{i18n::tr, ipc::ServiceLink};

/// Height of the strip with the title and the description.
const STRIP_HEIGHT: f32 = 56.0;

/// What the card shows where the thumbnail goes.
pub enum Thumb {
    /// The decoded thumbnail.
    Ready(Arc<RenderImage>),
    /// Not decoded yet.
    Loading,
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
    /// This build plays only Video themes (D-17).
    UnsupportedType,
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

/// Whether Use works on `theme` now, in the priority of the UI-SPEC: no connection, a library move, a newer theme, a type this build does
/// not play, the theme that plays already, then a fullscreen rule. A selected theme that does not run can be used again.
pub fn use_state(theme: &LibraryTheme, runtime: Option<&RuntimeState>, connected: bool) -> CardUse {
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
    if theme.info.kind != WallpaperType::Video {
        return CardUse::Disabled(DisabledReason::UnsupportedType);
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
        .when(data.selected, |card| {
            card.child(
                div()
                    .id(("library-card-selected", index))
                    .absolute()
                    .top_2()
                    .left_2()
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

/// A card-sized placeholder for the grid that shows while the library is scanned.
pub fn render_skeleton_card(width: Pixels) -> impl IntoElement {
    Skeleton::new().flex_none().w(width).h(px(CARD_HEIGHT)).rounded(px(8.))
}

#[cfg(test)]
mod tests {
    // Not `use super::*`: the crate root glob-imports GPUI, whose `test` macro would shadow the built-in attribute.
    use super::{CardData, CardUse, DisabledReason, Thumb, render_card, strip_visible, use_state};
    use gpui_kit::{
        Context, FocusHandle, ImageSource, IntoElement, Render, RenderImage, Styled as _, TestAppContext,
        VisualTestContext, Window, img, px,
    };
    use image::{Delay, Frame, Rgba, RgbaImage};
    use std::{path::PathBuf, rc::Rc, sync::Arc, time::Duration};
    use sucrose_core::{
        library::{Compatibility, LibraryTheme},
        theme::{SucroseInfo, WallpaperType},
        version::LegacyVersion,
    };
    use sucrose_ipc::portal::{RuleState, RuntimeState};

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
        }
    }

    fn state(theme: &LibraryTheme, runtime: &RuntimeState, connected: bool) -> CardUse {
        use_state(theme, Some(runtime), connected)
    }

    #[test]
    fn a_compatible_video_theme_can_be_used() {
        let idle = runtime(None, false, RuleState::None);
        assert_eq!(state(&video(), &idle, true), CardUse::Enabled);
        assert_eq!(
            use_state(&video(), None, true),
            CardUse::Enabled,
            "no runtime report yet is not a reason"
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
    fn every_type_but_video_is_not_supported_yet() {
        let idle = runtime(None, false, RuleState::None);
        for kind in [
            WallpaperType::Gif,
            WallpaperType::Url,
            WallpaperType::Web,
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
        assert_eq!(&*DisabledReason::Selected.tooltip(), "Use (Selected)");
        assert_eq!(&*DisabledReason::Paused.tooltip(), "Use (Paused)");
        assert_eq!(&*DisabledReason::Closed.tooltip(), "Use (Closed)");
        assert_eq!(
            &*DisabledReason::Moving.tooltip(),
            "Please wait while the location is being changed"
        );
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
