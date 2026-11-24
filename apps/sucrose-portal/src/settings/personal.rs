//! The Personal page (UI-SPEC E7): the Phase 3 keys in their legacy places. Every control sends a SettingChange and shows
//! only what the service broadcast.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _,
    input::{Input, InputEvent},
    switch::Switch,
};
use gpui_kit::*;
use sucrose_core::settings::{SortKind, SortMode};
use sucrose_ipc::portal::{PortalRequest, PortalSettings, SettingChange};

use super::row::{
    Choice, ChoiceSelect, ControlState, NUMBER_WIDTH, NumberField, SettingRow, choice_select, control_state,
    footer_line, label_text, on_choice, select_box, show_choice,
};
use crate::{
    i18n::{tr, tr_with},
    ipc::ServiceLink,
};

/// The numeric fields of the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Number {
    /// Spacing Distance of the adaptive layout.
    Margin,
    /// Layout Order of the adaptive layout.
    Layout,
    /// Library Pagination.
    Pagination,
}

impl Number {
    /// Every field, in the order of `PersonalPage::fields`.
    const ALL: [Self; 3] = [Self::Margin, Self::Layout, Self::Pagination];

    /// The least and the most value the field takes.
    pub fn range(self) -> (u8, u8) {
        match self {
            Self::Margin => (5, 25),
            Self::Layout => (0, 100),
            Self::Pagination => (1, 100),
        }
    }

    /// The value the service reported for this field.
    pub fn value(self, settings: &PortalSettings) -> u8 {
        match self {
            Self::Margin => settings.adaptive_margin,
            Self::Layout => settings.adaptive_layout,
            Self::Pagination => settings.library_pagination,
        }
    }

    /// The change that sets this field to `value`.
    pub fn change(self, value: u8) -> SettingChange {
        match self {
            Self::Margin => SettingChange::AdaptiveMargin(value),
            Self::Layout => SettingChange::AdaptiveLayout(value),
            Self::Pagination => SettingChange::LibraryPagination(value),
        }
    }

    /// The field a change belongs to, if it belongs to one.
    pub fn of(change: &SettingChange) -> Option<Self> {
        match change {
            SettingChange::AdaptiveMargin(_) => Some(Self::Margin),
            SettingChange::AdaptiveLayout(_) => Some(Self::Layout),
            SettingChange::LibraryPagination(_) => Some(Self::Pagination),
            _ => None,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Margin => 0,
            Self::Layout => 1,
            Self::Pagination => 2,
        }
    }
}

/// The Personal settings page.
pub struct PersonalPage {
    link: ServiceLink,
    settings: Option<PortalSettings>,
    connected: bool,
    fields: [NumberField; 3],
    sort_mode: ChoiceSelect<SortMode>,
    sort_kind: ChoiceSelect<SortKind>,
    _subscriptions: Vec<Subscription>,
}

