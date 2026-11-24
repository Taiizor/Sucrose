//! The expander row of the settings pages (UI-SPEC E7) and the rules for committing, clamping and reverting a value.
//! A control shows only the value the service broadcast: a change is sent, the control keeps what was broadcast, and a
//! rejected change puts the broadcast value back.

use gpui_kit::component::{
    ActiveTheme, IndexPath, StyledExt as _, h_flex,
    input::InputState,
    select::{Select, SelectEvent, SelectItem, SelectState},
    skeleton::Skeleton,
    tooltip::Tooltip,
    v_flex,
};
use gpui_kit::*;

use crate::i18n::tr;

/// Width of a select (UI-SPEC: settings row controls).
pub const SELECT_WIDTH: f32 = 200.0;
/// Width of a slider.
#[allow(dead_code, reason = "the General page of this plan's last task uses it")]
pub const SLIDER_WIDTH: f32 = 220.0;
/// Width of a numeric field.
pub const NUMBER_WIDTH: f32 = 100.0;
/// Radius of a row (UI-SPEC: 8 px).
const ROW_RADIUS: f32 = 8.0;
/// Opacity of the row's surface over the page (UI-SPEC: alpha 0.55).
const ROW_ALPHA: f32 = 0.55;

/// The number a numeric field commits: its text clamped to `min..=max`; an empty or non-numeric text keeps `previous`.
pub fn commit_number(text: &str, min: u8, max: u8, previous: u8) -> u8 {
    let text = text.trim();
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return previous;
    }
    // A text of only digits that is too long for a number is simply too large.
    let value = text.parse::<u64>().unwrap_or(u64::MAX);
    let clamped = value.clamp(u64::from(min), u64::from(max));
    u8::try_from(clamped).unwrap_or(max)
}

/// The percentage a slider value stands for.
#[allow(dead_code, reason = "the General page of this plan's last task uses it")]
pub fn slider_percent(value: f32) -> u8 {
    // Clamped to 0..=100 first, so the cast cannot overflow.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let percent = value.round().clamp(0.0, 100.0) as u8;
    percent
}

/// The label of a legacy key without the trailing colon the XAML labels carry.
pub fn label_text(key: &str) -> SharedString {
    let text = tr(key);
    SharedString::from(text.trim_end().trim_end_matches(':').trim_end().to_owned())
}

/// What a row's controls can do now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlState {
    /// The service has reported no settings yet: a skeleton stands where the control will be.
    Loading,
    /// The service is not connected: the control keeps the last broadcast value and cannot be used.
    Disabled,
    /// The control works.
    Enabled,
}

/// The state of the controls for what the Portal knows about the service.
pub fn control_state(has_settings: bool, connected: bool) -> ControlState {
    if !has_settings {
        ControlState::Loading
    } else if !connected {
        ControlState::Disabled
    } else {
        ControlState::Enabled
    }
}

impl ControlState {
    /// Whether the controls refuse input.
    pub fn disabled(self) -> bool {
        self != Self::Enabled
    }
}

/// An expander row: a title over a description, a control at the end and a footer of sub-options.
pub struct SettingRow {
    title: SharedString,
    description: Option<SharedString>,
    control: Option<AnyElement>,
    control_width: f32,
    footer: Vec<AnyElement>,
    state: ControlState,
}

impl SettingRow {
    /// A row with `title` and an optional `description`.
    pub fn new(title: SharedString, description: Option<SharedString>) -> Self {
        Self {
            title,
            description,
            control: None,
            control_width: SELECT_WIDTH,
            footer: Vec::new(),
            state: ControlState::Enabled,
        }
    }

    /// The control at the end of the row, `width` wide while its skeleton stands in.
    pub fn control(mut self, control: impl IntoElement, width: f32) -> Self {
        self.control = Some(control.into_any_element());
        self.control_width = width;
        self
    }

    /// One line of the footer under the row's border.
    pub fn footer(mut self, line: impl IntoElement) -> Self {
        self.footer.push(line.into_any_element());
        self
    }

    /// What the controls can do (a skeleton replaces them while loading).
    pub fn state(mut self, state: ControlState) -> Self {
        self.state = state;
        self
    }

