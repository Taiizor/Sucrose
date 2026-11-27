//! What this machine can play with, and the fallbacks that run (ENG-06, ENG-07, D-11, D-15, D-028).
//!
//! The service decides from files and registry values only: it never loads an engine library, because only renderers
//! load engines (D-028) and the service stays light (D-008). The report is built at service start and again when a
//! Portal asks; a fallback applies to the current run, the configured engine stays in the settings (D-11).

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use sucrose_config::model::Warehouse;
use sucrose_core::{settings::EngineType, theme::WallpaperType};
use sucrose_engine::{
    Availability, EngineId, Hint, Unsupported, VC_REDIST_MIN, WEBVIEW2_FLOOR, choose_engine, fallback_order,
    version_at_least, version_newer_than,
};
use sucrose_ipc::portal::{EngineReportView, EngineState, EngineStateView, FallbackView, HintView, OfferedView};
use sucrose_platform::{Arch, Platform, VcRedist};

use crate::install::Prerequisite;

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

    /// What `engine` can play a wallpaper of `kind` with in this build: the engine's own state, except that the web
    /// engines (CEF and WebView2) play only the types whose page they have: Web themes, and the types that have a
    /// generated page (GIF, Video and YouTube) or an address (URL). A type they cannot play yet counts as not in
    /// this build.
    pub fn availability_for(&self, kind: WallpaperType, engine: EngineId) -> Availability {
        let plays = match engine {
            EngineId::Cef | EngineId::WebView2 => {
                matches!(
                    kind,
                    WallpaperType::Web
                        | WallpaperType::Gif
                        | WallpaperType::Video
                        | WallpaperType::Url
                        | WallpaperType::YouTube
                )
            }
            _ => true,
        };
        if !plays {
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
    pub fn view(
        &self,
        kinds: &[WallpaperType],
        fallbacks: &Fallbacks,
        warehouse: &Warehouse,
        now: i64,
    ) -> EngineReportView {
        EngineReportView {
            engines: ALL
                .iter()
                .map(|engine| EngineStateView {
                    engine: engine.to_legacy(),
                    state: state_view(&self.availability(*engine)),
                })
                .collect(),
            supported: kinds.to_vec(),
            offered: kinds
                .iter()
                .map(|kind| OfferedView {
                    kind: *kind,
                    engines: ALL
                        .iter()
                        .filter(|engine| {
                            engine.supports(*kind)
                                && !matches!(
                                    self.availability_for(*kind, **engine),
                                    Availability::Unsupported {
                                        why: Unsupported::NotInThisBuild
                                    }
                                )
                        })
                        .map(|engine| engine.to_legacy())
                        .collect(),
                })
                .collect(),
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
                    notify: !snoozed(warehouse, note.configured, now),
                })
                .collect(),
        }
    }
}

/// How long "Don't remind me for a day" keeps the fallback toast of an engine quiet (D-16).
pub const SNOOZE_SECS: i64 = 86_400;

/// The Warehouse field that holds the snooze of an engine's fallback toast: the legacy `CefSharpTime` and `WebViewTime`
/// (D-16). Every other engine has none (its snooze lasts until the Portal restarts, UI-SPEC E3).
fn snooze_field(warehouse: &mut Warehouse, engine: EngineId) -> Option<&mut String> {
    match engine {
        EngineId::Cef => Some(&mut warehouse.cef_sharp_time),
        EngineId::WebView2 => Some(&mut warehouse.web_view_time),
        _ => None,
    }
}

/// Seconds since the Unix epoch.
pub fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| i64::try_from(since.as_secs()).unwrap_or(i64::MAX))
}

