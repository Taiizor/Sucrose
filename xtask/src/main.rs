//! Project automation: `cargo xtask <task>` (D-014). Runs the same from PowerShell, WSL, macOS and Linux.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

mod bindgen_mpv;
mod fetch_engines;
mod localize;
mod measure;

const USAGE: &str = "usage: cargo xtask <task>

tasks:
  bindgen-mpv    regenerate the libmpv bindings from client.h (needs --features bindgen)
  ci             run the checks CI runs: fmt, clippy, test
  fetch-engines  download, verify and place pinned engine binaries (xtask/engines.toml)
  localize       convert the legacy XAML strings to the Portal's rust-i18n YAML (--lang EN, --check)
  measure        sample RAM, CPU and GPU of the running wallpaper processes (--side rust|csharp, --row)
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("bindgen-mpv") => bindgen_mpv::run(&args[2..]),
        Some("ci") => ci(),
        Some("fetch-engines") => fetch_engines::run(&args[2..]),
        Some("localize") => localize::run(&args[2..]),
        Some("measure") => measure::run(&args[2..]),
        _ => {
            eprint!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the same checks as `.github/workflows/rust.yml`, stopping at the first failure.
fn ci() -> ExitCode {
    let steps: [&[&str]; 3] = [
        &["fmt", "--all", "--check"],
        &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
        &["test", "--workspace"],
    ];
    for args in steps {
        println!("$ cargo {}", args.join(" "));
        let passed = Command::new(cargo())
            .args(args)
            .status()
            .is_ok_and(|status| status.success());
        if !passed {
            eprintln!("failed: cargo {}", args.join(" "));
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

/// The workspace root, the parent of this crate.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Cargo's target folder: `CARGO_TARGET_DIR` when set, else `<root>/target`.
fn target_dir() -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| workspace_root().join("target"), PathBuf::from)
}

/// The cargo that started xtask, so the pinned toolchain is used.
fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}
