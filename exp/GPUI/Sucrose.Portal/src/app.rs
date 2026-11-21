use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::{
    ActiveTheme, Icon, IndexPath, Sizable, Theme, ThemeMode, TitleBar, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    menu::{DropdownMenu, PopupMenuItem},
    notification::Notification,
    pagination::Pagination,
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectState},
    skeleton::Skeleton,
    slider::{Slider, SliderEvent, SliderState},
    switch::Switch,
    tab::{Tab, TabBar},
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::data::{self, Settings, StoreCategory, StoreItem, Wallpaper};
use crate::i18n::{self, Lang, t};
use crate::thumbs::{self, ThumbSource};

pub const DEFAULT_TINT: f32 = 0.78;
const PAGE_SIZE: usize = 30;
const GAP: f32 = 12.0;
const MIN_CARD_WIDTH: f32 = 270.0;
const SIDEBAR_WIDTH: f32 = 220.0;

/// Applies the light/dark theme with the Windows 11 accent and a translucent
/// background so the window backdrop (Acrylic/Mica) shows through.
pub fn apply_theme(dark: bool, tint: f32, cx: &mut App) {
    Theme::change(if dark { ThemeMode::Dark } else { ThemeMode::Light }, None, cx);

    let accent: Hsla = if dark { rgb(0x60CDFF) } else { rgb(0x005FB8) }.into();
    let theme = Theme::global_mut(cx);
    theme.primary = accent;
    theme.primary_hover = accent.opacity(0.9);
    theme.primary_active = accent.opacity(0.8);
    theme.primary_foreground = if dark { black() } else { white() };
    theme.ring = accent;
    theme.slider_bar = accent;
    theme.background = theme.background.opacity(tint);
    theme.title_bar = theme.title_bar.opacity(tint * 0.5);
    theme.status_bar = theme.status_bar.opacity(tint * 0.5);
    theme.sidebar = theme.sidebar.opacity(tint * 0.4);
    cx.refresh_windows();
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Library,
    Store,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    General,
    Personal,
    System,
    Wallpaper,
    Performance,
    Other,
    Donate,
}

impl Section {
    const ALL: [Section; 7] = [
        Section::General,
        Section::Personal,
        Section::System,
        Section::Wallpaper,
        Section::Performance,
        Section::Other,
        Section::Donate,
    ];

