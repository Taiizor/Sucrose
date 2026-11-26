//! What this machine can play with, and the fallbacks that run (ENG-06, ENG-07, D-11, D-15, D-028).
//!
//! The service decides from files and registry values only: it never loads an engine library, because only renderers
//! load engines (D-028) and the service stays light (D-008). The report is built at service start and again when a
//! Portal asks; a fallback applies to the current run, the configured engine stays in the settings (D-11).

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use sucrose_core::{settings::EngineType, theme::WallpaperType};
use sucrose_engine::{
    Availability, EngineId, Hint, Unsupported, VC_REDIST_MIN, choose_engine, fallback_order, version_at_least,
};
use sucrose_ipc::portal::{EngineReportView, EngineState, EngineStateView, FallbackView, HintView};
use sucrose_platform::{Arch, Platform, VcRedist};

/// Every engine the report covers, in the order the Portal lists them.
pub const ALL: [EngineId; 7] = [
    EngineId::Mpv,
    EngineId::Vlc,
    EngineId::Cef,
    EngineId::WebView2,
    EngineId::BuiltInGif,
    EngineId::BuiltInVideo,
    EngineId::AppEmbedder,
];

/// What a Use or a start says when no engine can play the wallpaper's type (UI-SPEC, Use-disabled reason 4).
pub const NO_ENGINE: &str = "No installed engine can play this wallpaper.";

/// The files CEF needs next to the executable (D-028): the library, its Chromium helper, the data files and the
/// V8 snapshot.
const CEF_FILES: [&str; 5] = [
    "libcef.dll",
    "chrome_elf.dll",
    "icudtl.dat",
    "resources.pak",
    "v8_context_snapshot.bin",
];

/// The folders CEF needs next to the executable.
const CEF_DIRS: [&str; 1] = ["locales"];

/// The name the user knows an engine by, for the tray and the log.
pub fn label(engine: EngineId) -> &'static str {
    match engine {
        EngineId::Mpv => "mpv",
        EngineId::Vlc => "libVLC",
        EngineId::Cef => "CEF",
        EngineId::WebView2 => "WebView2",
        EngineId::BuiltInGif => "the built-in GIF renderer",
        EngineId::BuiltInVideo => "the built-in video player",
        EngineId::AppEmbedder => "the application embedder",
    }
}

/// What the machine can run of each engine (D-028): the report [`choose_engine`] works from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineReport {
    states: Vec<(EngineId, Availability)>,
}

impl EngineReport {
    /// A report that asks `state` for every engine.
    pub fn from_fn(state: impl Fn(EngineId) -> Availability) -> Self {
        Self {
            states: ALL.iter().map(|engine| (*engine, state(*engine))).collect(),
        }
    }

    /// What is known of `engine`; an engine the report does not list is not in this build.
    pub fn availability(&self, engine: EngineId) -> Availability {
        self.states.iter().find(|(known, _)| *known == engine).map_or(
            Availability::Unsupported {
                why: Unsupported::NotInThisBuild,
            },
            |(_, availability)| availability.clone(),
        )
    }

    /// What `engine` can play a wallpaper of `kind` with in this build: the engine's own state, except that CEF
    /// plays only Web themes until its generated pages for GIF and video themes arrive in a later round, so a type
    /// it cannot play yet counts as not in this build.
    pub fn availability_for(&self, kind: WallpaperType, engine: EngineId) -> Availability {
        if engine == EngineId::Cef && kind != WallpaperType::Web {
            return Availability::Unsupported {
                why: Unsupported::NotInThisBuild,
            };
        }
        self.availability(engine)
    }
}

