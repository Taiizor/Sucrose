//! The mpv options of a wallpaper: what never changes for an embedded instance and what follows the user's settings.
//!
//! The generated set is re-derived for the pinned mpv (v0.42.0-38) from the legacy MpvPlayer.config, not copied: keys
//! that no longer exist or mean nothing when embedded are left out (RESEARCH "Pattern 2"). Option names come from
//! constants and values from typed settings, so no user text ever becomes an option name or value (T-02-24).

use std::path::{Path, PathBuf};

use sucrose_core::settings::Stretch;
use sucrose_ipc::protocol::PlaybackSettings;

/// One mpv option: its name and its value as mpv_set_option_string takes them.
pub type MpvOption = (&'static str, String);

/// Every option name this module can emit. A new name needs a check against `mpv --list-options` of the pinned build
/// first (the ignored test in tests/options.rs applies them all to the real libmpv).
pub const ALLOWED_OPTIONS: &[&str] = &[
    // embedding_options
    "vo",
    "gpu-context",
    "idle",
    "keep-open",
    "stop-screensaver",
    "media-controls",
    "osc",
    "input-default-bindings",
    "input-vo-keyboard",
    "input-cursor",
    "cursor-autohide",
    "border",
    "load-scripts",
    "ytdl",
    "terminal",
    "config",
    // generated_options
    "profile",
    "hwdec",
    "framedrop",
    "video-sync",
    "cache",
    "demuxer-max-bytes",
    "demuxer-seekable-cache",
    "volume",
    "volume-max",
    "audio-channels",
    "audio-samplerate",
    "audio-buffer",
    "loop-file",
    "deband",
    "interpolation",
    "scale-antiring",
    "temporal-dither",
    "gpu-shader-cache",
    "gpu-shader-cache-dir",
    "panscan",
    "keepaspect",
    "video-unscaled",
];

/// The file name of the user's own mpv config in the C# app's mpv folder (D-10).
const USER_CONFIG_FILE: &str = "uMpvPlayer.config";

fn option(name: &'static str, value: &str) -> MpvOption {
    (name, value.to_owned())
}

/// The options every embedded instance gets, whatever the user configured: hardening for a headless instance (no
/// user config, scripts, URL resolver, UI or input) and the embedding itself. They are applied after a user config
/// so that it cannot turn them off (T-02-14). `wid` is not part of the list: the player sets it from its argument.
#[must_use]
pub fn embedding_options() -> Vec<MpvOption> {
    let mut options = vec![
        option("config", "no"),
        option("load-scripts", "no"),
        option("ytdl", "no"),
        option("osc", "no"),
        option("input-default-bindings", "no"),
        option("input-vo-keyboard", "no"),
        option("input-cursor", "no"),
        option("cursor-autohide", "no"),
        option("border", "no"),
        option("terminal", "no"),
        option("vo", "gpu"),
        option("idle", "yes"),
        option("keep-open", "no"),
        option("stop-screensaver", "no"),
        option("media-controls", "no"),
    ];
    if cfg!(windows) {
        options.push(option("gpu-context", "d3d11"));
    }
    options
}

/// The look, performance and audio options for `settings`; the shader cache lives in `shader_cache`, a folder of
/// Sucrose's own data (never the C# app's). Applied when the user has no mpv config of their own.
#[must_use]
pub fn generated_options(settings: &PlaybackSettings, shader_cache: &Path) -> Vec<MpvOption> {
    // The profile comes first so that every explicit option after it wins.
    let mut options = vec![
        option("profile", "low-latency"),
        option("hwdec", if settings.hardware_decoding { "auto-safe" } else { "no" }),
        option("framedrop", "decoder"),
        option("video-sync", "audio"),
        option("cache", "yes"),
        option("demuxer-max-bytes", "64M"),
        option("demuxer-seekable-cache", "yes"),
        ("volume", settings.volume.min(100).to_string()),
        option("volume-max", "100"),
        option("audio-channels", "stereo"),
        option("audio-samplerate", "48000"),
        option("audio-buffer", "0.1"),
        option("loop-file", if settings.looping { "inf" } else { "no" }),
        option("deband", "yes"),
        option("interpolation", "no"),
        option("scale-antiring", "0"),
        option("temporal-dither", "no"),
    ];
    options.extend(shader_cache_options(shader_cache));
    options.extend(stretch_options(settings.stretch));
    options
}

/// The shader cache options for the folder `shader_cache`. The renderer adds them to the embedding options too, so
/// that a user config cannot send the cache into the C# app's folder (D-08).
#[must_use]
pub fn shader_cache_options(shader_cache: &Path) -> Vec<MpvOption> {
    // mpv would fall back to its own default folder for a path it cannot be given, so the cache is off instead.
    match shader_cache.to_str() {
        Some(dir) if !dir.contains('\0') => {
            vec![option("gpu-shader-cache", "yes"), option("gpu-shader-cache-dir", dir)]
        }
        _ => vec![option("gpu-shader-cache", "no")],
    }
}

/// The stretch mapping of the legacy player (docs/legacy/engines.md section 3): `panscan` only for UniformToFill,
/// `keepaspect` only for Fill, `video-unscaled` only for None, and mpv's own fit for Uniform.
fn stretch_options(stretch: Stretch) -> Option<MpvOption> {
    match stretch {
        Stretch::UniformToFill => Some(option("panscan", "1.0")),
        Stretch::Fill => Some(option("keepaspect", "no")),
        Stretch::None => Some(option("video-unscaled", "yes")),
        Stretch::Uniform => None,
    }
}

/// The user's own mpv config inside `legacy_mpv_dir` (the C# app's `Cache\MpvPlayer`), when it exists. Only its
/// path is returned: the file is read by mpv and never written (D-08, D-10).
#[must_use]
pub fn user_config_path(legacy_mpv_dir: &Path) -> Option<PathBuf> {
    let path = legacy_mpv_dir.join(USER_CONFIG_FILE);
    path.is_file().then_some(path)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn settings(stretch: Stretch) -> PlaybackSettings {
        PlaybackSettings {
            stretch,
            ..PlaybackSettings::default()
        }
    }

    fn generated(settings: &PlaybackSettings) -> Vec<MpvOption> {
        generated_options(settings, Path::new("cache"))
    }

    fn value_of(options: &[MpvOption], name: &str) -> Option<String> {
        options
            .iter()
            .find_map(|(option, value)| (*option == name).then(|| value.clone()))
    }

    const STRETCH_NAMES: [&str; 3] = ["panscan", "keepaspect", "video-unscaled"];

    #[test]
    fn each_stretch_value_emits_exactly_its_legacy_option() {
        for (stretch, expected) in [
            (Stretch::UniformToFill, Some(("panscan", "1.0"))),
            (Stretch::Fill, Some(("keepaspect", "no"))),
            (Stretch::None, Some(("video-unscaled", "yes"))),
            (Stretch::Uniform, None),
        ] {
            let options = generated(&settings(stretch));
            let emitted: Vec<_> = options
                .iter()
                .filter(|(name, _)| STRETCH_NAMES.contains(name))
                .map(|(name, value)| (*name, value.as_str()))
                .collect();
            assert_eq!(emitted, expected.into_iter().collect::<Vec<_>>(), "{stretch:?}");
        }
    }

    #[test]
    fn volume_is_passed_through_at_both_ends() {
        for volume in [0_u8, 100] {
            let options = generated(&PlaybackSettings {
                volume,
                ..PlaybackSettings::default()
            });
            assert_eq!(value_of(&options, "volume"), Some(volume.to_string()));
        }
    }

    #[test]
    fn volume_above_the_range_is_clamped() {
        let options = generated(&PlaybackSettings {
            volume: 250,
            ..PlaybackSettings::default()
        });
        assert_eq!(value_of(&options, "volume").as_deref(), Some("100"));
    }

    #[test]
    fn looping_maps_to_loop_file() {
        let on = generated(&PlaybackSettings::default());
        let off = generated(&PlaybackSettings {
            looping: false,
            ..PlaybackSettings::default()
        });
        assert_eq!(value_of(&on, "loop-file").as_deref(), Some("inf"));
        assert_eq!(value_of(&off, "loop-file").as_deref(), Some("no"));
    }

    #[test]
    fn hardware_decoding_maps_to_hwdec() {
        let on = generated(&PlaybackSettings::default());
        let off = generated(&PlaybackSettings {
            hardware_decoding: false,
            ..PlaybackSettings::default()
        });
        assert_eq!(value_of(&on, "hwdec").as_deref(), Some("auto-safe"));
        assert_eq!(value_of(&off, "hwdec").as_deref(), Some("no"));
        // hwdec-codecs=all only with measured evidence (A19).
        assert_eq!(value_of(&on, "hwdec-codecs"), None);
    }

    #[test]
    fn the_shader_cache_goes_to_the_given_folder() {
        let options = generated_options(&PlaybackSettings::default(), Path::new("data/Cache/mpv-shaders"));
        assert_eq!(
            value_of(&options, "gpu-shader-cache-dir").as_deref(),
            Some("data/Cache/mpv-shaders")
        );
    }

    #[test]
    fn embedding_options_harden_the_instance() {
        let options = embedding_options();
        for (name, expected) in [
            ("load-scripts", "no"),
            ("ytdl", "no"),
            ("config", "no"),
            ("input-default-bindings", "no"),
            ("input-vo-keyboard", "no"),
            ("osc", "no"),
        ] {
            assert_eq!(
                value_of(&options, name).as_deref(),
                Some(expected),
                "{name} must be hardened"
            );
        }
        assert_eq!(value_of(&options, "wid"), None, "Player::open sets wid itself");
    }

    #[test]
    fn no_option_is_emitted_twice() {
        for stretch in Stretch::ALL {
            let mut all = embedding_options();
            all.extend(generated(&settings(*stretch)));
            let names: HashSet<_> = all.iter().map(|(name, _)| *name).collect();
            assert_eq!(names.len(), all.len(), "an option is emitted twice for {stretch:?}");
        }
    }

    #[test]
    fn only_allow_listed_names_are_emitted() {
        for stretch in Stretch::ALL {
            for (volume, looping, hardware_decoding) in [(0, false, false), (100, true, true)] {
                let mut all = embedding_options();
                all.extend(generated(&PlaybackSettings {
                    volume,
                    looping,
                    stretch: *stretch,
                    hardware_decoding,
                }));
                for (name, _) in all {
                    assert!(ALLOWED_OPTIONS.contains(&name), "{name} is not on the allow-list");
                }
            }
        }
    }

    #[test]
    fn the_user_config_is_found_only_as_a_file() {
        let dir = std::env::temp_dir().join(format!("sucrose-config-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create the scratch folder");

        assert_eq!(user_config_path(&dir), None, "no file yet");
        std::fs::create_dir(dir.join(USER_CONFIG_FILE)).expect("create a folder with the config's name");
        assert_eq!(user_config_path(&dir), None, "a folder is not a config");
        std::fs::remove_dir(dir.join(USER_CONFIG_FILE)).expect("remove the folder");
        std::fs::write(dir.join(USER_CONFIG_FILE), "volume=50\n").expect("write the config");
        assert_eq!(user_config_path(&dir), Some(dir.join(USER_CONFIG_FILE)));

        std::fs::remove_dir_all(&dir).expect("remove the scratch folder");
    }
}
