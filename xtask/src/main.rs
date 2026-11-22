//! Project automation: `cargo xtask <task>` (D-014). Runs the same from PowerShell, WSL, macOS and Linux.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::process::{Command, ExitCode};

mod fetch_engines;

const USAGE: &str = "usage: cargo xtask <task>

tasks:
  ci             run the checks CI runs: fmt, clippy, test
  fetch-engines  download, verify and place pinned engine binaries (xtask/engines.toml)
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("ci") => ci(),
        Some("fetch-engines") => fetch_engines::run(&args[2..]),
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

/// The cargo that started xtask, so the pinned toolchain is used.
fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}
