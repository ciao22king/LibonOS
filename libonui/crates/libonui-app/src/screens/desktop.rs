//! The desktop: top panel, floating dock, window frames, notifications, and the
//! dispatch that decides which screen draws inside each window.

use super::{files, settings, store, terminal, text_editor, Action};
use crate::app::{Desktop, NotifyKind};
use crate::components::{self, Button};
use crate::icons;
use crate::theme::{corner, Fonts, Theme};
use egui::{Align, Context, Layout, Rect, RichText, Sense, Stroke, Vec2};
use libonui_core::system;
use libonui_core::wm::{WindowId, WindowState};

const PANEL_H: f32 = 36.0;
const DOCK_H: f32 = 60.0;

pub fn show(
    ctx: &Context,
    theme: &Theme,
    fonts: &Fonts,
    desktop: &mut Desktop,
    should_exit: &mut bool,
) -> Action {
    desktop.tick();
    let mut action = Action::None;

    let screen = ctx.screen_rect();
    desktop
        .session
        .wm
        .set_viewport(screen.width(), screen.height() - PANEL_H - DOCK_H - 8.0);

    // A dark scrim so windows read as floating on the wallpaper.
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(crate::theme::c(theme.tokens.canvas)))
        .show(ctx, |ui| {
            components::wallpaper(ui, theme, desktop.wallpaper());
        });

    top_panel(ctx, theme, fonts, desktop);

    // Advance store installs and notify on completion.
    let finished = desktop.store.tick();
    for package in finished {
        desktop.notify(
            format!("{} installato", package.name),
            "Lo trovi nel menu applicazioni e nella ricerca.",
            NotifyKind::Success,
        );
    }
    // Notices the Store raised this frame (a missing backend, a failed step).
    let notices = std::mem::take(&mut desktop.store.notices);
    for (title, body, kind) in notices {
        desktop.notify(title, body, kind);
    }

    windows(ctx, theme, fonts, desktop);
    dock(ctx, theme, desktop);
    start_menu(ctx, theme, fonts, desktop);
    session_menu(ctx, theme, fonts, desktop, &mut action, should_exit);
    notifications(ctx, theme, fonts, desktop);
    notification_centre(ctx, theme, fonts, desktop);

    action
}

/// The session menu behind the user's name: lock, settings, log out, power off.
/// Whether a click landed outside `panel` this frame, the signal every overlay
/// uses to dismiss itself.
fn clicked_outside(ctx: &Context, panel: Rect) -> bool {
    ctx.input(|i| i.pointer.any_click())
        && ctx
            .input(|i| i.pointer.interact_pos())
            .map(|p| !panel.contains(p))
            .unwrap_or(false)
}

fn session_menu(
    ctx: &Context,
    theme: &Theme,
    fonts: &Fonts,
    desktop: &mut Desktop,
    action: &mut Action,
    should_exit: &mut bool,
) {
    if !desktop.session_menu_open {
        return;
    }
    desktop.bell_open = false;
    let screen = ctx.screen_rect();
    let width = 230.0;
    let height = 4.0 * 38.0 + 24.0;
    let pos = egui::pos2(screen.max.x - width - 12.0, PANEL_H + 8.0);
    let panel = Rect::from_min_size(pos, Vec2::new(width, height));

    let clicked_outside = clicked_outside(ctx, panel);

    egui::Area::new(egui::Id::new("libon-session"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            ui.painter()
                .rect_filled(panel, corner(theme.tokens.radius.lg), theme.glass(246));
            ui.painter().rect_stroke(
                panel,
                corner(theme.tokens.radius.lg),
                Stroke::new(1.0_f32, theme.border_strong()),
            );
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(panel.shrink(8.0)), |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(format!("@{}", desktop.session.username))
                            .font(fonts.caption())
                            .color(theme.text_muted()),
                    );
                    ui.add_space(theme.tokens.space.xs);
                    let items: [(&str, &'static icons::Icon); 4] = [
                        ("Account", &icons::USER),
                        ("Blocca", &icons::LOCK),
                        ("Esci dalla sessione", &icons::CHEVRON_LEFT),
                        ("Spegni", &icons::POWER),
                    ];
                    for (label, icon) in items {
                        let (r, resp) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 34.0),
                            Sense::click(),
                        );
                        let hovered = resp.hovered();
                        if hovered {
                            ui.painter().rect_filled(
                                r,
                                corner(theme.tokens.radius.sm),
                                theme.surface_raised(),
                            );
                        }
                        icon.paint(
                            ui,
                            Rect::from_min_size(
                                egui::pos2(r.min.x + 10.0, r.center().y - 8.0),
                                Vec2::splat(16.0),
                            ),
                            if hovered {
                                theme.accent()
                            } else {
                                theme.text_secondary()
                            },
                            1.6,
                        );
                        ui.painter().text(
                            egui::pos2(r.min.x + 36.0, r.center().y),
                            egui::Align2::LEFT_CENTER,
                            label,
                            fonts.label(),
                            theme.text(),
                        );
                        if hovered {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if resp.clicked() {
                            match label {
                                "Account" => {
                                    desktop.session.launch("settings", "Impostazioni");
                                    desktop.settings_tab = crate::app::SettingsTab::Account;
                                }
                                "Blocca" | "Esci dalla sessione" => *action = Action::LogOut,
                                "Spegni" => {
                                    *action = Action::Exit;
                                    *should_exit = true;
                                }
                                _ => {}
                            }
                            desktop.session_menu_open = false;
                        }
                    }
                });
            });
        });

    if clicked_outside {
        desktop.session_menu_open = false;
    }
}

