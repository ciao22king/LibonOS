//! Settings: appearance, the two kinds of OS update, the package manager,
//! account, network, privacy and about.
//!
//! The Updates tab is where the rule the user asked for lives: an in-place
//! update can be started here or from the terminal, but a **full system upgrade**
//! — crossing a major version — is offered only here, behind an explicit
//! confirmation that says what it will do.

use crate::app::{Desktop, NotifyKind, SettingsTab, UpdateRun};
use crate::components::{self, Button};
use crate::icons;
use crate::theme::{Fonts, Theme};
use egui::{RichText, Vec2};
use libonui_core::packages::{manager_for, PackageManagerKind};
use libonui_core::system;
use libonui_core::tokens::ThemeChoice;
use libonui_core::version::{self, UpdateKind};
use libonui_core::WallpaperStyle;

pub fn show(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Impostazioni")
                .font(fonts.title())
                .strong()
                .color(theme.text()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            components::pill(ui, theme, &libonui_core::banner(), theme.accent_alt());
        });
    });
    ui.add_space(theme.tokens.space.md);

    ui.horizontal_top(|ui| {
        // Left rail.
        ui.vertical(|ui| {
            ui.set_width(180.0);
            for tab in SettingsTab::ALL {
                let selected = desktop.settings_tab == tab;
                let (rect, resp) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 36.0),
                    egui::Sense::click(),
                );
                let hovered = resp.hovered();
                if selected {
                    ui.painter().rect_filled(
                        rect,
                        crate::theme::corner(theme.tokens.radius.sm),
                        crate::theme::c(theme.tokens.accent_soft),
                    );
                } else if hovered {
                    ui.painter().rect_filled(
                        rect,
                        crate::theme::corner(theme.tokens.radius.sm),
                        theme.surface_raised(),
                    );
                }
                let color = if selected {
                    theme.accent()
                } else {
                    theme.text_secondary()
                };
                let icon = tab_icon(tab);
                icon.paint(
                    ui,
                    egui::Rect::from_min_size(
                        egui::pos2(rect.min.x + 12.0, rect.center().y - 8.0),
                        Vec2::splat(16.0),
                    ),
                    color,
                    1.6,
                );
                ui.painter().text(
                    egui::pos2(rect.min.x + 38.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    tab.label(),
                    fonts.label(),
                    if selected { theme.text() } else { color },
                );
                if hovered {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if resp.clicked() {
                    desktop.settings_tab = tab;
                }
                ui.add_space(2.0);
            }
        });

        ui.add_space(theme.tokens.space.lg);
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical().show(ui, |ui| match desktop.settings_tab {
                SettingsTab::Appearance => appearance(ui, theme, fonts, desktop),
                SettingsTab::Updates => updates(ui, theme, fonts, desktop),
                SettingsTab::Packages => packages(ui, theme, fonts, desktop),
                SettingsTab::Account => account(ui, theme, fonts, desktop),
                SettingsTab::Network => network(ui, theme, fonts, desktop),
                SettingsTab::Privacy => privacy(ui, theme, fonts, desktop),
                SettingsTab::About => about(ui, theme, fonts, desktop),
            });
        });
    });
}

