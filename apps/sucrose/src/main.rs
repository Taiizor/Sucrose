//! The Sucrose binary (D-008): the service by default, a wallpaper renderer with `--render <slot>`.
//!
//! The renderer is the same executable started by the service, so the service itself stays small and never loads an
//! engine (D-017). Release builds use the Windows GUI subsystem, so neither mode opens a console window.
//!
//! CEF subprocesses are this executable too (D-32): CEF relaunches it with `--type=...`, and such a command line goes to
//! CEF's own entry before any Sucrose code runs. `libcef.dll` is delay-loaded (`build.rs`), so the service and the mpv
//! renderers never map it.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod fullscreen;
mod library_ops;
mod lifecycle;
mod modules;
mod portal_link;
mod render;
mod service;
mod settings_host;
mod startup;
mod supervisor;
mod topology;

use std::process::ExitCode;

use anyhow::Context as _;

/// What the command line asks for.
#[derive(Debug, PartialEq, Eq)]
enum Mode {
    /// No argument: the service, which opens the Portal at start. `--no-portal` (given by a Portal that starts the
    /// service itself, D-12) keeps it from opening another one.
    Service {
        /// Whether the service opens the Portal when it starts.
        open_portal: bool,
    },
    /// `--render <slot>`: a renderer for that slot.
    Render(u8),
}

/// Reads the arguments (without the program name). Anything but no argument, `--no-portal` or `--render <slot>` is an
/// error, so a typo in a spawn argument cannot silently start a service instead of a renderer.
fn parse_mode(args: &[String]) -> Result<Mode, String> {
    match args {
        [] => Ok(Mode::Service { open_portal: true }),
        [flag] if flag == "--no-portal" => Ok(Mode::Service { open_portal: false }),
        [flag, slot] if flag == "--render" => slot
            .parse()
            .map(Mode::Render)
            .map_err(|_| "the renderer slot must be a number from 0 to 255".to_owned()),
        _ => Err(format!(
            "unknown arguments ({} given); use no argument, --no-portal or --render <slot>",
            args.len()
        )),
    }
}

/// The exit code for a command line that cannot be understood.
const EXIT_USAGE: u8 = 64;

fn main() -> anyhow::Result<ExitCode> {
    // First of all: a CEF subprocess (`--type=...`) never reaches the log, the single-instance check or the parser.
    if let Some(code) = sucrose_engine_cef::run_if_subprocess() {
        return Ok(ExitCode::from(u8::try_from(code).unwrap_or(1)));
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_mode(&args) {
        Ok(Mode::Render(slot)) => {
            let _log = sucrose_log::init(&format!("Renderer-{slot}"))?;
            render::run(slot)
        }
        Ok(Mode::Service { open_portal }) => {
            let _log = sucrose_log::init("Service")?;
            tracing::info!(open_portal, pid = std::process::id(), "the service is starting");
            service::run(open_portal).map(|()| ExitCode::SUCCESS)
        }
        Err(message) => {
            // The release build has no console, so the log is the only place the reason can be read.
            let _log = sucrose_log::init("Service").context("cannot start the log to report a bad command line")?;
            tracing::error!(%message, "the command line is not understood, nothing was started");
            Ok(ExitCode::from(EXIT_USAGE))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn no_argument_is_the_service_and_a_render_flag_with_a_slot_is_a_renderer() {
        assert_eq!(parse_mode(&args(&[])), Ok(Mode::Service { open_portal: true }));
        assert_eq!(parse_mode(&args(&["--render", "3"])), Ok(Mode::Render(3)));
    }

    #[test]
    fn no_portal_is_the_service_that_opens_no_portal() {
        assert_eq!(
            parse_mode(&args(&["--no-portal"])),
            Ok(Mode::Service { open_portal: false })
        );
    }

    #[test]
    fn a_cef_subprocess_command_line_never_reaches_the_sucrose_parser() {
        let cef: Vec<OsString> = ["--type=gpu-process", "--lang=en-US"]
            .iter()
            .map(OsString::from)
            .collect();
        // The strict parser would refuse this command line, so only the order in `main` keeps CEF's relaunches alive.
        assert!(parse_mode(&args(&["--type=gpu-process", "--lang=en-US"])).is_err());
        assert!(sucrose_engine_cef::is_cef_subprocess(&cef));
        // Every command line the parser accepts stays Sucrose's.
        for ours in [&[][..], &["--no-portal"], &["--render", "2"]] {
            let ours: Vec<OsString> = ours.iter().map(OsString::from).collect();
            assert!(!sucrose_engine_cef::is_cef_subprocess(&ours), "{ours:?}");
        }
        // The routing order itself: the subprocess entry is called before the parser in `main`.
        let source = include_str!("main.rs");
        let body = &source[source.find("fn main()").expect("main exists")..];
        let entry = body
            .find("sucrose_engine_cef::run_if_subprocess()")
            .expect("main routes CEF");
        let parser = body.find("parse_mode(&args)").expect("main parses");
        let log = body.find("sucrose_log::init").expect("main starts the log");
        assert!(
            entry < parser && entry < log,
            "run_if_subprocess must be the first thing main does"
        );
    }

    #[test]
    fn anything_else_is_refused_instead_of_starting_the_service() {
        for bad in [
            &["--render"][..],
            &["--render", "1", "extra"],
            &["--render", "256"],
            &["--render", "x"],
            &["--rendr", "1"],
            &["something"],
            &["--no-portal", "x"],
            &["--no-portal", "--no-portal"],
            &["--no-portal", "--render", "1"],
            &["--no-porta"],
        ] {
            assert!(parse_mode(&args(bad)).is_err(), "{bad:?}");
        }
    }
}
