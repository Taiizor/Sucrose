//! One theme card (UI-SPEC E2): the cover-fit thumbnail, the title strip and the newer-version pill.

use std::sync::Arc;

use gpui_kit::assets::IconName;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, skeleton::Skeleton, tooltip::Tooltip};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::layout::CARD_HEIGHT;
use crate::i18n::tr;

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
}

/// The card for `data`, `width` wide.
pub fn render_card(data: CardData, width: Pixels, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let index = data.index;
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
        .child(layer)
        .child(strip)
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
