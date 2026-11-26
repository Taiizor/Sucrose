//! The Wallpaper page (UI-SPEC "Settings pages: Wallpaper"): the Extensions group with the Video Player and the Engines
//! group with the Web Engine. Later plans add the Wallpaper group, the other engine rows and the Web Engine Settings.
//! Every select sends a SettingChange and shows only what the service broadcast; the choices come from the engine report
//! (the Portal never decides which engines exist), plus the stored engine when the report does not offer it.

use gpui_kit::*;
use sucrose_core::{settings::EngineType, theme::WallpaperType};
use sucrose_ipc::portal::{EngineReportView, EngineState, PortalRequest, PortalSettings, SettingChange};

use super::row::{
    Choice, ChoiceSelect, ControlState, SELECT_WIDTH, SettingRow, choice_select, control_state, on_choice, select_box,
    show_choice,
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

/// The Wallpaper settings page.
pub struct WallpaperPage {
    link: ServiceLink,
    settings: Option<PortalSettings>,
    engines: Option<EngineReportView>,
    connected: bool,
    /// One select per [`EngineRow`], in the order of [`EngineRow::ALL`].
    selects: [ChoiceSelect<EngineType>; 2],
    _subscriptions: Vec<Subscription>,
}

impl WallpaperPage {
    /// A page that shows skeletons until the service reports the settings.
    pub fn new(link: ServiceLink, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let selects = EngineRow::ALL.map(|_| choice_select(Vec::new(), None, window, cx));
        let subscriptions = EngineRow::ALL
            .into_iter()
            .map(|row| {
                on_choice(&selects[row.index()], window, cx, move |this, engine, _, _| {
                    this.send(row.change(engine));
                })
            })
            .collect();
        Self {
            link,
            settings: None,
            engines: None,
            connected: false,
            selects,
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
        if EngineRow::of(change).is_some() {
            self.refresh(window, cx);
        }
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

    use super::{EngineRow, engine_choices};

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