fn top_panel(ctx: &Context, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    egui::TopBottomPanel::top("libon-panel")
        .exact_height(PANEL_H)
        .frame(
            egui::Frame::none()
                .fill(theme.glass(232))
                .stroke(Stroke::new(1.0_f32, theme.border()))
                .inner_margin(egui::Margin::symmetric(10.0, 0.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // Activities button.
                let resp = Button::new("Attività")
                    .ghost()
                    .icon(&icons::GRID)
                    .show(ui, theme);
                if resp.clicked() {
                    desktop.session.start_menu_open = !desktop.session.start_menu_open;
                }
                ui.add_space(theme.tokens.space.xs);

                // The focused window's title.
                let title = desktop
                    .session
                    .wm
                    .focused()
                    .map(|w| w.title.clone())
                    .unwrap_or_else(|| "Desktop".to_string());
                ui.label(RichText::new(title).font(fonts.label()).color(theme.text()));

                // Right side: status, clock, and the session menu.
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let resp = Button::new(&desktop.session.user_first_name)
                        .ghost()
                        .icon(&icons::USER)
                        .show(ui, theme);
                    if resp.clicked() {
                        desktop.session_menu_open = !desktop.session_menu_open;
                    }
                    ui.add_space(theme.tokens.space.sm);

                    // Quick status indicators.
                    components::status_dot(ui, theme, theme.success(), "Rete");
                    ui.add_space(theme.tokens.space.sm);
                    icons::VOLUME.show(ui, 17.0, theme.text_secondary());
                    ui.add_space(theme.tokens.space.sm);
                    icons::BATTERY.show(ui, 18.0, theme.text_secondary());
                    ui.add_space(theme.tokens.space.md);
                    ui.label(
                        RichText::new(desktop.clock.format("%H:%M").to_string())
                            .font(fonts.label())
                            .strong()
                            .color(theme.text()),
                    );
                    ui.add_space(theme.tokens.space.md);
                    if bell_button(ui, theme, desktop).clicked() {
                        desktop.bell_open = !desktop.bell_open;
                        desktop.session_menu_open = false;
                    }
                });
            });
        });
}