/// A theme swatch: a small preview of the palette with the name under it, so the
/// choice is legible without reading a label alone. Returns true when clicked.
fn theme_swatch(ui: &mut egui::Ui, fonts: &Fonts, choice: ThemeChoice, selected: bool) -> bool {
    let tokens = choice.tokens();
    let preview = Theme::from_tokens(tokens);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(150.0, 86.0), egui::Sense::click());
    let hovered = resp.hovered();
    let rounding = crate::theme::corner(tokens.radius.md);
    // The preview paints its own mini window.
    ui.painter().rect_filled(rect, rounding, preview.surface());
    ui.painter().rect_stroke(
        rect,
        rounding,
        egui::Stroke::new(
            if selected { 2.0 } else { 1.0_f32 },
            if selected {
                preview.accent()
            } else {
                preview.border()
            },
        ),
    );
    let inner = rect.shrink(10.0);
    ui.painter().rect_filled(
        egui::Rect::from_min_size(inner.min, Vec2::new(inner.width(), 10.0)),
        crate::theme::corner(3.0),
        preview.accent(),
    );
    for (i, w) in [0.8f32, 0.6, 0.7].iter().enumerate() {
        let y = inner.min.y + 22.0 + i as f32 * 10.0;
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(inner.min.x, y),
                Vec2::new(inner.width() * w, 5.0),
            ),
            crate::theme::corner(2.5),
            preview.text_muted(),
        );
    }
    ui.painter().text(
        egui::pos2(rect.center().x, rect.max.y + 12.0),
        egui::Align2::CENTER_CENTER,
        choice.label(),
        fonts.label(),
        if selected {
            preview.accent()
        } else {
            ui.visuals().weak_text_color()
        },
    );
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

fn tab_icon(tab: SettingsTab) -> &'static icons::Icon {
    match tab {
        SettingsTab::Appearance => &icons::IMAGE,
        SettingsTab::Updates => &icons::DOWNLOAD,
        SettingsTab::Packages => &icons::BAG,
        SettingsTab::Account => &icons::USER,
        SettingsTab::Network => &icons::WIFI,
        SettingsTab::Privacy => &icons::SHIELD,
        SettingsTab::About => &icons::HELP,
    }
}

fn appearance(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    components::section(ui, theme, "Tema", |ui| {
        ui.horizontal(|ui| {
            for choice in [ThemeChoice::Dark, ThemeChoice::Light] {
                if theme_swatch(ui, fonts, choice, desktop.config.theme == choice) {
                    desktop.config.theme = choice;
                }
                ui.add_space(theme.tokens.space.md);
            }
        });
        ui.label(
            RichText::new("Il tema cambia subito, senza riavviare la sessione.")
                .font(fonts.caption())
                .color(theme.text_muted()),
        );
    });

    components::section(ui, theme, "Sfondo", |ui| {
        ui.horizontal(|ui| {
            for style in WallpaperStyle::ALL {
                let selected = desktop.config.wallpaper == style;
                let resp = Button::new(style.label()).show(ui, theme);
                if selected {
                    ui.painter().rect_stroke(
                        resp.rect.expand(2.0),
                        crate::theme::corner(theme.tokens.radius.md + 2.0),
                        egui::Stroke::new(2.0_f32, theme.accent()),
                    );
                }
                if resp.clicked() {
                    desktop.config.wallpaper = style;
                }
            }
        });
    });

    components::section(ui, theme, "Accessibilità e testo", |ui| {
        let mut scale = desktop.config.font_scale;
        ui.label(
            RichText::new(format!("Dimensione testo: {:.0}%", scale * 100.0))
                .font(fonts.label())
                .color(theme.text_secondary()),
        );
        if ui
            .add(egui::Slider::new(&mut scale, 0.8..=1.6).show_value(false))
            .changed()
        {
            desktop.config.font_scale = scale;
        }
        ui.add_space(theme.tokens.space.sm);
        components::toggle_row(
            ui,
            theme,
            "Animazioni",
            "Dissolvenze e transizioni fra le schermate.",
            &mut desktop.config.animations,
        );
        components::toggle_row(
            ui,
            theme,
            "Luce notturna",
            "Riduce la luce blu dopo il tramonto.",
            &mut desktop.config.night_light,
        );
    });
    desktop.save_config();
}

