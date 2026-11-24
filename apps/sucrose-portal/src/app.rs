//! The Portal's root entity: the connection to the service, what the service reported, and the window layout.

use std::time::Duration;

use gpui_kit::component::{ActiveTheme, TitleBar, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use sucrose_core::settings::{Backdrop, ThemeMode};
use sucrose_ipc::portal::{PortalEvent, PortalPage, PortalSettings, RuntimeState, SettingChange};

use sucrose_ipc::portal::UseOutcome;

use crate::{
    background::BackgroundImage,
    banner,
    dialogs::tray_hint::{self, TrayHint, should_hint},
    header::Header,
    i18n::tr,
    ipc::{ConnectionState, LinkEvent, ServiceLink, next_state},
    library::LibraryView,
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
    /// The page the Portal was opened on.
    pub page: PortalPage,
    /// Sends requests to the service.
    pub link: ServiceLink,
    /// The library page.
    pub library: Entity<LibraryView>,
    /// The header row: the Library tab and the search box.
    pub header: Entity<Header>,
    /// The status bar at the bottom: build, memory of the Sucrose processes, quote and copyright.
    pub status_bar: Entity<StatusBar>,
    /// The task that awaits the link's events; dropped with the app.
    _events: Task<()>,
    /// Watches the keyboard for Ctrl+F, which works wherever the focus is.
    _keys: Subscription,
    /// Follows the OS light/dark setting while the theme mode is System.
    _appearance: Subscription,
    /// Whether this OS can show a window backdrop (read once at start; D-22).
    window_backdrop: bool,
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
        page: PortalPage,
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
        let header = cx.new({
            let library = library.clone();
            let link = link.clone();
            |cx| Header::new(library, link, window, cx)
        });
        // Ctrl+F (Cmd+F on macOS) puts the cursor in the search box from anywhere in the window.
        let keys = cx.observe_keystrokes(|this, event, window, cx| {
            if event.keystroke.modifiers.secondary(cx) && event.keystroke.key == "f" {
                this.header.update(cx, |header, cx| header.focus_search(window, cx));
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
        let status_bar = cx.new(|_| StatusBar::new());
        let mut app = Self {
            state: ConnectionState::Connecting,
            settings: None,
            runtime: None,
            page,
            link,
            library,
            header,
            status_bar,
            _events: task,
            _keys: keys,
            _appearance: appearance,
            window_backdrop,
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
    fn apply_backdrop(&self, window: &mut Window) {
        let appearance = theme::appearance(self.backdrop(), self.window_backdrop);
        window.set_background_appearance(appearance);
        tracing::info!(backdrop = ?self.backdrop(), supported = self.window_backdrop, ?appearance, "backdrop applied");
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
                    self.settings = Some(settings);
                    self.runtime = Some(runtime);
                    self.apply_theme(window, cx);
                    self.apply_backdrop(window);
                    self.apply_background(window, cx);
                }
                PortalEvent::Changed(change) => {
                    if let Some(settings) = &mut self.settings {
                        settings.apply(&change);
                        self.library
                            .update(cx, |library, cx| library.update_settings(settings, cx));
                    }
                    if matches!(change, SettingChange::ThemeMode(_)) {
                        self.apply_theme(window, cx);
                    }
                    if matches!(change, SettingChange::Backdrop(_)) {
                        self.apply_backdrop(window);
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
                    self.runtime = Some(runtime);
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
                PortalEvent::Rejected { reason, .. } => {
                    tracing::warn!(%reason, "the service did not apply a setting");
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
                        self.page = page;
                    }
                    // Windows restores a minimized window as part of activating it.
                    window.activate_window();
                }
                PortalEvent::Close => {
                    tracing::info!("the service is stopping, closing the Portal");
                    cx.quit();
                }
                _ => {}
            }
        }
        cx.notify();
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
                .child(v_flex().flex_1().min_h_0().child(self.library.clone()))
                .child(self.status_bar.clone()),
        )
    }
}
