//! A small text editor window.

use crate::icons;
use crate::theme::{Fonts, Theme};
use egui::RichText;

pub fn show(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts) {
    let id = egui::Id::new("libon-editor-buffer");
    let mut buffer: String = ui.ctx().data_mut(|d| d.get_temp(id)).unwrap_or_else(|| {
        "Note su LibonOS\n\
         ===============\n\n\
         Questa finestra è l'editor di testo di LibonUI.\n\n\
         - Il tema si cambia da Impostazioni › Aspetto\n\
         - Le app si installano dallo Store\n\
         - `libon update` aggiorna il sistema in-place\n\
         - Gli aggiornamenti completi si applicano da Impostazioni\n"
            .to_string()
    });

    ui.horizontal(|ui| {
        icons::NOTE.show(ui, 18.0, theme.accent());
        ui.add_space(6.0);
        ui.label(
            RichText::new("note-libonos.md")
                .font(fonts.label())
                .color(theme.text()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{} caratteri", buffer.chars().count()))
                    .font(fonts.caption())
                    .color(theme.text_muted()),
            );
        });
    });
    ui.add_space(theme.tokens.space.sm);

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add(
            egui::TextEdit::multiline(&mut buffer)
                .font(fonts.mono())
                .desired_width(f32::INFINITY)
                .desired_rows(20)
                .frame(false),
        );
    });

    ui.ctx().data_mut(|d| d.insert_temp(id, buffer));
}