    /// The row as an element.
    pub fn build(self, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let loading = self.state == ControlState::Loading;
        let mut texts = v_flex().flex_1().min_w_0().gap_1().child(
            // A title is one line with an ellipsis.
            div().text_sm().font_semibold().truncate().child(self.title),
        );
        if let Some(description) = self.description {
            // A description wraps.
            texts = texts.child(div().text_xs().text_color(theme.muted_foreground).child(description));
        }
        let mut top = h_flex().w_full().gap_4().items_center().child(texts);
        if loading {
            if self.control.is_some() {
                top = top.child(
                    Skeleton::new()
                        .flex_none()
                        .w(px(self.control_width))
                        .h(px(32.))
                        .rounded(px(6.)),
                );
            }
        } else if let Some(control) = self.control {
            top = top.child(div().flex_none().child(control));
        }
        let mut row = v_flex()
            .w_full()
            .gap_3()
            .p_4()
            .rounded(px(ROW_RADIUS))
            .border_1()
            .border_color(theme.border)
            .bg(theme.secondary.opacity(ROW_ALPHA))
            .child(top);
        if !self.footer.is_empty() && !loading {
            row = row.child(
                v_flex()
                    .w_full()
                    .gap_2()
                    .pt_3()
                    .border_t_1()
                    .border_color(theme.border)
                    .children(self.footer),
            );
        }
        row.into_any_element()
    }
}

/// One line of a footer: `label` at the start, `control` at the end.
pub fn footer_line(label: SharedString, control: impl IntoElement) -> AnyElement {
    h_flex()
        .w_full()
        .gap_4()
        .items_center()
        .justify_between()
        .child(div().flex_1().min_w_0().text_sm().truncate().child(label))
        .child(div().flex_none().child(control))
        .into_any_element()
}

/// One option of a select. An option with a `reason` stays visible but cannot be chosen, and the reason is its tooltip.
#[derive(Clone)]
pub struct Choice<T: Clone + PartialEq + 'static> {
    /// The value the option stands for.
    pub value: T,
    /// The text of the option.
    pub title: SharedString,
    /// Why the option cannot be chosen, if it cannot.
    pub reason: Option<SharedString>,
}

impl<T: Clone + PartialEq + 'static> Choice<T> {
    /// An option that can be chosen.
    pub fn new(value: T, title: SharedString) -> Self {
        Self {
            value,
            title,
            reason: None,
        }
    }
}

impl<T: Clone + PartialEq + 'static> SelectItem for Choice<T> {
    type Value = T;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &T {
        &self.value
    }

    fn disabled(&self) -> bool {
        self.reason.is_some()
    }

    fn render(&self, _: &mut Window, _: &mut App) -> impl IntoElement {
        match &self.reason {
            None => div().child(self.title.clone()).into_any_element(),
            Some(reason) => {
                let reason = reason.clone();
                div()
                    .id(self.title.clone())
                    .child(self.title.clone())
                    .tooltip(move |window, cx| Tooltip::new(reason.clone()).build(window, cx))
                    .into_any_element()
            }
        }
    }
}

/// The state of a select over [`Choice`]s.
pub type ChoiceSelect<T> = Entity<SelectState<Vec<Choice<T>>>>;

/// A select over `items` with `selected` chosen (nothing chosen while the service has reported nothing).
pub fn choice_select<T: Clone + PartialEq + 'static, V: 'static>(
    items: Vec<Choice<T>>,
    selected: Option<&T>,
    window: &mut Window,
    cx: &mut Context<V>,
) -> ChoiceSelect<T> {
    let index = selected
        .and_then(|value| items.iter().position(|choice| &choice.value == value))
        .map(IndexPath::new);
    cx.new(|cx| SelectState::new(items, index, window, cx))
}

/// Shows `value` in the select without telling anyone (a broadcast value, or the revert of a rejected change).
pub fn show_choice<T: Clone + PartialEq + 'static>(
    select: &ChoiceSelect<T>,
    value: &T,
    window: &mut Window,
    cx: &mut App,
) {
    select.update(cx, |state, cx| state.set_selected_value(value, window, cx));
}

/// Calls `handler` with the value the user chose.
pub fn on_choice<T: Clone + PartialEq + 'static, V: 'static>(
    select: &ChoiceSelect<T>,
    window: &mut Window,
    cx: &mut Context<V>,
    handler: impl Fn(&mut V, T, &mut Window, &mut Context<V>) + 'static,
) -> Subscription {
    cx.subscribe_in(
        select,
        window,
        move |this, _, event: &SelectEvent<Vec<Choice<T>>>, window, cx| {
            if let SelectEvent::Confirm(Some(value)) = event {
                handler(this, value.clone(), window, cx);
            }
        },
    )
}

