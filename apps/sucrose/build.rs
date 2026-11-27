//! Delay-loads libcef.dll into the sucrose binary and its test executable (D-32), and embeds the Windows application
//! manifest of the sucrose binary.

use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if os == "windows" {
        embed_manifest();
    }
    // Same targets as the `cef` dependency in Cargo.toml.
    if os == "windows" && matches!(arch.as_str(), "x86_64" | "aarch64") {
        // Delay-load libcef.dll (D-32): it is mapped on the first CEF call, so a process that never calls CEF never
        // loads it. Test executables get the same, or they would need libcef.dll beside them to start.
        println!("cargo::rustc-link-arg=/DELAYLOAD:libcef.dll");
        println!("cargo::rustc-link-lib=delayimp");
    }
}

/// Embeds `resources/windows/sucrose.manifest.xml` into the sucrose binary with the MSVC linker. CEF also runs this
/// executable as its GPU, renderer and utility subprocesses (D-036), and Chromium's GPU process needs the Windows 10
/// `supportedOS` entry that CEF's own sample executables carry: without it the GPU process crashes on its first command
/// buffer, Chromium turns GL off after three crashes and no page gets WebGL. The test executables need no manifest.
fn embed_manifest() {
    let manifest: PathBuf = [
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default().as_str(),
        "resources",
        "windows",
        "sucrose.manifest.xml",
    ]
    .iter()
    .collect();
    println!("cargo::rerun-if-changed={}", manifest.display());
    if std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() != "msvc" {
        println!(
            "cargo::warning=the Windows application manifest is embedded by the MSVC linker only; this sucrose binary \
             has none, so CEF's GPU subprocess will crash and web wallpapers get no WebGL"
        );
        return;
    }
    // The linker merges the input into the manifest it writes and stores the result as resource 1 of the binary. The
    // input carries its own trustInfo (asInvoker), so the linker must not add a second one.
    println!("cargo::rustc-link-arg-bin=sucrose=/MANIFEST:EMBED");
    println!(
        "cargo::rustc-link-arg-bin=sucrose=/MANIFESTINPUT:{}",
        manifest.display()
    );
    println!("cargo::rustc-link-arg-bin=sucrose=/MANIFESTUAC:NO");
}