/// Whether the toast of `engine`'s fallback is snoozed at `now`. Like the legacy apps the field holds the time until
/// which to stay quiet (`DateTime.Now.AddDays(1)`), so a migrated value counts. A time in the past, one more than a
/// day ahead (a changed clock must not silence a fallback for good), an empty or an unparsable text is no snooze.
fn snoozed(warehouse: &Warehouse, engine: EngineId, now: i64) -> bool {
    let text = match engine {
        EngineId::Cef => &warehouse.cef_sharp_time,
        EngineId::WebView2 => &warehouse.web_view_time,
        _ => return false,
    };
    parse_time(text).is_some_and(|until| until > now && until <= now + SNOOZE_SECS + 60)
}

/// Whether `engine` has a Warehouse field to keep a snooze in (T-04-34): only CEF and WebView2 do.
pub fn can_snooze(engine: EngineId) -> bool {
    matches!(engine, EngineId::Cef | EngineId::WebView2)
}

/// Starts the snooze of `engine` at `now`: the quiet time is written into its Warehouse field in the legacy text format.
/// Returns whether the engine has a field to write.
pub fn snooze(warehouse: &mut Warehouse, engine: EngineId, now: i64) -> bool {
    match snooze_field(warehouse, engine) {
        Some(field) => {
            *field = format_time(now + SNOOZE_SECS);
            true
        }
        None => false,
    }
}

/// Days since 1970-01-01 of a civil date (Howard Hinnant's algorithm).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The civil date (year, month, day) of a day count since 1970-01-01.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_part + 2) / 5 + 1;
    let month = if month_part < 10 {
        month_part + 3
    } else {
        month_part - 9
    };
    (year_of_era + era * 400 + i64::from(month <= 2), month, day)
}

/// Writes `unix` seconds as the legacy date-time text (`2026-11-22T01:02:03.0000000+00:00`): UTC with an explicit
/// offset, which .NET reads as the same instant (the local offset would need OS code outside `sucrose-platform`).
fn format_time(unix: i64) -> String {
    let (days, rest) = (unix.div_euclid(86_400), unix.rem_euclid(86_400));
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.0000000+00:00",
        rest / 3_600,
        rest % 3_600 / 60,
        rest % 60
    )
}

