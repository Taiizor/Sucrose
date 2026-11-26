//! The Wallpaper page (UI-SPEC "Settings pages: Wallpaper"): the Wallpaper group (Interaction Method, Loop Mode, Shuffle
//! Mode and Hardware Acceleration; Screen Layout and Stretch Mode wait for Round C), the Extensions group with the Video
//! Player and the Engines group with the Web Engine. Later plans add the other engine rows and the Web Engine Settings.
//! Every control sends a SettingChange and shows only what the service broadcast; the engine choices come from the
//! engine report (the Portal never decides which engines exist), plus the stored engine when the report does not offer it.

use gpui_kit::component::{Disableable as _, checkbox::Checkbox, switch::Switch};
use gpui_kit::*;
use sucrose_core::{
    settings::{EngineType, InputModuleType, InputType},
    theme::WallpaperType,
};
use sucrose_ipc::portal::{EngineReportView, EngineState, PortalRequest, PortalSettings, SettingChange};

use super::row::{
    Choice, ChoiceSelect, ControlState, SELECT_WIDTH, SettingRow, choice_select, control_state, footer_line,
    label_text, on_choice, select_box, show_choice,
};
use crate::{i18n::tr, ipc::ServiceLink};

/// The engine rows of the page, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineRow {
    /// Video Player, in the Extensions group.
    Video,
    /// Web Engine, in the Engines group.
    Web,
}

impl EngineRow {
    /// Every row, in the order of `WallpaperPage::selects`.
    pub const ALL: [Self; 2] = [Self::Video, Self::Web];

    /// The wallpaper type whose engine the row sets.
    pub fn kind(self) -> WallpaperType {
        match self {
            Self::Video => WallpaperType::Video,
            Self::Web => WallpaperType::Web,
        }
    }

    /// The engines the legacy app lets this type use, in the legacy order.
    pub fn allowed(self) -> &'static [EngineType] {
        match self {
            Self::Video => EngineType::VIDEO,
            Self::Web => EngineType::WEB,
        }
    }

    /// The engine the service reported for this type.
    pub fn value(self, settings: &PortalSettings) -> EngineType {
        match self {
            Self::Video => settings.video_engine,
            Self::Web => settings.web_engine,
        }
    }

    /// The change that sets this row to `engine`.
    pub fn change(self, engine: EngineType) -> SettingChange {
        match self {
            Self::Video => SettingChange::VideoEngine(engine),
            Self::Web => SettingChange::WebEngine(engine),
        }
    }

    /// The row a change belongs to, if it belongs to one.
    pub fn of(change: &SettingChange) -> Option<Self> {
        match change {
            SettingChange::VideoEngine(_) => Some(Self::Video),
            SettingChange::WebEngine(_) => Some(Self::Web),
            _ => None,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Video => 0,
            Self::Web => 1,
        }
    }

    /// The legacy keys of the row's title and description.
    fn keys(self) -> (&'static str, &'static str) {
        match self {
            Self::Video => (
                "Portal.WallpaperSettingPage.VideoPlayer",
                "Portal.WallpaperSettingPage.VideoPlayer.Description",
            ),
            Self::Web => (
                "Portal.WallpaperSettingPage.WebPlayer",
                "Portal.WallpaperSettingPage.WebPlayer.Description",
            ),
        }
    }
}

/// The switch rows of the Wallpaper group, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// Loop Mode: running wallpapers follow at once.
    Loop,
    /// Shuffle Mode: running web pages follow at once.
    Shuffle,
    /// Hardware Acceleration: read the next time a wallpaper starts.
    HardwareAcceleration,
}

impl Flag {
    /// Every switch, in the legacy order.
    pub const ALL: [Self; 3] = [Self::Loop, Self::Shuffle, Self::HardwareAcceleration];

    /// The value the service reported for this switch.
    pub fn value(self, settings: &PortalSettings) -> bool {
        match self {
            Self::Loop => settings.wallpaper_loop,
            Self::Shuffle => settings.wallpaper_shuffle,
            Self::HardwareAcceleration => settings.hardware_acceleration,
        }
    }

    /// The change that sets this switch to `on`.
    pub fn change(self, on: bool) -> SettingChange {
        match self {
            Self::Loop => SettingChange::WallpaperLoop(on),
            Self::Shuffle => SettingChange::WallpaperShuffle(on),
            Self::HardwareAcceleration => SettingChange::HardwareAcceleration(on),
        }
    }

