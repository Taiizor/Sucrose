//! The Wallpaper page (UI-SPEC "Settings pages: Wallpaper"): the Wallpaper group (Interaction Method, Loop Mode, Shuffle
//! Mode and Hardware Acceleration; Screen Layout and Stretch Mode wait for Round C), the Extensions group with the Video
//! Player and the Engines group with the Web Engine. Later plans add the other engine rows and the Web Engine Settings.
//! Every control sends a SettingChange and shows only what the service broadcast; the engine choices come from the
//! engine report (the Portal never decides which engines exist), plus the stored engine when the report does not offer it.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, button::Button, checkbox::Checkbox, h_flex,
    spinner::Spinner, switch::Switch,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use sucrose_core::{
    settings::{EngineType, InputModuleType, InputType},
    theme::WallpaperType,
};
use sucrose_ipc::portal::{
    EngineReportView, EngineState, HintView, PortalRequest, PortalSettings, PrerequisiteView, RuntimeState,
    SettingChange, SwitchView,
};

use super::arguments::{ArgumentsEditor, saved_list};
use super::row::{
    Choice, ChoiceSelect, ControlState, SELECT_WIDTH, SettingRow, choice_select, control_state, footer_line,
    label_text, on_choice, select_box, show_choice,
};
use crate::{
    dialogs::{
        install::open_install_dialog,
        restart::{RestartSetting, affected_engines, open_restart_prompt, should_prompt},
    },
    i18n::{tr, tr_with},
    ipc::ServiceLink,
    theme,
};

/// The engine rows of the page, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineRow {
    /// Gif Player, in the Extensions group.
    Gif,
    /// Video Player, in the Extensions group.
    Video,
    /// Url Engine, in the Engines group.
    Url,
    /// Web Engine, in the Engines group.
    Web,
    /// YouTube Engine, in the Engines group.
    YouTube,
}

impl EngineRow {
    /// Every row, in the order of `WallpaperPage::selects` (the legacy page order: Gif and Video in Extensions, then Url,
    /// Web and YouTube in Engines).
    pub const ALL: [Self; 5] = [Self::Gif, Self::Video, Self::Url, Self::Web, Self::YouTube];

    /// The wallpaper type whose engine the row sets.
    pub fn kind(self) -> WallpaperType {
        match self {
            Self::Gif => WallpaperType::Gif,
            Self::Video => WallpaperType::Video,
            Self::Url => WallpaperType::Url,
            Self::Web => WallpaperType::Web,
            Self::YouTube => WallpaperType::YouTube,
        }
    }