fn updates(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    // An update in progress takes over the tab.
    if let Some(run) = &desktop.update_run {
        update_progress(ui, theme, fonts, run);
        if run.progress() >= 1.0 {
            let target = run.target.clone();
            let major = run.major;
            desktop.update_run = None;
            desktop.notify(
                if major {
                    "Aggiornamento di sistema completato"
                } else {
                    "Aggiornamento completato"
                },
                format!("LibonOS è ora alla versione {target}"),
                NotifyKind::Success,
            );
        }
        ui.ctx().request_repaint();
        return;
    }

    components::section(ui, theme, "Versione installata", |ui| {
        let (name, release) = system::os_identity();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(name)
                    .font(fonts.heading())
                    .strong()
                    .color(theme.text()),
            );
            components::pill(
                ui,
                theme,
                if desktop.config.beta_channel {
                    "canale beta"
                } else {
                    "canale stable"
                },
                theme.accent_alt(),
            );
        });
        ui.label(
            RichText::new(release)
                .font(fonts.label())
                .color(theme.text_secondary()),
        );
    });

    // In-place update.
    components::section(ui, theme, "Aggiornamento in-place", |ui| {
        let status = &desktop.update;
        match status.kind {
            UpdateKind::UpToDate => {
                ui.horizontal(|ui| {
                    icons::CHECK.show(ui, 18.0, theme.success());
                    ui.add_space(theme.tokens.space.sm);
                    ui.label(
                        RichText::new("Sei già all'ultima versione della tua linea.")
                            .font(fonts.label())
                            .color(theme.text_secondary()),
                    );
                });
            }
            _ => {
                ui.label(
                    RichText::new(format!("{} → {}", status.current, status.target))
                        .font(fonts.heading())
                        .color(theme.text()),
                );
                ui.label(
                    RichText::new(&status.summary)
                        .font(fonts.label())
                        .color(theme.text_secondary()),
                );
                ui.add_space(theme.tokens.space.sm);
                ui.horizontal(|ui| {
                    components::pill(
                        ui,
                        theme,
                        &version::human_size(status.size_bytes),
                        theme.info(),
                    );
                    if status.mandatory {
                        components::pill(ui, theme, "Consigliato", theme.warning());
                    }
                    components::pill(
                        ui,
                        theme,
                        "Anche da terminale: libon update",
                        theme.text_muted(),
                    );
                });
                ui.add_space(theme.tokens.space.md);
                if Button::new("Installa aggiornamento")
                    .primary()
                    .icon(&icons::DOWNLOAD)
                    .show(ui, theme)
                    .clicked()
                {
                    desktop.update_run = Some(UpdateRun {
                        started: std::time::Instant::now(),
                        major: false,
                        target: status.target.clone(),
                    });
                }
            }
        }
    });

    // Full system upgrade — the Settings-only path.
    components::section(
        ui,
        theme,
        "Aggiornamento di sistema completo",
        |ui| match &desktop.major_update {
            Some(major) => {
                ui.label(
                    RichText::new(format!("{} → {}", major.current, major.target))
                        .font(fonts.heading())
                        .strong()
                        .color(theme.accent()),
                );
                ui.label(
                    RichText::new(&major.title)
                        .font(fonts.label())
                        .color(theme.text()),
                );
                ui.add_space(theme.tokens.space.xs);
                ui.label(
                    RichText::new(&major.summary)
                        .font(fonts.label())
                        .color(theme.text_secondary()),
                );
                ui.add_space(theme.tokens.space.sm);
                ui.horizontal(|ui| {
                    components::pill(
                        ui,
                        theme,
                        &version::human_size(major.size_bytes),
                        theme.info(),
                    );
                    components::pill(ui, theme, "Solo da Impostazioni", theme.warning());
                    if major.mandatory {
                        components::pill(ui, theme, "Obbligatorio", theme.danger());
                    }
                });
                ui.add_space(theme.tokens.space.md);
                components::card(ui, theme, false, |ui| {
                    ui.horizontal(|ui| {
                        icons::SHIELD.show(ui, 20.0, theme.warning());
                        ui.add_space(theme.tokens.space.sm);
                        ui.label(
                            RichText::new(
                                "Un aggiornamento di sistema completo scarica una nuova immagine e \
                                 riscrive la versione installata. Richiede un riavvio e circa 20 minuti. \
                                 Non è disponibile da terminale: va avviato da qui.",
                            )
                            .font(fonts.caption())
                            .color(theme.text_secondary()),
                        );
                    });
                });
                ui.add_space(theme.tokens.space.md);
                let label = format!("Aggiorna a {}", major.target);
                if Button::new(&label)
                    .danger()
                    .icon(&icons::POWER)
                    .show(ui, theme)
                    .clicked()
                {
                    desktop.update_run = Some(UpdateRun {
                        started: std::time::Instant::now(),
                        major: true,
                        target: major.target.clone(),
                    });
                }
            }
            None => {
                ui.label(
                    RichText::new("Nessuna nuova versione principale disponibile.")
                        .font(fonts.label())
                        .color(theme.text_muted()),
                );
            }
        },
    );

    components::section(ui, theme, "Canale di aggiornamento", |ui| {
        if components::toggle_row(
            ui,
            theme,
            "Canale beta",
            "Ricevi in anteprima le versioni in prova. Meno stabili.",
            &mut desktop.config.beta_channel,
        ) {
            desktop.refresh_updates();
        }
        components::toggle_row(
            ui,
            theme,
            "Aggiornamenti automatici",
            "Applica gli aggiornamenti in-place quando il PC è inattivo.",
            &mut desktop.config.automatic_updates,
        );
    });
    desktop.save_config();
}

