//! The shared Restart wallpapers prompt (UI-SPEC "Restart prompt", E12, D-17). A setting that a renderer reads only when it
//! starts (the certificate switch, the CEF arguments, Hardware Acceleration) is saved first; when a wallpaper that reads it
//! runs, this dialog asks whether to restart those wallpapers now. Restart now restarts only the wallpapers on the
//! affected engines, Later, Escape and a click outside keep the saved change for the next start. Nothing here restarts
//! anything by itself.

use gpui_kit::component::{
    WindowExt as _,
    button::{Button, ButtonVariants as _},
    dialog::DialogFooter,
};
use gpui_kit::*;
use sucrose_core::settings::EngineType;
use sucrose_ipc::portal::PortalRequest;

use super::{DIALOG_WIDTH, body};
use crate::{
    i18n::{tr, tr_with},
    ipc::ServiceLink,
    settings::wallpaper::engine_label_key,
};

/// The engines in the order the prompt names them: the web engines first, as the settings page lists them.
const ENGINE_ORDER: [EngineType; 8] = [
    EngineType::WebViewLive,
    EngineType::CefSharpLive,
    EngineType::MpvPlayerLive,
    EngineType::VlcPlayerLive,
    EngineType::NebulaLive,
    EngineType::VexanaLive,
    EngineType::XavierLive,
    EngineType::AuroraLive,
];

/// A setting that raises the prompt: one a renderer reads when it starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartSetting {
    #[allow(dead_code, reason = "raised by the arguments editor of the next task")]
    /// The user's switches of one web engine; only wallpapers on that engine read them.
    Arguments(EngineType),
    /// Allow invalid certificates; both web engines read it.
    Certificates,
    /// Hardware Acceleration; every engine reads it.
    HardwareAcceleration,
}

impl RestartSetting {
    /// Whether a wallpaper on `engine` reads this setting when it starts.
    pub fn read_by(self, engine: EngineType) -> bool {
        match self {
            Self::Arguments(own) => engine == own,
            Self::Certificates => matches!(engine, EngineType::WebViewLive | EngineType::CefSharpLive),
            Self::HardwareAcceleration => true,
        }
    }

    /// The key of the text that names the setting in the body ("The certificate setting").
    fn key(self) -> &'static str {
        match self {
            Self::Arguments(_) => "Portal.RestartPrompt.Setting.Arguments",
            Self::Certificates => "Portal.RestartPrompt.Setting.Certificates",
            Self::HardwareAcceleration => "Portal.RestartPrompt.Setting.HardwareAcceleration",
        }
    }
}

/// The engines among `running` whose wallpapers read `setting`, each once, in the order the prompt names them.
pub fn affected_engines(setting: RestartSetting, running: &[EngineType]) -> Vec<EngineType> {
    ENGINE_ORDER
        .into_iter()
        .filter(|engine| running.contains(engine) && setting.read_by(*engine))
        .collect()
}

/// Whether a committed change raises the prompt: the committed value differs from the stored one and at least one
/// wallpaper that reads the setting runs (E12: repeat and nothing-affected rows).
pub fn should_prompt<T: PartialEq>(setting: RestartSetting, committed: &T, stored: &T, running: &[EngineType]) -> bool {
    committed != stored && !affected_engines(setting, running).is_empty()
}

/// The engines as the body names them: "CEF", "WebView2 and CEF", "A, B and C". The built-in Gif engines share a label,
/// which is named once.
pub fn engines_text(engines: &[EngineType]) -> String {
    let mut names: Vec<String> = Vec::new();
    for engine in engines {
        let name = tr(engine_label_key(*engine)).to_string();
        if !names.contains(&name) {
            names.push(name);
        }
    }
    match names.as_slice() {
        [] => String::new(),
        [only] => only.clone(),
        [head @ .., last] => tr_with(
            "Portal.RestartPrompt.Engines.Both",
            &[("first", &head.join(", ")), ("second", last)],
        )
        .to_string(),
    }
}

/// The body of the prompt: "The certificate setting applies when the CEF wallpapers restart."
pub fn prompt_text(setting: RestartSetting, engines: &[EngineType]) -> String {
    tr_with(
        "Portal.RestartPrompt.Body",
        &[("setting", &tr(setting.key())), ("engines", &engines_text(engines))],
    )
    .to_string()
}