impl PersonalPage {
    /// A page that shows skeletons until the service reports the settings.
    pub fn new(link: ServiceLink, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let fields = Number::ALL.map(|number| {
            let (min, max) = number.range();
            NumberField::new(min, max, window, cx)
        });
        let sort_mode = choice_select(sort_mode_choices(), None, window, cx);
        let sort_kind = choice_select(sort_kind_choices(), None, window, cx);
        // Enter and leaving the field commit what was typed.
        let mut subscriptions: Vec<Subscription> = Number::ALL
            .into_iter()
            .map(|number| {
                cx.subscribe_in(
                    &fields[number.index()].state,
                    window,
                    move |this, _, event: &InputEvent, window, cx| {
                        if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                            this.commit(number, window, cx);
                        }
                    },
                )
            })
            .collect();
        subscriptions.push(on_choice(&sort_mode, window, cx, |this, mode, _, _| {
            this.send(SettingChange::LibrarySortMode(mode));
        }));
        subscriptions.push(on_choice(&sort_kind, window, cx, |this, kind, _, _| {
            this.send(SettingChange::LibrarySortKind(kind));
        }));
        Self {
            link,
            settings: None,
            connected: false,
            fields,
            sort_mode,
            sort_kind,
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

    /// The service reported every setting (Welcome): every control shows its value.
    pub fn load(&mut self, settings: &PortalSettings, window: &mut Window, cx: &mut Context<Self>) {
        self.settings = Some(settings.clone());
        for number in Number::ALL {
            self.fields[number.index()].show(number.value(settings), window, cx);
        }
        show_choice(&self.sort_mode, &settings.library_sort_mode, window, cx);
        show_choice(&self.sort_kind, &settings.library_sort_kind, window, cx);
        cx.notify();
    }

    /// A setting changed or its change was refused: `settings` holds the broadcast values, and the control of `change`
    /// shows its broadcast value again.
    pub fn on_setting(
        &mut self,
        settings: &PortalSettings,
        change: &SettingChange,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings = Some(settings.clone());
        if let Some(number) = Number::of(change) {
            self.fields[number.index()].show(number.value(settings), window, cx);
        }
        match change {
            SettingChange::LibrarySortMode(_) => show_choice(&self.sort_mode, &settings.library_sort_mode, window, cx),
            SettingChange::LibrarySortKind(_) => show_choice(&self.sort_kind, &settings.library_sort_kind, window, cx),
            _ => {}
        }
        cx.notify();
    }

    /// Whether the cursor is in a field of this page (Escape then belongs to the field).
    pub fn field_focused(&self, window: &Window, cx: &App) -> bool {
        self.fields.iter().any(|field| field.focused(window, cx))
            || self.sort_mode.read(cx).focus_handle(cx).contains_focused(window, cx)
            || self.sort_kind.read(cx).focus_handle(cx).contains_focused(window, cx)
    }

    /// Sends `change`; the controls keep showing the broadcast value until the service answers.
    fn send(&self, change: SettingChange) {
        if self.state() == ControlState::Enabled {
            self.link.send(PortalRequest::Change(change));
        }
    }

    /// A switch that shows `checked` and sends the change `make` builds from the new value.
    fn switch(
        &self,
        id: &'static str,
        checked: bool,
        disabled: bool,
        label: SharedString,
        make: fn(bool) -> SettingChange,
        cx: &mut Context<Self>,
    ) -> Switch {
        Switch::new(id)
            .checked(checked)
            .disabled(disabled)
            .accessibility_label(label)
            .on_click(cx.listener(move |this, value: &bool, _, _| this.send(make(*value))))
    }

    fn state(&self) -> ControlState {
        control_state(self.settings.is_some(), self.connected)
    }

    /// Commits a numeric field: the clamped value is shown and sent when it differs from the broadcast one.
    fn commit(&mut self, number: Number, window: &mut Window, cx: &mut Context<Self>) {
        let Some(settings) = &self.settings else { return };
        let previous = number.value(settings);
        let field = &self.fields[number.index()];
        if self.state() != ControlState::Enabled {
            field.show(previous, window, cx);
            return;
        }
        if let Some(value) = field.commit(previous, window, cx) {
            self.link.send(PortalRequest::Change(number.change(value)));
        }
    }

    fn number_input(&self, number: Number) -> impl IntoElement {
        div()
            .w(px(NUMBER_WIDTH))
            .child(Input::new(&self.fields[number.index()].state).disabled(self.state().disabled()))
    }
}

impl Render for PersonalPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state();
        let disabled = state.disabled();
        // Until the first Welcome nothing is shown; the rows are skeletons then.
        let settings = self.settings.clone();
        let flag = |read: fn(&PortalSettings) -> bool| settings.as_ref().is_some_and(read);

        let bin = super::bin_name();
        let corrupt = SettingRow::new(
            tr("Portal.PersonalSettingPage.Delete"),
            Some(tr_with(
                "Portal.PersonalSettingPage.Delete.Description",
                &[("bin", bin)],
            )),
        )
        .control(
            self.switch(
                "corrupt-themes",
                flag(|s| s.delete_corrupt),
                disabled,
                tr("Portal.PersonalSettingPage.Delete"),
                SettingChange::DeleteCorrupt,
                cx,
            ),
            40.,
        )
        .footer(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(tr_with("Portal.PersonalSettingPage.Delete.DeleteHint", &[("bin", bin)])),
        )
        .state(state)
        .build(cx);

        let preview_on = flag(|s| s.library_preview);
        let preview = SettingRow::new(
            tr("Portal.PersonalSettingPage.Preview"),
            Some(tr("Portal.PersonalSettingPage.Preview.Description")),
        )
        .control(
            self.switch(
                "library-preview",
                preview_on,
                disabled,
                label_text("Portal.PersonalSettingPage.Preview.LibraryPreview"),
                SettingChange::LibraryPreview,
                cx,
            ),
            40.,
        )
        .footer(footer_line(
            label_text("Portal.PersonalSettingPage.Preview.LibraryPreviewHide"),
            // Hide Details has nothing to hide while the preview is off.
            self.switch(
                "library-preview-hide",
                flag(|s| s.library_preview_hide),
                disabled || !preview_on,
                label_text("Portal.PersonalSettingPage.Preview.LibraryPreviewHide"),
                SettingChange::LibraryPreviewHide,
                cx,
            ),
        ))
        .state(state)
        .build(cx);

