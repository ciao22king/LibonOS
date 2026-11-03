//! **LibonUI** — the LibonOS desktop shell.
//!
//! Run it plainly to start the session:
//!
//! ```text
//! libonui                 # setup on a fresh machine, otherwise login
//! libonui --demo          # open straight into a populated desktop
//! libonui --state <dir>   # keep state somewhere other than the user's home
//! libonui --shots <dir>   # render the screens to PNGs and exit (used for docs)
//! ```

mod app;
mod components;
mod icons;
mod screens;
mod shots;
mod store;
mod theme;

use app::{App, LaunchMode};
use libonui_core::config::StatePaths;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("libonui — la shell desktop di LibonOS");
        println!("  --demo            apre subito un desktop già avviato");
        println!("  --state <dir>     usa <dir> per account e impostazioni");
        println!("  --shots <dir>     genera le anteprime delle schermate in <dir>");
        return Ok(());
    }

    let demo = args.iter().any(|a| a == "--demo");
    let state_dir = arg_value(&args, "--state");
    let shots_dir = arg_value(&args, "--shots");

    let paths = state_dir
        .map(StatePaths::at)
        .unwrap_or_else(StatePaths::user);
    let mode = if demo || shots_dir.is_some() {
        LaunchMode::Demo
    } else {
        LaunchMode::Normal
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LibonOS")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1024.0, 680.0])
            .with_app_id("libonui"),
        ..Default::default()
    };

    if let Some(dir) = shots_dir {
        return shots::run(&dir, paths, native_options);
    }

    eframe::run_native(
        "LibonOS",
        native_options,
        Box::new(move |cc| {
            let app = App::new(paths, mode);
            theme::install_style(&cc.egui_ctx, &app.theme);
            app.install_fonts(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        screens::run(self, ctx);
        if self.script.is_some() {
            shots::tick(self, ctx);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let c = self.theme.tokens.canvas;
        [
            c.r as f32 / 255.0,
            c.g as f32 / 255.0,
            c.b as f32 / 255.0,
            1.0,
        ]
    }
}
