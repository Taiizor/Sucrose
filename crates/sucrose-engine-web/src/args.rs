//! The Chromium command line switches of the web engines (D-17, D-021, WAPI-09, WAPI-10): the model, the converters
//! between the two legacy storage shapes, the legacy defaults, Sucrose's own switches and the merge.
//!
//! The legacy app stores the CefSharp switches as a dictionary (`Engine.CefArguments`) and the WebView2 switches as a
//! list of `--name=value` strings (`Engine.WebArguments`). Both convert to one ordered list of [`Switch`] values here.
//! Sucrose's own switches (hardware acceleration, the debugging origin rule) are **computed** from the settings every
//! time and merged in; they are never written into the user's lists, so the user's text stays what the user typed.
//! No OS code and no CEF types: it builds and tests on every OS.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The legacy CefSharp keys that are CEF *settings*, not Chromium switches. The windowed CEF host with an external
/// message pump (D-013, D-036) has no multi-threaded message loop and no off-screen rendering, so these two are
/// skipped with a log line instead of reaching the command line, where they would mean nothing or the wrong thing.
pub const CEF_SETTING_KEYS: [&str; 2] = ["multi-threaded-message-loop", "off-screen-rendering-enabled"];

/// From this Windows build on (24H2, the legacy `IsGermanium`), CEF is not given `disable-gpu` and `disable-gpu-vsync`
/// when hardware acceleration is off, and the user's `disable-gpu-compositing` is dropped (engines.md `2, `9).
pub const GERMANIUM_BUILD: u32 = 26000;

/// The lowest debugging port CEF accepts; it ignores a lower one.
const MIN_DEBUGGING_PORT: u16 = 1024;

/// The Chromium switches whose value is a comma separated list; two of them on one command line would make Chromium
/// keep only one, so they are merged into one switch (Pitfall 9).
const FEATURE_LIST_SWITCHES: [&str; 2] = ["enable-features", "disable-features"];

/// The legacy default switches in the legacy order, from `SE/Manage/Internal.cs`: a value of `1` is a switch that
/// stands alone in the WebView2 list. `off-screen-rendering-enabled` exists only in the CefSharp dictionary.
const LEGACY_DEFAULTS: &[(&str, &str)] = &[
    ("disable-gpu-compositing", "1"),
    ("disable-direct-write", "1"),
    ("enable-begin-frame-scheduling", "1"),
    ("disable-breakpad", "1"),
    ("disable-extensions", "1"),
    ("disable-third-party-extensions", "1"),
    ("multi-threaded-message-loop", "1"),
    ("no-sandbox", "1"),
    ("off-screen-rendering-enabled", "1"),
    ("disable-back-forward-cache", "1"),
    ("disable-web-security", "1"),
    ("disable-geolocation", "1"),
    ("disable-oor-cors", "1"),
    ("disable-surfaces", "1"),
    ("autoplay-policy", "no-user-gesture-required"),
    ("enable-media-stream", "1"),
    ("enable-accelerated-video-decode", "1"),
    ("allow-running-insecure-content", "1"),
    ("use-fake-ui-for-media-stream", "1"),
    ("enable-usermedia-screen-capture", "1"),
    ("enable-usermedia-screen-capturing", "1"),
    ("debug-plugin-loading", "1"),
    ("allow-outdated-plugins", "1"),
    ("always-authorize-plugins", "1"),
    ("enable-npapi", "1"),
    ("disable-speech-input", "1"),
    ("allow-file-access", "1"),
    ("allow-file-access-from-files", "1"),
    ("allow-file-access-from-file-urls", "1"),
    ("allow-universal-access-from-files", "1"),
    ("unsafely-disable-devtools-self-xss-warnings", "1"),
    ("disable-site-isolation-trials", "1"),
    ("disable-blink-features", "BlockCredentialedSubresources"),
    ("enable-features", "ThirdPartyStoragePartitioning,PartitionedCookies"),
    (
        "disable-features",
        "MediaEngagementBypassAutoplayPolicies,CrossSiteDocumentBlockingIfIsolating,BlockInsecurePrivateNetworkRequests,CrossSiteDocumentBlockingAlways,PreloadMediaEngagementData,HardwareMediaKeyHandling,OutOfBlinkCors,IsolateOrigins",
    ),
];

