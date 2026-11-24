//! The Portal's root entity: the connection to the service, what the service reported, and the window layout.

use gpui_kit::component::{ActiveTheme, TitleBar, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use sucrose_ipc::portal::{PortalEvent, PortalPage, PortalSettings, RuntimeState};

use crate::{
    banner,
    ipc::{ConnectionState, LinkEvent, ServiceLink, next_state},
    library::LibraryView,
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
    /// The task that awaits the link's events; dropped with the app.
    _events: Task<()>,
}

impl PortalApp {
    /// Builds the app, applies `early` (the events that were ready before the window opened) and starts awaiting the
    /// link's events on the GPUI thread.
    pub fn new(
        page: PortalPage,
        link: ServiceLink,
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
            |_| LibraryView::new(window.scale_factor(), link)
        });
        let mut app = Self {
            state: ConnectionState::Connecting,
            settings: None,
            runtime: None,
            page,
            link,
            library,
            _events: task,
        };
        for event in early {
            app.on_link_event(event, window, cx);
        }
        app
    }

    /// Applies one event of the link to the app.
    pub fn on_link_event(&mut self, event: LinkEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.state = next_state(self.state, &event);
        let connected = self.state == ConnectionState::Connected;
        self.library
            .update(cx, |library, cx| library.set_connected(connected, cx));
        if let LinkEvent::Event(event) = event {
            match event {
                PortalEvent::Welcome { settings, runtime } => {
                    self.library.update(cx, |library, cx| {
                        library.load(&settings, cx);
                        library.set_runtime(runtime.clone(), cx);
                    });
                    self.settings = Some(settings);
                    self.runtime = Some(runtime);
                }
                PortalEvent::Changed(change) => {
                    if let Some(settings) = &mut self.settings {
                        settings.apply(&change);
                        self.library
                            .update(cx, |library, cx| library.update_settings(settings, cx));
                    }
                }
                PortalEvent::Runtime(runtime) => {
                    self.library
                        .update(cx, |library, cx| library.set_runtime(runtime.clone(), cx));
                    self.runtime = Some(runtime);
                }
                PortalEvent::UseResult { folder, outcome } => {
                    self.library
                        .update(cx, |library, cx| library.on_use_result(&folder, &outcome, cx));
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
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(TitleBar::new().child(div().text_xs().child(crate::window_title())))
            .when_some(
                banner::connection_banner(self.state, cx.listener(|this, _: &ClickEvent, _, _| this.link.retry())),
                |column, banner| column.child(banner),
            )
            .child(v_flex().flex_1().min_h_0().child(self.library.clone()))
    }
}