    fn label(self) -> &'static str {
        t(match self {
            Section::General => "general",
            Section::Personal => "personal",
            Section::System => "system",
            Section::Wallpaper => "wallpaper",
            Section::Performance => "performance",
            Section::Other => "other",
            Section::Donate => "donate",
        })
    }

    fn icon(self) -> Lucide {
        match self {
            Section::General => Lucide::Settings,
            Section::Personal => Lucide::UserRound,
            Section::System => Lucide::Monitor,
            Section::Wallpaper => Lucide::Wallpaper,
            Section::Performance => Lucide::Gauge,
            Section::Other => Lucide::Info,
            Section::Donate => Lucide::HandHeart,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Backdrop {
    Acrylic,
    Mica,
    MicaAlt,
    Solid,
}

impl Backdrop {
    const ALL: [Backdrop; 4] = [Backdrop::Acrylic, Backdrop::Mica, Backdrop::MicaAlt, Backdrop::Solid];

    fn appearance(self) -> WindowBackgroundAppearance {
        match self {
            Backdrop::Acrylic => WindowBackgroundAppearance::Blurred,
            Backdrop::Mica => WindowBackgroundAppearance::MicaBackdrop,
            Backdrop::MicaAlt => WindowBackgroundAppearance::MicaAltBackdrop,
            Backdrop::Solid => WindowBackgroundAppearance::Opaque,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Backdrop::Acrylic => t("acrylic"),
            Backdrop::Mica => "Mica",
            Backdrop::MicaAlt => "Mica Alt",
            Backdrop::Solid => t("solid"),
        }
    }
}

#[derive(Clone)]
enum Thumb {
    Loading,
    Ready(Arc<RenderImage>),
    Failed,
}

enum Meta {
    Loading,
    Ready(SharedString, SharedString),
    Failed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Download {
    Running,
    Done,
}

type Choice = Entity<SelectState<Vec<SharedString>>>;
type ChoiceEvent = SelectEvent<Vec<SharedString>>;

struct Choices {
    language: Choice,
    theme: Choice,
    backdrop: Choice,
    startup: Choice,
    tray: Choice,
    fullscreen: Choice,
}

struct Sliders {
    tint: Entity<SliderState>,
    volume: Entity<SliderState>,
    cpu: Entity<SliderState>,
    memory: Entity<SliderState>,
    battery: Entity<SliderState>,
}

pub struct Portal {
    page: Page,
    previous_page: Page,
    section: Section,
    search: Entity<InputState>,
    query: String,
    library_dir: Option<PathBuf>,
    library: Option<Vec<Wallpaper>>,
    library_page: usize,
    selected: Option<String>,
    hovered: Option<String>,
    store: Option<Vec<StoreCategory>>,
    store_category: usize,
    store_page: usize,
    downloads: HashMap<String, Download>,
    thumbs: HashMap<String, Thumb>,
    metas: HashMap<String, Meta>,
    playing: bool,
    dark: bool,
    backdrop: Backdrop,
    tint: f32,
    memory_mb: f32,
    ticks: usize,
    toggles: HashMap<&'static str, bool>,
    choices: Choices,
    sliders: Sliders,
    logo: Arc<Image>,
    choice_subscriptions: Vec<Subscription>,
    _subscriptions: Vec<Subscription>,
}

impl Portal {
    pub fn new(settings: Settings, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder(t("search")));
        let sliders = Sliders {
            tint: Self::slider(DEFAULT_TINT * 100.0, cx),
            volume: Self::slider(100.0, cx),
            cpu: Self::slider(70.0, cx),
            memory: Self::slider(80.0, cx),
            battery: Self::slider(50.0, cx),
        };

        let mut subscriptions = vec![
            cx.subscribe(&search, |this, state, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.query = state.read(cx).value().trim().to_lowercase();
                    this.library_page = 0;
                    this.store_page = 0;
                    cx.notify();
                }
            }),
            cx.subscribe_in(&sliders.tint, window, |this, _, event: &SliderEvent, window, cx| {
                if let SliderEvent::Change(value) = event {
                    this.tint = value.start() / 100.0;
                    this.apply_look(window, cx);
                }
            }),
        ];
        for slider in [&sliders.volume, &sliders.cpu, &sliders.memory, &sliders.battery] {
            subscriptions.push(cx.subscribe(slider, |_, _, _: &SliderEvent, cx| cx.notify()));
        }

        let backdrop = Backdrop::Acrylic;
        let (choices, choice_subscriptions) = Self::build_choices(settings.dark, backdrop, window, cx);
        let mut toggles = HashMap::new();
        toggles.insert("library_preview", settings.library_preview);
        toggles.insert("tray_exit", true);
        toggles.insert("remote", true);
        toggles.insert("saver", true);

        // --page=store|settings opens a page directly (handy for screenshots).
        let page = match std::env::args().find_map(|arg| arg.strip_prefix("--page=").map(str::to_owned)).as_deref() {
            Some("store") => Page::Store,
            Some("settings") => Page::Settings,
            _ => Page::Library,
        };

        let section = match std::env::args().find_map(|arg| arg.strip_prefix("--section=").map(str::to_owned)).as_deref() {
            Some("personal") => Section::Personal,
            Some("performance") => Section::Performance,
            Some("other") => Section::Other,
            _ => Section::General,
        };

        let mut this = Self {
            page,
            previous_page: Page::Library,
            section,
            search,
            query: String::new(),
            library_dir: settings.library.clone(),
            library: None,
            library_page: 0,
            selected: settings.selected.clone(),
            hovered: None,
            store: None,
            store_category: 0,
            store_page: 0,
            downloads: HashMap::new(),
            thumbs: HashMap::new(),
            metas: HashMap::new(),
            playing: true,
            dark: settings.dark,
            backdrop,
            tint: DEFAULT_TINT,
            memory_mb: 0.0,
            ticks: 0,
            toggles,
            choices,
            sliders,
            logo: Arc::new(Image::from_bytes(ImageFormat::Png, include_bytes!("../assets/logo.png").to_vec())),
            choice_subscriptions,
            _subscriptions: subscriptions,
        };

        this.load_library(cx);

        let adult = settings.store_adult;
        cx.spawn(async move |this, cx| {
            let store = cx.background_executor().spawn(async move { data::load_store(adult) }).await;
            this.update(cx, |this, cx| {
                this.store = Some(store);
                cx.notify();
            })
            .ok();
        })
        .detach();

        cx.spawn(async move |this, cx| {
            loop {
                let memory = memory_stats::memory_stats().map_or(0.0, |m| m.physical_mem as f32 / (1024.0 * 1024.0));
                let alive = this.update(cx, |this, cx| {
                    this.memory_mb = memory;
                    this.ticks += 1;
                    cx.notify();
                });
                if alive.is_err() {
                    break;
                }
                cx.background_executor().timer(Duration::from_secs(1)).await;
            }
        })
        .detach();

        this
    }

    fn slider(value: f32, cx: &mut Context<Self>) -> Entity<SliderState> {
        cx.new(|_| SliderState::new().min(0.0).max(100.0).step(1.0).default_value(value))
    }

    fn choice(items: Vec<SharedString>, selected: usize, window: &mut Window, cx: &mut Context<Self>) -> Choice {
        cx.new(|cx| SelectState::new(items, Some(IndexPath::new(selected)), window, cx))
    }

    fn selected_row(state: &Choice, cx: &App) -> Option<usize> {
        state.read(cx).selected_index(cx).map(|ix| ix.row)
    }

    fn build_choices(dark: bool, backdrop: Backdrop, window: &mut Window, cx: &mut Context<Self>) -> (Choices, Vec<Subscription>) {
        let language = Self::choice(
            vec!["Türkçe".into(), "English".into()],
            usize::from(i18n::lang() == Lang::En),
            window,
            cx,
        );
        let theme = Self::choice(vec![t("dark").into(), t("light").into()], usize::from(!dark), window, cx);
        let backdrop_choice = Self::choice(
            Backdrop::ALL.iter().map(|b| SharedString::from(b.label())).collect(),
            Backdrop::ALL.iter().position(|b| *b == backdrop).unwrap_or(0),
            window,
            cx,
        );
        let startup = Self::choice(
            vec![t("none").into(), "Normal".into(), t("priority").into(), t("scheduler").into()],
            3,
            window,
            cx,
        );
        let tray = Self::choice(vec![t("visible").into(), t("invisible").into()], 0, window, cx);
        let fullscreen = Self::choice(vec![t("pause").into(), t("close").into(), t("continue").into()], 0, window, cx);

        let subscriptions = vec![
            cx.subscribe_in(&language, window, |this, state, _: &ChoiceEvent, window, cx| {
                let lang = if Self::selected_row(state, cx) == Some(1) { Lang::En } else { Lang::Tr };
                if lang != i18n::lang() {
                    this.set_language(lang, window, cx);
                }
            }),
            cx.subscribe_in(&theme, window, |this, state, _: &ChoiceEvent, window, cx| {
                this.dark = Self::selected_row(state, cx) != Some(1);
                this.apply_look(window, cx);
            }),
            cx.subscribe_in(&backdrop_choice, window, |this, state, _: &ChoiceEvent, window, cx| {
                if let Some(row) = Self::selected_row(state, cx) {
                    this.backdrop = Backdrop::ALL[row.min(Backdrop::ALL.len() - 1)];
                    this.apply_look(window, cx);
                }
            }),
        ];

        (
            Choices { language, theme, backdrop: backdrop_choice, startup, tray, fullscreen },
            subscriptions,
        )
    }

    fn set_language(&mut self, lang: Lang, window: &mut Window, cx: &mut Context<Self>) {
        i18n::set_lang(lang);
        self.metas.clear();
        let (choices, subscriptions) = Self::build_choices(self.dark, self.backdrop, window, cx);
        self.choices = choices;
        self.choice_subscriptions = subscriptions;
        self.search.update(cx, |state, cx| state.set_placeholder(t("search"), window, cx));
        cx.notify();
    }

    fn apply_look(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.set_background_appearance(self.backdrop.appearance());
        let tint = if self.backdrop == Backdrop::Solid { 1.0 } else { self.tint.clamp(0.0, 1.0) };
        apply_theme(self.dark, tint, cx);
        cx.notify();
    }

    fn sync_theme_choice(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let row = usize::from(!self.dark);
        self.choices
            .theme
            .update(cx, |state, cx| state.set_selected_index(Some(IndexPath::new(row)), window, cx));
    }

    fn load_library(&mut self, cx: &mut Context<Self>) {
        self.library = None;
        let dir = self.library_dir.clone();
        cx.spawn(async move |this, cx| {
            let library = cx
                .background_executor()
                .spawn(async move { dir.map(|d| data::load_library(&d)).unwrap_or_default() })
                .await;
            this.update(cx, |this, cx| {
                this.library = Some(library);
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Returns the decoded thumbnail, starting a background decode the first time.
    fn thumb(&mut self, key: String, source: ThumbSource, cx: &mut Context<Self>) -> Thumb {
        if let Some(state) = self.thumbs.get(&key) {
            return state.clone();
        }
        self.thumbs.insert(key.clone(), Thumb::Loading);
        let task = cx.background_executor().spawn(async move { thumbs::load(source) });
        cx.spawn(async move |this, cx| {
            let image = task.await;
            this.update(cx, |this, cx| {
                this.thumbs.insert(key, image.map_or(Thumb::Failed, Thumb::Ready));
                cx.notify();
            })
            .ok();
        })
        .detach();
        Thumb::Loading
    }

    /// Returns the (localized) store title and description, fetching them on first use.
    fn meta(&mut self, item: &StoreItem, cx: &mut Context<Self>) -> Option<(SharedString, SharedString)> {
        match self.metas.get(&item.key) {
            Some(Meta::Ready(title, description)) => return Some((title.clone(), description.clone())),
            Some(_) => return None,
            None => {}
        }
        self.metas.insert(item.key.clone(), Meta::Loading);
        let key = item.key.clone();
        let item = item.clone();
        let culture = if i18n::lang() == Lang::Tr { "TR" } else { "EN" };
        let task = cx.background_executor().spawn(async move { data::load_store_meta(&item, culture) });
        cx.spawn(async move |this, cx| {
            let meta = task.await;
            this.update(cx, |this, cx| {
                this.metas.insert(
                    key,
                    meta.map_or(Meta::Failed, |(title, description)| Meta::Ready(title.into(), description.into())),
                );
                cx.notify();
            })
            .ok();
        })
        .detach();
        None
    }

    fn download(&mut self, key: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.downloads.contains_key(&key) {
            return;
        }
        self.downloads.insert(key.clone(), Download::Running);
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(1400)).await;
            this.update_in(cx, |this, window, cx| {
                this.downloads.insert(key, Download::Done);
                window.push_notification(Notification::success(t("download_done_note")), cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn toggle(&self, key: &str) -> bool {
        self.toggles.get(key).copied().unwrap_or(false)
    }

    // MARK: Chrome

    fn render_title_bar(&self, cx: &App) -> impl IntoElement {
        TitleBar::new().child(
            h_flex()
                .gap_2()
                .items_center()
                .child(img(self.logo.clone()).size(px(16.)))
                .child(div().text_xs().text_color(cx.theme().foreground).child("Sucrose Wallpaper Engine")),
        )
    }

    fn action(
        &self,
        id: &'static str,
        icon: Lucide,
        tooltip: &'static str,
        cx: &Context<Self>,
        handler: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Button {
        Button::new(id)
            .ghost()
            .icon(Icon::new(icon))
            .tooltip(tooltip)
            .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| handler(this, window, cx)))
    }

    fn more_menu(&self, cx: &Context<Self>) -> impl IntoElement {
        let this = cx.entity().downgrade();
        Button::new("more")
            .ghost()
            .icon(Icon::new(Lucide::EllipsisVertical))
            .tooltip(t("more"))
            .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, _| {
                let refresh = {
                    let this = this.clone();
                    move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                        this.update(cx, |this, cx| this.load_library(cx)).ok();
                    }
                };
                let soon = |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                    window.push_notification(Notification::info(t("not_in_prototype")), cx)
                };
                menu.item(PopupMenuItem::new(t("cycling")).icon(Icon::new(Lucide::RefreshCw)).on_click(soon))
                    .item(PopupMenuItem::new(t("refresh_library")).icon(Icon::new(Lucide::RotateCcw)).on_click(refresh))
                    .item(PopupMenuItem::new(t("report_bug")).icon(Icon::new(Lucide::Bug)).on_click(soon))
                    .separator()
                    .item(
                        PopupMenuItem::new("GitHub")
                            .icon(Icon::new(Lucide::Github))
                            .on_click(|_, _, cx| cx.open_url("https://github.com/Taiizor/Sucrose")),
                    )
            })
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let in_settings = self.page == Page::Settings;
        let left: AnyElement = if in_settings {
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    Button::new("back")
                        .ghost()
                        .icon(Icon::new(Lucide::ArrowLeft))
                        .tooltip(t("back"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.page = this.previous_page;
                            cx.notify();
                        })),
                )
                .child(div().text_base().font_weight(FontWeight::SEMIBOLD).child(t("settings")))
                .into_any_element()
        } else {
            TabBar::new("navigation")
                .underline()
                .selected_index(usize::from(self.page == Page::Store))
                .on_click(cx.listener(|this, ix: &usize, _, cx| {
                    this.page = if *ix == 1 { Page::Store } else { Page::Library };
                    cx.notify();
                }))
                .child(Tab::new().label(t("library")).prefix(Icon::new(Lucide::LibraryBig).small()))
                .child(Tab::new().label(t("store")).prefix(Icon::new(Lucide::Store).small()))
                .into_any_element()
        };

        let playing = self.playing;
        let actions = h_flex()
            .gap_0p5()
            .items_center()
            .child(self.action(
                "toggle-theme",
                if self.dark { Lucide::Sun } else { Lucide::Moon },
                t("toggle_theme"),
                cx,
                |this, window, cx| {
                    this.dark = !this.dark;
                    this.sync_theme_choice(window, cx);
                    this.apply_look(window, cx);
                },
            ))
            .child(self.action("create", Lucide::Plus, t("create"), cx, |_, window, cx| {
                window.push_notification(Notification::info(t("not_in_prototype")), cx)
            }))
            .child(self.action(
                "play",
                if playing { Lucide::Pause } else { Lucide::Play },
                if playing { t("pause_wallpaper") } else { t("play_wallpaper") },
                cx,
                |this, _, cx| {
                    this.playing = !this.playing;
                    cx.notify();
                },
            ))
            .child(self.action("display", Lucide::MonitorCog, t("display"), cx, |_, window, cx| {
                window.push_notification(Notification::info(t("not_in_prototype")), cx)
            }))
            .child(self.more_menu(cx))
            .child(div().w(px(1.)).h(px(20.)).mx_1().bg(cx.theme().border))
            .child(self.action("settings", Lucide::Settings, t("settings"), cx, |this, _, cx| {
                if this.page != Page::Settings {
                    this.previous_page = this.page;
                    this.page = Page::Settings;
                }
                cx.notify();
            }));

        h_flex()
            .w_full()
            .h(px(52.))
            .flex_shrink_0()
            .px_3()
            .gap_3()
            .items_center()
            .child(div().flex_1().min_w_0().child(left))
            .child(
                div()
                    .w(px(380.))
                    .flex_shrink_0()
                    .when(in_settings, |this| this.invisible())
                    .child(Input::new(&self.search).prefix(Icon::new(Lucide::Search).small()).cleanable(true)),
            )
            .child(h_flex().flex_1().justify_end().child(actions))
    }

    fn render_status_bar(&self, cx: &App) -> impl IntoElement {
        let theme = cx.theme();
        let arch = match std::env::consts::ARCH {
            "x86_64" => "x64",
            "x86" => "x86",
            "aarch64" => "ARM64",
            other => other,
        };

        h_flex()
            .w_full()
            .h(px(28.))
            .flex_shrink_0()
            .px_3()
            .gap_4()
            .items_center()
            .text_xs()
            .text_color(theme.muted_foreground)
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.status_bar)
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        h_flex()
                            .gap_1()
                            .items_center()
                            .child(Icon::new(Lucide::FlaskConical).xsmall())
                            .child(t("prerelease")),
                    )
                    .child("GPUI 0.3")
                    .child(arch)
                    .child(concat!("v", env!("CARGO_PKG_VERSION")))
                    .child(
                        h_flex()
                            .gap_1()
                            .items_center()
                            .child(Icon::new(Lucide::MemoryStick).xsmall())
                            .child(format!("{:.1} MB", self.memory_mb)),
                    ),
            )
            .child(div().flex_1().min_w_0().text_center().truncate().child(i18n::quote(self.ticks / 20)))
            .child(div().flex_shrink_0().child(t("copyright")))
    }

    // MARK: Library

    fn render_library(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(library) = self.library.as_ref() else {
            return render_loading(window);
        };
        let query = self.query.clone();
        let items: Vec<Wallpaper> = library
            .iter()
            .filter(|w| matches_query(&query, &[&w.title, &w.description]))
            .cloned()
            .collect();
        if items.is_empty() {
            return if query.is_empty() {
                empty_state(Lucide::LibraryBig, t("library_empty"), cx)
            } else {
                empty_state(Lucide::Search, t("no_results"), cx)
            };
        }

        let pages = items.len().div_ceil(PAGE_SIZE);
        self.library_page = self.library_page.min(pages - 1);
        let page = self.library_page;
        let (columns, height) = grid_metrics(window.viewport_size().width / px(1.0) - 24.0);
        let cards: Vec<AnyElement> = items
            .into_iter()
            .skip(page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .enumerate()
            .map(|(ix, wallpaper)| self.library_card(ix, wallpaper, height, cx))
            .collect();

        paged_grid(
            "library",
            columns,
            cards,
            page,
            pages,
            cx.listener(|this, page: &usize, _, cx| {
                this.library_page = page.saturating_sub(1);
                cx.notify();
            }),
        )
    }

    fn library_card(&mut self, ix: usize, wallpaper: Wallpaper, height: f32, cx: &mut Context<Self>) -> AnyElement {
        let thumb = self.thumb(
            wallpaper.thumbnail.to_string_lossy().into_owned(),
            ThumbSource::File(wallpaper.thumbnail.clone()),
            cx,
        );
        let active = self.selected.as_deref() == Some(wallpaper.id.as_str());
        let preview = (self.toggle("library_preview") && self.hovered.as_deref() == Some(wallpaper.id.as_str()))
            .then(|| wallpaper.preview.clone())
            .flatten();
        let menu = self.library_menu(ix, &wallpaper, cx);
        let select_id = wallpaper.id.clone();
        let hover_id = wallpaper.id.clone();

        card_shell(SharedString::from(wallpaper.id.clone()), height, active, cx)
            .child(card_image(thumb, cx))
            .when_some(preview, |this, path| {
                this.child(img(path).absolute().inset_0().size_full().object_fit(ObjectFit::Cover))
            })
            .child(kind_badge(wallpaper.kind))
            .when(active, |this| this.child(active_badge(cx)))
            .child(card_footer(wallpaper.title.into(), wallpaper.description.into(), menu))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected = Some(select_id.clone());
                cx.notify();
            }))
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                if *hovered {
                    this.hovered = Some(hover_id.clone());
                } else if this.hovered.as_deref() == Some(hover_id.as_str()) {
                    this.hovered = None;
                }
                cx.notify();
            }))
            .into_any_element()
    }

    fn library_menu(&self, ix: usize, wallpaper: &Wallpaper, cx: &Context<Self>) -> AnyElement {
        let this = cx.entity().downgrade();
        let id = wallpaper.id.clone();
        let folder = wallpaper.folder.clone();

        div()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                Button::new(("library-menu", ix))
                    .ghost()
                    .small()
                    .icon(Icon::new(Lucide::Ellipsis).text_color(white()))
                    .dropdown_menu(move |menu, _, _| {
                        let use_it = {
                            let this = this.clone();
                            let id = id.clone();
                            move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                                this.update(cx, |this, cx| {
                                    this.selected = Some(id.clone());
                                    cx.notify();
                                })
                                .ok();
                            }
                        };
                        let reveal = {
                            let folder = folder.clone();
                            move |_: &ClickEvent, _: &mut Window, cx: &mut App| cx.reveal_path(&folder)
                        };
                        let remove = {
                            let this = this.clone();
                            let id = id.clone();
                            move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                                this.update(cx, |this, cx| {
                                    if let Some(library) = this.library.as_mut() {
                                        library.retain(|w| w.id != id);
                                    }
                                    cx.notify();
                                })
                                .ok();
                                window.push_notification(Notification::info(t("removed_note")), cx);
                            }
                        };
                        let soon = |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                            window.push_notification(Notification::info(t("not_in_prototype")), cx)
                        };

                        menu.item(PopupMenuItem::new(t("use")).icon(Icon::new(Lucide::CirclePlay)).on_click(use_it))
                            .item(PopupMenuItem::new(t("customize")).icon(Icon::new(Lucide::SlidersHorizontal)).on_click(soon))
                            .item(PopupMenuItem::new(t("edit")).icon(Icon::new(Lucide::Pencil)).on_click(soon))
                            .item(PopupMenuItem::new(t("show_in_folder")).icon(Icon::new(Lucide::FolderOpen)).on_click(reveal))
                            .item(PopupMenuItem::new(t("share")).icon(Icon::new(Lucide::Share2)).on_click(soon))
                            .item(PopupMenuItem::new(t("report")).icon(Icon::new(Lucide::Flag)).on_click(soon))
                            .separator()
                            .item(PopupMenuItem::new(t("delete")).icon(Icon::new(Lucide::Trash)).on_click(remove))
                    }),
            )
            .into_any_element()
    }

    // MARK: Store

    fn render_store(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(store) = self.store.as_ref() else {
            return render_loading(window);
        };
        if store.is_empty() {
            return empty_state(Lucide::Store, t("store_unavailable"), cx);
        }

        let total: usize = store.iter().map(|c| c.items.len()).sum();
        let mut categories: Vec<(SharedString, Lucide, usize)> = vec![(t("all").into(), Lucide::House, total)];
        categories.extend(
            store
                .iter()
                .map(|c| (SharedString::from(c.name.clone()), category_icon(&c.name), c.items.len())),
        );
        let selected_category = self.store_category.min(categories.len() - 1);
        let query = self.query.clone();
        let filter = |item: &&StoreItem| matches_query(&query, &[&item.key]);
        let items: Vec<StoreItem> = if selected_category == 0 {
            store.iter().flat_map(|c| c.items.iter()).filter(filter).cloned().collect()
        } else {
            store[selected_category - 1].items.iter().filter(filter).cloned().collect()
        };
        self.store_category = selected_category;

        let navigation = v_flex()
            .id("store-categories")
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex_shrink_0()
            .p_2()
            .gap_0p5()
            .overflow_y_scrollbar()
            .children(categories.into_iter().enumerate().map(|(ix, (label, icon, count))| {
                nav_item(("store-category", ix), icon, label, Some(count), ix == selected_category, cx).on_click(
                    cx.listener(move |this, _, _, cx| {
                        this.store_category = ix;
                        this.store_page = 0;
                        cx.notify();
                    }),
                )
            }));

        let pages = items.len().div_ceil(PAGE_SIZE).max(1);
        self.store_page = self.store_page.min(pages - 1);
        let page = self.store_page;
        let (columns, height) = grid_metrics(window.viewport_size().width / px(1.0) - SIDEBAR_WIDTH - 24.0);
        let cards: Vec<AnyElement> = items
            .into_iter()
            .skip(page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .enumerate()
            .map(|(ix, item)| self.store_card(ix, item, height, cx))
            .collect();

        let grid = if cards.is_empty() {
            empty_state(Lucide::Search, t("no_results"), cx)
        } else {
            paged_grid(
                "store",
                columns,
                cards,
                page,
                pages,
                cx.listener(|this, page: &usize, _, cx| {
                    this.store_page = page.saturating_sub(1);
                    cx.notify();
                }),
            )
        };

        h_flex()
            .size_full()
            .child(navigation)
            .child(div().flex_1().h_full().min_w_0().child(grid))
            .into_any_element()
    }

    fn store_card(&mut self, ix: usize, item: StoreItem, height: f32, cx: &mut Context<Self>) -> AnyElement {
        let thumb = self.thumb(
            item.cover_url(),
            ThumbSource::Store { cache: item.cache.join(&item.cover), url: item.cover_url() },
            cx,
        );
        let (title, description) = self
            .meta(&item, cx)
            .unwrap_or_else(|| (item.fallback_title().into(), item.category.clone().into()));
        let state = self.downloads.get(&item.key).copied();
        let (icon, tooltip) = match state {
            Some(Download::Done) => (Lucide::CircleCheck, t("in_library")),
            _ => (Lucide::CloudDownload, t("download")),
        };
        let key = item.key.clone();
        let button = div()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                Button::new(("store-download", ix))
                    .ghost()
                    .icon(Icon::new(icon).text_color(white()))
                    .tooltip(tooltip)
                    .loading(state == Some(Download::Running))
                    .on_click(cx.listener(move |this, _, window, cx| this.download(key.clone(), window, cx))),
            );

        card_shell(SharedString::from(format!("store-{}", item.key)), height, false, cx)
            .child(card_image(thumb, cx))
            .child(card_footer(title, description, button.into_any_element()))
            .into_any_element()
    }

    // MARK: Settings

    fn render_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let section = self.section;
        // A definite width keeps flex children from collapsing inside the scroll area.
        let content_width = (window.viewport_size().width / px(1.0) - SIDEBAR_WIDTH - 40.0).clamp(320.0, 960.0);
        let navigation = v_flex()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex_shrink_0()
            .p_2()
            .gap_0p5()
            .children(Section::ALL.iter().enumerate().map(|(ix, item)| {
                let item = *item;
                nav_item(("settings-section", ix), item.icon(), item.label().into(), None, item == section, cx).on_click(
                    cx.listener(move |this, _, _, cx| {
                        this.section = item;
                        cx.notify();
                    }),
                )
            }));

        let cards = match section {
            Section::General => self.general_cards(cx),
            Section::Personal => self.personal_cards(cx),
            Section::Performance => self.performance_cards(cx),
            Section::Other => self.about_cards(cx),
            _ => vec![placeholder_card(cx)],
        };

        h_flex()
            .size_full()
            .child(navigation)
            .child(
                div().flex_1().h_full().min_w_0().child(
                    v_flex()
                        .id("settings-content")
                        .size_full()
                        .overflow_y_scrollbar()
                        .child(
                            v_flex()
                                .w(px(content_width + 32.0))
                                .p_4()
                                .gap_2()
                                .child(div().text_2xl().font_weight(FontWeight::SEMIBOLD).pb_2().child(section.label()))
                                .children(cards),
                        ),
                ),
            )
            .into_any_element()
    }

    fn toggle_row(&self, key: &'static str, label: &'static str, cx: &Context<Self>) -> AnyElement {
        h_flex()
            .w_full()
            .justify_between()
            .items_center()
            .gap_4()
            .child(div().text_sm().child(label))
            .child(Switch::new(key).checked(self.toggle(key)).on_click(cx.listener(move |this, checked: &bool, _, cx| {
                this.toggles.insert(key, *checked);
                cx.notify();
            })))
            .into_any_element()
    }

    fn general_cards(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        let tint = self.sliders.tint.read(cx).value().start();
        let volume = self.sliders.volume.read(cx).value().start();
        let location = self
            .library_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "-".into());

        vec![
            setting_card(Lucide::Languages, t("language"), t("language_desc"), select_box(&self.choices.language), None, cx),
            setting_card(Lucide::SwatchBook, t("theme"), t("theme_desc"), select_box(&self.choices.theme), None, cx),
            setting_card(
                Lucide::AppWindow,
                t("backdrop"),
                t("backdrop_desc"),
                select_box(&self.choices.backdrop),
                Some(labeled_slider(t("backdrop_opacity"), &self.sliders.tint, tint, cx)),
                cx,
            ),
            setting_card(Lucide::Power, t("startup"), t("startup_desc"), select_box(&self.choices.startup), None, cx),
            setting_card(
                Lucide::Monitor,
                t("tray"),
                t("tray_desc"),
                select_box(&self.choices.tray),
                Some(self.toggle_row("tray_exit", t("tray_exit"), cx)),
                cx,
            ),
            setting_card(
                Lucide::Volume2,
                t("volume"),
                t("volume_desc"),
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(Slider::new(&self.sliders.volume).w(px(200.)))
                    .child(value_label(volume, cx)),
                Some(
                    v_flex()
                        .gap_2()
                        .child(self.toggle_row("volume_active", t("volume_active"), cx))
                        .child(self.toggle_row("volume_desktop", t("volume_desktop"), cx))
                        .into_any_element(),
                ),
                cx,
            ),
            setting_card(
                Lucide::HardDrive,
                t("library_location"),
                t("library_location_desc"),
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(div().text_xs().text_color(cx.theme().muted_foreground).child(location))
                    .child(Button::new("change-library").outline().small().label(t("change")).on_click(
                        |_, window, cx| window.push_notification(Notification::info(t("not_in_prototype")), cx),
                    )),
                Some(self.toggle_row("library_move", t("library_move"), cx)),
                cx,
            ),
        ]
    }

    fn personal_cards(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        vec![setting_card(
            Lucide::Images,
            t("library_preview"),
            t("library_preview_desc"),
            Switch::new("library-preview-switch").checked(self.toggle("library_preview")).on_click(cx.listener(
                |this, checked: &bool, _, cx| {
                    this.toggles.insert("library_preview", *checked);
                    cx.notify();
                },
            )),
            None,
            cx,
        )]
    }

    fn performance_cards(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        let slider_card = |icon: Lucide, title: &'static str, description: &'static str, state: &Entity<SliderState>| {
            let value = state.read(cx).value().start();
            setting_card(
                icon,
                title,
                description,
                h_flex().gap_3().items_center().child(Slider::new(state).w(px(200.))).child(value_label(value, cx)),
                None,
                cx,
            )
        };
        let switch_card = |icon: Lucide, title: &'static str, description: &'static str, key: &'static str| {
            setting_card(
                icon,
                title,
                description,
                Switch::new(key).checked(self.toggle(key)).on_click(cx.listener(move |this, checked: &bool, _, cx| {
                    this.toggles.insert(key, *checked);
                    cx.notify();
                })),
                None,
                cx,
            )
        };

        vec![
            slider_card(Lucide::Cpu, t("cpu"), t("cpu_desc"), &self.sliders.cpu),
            slider_card(Lucide::MemoryStick, t("memory"), t("memory_desc"), &self.sliders.memory),
            slider_card(Lucide::Battery, t("battery"), t("battery_desc"), &self.sliders.battery),
            setting_card(Lucide::Fullscreen, t("fullscreen"), t("fullscreen_desc"), select_box(&self.choices.fullscreen), None, cx),
            switch_card(Lucide::MonitorSmartphone, t("remote"), t("remote_desc"), "remote"),
            switch_card(Lucide::BatteryLow, t("saver"), t("saver_desc"), "saver"),
        ]
    }

    fn about_cards(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        let theme = cx.theme();
        let row = |label: &'static str, value: String| {
            h_flex()
                .w_full()
                .justify_between()
                .text_sm()
                .child(div().text_color(theme.muted_foreground).child(label))
                .child(value)
        };
        let details = v_flex()
            .gap_1()
            .child(row(t("about_ui"), "GPUI 0.3.12 + gpui-kit 0.10".into()))
            .child(row(t("about_language"), "Rust".into()))
            .child(row(t("about_platform"), format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)))
            .child(row(t("about_memory"), format!("{:.1} MB", self.memory_mb)))
            .into_any_element();

        vec![setting_card(
            Lucide::Info,
            t("about"),
            t("about_desc"),
            Button::new("github")
                .outline()
                .small()
                .icon(Icon::new(Lucide::Github))
                .label("GitHub")
                .on_click(|_, _, cx| cx.open_url("https://github.com/Taiizor/Sucrose")),
            Some(details),
            cx,
        )]
    }
}

