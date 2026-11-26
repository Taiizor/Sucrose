//! The engine identities, the availability report and the pure choice of the engine that plays a wallpaper type
//! (D-10, D-11, ENG-07). The settings keep the legacy engine names; only this module turns them into engines.

use sucrose_core::{settings::EngineType, theme::WallpaperType};

/// An engine Sucrose can play a wallpaper with (D-10). Several legacy names can map to one engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EngineId {
    /// libmpv, loaded at runtime (`MpvPlayerLive`).
    Mpv,
    /// libVLC, loaded at runtime (`VlcPlayerLive`).
    Vlc,
    /// The bundled Chromium Embedded Framework (`CefSharpLive`).
    Cef,
    /// The WebView2 runtime (`WebViewLive`).
    WebView2,
    /// The built-in GIF renderer (`VexanaLive` and `XavierLive`).
    BuiltInGif,
    /// The built-in video player of the operating system (`NebulaLive`).
    BuiltInVideo,
    /// The application embedder (`AuroraLive`).
    AppEmbedder,
}

impl EngineId {
    /// The engine a legacy engine name stands for (D-10); the stored name never changes.
    pub fn from_legacy(engine: EngineType) -> Self {
        match engine {
            EngineType::VexanaLive | EngineType::XavierLive => Self::BuiltInGif,
            EngineType::NebulaLive => Self::BuiltInVideo,
            EngineType::AuroraLive => Self::AppEmbedder,
            EngineType::WebViewLive => Self::WebView2,
            EngineType::CefSharpLive => Self::Cef,
            EngineType::MpvPlayerLive => Self::Mpv,
            EngineType::VlcPlayerLive => Self::Vlc,
        }
    }

    /// The one canonical legacy name the Portal writes for this engine (D-10): `XavierLive` for the built-in GIF
    /// renderer, `NebulaLive` for the built-in video player.
    pub fn to_legacy(self) -> EngineType {
        match self {
            Self::Mpv => EngineType::MpvPlayerLive,
            Self::Vlc => EngineType::VlcPlayerLive,
            Self::Cef => EngineType::CefSharpLive,
            Self::WebView2 => EngineType::WebViewLive,
            Self::BuiltInGif => EngineType::XavierLive,
            Self::BuiltInVideo => EngineType::NebulaLive,
            Self::AppEmbedder => EngineType::AuroraLive,
        }
    }

    /// Whether this engine can play a wallpaper of the given type (the legacy allowed lists, D-10). The web engines
    /// also play GIF and video themes as generated pages.
    pub fn supports(self, kind: WallpaperType) -> bool {
        match self {
            Self::Mpv | Self::Vlc => matches!(kind, WallpaperType::Gif | WallpaperType::Video),
            Self::Cef | Self::WebView2 => !matches!(kind, WallpaperType::Application),
            Self::BuiltInGif => kind == WallpaperType::Gif,
            Self::BuiltInVideo => kind == WallpaperType::Video,
            Self::AppEmbedder => kind == WallpaperType::Application,
        }
    }
}

/// What the user can do about an engine that is not usable (ENG-06): the repair hint of an availability report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hint {
    /// The engine's files are not next to the executable; reinstall or repair Sucrose.
    FilesMissing,
    /// The Visual C++ Redistributable is missing; `needed` names the version to install.
    VcRedistMissing {
        /// The version the engine needs.
        needed: String,
    },
    /// A user-installed runtime (WebView2) is not installed.
    RuntimeMissing,
    /// A user-installed runtime is older than the engine needs.
    RuntimeOutdated {
        /// The version found.
        found: String,
        /// The version needed.
        needed: String,
    },
}

/// Why an engine has no build for this machine (D-027).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsupported {
    /// No build exists for this processor architecture.
    Architecture,
    /// This build of Sucrose does not include the engine.
    NotInThisBuild,
}

/// What the machine can run of one engine (D-028): the report the choice works from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Availability {
    /// Present and loadable.
    Installed {
        /// The engine's version when it reports one.
        version: Option<String>,
    },
    /// Not present.
    Missing {
        /// How to get it.
        hint: Hint,
    },
    /// Present but it cannot be loaded.
    NotLoadable {
        /// The loader's reason.
        reason: String,
        /// How to repair it.
        hint: Hint,
    },
    /// No build exists for this machine or this Sucrose build.
    Unsupported {
        /// Why.
        why: Unsupported,
    },
}