/// The only legacy default that the WebView2 list does not have.
const CEF_ONLY_DEFAULT: &str = "off-screen-rendering-enabled";

/// Why a switch name or a debugging port was refused. Every variant is a value for the caller to log, never a panic.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ArgError {
    /// The name is empty.
    #[error("a switch name cannot be empty")]
    EmptyName,
    /// The name has a character other than a letter, a digit or a dash, or starts with a dash.
    #[error("{0:?} is not a switch name: use letters, digits and dashes")]
    InvalidName(String),
    /// The debugging port plus the renderer slot is outside the range CEF accepts.
    #[error("the debugging port {base} plus slot {slot} is outside 1024..=65535")]
    PortOutOfRange {
        /// The configured `DeveloperPort`.
        base: u16,
        /// The renderer slot added to it.
        slot: u8,
    },
}

/// One Chromium command line switch: a name and, optionally, a value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Switch {
    /// The name without leading dashes, as in `autoplay-policy`.
    pub name: String,
    /// The value after the equals sign; `None` for a switch that stands alone.
    pub value: Option<String>,
}

impl Switch {
    /// A switch that stands alone.
    #[must_use]
    pub fn flag(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            value: None,
        }
    }

    /// A switch with a value.
    #[must_use]
    pub fn with_value(name: &str, value: &str) -> Self {
        Self {
            name: name.to_owned(),
            value: Some(value.to_owned()),
        }
    }

    /// The switch as the command line shows it: `--name` or `--name=value`.
    #[must_use]
    pub fn render(&self) -> String {
        match &self.value {
            Some(value) => format!("--{}={value}", self.name),
            None => format!("--{}", self.name),
        }
    }
}

/// Which web engine the switches are for: Sucrose's own switches differ (CEF takes the debugging port from its
/// settings, WebView2 only from the command line).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebEngine {
    /// The bundled Chromium Embedded Framework.
    Cef,
    /// The WebView2 runtime.
    WebView2,
}

/// What Sucrose's own switches are computed from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SucroseInputs {
    /// `Engine.HardwareAcceleration`.
    pub hardware_acceleration: bool,
    /// The debugging port this renderer listens on (the configured port plus its slot), when there is one.
    pub debugging_port: Option<u16>,
    /// The Windows build number, 0 when unknown.
    pub windows_build: u32,
    /// The engine the switches are for.
    pub engine: WebEngine,
}

/// The result of preparing the command line of a CEF browser process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CefPlan {
    /// The switches to append, in order.
    pub applied: Vec<Switch>,
    /// The CEF setting keys that were left out (see [`CEF_SETTING_KEYS`]).
    pub skipped_settings: Vec<String>,
    /// The entries whose names are not valid switch names, and why; they are left out.
    pub invalid: Vec<(String, ArgError)>,
}

/// Checks that `name` can be a switch name: letters, digits and dashes, not starting with a dash. Switches go to the
/// browser one by one through its command line API, never through a shell, so this only keeps garbage out (T-04-20).
///
/// # Errors
/// [`ArgError::EmptyName`] or [`ArgError::InvalidName`].
pub fn validate_name(name: &str) -> Result<(), ArgError> {
    if name.is_empty() {
        return Err(ArgError::EmptyName);
    }
    let valid = name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') && !name.starts_with('-');
    if valid {
        Ok(())
    } else {
        Err(ArgError::InvalidName(name.to_owned()))
    }
}

/// The names that occur more than once in `switches`, each once, in the order they first repeat.
#[must_use]
pub fn repeated_names(switches: &[Switch]) -> Vec<String> {
    let mut seen: Vec<&str> = Vec::new();
    let mut repeated: Vec<String> = Vec::new();
    for switch in switches {
        if seen.contains(&switch.name.as_str()) {
            if !repeated.contains(&switch.name) {
                repeated.push(switch.name.clone());
            }
        } else {
            seen.push(&switch.name);
        }
    }
    repeated
}