impl Render for Portal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let background = cx.theme().background;
        let foreground = cx.theme().foreground;
        let content = match self.page {
            Page::Library => self.render_library(window, cx),
            Page::Store => self.render_store(window, cx),
            Page::Settings => self.render_settings(window, cx),
        };

        v_flex()
            .size_full()
            .bg(background)
            .text_color(foreground)
            .child(self.render_title_bar(cx))
            .child(self.render_header(cx))
            .child(div().flex_1().min_h_0().w_full().child(content))
            .child(self.render_status_bar(cx))
    }
}

// MARK: Helpers

fn matches_query(query: &str, fields: &[&str]) -> bool {
    query.is_empty() || fields.iter().any(|field| field.to_lowercase().contains(query))
}

/// Number of columns and card height for the available width.
fn grid_metrics(width: f32) -> (u16, f32) {
    let width = width.max(MIN_CARD_WIDTH);
    let columns = ((width + GAP) / (MIN_CARD_WIDTH + GAP)).floor().clamp(1.0, 6.0);
    let card_width = (width - GAP * (columns - 1.0)) / columns;
    (columns as u16, (card_width * 9.0 / 16.0).max(140.0))
}

fn render_loading(window: &Window) -> AnyElement {
    let (columns, height) = grid_metrics(window.viewport_size().width / px(1.0) - 24.0);
    div()
        .size_full()
        .p_3()
        .child(
            div()
                .grid()
                .grid_cols(columns)
                .gap_3()
                .children((0..usize::from(columns) * 3).map(|_| Skeleton::new().w_full().h(px(height)).rounded_lg())),
        )
        .into_any_element()
}