impl Availability {
    /// Whether the engine can play now: only `Installed`.
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Installed { .. })
    }
}

/// The outcome of [`choose_engine`] (D-11, ENG-07): the engine that plays, and when it is not the configured one,
/// the engine it fell back from and why. The setting itself is never touched; a fallback applies to the current run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    /// The engine to start, or `None` when no engine for the type is available, so the service can refuse the
    /// theme with a reason instead of starting a renderer that fails.
    pub engine: Option<EngineId>,
    /// The configured engine and its availability when it could not be used.
    pub fell_back_from: Option<(EngineId, Availability)>,
}

/// The legacy allowed engines of a wallpaper type (`GifEngineType` and the others).
fn allowed(kind: WallpaperType) -> &'static [EngineType] {
    match kind {
        WallpaperType::Gif => EngineType::GIF,
        WallpaperType::Url => EngineType::URL,
        WallpaperType::Web => EngineType::WEB,
        WallpaperType::Video => EngineType::VIDEO,
        WallpaperType::YouTube => EngineType::YOUTUBE,
        WallpaperType::Application => EngineType::APPLICATION,
    }
}

/// The legacy default engine of a wallpaper type (D-10, D-024): CEF for web types, mpv for Gif and Video, the
/// embedder for Application.
fn legacy_default(kind: WallpaperType) -> EngineType {
    match kind {
        WallpaperType::Gif | WallpaperType::Video => EngineType::MpvPlayerLive,
        WallpaperType::Url | WallpaperType::Web | WallpaperType::YouTube => EngineType::CefSharpLive,
        WallpaperType::Application => EngineType::AuroraLive,
    }
}

/// The engines that can stand in for the configured one, in the order they are tried (D-11, discretion, research
/// Pattern 1). The configured engine always comes first and is skipped here when it appears again.
///
/// - Web, Url, YouTube: the other web engine (CEF, then WebView2).
/// - Gif: built-in GIF, mpv, libVLC, then a web engine playing a generated page.
/// - Video: mpv, built-in video, libVLC, then a web engine.
/// - Application: the embedder only; an application is never faked by another engine.
pub fn fallback_order(kind: WallpaperType) -> &'static [EngineId] {
    use EngineId::{AppEmbedder, BuiltInGif, BuiltInVideo, Cef, Mpv, Vlc, WebView2};
    match kind {
        WallpaperType::Url | WallpaperType::Web | WallpaperType::YouTube => &[Cef, WebView2],
        WallpaperType::Gif => &[BuiltInGif, Mpv, Vlc, Cef, WebView2],
        WallpaperType::Video => &[Mpv, BuiltInVideo, Vlc, Cef, WebView2],
        WallpaperType::Application => &[AppEmbedder],
    }
}