/// The legacy CefSharp dictionary as a list: a key with an empty value is a switch that stands alone. Entries are not
/// validated here (unknown switches pass through, D-17); [`validate_name`] is the caller's gate.
#[must_use]
pub fn from_cef_map(map: &BTreeMap<String, String>) -> Vec<Switch> {
    map.iter()
        .map(|(name, value)| Switch {
            name: name.clone(),
            value: (!value.is_empty()).then(|| value.clone()),
        })
        .collect()
}

/// The list as the legacy CefSharp dictionary: a switch that stands alone has an empty value, and of a repeated name
/// the last value stays.
#[must_use]
pub fn to_cef_map(switches: &[Switch]) -> BTreeMap<String, String> {
    switches
        .iter()
        .map(|switch| (switch.name.clone(), switch.value.clone().unwrap_or_default()))
        .collect()
}

/// The legacy WebView2 list as switches: `--name=value`, `--name` (one or two leading dashes) and a bare `name` all
/// parse; a blank entry is dropped. The text after the first equals sign is the value as it is.
#[must_use]
pub fn from_web_list(list: &[String]) -> Vec<Switch> {
    list.iter()
        .filter_map(|entry| {
            let trimmed = entry.trim();
            let body = trimmed
                .strip_prefix("--")
                .or_else(|| trimmed.strip_prefix('-'))
                .unwrap_or(trimmed);
            if body.is_empty() {
                return None;
            }
            Some(match body.split_once('=') {
                Some((name, value)) => Switch::with_value(name, value),
                None => Switch::flag(body),
            })
        })
        .collect()
}

/// The list as the legacy WebView2 list: `--name=value` strings, in order, repeated names kept.
#[must_use]
pub fn to_web_list(switches: &[Switch]) -> Vec<String> {
    switches.iter().map(Switch::render).collect()
}

/// The legacy default CefSharp dictionary (`SE/Manage/Internal.cs` `CefArguments`), exactly: what Reset gives and what an
/// empty dictionary means (the legacy app writes it back when the dictionary is empty). It includes the two CEF
/// setting keys of [`CEF_SETTING_KEYS`]; they are skipped when the switches are applied, not here.
#[must_use]
pub fn legacy_defaults_cef() -> BTreeMap<String, String> {
    LEGACY_DEFAULTS
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect()
}

/// The legacy default WebView2 list (`SE/Manage/Internal.cs` `WebArguments`), exactly and in order: the CefSharp defaults
/// without `off-screen-rendering-enabled`, a value of `1` written as a switch that stands alone.
#[must_use]
pub fn legacy_defaults_web() -> Vec<String> {
    LEGACY_DEFAULTS
        .iter()
        .filter(|(name, _)| *name != CEF_ONLY_DEFAULT)
        .map(|(name, value)| match *value {
            "1" => format!("--{name}"),
            other => format!("--{name}={other}"),
        })
        .collect()
}

/// Sucrose's own switches for a renderer, computed from the settings and never stored (D-17):
///
/// - hardware acceleration on adds `enable-gpu` and `enable-gpu-vsync` (`=1` for CEF, as the legacy dictionary did);
///   off adds `disable-gpu` and `disable-gpu-vsync`, for CEF only below Windows build [`GERMANIUM_BUILD`];
/// - a debugging port adds `remote-allow-origins=*` (legacy, kept under D-021); WebView2 also gets the port and its
///   DevTools feature switch, while CEF takes the port from its settings.
#[must_use]
pub fn sucrose_switches(inputs: &SucroseInputs) -> Vec<Switch> {
    let mut switches = Vec::new();
    let cef = inputs.engine == WebEngine::Cef;
    let mut gpu = |name: &str| {
        switches.push(if cef {
            Switch::with_value(name, "1")
        } else {
            Switch::flag(name)
        })
    };
    if inputs.hardware_acceleration {
        gpu("enable-gpu");
        gpu("enable-gpu-vsync");
    } else if !cef || inputs.windows_build < GERMANIUM_BUILD {
        gpu("disable-gpu");
        gpu("disable-gpu-vsync");
    }
    if let Some(port) = inputs.debugging_port {
        if inputs.engine == WebEngine::WebView2 {
            switches.push(Switch::with_value("remote-debugging-port", &port.to_string()));
        }
        switches.push(Switch::with_value("remote-allow-origins", "*"));
        if inputs.engine == WebEngine::WebView2 {
            switches.push(Switch::with_value(
                "enable-features",
                "msEdgeDevToolsWdpRemoteDebugging",
            ));
        }
    }
    switches
}

