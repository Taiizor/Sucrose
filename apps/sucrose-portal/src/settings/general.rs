//! The General page (UI-SPEC E7, E8, E9): theme mode, notification area icon, window backdrop with its background image,
//! and the library location. Every control sends a SettingChange and shows only what the service broadcast.

use std::{path::Path, time::Duration};

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    slider::{Slider, SliderEvent, SliderState},
    tooltip::Tooltip,
};
use gpui_kit::*;
use sucrose_core::settings::{Backdrop, Stretch, ThemeMode};
use sucrose_ipc::portal::{PortalRequest, PortalSettings, SettingChange};

use super::row::{
    Choice, ChoiceSelect, ControlState, SELECT_WIDTH, SLIDER_WIDTH, SettingRow, choice_select, control_state,
    footer_line, label_text, on_choice, select_box, show_choice, slider_percent,
};
use crate::{
    background::{self, BackgroundError, BackgroundImage},
    i18n::{tr, tr_with},
    ipc::ServiceLink,
    library,
};

/// Why a backdrop cannot be chosen on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackdropReason {
    /// Windows before 11 (22H2) cannot show Mica, Acrylic or Tabbed.
    RequiresWindows11,
    /// Another operating system has no such material; carries its display name.
    UnavailableOn(String),
}

/// One option of the Window Backdrop select.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackdropOption {
    /// The backdrop the option stands for.
    pub value: Backdrop,
    /// Why it cannot be chosen, if it cannot.
    pub reason: Option<BackdropReason>,
}

/// The options of Window Backdrop for a machine that can (`supported`) or cannot show the materials, running `os` (the
/// value of `std::env::consts::OS`).
pub fn backdrop_options(supported: bool, os: &str) -> Vec<BackdropOption> {
    let reason = if os == "windows" {
        BackdropReason::RequiresWindows11
    } else {
        BackdropReason::UnavailableOn(os_display(os))
    };
    Backdrop::ALL
        .iter()
        .map(|value| BackdropOption {
            value: *value,
            // None is the plain window: it works everywhere.
            reason: (!supported && *value != Backdrop::None).then(|| reason.clone()),
        })
        .collect()
}

/// The display name of an `std::env::consts::OS` value.
fn os_display(os: &str) -> String {
    match os {
        "windows" => "Windows".to_owned(),
        "macos" => "macOS".to_owned(),
        "linux" => "Linux".to_owned(),
        other => other.to_owned(),
    }
}

/// The text that says why `reason` blocks a backdrop.
pub fn reason_text(reason: &BackdropReason) -> SharedString {
    match reason {
        BackdropReason::RequiresWindows11 => tr("Portal.GeneralSettingPage.WindowBackdrop.RequiresWin11"),
        BackdropReason::UnavailableOn(os) => {
            tr_with("Portal.GeneralSettingPage.WindowBackdrop.Unavailable", &[("os", os)])
        }
    }
}

/// Whether the file at `path` may be offered as the background image by its extension (the legacy file filter).
pub fn image_choice(path: &Path) -> Result<(), BackgroundError> {
    if background::supported_extension(path) {
        Ok(())
    } else {
        Err(BackgroundError::Unsupported)
    }
}

/// The checks a chosen file passes before it is sent to the service: its extension, then that it can be opened. It does
/// file I/O, so it runs on the background executor.
fn check_image_file(path: &Path) -> Result<(), BackgroundError> {
    image_choice(path)?;
    std::fs::File::open(path)
        .map(|_| ())
        .map_err(|error| background::classify_error(&error))
}

/// `path` cut to `max_chars` characters, keeping its end (the file name is what tells paths apart).
pub fn truncate_path(path: &str, max_chars: usize) -> String {
    let count = path.chars().count();
    if count <= max_chars || max_chars == 0 {
        return path.to_owned();
    }
    let keep = max_chars - 1;
    let tail: String = path.chars().skip(count - keep).collect();
    format!("\u{2026}{tail}")
}

/// How many characters of a path the rows show before the end is cut; the row's own width cuts what is left.
const PATH_CHARS: usize = 64;
/// How long the opacity slider rests before its value is sent.
const SLIDER_REST: Duration = Duration::from_millis(150);

