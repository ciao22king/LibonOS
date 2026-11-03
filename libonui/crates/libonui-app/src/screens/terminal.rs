//! A terminal window that runs `libon` and a few read-only shell commands.
//!
//! It is deliberately not a general-purpose shell: it recognises `libon` and a
//! small set of safe commands, and explains the major-upgrade rule when asked to
//! upgrade the OS.

use crate::icons;
use crate::theme::{corner, Fonts, Theme};
use egui::{Key, RichText, Sense};
use libonui_core::packages::{manager_for, PackageManagerKind};
use libonui_core::version::{self, UpdateKind, UpdateStatus};
use libonui_core::{channels, system};

#[derive(Clone)]
pub struct TerminalState {
    lines: Vec<Line>,
    input: String,
    cwd: String,
}

#[derive(Clone)]
struct Line {
    text: String,
    kind: LineKind,
}

#[derive(Clone, Copy, PartialEq)]
enum LineKind {
    Input,
    Output,
    Error,
    Muted,
}

impl Default for TerminalState {
    fn default() -> Self {
        let mut s = Self {
            lines: Vec::new(),
            input: String::new(),
            cwd: "~".into(),
        };
        s.push(
            format!("{} — terminale di LibonOS", libonui_core::banner()),
            LineKind::Muted,
        );
        s.push(
            "Scrivi `libon help` per l'elenco dei comandi.",
            LineKind::Muted,
        );
        s
    }
}

impl TerminalState {
    fn push(&mut self, text: impl Into<String>, kind: LineKind) {
        self.lines.push(Line {
            text: text.into(),
            kind,
        });
        if self.lines.len() > 500 {
            self.lines.drain(0..50);
        }
    }

    fn run(&mut self, line: String) {
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        self.push(format!("{} $ {}", self.cwd, trimmed), LineKind::Input);
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        match parts.as_slice() {
            ["clear"] => self.lines.clear(),
            ["help"] | ["libon", "help"] | ["libon", "--help"] => self.help(),
            ["libon", "status"] | ["status"] => self.status(),
            ["libon", "update"] | ["update"] => self.update(),
            ["libon", "upgrade"] | ["upgrade"] => self.upgrade(),
            ["libon", "install", app] => {
                let plan = libonui_core::packages::appimage::resolve(app, &std::env::temp_dir());
                match plan {
                    Some(p) => {
                        self.push(format!("Risolvo {}…", p.display_name), LineKind::Output);
                        self.push(format!("URL: {}", p.url), LineKind::Muted);
                        self.push(
                            format!("Scaricherei in {}", p.dest.display()),
                            LineKind::Output,
                        );
                        self.push(
                            "In una sessione reale libon scarica l'AppImage e la registra nel menu.",
                            LineKind::Muted,
                        );
                    }
                    None => self.push(
                        format!("Nessuna AppImage conosciuta per «{app}»."),
                        LineKind::Error,
                    ),
                }
            }
            ["libon", "search", rest @ ..] => {
                let kind = PackageManagerKind::Libon;
                let results = manager_for(kind).search(&rest.join(" "));
                if results.is_empty() {
                    self.push("Nessun risultato.", LineKind::Muted);
                } else {
                    for p in results {
                        self.push(
                            format!(
                                "{}  {}  {}",
                                p.id,
                                version::human_size(p.size_bytes),
                                p.summary
                            ),
                            LineKind::Output,
                        );
                    }
                }
            }
            ["neofetch"] | ["libon", "info"] => self.neofetch(),
            ["ls"] => {
                self.push(
                    "Applications/  Documenti/  Immagini/  Musica/  Scaricati/  Video/",
                    LineKind::Output,
                );
            }
            ["whoami"] => self.push("utente libon", LineKind::Output),
            ["uname", "-a"] => {
                self.push("LibonOS 7.4-libon x86_64 (kernel Linux)", LineKind::Output)
            }
            ["exit"] => self.push("Chiudi la finestra per uscire.", LineKind::Muted),
            [cmd, ..] => self.push(
                format!("{cmd}: comando non riconosciuto in questa shell limitata."),
                LineKind::Error,
            ),
            [] => {}
        }
    }

    fn help(&mut self) {
        for l in [
            "libon install <app>   cerca l'app, scarica l'AppImage, la registra",
            "libon search <testo>  cerca nel catalogo",
            "libon update          aggiornamento in-place di OS e app",
            "libon upgrade         info sull'aggiornamento di sistema completo",
            "libon status          versione, canale, stato",
            "neofetch              riepilogo del sistema",
            "clear                 pulisce il terminale",
        ] {
            self.push(l, LineKind::Output);
        }
        self.push(
            "Un aggiornamento completo (es. 1.0.0 → 2.0.0) si applica da Impostazioni.",
            LineKind::Muted,
        );
    }