        let adaptive = SettingRow::new(
            tr("Portal.PersonalSettingPage.Adaptive"),
            Some(tr("Portal.PersonalSettingPage.Adaptive.Description")),
        )
        .footer(footer_line(
            label_text("Portal.PersonalSettingPage.Adaptive.AdaptiveMargin"),
            self.number_input(Number::Margin),
        ))
        .footer(footer_line(
            label_text("Portal.PersonalSettingPage.Adaptive.AdaptiveLayout"),
            self.number_input(Number::Layout),
        ))
        .state(state)
        .build(cx);

        let sorting = SettingRow::new(
            tr("Portal.PersonalSettingPage.Sort"),
            Some(tr("Portal.PersonalSettingPage.Sort.Description")),
        )
        .footer(footer_line(
            label_text("Portal.PersonalSettingPage.Sort.SortMode"),
            select_box(&self.sort_mode, disabled),
        ))
        .footer(footer_line(
            label_text("Portal.PersonalSettingPage.Sort.SortKind"),
            select_box(&self.sort_kind, disabled),
        ))
        .state(state)
        .build(cx);

        let pagination = SettingRow::new(
            tr("Portal.PersonalSettingPage.Library"),
            Some(tr("Portal.PersonalSettingPage.Library.Description")),
        )
        .control(self.number_input(Number::Pagination), NUMBER_WIDTH)
        .state(state)
        .build(cx);

        super::column(
            "settings-personal",
            vec![
                super::group(tr("Portal.SettingsPage.GroupLibrary"), vec![corrupt]),
                super::group(
                    tr("Portal.SettingsPage.GroupAppearance"),
                    vec![preview, adaptive, sorting, pagination],
                ),
            ],
        )
    }
}

/// The options of Sort Mode, in the order of the legacy select.
pub fn sort_mode_choices() -> Vec<Choice<SortMode>> {
    [
        (SortMode::Name, "Portal.Enum.SortModeType.Name"),
        (SortMode::Creation, "Portal.Enum.SortModeType.Creation"),
        (SortMode::Modification, "Portal.Enum.SortModeType.Modification"),
    ]
    .into_iter()
    .map(|(mode, key)| Choice::new(mode, tr(key)))
    .collect()
}

/// The options of Sort Type.
pub fn sort_kind_choices() -> Vec<Choice<SortKind>> {
    [
        (SortKind::Ascending, "Portal.Enum.SortKindType.Ascending"),
        (SortKind::Descending, "Portal.Enum.SortKindType.Descending"),
    ]
    .into_iter()
    .map(|(kind, key)| Choice::new(kind, tr(key)))
    .collect()
}

#[cfg(test)]
mod tests {
    use sucrose_ipc::portal::SettingChange;

    use sucrose_core::settings::{SortKind, SortMode};

    use super::{Number, sort_kind_choices, sort_mode_choices};

    #[test]
    fn settings_number_fields_have_the_legacy_ranges() {
        assert_eq!(Number::Margin.range(), (5, 25));
        assert_eq!(Number::Layout.range(), (0, 100));
        assert_eq!(Number::Pagination.range(), (1, 100));
    }

    #[test]
    fn settings_number_field_changes_name_their_setting() {
        assert_eq!(Number::Pagination.change(10), SettingChange::LibraryPagination(10));
        assert_eq!(Number::Margin.change(7), SettingChange::AdaptiveMargin(7));
        assert_eq!(Number::Layout.change(0), SettingChange::AdaptiveLayout(0));
    }

    #[test]
    fn settings_number_field_is_found_from_its_change() {
        for number in Number::ALL {
            assert_eq!(Number::of(&number.change(9)), Some(number));
        }
        assert_eq!(Number::of(&SettingChange::DeleteCorrupt(true)), None);
    }

    #[test]
    fn settings_sort_mode_offers_name_creation_and_modification() {
        let modes: Vec<SortMode> = sort_mode_choices().into_iter().map(|choice| choice.value).collect();
        assert_eq!(modes, [SortMode::Name, SortMode::Creation, SortMode::Modification]);
        let titles: Vec<String> = sort_mode_choices()
            .into_iter()
            .map(|choice| choice.title.to_string())
            .collect();
        assert_eq!(titles, ["Name", "Creation", "Modification"]);
    }

    #[test]
    fn settings_sort_type_offers_both_directions_with_the_legacy_text() {
        let kinds: Vec<SortKind> = sort_kind_choices().into_iter().map(|choice| choice.value).collect();
        assert_eq!(kinds, [SortKind::Ascending, SortKind::Descending]);
        let titles: Vec<String> = sort_kind_choices()
            .into_iter()
            .map(|choice| choice.title.to_string())
            .collect();
        assert_eq!(titles, ["Ascending [A-Z]", "Descending [Z-A]"]);
    }
}