/// Draw every visible window back to front, with its frame and controls.
fn windows(ctx: &Context, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    let visible: Vec<WindowId> = desktop
        .session
        .wm
        .visible_back_to_front()
        .iter()
        .map(|w| w.id)
        .collect();
    let focused = desktop.session.wm.focused_id();

    for id in visible {
        let Some(win) = desktop.session.wm.get(id) else {
            continue;
        };
        let rect = window_rect(win, ctx.screen_rect());
        let is_focused = focused == Some(id);
        let app_id = win.app_id.clone();
        let title = win.title.clone();
        let state = win.state;

        let area = egui::Area::new(egui::Id::new(("window", id)))
            .order(egui::Order::Middle)
            .fixed_pos(rect.min);
        let inner = area.show(ctx, |ui| {
            ui.set_clip_rect(rect.intersect(ctx.screen_rect()));
            let frame_rect = Rect::from_min_size(rect.min, rect.size());
            let rounding = corner(theme.tokens.radius.lg);
            let painter = ui.painter();
            painter.rect_filled(frame_rect, rounding, theme.surface());
            painter.rect_stroke(
                frame_rect,
                rounding,
                Stroke::new(
                    if is_focused { 1.4 } else { 1.0_f32 },
                    if is_focused {
                        crate::theme::c(theme.tokens.border_strong)
                    } else {
                        theme.border()
                    },
                ),
            );

            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(frame_rect.shrink(1.0)));
            child.vertical(|ui| {
                // Title bar.
                let (bar, bar_resp) = ui.allocate_exact_size(
                    Vec2::new(frame_rect.width() - 2.0, 36.0),
                    Sense::click_and_drag(),
                );
                draw_title_bar(
                    ui,
                    theme,
                    fonts,
                    bar,
                    &title,
                    &app_id,
                    is_focused,
                    state,
                    &mut |a| {
                        apply_window_action(desktop, id, a);
                    },
                );

                // Focus on any click in the frame.
                if bar_resp.clicked() {
                    desktop.session.wm.focus(id);
                }
                if bar_resp.dragged() {
                    let delta = bar_resp.drag_delta();
                    desktop.session.wm.move_by(id, delta.x, delta.y);
                }

                ui.separator();

                // Body.
                let body = Rect::from_min_max(
                    egui::pos2(frame_rect.min.x + 1.0, bar.max.y + 1.0),
                    egui::pos2(frame_rect.max.x - 1.0, frame_rect.max.y - 1.0),
                );
                let mut body_ui = ui.new_child(egui::UiBuilder::new().max_rect(body.shrink(10.0)));
                let resp = body_ui.interact(body, egui::Id::new(("body", id)), Sense::click());
                if resp.clicked() {
                    desktop.session.wm.focus(id);
                }
                draw_app(&app_id, &mut body_ui, theme, fonts, desktop);

                // Resize grip in the corner.
                let grip = Rect::from_min_size(
                    egui::pos2(frame_rect.max.x - 16.0, frame_rect.max.y - 16.0),
                    Vec2::splat(16.0),
                );
                let grip_resp = ui.interact(grip, egui::Id::new(("grip", id)), Sense::drag());
                if grip_resp.dragged() {
                    let d = grip_resp.drag_delta();
                    desktop.session.wm.resize_by(id, d.x, d.y);
                }
            });
        });
        let _ = inner;
    }
}

#[derive(Clone, Copy)]
enum WinAction {
    Close,
    Minimize,
    Maximize,
}

fn apply_window_action(desktop: &mut Desktop, id: WindowId, a: WinAction) {
    match a {
        WinAction::Close => desktop.session.wm.close(id),
        WinAction::Minimize => desktop.session.wm.minimize(id),
        WinAction::Maximize => desktop.session.wm.toggle_maximize(id),
    }
}