    fn status(&mut self) {
        let current = system::os_version();
        let channel = channels::bundled_stable();
        let st = UpdateStatus::check(&current, &channel);
        let (name, release) = system::os_identity();
        self.push(format!("{name} {release}"), LineKind::Output);
        self.push(format!("canale: {}", channel.name), LineKind::Muted);
        self.push(
            match st.kind {
                UpdateKind::UpToDate => "stato: aggiornato".to_string(),
                _ => format!("stato: aggiornamento {} disponibile", st.target),
            },
            LineKind::Output,
        );
    }

    fn update(&mut self) {
        let current = system::os_version();
        let channel = channels::bundled_stable();
        let st = UpdateStatus::check(&current, &channel);
        match st.kind {
            UpdateKind::UpToDate => self.push("Nessun aggiornamento in-place.", LineKind::Muted),
            _ => {
                self.push(
                    format!(
                        "Aggiornamento in-place {} → {} disponibile.",
                        st.current, st.target
                    ),
                    LineKind::Output,
                );
                self.push(
                    "In una sessione reale libon scarica e applica l'aggiornamento.",
                    LineKind::Muted,
                );
            }
        }
        if let Some(major) = UpdateStatus::check_major_upgrade(&current, &channel) {
            self.push(
                format!(
                    "Nota: LibonOS {} è disponibile come aggiornamento di sistema completo. Applicalo da Impostazioni.",
                    major.target
                ),
                LineKind::Muted,
            );
        }
    }

    fn upgrade(&mut self) {
        let current = system::os_version();
        let channel = channels::bundled_stable();
        match UpdateStatus::check_major_upgrade(&current, &channel) {
            Some(major) => {
                self.push(
                    format!(
                        "Aggiornamento completo disponibile: {} → {}",
                        current, major.target
                    ),
                    LineKind::Output,
                );
                self.push(major.summary.clone(), LineKind::Muted);
                self.push(
                    "Questo aggiornamento si avvia solo da Impostazioni › Aggiornamenti.",
                    LineKind::Error,
                );
            }
            None => self.push(
                "Nessun aggiornamento completo disponibile.",
                LineKind::Muted,
            ),
        }
    }

    fn neofetch(&mut self) {
        let (name, release) = system::os_identity();
        self.push(
            format!("        /\\        {name} {release}"),
            LineKind::Output,
        );
        self.push("       /  \\       Shell: LibonUI (Rust)", LineKind::Output);
        self.push(
            "      / /\\ \\      Kernel: Linux 7.4-libon",
            LineKind::Output,
        );
        self.push(
            "     / ____ \\     Gestore: libon (preinstallato)",
            LineKind::Output,
        );
        self.push("    /_/    \\_\\    Browser: Firefox", LineKind::Output);
    }
}

pub fn show(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts) {
    let id = egui::Id::new("libon-terminal-state");
    let mut state = ui
        .ctx()
        .data_mut(|d| d.get_temp::<TerminalState>(id))
        .unwrap_or_default();

    ui.horizontal(|ui| {
        icons::TERMINAL.show(ui, 18.0, theme.accent());
        ui.add_space(6.0);
        ui.label(
            RichText::new("Terminale")
                .font(fonts.heading())
                .color(theme.text()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{} · libon", state.cwd))
                    .font(fonts.caption())
                    .color(theme.text_muted()),
            );
        });
    });
    ui.add_space(theme.tokens.space.sm);

    // Output.
    let area_h = (ui.available_height() - 44.0).max(80.0);
    egui::Frame::none()
        .fill(crate::theme::c(theme.tokens.canvas))
        .rounding(corner(theme.tokens.radius.md))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_min_height(area_h);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &state.lines {
                        let color = match line.kind {
                            LineKind::Input => theme.accent(),
                            LineKind::Output => theme.text(),
                            LineKind::Error => theme.danger(),
                            LineKind::Muted => theme.text_muted(),
                        };
                        ui.label(RichText::new(&line.text).font(fonts.mono()).color(color));
                    }
                });
        });

    // Prompt.
    ui.add_space(theme.tokens.space.sm);
    ui.horizontal(|ui| {
        ui.label(RichText::new("$").font(fonts.mono()).color(theme.accent()));
        let resp = ui.add(
            egui::TextEdit::singleline(&mut state.input)
                .font(fonts.mono())
                .hint_text("libon help")
                .desired_width(f32::INFINITY)
                .frame(false),
        );
        let submitted = resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        resp.request_focus();
        if submitted {
            let cmd = std::mem::take(&mut state.input);
            state.run(cmd);
        }
    });
    let _ = Sense::hover();

    ui.ctx().data_mut(|d| d.insert_temp(id, state));
}
