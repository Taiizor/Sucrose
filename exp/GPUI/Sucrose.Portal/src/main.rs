#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod data;
mod i18n;
mod thumbs;

use gpui_kit::component::TitleBar;
use gpui_kit::*;

fn main() {
    let mut settings = data::Settings::load();
    let args: Vec<String> = std::env::args().collect();
    // --no-focus: open without stealing focus (used for automated screenshots).
    let no_focus = args.iter().any(|arg| arg == "--no-focus");
    if args.iter().any(|arg| arg == "--light") {
        settings.dark = false;
    }
    i18n::set_lang(if settings.culture.eq_ignore_ascii_case("TR") {
        i18n::Lang::Tr
    } else {
        i18n::Lang::En
    });

    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(move |cx| {
            gpui_kit::init(cx);
            app::apply_theme(settings.dark, app::DEFAULT_TINT, cx);

            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();

            let bounds = Bounds::centered(None, size(px(1180.), px(760.)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(860.), px(560.))),
                window_background: WindowBackgroundAppearance::Blurred,
                titlebar: Some(TitlebarOptions {
                    title: Some("Sucrose Wallpaper Engine".into()),
                    ..TitleBar::title_bar_options()
                }),
                focus: !no_focus,
                ..TitleBar::window_options()
            };

            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| app::Portal::new(settings, window, cx))
            })
            .expect("failed to open the Portal window");
            if !no_focus {
                cx.activate(true);
            }
        });
}