impl EngineReport {
    /// What the Portal is told (ENG-06, ENG-07): every engine with its state, the types of `kinds` some available
    /// engine can play, and the fallbacks that run now.
    pub fn view(&self, kinds: &[WallpaperType], fallbacks: &Fallbacks) -> EngineReportView {
        EngineReportView {
            engines: ALL
                .iter()
                .map(|engine| EngineStateView {
                    engine: engine.to_legacy(),
                    state: state_view(&self.availability(*engine)),
                })
                .collect(),
            supported: kinds.to_vec(),
            playable: kinds
                .iter()
                .copied()
                .filter(|kind| {
                    fallback_order(*kind)
                        .iter()
                        .any(|engine| self.availability_for(*kind, *engine).is_available())
                })
                .collect(),
            fallbacks: fallbacks
                .notes()
                .iter()
                .map(|note| FallbackView {
                    kind: note.kind,
                    configured: note.configured.to_legacy(),
                    playing: note.playing.to_legacy(),
                })
                .collect(),
        }
    }
}

/// The Portal's form of a hint.
fn hint_view(hint: &Hint) -> HintView {
    match hint {
        Hint::FilesMissing => HintView::FilesMissing,
        Hint::VcRedistMissing { needed } => HintView::VcRedistMissing { needed: needed.clone() },
        Hint::RuntimeMissing => HintView::RuntimeMissing,
        Hint::RuntimeOutdated { found, needed } => HintView::RuntimeOutdated {
            found: found.clone(),
            needed: needed.clone(),
        },
    }
}

/// Whether Sucrose can fix a missing part itself: a prerequisite it can download and run (04-18), not its own files.
fn installable(hint: &Hint) -> bool {
    !matches!(hint, Hint::FilesMissing)
}

/// The Portal's form of an availability.
fn state_view(availability: &Availability) -> EngineState {
    match availability {
        Availability::Installed { version } => EngineState::Ready {
            version: version.clone(),
        },
        Availability::Missing { hint } => EngineState::Missing {
            hint: hint_view(hint),
            installable: installable(hint),
        },
        Availability::NotLoadable { reason, hint } => EngineState::NotLoadable {
            reason: reason.clone(),
            hint: hint_view(hint),
        },
        Availability::Unsupported { why } => EngineState::Unsupported {
            architecture: *why == Unsupported::Architecture,
        },
    }
}

/// Builds the report for this machine from files and the registry, without loading anything (D-028): libmpv next to
/// the executable or on an absolute `PATH` entry, CEF with its files next to the executable, and every other engine
/// as not in this build.
pub fn detect(exe_dir: &Path, platform: &dyn Platform) -> EngineReport {
    let found = platform.vc_redist(Arch::current());
    tracing::info!(
        installed = found.as_ref().map(|found| found.installed),
        version = found.as_ref().map(|found| found.version.as_str()),
        arch = ?Arch::current(),
        "Visual C++ Redistributable"
    );
    let report = detect_with(
        exe_dir,
        std::env::var_os("PATH").as_deref(),
        &|path| path.exists(),
        vc_missing(found.as_ref()),
    );
    tracing::info!(
        mpv = ?report.availability(EngineId::Mpv),
        cef = ?report.availability(EngineId::Cef),
        "engines detected"
    );
    report
}

/// The repair hint when the Visual C++ Redistributable is missing or older than CEF accepts (D-15); `None` when it is
/// fine or the OS has no such prerequisite (`found` is `None`).
fn vc_missing(found: Option<&VcRedist>) -> Option<Hint> {
    let found = found?;
    if found.installed && version_at_least(&found.version, VC_REDIST_MIN) {
        return None;
    }
    Some(Hint::VcRedistMissing {
        needed: VC_REDIST_MIN.to_owned(),
    })
}

/// [`detect`] with the machine's answers given: `exists` says whether a file or folder is there, and
/// `vc_missing` is the repair hint when the Visual C++ Redistributable CEF needs is missing or too old.
fn detect_with(
    exe_dir: &Path,
    path_var: Option<&OsStr>,
    exists: &dyn Fn(&Path) -> bool,
    vc_missing: Option<Hint>,
) -> EngineReport {
    EngineReport::from_fn(|engine| match engine {
        EngineId::Mpv => match sucrose_engine_mpv::loader::locate(exe_dir, path_var) {
            Some(_) => Availability::Installed { version: None },
            None => Availability::Missing {
                hint: Hint::FilesMissing,
            },
        },
        EngineId::Cef => cef_state(exe_dir, exists, vc_missing.clone()),
        _ => Availability::Unsupported {
            why: Unsupported::NotInThisBuild,
        },
    })
}

