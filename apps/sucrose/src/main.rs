//! The Sucrose binary (D-008): the service by default, a wallpaper renderer with `--render <slot>`.
//!
//! The renderer is the same executable started by the service, so the service itself stays small and never loads an
//! engine (D-017). Release builds use the Windows GUI subsystem, so neither mode opens a console window.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod fullscreen;
mod portal_link;
mod render;
mod service;
mod settings_host;
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_mode(&args) {
        Ok(Mode::Render(slot)) => {
            let _log = sucrose_log::init(&format!("Renderer-{slot}"))?;
            render::run(slot)
        }
        Ok(Mode::Service { open_portal }) => {
            let _log = sucrose_log::init("Service")?;
            // Plan 15 acts on the flag; until then the service never opens a Portal itself.
            tracing::info!(open_portal, "the service is starting");
            service::run().map(|()| ExitCode::SUCCESS)
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