    /// The element id of the switch and the legacy keys of the row's title and description.
    fn keys(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Loop => (
                "wallpaper-loop",
                "Portal.WallpaperSettingPage.LoopMode",
                "Portal.WallpaperSettingPage.LoopMode.Description",
            ),
            Self::Shuffle => (
                "wallpaper-shuffle",
                "Portal.WallpaperSettingPage.ShuffleMode",
                "Portal.WallpaperSettingPage.ShuffleMode.Description",
            ),
            Self::HardwareAcceleration => (
                "wallpaper-hardware-acceleration",
                "Portal.WallpaperSettingPage.HardwareAcceleration",
                "Portal.WallpaperSettingPage.HardwareAcceleration.Description",
            ),
        }
    }
}

/// The key of an interaction method's label.
pub fn input_type_label_key(input: InputType) -> &'static str {
    match input {
        InputType::Close => "Portal.Enum.InputType.Close",
        InputType::OnlyMouse => "Portal.Enum.InputType.OnlyMouse",
        InputType::OnlyKeyboard => "Portal.Enum.InputType.OnlyKeyboard",
        InputType::MouseKeyboard => "Portal.Enum.InputType.MouseKeyboard",
    }
}

/// The options of the Interaction Method select, in the legacy order.
pub fn input_type_choices() -> Vec<Choice<InputType>> {
    [
        InputType::Close,
        InputType::OnlyMouse,
        InputType::OnlyKeyboard,
        InputType::MouseKeyboard,
    ]
    .into_iter()
    .map(|input| Choice::new(input, tr(input_type_label_key(input))))
    .collect()
}

/// The options of the Interaction Module select: RawInput is the only module (the legacy Native one is not ported).
pub fn input_module_choices() -> Vec<Choice<InputModuleType>> {
    vec![Choice::new(
        InputModuleType::RawInput,
        tr("Portal.Enum.InputModuleType.RawInput"),
    )]
}

/// The key of an engine's label.
pub fn engine_label_key(engine: EngineType) -> &'static str {
    match engine {
        EngineType::VexanaLive | EngineType::XavierLive => "Portal.Enum.EngineType.BuiltInGif",
        EngineType::NebulaLive => "Portal.Enum.EngineType.BuiltInVideo",
        EngineType::WebViewLive => "Portal.Enum.EngineType.WebView2",
        EngineType::CefSharpLive => "Portal.Enum.EngineType.Cef",
        EngineType::MpvPlayerLive => "Portal.Enum.EngineType.MpvPlayer",
        EngineType::VlcPlayerLive => "Portal.Enum.EngineType.VlcPlayer",
        EngineType::AuroraLive => "Portal.Enum.EngineType.Aurora",
    }
}

/// The options of a row's select: the engines of the type's legacy list that the report offers (installed or not), plus
/// the stored engine when it is not among them, in the legacy order. An engine that is not installed carries the muted
/// "Not installed" suffix and stays selectable (D-11). Before the report arrives only the stored engine is listed.
pub fn engine_choices(
    row: EngineRow,
    report: Option<&EngineReportView>,
    stored: EngineType,
) -> Vec<Choice<EngineType>> {
    let offered: &[EngineType] = report
        .and_then(|report| report.offered.iter().find(|offered| offered.kind == row.kind()))
        .map_or(&[], |offered| offered.engines.as_slice());
    row.allowed()
        .iter()
        .copied()
        .filter(|engine| offered.contains(engine) || *engine == stored)
        .map(|engine| {
            let choice = Choice::new(engine, tr(engine_label_key(engine)));
            // Without a report nothing is known to be missing.
            let installed = report.is_none_or(|report| {
                report
                    .engines
                    .iter()
                    .any(|known| known.engine == engine && matches!(known.state, EngineState::Ready { .. }))
            });
            if installed {
                choice
            } else {
                choice.with_suffix(tr("Portal.WallpaperSettingPage.Engine.NotInstalled"))
            }
        })
        .collect()
}

/// Whether a change moves a select of this page, so the selects must show the broadcast value again (the switches are
/// drawn from the settings and need nothing).
fn needs_refresh(change: &SettingChange) -> bool {
    EngineRow::of(change).is_some() || matches!(change, SettingChange::InputType(_) | SettingChange::InputModuleType(_))
}

