//! Sucrose Portal (D-004). For now an empty window that proves the UI stack builds and runs on every OS.
//!
//! `--no-focus` opens the window without taking focus (for automated checks).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui_kit::component::{ActiveTheme, TitleBar, v_flex};
use gpui_kit::*;

struct Portal;

impl Render for Portal {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(TitleBar::new().child(div().text_xs().child(sucrose_core::APP_LONG_NAME)))
            .child(
                v_flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .child(div().text_2xl().child(sucrose_core::APP_NAME))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("v{}", sucrose_core::VERSION)),
                    ),
            )
    }
}

fn main() -> anyhow::Result<()> {
    let _log = sucrose_log::init("Portal")?;
    let no_focus = std::env::args().any(|arg| arg == "--no-focus");

    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();

            let bounds = Bounds::centered(None, size(px(900.), px(600.)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(900.), px(600.))),
                focus: !no_focus,
                titlebar: Some(TitlebarOptions {
                    title: Some(sucrose_core::APP_LONG_NAME.into()),
                    ..TitleBar::title_bar_options()
                }),
                ..TitleBar::window_options()
            };

            if let Err(error) = gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| Portal)) {
                tracing::error!(%error, "cannot open the Portal window");
                cx.quit();
                return;
            }
            if !no_focus {
                cx.activate(true);
            }
        });

    Ok(())
}
