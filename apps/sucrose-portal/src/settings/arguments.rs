//! The Arguments editor of a web engine row, CEF or WebView2 (UI-SPEC "Arguments editor", E4, D-17, WAPI-10): the command
//! line switches Chromium starts with. The service owns the rules and the storage format: it sends the stored switches, the read-only ones Sucrose
//! adds from the settings and the legacy defaults, and it validates a saved list (the Portal links no engine crate). The
//! editor edits a draft that nothing stores until Save Arguments; the draft survives page switches until the Portal closes.
//! The pure [`ArgumentsDraft`] is tested apart from the view.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement as _,
    skeleton::Skeleton,
    tooltip::Tooltip,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use sucrose_core::settings::EngineType;
use sucrose_ipc::portal::{MAX_SWITCHES, PortalRequest, SettingChange, SwitchView};

use super::wallpaper::engine_label_key;
use crate::{
    dialogs::restart::{RestartSetting, affected_engines, open_restart_prompt, should_prompt},
    i18n::{tr, tr_with},
    ipc::ServiceLink,
};

/// Height of one row (UI-SPEC: argument row height 40).
const ROW_HEIGHT: f32 = 40.0;
/// The list body scrolls past this height (UI-SPEC).
const LIST_MAX_HEIGHT: f32 = 240.0;
/// Width of the value input (UI-SPEC).
const VALUE_WIDTH: f32 = 200.0;
/// A value longer than this gets its full text as a tooltip (it truncates in the box).
const TOOLTIP_FROM_CHARS: usize = 24;

/// Why a row cannot be saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowError {
    /// The name is empty while the row has a value, or has a character other than letters, digits and dashes.
    Invalid,
    /// An earlier row has the same name.
    Duplicate,
}

impl RowError {
    /// The text under the row.
    pub fn text(self) -> SharedString {
        match self {
            Self::Invalid => tr("Portal.WallpaperSettingPage.Arguments.Invalid"),
            Self::Duplicate => tr("Portal.WallpaperSettingPage.Arguments.Duplicate"),
        }
    }
}

/// One row of the draft, as typed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DraftRow {
    /// The name box.
    pub name: String,
    /// The value box.
    pub value: String,
}

impl DraftRow {
    /// A row for a switch of the service.
    fn of(switch: &SwitchView) -> Self {
        Self {
            name: switch.name.clone(),
            value: switch.value.clone().unwrap_or_default(),
        }
    }
}

/// A name as it is stored: trimmed, without leading dashes (the stored CEF form is the dictionary key).
pub fn normalize_name(text: &str) -> String {
    text.trim().trim_start_matches('-').to_owned()
}

/// Whether a normalized name is letters, digits and dashes. The service checks the same again when it saves (D-17).
fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// The list in a fixed order, so two lists are equal whatever order the user and the service keep them in.
fn sorted(list: &[SwitchView]) -> Vec<(String, Option<String>)> {
    let mut sorted: Vec<(String, Option<String>)> = list
        .iter()
        .map(|switch| (switch.name.clone(), switch.value.clone()))
        .collect();
    sorted.sort();
    sorted
}

/// What the user is editing: the rows, the list the service stored and the defaults Reset puts back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentsDraft {
    rows: Vec<DraftRow>,
    stored: Vec<SwitchView>,
    defaults: Vec<SwitchView>,
}

impl ArgumentsDraft {
    /// A draft that shows `stored` and resets to `defaults`.
    pub fn new(stored: Vec<SwitchView>, defaults: Vec<SwitchView>) -> Self {
        Self {
            rows: stored.iter().map(DraftRow::of).collect(),
            stored,
            defaults,
        }
    }

    /// The rows, in the order they are drawn.
    pub fn rows(&self) -> &[DraftRow] {
        &self.rows
    }

    /// The list the service stored.
    pub fn stored(&self) -> &[SwitchView] {
        &self.stored
    }

    /// Adds an empty row at the end.
    pub fn add_row(&mut self) {
        self.rows.push(DraftRow::default());
    }