/// The Wallpaper settings page.
pub struct WallpaperPage {
    link: ServiceLink,
    settings: Option<PortalSettings>,
    engines: Option<EngineReportView>,
    connected: bool,
    /// One select per [`EngineRow`], in the order of [`EngineRow::ALL`].
    selects: [ChoiceSelect<EngineType>; 2],
    /// The Interaction Method select.
    input_type: ChoiceSelect<InputType>,
    /// The Interaction Module select (RawInput only).
    input_module: ChoiceSelect<InputModuleType>,
    _subscriptions: Vec<Subscription>,
}

impl WallpaperPage {
    /// A page that shows skeletons until the service reports the settings.
    pub fn new(link: ServiceLink, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let selects = EngineRow::ALL.map(|_| choice_select(Vec::new(), None, window, cx));
        let input_type = choice_select(input_type_choices(), None, window, cx);
        let input_module = choice_select(input_module_choices(), None, window, cx);
        let mut subscriptions: Vec<Subscription> = EngineRow::ALL
            .into_iter()
            .map(|row| {
                on_choice(&selects[row.index()], window, cx, move |this, engine, _, _| {
                    this.send(row.change(engine));
                })
            })
            .collect();
        subscriptions.push(on_choice(&input_type, window, cx, |this, input, _, _| {
            this.send(SettingChange::InputType(input));
        }));
        subscriptions.push(on_choice(&input_module, window, cx, |this, module, _, _| {
            this.send(SettingChange::InputModuleType(module));
        }));
        Self {
            link,
            settings: None,
            engines: None,
            connected: false,
            selects,
            input_type,
            input_module,
            _subscriptions: subscriptions,
        }
    }

    /// Whether the service is connected.
    pub fn set_connected(&mut self, connected: bool, cx: &mut Context<Self>) {
        if self.connected != connected {
            self.connected = connected;
            cx.notify();
        }
    }

    /// The service reported every setting (Welcome): every select shows its engine.
    pub fn load(&mut self, settings: &PortalSettings, window: &mut Window, cx: &mut Context<Self>) {
        self.settings = Some(settings.clone());
        self.refresh(window, cx);
        cx.notify();
    }

    /// A setting changed or its change was refused: the select of `change` shows its broadcast value again.
    pub fn on_setting(
        &mut self,
        settings: &PortalSettings,
        change: &SettingChange,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings = Some(settings.clone());
        if needs_refresh(change) {
            self.refresh(window, cx);
        }
        // The switches and the checkbox are drawn from `settings`, so they show the broadcast value again by themselves.
        cx.notify();
    }

    /// The service sent the engine report: the choices follow it.
    pub fn set_engines(&mut self, report: &EngineReportView, window: &mut Window, cx: &mut Context<Self>) {
        self.engines = Some(report.clone());
        self.refresh(window, cx);
        cx.notify();
    }

    /// The page was opened: look at the engines again, so an engine installed meanwhile shows as installed.
    pub fn on_opened(&self) {
        if self.state() == ControlState::Enabled {
            self.link.send(PortalRequest::DetectEngines);
        }
    }

    /// Whether the keyboard is in a select of this page (Escape then belongs to the select).
    pub fn field_focused(&self, window: &Window, cx: &App) -> bool {
        self.selects
            .iter()
            .any(|select| select.read(cx).focus_handle(cx).contains_focused(window, cx))
            || self.input_type.read(cx).focus_handle(cx).contains_focused(window, cx)
            || self.input_module.read(cx).focus_handle(cx).contains_focused(window, cx)
    }

    /// Puts the choices and the broadcast engine into every select, without telling anyone.
    fn refresh(&self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(settings) = &self.settings else { return };
        for row in EngineRow::ALL {
            let stored = row.value(settings);
            let items = engine_choices(row, self.engines.as_ref(), stored);
            let select = &self.selects[row.index()];
            select.update(cx, |state, cx| state.set_items(items, window, cx));
            show_choice(select, &stored, window, cx);
        }
        show_choice(&self.input_type, &settings.input_type, window, cx);
        show_choice(&self.input_module, &settings.input_module_type, window, cx);
    }

