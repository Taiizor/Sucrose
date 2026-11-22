//! Loads the libmpv that `cargo xtask fetch-engines` placed under `target/engines` (ignored by default).
#![cfg(windows)]

use std::path::PathBuf;

use sucrose_engine_mpv::loader;

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

#[test]
#[ignore = "needs cargo xtask fetch-engines"]
fn loads_the_fetched_libmpv_by_absolute_path() {
    let path = fetched_dll();
    let mpv = loader::load(&path).unwrap_or_else(|error| panic!("{error} ({})", error.hint()));
    assert_eq!(mpv.path, path);
    assert_eq!(mpv.client_api.0, 2, "client API major version");
    assert!(
        mpv.client_api.1 >= 6,
        "the pinned build reports client API 2.6, got {:?}",
        mpv.client_api
    );
}