    /// Removes row `index`; a row that is not there changes nothing.
    pub fn remove_row(&mut self, index: usize) {
        if index < self.rows.len() {
            self.rows.remove(index);
        }
    }

    /// Sets the name text of row `index`.
    pub fn set_name(&mut self, index: usize, text: &str) {
        if let Some(row) = self.rows.get_mut(index) {
            row.name = text.to_owned();
        }
    }

    /// Sets the value text of row `index`.
    pub fn set_value(&mut self, index: usize, text: &str) {
        if let Some(row) = self.rows.get_mut(index) {
            row.value = text.to_owned();
        }
    }

    /// Puts the legacy defaults into the rows. Nothing is stored until Save.
    pub fn reset(&mut self) {
        self.rows = self.defaults.iter().map(DraftRow::of).collect();
    }

    /// The service stored `list`: it is the stored list now and the rows show it.
    pub fn mark_saved(&mut self, list: Vec<SwitchView>) {
        self.rows = list.iter().map(DraftRow::of).collect();
        self.stored = list;
    }

    /// The service sent its lists again: the stored list and the defaults follow, the rows only when the user has not
    /// edited them (the draft survives page switches, UI-SPEC).
    pub fn refresh(&mut self, stored: Vec<SwitchView>, defaults: Vec<SwitchView>) {
        let untouched = !self.is_dirty();
        self.stored = stored;
        self.defaults = defaults;
        if untouched {
            self.rows = self.stored.iter().map(DraftRow::of).collect();
        }
    }

    /// A row with nothing typed in it: it is left out of the list and is no error.
    fn blank(row: &DraftRow) -> bool {
        normalize_name(&row.name).is_empty() && row.value.trim().is_empty()
    }

    /// What is wrong with row `index`, if anything.
    pub fn error(&self, index: usize) -> Option<RowError> {
        let row = self.rows.get(index)?;
        if Self::blank(row) {
            return None;
        }
        let name = normalize_name(&row.name);
        if !valid_name(&name) {
            return Some(RowError::Invalid);
        }
        let repeats = self.rows[..index]
            .iter()
            .any(|earlier| normalize_name(&earlier.name) == name);
        repeats.then_some(RowError::Duplicate)
    }

    /// The list the rows make: names normalized, values trimmed with an empty one meaning none, blank rows left out.
    pub fn to_switches(&self) -> Vec<SwitchView> {
        self.rows
            .iter()
            .filter(|row| !Self::blank(row))
            .map(|row| {
                let value = row.value.trim();
                SwitchView {
                    name: normalize_name(&row.name),
                    value: (!value.is_empty()).then(|| value.to_owned()),
                }
            })
            .collect()
    }

    /// Whether the rows differ from what the service stored.
    pub fn is_dirty(&self) -> bool {
        sorted(&self.to_switches()) != sorted(&self.stored)
    }

    /// Whether Save Arguments is enabled: the draft differs, every row is valid and the list fits.
    pub fn can_save(&self) -> bool {
        self.is_dirty()
            && (0..self.rows.len()).all(|index| self.error(index).is_none())
            && self.to_switches().len() <= MAX_SWITCHES
    }
}

/// The two boxes of a row.
struct Fields {
    name: Entity<InputState>,
    value: Entity<InputState>,
}

/// What the service sent that is not part of the draft.
struct Loaded {
    draft: ArgumentsDraft,
    /// The switches Sucrose adds, read-only.
    sucrose: Vec<Fields>,
}

/// The change that saves `list` as the switches of `engine`: WebView2 has its own list, every other engine (CEF) the
/// dictionary.
pub fn save_change(engine: EngineType, list: Vec<SwitchView>) -> SettingChange {
    match engine {
        EngineType::WebViewLive => SettingChange::WebArguments(list),
        _ => SettingChange::CefArguments(list),
    }
}

/// The engine and the list of a change that stores switches; `None` for every other change.
pub fn saved_list(change: &SettingChange) -> Option<(EngineType, &[SwitchView])> {
    match change {
        SettingChange::CefArguments(list) => Some((EngineType::CefSharpLive, list)),
        SettingChange::WebArguments(list) => Some((EngineType::WebViewLive, list)),
        _ => None,
    }
}