/// CEF: built for this architecture, its files next to the executable and its prerequisite installed.
fn cef_state(exe_dir: &Path, exists: &dyn Fn(&Path) -> bool, vc_missing: Option<Hint>) -> Availability {
    if !sucrose_engine_cef::BUILT {
        // Windows on another architecture has no pinned dist; the other systems get theirs in a later round.
        let why = if sucrose_engine_cef::NO_BUILD_FOR_ARCHITECTURE {
            Unsupported::Architecture
        } else {
            Unsupported::NotInThisBuild
        };
        return Availability::Unsupported { why };
    }
    let present = |name: &&str| exists(&exe_dir.join(name));
    if !CEF_FILES.iter().all(present) || !CEF_DIRS.iter().all(present) {
        return Availability::Missing {
            hint: Hint::FilesMissing,
        };
    }
    if let Some(hint) = vc_missing {
        return Availability::Missing { hint };
    }
    Availability::Installed {
        version: Some(sucrose_engine_cef::CEF_VERSION.to_owned()),
    }
}

/// The folder of the running executable, where the engines' files are looked for.
pub fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_default()
}

/// One sentence on what the user can do about an engine that is not usable (ENG-06).
pub fn hint_text(engine: EngineId, hint: &Hint) -> String {
    let name = label(engine);
    match hint {
        Hint::FilesMissing => format!("The files of {name} are missing. Reinstall or repair Sucrose."),
        Hint::VcRedistMissing { needed } => {
            format!("{name} needs the Visual C++ Redistributable {needed} or newer.")
        }
        Hint::RuntimeMissing => format!("{name} needs the WebView2 Runtime."),
        Hint::RuntimeOutdated { found, needed } => {
            format!("{name} needs a WebView2 Runtime newer than {needed} (found {found}).")
        }
    }
}

/// Why an engine cannot play, in one clause for the log.
fn reason_text(engine: EngineId, availability: &Availability) -> String {
    match availability {
        Availability::Installed { .. } => "it is available".to_owned(),
        Availability::Missing { hint } | Availability::NotLoadable { hint, .. } => match hint {
            Hint::FilesMissing => "its files are missing".to_owned(),
            Hint::VcRedistMissing { needed } => format!("the Visual C++ Redistributable {needed} or newer is missing"),
            Hint::RuntimeMissing => "the WebView2 Runtime is missing".to_owned(),
            Hint::RuntimeOutdated { found, needed } => {
                format!("the WebView2 Runtime {found} is not newer than {needed}")
            }
        },
        Availability::Unsupported {
            why: Unsupported::Architecture,
        } => format!("there is no build of {} for this processor architecture", label(engine)),
        Availability::Unsupported {
            why: Unsupported::NotInThisBuild,
        } => format!(
            "this build of Sucrose does not play this wallpaper with {}",
            label(engine)
        ),
    }
}

/// A job that plays with another engine than the configured one, for this run only (D-11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackNote {
    /// The wallpaper type.
    pub kind: WallpaperType,
    /// The engine the settings ask for, which could not be used.
    pub configured: EngineId,
    /// The engine that plays instead.
    pub playing: EngineId,
    /// Why the configured engine could not be used, one clause.
    pub reason: String,
}

/// What [`pick`] chose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picked {
    /// The engine that plays.
    pub engine: EngineId,
    /// Set when it is not the configured one.
    pub fallback: Option<FallbackNote>,
}