    /// The engines the legacy app lets this type use, in the legacy order.
    pub fn allowed(self) -> &'static [EngineType] {
        match self {
            Self::Gif => EngineType::GIF,
            Self::Video => EngineType::VIDEO,
            Self::Url => EngineType::URL,
            Self::Web => EngineType::WEB,
            Self::YouTube => EngineType::YOUTUBE,
        }
    }

    /// The engine the service reported for this type. The legacy names of the built-in GIF renderer are one choice
    /// (D-10): a stored `VexanaLive` is shown as `XavierLive`, the name the Portal writes.
    pub fn value(self, settings: &PortalSettings) -> EngineType {
        match self {
            Self::Gif => match settings.gif_engine {
                EngineType::VexanaLive => EngineType::XavierLive,
                engine => engine,
            },
            Self::Video => settings.video_engine,
            Self::Url => settings.url_engine,
            Self::Web => settings.web_engine,
            Self::YouTube => settings.youtube_engine,
        }
    }

    /// The change that sets this row to `engine`.
    pub fn change(self, engine: EngineType) -> SettingChange {
        match self {
            Self::Gif => SettingChange::GifEngine(engine),
            Self::Video => SettingChange::VideoEngine(engine),
            Self::Url => SettingChange::UrlEngine(engine),
            Self::Web => SettingChange::WebEngine(engine),
            Self::YouTube => SettingChange::YouTubeEngine(engine),
        }
    }

    /// The row a change belongs to, if it belongs to one.
    pub fn of(change: &SettingChange) -> Option<Self> {
        match change {
            SettingChange::GifEngine(_) => Some(Self::Gif),
            SettingChange::VideoEngine(_) => Some(Self::Video),
            SettingChange::UrlEngine(_) => Some(Self::Url),
            SettingChange::WebEngine(_) => Some(Self::Web),
            SettingChange::YouTubeEngine(_) => Some(Self::YouTube),
            _ => None,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Gif => 0,
            Self::Video => 1,
            Self::Url => 2,
            Self::Web => 3,
            Self::YouTube => 4,
        }
    }

    /// The legacy keys of the row's title and description.
    fn keys(self) -> (&'static str, &'static str) {
        match self {
            Self::Gif => (
                "Portal.WallpaperSettingPage.GifPlayer",
                "Portal.WallpaperSettingPage.GifPlayer.Description",
            ),
            Self::Video => (
                "Portal.WallpaperSettingPage.VideoPlayer",
                "Portal.WallpaperSettingPage.VideoPlayer.Description",
            ),
            Self::Url => (
                "Portal.WallpaperSettingPage.UrlPlayer",
                "Portal.WallpaperSettingPage.UrlPlayer.Description",
            ),
            Self::Web => (
                "Portal.WallpaperSettingPage.WebPlayer",
                "Portal.WallpaperSettingPage.WebPlayer.Description",
            ),
            Self::YouTube => (
                "Portal.WallpaperSettingPage.YouTubePlayer",
                "Portal.WallpaperSettingPage.YouTubePlayer.Description",
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
        // VexanaLive is the same renderer as XavierLive (D-10); the list shows it once, under the name the Portal writes.
        .filter(|engine| *engine != EngineType::VexanaLive)
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

/// The key of a wallpaper type's name in the engine alert.
pub fn type_label_key(kind: WallpaperType) -> &'static str {
    match kind {
        WallpaperType::Gif => "Portal.WallpaperSettingPage.Type.Gif",
        WallpaperType::Url => "Portal.WallpaperSettingPage.Type.Url",
        WallpaperType::Web => "Portal.WallpaperSettingPage.Type.Web",
        WallpaperType::Video => "Portal.WallpaperSettingPage.Type.Video",
        WallpaperType::YouTube => "Portal.WallpaperSettingPage.Type.YouTube",
        WallpaperType::Application => "Portal.WallpaperSettingPage.Type.Application",
    }
}

/// What the user can do about an engine that cannot be used (UI-SPEC Copywriting: Hints). An engine this build does not
/// include has no hint.
pub fn hint_text(engine: EngineType, state: &EngineState) -> Option<SharedString> {
    let hint = match state {
        EngineState::Missing { hint, .. } | EngineState::NotLoadable { hint, .. } => hint,
        EngineState::Ready { .. } | EngineState::Unsupported { .. } => return None,
    };
    let name = tr(engine_label_key(engine));
    Some(match hint {
        HintView::FilesMissing => tr_with(
            "Portal.WallpaperSettingPage.Engine.Hint.FilesMissing",
            &[("engine", &name)],
        ),
        HintView::VcRedistMissing { needed } => tr_with(
            "Portal.WallpaperSettingPage.Engine.Hint.VcMissing",
            &[("needed", needed)],
        ),
        HintView::RuntimeMissing => tr("Portal.WallpaperSettingPage.Engine.Hint.RuntimeMissing"),
        HintView::RuntimeOutdated { found, needed } => tr_with(
            "Portal.WallpaperSettingPage.Engine.Hint.RuntimeOutdated",
            &[("found", found), ("needed", needed)],
        ),
    })
}

/// The persistent warning under the row of a type whose chosen engine cannot be used (UI-SPEC E1, E3): which engine,
/// what plays instead (when something can) and the hint.
pub fn alert_text(
    kind: WallpaperType,
    engine: EngineType,
    fallback: Option<EngineType>,
    state: &EngineState,
) -> SharedString {
    let engine_name = tr(engine_label_key(engine));
    let type_name = tr(type_label_key(kind));
    let sentence = match fallback {
        Some(fallback) => tr_with(
            "Portal.WallpaperSettingPage.Engine.Fallback",
            &[
                ("engine", &engine_name),
                ("type", &type_name),
                ("fallback", &tr(engine_label_key(fallback))),
            ],
        ),
        None => tr_with(
            "Portal.WallpaperSettingPage.Engine.NoFallback",
            &[("engine", &engine_name), ("type", &type_name)],
        ),
    };
    match hint_text(engine, state) {
        Some(hint) => SharedString::from(format!("{sentence} {hint}")),
        None => sentence,
    }
}

/// What the status line of a web-engine row shows (UI-SPEC E2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusLine {
    /// The service has not reported yet.
    Checking,
    /// The engine can be used.
    Ready {
        /// "CEF is ready."
        text: String,
        /// The version, when the report carries one.
        version: Option<String>,
    },
    /// The engine cannot be used: the hint, or the plain sentence when there is none.
    Warning {
        /// What the user reads.
        text: String,
    },
}

/// The status line of `engine` from its state in the report (`None` until the first report).
pub fn status_line(engine: EngineType, state: Option<&EngineState>) -> StatusLine {
    let name = tr(engine_label_key(engine));
    match state {
        None => StatusLine::Checking,
        Some(EngineState::Ready { version }) => StatusLine::Ready {
            text: tr_with("Portal.WallpaperSettingPage.Engine.Ready", &[("engine", &name)]).to_string(),
            version: version.as_deref().map(|version| {
                tr_with("Portal.WallpaperSettingPage.Engine.Version", &[("version", version)]).to_string()
            }),
        },
        Some(state) => StatusLine::Warning {
            text: hint_text(engine, state)
                .unwrap_or_else(|| tr_with("Portal.FallbackToast.Title", &[("engine", &name)]))
                .to_string(),
        },
    }
}

/// The state of `engine` in the report, when the report lists it.
fn state_of(report: Option<&EngineReportView>, engine: EngineType) -> Option<&EngineState> {
    report?
        .engines
        .iter()
        .find(|known| known.engine == engine)
        .map(|known| &known.state)
}

/// What plays `kind` while its engine `chosen` cannot be used: the fallback the service runs now, else the first
/// installed engine of those the report offers for the type, in the service's order.
fn fallback_engine(report: &EngineReportView, kind: WallpaperType, chosen: EngineType) -> Option<EngineType> {
    if let Some(running) = report.fallbacks.iter().find(|fallback| fallback.kind == kind) {
        return Some(running.playing);
    }
    report
        .offered
        .iter()
        .find(|offered| offered.kind == kind)?
        .engines
        .iter()
        .copied()
        .find(|engine| *engine != chosen && matches!(state_of(Some(report), *engine), Some(EngineState::Ready { .. })))
}

/// The alert under the row of `row`, when the engine the settings chose for its type cannot be used. It has no snooze:
/// it stays until the engine is available (D-16). Nothing shows before the report arrives or for an engine it does not
/// list.
pub fn row_alert(row: EngineRow, settings: &PortalSettings, report: Option<&EngineReportView>) -> Option<SharedString> {
    let chosen = row.value(settings);
    let state = state_of(report, chosen)?;
    if matches!(state, EngineState::Ready { .. }) {
        return None;
    }
    let fallback = report.and_then(|report| fallback_engine(report, row.kind(), chosen));
    Some(alert_text(row.kind(), chosen, fallback, state))
}

/// What the Install or Repair button of an engine alert or fallback toast does (UI-SPEC E3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallAction {
    /// The part the install dialog installs.
    pub prerequisite: PrerequisiteView,
    /// `true` when the part is there but outdated, so the button says Repair instead of Install.
    pub repair: bool,
}