    /// Sends `change`; the selects keep showing the broadcast value until the service answers.
    fn send(&self, change: SettingChange) {
        if self.state() == ControlState::Enabled {
            self.link.send(PortalRequest::Change(change));
        }
    }

    fn state(&self) -> ControlState {
        control_state(self.settings.is_some(), self.connected)
    }

    /// A switch that shows `checked` and sends the change of `flag`.
    fn switch(&self, flag: Flag, checked: bool, cx: &mut Context<Self>) -> Switch {
        let (id, title, _) = flag.keys();
        Switch::new(id)
            .checked(checked)
            .disabled(self.state().disabled())
            .accessibility_label(label_text(title))
            .on_click(cx.listener(move |this, value: &bool, _, _| this.send(flag.change(*value))))
    }

    /// The Wallpaper group: Interaction Method with its module and desktop-focus options in the footer, then the Loop
    /// Mode, Shuffle Mode and Hardware Acceleration switches (the legacy order).
    fn wallpaper_group(&self, cx: &mut Context<Self>) -> AnyElement {
        let state = self.state();
        let settings = self.settings.clone();
        let interaction = SettingRow::new(
            tr("Portal.WallpaperSettingPage.InputMode"),
            Some(tr("Portal.WallpaperSettingPage.InputMode.Description")),
        )
        .control(select_box(&self.input_type, state.disabled()), SELECT_WIDTH)
        .footer(footer_line(
            label_text("Portal.WallpaperSettingPage.InputMode.InputModule"),
            select_box(&self.input_module, state.disabled()),
        ))
        .footer(
            Checkbox::new("wallpaper-input-desktop")
                .label(tr("Portal.WallpaperSettingPage.InputMode.InputDesktop"))
                .checked(settings.as_ref().is_some_and(|s| s.input_desktop))
                .disabled(state.disabled())
                .on_click(cx.listener(|this, value: &bool, _, _| this.send(SettingChange::InputDesktop(*value)))),
        )
        .state(state)
        .build(cx);
        let mut rows = vec![interaction];
        for flag in Flag::ALL {
            let (_, title, description) = flag.keys();
            let on = settings.as_ref().is_some_and(|s| flag.value(s));
            rows.push(
                SettingRow::new(tr(title), Some(tr(description)))
                    .control(self.switch(flag, on, cx), 40.)
                    .state(state)
                    .build(cx),
            );
        }
        super::group(tr("Portal.SettingsPage.GroupWallpaper"), rows)
    }

    /// The row of `row`: title, description and the engine select.
    fn engine_row(&self, row: EngineRow, cx: &App) -> AnyElement {
        let state = self.state();
        let (title, description) = row.keys();
        SettingRow::new(tr(title), Some(tr(description)))
            .control(select_box(&self.selects[row.index()], state.disabled()), SELECT_WIDTH)
            .state(state)
            .build(cx)
    }
}

