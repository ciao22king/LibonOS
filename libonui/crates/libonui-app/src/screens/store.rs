//! The Store: browse a catalog, install with one click, and watch progress.

use crate::components::{self, Button};
use crate::icons;
use crate::store::StoreState;
use crate::theme::{corner, Fonts, Theme};
use egui::{Color32, RichText, Sense, Stroke, Vec2};
use libonui_core::packages::{Category, InstallPhase};
use libonui_core::version::human_size;

pub fn show(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, store: &mut StoreState) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Store")
                .font(fonts.title())
                .strong()
                .color(theme.text()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            components::pill(
                ui,
                theme,
                store.manager().kind().label(),
                theme.accent_alt(),
            );
        });
    });
    ui.add_space(theme.tokens.space.md);

    // Search.
    ui.horizontal(|ui| {
        icons::SEARCH.show(ui, 18.0, theme.text_muted());
        ui.add_space(6.0);
        let mut q = store.query.clone();
        let resp = ui.add(
            egui::TextEdit::singleline(&mut q)
                .hint_text("Cerca un'app o una descrizione…")
                .desired_width(ui.available_width() - 120.0)
                .frame(false),
        );
        if resp.changed() {
            store.set_query(q);
        }
        ui.add_space(theme.tokens.space.sm);
        if Button::new("Aggiungi dal nome")
            .ghost()
            .icon(&icons::PLUS)
            .show(ui, theme)
            .clicked()
        {
            store.catalog_open = !store.catalog_open;
        }
        if Button::new("Aggiorna catalogo")
            .ghost()
            .icon(&icons::REFRESH)
            .show(ui, theme)
            .clicked()
        {
            store.load_featured();
        }
    });

    if store.catalog_open {
        catalog_entry(ui, theme, fonts, store);
        ui.add_space(theme.tokens.space.md);
    }
    ui.add_space(theme.tokens.space.sm);

    // Category chips.
    ui.horizontal_wrapped(|ui| {
        let selected_all = store.category.is_none();
        if chip(ui, theme, "Tutte", selected_all) {
            store.set_category(None);
        }
        for cat in Category::ALL {
            if chip(ui, theme, cat.label(), store.category == Some(cat)) {
                store.set_category(Some(cat));
            }
        }
    });
    ui.add_space(theme.tokens.space.md);
    ui.separator();
    ui.add_space(theme.tokens.space.md);

    // Active installs.
    let active: Vec<(String, String, InstallPhase, f32)> = store
        .active_jobs()
        .map(|j| {
            (
                j.package.name.clone(),
                j.package.id.clone(),
                j.phase.clone(),
                j.phase.fraction().unwrap_or(0.0),
            )
        })
        .collect();
    if !active.is_empty() {
        for (name, _id, phase, frac) in &active {
            components::card(ui, theme, false, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(name).font(fonts.label()).color(theme.text()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(phase.label())
                                .font(fonts.caption())
                                .color(theme.accent_alt()),
                        );
                    });
                });
                ui.add_space(theme.tokens.space.sm);
                components::progress(ui, theme, Some(*frac), 6.0);
                if let InstallPhase::Downloading { received, total } = phase {
                    ui.add_space(theme.tokens.space.xs);
                    ui.label(
                        RichText::new(format!(
                            "{} di {}",
                            human_size(*received),
                            human_size(*total)
                        ))
                        .font(fonts.caption())
                        .color(theme.text_muted()),
                    );
                }
            });
            ui.add_space(theme.tokens.space.sm);
        }
        ui.add_space(theme.tokens.space.sm);
    }

    // Results.
    let results = store.results.clone();
    if results.is_empty() {
        components::empty_state(
            ui,
            theme,
            &icons::SEARCH,
            "Nessun risultato",
            "Prova un altro termine, o cambia categoria.",
        );
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for package in results {
            let busy = store.is_busy(&package.id);
            let done = store.is_done(&package.id);
            let mut install = false;
            components::card(ui, theme, false, |ui| {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
                    ui.painter().rect_filled(
                        r,
                        corner(theme.tokens.radius.md),
                        crate::theme::c(theme.tokens.accent_soft),
                    );
                    icons::for_app(&package.id).paint(ui, r.shrink(10.0), theme.accent(), 1.6);

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&package.name)
                                    .font(fonts.heading())
                                    .strong()
                                    .color(theme.text()),
                            );
                            components::pill(
                                ui,
                                theme,
                                package.category.label(),
                                theme.accent_alt(),
                            );
                            if package.installed || done {
                                components::pill(ui, theme, "Installato", theme.success());
                            }
                        });
                        ui.label(
                            RichText::new(&package.summary)
                                .font(fonts.caption())
                                .color(theme.text_muted()),
                        );
                        ui.label(
                            RichText::new(format!(
                                "{} · {}",
                                package.id,
                                human_size(package.size_bytes)
                            ))
                            .font(fonts.caption())
                            .color(theme.text_muted()),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if package.installed || done {
                            Button::new("Apri")
                                .icon(&icons::CHEVRON_RIGHT)
                                .show(ui, theme);
                        } else if busy {
                            Button::new("Installazione…").enabled(false).show(ui, theme);
                        } else if Button::new("Installa")
                            .primary()
                            .icon(&icons::DOWNLOAD)
                            .show(ui, theme)
                            .clicked()
                        {
                            install = true;
                        }
                    });
                });
            });
            if install {
                store.install(package.clone());
            }
            ui.add_space(theme.tokens.space.sm);
        }
    });
}

