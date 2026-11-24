//! The Personal page (UI-SPEC E7): the Phase 3 keys in their legacy places. Every control sends a SettingChange and shows
//! only what the service broadcast.

use gpui_kit::component::input::{Input, InputEvent};
use gpui_kit::*;
use sucrose_ipc::portal::{PortalRequest, PortalSettings, SettingChange};

use super::row::{ControlState, NUMBER_WIDTH, NumberField, SettingRow, control_state};
use crate::{i18n::tr, ipc::ServiceLink};

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
    _subscriptions: Vec<Subscription>,
}

impl PersonalPage {
    /// A page that shows skeletons until the service reports the settings.
    pub fn new(link: ServiceLink, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let fields = Number::ALL.map(|number| {
            let (min, max) = number.range();
            NumberField::new(min, max, window, cx)
        });
        // Enter and leaving the field commit what was typed.
        let subscriptions = Number::ALL
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
        Self {
            link,
            settings: None,
            connected: false,
            fields,
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
        cx.notify();
    }

    /// Whether the cursor is in a field of this page (Escape then belongs to the field).
    pub fn field_focused(&self, window: &Window, cx: &App) -> bool {
        self.fields.iter().any(|field| field.focused(window, cx))
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
        let pagination = SettingRow::new(
            tr("Portal.PersonalSettingPage.Library"),
            Some(tr("Portal.PersonalSettingPage.Library.Description")),
        )
        .control(self.number_input(Number::Pagination), NUMBER_WIDTH)
        .state(state)
        .build(cx);
        super::column(
            "settings-personal",
            vec![super::group(
                tr("Portal.SettingsPage.GroupAppearance"),
                vec![pagination],
            )],
        )
    }
}

#[cfg(test)]
mod tests {
    use sucrose_ipc::portal::SettingChange;

    use super::Number;

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
}