/// Chooses the engine that plays a wallpaper of `kind` from `report` (D-11): the configured engine when it is
/// available, otherwise the first of the fallback order that is. With no engine available the error is the sentence
/// the tray and the Portal show, with the hint for the configured engine when it has one.
pub fn pick(report: &EngineReport, kind: WallpaperType, configured: EngineType) -> Result<Picked, String> {
    let choice = choose_engine(kind, configured, &|engine| report.availability_for(kind, engine));
    let Some(engine) = choice.engine else {
        let hint = choice
            .fell_back_from
            .as_ref()
            .and_then(|(wanted, availability)| match availability {
                Availability::Missing { hint } | Availability::NotLoadable { hint, .. } => {
                    Some(hint_text(*wanted, hint))
                }
                _ => None,
            });
        return Err(match hint {
            Some(hint) => format!("{NO_ENGINE} {hint}"),
            None => NO_ENGINE.to_owned(),
        });
    };
    let fallback = choice.fell_back_from.map(|(wanted, availability)| FallbackNote {
        kind,
        configured: wanted,
        playing: engine,
        reason: reason_text(wanted, &availability),
    });
    Ok(Picked { engine, fallback })
}

/// The fallbacks that run now, one per wallpaper type (D-11). Nothing is stored: a restart starts with none, and the
/// configured engine is used again as soon as it is available.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Fallbacks {
    notes: Vec<FallbackNote>,
}

impl Fallbacks {
    /// Records the fallback a job of `kind` plays with, or that it plays with the configured engine (`None`). A
    /// fallback that is new or changed is logged once.
    pub fn set(&mut self, kind: WallpaperType, note: Option<FallbackNote>) {
        let known = self.notes.iter().position(|known| known.kind == kind);
        match (known, note) {
            (Some(index), Some(note)) => {
                if self.notes[index] != note {
                    log_fallback(&note);
                    self.notes[index] = note;
                }
            }
            (None, Some(note)) => {
                log_fallback(&note);
                self.notes.push(note);
            }
            (Some(index), None) => {
                self.notes.remove(index);
            }
            (None, None) => {}
        }
    }

    /// Forgets every fallback: nothing plays.
    pub fn clear(&mut self) {
        self.notes.clear();
    }

    /// The fallbacks that run now.
    pub fn notes(&self) -> &[FallbackNote] {
        &self.notes
    }
}

