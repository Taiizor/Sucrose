//! The header row of the library mode (UI-SPEC E3, E4): the Library tab, the search box, the light/dark toggle, Premium and
//! the more menu with Help and About (D-15: only what Phase 3 has). Items follow each other in flex order, so the row
//! mirrors for Arabic and Hebrew (UI-SPEC E4 RTL row). Later plans add the settings gear.

use std::time::Duration;

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    menu::{DropdownMenu as _, PopupMenuItem},
    tab::{Tab, TabBar},
    tooltip::Tooltip,
};
use gpui_kit::*;
use sucrose_core::settings::ThemeMode;
use sucrose_ipc::portal::{PortalRequest, SettingChange};

use crate::{
    dialogs,
    i18n::tr,
    ipc::ServiceLink,
    library::LibraryView,
    settings::{Mode, SettingsTab},
};

/// The premium theme store, opened in the OS browser (legacy value of `Url.PremiumStore`; fixed at build time, T-03-42).
const PREMIUM_URL: &str = "https://sucrose.dev/premium";
/// Height of the row (UI-SPEC spacing: 2xl).
const HEADER_HEIGHT: f32 = 48.0;
/// The search box width, and the least it shrinks to before anything else collapses.
const SEARCH_WIDTH: f32 = 320.0;
const SEARCH_MIN_WIDTH: f32 = 200.0;
/// The longest search text, in characters (legacy parity).
pub const SEARCH_MAX_CHARS: usize = 20;
/// How long typing must pause before the library is filtered.
const DEBOUNCE: Duration = Duration::from_millis(500);

/// The search text as the library filters by it: trimmed, lowercased and cut to [`SEARCH_MAX_CHARS`] characters.
pub fn normalize_query(text: &str) -> String {
    text.trim().to_lowercase().chars().take(SEARCH_MAX_CHARS).collect()
}

/// The theme mode the toggle asks for: the opposite of what the window shows (an explicit Light or Dark, also from System).
pub fn toggle_target(dark: bool) -> ThemeMode {
    if dark { ThemeMode::Light } else { ThemeMode::Dark }
}

/// The header asks for another mode: the gear, Back or a settings tab was pressed.
pub struct HeaderEvent(pub Mode);

impl EventEmitter<HeaderEvent> for Header {}

/// The header row. The search text lives here, so it survives a visit to the settings.
pub struct Header {
    /// Which mode's row is drawn.
    mode: Mode,
    library: Entity<LibraryView>,
    /// Sends the theme change to the service.
    link: ServiceLink,
    /// Whether the window shows the dark theme (what the toggle's icon reflects).
    dark: bool,
    search: Entity<InputState>,
    /// The pending debounce; replacing it cancels the one before.
    pending: Option<Task<()>>,
    _subscription: Subscription,
}

impl Header {
    /// The header that filters `library`.
    pub fn new(
        library: Entity<LibraryView>,
        link: ServiceLink,
        mode: Mode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(tr("Portal.MainWindow.SearchBoxHint"))
                // The box takes at most the legacy 20 characters: an edit that would pass that is refused.
                .validate(|text, _| text.chars().count() <= SEARCH_MAX_CHARS)
                // Escape clears the box while it is focused (UI-SPEC E3).
                .clean_on_escape()
        });
        let subscription = cx.subscribe(&search, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.on_search_changed(cx);
            }
        });
        Self {
            mode,
            library,
            link,
            dark: false,
            search,
            pending: None,
            _subscription: subscription,
        }
    }

    /// The window now shows the dark (or light) theme: the toggle's icon follows.
    pub fn set_dark(&mut self, dark: bool, cx: &mut Context<Self>) {
        if self.dark != dark {
            self.dark = dark;
            cx.notify();
        }
    }

    /// The window changed mode: the row of that mode is drawn.
    pub fn set_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
        if self.mode != mode {
            self.mode = mode;
            cx.notify();
        }
    }

    /// Puts the cursor in the search box (Ctrl+F).
    pub fn focus_search(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search.update(cx, |state, cx| state.focus(window, cx));
    }

    /// The box text changed: filter after the debounce, or at once when the box was emptied (clear button, Escape).
    fn on_search_changed(&mut self, cx: &mut Context<Self>) {
        let query = normalize_query(&self.search.read(cx).value());
        if query.is_empty() {
            self.pending = None;
            self.apply(&query, cx);
            return;
        }
        let timer = cx.background_executor().timer(DEBOUNCE);
        self.pending = Some(cx.spawn(async move |this, cx| {
            timer.await;
            this.update(cx, |this, cx| this.apply(&query, cx)).ok();
        }));
    }

    /// Hands the query to the library. It stays in this process: nothing is sent anywhere and it is not logged.
    fn apply(&mut self, query: &str, cx: &mut Context<Self>) {
        self.library.update(cx, |library, cx| library.set_query(query, cx));
    }
}

