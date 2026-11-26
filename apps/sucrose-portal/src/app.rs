//! The Portal's root entity: the connection to the service, what the service reported, and the window layout.

use std::time::Duration;

use gpui_kit::component::{ActiveTheme, TitleBar, WindowExt as _, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use sucrose_core::settings::{Backdrop, ThemeMode};
use sucrose_ipc::portal::{EngineReportView, PortalEvent, PortalSettings, RuntimeState, SettingChange};

use sucrose_ipc::portal::UseOutcome;

use crate::{
    background::BackgroundImage,
    banner,
    dialogs::tray_hint::{self, TrayHint, should_hint},
    header::{Header, HeaderEvent},
    i18n::{tr, tr_count_with},
    ipc::{ConnectionState, LinkEvent, ServiceLink, next_state},
    library::{ChangeLocation, LibraryView, bin_name},
    settings::{Mode, SettingsTab, general::GeneralPage, personal::PersonalPage},
    status_bar::StatusBar,
    theme, toast,
};

/// The Portal window's content. Later plans add the pages as children.
#[allow(
    dead_code,
    reason = "the library and settings pages of the following plans read these fields"
)]
pub struct PortalApp {
    /// Where the connection to the service stands.
    pub state: ConnectionState,
    /// The settings the service reported (Welcome and Changed); never read from a file (D-11).
    pub settings: Option<PortalSettings>,
    /// What the service is doing.
    pub runtime: Option<RuntimeState>,
    /// What the service detected about the engines; the Wallpaper settings page and the cards read it.
    pub engines: Option<EngineReportView>,
    /// What the window shows: the library or a settings page.
    pub mode: Mode,
    /// Sends requests to the service.
    pub link: ServiceLink,
    /// The library page.
    pub library: Entity<LibraryView>,
    /// The header row: the Library tab and the search box, or the settings tabs.
    pub header: Entity<Header>,
    /// The General settings page.
    pub general: Entity<GeneralPage>,
    /// The Personal settings page.
    pub personal: Entity<PersonalPage>,
    /// The status bar at the bottom: build, memory of the Sucrose processes, quote and copyright.
    pub status_bar: Entity<StatusBar>,
    /// The task that awaits the link's events; dropped with the app.
    _events: Task<()>,
    /// Watches the keyboard for Ctrl+F and for Escape in the settings, which work wherever the focus is.
    _keys: Subscription,
    /// Follows the gear, Back and the settings tabs of the header.
    _header_events: Subscription,
    /// Follows the library's Change Library Location buttons.
    _library_events: Subscription,
    /// Follows the OS light/dark setting while the theme mode is System.
    _appearance: Subscription,
    /// Whether this OS can show a window backdrop (read once at start; D-22).
    window_backdrop: bool,
    /// The appearance this view last set on the window; None until the first apply. GPUI's Window has no public getter
    /// for it.
    applied_appearance: Option<WindowBackgroundAppearance>,
    /// The background image layer, below the page tint.
    pub background: Entity<BackgroundImage>,
    /// The tray hint was shown during this window's life; closing again closes at once.
    tray_hint_shown: bool,
    /// Close of the tray hint was pressed: the window closes when the service answers, or after [`HINT_WRITE_WAIT`].
    closing_after_hint: bool,
    /// Closes the window when the service does not answer; dropped with the app.
    _hint_timer: Option<Task<()>>,
}

/// How long the window waits for the service to confirm that the hint is off before it closes anyway.
const HINT_WRITE_WAIT: Duration = Duration::from_secs(1);