/// A select in a box of its own width: the select takes the full row width otherwise (docs/GPUI.md).
pub fn select_box<T: Clone + PartialEq + 'static>(select: &ChoiceSelect<T>, disabled: bool) -> AnyElement {
    div()
        .w(px(SELECT_WIDTH))
        .flex_shrink_0()
        .child(Select::new(select).disabled(disabled))
        .into_any_element()
}

/// A numeric field: digits only, at most three, committed with [`commit_number`].
pub struct NumberField {
    /// The text box.
    pub state: Entity<InputState>,
    /// The least value.
    pub min: u8,
    /// The most value.
    pub max: u8,
}

impl NumberField {
    /// A field for `min..=max`.
    pub fn new<V: 'static>(min: u8, max: u8, window: &mut Window, cx: &mut Context<V>) -> Self {
        let state = cx.new(|cx| {
            InputState::new(window, cx)
                .validate(|text, _| text.len() <= 3 && text.bytes().all(|byte| byte.is_ascii_digit()))
        });
        Self { state, min, max }
    }

    /// Shows `value` without raising a change.
    pub fn show(&self, value: u8, window: &mut Window, cx: &mut App) {
        let text = SharedString::from(value.to_string());
        self.state.update(cx, |state, cx| state.set_value(text, window, cx));
    }

    /// Commits the text: the field shows the clamped value and the value is returned when it differs from `previous`.
    pub fn commit(&self, previous: u8, window: &mut Window, cx: &mut App) -> Option<u8> {
        let text = self.state.read(cx).value();
        let value = commit_number(&text, self.min, self.max, previous);
        if text.as_ref() != value.to_string() {
            self.show(value, window, cx);
        }
        (value != previous).then_some(value)
    }

    /// Whether the cursor is in the field.
    pub fn focused(&self, window: &Window, cx: &App) -> bool {
        self.state.read(cx).focus_handle(cx).contains_focused(window, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::{ControlState, commit_number, control_state, label_text, slider_percent};

    #[test]
    fn settings_number_empty_text_keeps_the_previous_value() {
        assert_eq!(commit_number("", 1, 100, 30), 30);
        assert_eq!(commit_number("   ", 1, 100, 30), 30);
    }

    #[test]
    fn settings_number_below_the_range_is_raised_to_the_minimum() {
        assert_eq!(commit_number("0", 1, 100, 30), 1);
        assert_eq!(commit_number("3", 5, 25, 10), 5);
    }

    #[test]
    fn settings_number_above_the_range_is_cut_to_the_maximum() {
        assert_eq!(commit_number("250", 1, 100, 30), 100);
        assert_eq!(commit_number("99999999999999999999", 1, 100, 30), 100);
    }

    #[test]
    fn settings_number_inside_the_range_is_kept() {
        assert_eq!(commit_number("42", 1, 100, 30), 42);
        assert_eq!(commit_number(" 7 ", 0, 100, 30), 7);
        assert_eq!(commit_number("0", 0, 100, 30), 0);
    }

    #[test]
    fn settings_number_that_is_not_a_number_keeps_the_previous_value() {
        assert_eq!(commit_number("4a", 1, 100, 30), 30);
        assert_eq!(commit_number("-5", 1, 100, 30), 30);
        assert_eq!(commit_number("1.5", 1, 100, 30), 30);
    }

    #[test]
    fn settings_controls_wait_for_the_first_settings_then_follow_the_connection() {
        assert_eq!(control_state(false, false), ControlState::Loading);
        assert_eq!(control_state(false, true), ControlState::Loading);
        assert_eq!(control_state(true, false), ControlState::Disabled);
        assert_eq!(control_state(true, true), ControlState::Enabled);
        assert!(ControlState::Loading.disabled());
        assert!(ControlState::Disabled.disabled());
        assert!(!ControlState::Enabled.disabled());
    }

    #[test]
    fn settings_slider_value_rounds_and_stays_within_zero_to_a_hundred() {
        assert_eq!(slider_percent(49.6), 50);
        assert_eq!(slider_percent(-3.0), 0);
        assert_eq!(slider_percent(140.0), 100);
    }

    #[test]
    fn settings_label_loses_the_trailing_colon_of_the_legacy_text() {
        assert_eq!(
            &*label_text("Portal.PersonalSettingPage.Preview.LibraryPreviewHide"),
            "Hide Details"
        );
    }
}