/// The install action of an engine in `state`, only when the service says it can install the missing part itself
/// (the VC++ Redistributable in Round A, the WebView2 Runtime in Round B). A missing file of the engine itself has no
/// button: it shows the reinstall hint alone (UI-SPEC E3 partial).
pub fn install_action(state: Option<&EngineState>) -> Option<InstallAction> {
    let EngineState::Missing {
        hint,
        installable: true,
    } = state?
    else {
        return None;
    };
    match hint {
        HintView::VcRedistMissing { .. } => Some(InstallAction {
            prerequisite: PrerequisiteView::VcRedist,
            repair: false,
        }),
        HintView::RuntimeMissing => Some(InstallAction {
            prerequisite: PrerequisiteView::WebView2Runtime,
            repair: false,
        }),
        HintView::RuntimeOutdated { .. } => Some(InstallAction {
            prerequisite: PrerequisiteView::WebView2Runtime,
            repair: true,
        }),
        HintView::FilesMissing => None,
    }
}

/// The install action of `engine` in the service's report.
pub fn engine_install_action(report: Option<&EngineReportView>, engine: EngineType) -> Option<InstallAction> {
    install_action(state_of(report, engine))
}

/// The install action under the row of `row`: the one of the engine the settings chose, when that engine cannot be used.
pub fn row_install_action(
    row: EngineRow,
    settings: &PortalSettings,
    report: Option<&EngineReportView>,
) -> Option<InstallAction> {
    engine_install_action(report, row.value(settings))
}

/// The label of an Install or Repair button.
pub fn install_label(action: InstallAction) -> SharedString {
    tr(if action.repair {
        "Portal.WallpaperSettingPage.Engine.Repair"
    } else {
        "Portal.WallpaperSettingPage.Engine.Install"
    })
}

/// Whether a change moves a select of this page, so the selects must show the broadcast value again (the switches are
/// drawn from the settings and need nothing).
fn needs_refresh(change: &SettingChange) -> bool {
    EngineRow::of(change).is_some() || matches!(change, SettingChange::InputType(_) | SettingChange::InputModuleType(_))
}

/// The setting a committed change belongs to when a renderer reads it only when it starts, with the committed value and the
/// value that was stored before; `None` for every other change. The restart prompt follows from these (E12).
pub fn restart_trigger(
    change: &SettingChange,
    previous: &PortalSettings,
    now: &PortalSettings,
) -> Option<(RestartSetting, bool, bool)> {
    match change {
        SettingChange::AllowInvalidCertificates(_) => Some((
            RestartSetting::Certificates,
            now.allow_invalid_certificates,
            previous.allow_invalid_certificates,
        )),
        SettingChange::HardwareAcceleration(_) => Some((
            RestartSetting::HardwareAcceleration,
            now.hardware_acceleration,
            previous.hardware_acceleration,
        )),
        _ => None,
    }
}