impl PortalApp {
    /// Builds the app, applies `early` (the events that were ready before the window opened) and starts awaiting the
    /// link's events on the GPUI thread.
    pub fn new(
        page: sucrose_ipc::portal::PortalPage,
        link: ServiceLink,
        window_backdrop: bool,
        early: Vec<LinkEvent>,
        events: async_channel::Receiver<LinkEvent>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let task = cx.spawn_in(window, async move |this, cx| {
            while let Ok(event) = events.recv().await {
                if this
                    .update_in(cx, |this, window, cx| this.on_link_event(event, window, cx))
                    .is_err()
                {
                    break;
                }
            }
        });
        let library = cx.new({
            let link = link.clone();
            |cx| LibraryView::new(link, window, cx)
        });
        let mode = Mode::from_page(page);
        let header = cx.new({
            let library = library.clone();
            let link = link.clone();
            |cx| Header::new(library, link, mode, window, cx)
        });
        let personal = cx.new({
            let link = link.clone();
            |cx| PersonalPage::new(link, window, cx)
        });
        let library_events = cx.subscribe_in(&library, window, |this, _, _: &ChangeLocation, window, cx| {
            this.set_mode(Mode::Settings(SettingsTab::General), window, cx);
        });
        let header_events = cx.subscribe_in(&header, window, |this, _, event: &HeaderEvent, window, cx| {
            this.set_mode(event.0, window, cx);
        });
        // Ctrl+F (Cmd+F on macOS) puts the cursor in the search box from anywhere in the window.
        let keys = cx.observe_keystrokes(|this, event, window, cx| {
            if event.keystroke.modifiers.secondary(cx) && event.keystroke.key == "f" {
                this.header.update(cx, |header, cx| header.focus_search(window, cx));
            }
            // Escape leaves the settings unless a field or a dialog has the keyboard (UI-SPEC Interaction: Settings navigation).
            if event.keystroke.key == "escape"
                && this.mode.is_settings()
                && !window.has_active_dialog(cx)
                && !this.field_focused(window, cx)
            {
                this.set_mode(this.mode.back(), window, cx);
            }
        });
        // The OS theme changed: only the System mode follows it; no polling.
        let appearance = theme::follow_os(
            window,
            cx,
            |this: &Self| this.theme_mode() == ThemeMode::System,
            |this, window, cx| this.apply_theme(window, cx),
        );
        let background = cx.new(|cx| BackgroundImage::new(window, cx));
        let general = cx.new({
            let link = link.clone();
            let background = background.clone();
            |cx| GeneralPage::new(link, window_backdrop, background, window, cx)
        });
        let status_bar = cx.new(|_| StatusBar::new());
        let mut app = Self {
            state: ConnectionState::Connecting,
            settings: None,
            runtime: None,
            engines: None,
            mode,
            link,
            library,
            header,
            general,
            personal,
            status_bar,
            _events: task,
            _keys: keys,
            _header_events: header_events,
            _library_events: library_events,
            _appearance: appearance,
            window_backdrop,
            applied_appearance: None,
            background,
            tray_hint_shown: false,
            closing_after_hint: false,
            _hint_timer: None,
        };
        // The first close with the hint on explains the tray (UI-05); every other close goes through.
        let this = cx.weak_entity();
        window.on_window_should_close(cx, move |window, cx| {
            this.update(cx, |app, cx| app.on_close_requested(window, cx))
                .unwrap_or(true)
        });
        for event in early {
            app.on_link_event(event, window, cx);
        }
        app
    }

    /// Shows `mode`. Leaving the library stops its hover preview; the library itself (page, scroll position, search text)
    /// is not rebuilt, so Back finds it as it was.
    pub fn set_mode(&mut self, mode: Mode, _window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == mode {
            return;
        }
        if mode.is_settings() && !self.mode.is_settings() {
            self.library.update(cx, |library, cx| library.suspend(cx));
        }
        self.mode = mode;
        self.header.update(cx, |header, cx| header.set_mode(mode, cx));
        // The General page reads the start entries each time it opens (D-08).
        if mode == Mode::Settings(SettingsTab::General) {
            self.general.update(cx, |page, cx| page.on_opened(cx));
        }
        cx.notify();
    }

    /// Whether the cursor is in a field of the settings page that is shown.
    fn field_focused(&self, window: &Window, cx: &App) -> bool {
        match self.mode {
            Mode::Settings(SettingsTab::Personal) => self.personal.read(cx).field_focused(window, cx),
            Mode::Settings(SettingsTab::General) => self.general.read(cx).field_focused(window, cx),
            Mode::Library => false,
        }
    }

    /// The window is about to close: false while the tray hint takes the close (it closes the window itself).
    fn on_close_requested(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let hint_on = self.settings.as_ref().is_some_and(|settings| settings.hint_tray_icon);
        let tray_visible = self.runtime.as_ref().is_some_and(|runtime| runtime.tray_visible);
        if !should_hint(hint_on, tray_visible, self.tray_hint_shown) {
            return true;
        }
        self.tray_hint_shown = true;
        let app = cx.weak_entity();
        let link = self.link.clone();
        let hint = cx.new(|cx| {
            TrayHint::new(
                link,
                move |window, cx| {
                    app.update(cx, |app, cx| app.finish_tray_hint(window, cx)).ok();
                },
                cx,
            )
        });
        tray_hint::open(hint, window, cx);
        cx.notify();
        false
    }

