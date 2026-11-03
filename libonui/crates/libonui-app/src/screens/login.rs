//! The lock screen. One account, one field, Enter to unlock.

use super::Action;
use crate::app::LoginState;
use crate::components::{self, Button, Field};
use crate::icons;
use crate::theme::{Fonts, Theme};
use egui::{Context, Key, RichText, Vec2};

pub fn show(ctx: &Context, theme: &Theme, fonts: &Fonts, state: &mut LoginState) -> Action {
    let mut action = Action::None;
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(crate::theme::c(theme.tokens.canvas)))
        .show(ctx, |ui| {
            components::wallpaper(ui, theme, libonui_core::WallpaperStyle::Aurora);

            let card_w = 420.0f32.min(ui.available_width() - 48.0);
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(ui.available_height() * 0.18);
                    components::card(ui, theme, true, |ui| {
                        ui.set_width(card_w - theme.tokens.space.lg * 2.0);
                        ui.vertical_centered(|ui| {
                            // A monogram in the accent, standing in for an avatar.
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::splat(72.0), egui::Sense::hover());
                            ui.painter().circle_filled(
                                rect.center(),
                                36.0,
                                crate::theme::c(theme.tokens.accent_soft),
                            );
                            let initial = state
                                .first_name
                                .chars()
                                .next()
                                .unwrap_or('L')
                                .to_uppercase()
                                .to_string();
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                initial,
                                fonts.title(),
                                theme.accent(),
                            );
                        });
                        ui.add_space(theme.tokens.space.md);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(format!("Ciao, {}", state.first_name))
                                    .font(fonts.title())
                                    .strong()
                                    .color(theme.text()),
                            );
                            ui.add_space(theme.tokens.space.xs);
                            ui.label(
                                RichText::new("Inserisci la password per sbloccare LibonOS")
                                    .font(fonts.label())
                                    .color(theme.text_muted()),
                            );
                        });
                        ui.add_space(theme.tokens.space.lg);

                        let resp = Field::new(&mut state.password, "Password")
                            .password()
                            .icon(&icons::LOCK)
                            .error(state.error.as_deref())
                            .show(ui, theme);
                        resp.request_focus();

                        ui.add_space(theme.tokens.space.lg);
                        if Button::new("Sblocca")
                            .primary()
                            .full_width()
                            .icon(&icons::CHEVRON_RIGHT)
                            .show(ui, theme)
                            .clicked()
                        {
                            action = Action::AttemptLogin;
                        }

                        let enter = ui.input(|i| i.key_pressed(Key::Enter));
                        if enter && !state.password.is_empty() {
                            action = Action::AttemptLogin;
                        }
                    });

                    ui.add_space(theme.tokens.space.lg);
                    ui.horizontal(|ui| {
                        components::status_dot(
                            ui,
                            theme,
                            theme.success(),
                            "Aggiornamenti automatici attivi",
                        );
                        ui.add_space(theme.tokens.space.lg);
                        components::status_dot(
                            ui,
                            theme,
                            theme.accent_alt(),
                            &libonui_core::banner(),
                        );
                    });
                },
            );
        });
    action
}