fn window_rect(win: &libonui_core::wm::Window, screen: Rect) -> Rect {
    if win.state == WindowState::Maximized {
        Rect::from_min_max(
            egui::pos2(screen.min.x + 6.0, screen.min.y + PANEL_H + 6.0),
            egui::pos2(screen.max.x - 6.0, screen.max.y - DOCK_H - 6.0),
        )
    } else {
        Rect::from_min_size(egui::pos2(win.x, win.y + PANEL_H), Vec2::new(win.w, win.h))
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_title_bar(
    ui: &mut egui::Ui,
    theme: &Theme,
    fonts: &Fonts,
    rect: Rect,
    title: &str,
    app_id: &str,
    focused: bool,
    state: WindowState,
    on_action: &mut dyn FnMut(WinAction),
) {
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, theme.surface());

    // App icon + title on the left.
    let icon_rect = Rect::from_min_size(
        egui::pos2(rect.min.x + 10.0, rect.center().y - 8.0),
        Vec2::splat(16.0),
    );
    icons::for_app(app_id).paint(
        ui,
        icon_rect,
        if focused {
            theme.accent()
        } else {
            theme.text_muted()
        },
        1.6,
    );
    ui.painter().text(
        egui::pos2(icon_rect.max.x + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        title,
        fonts.label(),
        if focused {
            theme.text()
        } else {
            theme.text_muted()
        },
    );

    // Window controls on the right: minimize, maximize, close.
    let btn = 26.0;
    let mut x = rect.max.x - 8.0 - btn;
    let controls = [
        (WinAction::Close, &icons::CLOSE, theme.danger()),
        (
            WinAction::Maximize,
            if state == WindowState::Maximized {
                &icons::RESTORE
            } else {
                &icons::MAXIMIZE
            },
            theme.text_secondary(),
        ),
        (
            WinAction::Minimize,
            &icons::MINIMIZE,
            theme.text_secondary(),
        ),
    ];
    for (a, icon, color) in controls {
        let r = Rect::from_min_size(egui::pos2(x, rect.center().y - btn / 2.0), Vec2::splat(btn));
        let resp = ui.interact(r, egui::Id::new((title, x as i32)), Sense::click());
        let c = if resp.hovered() {
            if matches!(a, WinAction::Close) {
                theme.danger()
            } else {
                theme.text()
            }
        } else {
            color
        };
        if resp.hovered() {
            ui.painter()
                .rect_filled(r, corner(theme.tokens.radius.sm), theme.surface_raised());
        }
        icon.paint(ui, r.shrink(7.0), c, 1.7);
        if resp.clicked() {
            on_action(a);
        }
        x -= btn + 2.0;
    }
}

/// Route a window's content to the right screen.
fn draw_app(app_id: &str, ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    match app_id {
        "store" => store::show(ui, theme, fonts, &mut desktop.store),
        "settings" => settings::show(ui, theme, fonts, desktop),
        "terminal" => terminal::show(ui, theme, fonts),
        "files" => files::show(ui, theme, fonts),
        "text-editor" => text_editor::show(ui, theme, fonts),
        "help" => help_app(ui, theme, fonts),
        other => generic_app(ui, theme, fonts, other),
    }
}

fn help_app(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts) {
    ui.label(
        RichText::new("Aiuto di LibonOS")
            .font(fonts.title())
            .strong()
            .color(theme.text()),
    );
    ui.add_space(theme.tokens.space.md);
    for (title, body) in [
        ("Installare un programma", "Apri lo Store, cerca l'app e premi Installa. libon scarica un AppImage senza chiedere la password."),
        ("Aggiornare il sistema", "Da terminale: `libon update`. Un aggiornamento completo di versione si fa da Impostazioni › Aggiornamenti."),
        ("Cambiare gestore pacchetti", "Impostazioni › Pacchetti: scegli APT, pacman o libon."),
        ("Trovare un file", "Apri File e usa la ricerca, oppure la barra in alto."),
    ] {
        components::card(ui, theme, false, |ui| {
            ui.label(RichText::new(title).font(fonts.heading()).color(theme.text()));
            ui.add_space(theme.tokens.space.xs);
            ui.label(RichText::new(body).font(fonts.label()).color(theme.text_secondary()));
        });
        ui.add_space(theme.tokens.space.sm);
    }
}

fn generic_app(ui: &mut egui::Ui, theme: &Theme, fonts: &Fonts, app_id: &str) {
    let app = system::PREINSTALLED.iter().find(|a| a.id == app_id);
    let (name, summary) = app
        .map(|a| (a.name, a.summary))
        .unwrap_or(("Applicazione", "Nessuna descrizione."));
    ui.vertical_centered(|ui| {
        ui.add_space(theme.tokens.space.xl);
        icons::for_app(app_id).show(ui, 48.0, theme.accent());
        ui.add_space(theme.tokens.space.md);
        ui.label(
            RichText::new(name)
                .font(fonts.title())
                .strong()
                .color(theme.text()),
        );
        ui.add_space(theme.tokens.space.xs);
        ui.label(
            RichText::new(summary)
                .font(fonts.label())
                .color(theme.text_muted()),
        );
    });
}

/// The dock: pinned apps plus a running indicator dot.
fn dock(ctx: &Context, theme: &Theme, desktop: &mut Desktop) {
    let screen = ctx.screen_rect();
    let dock_w = (desktop.session.dock.len() as f32 * 52.0 + 28.0).min(screen.width() - 40.0);
    let pos = egui::pos2(
        screen.center().x - dock_w / 2.0,
        screen.max.y - DOCK_H - 10.0,
    );

    egui::Area::new(egui::Id::new("libon-dock"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            let rect = Rect::from_min_size(pos, Vec2::new(dock_w, DOCK_H));
            ui.painter()
                .rect_filled(rect, corner(theme.tokens.radius.lg + 6.0), theme.glass(214));
            ui.painter().rect_stroke(
                rect,
                corner(theme.tokens.radius.lg + 6.0),
                Stroke::new(1.0_f32, theme.border()),
            );

            ui.allocate_new_ui(
                egui::UiBuilder::new().max_rect(rect.shrink2(Vec2::new(14.0, 6.0))),
                |ui| {
                    ui.horizontal_centered(|ui| {
                        let items: Vec<(String, String)> = desktop
                            .session
                            .dock
                            .iter()
                            .map(|d| (d.app_id.clone(), d.name.clone()))
                            .collect();
                        for (app_id, name) in items {
                            let running = desktop.session.wm.app_is_running(&app_id);
                            let (r, resp) =
                                ui.allocate_exact_size(Vec2::splat(44.0), Sense::click());
                            let hovered = resp.hovered();
                            if hovered {
                                ui.painter().rect_filled(
                                    r,
                                    corner(theme.tokens.radius.md),
                                    theme.surface_raised(),
                                );
                            }
                            icons::for_app(&app_id).paint(
                                ui,
                                r.shrink(11.0),
                                if running || hovered {
                                    theme.accent()
                                } else {
                                    theme.text_secondary()
                                },
                                1.7,
                            );
                            if running {
                                ui.painter().circle_filled(
                                    egui::pos2(r.center().x, r.max.y - 3.0),
                                    2.4,
                                    theme.accent(),
                                );
                            }
                            if hovered {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                resp.clone().on_hover_text(name.clone());
                            }
                            if resp.clicked() {
                                desktop.session.launch(&app_id, &name);
                            }
                            ui.add_space(6.0);
                        }
                    });
                },
            );
        });
}

/// The application menu, opened from the Activities button.
fn start_menu(ctx: &Context, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    if !desktop.session.start_menu_open {
        return;
    }
    let screen = ctx.screen_rect();
    let panel = Rect::from_min_size(
        egui::pos2(12.0, PANEL_H + 8.0),
        Vec2::new(
            340.0,
            (screen.height() - PANEL_H - DOCK_H - 30.0).min(520.0),
        ),
    );

    // Clicking outside closes the menu.
    let clicked_outside = clicked_outside(ctx, panel);

    egui::Area::new(egui::Id::new("libon-start"))
        .order(egui::Order::Foreground)
        .fixed_pos(panel.min)
        .show(ctx, |ui| {
            ui.painter()
                .rect_filled(panel, corner(theme.tokens.radius.lg), theme.glass(246));
            ui.painter().rect_stroke(
                panel,
                corner(theme.tokens.radius.lg),
                Stroke::new(1.0_f32, theme.border_strong()),
            );
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(panel.shrink(12.0)), |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        icons::SEARCH.show(ui, 16.0, theme.text_muted());
                        ui.add_space(6.0);
                        let search = egui::TextEdit::singleline(&mut desktop.panel_search)
                            .hint_text("Cerca applicazioni…")
                            .desired_width(f32::INFINITY)
                            .frame(false);
                        ui.add(search);
                    });
                    ui.add_space(theme.tokens.space.sm);
                    ui.separator();
                    ui.add_space(theme.tokens.space.sm);

                    let query = desktop.panel_search.to_lowercase();
                    let mut launches: Vec<(String, String)> = Vec::new();
                    let apps: Vec<(&str, &str, &str)> = system::PREINSTALLED
                        .iter()
                        .map(|a| (a.id, a.name, a.summary))
                        .collect();
                    for (id, name, summary) in apps {
                        if !query.is_empty()
                            && !name.to_lowercase().contains(&query)
                            && !summary.to_lowercase().contains(&query)
                        {
                            continue;
                        }
                        let (r, resp) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 40.0),
                            Sense::click(),
                        );
                        let hovered = resp.hovered();
                        if hovered {
                            ui.painter().rect_filled(
                                r,
                                corner(theme.tokens.radius.sm),
                                theme.surface_raised(),
                            );
                        }
                        icons::for_app(id).paint(
                            ui,
                            Rect::from_min_size(
                                egui::pos2(r.min.x + 10.0, r.center().y - 9.0),
                                Vec2::splat(18.0),
                            ),
                            if hovered {
                                theme.accent()
                            } else {
                                theme.text_secondary()
                            },
                            1.6,
                        );
                        ui.painter().text(
                            egui::pos2(r.min.x + 38.0, r.center().y),
                            egui::Align2::LEFT_CENTER,
                            name,
                            fonts.label(),
                            theme.text(),
                        );
                        if hovered {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if resp.clicked() {
                            launches.push((id.to_string(), name.to_string()));
                        }
                    }
                    if launches.is_empty() && !query.is_empty() {
                        ui.add_space(theme.tokens.space.md);
                        ui.label(
                            RichText::new("Nessuna app. Prova con «store» o «file».")
                                .font(fonts.caption())
                                .color(theme.text_muted()),
                        );
                    }
                    for (id, name) in launches {
                        desktop.session.launch(&id, &name);
                    }
                });
            });
        });

    if clicked_outside {
        desktop.session.start_menu_open = false;
    }
}