/// The editor of one web engine's switches.
pub struct ArgumentsEditor {
    link: ServiceLink,
    engine: EngineType,
    /// `None` until the stored arguments arrive (Loading).
    loaded: Option<Loaded>,
    fields: Vec<Fields>,
    connected: bool,
    /// Save was sent and the service has not answered.
    saving: bool,
    /// The engine of each wallpaper that runs, for the restart prompt.
    running: Vec<EngineType>,
    subscriptions: Vec<Subscription>,
}

impl ArgumentsEditor {
    /// An editor that is Loading until [`ArgumentsEditor::on_arguments`].
    pub fn new(link: ServiceLink, engine: EngineType) -> Self {
        Self {
            link,
            engine,
            loaded: None,
            fields: Vec::new(),
            connected: false,
            saving: false,
            running: Vec::new(),
            subscriptions: Vec::new(),
        }
    }

    /// Whether the service is connected.
    pub fn set_connected(&mut self, connected: bool, cx: &mut Context<Self>) {
        if self.connected != connected {
            self.connected = connected;
            cx.notify();
        }
    }

    /// The engines whose wallpapers run now.
    pub fn set_running(&mut self, running: Vec<EngineType>) {
        self.running = running;
    }

    /// Asks the service for the switches.
    pub fn request(&self) {
        if self.connected {
            self.link.send(PortalRequest::GetArguments { engine: self.engine });
        }
    }

    /// Whether the keyboard is in one of the boxes (Escape then belongs to the box).
    pub fn focused(&self, window: &Window, cx: &App) -> bool {
        self.fields.iter().any(|fields| {
            fields.name.read(cx).focus_handle(cx).contains_focused(window, cx)
                || fields.value.read(cx).focus_handle(cx).contains_focused(window, cx)
        })
    }

    /// A box that shows `text`.
    fn input(
        text: &str,
        placeholder: Option<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        let text = SharedString::from(text.to_owned());
        cx.new(|cx| {
            let state = InputState::new(window, cx).default_value(text);
            match placeholder {
                Some(placeholder) => state.placeholder(placeholder),
                None => state,
            }
        })
    }

