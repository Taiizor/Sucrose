//! The Other page (UI-SPEC "Other"): only the Developer group in this phase. Its Open DevTools button asks the service to
//! open the developer tools of a running web wallpaper; nothing else ever opens them (D-13). The page shows only what
//! the service reported: the monitors that play a wallpaper on a web engine come from the runtime state.

use gpui_kit::component::{Disableable as _, button::Button, h_flex, tooltip::Tooltip};
use gpui_kit::*;
use sucrose_core::settings::EngineType;
use sucrose_ipc::portal::{PortalRequest, PortalSettings, RuntimeState};

use super::row::{
    Choice, ChoiceSelect, ControlState, SettingRow, choice_select, control_state, footer_line, on_choice, select_box,
};
use crate::{i18n::tr, ipc::ServiceLink};

/// What the Open DevTools button can do for the wallpapers that run now (UI-SPEC E10: empty and partial).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevToolsState {
    /// No wallpaper plays on a web engine: the button is disabled with its tooltip.
    Disabled,
    /// One monitor plays one: the button targets it directly.
    Single(String),
    /// Several monitors play one: the Screen select picks the target.
    Choose(Vec<String>),
}

/// The state of the button for `running`, the monitor and the engine of each wallpaper the service reports. A monitor is
/// listed once, in the service's order. The developer-mode switch does not matter (D-13).
pub fn open_devtools_state(running: &[(String, EngineType)]) -> DevToolsState {
    let mut monitors: Vec<String> = Vec::new();
    for (monitor, engine) in running {
        if matches!(engine, EngineType::WebViewLive | EngineType::CefSharpLive) && !monitors.contains(monitor) {
            monitors.push(monitor.clone());
        }
    }
    match monitors.len() {
        0 => DevToolsState::Disabled,
        1 => DevToolsState::Single(monitors.remove(0)),
        _ => DevToolsState::Choose(monitors),
    }
}