fn theme_choices() -> Vec<Choice<ThemeMode>> {
    [
        (ThemeMode::Light, "Portal.GeneralSettingPage.ThemeMode.Light"),
        (ThemeMode::Dark, "Portal.GeneralSettingPage.ThemeMode.Dark"),
        (ThemeMode::System, "Portal.GeneralSettingPage.ThemeMode.System"),
    ]
    .into_iter()
    .map(|(mode, key)| Choice::new(mode, tr(key)))
    .collect()
}

fn notify_choices() -> Vec<Choice<bool>> {
    vec![
        Choice::new(true, tr("Portal.GeneralSettingPage.NotifyIcon.Notify.Show")),
        Choice::new(false, tr("Portal.GeneralSettingPage.NotifyIcon.Notify.Hide")),
    ]
}

fn backdrop_choices(supported: bool) -> Vec<Choice<Backdrop>> {
    backdrop_options(supported, std::env::consts::OS)
        .into_iter()
        .map(|option| {
            let key = match option.value {
                Backdrop::None => "Portal.Enum.WindowBackdropType.None",
                Backdrop::Mica => "Portal.Enum.WindowBackdropType.Mica",
                Backdrop::Acrylic => "Portal.Enum.WindowBackdropType.Acrylic",
                Backdrop::Tabbed => "Portal.Enum.WindowBackdropType.Tabbed",
            };
            Choice {
                value: option.value,
                title: tr(key),
                reason: option.reason.as_ref().map(reason_text),
            }
        })
        .collect()
}

fn stretch_choices() -> Vec<Choice<Stretch>> {
    [
        (Stretch::None, "Portal.Enum.Stretch.None"),
        (Stretch::Fill, "Portal.Enum.Stretch.Fill"),
        (Stretch::Uniform, "Portal.Enum.Stretch.Uniform"),
        (Stretch::UniformToFill, "Portal.Enum.Stretch.UniformToFill"),
    ]
    .into_iter()
    .map(|(stretch, key)| Choice::new(stretch, tr(key)))
    .collect()
}

/// The General settings page.
pub struct GeneralPage {
    link: ServiceLink,
    settings: Option<PortalSettings>,
    connected: bool,
    /// The background image layer, whose decode error the Background image row shows.
    background: Entity<BackgroundImage>,
    theme: ChoiceSelect<ThemeMode>,
    notify: ChoiceSelect<bool>,
    backdrop: ChoiceSelect<Backdrop>,
    stretch: ChoiceSelect<Stretch>,
    opacity: Entity<SliderState>,
    /// The send that waits for the slider to rest; replacing it cancels the one before.
    opacity_rest: Option<Task<()>>,
    /// Why the image the user picked last was not used.
    image_error: Option<BackgroundError>,
    _subscriptions: Vec<Subscription>,
}