impl Render for Header {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.mode {
            Mode::Library => self.library_row(cx),
            Mode::Settings(tab) => self.settings_row(tab, cx),
        }
    }
}

impl Header {
    /// The row of the settings mode (UI-SPEC E4): Back, then the General and Personal tabs; no search, no actions.
    fn settings_row(&self, tab: SettingsTab, cx: &mut Context<Self>) -> AnyElement {
        let back_tip = tr("Portal.MainWindow.MenuBackTip");
        let header = cx.entity();
        let selected = match tab {
            SettingsTab::General => 0,
            SettingsTab::Personal => 1,
            SettingsTab::Wallpaper => 2,
        };
        h_flex()
            .w_full()
            .h(px(HEADER_HEIGHT))
            .flex_shrink_0()
            .px_4()
            .gap_3()
            .items_center()
            .child(
                Button::new("settings-back")
                    .ghost()
                    .icon(IconName::ArrowLeft)
                    .tooltip(back_tip.clone())
                    .accessibility_label(back_tip)
                    .on_click(cx.listener(|_, _: &ClickEvent, _, cx| cx.emit(HeaderEvent(Mode::Library)))),
            )
            .child(
                TabBar::new("settings-tabs")
                    .underline()
                    .selected_index(selected)
                    .on_click(move |index, _, cx| {
                        let tab = if *index == 1 {
                            SettingsTab::Personal
                        } else {
                            SettingsTab::General
                        };
                        header.update(cx, |_, cx| cx.emit(HeaderEvent(Mode::Settings(tab))));
                    })
                    .child(Tab::new().label(tr("Portal.MainWindow.MenuGeneral")))
                    .child(Tab::new().label(tr("Portal.MainWindow.MenuPersonal"))),
            )
            .into_any_element()
    }