fn update_progress(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, run: &UpdateRun) {
    components::section(ui, theme, "Aggiornamento in corso", |ui| {
        ui.label(
            RichText::new(if run.major {
                "Aggiornamento di sistema completo"
            } else {
                "Aggiornamento in-place"
            })
            .font(fonts.heading())
            .strong()
            .color(theme.text()),
        );
        ui.label(
            RichText::new(format!("Versione di destinazione: {}", run.target))
                .font(fonts.label())
                .color(theme.text_secondary()),
        );
        ui.add_space(theme.tokens.space.lg);
        components::progress(ui, theme, Some(run.progress()), 8.0);
        ui.add_space(theme.tokens.space.sm);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(run.stage_label())
                    .font(fonts.label())
                    .color(theme.accent_alt()),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{:.0}%", run.progress() * 100.0))
                        .font(fonts.label())
                        .strong()
                        .color(theme.text()),
                );
            });
        });
        ui.add_space(theme.tokens.space.lg);
        components::card(ui, theme, false, |ui| {
            ui.label(
                RichText::new(
                    "Non spegnere il PC. Al termine la sessione riparte automaticamente.",
                )
                .font(fonts.caption())
                .color(theme.text_muted()),
            );
        });
    });
}

fn packages(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    components::section(ui, theme, "Gestore pacchetti", |ui| {
        ui.label(
            RichText::new(
                "Decide come lo Store e `libon install` procurano i programmi. libon è preinstallato.",
            )
            .font(fonts.label())
            .color(theme.text_secondary()),
        );
        ui.add_space(theme.tokens.space.md);
        let current = desktop.config.package_manager;
        for kind in PackageManagerKind::ALL {
            let selected = current == kind;
            let (rect, resp) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 62.0), egui::Sense::click());
            let hovered = resp.hovered();
            let fill = if selected {
                crate::theme::c(theme.tokens.accent_soft)
            } else if hovered {
                theme.surface_raised()
            } else {
                theme.surface()
            };
            let border = if selected {
                theme.accent()
            } else {
                theme.border()
            };
            ui.painter()
                .rect_filled(rect, crate::theme::corner(theme.tokens.radius.md), fill);
            ui.painter().rect_stroke(
                rect,
                crate::theme::corner(theme.tokens.radius.md),
                egui::Stroke::new(if selected { 1.6 } else { 1.0_f32 }, border),
            );
            ui.painter().text(
                egui::pos2(rect.min.x + 16.0, rect.min.y + 20.0),
                egui::Align2::LEFT_CENTER,
                kind.label(),
                fonts.heading(),
                theme.text(),
            );
            ui.painter().text(
                egui::pos2(rect.min.x + 16.0, rect.min.y + 42.0),
                egui::Align2::LEFT_CENTER,
                kind.description(),
                fonts.caption(),
                theme.text_muted(),
            );
            // Report whether the backend is actually present on this machine.
            let available = manager_for(kind).is_available();
            let (marker, marker_color) = if available {
                ("Disponibile", theme.success())
            } else {
                ("Non installato", theme.warning())
            };
            ui.painter().text(
                egui::pos2(rect.max.x - 16.0, rect.min.y + 20.0),
                egui::Align2::RIGHT_CENTER,
                marker,
                fonts.caption(),
                marker_color,
            );
            if kind.is_preinstalled() {
                let pill_pos = egui::pos2(rect.max.x - 200.0, rect.min.y + 20.0);
                let galley = ui.painter().layout_no_wrap(
                    "Preinstallato".into(),
                    fonts.caption(),
                    theme.accent_alt(),
                );
                let pill_rect = egui::Rect::from_min_size(
                    pill_pos - Vec2::new(0.0, 10.0),
                    galley.size() + Vec2::new(18.0, 6.0),
                );
                ui.painter().rect_filled(
                    pill_rect,
                    crate::theme::corner(theme.tokens.radius.pill),
                    theme.accent_alt().gamma_multiply(0.16),
                );
                ui.painter().galley(
                    pill_rect.min + Vec2::new(9.0, 3.0),
                    galley,
                    theme.accent_alt(),
                );
            }
            if hovered {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() && !selected {
                desktop.set_manager(kind);
                if available {
                    desktop.notify(
                        "Gestore pacchetti cambiato",
                        format!("Lo Store ora usa {}.", kind.label()),
                        NotifyKind::Success,
                    );
                } else {
                    desktop.notify(
                        format!("{} non è installato", kind.label()),
                        format!(
                            "Lo Store userà libon finché `{}` non è disponibile.",
                            kind.binary()
                        ),
                        NotifyKind::Warning,
                    );
                }
            }
            ui.add_space(theme.tokens.space.sm);
        }
    });

    components::section(ui, theme, "Da terminale", |ui| {
        components::card(ui, theme, false, |ui| {
            for line in [
                "libon install <app>   scarica l'AppImage e la registra",
                "libon search <testo>  cerca nel catalogo",
                "libon update          aggiornamento in-place",
                "libon upgrade         informazioni sull'aggiornamento completo",
            ] {
                ui.label(
                    RichText::new(line)
                        .font(fonts.mono())
                        .color(theme.text_secondary()),
                );
            }
        });
    });
    desktop.save_config();
}

