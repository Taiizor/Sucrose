//! The Sucrose binary (D-008): the service by default, a wallpaper renderer with `--render <slot>`.
//!
//! The renderer is the same executable started by the service, so the service itself stays small and never loads an
//! engine (D-017). Release builds use the Windows GUI subsystem, so neither mode opens a console window.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod render;
mod service;

use anyhow::Context as _;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [flag, slot] if flag == "--render" => {
            let slot: u8 = slot
                .parse()
                .context("the renderer slot must be a number from 0 to 255")?;
            let _log = sucrose_log::init(&format!("Renderer-{slot}"))?;
            render::run(slot)
        }
        _ => {
            let _log = sucrose_log::init("Service")?;
            service::run()
        }
    }
}
