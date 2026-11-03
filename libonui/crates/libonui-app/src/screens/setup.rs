//! First-run setup: the questions LibonOS asks before the desktop appears.
//!
//! One decision per screen, a visible step rail, and a plain reason whenever the
//! Next button refuses to advance.

use super::Action;
use crate::components::{self, Button, Field};
use crate::icons;
use crate::theme::{Fonts, Theme};
use egui::{Context, RichText, Vec2};
use libonui_core::onboarding::{SetupState, SetupStep};
use libonui_core::packages::PackageManagerKind;

pub fn show(ctx: &Context, theme: &Theme, fonts: &Fonts, state: &mut SetupState) -> Action {
    let mut action = Action::None;
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(crate::theme::c(theme.tokens.canvas)))
        .show(ctx, |ui| {
            components::wallpaper(ui, theme, libonui_core::WallpaperStyle::Aurora);

            ui.vertical_centered(|ui| {
                ui.add_space(theme.tokens.space.xl);
                ui.label(
                    RichText::new("LibonOS")
                        .font(fonts.display())
                        .strong()
                        .color(theme.text()),
                );
                ui.add_space(theme.tokens.space.xs);
                ui.label(
                    RichText::new("Configurazione iniziale")
                        .font(fonts.label())
                        .color(theme.text_muted()),
                );
                ui.add_space(theme.tokens.space.lg);
            });

            let card_w = 620.0f32.min(ui.available_width() - 48.0);
            ui.allocate_ui_with_layout(
                Vec2::new(card_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    // One authored entrance: the whole panel rises once and
                    // settles. Not a per-element effect.
                    let t = ui.ctx().animate_bool_with_time(
                        egui::Id::new("setup-entrance"),
                        true,
                        0.45,
                    );
                    let rise = (1.0 - t) * 14.0;
                    ui.add_space(rise * 0.5);
                    step_rail(ui, theme, fonts, state.step);
                    ui.add_space(theme.tokens.space.lg);

                    components::card(ui, theme, true, |ui| {
                        ui.set_width(card_w - theme.tokens.space.lg * 2.0);
                        ui.add_space(theme.tokens.space.xs);
                        ui.label(
                            RichText::new(state.step.title())
                                .font(fonts.title())
                                .strong()
                                .color(theme.text()),
                        );
                        ui.add_space(theme.tokens.space.md);

                        match state.step {
                            SetupStep::Welcome => welcome(ui, theme, fonts),
                            SetupStep::Name => name(ui, theme, fonts, state),
                            SetupStep::Password => password(ui, theme, fonts, state),
                            SetupStep::PackageManager => package_manager(ui, theme, fonts, state),
                            SetupStep::Privacy => privacy(ui, theme, fonts, state),
                            SetupStep::Finish => finish(ui, theme, fonts, state),
                        }

                        if let Some(err) = &state.error {
                            ui.add_space(theme.tokens.space.sm);
                            ui.label(RichText::new(err).font(fonts.label()).color(theme.danger()));
                        }

                        ui.add_space(theme.tokens.space.lg);
                        ui.separator();
                        ui.add_space(theme.tokens.space.md);
                        nav(ui, theme, fonts, state, &mut action);
                    });
                },
            );
        });
    action
}