fn account(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    components::section(ui, theme, "Il tuo account", |ui| {
        components::card(ui, theme, true, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(56.0), egui::Sense::hover());
                ui.painter().circle_filled(
                    rect.center(),
                    28.0,
                    crate::theme::c(theme.tokens.accent_soft),
                );
                let initial = desktop
                    .session
                    .user_first_name
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
                ui.add_space(theme.tokens.space.md);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&desktop.session.user_first_name)
                            .font(fonts.heading())
                            .strong()
                            .color(theme.text()),
                    );
                    ui.label(
                        RichText::new(format!("@{}", desktop.session.username))
                            .font(fonts.label())
                            .color(theme.text_muted()),
                    );
                });
            });
        });
        ui.add_space(theme.tokens.space.md);
        ui.horizontal(|ui| {
            Button::new("Cambia password")
                .icon(&icons::LOCK)
                .show(ui, theme);
            Button::new("Cifra cartella personale")
                .icon(&icons::SHIELD)
                .show(ui, theme);
        });
    });
}

fn network(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    components::section(ui, theme, "Rete", |ui| {
        for (name, detail, connected) in [
            ("Wi-Fi", "Casa-5G", true),
            ("Ethernet", "Non collegata", false),
            ("VPN", "Nessuna attiva", false),
        ] {
            ui.horizontal(|ui| {
                icons::WIFI.show(
                    ui,
                    18.0,
                    if connected {
                        theme.success()
                    } else {
                        theme.text_muted()
                    },
                );
                ui.add_space(theme.tokens.space.sm);
                ui.label(RichText::new(name).font(fonts.label()).color(theme.text()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(detail)
                            .font(fonts.caption())
                            .color(theme.text_muted()),
                    );
                });
            });
            ui.add_space(theme.tokens.space.sm);
        }
    });
    components::section(ui, theme, "Nome del PC", |ui| {
        components::Field::new(&mut desktop.config.hostname, "libon-pc").show(ui, theme);
    });
    desktop.save_config();
}