/// The Wallpaper settings page.
pub struct WallpaperPage {
    link: ServiceLink,
    settings: Option<PortalSettings>,
    engines: Option<EngineReportView>,
    /// The engine of each wallpaper that runs now, as the service last reported: which settings need the restart prompt.
    running: Vec<EngineType>,
    /// The Arguments editor in the CEF row's footer.
    arguments: Entity<ArgumentsEditor>,
    /// The Arguments editor in the WebView2 row's footer.
    web_arguments: Entity<ArgumentsEditor>,
    connected: bool,
    /// One select per [`EngineRow`], in the order of [`EngineRow::ALL`].
    selects: [ChoiceSelect<EngineType>; EngineRow::ALL.len()],
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
        let arguments = {
            let link = link.clone();
            cx.new(|_| ArgumentsEditor::new(link, EngineType::CefSharpLive))
        };
        let web_arguments = {
            let link = link.clone();
            cx.new(|_| ArgumentsEditor::new(link, EngineType::WebViewLive))
        };
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
            running: Vec::new(),
            arguments,
            web_arguments,
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
            for editor in self.editors() {
                editor.update(cx, |editor, cx| editor.set_connected(connected, cx));
            }
            cx.notify();
        }
    }

    /// The service reported what runs (Welcome and Runtime): the engines whose wallpapers a restart prompt may name.
    pub fn set_runtime(&mut self, runtime: &RuntimeState, cx: &mut Context<Self>) {
        self.running = runtime.running_engines.iter().map(|(_, engine)| *engine).collect();
        for editor in self.editors() {
            let running = self.running.clone();
            editor.update(cx, |editor, _| editor.set_running(running));
        }
    }

    /// The Arguments editors of the page: CEF's, then WebView2's.
    fn editors(&self) -> [&Entity<ArgumentsEditor>; 2] {
        [&self.arguments, &self.web_arguments]
    }

    /// The editor of the switches of `engine`, when the engine has one.
    fn editor_of(&self, engine: EngineType) -> Option<&Entity<ArgumentsEditor>> {
        match engine {
            EngineType::CefSharpLive => Some(&self.arguments),
            EngineType::WebViewLive => Some(&self.web_arguments),
            _ => None,
        }
    }

    /// The service sent the switches of an engine: the editor of that engine shows them.
    pub fn on_arguments(
        &mut self,
        engine: EngineType,
        lists: [Vec<SwitchView>; 3],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let [user, sucrose, defaults] = lists;
        if let Some(editor) = self.editor_of(engine) {
            editor.update(cx, |editor, cx| {
                editor.on_arguments(user, sucrose, defaults, window, cx)
            });
        }
    }

    /// The service stored a list of switches of `engine` (Changed): its draft shows it and the restart prompt may follow.
    pub fn on_arguments_saved(
        &mut self,
        engine: EngineType,
        list: &[SwitchView],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(editor) = self.editor_of(engine) {
            editor.update(cx, |editor, cx| editor.on_saved(list, window, cx));
        }
    }

    /// The service refused a change: a list of switches that was sent stays in the draft.
    pub fn on_rejected(&mut self, change: &SettingChange, cx: &mut Context<Self>) {
        if let Some((engine, _)) = saved_list(change)
            && let Some(editor) = self.editor_of(engine)
        {
            editor.update(cx, |editor, cx| editor.on_rejected(cx));
        }
    }

    /// The service reported every setting (Welcome): every select shows its engine.
    pub fn load(&mut self, settings: &PortalSettings, window: &mut Window, cx: &mut Context<Self>) {
        self.settings = Some(settings.clone());
        for editor in self.editors() {
            editor.update(cx, |editor, _| editor.request());
        }
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
        let previous = self.settings.replace(settings.clone());
        // The change is saved before the prompt opens, whatever the answer (UI-SPEC). A refused change leaves the
        // settings as they were, so it never prompts.
        if let Some(previous) = previous
            && let Some((setting, committed, stored)) = restart_trigger(change, &previous, settings)
            && should_prompt(setting, &committed, &stored, &self.running)
        {
            let engines = affected_engines(setting, &self.running);
            open_restart_prompt(setting, engines, self.link.clone(), window, cx);
        }
        if needs_refresh(change) {
            self.refresh(window, cx);
        }
        // Sucrose's own switches follow Hardware Acceleration (the read-only rows of the editor).
        if matches!(change, SettingChange::HardwareAcceleration(_)) {
            for editor in self.editors() {
                editor.update(cx, |editor, _| editor.request());
            }
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
    pub fn on_opened(&self, cx: &App) {
        if self.state() == ControlState::Enabled {
            self.link.send(PortalRequest::DetectEngines);
            for editor in self.editors() {
                editor.read(cx).request();
            }
        }
    }

    /// Whether the keyboard is in a select of this page (Escape then belongs to the select).
    pub fn field_focused(&self, window: &Window, cx: &App) -> bool {
        self.selects
            .iter()
            .any(|select| select.read(cx).focus_handle(cx).contains_focused(window, cx))
            || self.input_type.read(cx).focus_handle(cx).contains_focused(window, cx)
            || self.input_module.read(cx).focus_handle(cx).contains_focused(window, cx)
            || self.editors().iter().any(|editor| editor.read(cx).focused(window, cx))
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

    /// The row of `row`: title, description and the engine select; when the chosen engine cannot be used the footer
    /// opens by itself with the Warning alert.
    fn engine_row(&self, row: EngineRow, cx: &App) -> AnyElement {
        let state = self.state();
        let (title, description) = row.keys();
        let mut built = SettingRow::new(tr(title), Some(tr(description)))
            .control(select_box(&self.selects[row.index()], state.disabled()), SELECT_WIDTH)
            .state(state);
        let alert = self
            .settings
            .as_ref()
            .and_then(|settings| row_alert(row, settings, self.engines.as_ref()));
        if let Some(text) = alert {
            let action = self
                .settings
                .as_ref()
                .and_then(|settings| row_install_action(row, settings, self.engines.as_ref()));
            built = built.footer(warning_alert(
                row,
                text,
                action.map(|action| (action, self.link.clone())),
                cx,
            ));
        }
        built.build(cx)
    }

    /// The Allow invalid certificates row: one global switch for both web engines, off by default, with the info hint
    /// that turning it on matches the classic app and is less secure (D-12). The service saves it; the restart prompt
    /// follows the broadcast.
    fn certificates_row(&self, cx: &mut Context<Self>) -> AnyElement {
        let state = self.state();
        let on = self.settings.as_ref().is_some_and(|s| s.allow_invalid_certificates);
        let switch = Switch::new("web-certificates")
            .checked(on)
            .disabled(state.disabled())
            .accessibility_label(tr("Portal.WallpaperSettingPage.Certificates"))
            .on_click(cx.listener(|this, value: &bool, _, _| {
                this.send(SettingChange::AllowInvalidCertificates(*value));
            }));
        SettingRow::new(
            tr("Portal.WallpaperSettingPage.Certificates"),
            Some(tr("Portal.WallpaperSettingPage.Certificates.Description")),
        )
        .control(switch, 40.)
        .footer(info_alert(tr("Portal.WallpaperSettingPage.Certificates.Hint"), cx))
        .state(state)
        .build(cx)
    }

    /// The row of one web engine: its title, its status line and its Arguments editor (UI-SPEC E2, E4).
    fn web_engine_row(&self, engine: EngineType, editor: &Entity<ArgumentsEditor>, cx: &App) -> AnyElement {
        let line = status_line(engine, state_of(self.engines.as_ref(), engine));
        SettingRow::new(tr(engine_label_key(engine)), None)
            .status(status_view(&line, cx))
            .footer(editor.clone())
            .state(self.state())
            .build(cx)
    }

    /// The Web Engine Settings group: the certificate switch, then the WebView2 row and the CEF row, each with its
    /// status line and its own Arguments editor.
    fn web_engines_group(&self, cx: &mut Context<Self>) -> AnyElement {
        let certificates = self.certificates_row(cx);
        let webview2 = self.web_engine_row(EngineType::WebViewLive, &self.web_arguments, cx);
        let cef = self.web_engine_row(EngineType::CefSharpLive, &self.arguments, cx);
        super::group(
            tr("Portal.WallpaperSettingPage.WebEngines"),
            vec![certificates, webview2, cef],
        )
    }
}

/// The Warning alert of an engine row: the warning icon and the text on the warning fill (UI-SPEC E1).
/// With an install `action` it ends in the Install or Repair button, which opens the install dialog.
fn warning_alert(
    row: EngineRow,
    text: SharedString,
    action: Option<(InstallAction, ServiceLink)>,
    cx: &App,
) -> AnyElement {
    let colors = theme::warning_colors(cx);
    let id = match row {
        EngineRow::Gif => "engine-alert-gif",
        EngineRow::Video => "engine-alert-video",
        EngineRow::Url => "engine-alert-url",
        EngineRow::Web => "engine-alert-web",
        EngineRow::YouTube => "engine-alert-youtube",
    };
    h_flex()
        .id(id)
        .w_full()
        .gap_2()
        .items_start()
        .px_3()
        .py_2()
        .rounded(px(6.))
        .border_1()
        .border_color(colors.border)
        .bg(colors.background)
        .child(
            div().flex_none().child(
                Icon::new(IconName::TriangleAlert)
                    .with_size(px(16.))
                    .text_color(colors.icon),
            ),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_sm()
                .text_color(cx.theme().foreground)
                .child(text),
        )
        .when_some(action, |alert, (action, link)| {
            alert.child(
                div().flex_none().child(
                    Button::new(match row {
                        EngineRow::Gif => "engine-install-gif",
                        EngineRow::Video => "engine-install-video",
                        EngineRow::Url => "engine-install-url",
                        EngineRow::Web => "engine-install-web",
                        EngineRow::YouTube => "engine-install-youtube",
                    })
                    .small()
                    .outline()
                    .label(install_label(action))
                    .on_click(move |_, window, cx| {
                        open_install_dialog(action.prerequisite, link.clone(), window, cx);
                    }),
                ),
            )
        })
        .into_any_element()
}

/// An info alert: the info icon and the text on a quiet fill (UI-SPEC "alert style, info").
pub(super) fn info_alert(text: SharedString, cx: &App) -> AnyElement {
    let theme = cx.theme();
    h_flex()
        .id("info-alert")
        .w_full()
        .gap_2()
        .items_start()
        .px_3()
        .py_2()
        .rounded(px(6.))
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .child(
            div()
                .flex_none()
                .child(Icon::new(IconName::Info).with_size(px(16.)).text_color(theme.info)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_sm()
                .text_color(theme.foreground)
                .child(text),
        )
        .into_any_element()
}

/// The status line under a web-engine row title: icon and text, with the version muted (UI-SPEC E2).
fn status_view(line: &StatusLine, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let row = h_flex()
        .gap_2()
        .items_center()
        .text_xs()
        .text_color(theme.muted_foreground);
    match line {
        StatusLine::Checking => row
            .child(Spinner::new().with_size(px(16.)).color(theme.primary))
            .child(tr("Portal.WallpaperSettingPage.Engine.Checking"))
            .into_any_element(),
        StatusLine::Ready { text, version } => row
            .child(
                Icon::new(IconName::CircleCheck)
                    .with_size(px(16.))
                    .text_color(theme.success),
            )
            .child(div().text_color(theme.foreground).child(text.clone()))
            .children(version.clone())
            .into_any_element(),
        StatusLine::Warning { text } => row
            .child(
                Icon::new(IconName::TriangleAlert)
                    .with_size(px(16.))
                    .text_color(theme::warning_colors(cx).icon),
            )
            .child(div().flex_1().min_w_0().child(text.clone()))
            .into_any_element(),
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
                    vec![
                        self.engine_row(EngineRow::Gif, cx),
                        self.engine_row(EngineRow::Video, cx),
                    ],
                ),
                super::group(
                    tr("Portal.Area.Engines"),
                    vec![
                        self.engine_row(EngineRow::Url, cx),
                        self.engine_row(EngineRow::Web, cx),
                        self.engine_row(EngineRow::YouTube, cx),
                    ],
                ),
                self.web_engines_group(cx),
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

    use super::{
        EngineRow, Flag, RestartSetting, StatusLine, alert_text, engine_choices, hint_text, input_module_choices,
        input_type_choices, needs_refresh, restart_trigger, row_alert, row_install_action, should_prompt, status_line,
    };
    use sucrose_ipc::portal::{FallbackView, HintView};

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
            allow_invalid_certificates: false,
            developer_mode: false,
            developer_port: 0,
        }
    }

    #[test]
    fn restart_trigger_names_the_setting_and_both_values_of_a_committed_switch() {
        let before = settings();
        let mut after = settings();
        after.allow_invalid_certificates = true;
        after.hardware_acceleration = false;

        assert_eq!(
            restart_trigger(&SettingChange::AllowInvalidCertificates(true), &before, &after),
            Some((RestartSetting::Certificates, true, false))
        );
        assert_eq!(
            restart_trigger(&SettingChange::HardwareAcceleration(false), &before, &after),
            Some((RestartSetting::HardwareAcceleration, false, true))
        );
        assert_eq!(
            restart_trigger(&SettingChange::WallpaperLoop(false), &before, &after),
            None
        );
        assert_eq!(
            restart_trigger(&SettingChange::WebEngine(EngineType::CefSharpLive), &before, &after),
            None
        );
    }

    #[test]
    fn restart_trigger_of_a_refused_change_carries_equal_values_so_nothing_prompts() {
        let same = settings();
        let (setting, committed, stored) =
            restart_trigger(&SettingChange::AllowInvalidCertificates(true), &same, &same).expect("a trigger");

        assert!(!should_prompt(
            setting,
            &committed,
            &stored,
            &[EngineType::CefSharpLive]
        ));
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

    /// A report of Round B: the built-in GIF renderer, mpv, WebView2 and CEF are in this build and installed; libVLC is not.
    fn round_b() -> EngineReportView {
        let both_web = vec![EngineType::WebViewLive, EngineType::CefSharpLive];
        EngineReportView {
            engines: vec![
                state(EngineType::XavierLive, ready()),
                state(EngineType::MpvPlayerLive, ready()),
                state(EngineType::WebViewLive, ready()),
                state(EngineType::CefSharpLive, ready()),
                state(
                    EngineType::VlcPlayerLive,
                    EngineState::Unsupported { architecture: false },
                ),
            ],
            supported: vec![
                WallpaperType::Gif,
                WallpaperType::Video,
                WallpaperType::Url,
                WallpaperType::Web,
                WallpaperType::YouTube,
            ],
            playable: vec![
                WallpaperType::Gif,
                WallpaperType::Video,
                WallpaperType::Url,
                WallpaperType::Web,
                WallpaperType::YouTube,
            ],
            offered: vec![
                OfferedView {
                    kind: WallpaperType::Gif,
                    engines: vec![
                        EngineType::XavierLive,
                        EngineType::MpvPlayerLive,
                        EngineType::WebViewLive,
                        EngineType::CefSharpLive,
                    ],
                },
                OfferedView {
                    kind: WallpaperType::Video,
                    engines: vec![
                        EngineType::MpvPlayerLive,
                        EngineType::WebViewLive,
                        EngineType::CefSharpLive,
                    ],
                },
                OfferedView {
                    kind: WallpaperType::Url,
                    engines: both_web.clone(),
                },
                OfferedView {
                    kind: WallpaperType::Web,
                    engines: both_web.clone(),
                },
                OfferedView {
                    kind: WallpaperType::YouTube,
                    engines: both_web,
                },
            ],
            fallbacks: Vec::new(),
        }
    }

    #[test]
    fn wallpaper_the_rows_follow_the_legacy_page_order_and_each_one_owns_its_type() {
        assert_eq!(
            EngineRow::ALL.as_slice(),
            [
                EngineRow::Gif,
                EngineRow::Video,
                EngineRow::Url,
                EngineRow::Web,
                EngineRow::YouTube
            ]
            .as_slice()
        );
        let kinds: Vec<WallpaperType> = EngineRow::ALL.into_iter().map(EngineRow::kind).collect();
        assert_eq!(
            kinds,
            vec![
                WallpaperType::Gif,
                WallpaperType::Video,
                WallpaperType::Url,
                WallpaperType::Web,
                WallpaperType::YouTube
            ]
        );
        let mut indexes: Vec<usize> = EngineRow::ALL.into_iter().map(EngineRow::index).collect();
        indexes.sort_unstable();
        assert_eq!(
            indexes,
            vec![0, 1, 2, 3, 4],
            "each row has its own select, in the order of ALL"
        );
        for (position, row) in EngineRow::ALL.into_iter().enumerate() {
            assert_eq!(row.index(), position);
        }
    }

    #[test]
    fn wallpaper_the_new_rows_read_and_send_their_own_engine() {
        let mut settings = settings();
        settings.gif_engine = EngineType::XavierLive;
        settings.url_engine = EngineType::WebViewLive;
        settings.youtube_engine = EngineType::CefSharpLive;
        assert_eq!(EngineRow::Gif.value(&settings), EngineType::XavierLive);
        assert_eq!(EngineRow::Url.value(&settings), EngineType::WebViewLive);
        assert_eq!(EngineRow::YouTube.value(&settings), EngineType::CefSharpLive);
        assert_eq!(
            EngineRow::Gif.change(EngineType::XavierLive),
            SettingChange::GifEngine(EngineType::XavierLive)
        );
        assert_eq!(
            EngineRow::Url.change(EngineType::WebViewLive),
            SettingChange::UrlEngine(EngineType::WebViewLive)
        );
        assert_eq!(
            EngineRow::YouTube.change(EngineType::CefSharpLive),
            SettingChange::YouTubeEngine(EngineType::CefSharpLive)
        );
        assert_eq!(EngineRow::Gif.allowed(), EngineType::GIF);
        assert_eq!(EngineRow::Url.allowed(), EngineType::URL);
        assert_eq!(EngineRow::YouTube.allowed(), EngineType::YOUTUBE);
        assert_eq!(
            EngineRow::Gif.keys().0,
            "Portal.WallpaperSettingPage.GifPlayer",
            "the legacy title keys"
        );
        assert_eq!(EngineRow::Url.keys().0, "Portal.WallpaperSettingPage.UrlPlayer");
        assert_eq!(EngineRow::YouTube.keys().0, "Portal.WallpaperSettingPage.YouTubePlayer");
    }

    #[test]
    fn wallpaper_the_gif_player_lists_built_in_webview2_cef_and_mpv_without_a_suffix() {
        let choices = engine_choices(EngineRow::Gif, Some(&round_b()), EngineType::MpvPlayerLive);

        assert_eq!(
            shown(&choices),
            [
                (EngineType::XavierLive, "Built-in".to_owned(), false),
                (EngineType::WebViewLive, "WebView2".to_owned(), false),
                (EngineType::CefSharpLive, "CEF".to_owned(), false),
                (EngineType::MpvPlayerLive, "mpv".to_owned(), false),
            ],
            "the legacy order of the type, libVLC is not in this build"
        );
    }

    #[test]
    fn wallpaper_the_gif_player_adds_a_stored_libvlc_with_the_not_installed_suffix() {
        let choices = engine_choices(EngineRow::Gif, Some(&round_b()), EngineType::VlcPlayerLive);

        assert_eq!(
            shown(&choices).last(),
            Some(&(EngineType::VlcPlayerLive, "libVLC".to_owned(), true))
        );
        assert_eq!(choices.len(), 5);
        assert_eq!(choices[4].suffix.as_deref(), Some("Not installed"));
    }

    #[test]
    fn wallpaper_the_gif_player_shows_the_two_legacy_built_in_names_as_one_choice() {
        // Vexana and Xavier are the same renderer (D-10); the Portal writes Xavier, a stored Vexana shows as it.
        let mut settings = settings();
        settings.gif_engine = EngineType::VexanaLive;
        assert_eq!(EngineRow::Gif.value(&settings), EngineType::XavierLive);

        let choices = engine_choices(EngineRow::Gif, Some(&round_b()), EngineRow::Gif.value(&settings));
        assert_eq!(
            shown(&choices)[0],
            (EngineType::XavierLive, "Built-in".to_owned(), false)
        );
        assert_eq!(
            choices
                .iter()
                .filter(|choice| choice.title.as_ref() == "Built-in")
                .count(),
            1
        );
        assert_eq!(row_alert(EngineRow::Gif, &settings, Some(&round_b())), None);
    }

    #[test]
    fn wallpaper_the_video_url_and_youtube_rows_list_what_the_report_offers() {
        let report = round_b();
        assert_eq!(
            shown(&engine_choices(
                EngineRow::Video,
                Some(&report),
                EngineType::MpvPlayerLive
            )),
            [
                (EngineType::WebViewLive, "WebView2".to_owned(), false),
                (EngineType::CefSharpLive, "CEF".to_owned(), false),
                (EngineType::MpvPlayerLive, "mpv".to_owned(), false),
            ]
        );
        for row in [EngineRow::Url, EngineRow::Web, EngineRow::YouTube] {
            assert_eq!(
                shown(&engine_choices(row, Some(&report), EngineType::CefSharpLive)),
                [
                    (EngineType::WebViewLive, "WebView2".to_owned(), false),
                    (EngineType::CefSharpLive, "CEF".to_owned(), false),
                ],
                "{row:?}"
            );
        }
    }

    #[test]
    fn wallpaper_alert_a_gif_set_to_libvlc_names_the_built_in_fallback_the_service_runs() {
        let mut settings = settings();
        settings.gif_engine = EngineType::VlcPlayerLive;
        let mut report = round_b();
        report.fallbacks = vec![FallbackView {
            kind: WallpaperType::Gif,
            configured: EngineType::VlcPlayerLive,
            playing: EngineType::XavierLive,
            notify: true,
        }];

        let alert = row_alert(EngineRow::Gif, &settings, Some(&report));

        assert_eq!(
            alert.as_deref(),
            Some("libVLC isn't available, so GIF wallpapers play with Built-in for now.")
        );
        assert_eq!(
            row_install_action(EngineRow::Gif, &settings, Some(&report)),
            None,
            "a build without libVLC has nothing to install"
        );
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

    fn vc_missing() -> EngineState {
        EngineState::Missing {
            hint: HintView::VcRedistMissing {
                needed: "14.40.33816.0".to_owned(),
            },
            installable: true,
        }
    }

    #[test]
    fn wallpaper_alert_a_type_set_to_an_engine_this_build_lacks_names_the_fallback_and_has_no_hint() {
        let mut settings = settings();
        settings.web_engine = EngineType::WebViewLive;

        let alert = row_alert(EngineRow::Web, &settings, Some(&round_a()));

        assert_eq!(
            alert.as_deref(),
            Some("WebView2 isn't available, so Web wallpapers play with CEF for now.")
        );
    }

    #[test]
    fn wallpaper_alert_the_running_fallback_wins_over_the_guess() {
        let mut settings = settings();
        settings.web_engine = EngineType::WebViewLive;
        let mut report = round_a();
        report.engines.push(state(EngineType::VlcPlayerLive, ready()));
        report.offered[1].engines = vec![EngineType::VlcPlayerLive, EngineType::CefSharpLive];
        report.fallbacks = vec![FallbackView {
            kind: WallpaperType::Web,
            configured: EngineType::WebViewLive,
            playing: EngineType::CefSharpLive,
            notify: true,
        }];

        let alert = row_alert(EngineRow::Web, &settings, Some(&report));

        assert_eq!(
            alert.as_deref(),
            Some("WebView2 isn't available, so Web wallpapers play with CEF for now.")
        );
    }

    #[test]
    fn wallpaper_alert_a_missing_prerequisite_adds_its_hint_and_nothing_plays_instead() {
        let mut report = round_a();
        report.engines[1].state = vc_missing();
        let settings = settings();

        let alert = row_alert(EngineRow::Web, &settings, Some(&report));

        assert_eq!(
            alert.as_deref(),
            Some(
                "CEF isn't available, and no installed engine can play Web wallpapers for now. CEF needs the Microsoft Visual C++ Redistributable 14.40.33816.0 or newer. Install it to use CEF."
            )
        );
    }

    #[test]
    fn wallpaper_alert_the_texts_of_the_other_hints() {
        let files = EngineState::NotLoadable {
            reason: "x".to_owned(),
            hint: HintView::FilesMissing,
        };
        assert_eq!(
            hint_text(EngineType::MpvPlayerLive, &files).as_deref(),
            Some("The mpv files are missing or can't be loaded. Reinstall Sucrose to repair them.")
        );
        let outdated = EngineState::Missing {
            hint: HintView::RuntimeOutdated {
                found: "100.0".to_owned(),
                needed: "120.0".to_owned(),
            },
            installable: true,
        };
        assert_eq!(
            hint_text(EngineType::WebViewLive, &outdated).as_deref(),
            Some("The WebView2 Runtime 100.0 is older than the required 120.0. Update it to use WebView2.")
        );
        assert_eq!(hint_text(EngineType::CefSharpLive, &ready()), None);
        assert_eq!(
            hint_text(
                EngineType::WebViewLive,
                &EngineState::Unsupported { architecture: false }
            ),
            None
        );
        assert_eq!(
            alert_text(
                WallpaperType::Video,
                EngineType::MpvPlayerLive,
                Some(EngineType::VlcPlayerLive),
                &files
            ),
            "mpv isn't available, so Video wallpapers play with libVLC for now. The mpv files are missing or can't be loaded. Reinstall Sucrose to repair them."
        );
    }

    #[test]
    fn wallpaper_install_a_missing_vc_redist_the_service_can_install_gets_an_install_button() {
        let action = super::install_action(Some(&vc_missing()));

        assert_eq!(
            action,
            Some(super::InstallAction {
                prerequisite: sucrose_ipc::portal::PrerequisiteView::VcRedist,
                repair: false
            })
        );
        assert_eq!(super::install_label(action.expect("an action")).as_ref(), "Install");
    }

    #[test]
    fn wallpaper_install_an_outdated_runtime_says_repair_and_missing_files_have_no_button() {
        let outdated = EngineState::Missing {
            hint: HintView::RuntimeOutdated {
                found: "1".to_owned(),
                needed: "2".to_owned(),
            },
            installable: true,
        };
        let action = super::install_action(Some(&outdated)).expect("an action");

        assert!(action.repair);
        assert_eq!(super::install_label(action).as_ref(), "Repair");
        assert_eq!(
            super::install_action(Some(&missing())),
            None,
            "FilesMissing: hint alone"
        );
        let files_but_installable = EngineState::Missing {
            hint: HintView::FilesMissing,
            installable: true,
        };
        assert_eq!(super::install_action(Some(&files_but_installable)), None);
    }

    #[test]
    fn wallpaper_install_a_missing_or_outdated_webview2_runtime_gets_install_or_repair_and_a_ready_one_nothing() {
        use sucrose_ipc::portal::PrerequisiteView;

        let missing_runtime = EngineState::Missing {
            hint: HintView::RuntimeMissing,
            installable: true,
        };
        let action = super::install_action(Some(&missing_runtime));
        assert_eq!(
            action,
            Some(super::InstallAction {
                prerequisite: PrerequisiteView::WebView2Runtime,
                repair: false
            })
        );
        assert_eq!(super::install_label(action.expect("an action")).as_ref(), "Install");

        let outdated = EngineState::Missing {
            hint: HintView::RuntimeOutdated {
                found: "100.0".to_owned(),
                needed: "131.0.2903.70".to_owned(),
            },
            installable: true,
        };
        let action = super::install_action(Some(&outdated)).expect("an action");
        assert_eq!(action.prerequisite, PrerequisiteView::WebView2Runtime);
        assert_eq!(super::install_label(action).as_ref(), "Repair");

        assert_eq!(
            super::install_action(Some(&ready())),
            None,
            "a ready runtime offers nothing"
        );
        let without_download = EngineState::Missing {
            hint: HintView::RuntimeMissing,
            installable: false,
        };
        assert_eq!(super::install_action(Some(&without_download)), None);
    }

    #[test]
    fn wallpaper_install_nothing_is_offered_for_a_ready_engine_an_unsupported_one_or_no_report() {
        assert_eq!(super::install_action(Some(&ready())), None);
        assert_eq!(
            super::install_action(Some(&EngineState::Unsupported { architecture: false })),
            None
        );
        assert_eq!(super::install_action(None), None);
        let not_installable = EngineState::Missing {
            hint: HintView::VcRedistMissing { needed: String::new() },
            installable: false,
        };
        assert_eq!(super::install_action(Some(&not_installable)), None);
    }

    #[test]
    fn wallpaper_install_the_row_follows_the_engine_the_settings_chose() {
        let mut report = round_a();
        report.engines[1] = state(EngineType::CefSharpLive, vc_missing());

        assert!(super::engine_install_action(Some(&report), EngineType::CefSharpLive).is_some());
        assert_eq!(
            super::engine_install_action(Some(&report), EngineType::MpvPlayerLive),
            None
        );
    }

    #[test]
    fn wallpaper_alert_is_gone_when_the_engine_is_ready_and_absent_before_the_report() {
        let settings = settings();

        assert_eq!(row_alert(EngineRow::Web, &settings, Some(&round_a())), None);
        assert_eq!(row_alert(EngineRow::Web, &settings, None), None);
        let mut other = settings.clone();
        other.web_engine = EngineType::VlcPlayerLive;
        assert_eq!(
            row_alert(EngineRow::Web, &other, Some(&round_a())),
            None,
            "an engine the report does not list is not claimed missing"
        );
    }

    #[test]
    fn wallpaper_webview2_status_line_names_the_runtime_version_or_the_hint() {
        let web = EngineType::WebViewLive;

        assert_eq!(status_line(web, None), StatusLine::Checking);
        assert_eq!(
            status_line(
                web,
                Some(&EngineState::Ready {
                    version: Some("158.0.4370.33".to_owned())
                })
            ),
            StatusLine::Ready {
                text: "WebView2 is ready.".to_owned(),
                version: Some("Version 158.0.4370.33".to_owned())
            }
        );
        assert_eq!(
            status_line(
                web,
                Some(&EngineState::Missing {
                    hint: HintView::RuntimeMissing,
                    installable: false
                })
            ),
            StatusLine::Warning {
                text: "Install the WebView2 Runtime to use WebView2.".to_owned()
            }
        );
        let StatusLine::Warning { text } = status_line(
            web,
            Some(&EngineState::Missing {
                hint: HintView::RuntimeOutdated {
                    found: "100.0".to_owned(),
                    needed: "120.0".to_owned(),
                },
                installable: true,
            }),
        ) else {
            panic!("an outdated runtime is a warning");
        };
        assert!(text.contains("100.0") && text.contains("120.0"), "{text}");
    }

    #[test]
    fn wallpaper_status_line_is_checking_until_the_report_then_ready_with_the_version_or_the_hint() {
        let cef = EngineType::CefSharpLive;

        assert_eq!(status_line(cef, None), StatusLine::Checking);
        assert_eq!(
            status_line(
                cef,
                Some(&EngineState::Ready {
                    version: Some("156.0".to_owned())
                })
            ),
            StatusLine::Ready {
                text: "CEF is ready.".to_owned(),
                version: Some("Version 156.0".to_owned())
            }
        );
        assert_eq!(
            status_line(cef, Some(&ready())),
            StatusLine::Ready {
                text: "CEF is ready.".to_owned(),
                version: None
            }
        );
        assert_eq!(
            status_line(cef, Some(&vc_missing())),
            StatusLine::Warning {
                text:
                    "CEF needs the Microsoft Visual C++ Redistributable 14.40.33816.0 or newer. Install it to use CEF."
                        .to_owned()
            }
        );
        assert_eq!(
            status_line(cef, Some(&EngineState::Unsupported { architecture: true })),
            StatusLine::Warning {
                text: "CEF isn't available.".to_owned()
            }
        );
    }
}