    /// The boxes of row `index` of the draft, typing into them edits the draft.
    fn push_fields(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(row) = self
            .loaded
            .as_ref()
            .and_then(|loaded| loaded.draft.rows().get(index))
            .cloned()
        else {
            return;
        };
        let name = Self::input(
            &row.name,
            Some(tr("Portal.WallpaperSettingPage.Arguments.NamePlaceholder")),
            window,
            cx,
        );
        let value = Self::input(
            &row.value,
            Some(tr("Portal.WallpaperSettingPage.Arguments.ValuePlaceholder")),
            window,
            cx,
        );
        self.subscriptions.push(
            cx.subscribe_in(&name, window, move |this, input, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    let text = input.read(cx).value();
                    if let Some(loaded) = this.loaded.as_mut() {
                        loaded.draft.set_name(index, &text);
                    }
                    cx.notify();
                }
            }),
        );
        self.subscriptions.push(
            cx.subscribe_in(&value, window, move |this, input, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    let text = input.read(cx).value();
                    if let Some(loaded) = this.loaded.as_mut() {
                        loaded.draft.set_value(index, &text);
                    }
                    cx.notify();
                }
            }),
        );
        self.fields.push(Fields { name, value });
    }

    /// New boxes for every row: after a row left, a reset, or lists from the service.
    fn rebuild(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.fields.clear();
        self.subscriptions.clear();
        let count = self.loaded.as_ref().map_or(0, |loaded| loaded.draft.rows().len());
        for index in 0..count {
            self.push_fields(index, window, cx);
        }
        cx.notify();
    }

    /// The service sent the switches of the engine.
    pub fn on_arguments(
        &mut self,
        user: Vec<SwitchView>,
        sucrose: Vec<SwitchView>,
        defaults: Vec<SwitchView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let read_only = sucrose
            .iter()
            .map(|switch| Fields {
                name: Self::input(&switch.name, None, window, cx),
                value: Self::input(switch.value.as_deref().unwrap_or_default(), None, window, cx),
            })
            .collect();
        let keep_rows = match self.loaded.as_mut() {
            Some(loaded) => {
                let untouched = !loaded.draft.is_dirty();
                loaded.draft.refresh(user, defaults);
                loaded.sucrose = read_only;
                untouched
            }
            None => {
                self.loaded = Some(Loaded {
                    draft: ArgumentsDraft::new(user, defaults),
                    sucrose: read_only,
                });
                true
            }
        };
        if keep_rows || self.fields.is_empty() {
            self.rebuild(window, cx);
        }
        cx.notify();
    }

    /// The service stored `list`: the draft shows it, and a wallpaper that reads the arguments raises the prompt.
    pub fn on_saved(&mut self, list: &[SwitchView], window: &mut Window, cx: &mut Context<Self>) {
        self.saving = false;
        let Some(loaded) = self.loaded.as_mut() else {
            return;
        };
        let previous = loaded.draft.stored().to_vec();
        loaded.draft.mark_saved(list.to_vec());
        self.rebuild(window, cx);
        let setting = RestartSetting::Arguments(self.engine);
        if should_prompt(setting, &sorted(list), &sorted(&previous), &self.running) {
            let engines = affected_engines(setting, &self.running);
            open_restart_prompt(setting, engines, self.link.clone(), window, cx);
        }
    }

    /// The service refused the list or could not write it: the draft stays as it is (the page shows the toast).
    pub fn on_rejected(&mut self, cx: &mut Context<Self>) {
        self.saving = false;
        cx.notify();
    }

    fn engine_name(&self) -> String {
        tr(engine_label_key(self.engine)).to_string()
    }

    /// Whether the controls can be used.
    fn enabled(&self) -> bool {
        self.connected && self.loaded.is_some() && !self.saving
    }

    /// A box for the list: `value` as tooltip when it is long.
    fn boxed(id: String, width: Option<f32>, input: AnyElement, tip: Option<SharedString>) -> AnyElement {
        div()
            .id(id)
            .flex_shrink_0()
            .when(width.is_none(), |this| this.flex_1().min_w_0())
            .when_some(width, |this, width| this.w(px(width)))
            .when_some(tip, |this, tip| {
                this.tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
            })
            .child(input)
            .into_any_element()
    }

    /// One read-only row of Sucrose's own switches: muted, locked, with the explanation as tooltip.
    fn read_only_row(&self, index: usize, fields: &Fields, cx: &App) -> AnyElement {
        let muted = cx.theme().muted_foreground;
        let tip = tr("Portal.WallpaperSettingPage.Arguments.ReadOnly");
        h_flex()
            .id(format!("argument-sucrose-{index}"))
            .w_full()
            .h(px(ROW_HEIGHT))
            .gap_2()
            .items_center()
            .opacity(0.7)
            .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
            .child(
                div()
                    .flex_none()
                    .text_color(muted)
                    .child(Icon::new(IconName::Lock).with_size(px(12.))),
            )
            .child(Self::boxed(
                format!("argument-sucrose-name-{index}"),
                None,
                Input::new(&fields.name)
                    .prefix(div().text_color(muted).child("--"))
                    .disabled(true)
                    .into_any_element(),
                None,
            ))
            .child(Self::boxed(
                format!("argument-sucrose-value-{index}"),
                Some(VALUE_WIDTH),
                Input::new(&fields.value).disabled(true).into_any_element(),
                None,
            ))
            .child(div().flex_none().w(px(32.)))
            .into_any_element()
    }

    /// One row of the user's switches with its error text under it.
    fn user_row(&self, index: usize, draft: &ArgumentsDraft, fields: &Fields, cx: &mut Context<Self>) -> AnyElement {
        let muted = cx.theme().muted_foreground;
        let danger = cx.theme().danger;
        let enabled = self.enabled();
        let long_value = draft
            .rows()
            .get(index)
            .map(|row| row.value.clone())
            .filter(|text| text.chars().count() > TOOLTIP_FROM_CHARS);
        let remove_tip = tr("Portal.WallpaperSettingPage.Arguments.Remove");
        let row = h_flex()
            .w_full()
            .h(px(ROW_HEIGHT))
            .gap_2()
            .items_center()
            .child(Self::boxed(
                format!("argument-name-{index}"),
                None,
                Input::new(&fields.name)
                    .prefix(div().text_color(muted).child("--"))
                    .disabled(!enabled)
                    .into_any_element(),
                None,
            ))
            .child(Self::boxed(
                format!("argument-value-{index}"),
                Some(VALUE_WIDTH),
                Input::new(&fields.value).disabled(!enabled).into_any_element(),
                long_value.map(SharedString::from),
            ))
            .child(
                Button::new(format!("argument-remove-{index}"))
                    .ghost()
                    .icon(Icon::new(IconName::X))
                    .w(px(32.))
                    .h(px(32.))
                    .tooltip(remove_tip.clone())
                    .accessibility_label(remove_tip)
                    .disabled(!enabled)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if let Some(loaded) = this.loaded.as_mut() {
                            loaded.draft.remove_row(index);
                        }
                        this.rebuild(window, cx);
                    })),
            );
        match draft.error(index) {
            Some(error) => v_flex()
                .w_full()
                .child(row)
                .child(div().text_xs().text_color(danger).child(error.text()))
                .into_any_element(),
            None => row.into_any_element(),
        }
    }
}