    /// Close of the tray hint was pressed (it has sent the change to the service): the window closes once the service has
    /// answered, or after a second whatever happens (a refused write shows the hint again next time).
    fn finish_tray_hint(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.closing_after_hint = true;
        self._hint_timer = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(HINT_WRITE_WAIT).await;
            this.update_in(cx, |_, window, _| window.remove_window()).ok();
        }));
    }

    /// The stored theme mode; System until the service has reported the settings.
    fn theme_mode(&self) -> ThemeMode {
        self.settings
            .as_ref()
            .map_or(ThemeMode::System, |settings| settings.theme_mode)
    }

    /// The stored backdrop; None until the service has reported the settings.
    fn backdrop(&self) -> Backdrop {
        self.settings
            .as_ref()
            .map_or(Backdrop::None, |settings| settings.backdrop)
    }

    /// Tells the window which backdrop to draw. A stored value this OS cannot show is applied as opaque; the setting
    /// itself is never rewritten (D-22).
    ///
    /// The opaque fill that gpui-component's Root paints under the view is cleared while a backdrop shows (G-03-5). The
    /// Root is mounted only after the view's constructor returns, so the Welcome that arrived before the window existed
    /// finds no Root: the fill is then applied once more at the end of the current effect cycle.
    fn apply_backdrop(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let appearance = theme::appearance(self.backdrop(), self.window_backdrop);
        // Opaque first when the appearance changes, so the accent policy of an earlier Acrylic is cleared before Mica or
        // Tabbed is applied (WR-14); the target is always the last appearance set.
        let steps = theme::appearance_steps(self.applied_appearance, appearance);
        for step in &steps {
            window.set_background_appearance(*step);
        }
        self.applied_appearance = Some(appearance);
        // Windows 11 draws Mica and Tabbed through the DWM system backdrop type, which GPUI sets without clearing it
        // later (WR-14), so Sucrose sets and resets it itself and logs what the OS answered.
        let dwm_result = self
            .window_backdrop
            .then(|| sucrose_platform::current().set_window_backdrop(&*window, self.backdrop()));
        let dwm_text = dwm_result_text(dwm_result.as_ref());
        if matches!(dwm_result, Some(Err(_))) {
            tracing::warn!(dwm_result = %dwm_text, "the OS refused the window backdrop");
        }
        let root_transparent = theme::backdrop_active(self.backdrop(), self.window_backdrop);
        let mounted = theme::apply_root_fill(root_transparent, window, cx);
        tracing::info!(
            backdrop = ?self.backdrop(),
            supported = self.window_backdrop,
            ?appearance,
            ?steps,
            root_transparent,
            dwm_result = %dwm_text,
            "backdrop applied"
        );
        if !mounted {
            cx.defer_in(window, |this, window, cx| {
                let active = theme::backdrop_active(this.backdrop(), this.window_backdrop);
                if !theme::apply_root_fill(active, window, cx) {
                    tracing::warn!("the window has no base root, the backdrop stays covered");
                }
            });
        }
    }

    /// Gives the background layer the stored image, stretch and opacity.
    fn apply_background(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(settings) = &self.settings else { return };
        let (path, stretch, opacity) = (
            settings.background_image.clone(),
            settings.background_stretch,
            settings.background_opacity,
        );
        self.background
            .update(cx, |background, cx| background.set(&path, stretch, opacity, window, cx));
    }

    /// Applies the stored theme mode and tells the header what the window shows now.
    fn apply_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mode = self.theme_mode();
        theme::apply_mode(mode, window, cx);
        let dark = theme::effective_dark(mode, window);
        self.header.update(cx, |header, cx| header.set_dark(dark, cx));
    }

    /// Applies one event of the link to the app.
    pub fn on_link_event(&mut self, event: LinkEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.state = next_state(self.state, &event);
        let connected = self.state == ConnectionState::Connected;
        self.library
            .update(cx, |library, cx| library.set_connected(connected, cx));
        self.personal.update(cx, |page, cx| page.set_connected(connected, cx));
        self.general.update(cx, |page, cx| page.set_connected(connected, cx));
        if !connected {
            // The memory text shows "—" until the service reports again.
            self.status_bar.update(cx, |bar, cx| bar.set_memory(None, cx));
        }
        if let LinkEvent::Event(event) = event {
            match event {
                PortalEvent::Welcome { settings, runtime } => {
                    self.library.update(cx, |library, cx| {
                        library.load(&settings, cx);
                        library.set_runtime(runtime.clone(), cx);
                    });
                    self.personal.update(cx, |page, cx| page.load(&settings, window, cx));
                    self.general.update(cx, |page, cx| {
                        page.load(&settings, window, cx);
                        page.set_moving(runtime.library_moving, cx);
                    });
                    if self.mode == Mode::Settings(SettingsTab::General) {
                        self.general.update(cx, |page, cx| page.on_opened(cx));
                    }
                    self.settings = Some(settings);
                    self.runtime = Some(runtime);
                    self.apply_theme(window, cx);
                    self.apply_backdrop(window, cx);
                    self.apply_background(window, cx);
                }
                PortalEvent::Changed(change) => {
                    if let Some(settings) = &mut self.settings {
                        settings.apply(&change);
                        self.library
                            .update(cx, |library, cx| library.update_settings(settings, cx));
                        self.personal
                            .update(cx, |page, cx| page.on_setting(settings, &change, window, cx));
                        self.general
                            .update(cx, |page, cx| page.on_setting(settings, &change, window, cx));
                    }
                    if matches!(change, SettingChange::ThemeMode(_)) {
                        self.apply_theme(window, cx);
                    }
                    if matches!(change, SettingChange::Backdrop(_)) {
                        self.apply_backdrop(window, cx);
                    }
                    if self.closing_after_hint && matches!(change, SettingChange::HintTrayIcon(_)) {
                        window.remove_window();
                    }
                    // Stretch and opacity apply live to the image that is shown; a new path decodes again.
                    match change {
                        SettingChange::BackgroundImage(_) => self.apply_background(window, cx),
                        SettingChange::BackgroundStretch(stretch) => {
                            self.background
                                .update(cx, |background, cx| background.set_stretch(stretch, cx));
                        }
                        SettingChange::BackgroundOpacity(opacity) => {
                            self.background
                                .update(cx, |background, cx| background.set_opacity(opacity, cx));
                        }
                        _ => {}
                    }
                }
                PortalEvent::Runtime(runtime) => {
                    self.library
                        .update(cx, |library, cx| library.set_runtime(runtime.clone(), cx));
                    self.general
                        .update(cx, |page, cx| page.set_moving(runtime.library_moving, cx));
                    self.runtime = Some(runtime);
                }
                PortalEvent::Startup(info) => {
                    self.general.update(cx, |page, cx| page.on_startup(&info, window, cx));
                }
                PortalEvent::MoveResult(outcome) => {
                    let applied = self
                        .general
                        .update(cx, |page, cx| page.on_move_result(&outcome, window, cx));
                    // The library loads again from the new folder: a changed location makes it scan.
                    if let (Some(location), Some(settings)) = (applied, &mut self.settings) {
                        settings.library_location = location;
                        self.library
                            .update(cx, |library, cx| library.update_settings(settings, cx));
                    }
                }
                PortalEvent::LibraryMaintained(report) => {
                    self.library
                        .update(cx, |library, cx| library.on_maintained(&report, cx));
                    // The user hears about themes that left the library, and where they can get them back.
                    if report.corrupt_moved > 0 {
                        toast::show_info(
                            tr_count_with(
                                "Portal.LibraryPage.CorruptMoved",
                                report.corrupt_moved as usize,
                                &[("bin", bin_name())],
                            ),
                            window,
                            cx,
                        );
                    }
                }
                PortalEvent::Memory(report) => {
                    self.status_bar.update(cx, |bar, cx| bar.set_memory(Some(report), cx));
                }
                PortalEvent::UseResult { folder, outcome } => {
                    self.library
                        .update(cx, |library, cx| library.on_use_result(&folder, &outcome, cx));
                    // Whatever kept the wallpaper from changing, the user hears it; the old selection stays.
                    if !matches!(outcome, UseOutcome::Applied) {
                        toast::show_error(tr("Portal.LibraryCard.UseFailed"), window, cx);
                    }
                }
                PortalEvent::Rejected { reason, change } => {
                    tracing::warn!(%reason, "the service did not apply a setting");
                    // The control that sent it shows the broadcast value again.
                    if let Some(settings) = &self.settings {
                        self.personal
                            .update(cx, |page, cx| page.on_setting(settings, &change, window, cx));
                        self.general
                            .update(cx, |page, cx| page.on_setting(settings, &change, window, cx));
                    }
                    if self.closing_after_hint {
                        // The hint stays on and shows again next time; the user asked to close.
                        window.remove_window();
                    }
                    toast::show_error(tr("Portal.Setting.SaveFailed"), window, cx);
                }
                PortalEvent::AlreadyOpen { existing_pid } => {
                    tracing::info!(existing_pid, "already open");
                    cx.quit();
                }
                PortalEvent::Refused { reason } => tracing::warn!(%reason, "the service refused this Portal"),
                PortalEvent::Focus { page } => {
                    if let Some(page) = page {
                        self.set_mode(Mode::from_page(page), window, cx);
                    }
                    // Windows restores a minimized window as part of activating it.
                    window.activate_window();
                }
                PortalEvent::Close => {
                    tracing::info!("the service is stopping, closing the Portal");
                    cx.quit();
                }
                PortalEvent::Engines(report) => {
                    tracing::debug!(
                        engines = report.engines.len(),
                        fallbacks = report.fallbacks.len(),
                        "the service sent the engine report"
                    );
                    self.library
                        .update(cx, |library, cx| library.set_engines(report.clone(), cx));
                    self.engines = Some(report);
                }
            }
        }
        cx.notify();
    }
}