    /// The row of the library mode.
    fn library_row(&self, cx: &mut Context<Self>) -> AnyElement {
        let settings_tip = tr("Portal.MainWindow.MenuSettingTip");
        let divider = div().flex_none().w(px(1.)).h(px(20.)).bg(cx.theme().border);
        let tip = tr("Portal.MainWindow.SearchBoxTip");
        // Sun while dark, Moon while light; the tooltip names the theme a click switches to.
        let (icon, toggle_tip) = if self.dark {
            (IconName::Sun, tr("Portal.MainWindow.MenuLightTip"))
        } else {
            (IconName::Moon, tr("Portal.MainWindow.MenuDarkTip"))
        };
        let target = toggle_target(self.dark);
        let premium_tip = tr("Portal.MainWindow.MenuPremiumStoreTip");
        let more_tip = tr("Portal.MainWindow.MenuOtherOptionsTip");
        h_flex()
            .w_full()
            .h(px(HEADER_HEIGHT))
            .flex_shrink_0()
            .px_4()
            .gap_3()
            .items_center()
            .child(
                div().flex_none().child(
                    TabBar::new("header-tabs").underline().selected_index(0).child(
                        Tab::new()
                            .prefix(Icon::new(IconName::Library).small())
                            .label(tr("Portal.MainWindow.MenuLibrary")),
                    ),
                ),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("search-box")
                    .w(px(SEARCH_WIDTH))
                    .min_w(px(SEARCH_MIN_WIDTH))
                    .flex_shrink(1.)
                    .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                    .child(
                        Input::new(&self.search)
                            .prefix(Icon::new(IconName::Search).small())
                            .cleanable(true)
                            .aria_label(tr("Portal.MainWindow.SearchBoxHint")),
                    ),
            )
            .child(
                h_flex()
                    .flex_none()
                    .gap_2()
                    .items_center()
                    .child(
                        Button::new("theme-toggle")
                            .ghost()
                            .icon(icon)
                            .tooltip(toggle_tip.clone())
                            .accessibility_label(toggle_tip)
                            .on_click(cx.listener(move |this, _: &ClickEvent, _, _| {
                                this.link.send(PortalRequest::Change(SettingChange::ThemeMode(target)));
                            })),
                    )
                    .child(
                        Button::new("premium")
                            .ghost()
                            .icon(IconName::Crown)
                            .tooltip(premium_tip.clone())
                            .accessibility_label(premium_tip)
                            .on_click(|_, _, cx| dialogs::open_link(PREMIUM_URL, cx)),
                    )
                    .child(
                        Button::new("more")
                            .ghost()
                            .icon(IconName::EllipsisVertical)
                            .tooltip(more_tip.clone())
                            .accessibility_label(more_tip)
                            .dropdown_menu_with_anchor(Anchor::TopRight, |menu, _, _| {
                                menu.item(
                                    PopupMenuItem::new(tr("Portal.MainWindow.MenuHelp"))
                                        .icon(IconName::BookOpen)
                                        .on_click(|_, window, cx| dialogs::help::open(window, cx)),
                                )
                                .item(
                                    PopupMenuItem::new(tr("Portal.MainWindow.MenuAbout"))
                                        .icon(IconName::Info)
                                        .on_click(|_, window, cx| dialogs::about::open(window, cx)),
                                )
                            }),
                    )
                    .child(divider)
                    .child(
                        Button::new("settings-gear")
                            .ghost()
                            .icon(IconName::Settings)
                            .tooltip(settings_tip.clone())
                            .accessibility_label(settings_tip)
                            .on_click(cx.listener(|_, _: &ClickEvent, _, cx| {
                                cx.emit(HeaderEvent(Mode::Library.enter_settings()));
                            })),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use sucrose_core::settings::ThemeMode;

    use super::{PREMIUM_URL, SEARCH_MAX_CHARS, normalize_query, toggle_target};
    use crate::dialogs::urls;

    #[test]
    fn premium_is_a_page_of_the_official_site_over_https() {
        assert_eq!(PREMIUM_URL, format!("{}/premium", urls::OFFICIAL));
        assert!(PREMIUM_URL.starts_with("https://"));
    }

    #[test]
    fn theme_toggle_asks_for_the_opposite_of_what_is_shown() {
        assert_eq!(toggle_target(true), ThemeMode::Light);
        assert_eq!(toggle_target(false), ThemeMode::Dark);
    }

    #[test]
    fn search_text_is_trimmed_and_lowercased() {
        assert_eq!(normalize_query("  Sea NIGHT "), "sea night");
        assert_eq!(normalize_query("   "), "");
        assert_eq!(normalize_query(""), "");
    }

    #[test]
    fn search_text_is_cut_to_twenty_characters_not_bytes() {
        assert_eq!(normalize_query(&"a".repeat(30)), "a".repeat(SEARCH_MAX_CHARS));
        // Each of these is two bytes: the cut counts characters.
        assert_eq!(normalize_query(&"é".repeat(25)), "é".repeat(SEARCH_MAX_CHARS));
        assert_eq!(normalize_query(&"é".repeat(25)).chars().count(), SEARCH_MAX_CHARS);
    }
}
