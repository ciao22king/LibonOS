//! A file manager: places on the left, a listing in the middle.

use crate::icons;
use crate::theme::{corner, Fonts, Theme};
use egui::{RichText, Sense, Vec2};

const PLACES: &[(&str, &str)] = &[
    ("Home", "Cartella personale"),
    ("Documenti", "Documenti"),
    ("Immagini", "Immagini"),
    ("Scaricati", "Scaricati"),
    ("Musica", "Musica"),
    ("Video", "Video"),
];

struct Entry {
    name: &'static str,
    folder: bool,
    size: &'static str,
    modified: &'static str,
}

const ENTRIES: &[Entry] = &[
    Entry {
        name: "Documenti",
        folder: true,
        size: "—",
        modified: "oggi",
    },
    Entry {
        name: "Immagini",
        folder: true,
        size: "—",
        modified: "ieri",
    },
    Entry {
        name: "Scaricati",
        folder: true,
        size: "—",
        modified: "2 giorni fa",
    },
    Entry {
        name: "Musica",
        folder: true,
        size: "—",
        modified: "5 giorni fa",
    },
    Entry {
        name: "Video",
        folder: true,
        size: "—",
        modified: "1 settimana fa",
    },
    Entry {
        name: "note-libonos.md",
        folder: false,
        size: "4 KB",
        modified: "10 min fa",
    },
    Entry {
        name: "appunti-riunione.txt",
        folder: false,
        size: "12 KB",
        modified: "3 ore fa",
    },
    Entry {
        name: "screenshot-2026-11-03.png",
        folder: false,
        size: "820 KB",
        modified: "oggi",
    },
    Entry {
        name: "libonui-source.tar.gz",
        folder: false,
        size: "1,2 MB",
        modified: "ieri",
    },
    Entry {
        name: "firefox.AppImage",
        folder: false,
        size: "75 MB",
        modified: "oggi",
    },
];

pub fn show(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts) {
    ui.horizontal(|ui| {
        icons::FOLDER.show(ui, 18.0, theme.accent());
        ui.add_space(6.0);
        ui.label(
            RichText::new("File")
                .font(fonts.heading())
                .color(theme.text()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new("~ / Home")
                    .font(fonts.caption())
                    .color(theme.text_muted()),
            );
        });
    });
    ui.add_space(theme.tokens.space.sm);
    ui.separator();
    ui.add_space(theme.tokens.space.sm);

    ui.horizontal_top(|ui| {
        // Places rail.
        ui.vertical(|ui| {
            ui.set_width(160.0);
            for (name, _detail) in PLACES {
                let (rect, resp) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), Sense::click());
                let hovered = resp.hovered();
                if hovered {
                    ui.painter().rect_filled(
                        rect,
                        corner(theme.tokens.radius.sm),
                        theme.surface_raised(),
                    );
                }
                icons::FOLDER.paint(
                    ui,
                    egui::Rect::from_min_size(
                        egui::pos2(rect.min.x + 8.0, rect.center().y - 7.0),
                        Vec2::splat(14.0),
                    ),
                    if hovered {
                        theme.accent()
                    } else {
                        theme.text_muted()
                    },
                    1.5,
                );
                ui.painter().text(
                    egui::pos2(rect.min.x + 30.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    *name,
                    fonts.label(),
                    theme.text_secondary(),
                );
                if hovered {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            }
        });

        ui.add_space(theme.tokens.space.md);
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            // Column header.
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Nome")
                        .font(fonts.caption())
                        .color(theme.text_muted()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Modificato")
                            .font(fonts.caption())
                            .color(theme.text_muted()),
                    );
                    ui.add_space(60.0);
                    ui.label(
                        RichText::new("Dimensione")
                            .font(fonts.caption())
                            .color(theme.text_muted()),
                    );
                });
            });
            ui.add_space(theme.tokens.space.xs);
            egui::ScrollArea::vertical().show(ui, |ui| {
                for entry in ENTRIES {
                    let (rect, resp) = ui
                        .allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::click());
                    let hovered = resp.hovered();
                    if hovered {
                        ui.painter().rect_filled(
                            rect,
                            corner(theme.tokens.radius.sm),
                            theme.surface_raised(),
                        );
                    }
                    let icon = if entry.folder {
                        &icons::FOLDER
                    } else {
                        &icons::FILE
                    };
                    icon.paint(
                        ui,
                        egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + 6.0, rect.center().y - 8.0),
                            Vec2::splat(16.0),
                        ),
                        if entry.folder {
                            theme.accent()
                        } else {
                            theme.text_muted()
                        },
                        1.5,
                    );
                    ui.painter().text(
                        egui::pos2(rect.min.x + 30.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        entry.name,
                        fonts.label(),
                        theme.text(),
                    );
                    ui.painter().text(
                        egui::pos2(rect.max.x - 6.0, rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        entry.modified,
                        fonts.caption(),
                        theme.text_muted(),
                    );
                    ui.painter().text(
                        egui::pos2(rect.max.x - 90.0, rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        entry.size,
                        fonts.caption(),
                        theme.text_secondary(),
                    );
                    if hovered {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            });
        });
    });
}