/// The log line of a fallback (D-11): the stored names of the configured and the playing engine and why.
fn log_fallback(note: &FallbackNote) {
    tracing::warn!(
        r#type = ?note.kind,
        configured = ?note.configured.to_legacy(),
        playing = ?note.playing.to_legacy(),
        reason = %note.reason,
        "engine fallback"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nothing_there(_: &Path) -> bool {
        false
    }

    fn everything_there(_: &Path) -> bool {
        true
    }

    /// A folder with the named files, removed when dropped.
    struct Folder(PathBuf);

    impl Folder {
        fn with(files: &[&str]) -> Self {
            let path = std::env::temp_dir().join(format!("sucrose-engines-{}-{}", std::process::id(), files.len()));
            std::fs::create_dir_all(&path).expect("create the scratch folder");
            for name in files {
                std::fs::write(path.join(name), b"x").expect("write a scratch file");
            }
            Self(path)
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn mpv_name() -> &'static str {
        sucrose_engine_mpv::loader::LIBRARY_NAMES[0]
    }

    #[test]
    fn engines_next_to_the_executable_are_found_without_loading_anything() {
        let folder = Folder::with(&[mpv_name()]);

        let report = detect_with(&folder.0, None, &everything_there, None);

        assert_eq!(
            report.availability(EngineId::Mpv),
            Availability::Installed { version: None }
        );
    }

    #[test]
    fn a_missing_mpv_library_is_reported_with_the_repair_hint() {
        let folder = Folder::with(&[]);

        let report = detect_with(&folder.0, Some(OsStr::new("")), &nothing_there, None);

        assert_eq!(
            report.availability(EngineId::Mpv),
            Availability::Missing {
                hint: Hint::FilesMissing
            }
        );
    }

    #[test]
    fn cef_needs_every_file_next_to_the_executable() {
        let folder = Folder::with(&[]);

        let whole = detect_with(&folder.0, None, &everything_there, None).availability(EngineId::Cef);
        let torn =
            detect_with(&folder.0, None, &|path| !path.ends_with("icudtl.dat"), None).availability(EngineId::Cef);

        if sucrose_engine_cef::BUILT {
            assert!(
                matches!(whole, Availability::Installed { version: Some(_) }),
                "{whole:?}"
            );
            assert_eq!(
                torn,
                Availability::Missing {
                    hint: Hint::FilesMissing
                }
            );
        } else {
            assert!(matches!(whole, Availability::Unsupported { .. }), "{whole:?}");
        }
    }

    #[test]
    fn cef_with_a_missing_redistributable_is_missing_with_the_version_to_install() {
        let hint = Hint::VcRedistMissing {
            needed: "14.40.33816.0".to_owned(),
        };

        let state = cef_state(Path::new("."), &everything_there, Some(hint.clone()));

        if sucrose_engine_cef::BUILT {
            assert_eq!(state, Availability::Missing { hint });
        }
    }

    #[test]
    fn the_redistributable_must_be_installed_and_at_least_the_legacy_version() {
        let redist = |installed, version: &str| VcRedist {
            installed,
            version: version.to_owned(),
        };

        assert_eq!(vc_missing(None), None);
        assert_eq!(vc_missing(Some(&redist(true, "v14.44.35211.00"))), None);
        assert_eq!(vc_missing(Some(&redist(true, "v14.40.33816.00"))), None);
        for refused in [
            redist(false, "v14.44.35211.00"),
            redist(true, "v14.38.33135.00"),
            redist(true, ""),
            redist(false, ""),
        ] {
            assert_eq!(
                vc_missing(Some(&refused)),
                Some(Hint::VcRedistMissing {
                    needed: "14.40.33816.0".to_owned()
                }),
                "{refused:?}"
            );
        }
    }

    #[test]
    fn the_view_lists_every_engine_the_playable_types_and_the_fallbacks() {
        let report = EngineReport::from_fn(|engine| match engine {
            EngineId::Mpv => Availability::Installed {
                version: Some("0.40".to_owned()),
            },
            EngineId::Cef => Availability::Missing {
                hint: Hint::VcRedistMissing {
                    needed: "14.40.33816.0".to_owned(),
                },
            },
            _ => Availability::Unsupported {
                why: Unsupported::Architecture,
            },
        });
        let mut fallbacks = Fallbacks::default();
        fallbacks.set(
            WallpaperType::Web,
            Some(FallbackNote {
                kind: WallpaperType::Web,
                configured: EngineId::WebView2,
                playing: EngineId::Cef,
                reason: "x".to_owned(),
            }),
        );

        let view = report.view(&[WallpaperType::Video, WallpaperType::Web], &fallbacks);

        assert_eq!(view.engines.len(), ALL.len());
        assert_eq!(view.engines[0].engine, EngineType::MpvPlayerLive);
        assert_eq!(
            view.engines[0].state,
            EngineState::Ready {
                version: Some("0.40".to_owned())
            }
        );
        assert_eq!(
            view.engines[2].state,
            EngineState::Missing {
                hint: HintView::VcRedistMissing {
                    needed: "14.40.33816.0".to_owned()
                },
                installable: true
            }
        );
        assert_eq!(view.engines[3].state, EngineState::Unsupported { architecture: true });
        assert_eq!(view.supported, [WallpaperType::Video, WallpaperType::Web]);
        assert_eq!(view.playable, [WallpaperType::Video]);
        assert_eq!(
            view.fallbacks,
            [FallbackView {
                kind: WallpaperType::Web,
                configured: EngineType::WebViewLive,
                playing: EngineType::CefSharpLive
            }]
        );
    }

    #[test]
    fn missing_engine_files_are_not_installable_by_sucrose() {
        assert!(!installable(&Hint::FilesMissing));
        assert!(installable(&Hint::RuntimeMissing));
    }

    #[test]
    fn engines_that_are_not_built_report_not_in_this_build() {
        let report = detect_with(Path::new("."), None, &everything_there, None);

        for engine in [
            EngineId::Vlc,
            EngineId::WebView2,
            EngineId::BuiltInGif,
            EngineId::BuiltInVideo,
            EngineId::AppEmbedder,
        ] {
            assert_eq!(
                report.availability(engine),
                Availability::Unsupported {
                    why: Unsupported::NotInThisBuild
                },
                "{engine:?}"
            );
        }
    }

    #[test]
    fn cef_does_not_play_a_video_until_its_generated_pages_exist() {
        let report = EngineReport::from_fn(|_| Availability::Installed { version: None });

        assert!(
            report
                .availability_for(WallpaperType::Web, EngineId::Cef)
                .is_available()
        );
        assert!(
            !report
                .availability_for(WallpaperType::Video, EngineId::Cef)
                .is_available()
        );
    }

    fn only(available: &'static [EngineId]) -> EngineReport {
        EngineReport::from_fn(|engine| {
            if available.contains(&engine) {
                Availability::Installed { version: None }
            } else {
                Availability::Missing {
                    hint: Hint::FilesMissing,
                }
            }
        })
    }

    #[test]
    fn an_available_configured_engine_plays_without_a_fallback() {
        let picked = pick(&only(&[EngineId::Cef]), WallpaperType::Web, EngineType::CefSharpLive).expect("an engine");

        assert_eq!(picked.engine, EngineId::Cef);
        assert_eq!(picked.fallback, None);
    }

    #[test]
    fn a_missing_configured_engine_falls_back_for_this_run_and_says_why() {
        let report = EngineReport::from_fn(|engine| match engine {
            EngineId::Cef => Availability::Installed { version: None },
            _ => Availability::Unsupported {
                why: Unsupported::NotInThisBuild,
            },
        });

        let picked = pick(&report, WallpaperType::Web, EngineType::WebViewLive).expect("an engine");

        assert_eq!(picked.engine, EngineId::Cef);
        let note = picked.fallback.expect("a fallback");
        assert_eq!((note.configured, note.playing), (EngineId::WebView2, EngineId::Cef));
        assert!(note.reason.contains("does not play"), "{}", note.reason);
    }

    #[test]
    fn a_type_no_available_engine_can_play_is_refused_with_its_own_sentence_and_the_hint() {
        let refused = pick(&only(&[]), WallpaperType::Video, EngineType::MpvPlayerLive).expect_err("no engine");

        assert!(refused.starts_with(NO_ENGINE), "{refused}");
        assert!(refused.contains("files of mpv are missing"), "{refused}");
        let bare = pick(
            &EngineReport::from_fn(|_| Availability::Unsupported {
                why: Unsupported::NotInThisBuild,
            }),
            WallpaperType::Video,
            EngineType::MpvPlayerLive,
        )
        .expect_err("no engine");
        assert_eq!(bare, NO_ENGINE);
    }

    #[test]
    fn fallbacks_keep_one_note_per_type_and_forget_it_when_the_configured_engine_plays() {
        let note = |playing| FallbackNote {
            kind: WallpaperType::Web,
            configured: EngineId::WebView2,
            playing,
            reason: "x".to_owned(),
        };
        let mut fallbacks = Fallbacks::default();

        fallbacks.set(WallpaperType::Web, Some(note(EngineId::Cef)));
        fallbacks.set(WallpaperType::Web, Some(note(EngineId::Cef)));
        assert_eq!(fallbacks.notes().len(), 1);
        fallbacks.set(WallpaperType::Web, Some(note(EngineId::Vlc)));
        assert_eq!(fallbacks.notes()[0].playing, EngineId::Vlc);
        fallbacks.set(WallpaperType::Web, None);

        assert!(fallbacks.notes().is_empty());
    }

    #[test]
    fn the_hint_names_the_engine_and_what_to_do() {
        assert_eq!(
            hint_text(
                EngineId::Cef,
                &Hint::VcRedistMissing {
                    needed: "14.40.33816.0".to_owned()
                }
            ),
            "CEF needs the Visual C++ Redistributable 14.40.33816.0 or newer."
        );
        assert!(hint_text(EngineId::Mpv, &Hint::FilesMissing).contains("Reinstall or repair"));
    }
}