impl Render for WallpaperPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        super::column(
            "settings-wallpaper",
            vec![
                self.wallpaper_group(cx),
                super::group(
                    tr("Portal.Area.Extensions"),
                    vec![self.engine_row(EngineRow::Video, cx)],
                ),
                super::group(tr("Portal.Area.Engines"), vec![self.engine_row(EngineRow::Web, cx)]),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use sucrose_core::{settings::EngineType, theme::WallpaperType};
    use sucrose_ipc::portal::{EngineReportView, EngineState, EngineStateView, OfferedView, SettingChange};

    use sucrose_core::settings::{InputModuleType, InputType};
    use sucrose_ipc::portal::PortalSettings;

    use super::{EngineRow, Flag, engine_choices, input_module_choices, input_type_choices, needs_refresh};

    fn state(engine: EngineType, state: EngineState) -> EngineStateView {
        EngineStateView { engine, state }
    }

    fn ready() -> EngineState {
        EngineState::Ready { version: None }
    }

    fn missing() -> EngineState {
        EngineState::Missing {
            hint: sucrose_ipc::portal::HintView::FilesMissing,
            installable: false,
        }
    }

    /// A report where mpv and CEF are installed and nothing else is in this build.
    fn round_a() -> EngineReportView {
        EngineReportView {
            engines: vec![
                state(EngineType::MpvPlayerLive, ready()),
                state(EngineType::CefSharpLive, ready()),
                state(
                    EngineType::WebViewLive,
                    EngineState::Unsupported { architecture: false },
                ),
            ],
            supported: vec![WallpaperType::Video, WallpaperType::Web],
            playable: vec![WallpaperType::Video, WallpaperType::Web],
            offered: vec![
                OfferedView {
                    kind: WallpaperType::Video,
                    engines: vec![EngineType::MpvPlayerLive],
                },
                OfferedView {
                    kind: WallpaperType::Web,
                    engines: vec![EngineType::CefSharpLive],
                },
            ],
            fallbacks: Vec::new(),
        }
    }

    /// What a select shows for each option: the label, and the suffix in brackets when there is one.
    fn shown(choices: &[super::Choice<EngineType>]) -> Vec<(EngineType, String, bool)> {
        choices
            .iter()
            .map(|choice| (choice.value, choice.title.to_string(), choice.suffix.is_some()))
            .collect()
    }

    fn settings() -> PortalSettings {
        PortalSettings {
            app_visible: true,
            app_exit: true,
            run_startup: 0,
            theme_mode: sucrose_core::settings::ThemeMode::default(),
            backdrop: sucrose_core::settings::Backdrop::default(),
            background_image: String::new(),
            background_stretch: sucrose_core::settings::Stretch::default(),
            background_opacity: 100,
            library_pagination: 30,
            adaptive_layout: 0,
            adaptive_margin: 5,
            library_preview: false,
            library_preview_hide: false,
            library_sort_mode: sucrose_core::settings::SortMode::default(),
            library_sort_kind: sucrose_core::settings::SortKind::default(),
            library_location: String::new(),
            library_move: true,
            delete_corrupt: false,
            hint_tray_icon: true,
            gif_engine: EngineType::MpvPlayerLive,
            url_engine: EngineType::CefSharpLive,
            web_engine: EngineType::CefSharpLive,
            video_engine: EngineType::MpvPlayerLive,
            youtube_engine: EngineType::CefSharpLive,
            application_engine: EngineType::AuroraLive,
            wallpaper_loop: true,
            wallpaper_shuffle: false,
            hardware_acceleration: true,
            input_type: InputType::MouseKeyboard,
            input_module_type: InputModuleType::RawInput,
            input_desktop: false,
        }
    }

    #[test]
    fn wallpaper_group_the_interaction_method_lists_the_four_legacy_values_in_the_legacy_order() {
        let shown: Vec<(InputType, String)> = input_type_choices()
            .iter()
            .map(|choice| (choice.value, choice.title.to_string()))
            .collect();

        assert_eq!(
            shown,
            [
                (InputType::Close, "Closed".to_owned()),
                (InputType::OnlyMouse, "Mouse Only".to_owned()),
                (InputType::OnlyKeyboard, "Keyboard Only".to_owned()),
                (InputType::MouseKeyboard, "Mouse & Keyboard".to_owned()),
            ]
        );
    }

    #[test]
    fn wallpaper_group_the_interaction_module_select_offers_rawinput_only() {
        let shown: Vec<(InputModuleType, String)> = input_module_choices()
            .iter()
            .map(|choice| (choice.value, choice.title.to_string()))
            .collect();

        assert_eq!(shown, [(InputModuleType::RawInput, "RawInput".to_owned())]);
    }

    #[test]
    fn wallpaper_group_the_switches_follow_the_legacy_order_and_send_their_own_change() {
        assert_eq!(
            Flag::ALL,
            [Flag::Loop, Flag::Shuffle, Flag::HardwareAcceleration],
            "Loop Mode, Shuffle Mode, Hardware Acceleration"
        );
        assert_eq!(Flag::Loop.change(false), SettingChange::WallpaperLoop(false));
        assert_eq!(Flag::Shuffle.change(true), SettingChange::WallpaperShuffle(true));
        assert_eq!(
            Flag::HardwareAcceleration.change(false),
            SettingChange::HardwareAcceleration(false)
        );
    }

    #[test]
    fn wallpaper_group_each_switch_shows_the_value_the_service_reported() {
        let settings = settings();

        assert!(Flag::Loop.value(&settings));
        assert!(!Flag::Shuffle.value(&settings));
        assert!(Flag::HardwareAcceleration.value(&settings));
    }

    #[test]
    fn wallpaper_group_only_the_changes_of_a_select_refresh_the_selects() {
        assert!(needs_refresh(&SettingChange::InputType(InputType::Close)));
        assert!(needs_refresh(&SettingChange::InputModuleType(
            InputModuleType::RawInput
        )));
        assert!(needs_refresh(&SettingChange::VideoEngine(EngineType::MpvPlayerLive)));
        assert!(!needs_refresh(&SettingChange::WallpaperLoop(false)));
        assert!(!needs_refresh(&SettingChange::InputDesktop(true)));
        assert!(!needs_refresh(&SettingChange::LibraryPagination(5)));
    }

    #[test]
    fn wallpaper_rows_set_their_own_engine() {
        assert_eq!(
            EngineRow::Web.change(EngineType::CefSharpLive),
            SettingChange::WebEngine(EngineType::CefSharpLive)
        );
        assert_eq!(
            EngineRow::Video.change(EngineType::MpvPlayerLive),
            SettingChange::VideoEngine(EngineType::MpvPlayerLive)
        );
        for row in EngineRow::ALL {
            assert_eq!(EngineRow::of(&row.change(EngineType::default())), Some(row));
        }
        assert_eq!(EngineRow::of(&SettingChange::AppExit(true)), None);
    }

    #[test]
    fn wallpaper_a_type_with_one_installed_engine_still_lists_it() {
        let choices = engine_choices(EngineRow::Web, Some(&round_a()), EngineType::CefSharpLive);

        assert_eq!(shown(&choices), [(EngineType::CefSharpLive, "CEF".to_owned(), false)]);
    }

    #[test]
    fn wallpaper_the_stored_engine_is_listed_with_the_not_installed_suffix_when_the_build_has_no_such_engine() {
        let choices = engine_choices(EngineRow::Web, Some(&round_a()), EngineType::WebViewLive);

        assert_eq!(
            shown(&choices),
            [
                (EngineType::WebViewLive, "WebView2".to_owned(), true),
                (EngineType::CefSharpLive, "CEF".to_owned(), false),
            ],
            "the legacy order, the stored engine marked"
        );
        assert_eq!(choices[0].suffix.as_deref(), Some("Not installed"));
    }

    #[test]
    fn wallpaper_an_offered_engine_that_is_missing_stays_selectable_with_the_suffix() {
        let mut report = round_a();
        report.engines[1].state = missing();
        report.offered[1].engines = vec![EngineType::WebViewLive, EngineType::CefSharpLive];
        report.engines[2].state = ready();

        let choices = engine_choices(EngineRow::Web, Some(&report), EngineType::WebViewLive);

        assert_eq!(
            shown(&choices),
            [
                (EngineType::WebViewLive, "WebView2".to_owned(), false),
                (EngineType::CefSharpLive, "CEF".to_owned(), true),
            ]
        );
        assert!(
            choices.iter().all(|choice| choice.reason.is_none()),
            "nothing is disabled"
        );
    }

    #[test]
    fn wallpaper_the_video_player_lists_what_the_report_offers_in_the_legacy_order() {
        let mut report = round_a();
        report.offered[0].engines = vec![EngineType::MpvPlayerLive, EngineType::NebulaLive];
        report.engines.push(state(EngineType::NebulaLive, ready()));

        let choices = engine_choices(EngineRow::Video, Some(&report), EngineType::MpvPlayerLive);

        assert_eq!(
            shown(&choices),
            [
                (EngineType::NebulaLive, "Built-in".to_owned(), false),
                (EngineType::MpvPlayerLive, "mpv".to_owned(), false),
            ]
        );
    }

    #[test]
    fn wallpaper_before_the_report_only_the_stored_engine_is_listed_without_a_suffix() {
        let choices = engine_choices(EngineRow::Video, None, EngineType::MpvPlayerLive);

        assert_eq!(shown(&choices), [(EngineType::MpvPlayerLive, "mpv".to_owned(), false)]);
    }

    #[test]
    fn wallpaper_an_engine_the_type_never_allowed_is_not_listed_even_when_stored() {
        let choices = engine_choices(EngineRow::Web, Some(&round_a()), EngineType::MpvPlayerLive);

        assert_eq!(shown(&choices), [(EngineType::CefSharpLive, "CEF".to_owned(), false)]);
    }
}