fn paged_grid(
    id: &'static str,
    columns: u16,
    cards: Vec<AnyElement>,
    page: usize,
    pages: usize,
    on_page: impl Fn(&usize, &mut Window, &mut App) + 'static,
) -> AnyElement {
    v_flex()
        .id(id)
        .size_full()
        .overflow_y_scrollbar()
        .child(
            v_flex()
                .w_full()
                .p_3()
                .gap_3()
                .child(div().grid().grid_cols(columns).gap_3().children(cards))
                .when(pages > 1, |this| {
                    this.child(
                        h_flex().w_full().justify_center().pb_2().child(
                            Pagination::new(SharedString::from(format!("{id}-pages")))
                                .current_page(page + 1)
                                .total_pages(pages)
                                .on_click(on_page),
                        ),
                    )
                }),
        )
        .into_any_element()
}

fn empty_state(icon: Lucide, text: &'static str, cx: &App) -> AnyElement {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_3()
        .text_color(cx.theme().muted_foreground)
        .child(Icon::new(icon).size_10())
        .child(div().text_sm().child(text))
        .into_any_element()
}

fn card_shell(id: SharedString, height: f32, active: bool, cx: &App) -> Stateful<Div> {
    let theme = cx.theme();
    let hover = theme.primary.opacity(0.55);
    div()
        .id(id)
        .relative()
        .w_full()
        .h(px(height))
        .rounded(px(8.))
        .overflow_hidden()
        .cursor_pointer()
        .bg(theme.muted)
        .border_2()
        .border_color(if active { theme.primary } else { transparent_black() })
        .when(!active, |this| this.hover(move |style| style.border_color(hover)))
}