/// Notification cards in the top-right, fading out with age.
fn notifications(ctx: &Context, theme: &Theme, fonts: &Fonts, desktop: &Desktop) {
    if desktop.notifications.is_empty() {
        return;
    }
    let screen = ctx.screen_rect();
    let mut y = PANEL_H
        + 12.0
        + if desktop.bell_open {
            centre_height(desktop) + 8.0
        } else {
            0.0
        };
    for note in desktop.notifications.iter().rev() {
        let age = note.age_secs();
        let alpha = if age > 6.5 {
            (1.0 - (age - 6.5) / 1.5).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let width = 320.0;
        let height = 66.0;
        let pos = egui::pos2(screen.max.x - width - 16.0, y);
        let color = match note.kind {
            NotifyKind::Info => theme.info(),
            NotifyKind::Success => theme.success(),
            NotifyKind::Warning => theme.warning(),
            NotifyKind::Error => theme.danger(),
        }
        .gamma_multiply(alpha);
        let rect = Rect::from_min_size(pos, Vec2::new(width, height));

        egui::Area::new(egui::Id::new(("note", note.title.as_str(), note.created)))
            .order(egui::Order::Tooltip)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                let bg = theme.surface_raised().gamma_multiply(alpha * 0.97);
                ui.painter()
                    .rect_filled(rect, corner(theme.tokens.radius.lg), bg);
                ui.painter().rect_stroke(
                    rect,
                    corner(theme.tokens.radius.lg),
                    Stroke::new(1.0_f32, theme.border().gamma_multiply(alpha)),
                );
                // A status stripe on the leading edge, drawn as a rounded accent
                // rather than a flat border-left.
                ui.painter().circle_filled(
                    egui::pos2(rect.min.x + 20.0, rect.min.y + 24.0),
                    5.0,
                    color,
                );
                ui.painter().text(
                    egui::pos2(rect.min.x + 36.0, rect.min.y + 18.0),
                    egui::Align2::LEFT_CENTER,
                    &note.title,
                    fonts.label(),
                    theme.text().gamma_multiply(alpha),
                );
                ui.painter().text(
                    egui::pos2(rect.min.x + 36.0, rect.min.y + 40.0),
                    egui::Align2::LEFT_CENTER,
                    &note.body,
                    fonts.caption(),
                    theme.text_muted().gamma_multiply(alpha),
                );
            });
        y += height + 10.0;
    }
    // Keep fading smoothly.
    ctx.request_repaint_after(std::time::Duration::from_millis(120));
}

