//! Delay-loads libcef.dll into the sucrose binary and its test executable (D-32).

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    // Same targets as the `cef` dependency in Cargo.toml.
    if os == "windows" && matches!(arch.as_str(), "x86_64" | "aarch64") {
        // Delay-load libcef.dll (D-32): it is mapped on the first CEF call, so a process that never calls CEF never
        // loads it. Test executables get the same, or they would need libcef.dll beside them to start.
        println!("cargo::rustc-link-arg=/DELAYLOAD:libcef.dll");
        println!("cargo::rustc-link-lib=delayimp");
    }
}
