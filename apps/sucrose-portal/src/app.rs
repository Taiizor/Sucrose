//! The Portal's root entity: the connection to the service, what the service reported, and the window layout.

use gpui_kit::component::{ActiveTheme, TitleBar, v_flex};
use gpui_kit::*;
use sucrose_ipc::portal::{PortalEvent, PortalPage, PortalSettings, RuntimeState};

use crate::ipc::{ConnectionState, LinkEvent, ServiceLink, next_state};

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
    /// The task that awaits the link's events; dropped with the app.
    _events: Task<()>,
}

impl PortalApp {
    /// Builds the app and starts awaiting the link's events on the GPUI thread.
    pub fn new(
        page: PortalPage,
        link: ServiceLink,
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
        Self {
            state: ConnectionState::Connecting,
            settings: None,
            runtime: None,
            page,
            link,
            _events: task,
        }
    }

    /// Applies one event of the link to the app.
    pub fn on_link_event(&mut self, event: LinkEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.state = next_state(self.state, &event);
        if let LinkEvent::Event(PortalEvent::Welcome { settings, runtime }) = event {
            self.settings = Some(settings);
            self.runtime = Some(runtime);
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
            .child(v_flex().flex_1())
    }
}