impl Render for ArgumentsEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let engine = self.engine_name();
        let header = v_flex()
            .gap_1()
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .child(tr("Portal.WallpaperSettingPage.Arguments")),
            )
            .child(div().text_xs().text_color(muted).child(tr_with(
                "Portal.WallpaperSettingPage.Arguments.Description",
                &[("engine", &engine)],
            )));
        let enabled = self.enabled();
        let can_save = enabled && self.loaded.as_ref().is_some_and(|loaded| loaded.draft.can_save());
        let body = match self.loaded.as_ref() {
            None => Skeleton::new()
                .w_full()
                .h(px(ROW_HEIGHT))
                .rounded(px(6.))
                .into_any_element(),
            Some(loaded) => {
                let mut rows: Vec<AnyElement> = Vec::new();
                for (index, fields) in loaded.sucrose.iter().enumerate() {
                    rows.push(self.read_only_row(index, fields, cx));
                }
                let draft = loaded.draft.clone();
                for index in 0..draft.rows().len() {
                    if let Some(fields) = self.fields.get(index) {
                        let fields = Fields {
                            name: fields.name.clone(),
                            value: fields.value.clone(),
                        };
                        rows.push(self.user_row(index, &draft, &fields, cx));
                    }
                }
                if draft.rows().is_empty() {
                    rows.push(
                        div()
                            .text_sm()
                            .text_color(muted)
                            .child(tr_with(
                                "Portal.WallpaperSettingPage.Arguments.Empty",
                                &[("engine", &engine)],
                            ))
                            .into_any_element(),
                    );
                }
                v_flex()
                    .id("arguments-list")
                    .w_full()
                    .gap_1()
                    .max_h(px(LIST_MAX_HEIGHT))
                    .children(rows)
                    .overflow_y_scrollbar()
                    .into_any_element()
            }
        };
        let buttons = h_flex()
            .w_full()
            .gap_2()
            .items_center()
            .child(
                Button::new("argument-add")
                    .ghost()
                    .icon(Icon::new(IconName::Plus))
                    .label(tr("Portal.WallpaperSettingPage.Arguments.Add"))
                    .disabled(!enabled)
                    .on_click(cx.listener(|this, _, window, cx| {
                        let Some(loaded) = this.loaded.as_mut() else {
                            return;
                        };
                        loaded.draft.add_row();
                        let index = loaded.draft.rows().len() - 1;
                        this.push_fields(index, window, cx);
                        cx.notify();
                    })),
            )
            .child(
                Button::new("argument-reset")
                    .ghost()
                    .label(tr("Portal.WallpaperSettingPage.Arguments.Reset"))
                    .disabled(!enabled)
                    .on_click(cx.listener(|this, _, window, cx| {
                        if let Some(loaded) = this.loaded.as_mut() {
                            loaded.draft.reset();
                        }
                        this.rebuild(window, cx);
                    })),
            )
            .child(div().flex_1())
            .child(
                Button::new("argument-save")
                    .primary()
                    .label(tr("Portal.WallpaperSettingPage.Arguments.Save"))
                    .disabled(!can_save)
                    .on_click(cx.listener(|this, _, _, cx| {
                        let Some(loaded) = this.loaded.as_ref() else {
                            return;
                        };
                        if !loaded.draft.can_save() || this.saving {
                            return;
                        }
                        let list = loaded.draft.to_switches();
                        this.saving = true;
                        let change = save_change(this.engine, list);
                        this.link.send(PortalRequest::Change(change));
                        cx.notify();
                    })),
            );
        v_flex().w_full().gap_2().child(header).child(body).child(buttons)
    }
}

