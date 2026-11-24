//! Modal dialogs of the Portal (UI-SPEC "Dialogs", E11): Help, About, the tray hint and the Move Library confirmation. They are 480 px wide (About 560),
//! 12 px radius, with a title and actions that stay fixed while the body scrolls. Links open in the OS browser; every
//! target is an https address that is part of the build, never read from settings or themes (T-03-42).
//!
//! Rows put their parts in flex order (icon, text, trailing mark), never by left and right geometry, so a dialog mirrors
//! for Arabic and Hebrew (UI-SPEC E4 RTL row; the mirrored screenshot is due with UI-06).

pub mod about;
pub mod help;
pub mod move_library;
pub mod tray_hint;

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable as _, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    dialog::DialogFooter,
    h_flex,
    scroll::ScrollableElement as _,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

/// Width of Help and the tray hint (UI-SPEC).
pub const DIALOG_WIDTH: f32 = 480.0;
/// Width of About (UI-SPEC).
pub const ABOUT_WIDTH: f32 = 560.0;
/// The clearance a dialog keeps from the window height (UI-SPEC E11 overflow row).
const WINDOW_CLEARANCE: f32 = 64.0;
/// What the fixed parts of a dialog take: title, actions, padding and gaps, with some air.
const FIXED_CHROME: f32 = 160.0;
/// The body never shrinks below this, however small the window.
const MIN_BODY_HEIGHT: f32 = 96.0;

/// The links the Portal opens. All are https and fixed at build time (the legacy values of `Url.cs`).
pub mod urls {
    /// The official web page.
    pub const OFFICIAL: &str = "https://sucrose.dev";
    /// The source code.
    pub const REPOSITORY: &str = "https://github.com/Taiizor/Sucrose";
    /// The documentation (the repository wiki).
    pub const DOCUMENTATION: &str = "https://github.com/Taiizor/Sucrose/wiki";
    /// The discussions.
    pub const DISCUSSIONS: &str = "https://github.com/Taiizor/Sucrose/discussions";
    /// The issue form.
    pub const REPORT: &str = "https://github.com/Taiizor/Sucrose/issues/new/choose";
    /// The release notes of one version; the tag is `v{version}`.
    pub const RELEASE_TAG: &str = "https://github.com/Taiizor/Sucrose/releases/tag";
    /// The contributors.
    pub const CONTRIBUTORS: &str = "https://github.com/Taiizor";
    /// The terms of service.
    pub const TERMS_OF_SERVICE: &str = "https://github.com/Taiizor/Sucrose/blob/develop/.github/TERMS_OF_SERVICE.md";
    /// The privacy policy.
    pub const PRIVACY_POLICY: &str = "https://github.com/Taiizor/Sucrose/blob/develop/.github/PRIVACY_POLICY.md";
    /// The code of conduct.
    pub const CODE_OF_CONDUCT: &str = "https://github.com/Taiizor/Sucrose/blob/develop/.github/CODE_OF_CONDUCT.md";
}

/// The most the body of a dialog may take in a window of this height: the dialog as a whole stays 64 px short of the
/// window, and the fixed parts keep their space (UI-SPEC E11 overflow row).
pub fn body_max_height(window_height: f32) -> f32 {
    (window_height - WINDOW_CLEARANCE - FIXED_CHROME).max(MIN_BODY_HEIGHT)
}

/// Opens a link in the OS browser. GPUI reports no result for it, so when the OS cannot open the link nothing on screen
/// changes; the attempt is logged so a missing browser can be traced (UI-SPEC E4/E11 error row).
pub fn open_link(url: &str, cx: &mut App) {
    tracing::info!(url, "opening a link in the browser");
    cx.open_url(url);
}

/// The footer with the one primary Close action.
pub fn close_footer(label: SharedString) -> DialogFooter {
    DialogFooter::new().child(
        Button::new("dialog-close")
            .primary()
            .label(label)
            .on_click(|_, window, cx| window.close_dialog(cx)),
    )
}

/// The scrolling body of a dialog: its height is capped by [`body_max_height`], the title and the footer stay put.
pub fn body(window: &Window, rows: impl IntoIterator<Item = AnyElement>) -> impl IntoElement {
    let cap = body_max_height(f32::from(window.viewport_size().height));
    v_flex().gap_2().max_h(px(cap)).children(rows).overflow_y_scrollbar()
}

/// One link row: icon, title, optional description and the open-in-browser mark. It is a button, so the keyboard reaches
/// it. The title is underlined when the row is a plain link without a description (About).
pub fn link_row(
    id: &'static str,
    icon: IconName,
    title: SharedString,
    description: Option<SharedString>,
    url: String,
    underline: bool,
    cx: &App,
) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    Button::new(id)
        .ghost()
        .w_full()
        .h_auto()
        .py_2()
        .on_click(move |_, _, cx| open_link(&url, cx))
        .child(
            h_flex()
                .w_full()
                .gap_3()
                .items_center()
                .child(div().flex_none().child(Icon::new(icon)))
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .whitespace_normal()
                        .child(
                            div()
                                .text_sm()
                                .font_semibold()
                                .when(underline, Styled::underline)
                                .child(title),
                        )
                        .when_some(description, |column, description| {
                            column.child(div().text_xs().text_color(muted).child(description))
                        }),
                )
                .child(
                    div()
                        .flex_none()
                        .text_color(muted)
                        .child(Icon::new(IconName::ExternalLink).small()),
                ),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{MIN_BODY_HEIGHT, body_max_height, urls};

    const ALL: &[&str] = &[
        urls::OFFICIAL,
        urls::REPOSITORY,
        urls::DOCUMENTATION,
        urls::DISCUSSIONS,
        urls::REPORT,
        urls::RELEASE_TAG,
        urls::CONTRIBUTORS,
        urls::TERMS_OF_SERVICE,
        urls::PRIVACY_POLICY,
        urls::CODE_OF_CONDUCT,
    ];

    #[test]
    fn every_link_is_a_plain_https_address() {
        for url in ALL {
            assert!(url.starts_with("https://"), "{url}");
            assert!(!url.contains(char::is_whitespace), "{url}");
            assert!(!url.contains('@'), "{url} carries credentials");
            assert!(
                !url.contains('?') && !url.contains('#'),
                "{url} has a query or fragment"
            );
        }
    }

    #[test]
    fn the_body_leaves_the_dialog_sixty_four_pixels_short_of_the_window() {
        assert_eq!(body_max_height(600.0), 600.0 - 64.0 - 160.0);
        assert!(body_max_height(600.0) + 160.0 <= 600.0 - 64.0);
    }

    #[test]
    fn a_tiny_window_still_leaves_a_scrollable_body() {
        assert_eq!(body_max_height(100.0), MIN_BODY_HEIGHT);
        assert_eq!(body_max_height(0.0), MIN_BODY_HEIGHT);
    }
}