fn chip(ui: &mut egui::Ui, theme: &Theme, label: &str, selected: bool) -> bool {
    let fonts = Fonts::from(&theme.tokens);
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_string(), fonts.caption(), Color32::PLACEHOLDER);
    let size = galley.size() + Vec2::new(22.0, 10.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hovered = resp.hovered();
    let fill = if selected {
        theme.accent()
    } else if hovered {
        crate::theme::c(theme.tokens.surface_raised)
    } else {
        Color32::TRANSPARENT
    };
    let text = if selected {
        theme.on_accent()
    } else {
        theme.text_secondary()
    };
    ui.painter()
        .rect_filled(rect, corner(theme.tokens.radius.pill), fill);
    if !selected {
        ui.painter().rect_stroke(
            rect,
            corner(theme.tokens.radius.pill),
            Stroke::new(1.0_f32, theme.border()),
        );
    }
    let g = ui
        .painter()
        .layout_no_wrap(label.to_string(), fonts.caption(), text);
    let pos = rect.center() - g.size() / 2.0;
    ui.painter().galley(pos, g, text);
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

/// The Store's "add an app by name" form: what `libon install <nome>` does,
/// offered with the fields visible before anything is downloaded.
fn catalog_entry(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, store: &mut StoreState) {
    components::card(ui, theme, true, |ui| {
        ui.label(
            RichText::new("Aggiungi un'app dal nome")
                .font(fonts.heading())
                .strong()
                .color(theme.text()),
        );
        ui.label(
            RichText::new(
                "Scrivi il nome dell'app. libon cerca la release, scarica l'AppImage e la \
                 registra nel menu — come `libon install <nome>` da terminale.",
            )
            .font(fonts.caption())
            .color(theme.text_muted()),
        );
        ui.add_space(theme.tokens.space.sm);

        components::Field::new(&mut store.draft_name, "Es. Obsidian")
            .label("Nome dell'app")
            .icon(&icons::SEARCH)
            .show(ui, theme);
        ui.add_space(theme.tokens.space.sm);
        components::Field::new(&mut store.draft_summary, "Descrizione (facoltativa)")
            .label("Descrizione")
            .show(ui, theme);
        ui.add_space(theme.tokens.space.sm);

        ui.label(
            RichText::new("Categoria")
                .font(fonts.label())
                .color(theme.text_secondary()),
        );
        ui.add_space(theme.tokens.space.xs);
        ui.horizontal_wrapped(|ui| {
            for cat in Category::ALL {
                if chip(ui, theme, cat.label(), store.draft_category == cat) {
                    store.draft_category = cat;
                }
            }
        });

        ui.add_space(theme.tokens.space.md);
        ui.horizontal(|ui| {
            let ready = !store.draft_name.trim().is_empty();
            let add = Button::new("Aggiungi e installa")
                .primary()
                .icon(&icons::DOWNLOAD)
                .enabled(ready)
                .show(ui, theme);
            if add.clicked() {
                store.add_draft();
            }
            if Button::new("Annulla").ghost().show(ui, theme).clicked() {
                store.catalog_open = false;
            }
        });
    });
}