#[cfg(test)]
mod tests {
    use sucrose_ipc::portal::{MAX_SWITCHES, SwitchView};

    use sucrose_core::settings::EngineType;
    use sucrose_ipc::portal::SettingChange;

    use super::{ArgumentsDraft, RowError, normalize_name, save_change, saved_list};

    fn flag(name: &str) -> SwitchView {
        SwitchView {
            name: name.to_owned(),
            value: None,
        }
    }

    fn with(name: &str, value: &str) -> SwitchView {
        SwitchView {
            name: name.to_owned(),
            value: Some(value.to_owned()),
        }
    }

    fn draft() -> ArgumentsDraft {
        ArgumentsDraft::new(
            vec![
                flag("disable-extensions"),
                with("autoplay-policy", "no-user-gesture-required"),
            ],
            vec![
                flag("no-sandbox"),
                with("autoplay-policy", "document-user-activation-required"),
            ],
        )
    }

    #[test]
    fn arguments_the_editor_of_each_engine_saves_its_own_change() {
        let list = vec![flag("disable-extensions")];

        assert_eq!(
            save_change(EngineType::WebViewLive, list.clone()),
            SettingChange::WebArguments(list.clone())
        );
        assert_eq!(
            save_change(EngineType::CefSharpLive, list.clone()),
            SettingChange::CefArguments(list.clone())
        );
        assert_eq!(
            saved_list(&SettingChange::WebArguments(list.clone())),
            Some((EngineType::WebViewLive, list.as_slice()))
        );
        assert_eq!(
            saved_list(&SettingChange::CefArguments(list.clone())),
            Some((EngineType::CefSharpLive, list.as_slice()))
        );
        assert_eq!(saved_list(&SettingChange::DeveloperMode(true)), None);
    }

    #[test]
    fn arguments_a_new_draft_shows_the_stored_list_and_is_not_dirty() {
        let draft = draft();

        assert_eq!(draft.rows().len(), 2);
        assert_eq!(draft.rows()[1].value, "no-user-gesture-required");
        assert!(!draft.is_dirty());
        assert!(!draft.can_save(), "nothing to save");
    }

    #[test]
    fn arguments_a_row_can_be_added_and_removed() {
        let mut draft = draft();

        draft.add_row();
        assert_eq!(draft.rows().len(), 3);
        draft.remove_row(0);
        assert_eq!(draft.rows().len(), 2);
        assert_eq!(draft.rows()[0].name, "autoplay-policy");
        draft.remove_row(9);
        assert_eq!(draft.rows().len(), 2, "a row that is not there changes nothing");
    }

    #[test]
    fn arguments_a_blank_row_is_left_out_and_is_no_error() {
        let mut draft = draft();

        draft.add_row();

        assert_eq!(draft.error(2), None);
        assert_eq!(draft.to_switches().len(), 2);
        assert!(!draft.is_dirty(), "an empty row alone changes nothing");
    }