/// The progress rail: a filled segment per completed step, with labels.
fn step_rail(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, step: SetupStep) {
    let current = step.index();
    ui.horizontal(|ui| {
        for i in 0..SetupStep::ALL.len() {
            let done = i < current;
            let active = i == current;
            let color = if active {
                theme.accent()
            } else if done {
                theme.accent().gamma_multiply(0.7)
            } else {
                theme.border()
            };
            let (rect, _) = ui.allocate_exact_size(Vec2::new(46.0, 5.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, crate::theme::corner(2.5), color);
            if i < SetupStep::ALL.len() - 1 {
                ui.add_space(6.0);
            }
        }
        ui.add_space(theme.tokens.space.md);
        ui.label(
            RichText::new(format!("Passo {} di {}", current + 1, SetupStep::ALL.len()))
                .font(fonts.caption())
                .color(theme.text_muted()),
        );
    });
}

fn welcome(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts) {
    for line in [
        "Benvenuto in LibonOS. In tre passaggi creiamo il tuo account, scegliamo come installi i programmi e decidiamo cosa condividere.",
        "I dati restano sul tuo PC. La telemetria è spenta finché non la attivi tu.",
    ] {
        ui.label(RichText::new(line).font(fonts.body()).color(theme.text_secondary()));
        ui.add_space(theme.tokens.space.sm);
    }
    let space = theme.tokens.space;
    ui.add_space(space.xs);
    for (icon, text) in [
        (&icons::USER, "Nome e password per accedere al PC"),
        (&icons::BAG, "APT, pacman o libon come gestore pacchetti"),
        (&icons::SHIELD, "Privacy e aggiornamenti a tua scelta"),
    ] {
        ui.horizontal(|ui| {
            icon.show(ui, 18.0, theme.accent());
            ui.add_space(space.sm);
            ui.label(
                RichText::new(text)
                    .font(fonts.label())
                    .color(theme.text_secondary()),
            );
        });
        ui.add_space(space.xs);
    }
}

fn name(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, state: &mut SetupState) {
    ui.label(
        RichText::new("Questo nome appare nella schermata di accesso e nel menu del PC.")
            .font(fonts.body())
            .color(theme.text_secondary()),
    );
    ui.add_space(theme.tokens.space.md);
    let resp = Field::new(&mut state.answers.first_name, "Es. Giulia")
        .label("Nome")
        .icon(&icons::USER)
        .show(ui, theme);
    if state.step == SetupStep::Name {
        resp.request_focus();
    }
    ui.add_space(theme.tokens.space.sm);
    ui.label(
        RichText::new(format!(
            "Nome utente: {}",
            libonui_core::accounts::username_from(&state.answers.first_name)
        ))
        .font(fonts.caption())
        .color(theme.text_muted()),
    );
}

fn password(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, state: &mut SetupState) {
    ui.label(
        RichText::new("La password serve per accedere al PC. Non lascia mai questa macchina.")
            .font(fonts.body())
            .color(theme.text_secondary()),
    );
    ui.add_space(theme.tokens.space.md);
    Field::new(&mut state.answers.password, "Almeno 8 caratteri")
        .label("Password")
        .password()
        .icon(&icons::LOCK)
        .show(ui, theme);
    ui.add_space(theme.tokens.space.xs);
    components::strength_meter(ui, theme, state.password_strength());
    ui.add_space(theme.tokens.space.md);
    Field::new(&mut state.answers.confirm, "Ripeti la password")
        .label("Conferma password")
        .password()
        .icon(&icons::LOCK)
        .show(ui, theme);

    ui.add_space(theme.tokens.space.md);
    ui.horizontal(|ui| {
        components::toggle(ui, theme, &mut state.answers.encrypt_home);
        ui.add_space(theme.tokens.space.sm);
        ui.label(
            RichText::new("Cifra la mia cartella personale")
                .font(fonts.label())
                .color(theme.text_secondary()),
        );
    });
    ui.label(
        RichText::new("Consigliato. Se perdi la password i dati cifrati non sono recuperabili.")
            .font(fonts.caption())
            .color(theme.text_muted()),
    );
}

fn package_manager(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, state: &mut SetupState) {
    ui.label(
        RichText::new(
            "Scegli come LibonOS installa i programmi. Puoi cambiarlo in qualsiasi momento dalle Impostazioni.",
        )
        .font(fonts.body())
        .color(theme.text_secondary()),
    );
    ui.add_space(theme.tokens.space.md);

    for kind in PackageManagerKind::ALL {
        let selected = state.answers.package_manager == kind;
        let resp = manager_option(ui, theme, fonts, kind, selected);
        if resp.clicked() {
            state.answers.package_manager = kind;
        }
        ui.add_space(theme.tokens.space.sm);
    }
}

/// One selectable package-manager card. Returns its response so the caller can
/// react to a click.
fn manager_option(
    ui: &mut egui::Ui,
    theme: &Theme,
    fonts: &Fonts,
    kind: PackageManagerKind,
    selected: bool,
) -> egui::Response {
    let space = theme.tokens.space;
    let height = 74.0;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), height),
        egui::Sense::click(),
    );
    let hovered = response.hovered();
    let fill = if selected {
        crate::theme::c(theme.tokens.accent_soft)
    } else if hovered {
        crate::theme::c(theme.tokens.surface_overlay)
    } else {
        crate::theme::c(theme.tokens.surface)
    };
    let border = if selected {
        theme.accent()
    } else if hovered {
        crate::theme::c(theme.tokens.border_strong)
    } else {
        theme.border()
    };
    let rounding = crate::theme::corner(theme.tokens.radius.md);
    ui.painter().rect_filled(rect, rounding, fill);
    ui.painter().rect_stroke(
        rect,
        rounding,
        egui::Stroke::new(if selected { 1.6 } else { 1.0_f32 }, border),
    );

    // Radio mark.
    let mark_center = egui::pos2(rect.min.x + space.lg + 9.0, rect.center().y);
    if selected {
        ui.painter().circle_filled(mark_center, 9.0, theme.accent());
        ui.painter()
            .circle_filled(mark_center, 3.5, crate::theme::c(theme.tokens.on_accent));
    } else {
        ui.painter()
            .circle_stroke(mark_center, 9.0, egui::Stroke::new(1.5_f32, theme.border()));
    }

    let text_x = mark_center.x + 22.0;
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_max(
        egui::pos2(text_x, rect.min.y + space.md),
        egui::pos2(rect.max.x - space.md, rect.max.y - space.sm),
    )));
    child.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(kind.label())
                    .font(fonts.heading())
                    .strong()
                    .color(theme.text()),
            );
            if kind.is_preinstalled() {
                components::pill(ui, theme, "Preinstallato", theme.accent_alt());
            }
            if !kind.requires_admin() {
                components::pill(ui, theme, "Senza root", theme.success());
            }
        });
        ui.label(
            RichText::new(kind.description())
                .font(fonts.caption())
                .color(theme.text_muted()),
        );
    });

    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

