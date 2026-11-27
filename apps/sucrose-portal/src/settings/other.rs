//! The Other page (UI-SPEC "Other"): only the Developer group in this phase. Its Open DevTools button asks the service to
//! open the developer tools of a running web wallpaper; nothing else ever opens them (D-13). The page shows only what
//! the service reported: the monitors that play a wallpaper on a web engine come from the runtime state.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _,
    button::Button,
    h_flex,
    input::{Input, InputEvent},
    switch::Switch,
    tooltip::Tooltip,
};
use gpui_kit::*;
use sucrose_core::settings::EngineType;
use sucrose_ipc::portal::{PortalRequest, PortalSettings, RuntimeState, SettingChange};

use super::{
    row::{
        Choice, ChoiceSelect, ControlState, NUMBER_WIDTH, PortField, SettingRow, choice_select, control_state,
        footer_line, label_text, on_choice, select_box, show_choice,
    },
    wallpaper::info_alert,
};
use crate::{
    dialogs::restart::{RestartSetting, affected_engines, open_restart_prompt, should_prompt},
    i18n::tr,
    ipc::ServiceLink,
};

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
    /// The Debugging Port field.
    port: PortField,
    _subscriptions: Vec<Subscription>,
}

/// The restart prompt a committed developer setting raises, with the committed value and the stored one as text for
/// the comparison; `None` for every other change. Both web engines read these settings when a wallpaper starts.
pub fn restart_trigger(
    change: &SettingChange,
    previous: &PortalSettings,
    now: &PortalSettings,
) -> Option<(RestartSetting, String, String)> {
    match change {
        SettingChange::DeveloperMode(_) => Some((
            RestartSetting::DeveloperMode,
            now.developer_mode.to_string(),
            previous.developer_mode.to_string(),
        )),
        SettingChange::DeveloperPort(_) => Some((
            RestartSetting::DeveloperPort,
            now.developer_port.to_string(),
            previous.developer_port.to_string(),
        )),
        _ => None,
    }
}

/// The legacy hint with the debugging port in its address.
pub fn hint_text(port: u16) -> SharedString {
    SharedString::from(tr("Portal.OtherSettingPage.Developer.DeveloperHint").replace("{0}", &port.to_string()))
}