fn card_image(thumb: Thumb, cx: &App) -> AnyElement {
    match thumb {
        Thumb::Ready(image) => img(image).size_full().object_fit(ObjectFit::Cover).into_any_element(),
        Thumb::Loading => Skeleton::new().size_full().into_any_element(),
        Thumb::Failed => div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(Icon::new(Lucide::ImageOff).large().text_color(cx.theme().muted_foreground))
            .into_any_element(),
    }
}

fn card_footer(title: SharedString, description: SharedString, trailing: AnyElement) -> Div {
    h_flex()
        .absolute()
        .left_0()
        .right_0()
        .bottom_0()
        .h(px(56.))
        .pl_3()
        .pr_1()
        .gap_2()
        .items_center()
        .bg(linear_gradient(
            180.,
            linear_color_stop(hsla(0., 0., 0., 0.3), 0.),
            linear_color_stop(hsla(0., 0., 0., 0.75), 1.),
        ))
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_color(white())
                        .text_size(px(15.))
                        .font_weight(FontWeight::BOLD)
                        .truncate()
                        .child(title),
                )
                .child(div().text_color(white().opacity(0.82)).text_xs().truncate().child(description)),
        )
        .child(trailing)
}

fn kind_badge(kind: u64) -> Div {
    let (icon, label) = match kind {
        0 => (Lucide::Image, "GIF"),
        1 => (Lucide::Link, "URL"),
        2 => (Lucide::Globe, "Web"),
        3 => (Lucide::Clapperboard, "Video"),
        4 => (Lucide::CirclePlay, "YouTube"),
        _ => (Lucide::AppWindow, "App"),
    };
    h_flex()
        .absolute()
        .top_2()
        .left_2()
        .gap_1()
        .px_2()
        .py_0p5()
        .items_center()
        .rounded_full()
        .bg(hsla(0., 0., 0., 0.55))
        .text_color(white())
        .text_xs()
        .child(Icon::new(icon).xsmall().text_color(white()))
        .child(label)
}