fn privacy(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, state: &mut SetupState) {
    ui.label(
        RichText::new("Due scelte, entrambe reversibili dalle Impostazioni.")
            .font(fonts.body())
            .color(theme.text_secondary()),
    );
    ui.add_space(theme.tokens.space.md);
    components::toggle_row(
        ui,
        theme,
        "Aggiornamenti automatici",
        "Scarica e applica gli aggiornamenti in-place quando il PC è inattivo.",
        &mut state.answers.automatic_updates,
    );
    ui.add_space(theme.tokens.space.md);
    components::toggle_row(
        ui,
        theme,
        "Invia statistiche anonime",
        "Solo conteggi di utilizzo. Nessun contenuto, nessun identificatore.",
        &mut state.answers.telemetry_enabled,
    );
    ui.add_space(theme.tokens.space.sm);
    ui.label(
        RichText::new(if state.answers.telemetry_enabled {
            "La telemetria è attiva. Puoi spegnerla da Impostazioni › Privacy."
        } else {
            "La telemetria resta spenta. È la scelta predefinita di LibonOS."
        })
        .font(fonts.caption())
        .color(theme.text_muted()),
    );
}

fn finish(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, state: &mut SetupState) {
    ui.label(
        RichText::new("Ecco cosa stiamo per creare:")
            .font(fonts.body())
            .color(theme.text_secondary()),
    );
    ui.add_space(theme.tokens.space.md);

    let rows = [
        ("Nome", state.answers.first_name.clone()),
        (
            "Nome utente",
            libonui_core::accounts::username_from(&state.answers.first_name),
        ),
        (
            "Gestore pacchetti",
            state.answers.package_manager.label().to_string(),
        ),
        (
            "Cartella personale cifrata",
            if state.answers.encrypt_home {
                "Sì"
            } else {
                "No"
            }
            .to_string(),
        ),
        (
            "Aggiornamenti automatici",
            if state.answers.automatic_updates {
                "Sì"
            } else {
                "No"
            }
            .to_string(),
        ),
        (
            "Telemetria",
            if state.answers.telemetry_enabled {
                "Attiva"
            } else {
                "Spenta"
            }
            .to_string(),
        ),
    ];
    for (k, v) in rows {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(k)
                    .font(fonts.label())
                    .color(theme.text_muted()),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(v)
                        .font(fonts.label())
                        .strong()
                        .color(theme.text()),
                );
            });
        });
        ui.add_space(theme.tokens.space.xs);
    }
    ui.add_space(theme.tokens.space.sm);
    ui.label(
        RichText::new("Firefox e le app di sistema sono già installate.")
            .font(fonts.caption())
            .color(theme.success()),
    );
}

/// The Back / Next controls. The last step says "Inizia" and reports success.
fn nav(
    ui: &mut egui::Ui,
    theme: &Theme,
    _fonts: &Fonts,
    state: &mut SetupState,
    action: &mut Action,
) {
    ui.horizontal(|ui| {
        if state.step != SetupStep::Welcome
            && Button::new("Indietro")
                .icon(&icons::CHEVRON_LEFT)
                .show(ui, theme)
                .clicked()
        {
            state.back();
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let last = state.step.is_last();
            let label = if last { "Inizia" } else { "Avanti" };
            let btn = if last {
                Button::new(label).primary().icon(&icons::CHECK)
            } else {
                Button::new(label).primary().icon(&icons::CHEVRON_RIGHT)
            };
            if btn.show(ui, theme).clicked() {
                if last {
                    *action = Action::CompleteSetup;
                } else {
                    state.advance();
                }
            }
            if last {
                ui.add_space(theme.tokens.space.sm);
                ui.label(
                    RichText::new("I dati restano su questo PC")
                        .font(crate::theme::Fonts::from(&theme.tokens).caption)
                        .color(theme.text_muted()),
                );
            }
        });
    });
}