/// Reads the legacy date-time text (`YYYY-MM-DDTHH:MM:SS`, optional fraction, optional `Z` or `±HH:MM`) as Unix seconds.
/// A text without an offset is taken as UTC; a different text gives `None`.
fn parse_time(text: &str) -> Option<i64> {
    let text = text.trim();
    let (date, time) = text.split_once('T')?;
    let mut date = date.split('-');
    let (year, month, day) = (date.next()?, date.next()?, date.next()?);
    if date.next().is_some() {
        return None;
    }
    let (year, month, day): (i64, i64, i64) = (year.parse().ok()?, month.parse().ok()?, day.parse().ok()?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // The clock part ends where the offset starts: `Z`, `+` or `-`.
    let (clock, offset) = match time.find(['Z', '+', '-']) {
        Some(index) => time.split_at(index),
        None => (time, ""),
    };
    let clock = clock.split_once('.').map_or(clock, |(whole, _)| whole);
    let mut clock = clock.split(':');
    let (hour, minute, second) = (clock.next()?, clock.next()?, clock.next()?);
    if clock.next().is_some() {
        return None;
    }
    let (hour, minute, second): (i64, i64, i64) = (hour.parse().ok()?, minute.parse().ok()?, second.parse().ok()?);
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let offset_secs = match offset {
        "" | "Z" => 0,
        signed => {
            let sign = if signed.starts_with('-') { -1 } else { 1 };
            let (hours, minutes) = signed.get(1..)?.split_once(':')?;
            let (hours, minutes): (i64, i64) = (hours.parse().ok()?, minutes.parse().ok()?);
            if hours > 14 || minutes > 59 {
                return None;
            }
            sign * (hours * 3_600 + minutes * 60)
        }
    };
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second - offset_secs)
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

/// Whether Sucrose can fix a missing part itself: a prerequisite it can download and run (04-18, 04-27) for this
/// processor, not its own files.
fn installable(hint: &Hint) -> bool {
    installable_on(hint, Arch::current())
}

/// [`installable`] for the processor `arch`: true when the prerequisite the hint names has a download address there.
fn installable_on(hint: &Hint, arch: Arch) -> bool {
    let prerequisite = match hint {
        Hint::FilesMissing => return false,
        Hint::VcRedistMissing { .. } => Prerequisite::VcRedist,
        Hint::RuntimeMissing | Hint::RuntimeOutdated { .. } => Prerequisite::WebView2Runtime,
    };
    prerequisite.url(arch).is_some()
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
/// the executable or on an absolute `PATH` entry, CEF with its files next to the executable, WebView2 from the version of
/// the installed runtime, and every other engine as not in this build.
pub fn detect(exe_dir: &Path, platform: &dyn Platform) -> EngineReport {
    let found = platform.vc_redist(Arch::current());
    tracing::info!(
        installed = found.as_ref().map(|found| found.installed),
        version = found.as_ref().map(|found| found.version.as_str()),
        arch = ?Arch::current(),
        "Visual C++ Redistributable"
    );
    // The loader reads the installed runtime's version; no WebView is created in the service (D-008, D-028).
    let webview2 = sucrose_engine_webview2::runtime_version();
    tracing::info!(
        version = webview2.as_deref(),
        floor = WEBVIEW2_FLOOR,
        "WebView2 Runtime"
    );
    let report = detect_with(
        exe_dir,
        std::env::var_os("PATH").as_deref(),
        &|path| path.exists(),
        vc_missing(found.as_ref()),
        webview2.as_deref(),
    );
    tracing::info!(
        mpv = ?report.availability(EngineId::Mpv),
        cef = ?report.availability(EngineId::Cef),
        webview2 = ?report.availability(EngineId::WebView2),
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

/// Whether the Visual C++ Redistributable is installed and new enough now (or the OS has no such prerequisite): the rule
/// [`detect`] applies, asked again after an install.
pub fn vc_is_fine(found: Option<&VcRedist>) -> bool {
    vc_missing(found).is_none()
}

/// Whether a WebView2 Runtime newer than the legacy floor is installed (version `None`: none): the rule [`detect`]
/// applies, asked again after an install.
pub fn webview2_is_fine(version: Option<&str>) -> bool {
    matches!(webview2_state(true, version), Availability::Installed { .. })
}

/// [`detect`] with the machine's answers given: `exists` says whether a file or folder is there, and
/// `vc_missing` is the repair hint when the Visual C++ Redistributable CEF needs is missing or too old, and
/// `webview2_version` is the version of the installed WebView2 Runtime, `None` when there is none.
fn detect_with(
    exe_dir: &Path,
    path_var: Option<&OsStr>,
    exists: &dyn Fn(&Path) -> bool,
    vc_missing: Option<Hint>,
    webview2_version: Option<&str>,
) -> EngineReport {
    EngineReport::from_fn(|engine| match engine {
        EngineId::Mpv => match sucrose_engine_mpv::loader::locate(exe_dir, path_var) {
            Some(_) => Availability::Installed { version: None },
            None => Availability::Missing {
                hint: Hint::FilesMissing,
            },
        },
        EngineId::Cef => cef_state(exe_dir, exists, vc_missing.clone()),
        EngineId::WebView2 => webview2_state(sucrose_engine_webview2::BUILT, webview2_version),
        EngineId::BuiltInGif => built_in_gif_state(sucrose_engine_gif::BUILT),
        _ => Availability::Unsupported {
            why: Unsupported::NotInThisBuild,
        },
    })
}

/// The built-in GIF renderer: part of the executable, so it is installed wherever it is built (Windows); it has no files
/// and no version of its own.
fn built_in_gif_state(built: bool) -> Availability {
    if built {
        Availability::Installed { version: None }
    } else {
        Availability::Unsupported {
            why: Unsupported::NotInThisBuild,
        }
    }
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

/// WebView2: built for this system and a runtime installed that is newer than the legacy floor (D-15, ENG-06). The
/// runtime is the user's, so there are no files of ours to check; a runtime at the floor or older is outdated, and the
/// hint names what was found.
fn webview2_state(built: bool, runtime: Option<&str>) -> Availability {
    if !built {
        return Availability::Unsupported {
            why: Unsupported::NotInThisBuild,
        };
    }
    match runtime {
        None => Availability::Missing {
            hint: Hint::RuntimeMissing,
        },
        Some(found) if version_newer_than(found, WEBVIEW2_FLOOR) => Availability::Installed {
            version: Some(found.to_owned()),
        },
        Some(found) => Availability::Missing {
            hint: Hint::RuntimeOutdated {
                found: found.to_owned(),
                needed: WEBVIEW2_FLOOR.to_owned(),
            },
        },
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

    /// 2026-11-26T12:00:00Z.
    const NOW: i64 = 1_795_694_400;

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

        let report = detect_with(&folder.0, None, &everything_there, None, None);

        assert_eq!(
            report.availability(EngineId::Mpv),
            Availability::Installed { version: None }
        );
    }

    #[test]
    fn a_missing_mpv_library_is_reported_with_the_repair_hint() {
        let folder = Folder::with(&[]);

        let report = detect_with(&folder.0, Some(OsStr::new("")), &nothing_there, None, None);

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

        let whole = detect_with(&folder.0, None, &everything_there, None, None).availability(EngineId::Cef);
        let torn =
            detect_with(&folder.0, None, &|path| !path.ends_with("icudtl.dat"), None, None).availability(EngineId::Cef);

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

        let view = report.view(
            &[WallpaperType::Video, WallpaperType::Web],
            &fallbacks,
            &Warehouse::default(),
            NOW,
        );

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
                playing: EngineType::CefSharpLive,
                notify: true
            }]
        );
    }

    #[test]
    fn the_view_offers_each_type_the_engines_whose_slice_landed_for_it_installed_or_not() {
        let report = EngineReport::from_fn(|engine| match engine {
            EngineId::Mpv => Availability::Installed { version: None },
            EngineId::Cef => Availability::Missing {
                hint: Hint::FilesMissing,
            },
            _ => Availability::Unsupported {
                why: Unsupported::NotInThisBuild,
            },
        });

        let view = report.view(
            &[WallpaperType::Video, WallpaperType::Web],
            &Fallbacks::default(),
            &Warehouse::default(),
            NOW,
        );

        assert_eq!(
            view.offered,
            [
                OfferedView {
                    kind: WallpaperType::Video,
                    engines: vec![EngineType::MpvPlayerLive, EngineType::CefSharpLive],
                },
                OfferedView {
                    kind: WallpaperType::Web,
                    engines: vec![EngineType::CefSharpLive],
                },
            ],
            "CEF plays a video through its generated page, so Video offers it; WebView2 is not in this build"
        );
    }

    #[test]
    fn the_legacy_date_time_text_reads_and_writes_as_the_same_instant() {
        assert_eq!(parse_time("2026-11-26T12:00:00"), Some(NOW));
        assert_eq!(parse_time("2026-11-26T12:00:00Z"), Some(NOW));
        assert_eq!(parse_time("2026-11-26T15:00:00.1234567+03:00"), Some(NOW));
        assert_eq!(parse_time("2026-11-26T09:00:00-03:00"), Some(NOW));
        assert_eq!(parse_time("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_time(&format_time(NOW)), Some(NOW));
        assert_eq!(format_time(NOW), "2026-11-26T12:00:00.0000000+00:00");
    }

    #[test]
    fn a_text_that_is_not_a_date_time_does_not_parse() {
        for text in [
            "",
            "yesterday",
            "2026-13-01T00:00:00",
            "2026-11-26",
            "2026-11-26T25:00:00",
            "2026-11-26T12:00",
        ] {
            assert_eq!(parse_time(text), None, "{text:?}");
        }
    }

    #[test]
    fn a_fallback_is_announced_unless_its_engine_is_snoozed_for_now() {
        let mut warehouse = Warehouse::default();
        assert!(
            !snoozed(&warehouse, EngineId::WebView2, NOW),
            "the default is the minimum date"
        );

        warehouse.web_view_time = format_time(NOW + 3_600);
        assert!(snoozed(&warehouse, EngineId::WebView2, NOW), "the quiet time is ahead");
        assert!(
            !snoozed(&warehouse, EngineId::Cef, NOW),
            "the other engine has its own field"
        );
        assert!(
            !snoozed(&warehouse, EngineId::WebView2, NOW + 3_601),
            "the quiet time passed"
        );

        warehouse.web_view_time = format_time(NOW + 10 * SNOOZE_SECS);
        assert!(
            !snoozed(&warehouse, EngineId::WebView2, NOW),
            "a time far ahead is no snooze"
        );
        warehouse.web_view_time = "not a time".to_owned();
        assert!(
            !snoozed(&warehouse, EngineId::WebView2, NOW),
            "an unparsable text is no snooze"
        );
    }

    #[test]
    fn a_legacy_value_is_honoured_and_a_snooze_writes_the_field_of_its_engine_only() {
        let mut warehouse = Warehouse {
            cef_sharp_time: "2026-11-27T01:02:03.1234567+03:00".to_owned(),
            ..Warehouse::default()
        };
        assert!(snoozed(&warehouse, EngineId::Cef, NOW), "C# stored now plus a day");

        assert!(snooze(&mut warehouse, EngineId::WebView2, NOW));
        assert_eq!(warehouse.web_view_time, "2026-11-27T12:00:00.0000000+00:00");
        assert!(snoozed(&warehouse, EngineId::WebView2, NOW));
        let before = warehouse.clone();
        assert!(!snooze(&mut warehouse, EngineId::Mpv, NOW), "mpv has no field");
        assert!(!can_snooze(EngineId::Mpv) && can_snooze(EngineId::Cef));
        assert_eq!(warehouse, before, "nothing changes for an engine without a field");
    }

    #[test]
    fn the_view_does_not_notify_for_a_snoozed_engine() {
        let report = EngineReport::from_fn(|_| Availability::Unsupported {
            why: Unsupported::NotInThisBuild,
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
        let mut warehouse = Warehouse::default();
        snooze(&mut warehouse, EngineId::WebView2, NOW);

        let view = report.view(&[WallpaperType::Web], &fallbacks, &warehouse, NOW);

        assert!(!view.fallbacks[0].notify);
        let fresh = report.view(&[WallpaperType::Web], &fallbacks, &Warehouse::default(), NOW);
        assert!(fresh.fallbacks[0].notify);
    }

    #[test]
    fn missing_engine_files_are_not_installable_by_sucrose() {
        assert!(!installable(&Hint::FilesMissing));
        assert!(installable(&Hint::RuntimeMissing));
    }

    #[test]
    fn a_missing_or_outdated_webview2_runtime_is_installable_where_a_download_exists() {
        let outdated = Hint::RuntimeOutdated {
            found: "100.0".to_owned(),
            needed: WEBVIEW2_FLOOR.to_owned(),
        };
        for arch in [Arch::X86, Arch::X64, Arch::Arm64] {
            assert!(installable_on(&Hint::RuntimeMissing, arch), "{arch:?}");
            assert!(installable_on(&outdated, arch), "{arch:?}");
            assert!(!installable_on(&Hint::FilesMissing, arch), "{arch:?}");
        }
        assert!(!installable_on(&Hint::RuntimeMissing, Arch::Other));
        assert!(!installable_on(&outdated, Arch::Other));
        assert!(installable_on(
            &Hint::VcRedistMissing {
                needed: "14.40".to_owned()
            },
            Arch::X64
        ));
    }

    #[test]
    fn the_install_recheck_accepts_only_a_runtime_newer_than_the_floor() {
        assert!(webview2_is_fine(Some("158.0.4370.33")));
        assert!(!webview2_is_fine(Some(WEBVIEW2_FLOOR)));
        assert!(!webview2_is_fine(Some("100.0.1")));
        assert!(!webview2_is_fine(None));
    }

    #[test]
    fn engines_that_are_not_built_report_not_in_this_build() {
        let report = detect_with(Path::new("."), None, &everything_there, None, None);

        for engine in [EngineId::Vlc, EngineId::BuiltInVideo, EngineId::AppEmbedder] {
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
    fn the_built_in_gif_renderer_is_installed_where_it_is_built() {
        assert_eq!(built_in_gif_state(true), Availability::Installed { version: None });
        assert_eq!(
            built_in_gif_state(false),
            Availability::Unsupported {
                why: Unsupported::NotInThisBuild
            }
        );
        let report = detect_with(Path::new("."), None, &everything_there, None, None);
        assert_eq!(
            report.availability(EngineId::BuiltInGif),
            built_in_gif_state(sucrose_engine_gif::BUILT)
        );
    }

    #[test]
    fn the_web_engines_play_the_types_whose_pages_they_have_and_nothing_else() {
        let report = EngineReport::from_fn(|_| Availability::Installed { version: None });

        for engine in [EngineId::Cef, EngineId::WebView2] {
            for kind in [
                WallpaperType::Web,
                WallpaperType::Gif,
                WallpaperType::Video,
                WallpaperType::Url,
                WallpaperType::YouTube,
            ] {
                assert!(
                    report.availability_for(kind, engine).is_available(),
                    "{engine:?} plays {kind:?} (a theme page, a generated page or an address)"
                );
            }
            assert_eq!(
                report.availability_for(WallpaperType::Application, engine),
                Availability::Unsupported {
                    why: Unsupported::NotInThisBuild
                },
                "{engine:?} Application"
            );
        }
    }

    #[test]
    fn webview2_is_installed_with_the_version_of_a_runtime_newer_than_the_floor() {
        assert_eq!(
            webview2_state(true, Some("158.0.4370.33")),
            Availability::Installed {
                version: Some("158.0.4370.33".to_owned())
            }
        );
        assert_eq!(
            webview2_state(true, Some("131.0.2903.71")),
            Availability::Installed {
                version: Some("131.0.2903.71".to_owned())
            }
        );
    }

    #[test]
    fn webview2_at_the_floor_or_older_is_outdated_and_the_hint_says_what_was_found() {
        for found in ["131.0.2903.70", "131.0.2903.69", "100.0.1185.29"] {
            assert_eq!(
                webview2_state(true, Some(found)),
                Availability::Missing {
                    hint: Hint::RuntimeOutdated {
                        found: found.to_owned(),
                        needed: "131.0.2903.70".to_owned()
                    }
                },
                "{found}"
            );
        }
    }

    #[test]
    fn webview2_without_a_runtime_is_missing_and_without_the_engine_it_is_not_in_this_build() {
        assert_eq!(
            webview2_state(true, None),
            Availability::Missing {
                hint: Hint::RuntimeMissing
            }
        );
        assert_eq!(
            webview2_state(false, Some("158.0.4370.33")),
            Availability::Unsupported {
                why: Unsupported::NotInThisBuild
            }
        );
    }

    #[test]
    fn detect_asks_the_runtime_version_through_the_webview2_state() {
        let report = detect_with(Path::new("."), None, &everything_there, None, Some("158.0.4370.33"));

        let state = report.availability(EngineId::WebView2);

        if sucrose_engine_webview2::BUILT {
            assert_eq!(
                state,
                Availability::Installed {
                    version: Some("158.0.4370.33".to_owned())
                }
            );
        } else {
            assert!(matches!(state, Availability::Unsupported { .. }), "{state:?}");
        }
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