fn active_badge(cx: &App) -> Div {
    let theme = cx.theme();
    h_flex()
        .absolute()
        .top_2()
        .right_2()
        .gap_1()
        .px_2()
        .py_0p5()
        .items_center()
        .rounded_full()
        .bg(theme.primary)
        .text_color(theme.primary_foreground)
        .text_xs()
        .child(Icon::new(Lucide::Check).xsmall().text_color(theme.primary_foreground))
        .child(t("active"))
}

fn nav_item(
    id: impl Into<ElementId>,
    icon: Lucide,
    label: SharedString,
    count: Option<usize>,
    selected: bool,
    cx: &App,
) -> Stateful<Div> {
    let theme = cx.theme();
    let hover = theme.foreground.opacity(0.06);
    let selected_bg = theme.foreground.opacity(0.09);
    let accent = theme.primary;
    let muted = theme.muted_foreground;

    h_flex()
        .id(id)
        .relative()
        .w_full()
        .h(px(36.))
        .flex_shrink_0()
        .px_3()
        .gap_3()
        .items_center()
        .rounded_md()
        .cursor_pointer()
        .when(selected, |this| this.bg(selected_bg))
        .hover(move |style| style.bg(hover))
        .when(selected, |this| {
            this.child(div().absolute().left_0().top(px(10.)).h(px(16.)).w(px(3.)).rounded_full().bg(accent))
        })
        .child(Icon::new(icon).small())
        .child(div().flex_1().min_w_0().text_sm().truncate().child(label))
        .when_some(count, |this, count| this.child(div().text_xs().text_color(muted).child(count.to_string())))
}