fn privacy(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    components::section(ui, theme, "Telemetria", |ui| {
        components::toggle_row(
            ui,
            theme,
            "Invia statistiche anonime",
            "Solo conteggi di utilizzo, nessun contenuto. Spenta di serie.",
            &mut desktop.config.telemetry_enabled,
        );
        ui.label(
            RichText::new(if desktop.config.telemetry_enabled {
                "Attiva. Puoi spegnerla quando vuoi."
            } else {
                "Spenta. LibonOS non invia nulla."
            })
            .font(fonts.caption())
            .color(theme.text_muted()),
        );
    });
    components::section(ui, theme, "Permessi applicazioni", |ui| {
        for (app, perms) in [
            ("Firefox", "Rete, Notifiche"),
            ("File", "Cartella personale"),
            ("Terminale", "Cartella personale, Comandi"),
        ] {
            ui.horizontal(|ui| {
                ui.label(RichText::new(app).font(fonts.label()).color(theme.text()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(perms)
                            .font(fonts.caption())
                            .color(theme.text_muted()),
                    );
                });
            });
            ui.add_space(theme.tokens.space.sm);
        }
    });
    desktop.save_config();
}

fn about(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    components::section(ui, theme, "LibonOS", |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(theme.tokens.space.md);
            icons::GRID.show(ui, 52.0, theme.accent());
            ui.add_space(theme.tokens.space.sm);
            ui.label(
                RichText::new(libonui_core::banner())
                    .font(fonts.title())
                    .strong()
                    .color(theme.text()),
            );
            ui.label(
                RichText::new("Shell LibonUI · scritta in Rust")
                    .font(fonts.label())
                    .color(theme.text_muted()),
            );
        });
        ui.add_space(theme.tokens.space.lg);
        for (k, v) in [
            ("Kernel", "Linux 7.4"),
            ("Shell", "LibonUI"),
            ("Gestore pacchetti", desktop.config.package_manager.label()),
            (
                "Canale",
                if desktop.config.beta_channel {
                    "beta"
                } else {
                    "stable"
                },
            ),
            ("Architettura", std::env::consts::ARCH),
        ] {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(k)
                        .font(fonts.label())
                        .color(theme.text_muted()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(v).font(fonts.label()).color(theme.text()));
                });
            });
            ui.add_space(theme.tokens.space.xs);
        }
    });

    components::section(ui, theme, "Cosa include LibonOS", |ui| {
        for cap in system::capabilities() {
            ui.horizontal(|ui| {
                icons::CHECK.show(
                    ui,
                    16.0,
                    if cap.ready {
                        theme.success()
                    } else {
                        theme.warning()
                    },
                );
                ui.add_space(theme.tokens.space.sm);
                ui.label(
                    RichText::new(cap.name)
                        .font(fonts.label())
                        .color(theme.text()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(cap.summary)
                            .font(fonts.caption())
                            .color(theme.text_muted()),
                    );
                });
            });
            ui.add_space(theme.tokens.space.sm);
        }
    });
}