/// Opens the prompt for `setting`; `engines` are the affected engines (see [`affected_engines`]). Restart now sends the
/// restart for exactly those engines and closes the dialog; Later closes it. Escape and a click outside mean Later, and
/// the dialog layer returns the focus to the control that raised it.
pub fn open_restart_prompt(
    setting: RestartSetting,
    engines: Vec<EngineType>,
    link: ServiceLink,
    window: &mut Window,
    cx: &mut App,
) {
    let text = prompt_text(setting, &engines);
    let restart_focus = cx.focus_handle();
    let focus_on_open = restart_focus.clone();
    window.open_dialog(cx, move |dialog, window, _cx| {
        let link = link.clone();
        let engines = engines.clone();
        dialog
            .title(tr("Portal.RestartPrompt.Title"))
            .w(px(DIALOG_WIDTH))
            .child(body(window, [div().text_sm().child(text.clone()).into_any_element()]))
            .footer(
                DialogFooter::new()
                    .child(
                        Button::new("restart-later")
                            .outline()
                            .label(tr("Portal.RestartPrompt.Later"))
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("restart-now")
                            .primary()
                            .label(tr("Portal.RestartPrompt.Now"))
                            .track_focus(&restart_focus)
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                link.send(PortalRequest::RestartWallpapers {
                                    engines: engines.clone(),
                                });
                            }),
                    ),
            )
    });
    // Restart now is the focused button when the dialog opens (UI-SPEC).
    window.focus(&focus_on_open, cx);
}

#[cfg(test)]
mod tests {
    use sucrose_core::settings::EngineType::{CefSharpLive, MpvPlayerLive, NebulaLive, VexanaLive, WebViewLive};

    use super::{RestartSetting, affected_engines, engines_text, prompt_text, should_prompt};

    #[test]
    fn restart_the_body_names_the_setting_and_the_engines() {
        assert_eq!(
            prompt_text(RestartSetting::Certificates, &[CefSharpLive]),
            "The certificate setting applies when the CEF wallpapers restart."
        );
        assert_eq!(
            prompt_text(RestartSetting::Certificates, &[WebViewLive, CefSharpLive]),
            "The certificate setting applies when the WebView2 and CEF wallpapers restart."
        );
        assert_eq!(
            prompt_text(RestartSetting::Arguments(CefSharpLive), &[CefSharpLive]),
            "The new arguments applies when the CEF wallpapers restart."
        );
        assert_eq!(
            prompt_text(RestartSetting::HardwareAcceleration, &[MpvPlayerLive]),
            "The hardware acceleration setting applies when the mpv wallpapers restart."
        );
    }

    #[test]
    fn restart_three_engines_are_listed_with_commas_and_the_built_in_gif_engines_once() {
        assert_eq!(
            engines_text(&[WebViewLive, CefSharpLive, MpvPlayerLive]),
            "WebView2, CEF and mpv"
        );
        assert_eq!(engines_text(&[VexanaLive, VexanaLive]), engines_text(&[VexanaLive]));
        assert_eq!(engines_text(&[]), "");
    }

    #[test]
    fn restart_only_the_engines_that_read_the_setting_are_affected_in_a_fixed_order() {
        let running = [MpvPlayerLive, CefSharpLive, WebViewLive, CefSharpLive];

        assert_eq!(
            affected_engines(RestartSetting::Certificates, &running),
            [WebViewLive, CefSharpLive],
            "the certificate switch is read by the web engines, each named once"
        );
        assert_eq!(
            affected_engines(RestartSetting::HardwareAcceleration, &running),
            [WebViewLive, CefSharpLive, MpvPlayerLive]
        );
        assert_eq!(
            affected_engines(RestartSetting::Arguments(CefSharpLive), &running),
            [CefSharpLive],
            "the CEF arguments restart only the CEF wallpapers"
        );
        assert!(affected_engines(RestartSetting::Certificates, &[MpvPlayerLive, NebulaLive]).is_empty());
    }

    #[test]
    fn restart_the_prompt_needs_a_changed_value_and_a_running_wallpaper_that_reads_it() {
        let running = [CefSharpLive];

        assert!(should_prompt(RestartSetting::Certificates, &true, &false, &running));
        assert!(
            !should_prompt(RestartSetting::Certificates, &true, &true, &running),
            "a committed value equal to the stored one raises nothing"
        );
        assert!(
            !should_prompt(RestartSetting::Certificates, &true, &false, &[]),
            "nothing runs: the change applies at the next start"
        );
        assert!(
            !should_prompt(RestartSetting::Certificates, &true, &false, &[MpvPlayerLive]),
            "a player does not read the certificate switch"
        );
        assert!(should_prompt(
            RestartSetting::HardwareAcceleration,
            &false,
            &true,
            &[MpvPlayerLive]
        ));
        assert!(!should_prompt(
            RestartSetting::Arguments(CefSharpLive),
            &vec![1],
            &vec![1],
            &running
        ));
    }
}