    #[test]
    fn arguments_reset_puts_the_given_defaults_into_the_rows_and_makes_the_draft_dirty() {
        let mut draft = draft();

        draft.reset();

        assert_eq!(
            draft.to_switches(),
            [
                flag("no-sandbox"),
                with("autoplay-policy", "document-user-activation-required")
            ]
        );
        assert!(draft.is_dirty());
        assert!(draft.can_save());
        assert_eq!(draft.stored().len(), 2, "nothing is stored before Save");
    }

    #[test]
    fn arguments_dirty_compares_with_the_stored_list_whatever_the_order() {
        let mut draft = draft();
        draft.set_name(0, "autoplay-policy");
        draft.set_value(0, "no-user-gesture-required");
        draft.set_name(1, "disable-extensions");
        draft.set_value(1, "");

        assert!(!draft.is_dirty(), "the same switches in another order are not a change");

        draft.set_value(1, "1");
        assert!(draft.is_dirty());
        draft.set_value(1, "  ");
        assert!(!draft.is_dirty(), "a blank value means none");
    }

    #[test]
    fn arguments_a_value_without_a_name_is_invalid() {
        let mut draft = draft();
        draft.add_row();
        draft.set_value(2, "x");

        assert_eq!(draft.error(2), Some(RowError::Invalid));
        assert!(!draft.can_save());
    }

    #[test]
    fn arguments_a_name_with_a_space_or_another_character_is_invalid() {
        for name in ["has space", "a=b", "ünï", "x_y", "a b-c"] {
            let mut draft = draft();
            draft.add_row();
            draft.set_name(2, name);

            assert_eq!(draft.error(2), Some(RowError::Invalid), "{name:?}");
            assert!(!draft.can_save(), "{name:?}");
        }
    }

    #[test]
    fn arguments_a_repeated_name_is_a_duplicate_on_the_later_row_and_blocks_save() {
        let mut draft = draft();
        draft.add_row();
        draft.set_name(2, "--disable-extensions");

        assert_eq!(draft.error(0), None, "the first of two is fine");
        assert_eq!(draft.error(2), Some(RowError::Duplicate));
        assert!(!draft.can_save());

        draft.remove_row(2);
        draft.add_row();
        draft.set_name(2, "my-switch");
        assert!(draft.can_save(), "a new valid name is savable");
    }

    #[test]
    fn arguments_leading_dashes_and_blanks_are_stripped_for_the_stored_form() {
        assert_eq!(normalize_name("  --no-sandbox "), "no-sandbox");
        assert_eq!(normalize_name("-x"), "x");
        assert_eq!(normalize_name("--"), "");
        let mut draft = draft();
        draft.add_row();
        draft.set_name(2, " --my-switch ");
        draft.set_value(2, " 1 ");

        assert_eq!(draft.to_switches()[2], with("my-switch", "1"));
    }

    #[test]
    fn arguments_a_list_over_the_limit_cannot_be_saved() {
        let mut draft = ArgumentsDraft::new(Vec::new(), Vec::new());
        for index in 0..=MAX_SWITCHES {
            draft.add_row();
            draft.set_name(index, &format!("s{index}"));
        }

        assert!(draft.is_dirty());
        assert!(!draft.can_save());
    }

    #[test]
    fn arguments_a_saved_list_becomes_the_stored_one_and_the_rows() {
        let mut draft = draft();
        draft.reset();

        draft.mark_saved(vec![flag("x")]);

        assert_eq!(draft.stored(), [flag("x")]);
        assert_eq!(draft.rows().len(), 1);
        assert!(!draft.is_dirty());
    }

    #[test]
    fn arguments_new_lists_from_the_service_keep_the_edits_of_a_dirty_draft() {
        let mut draft = draft();
        draft.add_row();
        draft.set_name(2, "mine");

        draft.refresh(vec![flag("from-service")], vec![flag("default")]);

        assert_eq!(draft.rows().len(), 3, "the edits stay");
        assert_eq!(draft.stored(), [flag("from-service")]);
        let mut clean = ArgumentsDraft::new(vec![flag("a")], Vec::new());
        clean.refresh(vec![flag("b")], Vec::new());
        assert_eq!(clean.rows()[0].name, "b", "an untouched draft follows the service");
    }
}