/// Picks the engine that plays a wallpaper of `kind` (D-11, ENG-07): the configured engine when `availability`
/// says it is installed, otherwise the first available engine of [`fallback_order`]. A configured engine outside
/// the type's legacy allowed list (a hand-edited file) is treated as the type's legacy default first (T-04-11). The
/// configured setting is never changed; the caller applies the choice to the current run only.
pub fn choose_engine(
    kind: WallpaperType,
    configured: EngineType,
    availability: &dyn Fn(EngineId) -> Availability,
) -> Choice {
    // T-04-11: an engine the type does not allow counts as the type's default.
    let configured = if allowed(kind).contains(&configured) {
        configured
    } else {
        legacy_default(kind)
    };
    let wanted = EngineId::from_legacy(configured);

    let report = availability(wanted);
    if report.is_available() {
        return Choice {
            engine: Some(wanted),
            fell_back_from: None,
        };
    }
    let engine = fallback_order(kind)
        .iter()
        .copied()
        .filter(|candidate| *candidate != wanted)
        .find(|candidate| availability(*candidate).is_available());
    Choice {
        engine,
        fell_back_from: Some((wanted, report)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KINDS: [WallpaperType; 6] = [
        WallpaperType::Gif,
        WallpaperType::Url,
        WallpaperType::Web,
        WallpaperType::Video,
        WallpaperType::YouTube,
        WallpaperType::Application,
    ];

    fn missing() -> Availability {
        Availability::Missing {
            hint: Hint::FilesMissing,
        }
    }

    fn installed() -> Availability {
        Availability::Installed { version: None }
    }

    /// An availability report where exactly the listed engines are installed.
    fn only(present: &'static [EngineId]) -> impl Fn(EngineId) -> Availability {
        move |engine| {
            if present.contains(&engine) {
                installed()
            } else {
                missing()
            }
        }
    }

    #[test]
    fn choose_web_keeps_the_configured_cef_when_it_is_installed() {
        let choice = choose_engine(
            WallpaperType::Web,
            EngineType::CefSharpLive,
            &only(&[EngineId::Cef, EngineId::WebView2]),
        );
        assert_eq!(
            choice,
            Choice {
                engine: Some(EngineId::Cef),
                fell_back_from: None
            }
        );
    }

    #[test]
    fn choose_web_falls_back_to_webview2_and_names_the_missing_cef() {
        let choice = choose_engine(
            WallpaperType::Web,
            EngineType::CefSharpLive,
            &only(&[EngineId::WebView2]),
        );
        assert_eq!(choice.engine, Some(EngineId::WebView2));
        assert_eq!(choice.fell_back_from, Some((EngineId::Cef, missing())));
    }

    #[test]
    fn choose_web_falls_back_to_cef_when_webview2_is_not_in_this_build() {
        let report = |engine| match engine {
            EngineId::WebView2 => Availability::Unsupported {
                why: Unsupported::NotInThisBuild,
            },
            EngineId::Cef => installed(),
            _ => missing(),
        };
        let choice = choose_engine(WallpaperType::Web, EngineType::WebViewLive, &report);
        assert_eq!(choice.engine, Some(EngineId::Cef));
        assert_eq!(
            choice.fell_back_from,
            Some((
                EngineId::WebView2,
                Availability::Unsupported {
                    why: Unsupported::NotInThisBuild
                }
            ))
        );
    }

    #[test]
    fn choose_url_and_youtube_use_the_web_order_too() {
        for kind in [WallpaperType::Url, WallpaperType::YouTube] {
            let choice = choose_engine(kind, EngineType::CefSharpLive, &only(&[EngineId::WebView2]));
            assert_eq!(choice.engine, Some(EngineId::WebView2), "{kind:?}");
        }
    }

    #[test]
    fn choose_video_walks_mpv_built_in_vlc_then_a_web_engine_then_nothing() {
        let not_loadable = Availability::NotLoadable {
            reason: "no libmpv".into(),
            hint: Hint::FilesMissing,
        };
        let report = |present: &'static [EngineId]| {
            let not_loadable = not_loadable.clone();
            move |engine| {
                if present.contains(&engine) {
                    installed()
                } else if engine == EngineId::Mpv {
                    not_loadable.clone()
                } else {
                    missing()
                }
            }
        };
        let walk = |present: &'static [EngineId]| {
            choose_engine(WallpaperType::Video, EngineType::MpvPlayerLive, &report(present))
        };

        let all = walk(&[EngineId::BuiltInVideo, EngineId::Vlc, EngineId::Cef]);
        assert_eq!(all.engine, Some(EngineId::BuiltInVideo));
        assert_eq!(all.fell_back_from, Some((EngineId::Mpv, not_loadable.clone())));
        assert_eq!(walk(&[EngineId::Vlc, EngineId::Cef]).engine, Some(EngineId::Vlc));
        assert_eq!(walk(&[EngineId::WebView2]).engine, Some(EngineId::WebView2));
        assert_eq!(walk(&[]).engine, None);
    }

    #[test]
    fn choose_gif_uses_the_built_in_renderer_for_vexana_and_honours_cef() {
        let everything = only(&[EngineId::BuiltInGif, EngineId::Mpv, EngineId::Cef]);
        let vexana = choose_engine(WallpaperType::Gif, EngineType::VexanaLive, &everything);
        assert_eq!(
            vexana,
            Choice {
                engine: Some(EngineId::BuiltInGif),
                fell_back_from: None
            }
        );
        let cef = choose_engine(WallpaperType::Gif, EngineType::CefSharpLive, &everything);
        assert_eq!(
            cef,
            Choice {
                engine: Some(EngineId::Cef),
                fell_back_from: None
            }
        );
    }

    #[test]
    fn choose_gif_falls_back_from_the_built_in_renderer_to_mpv_before_a_web_engine() {
        let choice = choose_engine(
            WallpaperType::Gif,
            EngineType::XavierLive,
            &only(&[EngineId::Mpv, EngineId::Cef]),
        );
        assert_eq!(choice.engine, Some(EngineId::Mpv));
        assert_eq!(choice.fell_back_from, Some((EngineId::BuiltInGif, missing())));
    }

    #[test]
    fn choose_application_is_the_embedder_or_nothing_never_another_engine() {
        let embedder = choose_engine(
            WallpaperType::Application,
            EngineType::AuroraLive,
            &only(&[EngineId::AppEmbedder]),
        );
        assert_eq!(
            embedder,
            Choice {
                engine: Some(EngineId::AppEmbedder),
                fell_back_from: None
            }
        );
        let others = only(&[
            EngineId::Cef,
            EngineId::WebView2,
            EngineId::Mpv,
            EngineId::Vlc,
            EngineId::BuiltInVideo,
        ]);
        let none = choose_engine(WallpaperType::Application, EngineType::AuroraLive, &others);
        assert_eq!(
            none,
            Choice {
                engine: None,
                fell_back_from: Some((EngineId::AppEmbedder, missing()))
            }
        );
    }

    #[test]
    fn choose_replaces_a_configured_engine_outside_the_allowed_list_with_the_type_default() {
        // A hand-edited file names mpv for a web theme: the type's default, CEF, is used and no fallback is reported.
        let web = choose_engine(
            WallpaperType::Web,
            EngineType::MpvPlayerLive,
            &only(&[EngineId::Cef, EngineId::Mpv]),
        );
        assert_eq!(
            web,
            Choice {
                engine: Some(EngineId::Cef),
                fell_back_from: None
            }
        );
        // ... and an application theme can never end up on a web engine.
        let app = choose_engine(
            WallpaperType::Application,
            EngineType::CefSharpLive,
            &only(&[EngineId::Cef]),
        );
        assert_eq!(app.engine, None);
    }

    #[test]
    fn choose_with_no_engine_still_names_what_it_fell_back_from() {
        let choice = choose_engine(WallpaperType::Web, EngineType::WebViewLive, &|_| missing());
        assert_eq!(
            choice,
            Choice {
                engine: None,
                fell_back_from: Some((EngineId::WebView2, missing()))
            }
        );
    }

    #[test]
    fn choose_fallback_order_lists_only_engines_that_support_the_type() {
        for kind in KINDS {
            for engine in fallback_order(kind) {
                assert!(engine.supports(kind), "{engine:?} listed for {kind:?}");
            }
        }
    }

    #[test]
    fn choose_never_returns_an_engine_that_does_not_support_the_type() {
        let everything = |_| installed();
        for kind in KINDS {
            for configured in EngineType::ALL {
                let engine = choose_engine(kind, *configured, &everything).engine;
                assert!(
                    engine.is_some_and(|engine| engine.supports(kind)),
                    "{configured:?} for {kind:?}"
                );
            }
        }
    }

    #[test]
    fn legacy_names_round_trip_for_every_canonical_engine() {
        for engine in [
            EngineId::Mpv,
            EngineId::Vlc,
            EngineId::Cef,
            EngineId::WebView2,
            EngineId::BuiltInGif,
            EngineId::BuiltInVideo,
            EngineId::AppEmbedder,
        ] {
            assert_eq!(EngineId::from_legacy(engine.to_legacy()), engine);
        }
    }

    #[test]
    fn legacy_xavier_and_vexana_both_map_to_the_built_in_gif_renderer() {
        assert_eq!(EngineId::from_legacy(EngineType::VexanaLive), EngineId::BuiltInGif);
        assert_eq!(EngineId::from_legacy(EngineType::XavierLive), EngineId::BuiltInGif);
        assert_eq!(EngineId::BuiltInGif.to_legacy(), EngineType::XavierLive);
        assert_eq!(EngineId::from_legacy(EngineType::NebulaLive), EngineId::BuiltInVideo);
        assert_eq!(EngineId::BuiltInVideo.to_legacy(), EngineType::NebulaLive);
        assert_eq!(EngineId::from_legacy(EngineType::AuroraLive), EngineId::AppEmbedder);
    }

    #[test]
    fn legacy_defaults_are_inside_each_types_allowed_list() {
        for kind in KINDS {
            assert!(allowed(kind).contains(&legacy_default(kind)), "{kind:?}");
        }
    }

    #[test]
    fn legacy_allowed_lists_map_only_to_engines_that_support_the_type() {
        for kind in KINDS {
            for name in allowed(kind) {
                assert!(EngineId::from_legacy(*name).supports(kind), "{name:?} for {kind:?}");
            }
        }
    }
}