/// What the DWM call answered, for the log: "skipped" when it was not made, the HRESULT bits in hex, or the error text.
fn dwm_result_text(result: Option<&Result<(), sucrose_platform::PlatformError>>) -> String {
    match result {
        None => "skipped".to_owned(),
        Some(Ok(())) => "0x00000000".to_owned(),
        #[allow(clippy::cast_sign_loss, reason = "the HRESULT is shown as its bit pattern")]
        Some(Err(sucrose_platform::PlatformError::Os { code, .. })) => format!("{:#010x}", *code as u32),
        Some(Err(error)) => error.to_string(),
    }
}

impl Render for PortalApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The page tint is translucent while a backdrop shows through it (UI-SPEC Color).
        let image_active = self.background.read(cx).is_active();
        let alpha = theme::page_alpha(
            theme::backdrop_active(self.backdrop(), self.window_backdrop),
            image_active,
        );
        // From the bottom: the window backdrop, the background image, the tinted page, the content.
        div().relative().size_full().child(self.background.clone()).child(
            v_flex()
                .size_full()
                .bg(cx.theme().background.opacity(alpha))
                .text_color(cx.theme().foreground)
                .child(TitleBar::new().child(div().text_xs().child(crate::window_title())))
                .child(self.header.clone())
                .when_some(
                    banner::connection_banner(self.state, cx.listener(|this, _: &ClickEvent, _, _| this.link.retry())),
                    |column, banner| column.child(banner),
                )
                .child(v_flex().flex_1().min_h_0().child(match self.mode {
                    Mode::Library => self.library.clone().into_any_element(),
                    Mode::Settings(SettingsTab::Personal) => self.personal.clone().into_any_element(),
                    Mode::Settings(SettingsTab::General) => self.general.clone().into_any_element(),
                }))
                .child(self.status_bar.clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    use sucrose_platform::PlatformError;

    use super::dwm_result_text;

    #[test]
    fn app_dwm_result_text_names_each_outcome() {
        assert_eq!(dwm_result_text(None), "skipped");
        assert_eq!(dwm_result_text(Some(&Ok(()))), "0x00000000");
        let refused = Err(PlatformError::Os {
            context: "x",
            code: -2_147_024_809,
        });
        assert_eq!(dwm_result_text(Some(&refused)), "0x80070057");
        let unsupported = Err(PlatformError::Unsupported("window backdrop"));
        assert_eq!(
            dwm_result_text(Some(&unsupported)),
            PlatformError::Unsupported("window backdrop").to_string()
        );
    }
}