/// The debugging port of the renderer in `slot`: the configured `base` plus the slot, so no two renderers bind the same
/// port (Pitfall 11). `Ok(None)` for a base of 0 (no debugging).
///
/// # Errors
/// [`ArgError::PortOutOfRange`] when the sum overflows 65535 or lies below 1024, which CEF ignores.
pub fn debugging_port(base: u16, slot: u8) -> Result<Option<u16>, ArgError> {
    if base == 0 {
        return Ok(None);
    }
    base.checked_add(u16::from(slot))
        .filter(|port| *port >= MIN_DEBUGGING_PORT)
        .map(Some)
        .ok_or(ArgError::PortOutOfRange { base, slot })
}

/// Splits a comma separated feature list into its items, without blanks.
fn feature_items(value: Option<&str>) -> impl Iterator<Item = &str> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
}

/// Adds the items of `incoming` to the feature list switch `target`, skipping the ones it has.
fn merge_feature_list(target: &mut Switch, incoming: &Switch) {
    let mut items: Vec<String> = feature_items(target.value.as_deref()).map(str::to_owned).collect();
    for item in feature_items(incoming.value.as_deref()) {
        if !items.iter().any(|have| have == item) {
            items.push(item.to_owned());
        }
    }
    target.value = (!items.is_empty()).then(|| items.join(","));
}

/// Pushes `switch` onto `out`; a feature list switch is merged into the one already there instead of repeated.
fn push_merging_features(out: &mut Vec<Switch>, switch: &Switch) {
    if FEATURE_LIST_SWITCHES.contains(&switch.name.as_str())
        && let Some(existing) = out.iter_mut().find(|have| have.name == switch.name)
    {
        merge_feature_list(existing, switch);
        return;
    }
    let mut copy = switch.clone();
    if FEATURE_LIST_SWITCHES.contains(&copy.name.as_str()) {
        let items: Vec<&str> = feature_items(copy.value.as_deref()).collect();
        copy.value = (!items.is_empty()).then(|| items.join(","));
    }
    out.push(copy);
}

/// Merges the user's switches and Sucrose's own into one list (D-17, Pitfall 9):
///
/// - `enable-features` and `disable-features` values are merged into one switch, never repeated;
/// - a user switch that Sucrose also sets keeps Sucrose's value (and is logged), in the place of the user's first one;
/// - every other user switch stays as it is, in order, including a repeated one (Chromium takes the last).
#[must_use]
pub fn merge(user: &[Switch], sucrose: &[Switch]) -> Vec<Switch> {
    let mut merged: Vec<Switch> = Vec::with_capacity(user.len() + sucrose.len());
    for switch in user {
        push_merging_features(&mut merged, switch);
    }
    for switch in sucrose {
        if FEATURE_LIST_SWITCHES.contains(&switch.name.as_str()) {
            push_merging_features(&mut merged, switch);
            continue;
        }
        let Some(first) = merged.iter().position(|have| have.name == switch.name) else {
            merged.push(switch.clone());
            continue;
        };
        if merged[first].value != switch.value {
            tracing::warn!(
                name = %switch.name,
                user = ?merged[first].value,
                sucrose = ?switch.value,
                "a user switch is overridden by Sucrose's own"
            );
        }
        merged[first] = switch.clone();
        let mut index = first + 1;
        while index < merged.len() {
            if merged[index].name == switch.name {
                merged.remove(index);
            } else {
                index += 1;
            }
        }
    }
    merged
}