impl GeneralPage {
    /// A page that shows skeletons until the service reports the settings; `window_backdrop` says whether this OS can show
    /// a window backdrop.
    pub fn new(
        link: ServiceLink,
        window_backdrop: bool,
        background: Entity<BackgroundImage>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let theme = choice_select(theme_choices(), None, window, cx);
        let notify = choice_select(notify_choices(), None, window, cx);
        let backdrop = choice_select(backdrop_choices(window_backdrop), None, window, cx);
        let stretch = choice_select(stretch_choices(), None, window, cx);
        let opacity = cx.new(|_| SliderState::new().min(0.0).max(100.0).step(1.0).default_value(100.0));
        let subscriptions = vec![
            on_choice(&theme, window, cx, |this, mode, _, _| {
                this.send(SettingChange::ThemeMode(mode));
            }),
            on_choice(&notify, window, cx, |this, visible, _, _| {
                this.send(SettingChange::AppVisible(visible));
            }),
            on_choice(&backdrop, window, cx, |this, backdrop, _, _| {
                this.send(SettingChange::Backdrop(backdrop));
            }),
            on_choice(&stretch, window, cx, |this, stretch, _, _| {
                this.send(SettingChange::BackgroundStretch(stretch));
            }),
            // The slider sends on release, or when it has rested for a moment.
            cx.subscribe_in(&opacity, window, |this, _, event: &SliderEvent, _, cx| match event {
                SliderEvent::Change(value) => this.rest_then_send_opacity(slider_percent(value.start()), cx),
                SliderEvent::Release(value) => {
                    this.opacity_rest = None;
                    this.send_opacity(slider_percent(value.start()));
                }
            }),
            // The layer's decode error shows in the row.
            cx.observe(&background, |_, _, cx| cx.notify()),
        ];
        Self {
            link,
            settings: None,
            connected: false,
            background,
            theme,
            notify,
            backdrop,
            stretch,
            opacity,
            opacity_rest: None,
            image_error: None,
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
        show_choice(&self.theme, &settings.theme_mode, window, cx);
        show_choice(&self.notify, &settings.app_visible, window, cx);
        show_choice(&self.backdrop, &settings.backdrop, window, cx);
        show_choice(&self.stretch, &settings.background_stretch, window, cx);
        self.show_opacity(settings.background_opacity, window, cx);
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
        match change {
            SettingChange::ThemeMode(_) => show_choice(&self.theme, &settings.theme_mode, window, cx),
            SettingChange::AppVisible(_) => show_choice(&self.notify, &settings.app_visible, window, cx),
            SettingChange::Backdrop(_) => show_choice(&self.backdrop, &settings.backdrop, window, cx),
            SettingChange::BackgroundStretch(_) => {
                show_choice(&self.stretch, &settings.background_stretch, window, cx);
            }
            SettingChange::BackgroundOpacity(_) => self.show_opacity(settings.background_opacity, window, cx),
            _ => {}
        }
        cx.notify();
    }

    /// Whether the keyboard is in a select of this page (Escape then belongs to it).
    pub fn field_focused(&self, window: &Window, cx: &App) -> bool {
        self.theme.read(cx).focus_handle(cx).contains_focused(window, cx)
            || self.notify.read(cx).focus_handle(cx).contains_focused(window, cx)
            || self.backdrop.read(cx).focus_handle(cx).contains_focused(window, cx)
            || self.stretch.read(cx).focus_handle(cx).contains_focused(window, cx)
    }

    fn state(&self) -> ControlState {
        control_state(self.settings.is_some(), self.connected)
    }

    /// Sends `change`; the controls keep showing the broadcast value until the service answers.
    fn send(&self, change: SettingChange) {
        if self.state() == ControlState::Enabled {
            self.link.send(PortalRequest::Change(change));
        }
    }

    fn show_opacity(&self, percent: u8, window: &mut Window, cx: &mut App) {
        self.opacity
            .update(cx, |slider, cx| slider.set_value(f32::from(percent), window, cx));
    }

    fn send_opacity(&self, percent: u8) {
        if self.settings.as_ref().is_some_and(|s| s.background_opacity != percent) {
            self.send(SettingChange::BackgroundOpacity(percent));
        }
    }

    fn rest_then_send_opacity(&mut self, percent: u8, cx: &mut Context<Self>) {
        self.opacity_rest = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SLIDER_REST).await;
            this.update(cx, |this, cx| {
                this.send_opacity(percent);
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    /// Opens the file picker; the chosen file is checked on the background executor and only then sent to the service.
    fn choose_image(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(tr("Portal.GeneralSettingPage.WindowBackdrop.BackgroundImage.Title")),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = chosen.await else { return };
            let Some(path) = paths.into_iter().next() else { return };
            let checked = cx
                .background_executor()
                .spawn(async move {
                    check_image_file(&path)?;
                    path.to_str().map(str::to_owned).ok_or(BackgroundError::Unsupported)
                })
                .await;
            this.update_in(cx, |this, _, cx| {
                match checked {
                    Ok(path) => {
                        this.image_error = None;
                        this.send(SettingChange::BackgroundImage(path));
                    }
                    // The setting is not changed; the row says why.
                    Err(error) => this.image_error = Some(error),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn remove_image(&mut self, cx: &mut Context<Self>) {
        self.image_error = None;
        self.send(SettingChange::BackgroundImage(String::new()));
        cx.notify();
    }

    /// A switch-less checkbox that shows `checked` and sends the change `make` builds from the new value.
    fn checkbox(
        &self,
        id: &'static str,
        checked: bool,
        label: SharedString,
        make: fn(bool) -> SettingChange,
        cx: &mut Context<Self>,
    ) -> Checkbox {
        Checkbox::new(id)
            .label(label)
            .checked(checked)
            .disabled(self.state().disabled())
            .on_click(cx.listener(move |this, value: &bool, _, _| this.send(make(*value))))
    }
}

/// A path as a row shows it: cut from the front, ellipsis at the end by the row's width, the whole path as the tooltip.
fn path_text(id: &'static str, path: &str, width: Option<f32>, cx: &App) -> AnyElement {
    let full = SharedString::from(path.to_owned());
    let mut text = div()
        .id(id)
        .min_w_0()
        .truncate()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(truncate_path(path, PATH_CHARS));
    text = match width {
        Some(width) => text.w(px(width)).flex_none(),
        None => text.flex_1(),
    };
    text.tooltip(move |window, cx| Tooltip::new(full.clone()).build(window, cx))
        .into_any_element()
}

impl Render for GeneralPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state();
        let disabled = state.disabled();
        let settings = self.settings.clone();
        let muted = cx.theme().muted_foreground;

        let theme = SettingRow::new(
            tr("Portal.GeneralSettingPage.ThemeMode"),
            Some(tr("Portal.GeneralSettingPage.ThemeMode.Description")),
        )
        .control(select_box(&self.theme, disabled), SELECT_WIDTH)
        .state(state)
        .build(cx);

        let notify = SettingRow::new(
            tr("Portal.GeneralSettingPage.NotifyIcon"),
            Some(tr("Portal.GeneralSettingPage.NotifyIcon.Description")),
        )
        .control(select_box(&self.notify, disabled), SELECT_WIDTH)
        .footer(self.checkbox(
            "app-exit",
            settings.as_ref().is_some_and(|s| s.app_exit),
            tr("Portal.GeneralSettingPage.NotifyIcon.NotifyExit"),
            SettingChange::AppExit,
            cx,
        ))
        .state(state)
        .build(cx);

        // The background image: pick, remove and the path; a picked file that cannot be used, or a stored one that cannot
        // be decoded, says so in its row and changes nothing.
        let image_path = settings
            .as_ref()
            .map(|s| s.background_image.clone())
            .unwrap_or_default();
        let image_error = self.image_error.or_else(|| self.background.read(cx).error());
        let image_line = h_flex()
            .w_full()
            .gap_2()
            .items_center()
            .child(
                Button::new("background-select")
                    .primary()
                    .label(tr("Portal.GeneralSettingPage.WindowBackdrop.BackgroundImage.Select"))
                    .disabled(disabled)
                    .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.choose_image(window, cx))),
            )
            .child(
                Button::new("background-remove")
                    .label(tr("Portal.GeneralSettingPage.WindowBackdrop.BackgroundImage.Remove"))
                    .disabled(disabled || image_path.is_empty())
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.remove_image(cx))),
            )
            .child(if image_path.is_empty() {
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_xs()
                    .text_color(muted)
                    .child(tr("Portal.GeneralSettingPage.WindowBackdrop.BackgroundImage.None"))
                    .into_any_element()
            } else {
                path_text("background-path", &image_path, None, cx)
            });
        let opacity = self.opacity.read(cx).value().start();
        let mut backdrop = SettingRow::new(
            tr("Portal.GeneralSettingPage.WindowBackdrop"),
            Some(tr("Portal.GeneralSettingPage.WindowBackdrop.Description")),
        )
        .control(select_box(&self.backdrop, disabled), SELECT_WIDTH)
        .footer(image_line);
        if let Some(error) = image_error {
            backdrop = backdrop.footer(
                div()
                    .text_xs()
                    .text_color(cx.theme().danger)
                    .child(tr(error.text_key())),
            );
        }
        let backdrop = backdrop
            .footer(footer_line(
                label_text("Portal.GeneralSettingPage.WindowBackdrop.BackdropStretch"),
                select_box(&self.stretch, disabled),
            ))
            .footer(footer_line(
                label_text("Portal.GeneralSettingPage.WindowBackdrop.BackdropOpacity"),
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(Slider::new(&self.opacity).w(px(SLIDER_WIDTH)).disabled(disabled))
                    .child(
                        div()
                            .w(px(40.))
                            .text_xs()
                            .text_color(muted)
                            .child(format!("{}%", slider_percent(opacity))),
                    ),
            ))
            .state(state)
            .build(cx);

        // Library Location: the folder in use; plan 22 adds the folder picker and the move.
        let location = settings
            .as_ref()
            .map(|s| library::location_of(s).to_string_lossy().into_owned())
            .unwrap_or_default();
        let open_target = location.clone();
        let library_row = SettingRow::new(
            tr("Portal.GeneralSettingPage.PrivateLibrary"),
            Some(tr("Portal.GeneralSettingPage.PrivateLibrary.Description")),
        )
        .control(
            h_flex()
                .gap_2()
                .items_center()
                .child(path_text("library-path", &location, Some(LOCATION_WIDTH), cx))
                .child({
                    let tip = tr("Portal.GeneralSettingPage.PrivateLibrary.OpenFolder");
                    Button::new("library-open")
                        .ghost()
                        .icon(IconName::FolderOpen)
                        .tooltip(tip.clone())
                        .accessibility_label(tip)
                        .on_click(move |_, _, cx| cx.reveal_path(Path::new(&open_target)))
                }),
            LOCATION_WIDTH,
        )
        .footer(self.checkbox(
            "library-move",
            settings.as_ref().is_some_and(|s| s.library_move),
            tr("Portal.GeneralSettingPage.PrivateLibrary.LibraryMove"),
            SettingChange::LibraryMove,
            cx,
        ))
        .state(state)
        .build(cx);

        super::column(
            "settings-general",
            vec![
                super::group(tr("Portal.SettingsPage.GroupBehavior"), vec![theme, notify, backdrop]),
                super::group(tr("Portal.SettingsPage.GroupLibrary"), vec![library_row]),
            ],
        )
    }
}

/// Width of the library path in its row.
const LOCATION_WIDTH: f32 = 300.0;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use sucrose_core::settings::Backdrop;

