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
use sucrose_ipc::portal::{PortalRequest, RuntimeState};

use super::layout::CARD_HEIGHT;
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
    /// Whether a click can apply the theme: it sets the hand cursor and the hover ring.
    pub enabled: bool,
    /// What a click does; it checks again whether Use is allowed.
    pub on_use: Rc<dyn Fn(&mut Window, &mut App)>,
}

/// Whether a click on `theme` can apply it now (a first version: connected, compatible, a Video theme, and not the one
/// that plays already).
pub fn can_use(theme: &LibraryTheme, runtime: Option<&RuntimeState>, connected: bool) -> bool {
    let playing = runtime.is_some_and(|runtime| runtime.running && runtime.selected.as_deref() == Some(&theme.folder));
    connected
        && matches!(theme.compatibility, Compatibility::Compatible)
        && theme.info.kind == WallpaperType::Video
        && !playing
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
    let ring = theme.primary.opacity(0.55);
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
        .when(data.enabled, |card| {
            // The 2 px accent ring at 55 % and the hand cursor only where Use is possible (UI-SPEC E2).
            card.cursor_pointer()
                .hover(move |style| style.border_2().border_color(ring))
        })
        .on_click(move |_, window, cx| on_use(window, cx))
        .child(layer)
        .child(strip)
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