fn category_icon(name: &str) -> Lucide {
    match name {
        "Abstract" => Lucide::Shapes,
        "Ambience" => Lucide::Music,
        "Animals" => Lucide::PawPrint,
        "Animation" => Lucide::Clapperboard,
        "Anime" => Lucide::Sparkles,
        "Artworks" => Lucide::Brush,
        "Black & White" => Lucide::Contrast,
        "Cartoon and Comics" => Lucide::Palette,
        "Cities and Places" => Lucide::Building2,
        "Dynamic" => Lucide::Zap,
        "Enchanted Forests" => Lucide::Leaf,
        "Fantasy" => Lucide::Castle,
        "Film & TV" => Lucide::Film,
        "Game" => Lucide::Gamepad2,
        "Holidays" => Lucide::Gift,
        "Horror" => Lucide::Ghost,
        "Landscape" => Lucide::Mountain,
        "Minimalist" => Lucide::Square,
        "Nature" => Lucide::Trees,
        "Science Fiction" => Lucide::Rocket,
        "Superheroes" => Lucide::Shield,
        "System" => Lucide::Cpu,
        "Tavern" => Lucide::Beer,
        "Time" => Lucide::Clock,
        "Vehicles" => Lucide::Car,
        _ => Lucide::LayoutGrid,
    }
}