    use super::{BackdropReason, backdrop_options, image_choice, reason_text, truncate_path};
    use crate::background::BackgroundError;

    fn disabled(options: &[super::BackdropOption]) -> Vec<Backdrop> {
        options.iter().filter(|o| o.reason.is_some()).map(|o| o.value).collect()
    }

    #[test]
    fn settings_backdrop_options_list_the_four_materials_in_order() {
        let values: Vec<Backdrop> = backdrop_options(true, "windows").iter().map(|o| o.value).collect();
        assert_eq!(
            values,
            [Backdrop::None, Backdrop::Mica, Backdrop::Acrylic, Backdrop::Tabbed]
        );
        assert!(disabled(&backdrop_options(true, "windows")).is_empty());
    }

    #[test]
    fn settings_backdrop_options_before_windows_11_disable_everything_but_none() {
        let options = backdrop_options(false, "windows");
        assert_eq!(
            disabled(&options),
            [Backdrop::Mica, Backdrop::Acrylic, Backdrop::Tabbed]
        );
        assert_eq!(options[0].reason, None);
        for option in &options[1..] {
            assert_eq!(option.reason, Some(BackdropReason::RequiresWindows11));
        }
        assert_eq!(&*reason_text(&BackdropReason::RequiresWindows11), "Requires Windows 11");
    }

