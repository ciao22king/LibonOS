//! The screens, and the frame that dispatches between them.

pub mod desktop;
pub mod files;
pub mod login;
pub mod settings;
pub mod setup;
pub mod store;
pub mod terminal;
pub mod text_editor;

use crate::app::{App, Stage};
use egui::Context;

/// A request a screen makes of the shell, applied after the frame is drawn so a
/// screen never needs `&mut App` while it is borrowing its own state.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    /// Finish first-run setup and open the desktop.
    CompleteSetup,
    /// Try the typed password.
    AttemptLogin,
    /// Return to the login screen.
    LogOut,
    /// Close the session.
    Exit,
}

/// Draw one frame.
pub fn run(app: &mut App, ctx: &Context) {
    let action = {
        // Disjoint field borrows: the screen gets exactly what it needs.
        let App {
            stage,
            theme,
            fonts,
            should_exit,
            ..
        } = app;
        match stage {
            Stage::Setup(setup_state) => setup::show(ctx, theme, fonts, setup_state),
            Stage::Login(login_state) => login::show(ctx, theme, fonts, login_state),
            Stage::Desktop(desktop) => desktop::show(ctx, theme, fonts, desktop, should_exit),
        }
    };

    match action {
        Action::None => {}
        Action::CompleteSetup => app.complete_setup(),
        Action::AttemptLogin => app.attempt_login(),
        Action::LogOut => {
            app.save_config();
            let first_name = app
                .paths
                .load_account()
                .map(|a| a.first_name)
                .unwrap_or_else(|| "utente".into());
            app.stage = Stage::Login(crate::app::LoginState {
                password: String::new(),
                error: None,
                first_name,
                attempts: 0,
            });
        }
        Action::Exit => app.should_exit = true,
    }

    // A theme change during the frame should take effect on the next one.
    app.refresh_theme();
    if app.should_exit {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}
