//! The sucrose binary carries its Windows application manifest (build.rs). CEF also runs this binary as its GPU
//! subprocess (D-036), and without the Windows 10 compatibility entry that process crashes on its first command buffer,
//! so no web wallpaper gets WebGL (debug session cef-web-themes-stuck-loading).

#![cfg(windows)]

/// Whether `needle` occurs in `haystack`.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

#[test]
fn the_sucrose_binary_declares_windows_10_compatibility() {
    let binary = std::fs::read(env!("CARGO_BIN_EXE_sucrose")).expect("read the sucrose binary");

    assert!(
        contains(&binary, b"urn:schemas-microsoft-com:compatibility.v1"),
        "the binary has no compatibility section in an embedded manifest"
    );
    assert!(
        contains(&binary, b"{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"),
        "the manifest does not declare Windows 10 (and 11) support, which CEF's GPU process needs"
    );
}