impl OtherPage {
    /// A page that shows skeletons until the service reports the settings.
    pub fn new(link: ServiceLink, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let screens = choice_select(Vec::new(), None, window, cx);
        let subscription = on_choice(&screens, window, cx, |this, monitor, _, cx| {
            this.chosen = Some(monitor);
            cx.notify();
        });
        let port = PortField::new(window, cx);
        // Enter and leaving the field commit what was typed (UI-SPEC: not per keystroke).
        let port_commit = cx.subscribe_in(&port.state, window, |this, _, event: &InputEvent, window, cx| {
            if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                this.commit_port(window, cx);
            }
        });
        Self {
            link,
            settings: None,
            connected: false,
            running: Vec::new(),
            screens,
            chosen: None,
            port,
            _subscriptions: vec![subscription, port_commit],
        }
    }

    /// Whether the service is connected.
    pub fn set_connected(&mut self, connected: bool, cx: &mut Context<Self>) {
        if self.connected != connected {
            self.connected = connected;
            cx.notify();
        }
    }

    /// The service reported every setting (Welcome): the port field shows the stored port.
    pub fn load(&mut self, settings: &PortalSettings, window: &mut Window, cx: &mut Context<Self>) {
        self.settings = Some(settings.clone());
        self.port.show(settings.developer_port, window, cx);
        cx.notify();
    }

    /// A setting changed or its change was refused. The switch is drawn from the settings and the port field shows the
    /// broadcast port again; a committed change is saved before the restart prompt opens, whatever the answer, and a
    /// refused one leaves the settings as they were, so it never prompts (UI-SPEC E12).
    pub fn on_setting(
        &mut self,
        settings: &PortalSettings,
        change: &SettingChange,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let previous = self.settings.replace(settings.clone());
        if let Some(previous) = previous
            && let Some((setting, committed, stored)) = restart_trigger(change, &previous, settings)
        {
            let running: Vec<EngineType> = self.running.iter().map(|(_, engine)| *engine).collect();
            if should_prompt(setting, &committed, &stored, &running) {
                let engines = affected_engines(setting, &running);
                open_restart_prompt(setting, engines, self.link.clone(), window, cx);
            }
        }
        if matches!(change, SettingChange::DeveloperPort(_)) {
            self.port.show(settings.developer_port, window, cx);
        }
        cx.notify();
    }

    /// Commits the port field: the clamped value is shown and sent when it differs from the stored one; an empty
    /// field reverts.
    fn commit_port(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(settings) = &self.settings else { return };
        let previous = settings.developer_port;
        if self.state() != ControlState::Enabled {
            self.port.show(previous, window, cx);
            return;
        }
        if let Some(port) = self.port.commit(previous, window, cx) {
            self.link
                .send(PortalRequest::Change(SettingChange::DeveloperPort(port)));
        }
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
                show_choice(&self.screens, chosen, window, cx);
            }
        }
        cx.notify();
    }

    /// Whether the keyboard is in a select of this page (Escape then belongs to the select).
    pub fn field_focused(&self, window: &Window, cx: &App) -> bool {
        self.screens.read(cx).focus_handle(cx).contains_focused(window, cx) || self.port.focused(window, cx)
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

    /// The Default Developer Tools switch.
    fn developer_mode_switch(&self, cx: &mut Context<Self>) -> Switch {
        let on = self.settings.as_ref().is_some_and(|settings| settings.developer_mode);
        Switch::new("developer-mode")
            .checked(on)
            .disabled(self.state().disabled())
            .accessibility_label(label_text("Portal.OtherSettingPage.Developer.DeveloperMode"))
            .on_click(cx.listener(|this, value: &bool, _, _| {
                if this.state() == ControlState::Enabled {
                    this.link
                        .send(PortalRequest::Change(SettingChange::DeveloperMode(*value)));
                }
            }))
    }

    /// The Developer group: the Developer Tools row with the developer-mode switch, the debugging port, the legacy hint,
    /// the note that DevTools never open by themselves and the Open DevTools button.
    fn developer_group(&self, cx: &mut Context<Self>) -> AnyElement {
        let port = self.settings.as_ref().map_or(0, |settings| settings.developer_port);
        let mut row = SettingRow::new(
            tr("Portal.OtherSettingPage.Developer"),
            Some(tr("Portal.OtherSettingPage.Developer.Description")),
        )
        .footer(footer_line(
            tr("Portal.OtherSettingPage.Developer.DeveloperMode"),
            self.developer_mode_switch(cx),
        ))
        .footer(footer_line(
            tr("Portal.OtherSettingPage.Developer.DeveloperPort"),
            div()
                .w(px(NUMBER_WIDTH))
                .child(Input::new(&self.port.state).disabled(self.state().disabled())),
        ))
        .footer(info_alert(hint_text(port), cx))
        .footer(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(tr("Portal.OtherSettingPage.Developer.OpenDevTools.Note")),
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

    use sucrose_ipc::portal::{PortalSettings, SettingChange};

    use super::{DevToolsState, hint_text, open_devtools_state, restart_trigger, screen_label};
    use crate::dialogs::restart::RestartSetting;

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

    fn settings() -> PortalSettings {
        PortalSettings {
            app_visible: true,
            app_exit: false,
            run_startup: 0,
            theme_mode: sucrose_core::settings::ThemeMode::default(),
            backdrop: sucrose_core::settings::Backdrop::default(),
            background_image: String::new(),
            background_stretch: sucrose_core::settings::Stretch::default(),
            background_opacity: 0,
            library_pagination: 1,
            adaptive_layout: 0,
            adaptive_margin: 5,
            library_preview: false,
            library_preview_hide: false,
            library_sort_mode: sucrose_core::settings::SortMode::default(),
            library_sort_kind: sucrose_core::settings::SortKind::default(),
            library_location: String::new(),
            library_move: false,
            delete_corrupt: false,
            hint_tray_icon: false,
            gif_engine: sucrose_core::settings::EngineType::default(),
            url_engine: sucrose_core::settings::EngineType::default(),
            web_engine: sucrose_core::settings::EngineType::default(),
            video_engine: sucrose_core::settings::EngineType::default(),
            youtube_engine: sucrose_core::settings::EngineType::default(),
            application_engine: sucrose_core::settings::EngineType::default(),
            wallpaper_loop: true,
            wallpaper_shuffle: true,
            hardware_acceleration: true,
            input_type: sucrose_core::settings::InputType::default(),
            input_module_type: sucrose_core::settings::InputModuleType::default(),
            input_desktop: false,
            allow_invalid_certificates: false,
            developer_mode: false,
            developer_port: 0,
            webview_smartscreen: false,
        }
    }

    #[test]
    fn other_a_committed_developer_setting_names_its_restart_prompt() {
        let previous = settings();
        let mut now = settings();
        now.developer_mode = true;
        now.developer_port = 9222;

        assert_eq!(
            restart_trigger(&SettingChange::DeveloperMode(true), &previous, &now),
            Some((RestartSetting::DeveloperMode, "true".to_owned(), "false".to_owned()))
        );
        assert_eq!(
            restart_trigger(&SettingChange::DeveloperPort(9222), &previous, &now),
            Some((RestartSetting::DeveloperPort, "9222".to_owned(), "0".to_owned()))
        );
        assert_eq!(restart_trigger(&SettingChange::AppExit(true), &previous, &now), None);
    }

    #[test]
    fn other_the_legacy_hint_names_the_debugging_port() {
        let hint = hint_text(9222);

        assert!(hint.contains("http://localhost:9222"), "{hint}");
        assert!(!hint.contains("{0}"), "{hint}");
    }
}
