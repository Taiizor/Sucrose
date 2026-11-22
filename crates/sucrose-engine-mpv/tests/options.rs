//! Proves the generated mpv options and the player controls against the real libmpv that `cargo xtask fetch-engines`
//! placed under `target/engines` (ignored by default). No test opens a window: nothing is embedded and every playback
//! test uses `vo=null` and `ao=null`.
#![cfg(windows)]
// The option check talks to a raw libmpv handle before any player exists.
#![allow(unsafe_code)]

use std::{
    ffi::CString,
    path::PathBuf,
    sync::Arc,
    thread::sleep,
    time::{Duration, Instant},
};

use sucrose_core::settings::Stretch;
use sucrose_engine_mpv::{
    config::{embedding_options, generated_options},
    loader::{self, Mpv},
    player::{OpenOptions, Player},
};
use sucrose_ipc::protocol::PlaybackSettings;

/// `<target>/engines/<arch>/libmpv-2.dll`, where `fetch-engines` leaves the verified DLL.
fn fetched_dll() -> PathBuf {
    let target = std::env::var_os("CARGO_TARGET_DIR").map_or_else(
        || {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            root.ancestors()
                .nth(2)
                .expect("crate sits two folders below the workspace root")
                .join("target")
        },
        PathBuf::from,
    );
    let arch = match std::env::consts::ARCH {
        "x86" => "i686",
        other => other,
    };
    target.join("engines").join(arch).join("libmpv-2.dll")
}

fn load_mpv() -> Arc<Mpv> {
    let path = fetched_dll();
    Arc::new(loader::load(&path).unwrap_or_else(|error| panic!("{error} ({})", error.hint())))
}

fn shader_cache() -> PathBuf {
    std::env::temp_dir().join("sucrose-options-test-shaders")
}

/// Sets `options` on a fresh, never initialized handle and returns the first refusal as `name=value: message`.
fn first_refusal(mpv: &Mpv, options: &[(&str, String)]) -> Option<String> {
    // SAFETY: mpv_create has no preconditions; NULL is checked right below.
    let handle = unsafe { mpv.api.mpv_create() };
    assert!(!handle.is_null(), "mpv_create failed");
    let mut refusal = None;
    for (name, value) in options {
        let c_name = CString::new(*name).expect("option names have no NUL");
        let c_value = CString::new(value.as_str()).expect("option values have no NUL");
        // SAFETY: the handle is valid and not initialized; both strings are NUL-terminated and outlive the call.
        let status = unsafe { mpv.api.mpv_set_option_string(handle, c_name.as_ptr(), c_value.as_ptr()) };
        if status < 0 {
            // SAFETY: mpv_error_string accepts any code and returns a static NUL-terminated string.
            let message = unsafe { std::ffi::CStr::from_ptr(mpv.api.mpv_error_string(status)) };
            refusal = Some(format!("{name}={value}: {}", message.to_string_lossy()));
            break;
        }
    }
    // SAFETY: the handle is valid and never used again.
    unsafe { mpv.api.mpv_terminate_destroy(handle) };
    refusal
}

/// A player on a generated test stream without a window: `vo=null`, `ao=null`, the options of `settings`.
fn start(settings: PlaybackSettings, seconds: u32) -> Player {
    let mut embedding: Vec<_> = embedding_options()
        .into_iter()
        .filter(|(name, _)| !matches!(*name, "vo" | "gpu-context"))
        .collect();
    embedding.push(("vo", "null".to_owned()));
    embedding.push(("ao", "null".to_owned()));
    let generated = generated_options(&settings, &shader_cache());
    let options = OpenOptions {
        embedding: &embedding,
        generated: &generated,
        user_config: None,
    };
    let source = format!("av://lavfi:testsrc=duration={seconds}:size=320x240:rate=30");
    Player::open_source(load_mpv(), None, &source, &options)
        .unwrap_or_else(|error| panic!("the test stream cannot be played: {error}"))
}

fn time_pos(player: &Player) -> Option<f64> {
    player.get_property_string("time-pos")?.parse().ok()
}

/// Polls `check` every 20 ms until it returns true or `limit` passes.
fn wait_until(limit: Duration, mut check: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < limit {
        if check() {
            return true;
        }
        sleep(Duration::from_millis(20));
    }
    false
}

#[test]
#[ignore = "needs cargo xtask fetch-engines"]
fn every_generated_option_is_accepted_by_the_pinned_libmpv() {
    let mpv = load_mpv();

    // The check must be able to fail: an option that does not exist is refused.
    assert!(
        first_refusal(&mpv, &[("sucrose-no-such-option", "yes".to_owned())]).is_some(),
        "libmpv accepted an option that does not exist, so this test proves nothing"
    );

    for stretch in Stretch::ALL {
        for hardware_decoding in [true, false] {
            for looping in [true, false] {
                for volume in [0, 100] {
                    let settings = PlaybackSettings {
                        volume,
                        looping,
                        stretch: *stretch,
                        hardware_decoding,
                    };
                    let mut options = generated_options(&settings, &shader_cache());
                    options.extend(embedding_options());
                    let options: Vec<(&str, String)> = options;
                    if let Some(refusal) = first_refusal(&mpv, &options) {
                        panic!("libmpv refused {refusal} for {settings:?}");
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "needs cargo xtask fetch-engines"]
fn loop_file_inf_restarts_the_stream_at_its_end() {
    let player = start(PlaybackSettings::default(), 2);

    let reached_the_end = wait_until(Duration::from_secs(20), || {
        time_pos(&player).is_some_and(|pos| pos > 1.5)
    });
    assert!(reached_the_end, "the stream never got past 1.5 s");
    let looped = wait_until(Duration::from_secs(20), || {
        time_pos(&player).is_some_and(|pos| pos < 0.5)
    });
    assert!(looped, "the stream did not restart after its end");
}

#[test]
#[ignore = "needs cargo xtask fetch-engines"]
fn pause_stops_the_clock_and_resume_restarts_it() {
    let player = start(PlaybackSettings::default(), 120);
    assert!(
        wait_until(Duration::from_secs(20), || time_pos(&player)
            .is_some_and(|pos| pos > 0.3)),
        "the stream never started"
    );

    player.set_paused(true).expect("pause");
    assert_eq!(player.get_property_string("pause").as_deref(), Some("yes"));
    sleep(Duration::from_millis(200));
    let paused_at = time_pos(&player).expect("time-pos while paused");
    sleep(Duration::from_millis(500));
    let still = time_pos(&player).expect("time-pos while paused");
    assert!(
        (still - paused_at).abs() < 1e-6,
        "time-pos moved from {paused_at} to {still} while paused"
    );

    player.set_paused(false).expect("resume");
    assert_eq!(player.get_property_string("pause").as_deref(), Some("no"));
    assert!(
        wait_until(Duration::from_secs(10), || time_pos(&player)
            .is_some_and(|pos| pos > still + 0.2)),
        "time-pos did not advance after resuming"
    );
}

#[test]
#[ignore = "needs cargo xtask fetch-engines"]
fn set_volume_changes_the_volume_property() {
    let player = start(PlaybackSettings::default(), 120);

    player.set_volume(40).expect("set the volume");

    let volume: f64 = player
        .get_property_string("volume")
        .expect("the volume property")
        .parse()
        .expect("a number");
    assert!((volume - 40.0).abs() < 1e-6, "volume is {volume}");
}