/// The user's switches that Sucrose removes for the engine and Windows build: the legacy CefSharp host drops
/// `disable-gpu-compositing` from Windows 11 24H2 (build [`GERMANIUM_BUILD`]) on.
#[must_use]
pub fn dropped_user_switches(inputs: &SucroseInputs) -> &'static [&'static str] {
    if inputs.engine == WebEngine::Cef && inputs.windows_build >= GERMANIUM_BUILD {
        &["disable-gpu-compositing"]
    } else {
        &[]
    }
}

/// The effective switches for an engine: the user's, minus the ones Sucrose drops, merged with Sucrose's own.
#[must_use]
pub fn effective_switches(user: &[Switch], inputs: &SucroseInputs) -> Vec<Switch> {
    let dropped = dropped_user_switches(inputs);
    let kept: Vec<Switch> = user
        .iter()
        .filter(|switch| !dropped.contains(&switch.name.as_str()))
        .cloned()
        .collect();
    merge(&kept, &sucrose_switches(inputs))
}

/// The result of preparing the browser arguments of a WebView2 environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebView2Plan {
    /// The switches to pass, in order.
    pub applied: Vec<Switch>,
    /// The entries whose names are not valid switch names, and why; they are left out.
    pub invalid: Vec<(String, ArgError)>,
}

/// The text of `AdditionalBrowserArguments` for `switches`: one space separated line of `--name` and `--name=value`.
/// A value with a blank or a quote is wrapped in quotes (a quote inside is escaped with a backslash, as the Windows
/// command line parser reads it), so it stays one argument; nothing is passed through a shell.
#[must_use]
pub fn browser_arguments(switches: &[Switch]) -> String {
    switches
        .iter()
        .map(|switch| match &switch.value {
            Some(value) => format!("--{}={}", switch.name, quote_value(value)),
            None => format!("--{}", switch.name),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `value` as one command line argument part: unchanged when it has no blank and no quote, else wrapped in quotes with
/// the quotes inside escaped and the backslashes before a quote (or before the closing one) doubled, the rules the
/// Windows command line parser reads back.
fn quote_value(value: &str) -> String {
    if !value.chars().any(|c| c.is_whitespace() || c == '"') {
        return value.to_owned();
    }
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    let mut backslashes = 0_usize;
    for c in value.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                quoted.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
                quoted.push('"');
                backslashes = 0;
            }
            other => {
                quoted.extend(std::iter::repeat_n('\\', backslashes));
                quoted.push(other);
                backslashes = 0;
            }
        }
    }
    quoted.extend(std::iter::repeat_n('\\', backslashes * 2));
    quoted.push('"');
    quoted
}

/// Prepares the browser arguments of a WebView2 environment: [`effective_switches`] without the names that are not
/// switch names (reported in `invalid`). Nothing else is blocked: an unknown switch with a valid name passes through
/// (D-17).
#[must_use]
pub fn plan_webview2_command_line(user: &[Switch], inputs: &SucroseInputs) -> WebView2Plan {
    let mut plan = WebView2Plan {
        applied: Vec::new(),
        invalid: Vec::new(),
    };
    for switch in effective_switches(user, inputs) {
        if let Err(error) = validate_name(&switch.name) {
            plan.invalid.push((switch.name, error));
        } else {
            plan.applied.push(switch);
        }
    }
    plan
}

/// Prepares the command line of the CEF browser process: [`effective_switches`] without the CEF setting keys (reported
/// in `skipped_settings`) and without names that are not switch names (reported in `invalid`). Nothing else is blocked:
/// an unknown switch with a valid name passes through (D-17).
#[must_use]
pub fn plan_cef_command_line(user: &[Switch], inputs: &SucroseInputs) -> CefPlan {
    let mut plan = CefPlan {
        applied: Vec::new(),
        skipped_settings: Vec::new(),
        invalid: Vec::new(),
    };
    for switch in effective_switches(user, inputs) {
        if CEF_SETTING_KEYS.contains(&switch.name.as_str()) {
            plan.skipped_settings.push(switch.name);
        } else if let Err(error) = validate_name(&switch.name) {
            plan.invalid.push((switch.name, error));
        } else {
            plan.applied.push(switch);
        }
    }
    plan
}