    #[test]
    fn settings_backdrop_options_on_other_systems_name_the_system() {
        for (os, name) in [("macos", "macOS"), ("linux", "Linux")] {
            let options = backdrop_options(false, os);
            assert_eq!(
                disabled(&options),
                [Backdrop::Mica, Backdrop::Acrylic, Backdrop::Tabbed],
                "{os}"
            );
            let reason = options[1].reason.clone().expect("a reason");
            assert_eq!(reason, BackdropReason::UnavailableOn(name.to_owned()));
            assert_eq!(&*reason_text(&reason), format!("Not available on {name}"));
        }
    }

    #[test]
    fn settings_image_choice_accepts_the_legacy_extensions_in_any_case() {
        for name in ["a.png", "a.JPG", "b.jpeg", "c.TIFF", "d.webp", "e.GiF"] {
            assert_eq!(image_choice(Path::new(name)), Ok(()), "{name}");
        }
    }

    #[test]
    fn settings_image_choice_refuses_other_extensions() {
        for name in ["a.bmp", "a.svg", "a", "a.png.exe", ".png"] {
            assert_eq!(
                image_choice(Path::new(name)),
                Err(BackgroundError::Unsupported),
                "{name}"
            );
        }
    }

    #[test]
    fn settings_path_keeps_the_end_of_a_long_path() {
        let path = "C:\\Users\\Someone\\Pictures\\Wallpapers\\mountains.png";
        let cut = truncate_path(path, 24);
        assert_eq!(cut.chars().count(), 24);
        assert!(cut.starts_with('\u{2026}'));
        assert!(cut.ends_with("\\mountains.png"));
    }

    #[test]
    fn settings_path_that_fits_is_left_alone_and_the_cut_counts_characters() {
        assert_eq!(truncate_path("C:\\a.png", 24), "C:\\a.png");
        let wide = "é".repeat(30);
        let cut = truncate_path(&wide, 10);
        assert_eq!(cut.chars().count(), 10);
    }
}