/// The panel bell, with an unread dot when there are live notifications.
fn bell_button(ui: &mut egui::Ui, theme: &Theme, desktop: &Desktop) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
    let color = if resp.hovered() || desktop.bell_open {
        theme.accent()
    } else {
        theme.text_secondary()
    };
    icons::BELL.paint(ui, rect.shrink(5.0), color, 1.6);
    if !desktop.notifications.is_empty() && !desktop.bell_open {
        ui.painter().circle_filled(
            egui::pos2(rect.max.x - 5.0, rect.min.y + 5.0),
            3.5,
            theme.danger(),
        );
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// How tall the notification centre is for the current contents.
fn centre_height(desktop: &Desktop) -> f32 {
    let rows = desktop.notifications.len().min(5) as f32;
    let rows = if rows == 0.0 { 1.0 } else { rows };
    52.0 + rows * 62.0
}

/// The notification centre: what the bell opens, and the one place the running
/// log of events is readable after the toasts have faded.
fn notification_centre(ctx: &Context, theme: &Theme, fonts: &Fonts, desktop: &mut Desktop) {
    if !desktop.bell_open {
        return;
    }
    let screen = ctx.screen_rect();
    let width = 340.0;
    let height = centre_height(desktop);
    let pos = egui::pos2(screen.max.x - width - 12.0, PANEL_H + 8.0);
    let panel = Rect::from_min_size(pos, Vec2::new(width, height));

    let clicked_outside = clicked_outside(ctx, panel);

    egui::Area::new(egui::Id::new("libon-notification-centre"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            ui.painter()
                .rect_filled(panel, corner(theme.tokens.radius.lg), theme.glass(246));
            ui.painter().rect_stroke(
                panel,
                corner(theme.tokens.radius.lg),
                Stroke::new(1.0_f32, theme.border_strong()),
            );
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(panel.shrink(10.0)), |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Notifiche")
                                .font(fonts.heading())
                                .strong()
                                .color(theme.text()),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if !desktop.notifications.is_empty()
                                && Button::new("Cancella").ghost().show(ui, theme).clicked()
                            {
                                desktop.notifications.clear();
                            }
                        });
                    });
                    ui.add_space(theme.tokens.space.xs);

                    if desktop.notifications.is_empty() {
                        ui.add_space(theme.tokens.space.md);
                        ui.vertical_centered(|ui| {
                            icons::BELL.show(ui, 22.0, theme.text_muted());
                            ui.add_space(theme.tokens.space.xs);
                            ui.label(
                                RichText::new("Nessuna notifica")
                                    .font(fonts.label())
                                    .color(theme.text_muted()),
                            );
                        });
                        return;
                    }

                    for note in desktop.notifications.iter().rev().take(5) {
                        let (row, _) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 58.0),
                            Sense::hover(),
                        );
                        ui.painter().rect_filled(
                            row,
                            corner(theme.tokens.radius.md),
                            theme.surface(),
                        );
                        let color = match note.kind {
                            NotifyKind::Info => theme.info(),
                            NotifyKind::Success => theme.success(),
                            NotifyKind::Warning => theme.warning(),
                            NotifyKind::Error => theme.danger(),
                        };
                        ui.painter().circle_filled(
                            egui::pos2(row.min.x + 18.0, row.center().y),
                            4.5,
                            color,
                        );
                        ui.painter().text(
                            egui::pos2(row.min.x + 32.0, row.min.y + 20.0),
                            egui::Align2::LEFT_CENTER,
                            &note.title,
                            fonts.label(),
                            theme.text(),
                        );
                        ui.painter().text(
                            egui::pos2(row.min.x + 32.0, row.min.y + 40.0),
                            egui::Align2::LEFT_CENTER,
                            &note.body,
                            fonts.caption(),
                            theme.text_muted(),
                        );
                        ui.painter().text(
                            egui::pos2(row.max.x - 10.0, row.min.y + 20.0),
                            egui::Align2::RIGHT_CENTER,
                            format!("{:.0}s", note.age_secs()),
                            fonts.caption(),
                            theme.text_muted(),
                        );
                        ui.add_space(theme.tokens.space.xs);
                    }
                });
            });
        });

    if clicked_outside {
        desktop.bell_open = false;
    }
    ctx.request_repaint_after(std::time::Duration::from_millis(200));
}