fn setting_card(
    icon: Lucide,
    title: &'static str,
    description: &'static str,
    control: impl IntoElement,
    extra: Option<AnyElement>,
    cx: &App,
) -> AnyElement {
    let theme = cx.theme();
    let border = theme.border;

    v_flex()
        .w_full()
        .rounded_lg()
        .border_1()
        .border_color(border)
        .bg(theme.secondary.opacity(0.55))
        .px_4()
        .py_3()
        .gap_3()
        .child(
            h_flex()
                .w_full()
                .gap_4()
                .items_center()
                .child(Icon::new(icon).large())
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_0p5()
                        .child(div().text_sm().font_weight(FontWeight::MEDIUM).child(title))
                        .child(div().text_xs().text_color(theme.muted_foreground).child(description)),
                )
                .child(control),
        )
        .when_some(extra, |this, extra| {
            this.child(div().w_full().border_t_1().border_color(border).pt_3().pl(px(40.)).child(extra))
        })
        .into_any_element()
}

/// Select's own container wants the full row width, so it gets a fixed-width box.
fn select_box(state: &Choice) -> Div {
    div().w(px(200.)).flex_shrink_0().child(Select::new(state))
}

fn labeled_slider(label: &'static str, state: &Entity<SliderState>, value: f32, cx: &App) -> AnyElement {
    h_flex()
        .w_full()
        .justify_between()
        .items_center()
        .gap_4()
        .child(div().text_sm().child(label))
        .child(
            h_flex()
                .gap_3()
                .items_center()
                .child(Slider::new(state).w(px(220.)))
                .child(value_label(value, cx)),
        )
        .into_any_element()
}

fn value_label(value: f32, cx: &App) -> Div {
    div()
        .w(px(40.))
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(format!("{value:.0}%"))
}

fn placeholder_card(cx: &App) -> AnyElement {
    let theme = cx.theme();
    h_flex()
        .w_full()
        .rounded_lg()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary.opacity(0.4))
        .p_4()
        .gap_3()
        .items_center()
        .text_sm()
        .text_color(theme.muted_foreground)
        .child(Icon::new(Lucide::Info))
        .child(t("section_soon"))
        .into_any_element()
}