/// The text of a monitor in the Screen select: its device name without the Windows namespace prefix.
pub fn screen_label(monitor: &str) -> SharedString {
    SharedString::from(monitor.trim_start_matches(r"\\.\").to_owned())
}

/// The Other settings page.
pub struct OtherPage {
    link: ServiceLink,
    settings: Option<PortalSettings>,
    connected: bool,
    /// The monitor and the engine of each wallpaper that runs now, as the service last reported.
    running: Vec<(String, EngineType)>,
    /// The Screen select, shown when several monitors play a web wallpaper.
    screens: ChoiceSelect<String>,
    /// The monitor the Screen select chose.
    chosen: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl OtherPage {
    /// A page that shows skeletons until the service reports the settings.
    pub fn new(link: ServiceLink, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let screens = choice_select(Vec::new(), None, window, cx);
        let subscription = on_choice(&screens, window, cx, |this, monitor, _, cx| {
            this.chosen = Some(monitor);
            cx.notify();
        });
        Self {
            link,
            settings: None,
            connected: false,
            running: Vec::new(),
            screens,
            chosen: None,
            _subscriptions: vec![subscription],
        }
    }

    /// Whether the service is connected.
    pub fn set_connected(&mut self, connected: bool, cx: &mut Context<Self>) {
        if self.connected != connected {
            self.connected = connected;
            cx.notify();
        }
    }

    /// The service reported every setting (Welcome).
    pub fn load(&mut self, settings: &PortalSettings, cx: &mut Context<Self>) {
        self.settings = Some(settings.clone());
        cx.notify();
    }

    /// The service reported what runs (Welcome and Runtime): the Screen select lists the monitors of the web wallpapers.
    pub fn set_runtime(&mut self, runtime: &RuntimeState, window: &mut Window, cx: &mut Context<Self>) {
        self.running.clone_from(&runtime.running_engines);
        if let DevToolsState::Choose(monitors) = open_devtools_state(&self.running) {
            if !self.chosen.as_ref().is_some_and(|chosen| monitors.contains(chosen)) {
                self.chosen = monitors.first().cloned();
            }
            let items = monitors
                .iter()
                .map(|monitor| Choice::new(monitor.clone(), screen_label(monitor)))
                .collect();
            self.screens.update(cx, |state, cx| state.set_items(items, window, cx));
            if let Some(chosen) = &self.chosen {
                super::row::show_choice(&self.screens, chosen, window, cx);
            }
        }
        cx.notify();
    }

    /// Whether the keyboard is in a select of this page (Escape then belongs to the select).
    pub fn field_focused(&self, window: &Window, cx: &App) -> bool {
        self.screens.read(cx).focus_handle(cx).contains_focused(window, cx)
    }

    fn state(&self) -> ControlState {
        control_state(self.settings.is_some(), self.connected)
    }

    /// The monitor Open DevTools goes to now, if there is one.
    fn target(&self) -> Option<String> {
        match open_devtools_state(&self.running) {
            DevToolsState::Disabled => None,
            DevToolsState::Single(monitor) => Some(monitor),
            DevToolsState::Choose(monitors) => self
                .chosen
                .clone()
                .filter(|chosen| monitors.contains(chosen))
                .or_else(|| monitors.first().cloned()),
        }
    }

    /// Asks the service to open the developer tools of the chosen wallpaper (D-13); the answer is a toast on failure.
    fn open_devtools(&self) {
        if self.state() != ControlState::Enabled {
            return;
        }
        if let Some(monitor) = self.target() {
            self.link.send(PortalRequest::OpenDevTools { monitor });
        }
    }

    /// The footer of the Developer Tools row: the Screen select when several wallpapers run, then Open DevTools.
    fn open_devtools_lines(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let state = self.state();
        let devtools = open_devtools_state(&self.running);
        let enabled = !state.disabled() && devtools != DevToolsState::Disabled;
        let mut lines = Vec::new();
        if matches!(devtools, DevToolsState::Choose(_)) {
            lines.push(footer_line(
                tr("Portal.OtherSettingPage.Developer.OpenDevTools.Screen"),
                select_box(&self.screens, state.disabled()),
            ));
        }
        let button = Button::new("open-devtools")
            .outline()
            .label(tr("Portal.OtherSettingPage.Developer.OpenDevTools"))
            .disabled(!enabled)
            .on_click(cx.listener(|this, _: &ClickEvent, _, _| this.open_devtools()));
        // The reason the button is off is its tooltip: only when no web wallpaper runs.
        let boxed = if devtools == DevToolsState::Disabled {
            let reason = tr("Portal.OtherSettingPage.Developer.OpenDevTools.Disabled");
            div()
                .id("open-devtools-box")
                .tooltip(move |window, cx| Tooltip::new(reason.clone()).build(window, cx))
                .child(button)
                .into_any_element()
        } else {
            div().child(button).into_any_element()
        };
        lines.push(h_flex().w_full().justify_end().child(boxed).into_any_element());
        lines
    }

    /// The Developer group: the Developer Tools row.
    fn developer_group(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut row = SettingRow::new(
            tr("Portal.OtherSettingPage.Developer"),
            Some(tr("Portal.OtherSettingPage.Developer.Description")),
        )
        .state(self.state());
        for line in self.open_devtools_lines(cx) {
            row = row.footer(line);
        }
        super::group(tr("Portal.Area.Developer"), vec![row.build(cx)])
    }
}

impl Render for OtherPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        super::column("settings-other", vec![self.developer_group(cx)])
    }
}

#[cfg(test)]
mod tests {
    use sucrose_core::settings::EngineType::{CefSharpLive, MpvPlayerLive, WebViewLive};

    use super::{DevToolsState, open_devtools_state, screen_label};

    fn running(
        list: &[(&str, sucrose_core::settings::EngineType)],
    ) -> Vec<(String, sucrose_core::settings::EngineType)> {
        list.iter()
            .map(|(monitor, engine)| ((*monitor).to_owned(), *engine))
            .collect()
    }

    #[test]
    fn other_open_devtools_is_disabled_when_no_web_engine_wallpaper_runs() {
        assert_eq!(open_devtools_state(&[]), DevToolsState::Disabled);
        assert_eq!(
            open_devtools_state(&running(&[("D1", MpvPlayerLive), ("D2", MpvPlayerLive)])),
            DevToolsState::Disabled
        );
    }

    #[test]
    fn other_open_devtools_targets_the_only_web_wallpaper_directly() {
        assert_eq!(
            open_devtools_state(&running(&[("D1", CefSharpLive)])),
            DevToolsState::Single("D1".to_owned())
        );
        // Another engine's wallpaper does not make a Screen select necessary.
        assert_eq!(
            open_devtools_state(&running(&[("D1", MpvPlayerLive), ("D2", WebViewLive)])),
            DevToolsState::Single("D2".to_owned())
        );
    }

    #[test]
    fn other_open_devtools_asks_for_the_screen_when_several_web_wallpapers_run() {
        assert_eq!(
            open_devtools_state(&running(&[
                ("D1", CefSharpLive),
                ("D3", CefSharpLive),
                ("D1", CefSharpLive)
            ])),
            DevToolsState::Choose(vec!["D1".to_owned(), "D3".to_owned()]),
            "a monitor is listed once, in the service's order"
        );
    }

    #[test]
    fn other_screen_label_drops_the_windows_device_prefix() {
        assert_eq!(&*screen_label(r"\\.\DISPLAY1"), "DISPLAY1");
        assert_eq!(&*screen_label("DP-1"), "DP-1");
    }
}
