//! `libon` — the LibonOS package and update manager.
//!
//! Everyday use:
//!
//! ```text
//! libon install firefox      # look the app up, download the AppImage, run it
//! libon search notes         # search the catalog
//! libon list                 # what is installed
//! libon remove obsidian      # uninstall
//! libon update               # in-place OS + app update
//! libon upgrade              # explain how a full system upgrade is done
//! libon status               # versions and what an update would do
//! ```
//!
//! A **full system upgrade** — crossing a major version such as 1.0.0 → 2.0.0 —
//! is deliberately *not* available here. It rewrites the system image and is
//! offered only from Settings, so a stray command can never rebuild the machine
//! underneath the user.

mod commands;
mod net;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match commands::run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => code,
    }
}
