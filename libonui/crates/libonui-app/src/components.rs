//! The shared control set: buttons, fields, cards, meters and states.
//!
//! Everything the screens draw goes through here, so hover, focus, disabled and
//! error states look the same everywhere and a change to the system is a change
//! in one place. Focus rings are drawn explicitly so keyboard navigation is
//! visible on every control, not just the ones egui happens to outline.

use crate::icons::Icon;
use crate::theme::{c, corner, Theme};
use egui::{Align, Color32, Layout, Rect, Response, RichText, Sense, Stroke, TextEdit, Ui, Vec2};

/// The visual weight of a button.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    /// The single most important action on a surface.
    Primary,
    /// A normal action.
    Secondary,
    /// A quiet action that reveals itself on hover.
    Ghost,
    /// A destructive action.
    Danger,
}

/// A button with an optional leading icon.
pub struct Button<'a> {
    label: &'a str,
    icon: Option<&'static Icon>,
    kind: ButtonKind,
    enabled: bool,
    full_width: bool,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            kind: ButtonKind::Secondary,
            enabled: true,
            full_width: false,
        }
    }
    pub fn primary(mut self) -> Self {
        self.kind = ButtonKind::Primary;
        self
    }
    pub fn ghost(mut self) -> Self {
        self.kind = ButtonKind::Ghost;
        self
    }
    pub fn danger(mut self) -> Self {
        self.kind = ButtonKind::Danger;
        self
    }
    pub fn icon(mut self, icon: &'static Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn show(self, ui: &mut Ui, theme: &Theme) -> Response {
        let fonts = crate::theme::Fonts::from(&theme.tokens);
        let space = theme.tokens.space;

        let galley = ui.painter().layout_no_wrap(
            self.label.to_string(),
            fonts.label(),
            Color32::PLACEHOLDER,
        );
        let icon_w = if self.icon.is_some() { 20.0 } else { 0.0 };
        let pad_x = space.lg;
        let desired = Vec2::new((galley.size().x + icon_w + pad_x * 2.0).max(72.0), 38.0);
        let desired = if self.full_width {
            Vec2::new(ui.available_width(), desired.y)
        } else {
            desired
        };

        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(desired, sense);

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let hovered = self.enabled && response.hovered();
        let pressed = self.enabled && response.is_pointer_button_down_on();
        let focused = response.has_focus();

        let (fill, stroke_color, text_color) = match self.kind {
            ButtonKind::Primary => {
                let fill = if !self.enabled {
                    c(theme.tokens.accent_soft)
                } else if pressed {
                    c(theme.tokens.accent).gamma_multiply(0.88)
                } else if hovered {
                    c(theme.tokens.accent_hover)
                } else {
                    c(theme.tokens.accent)
                };
                let text = if self.enabled {
                    theme.on_accent()
                } else {
                    theme.text_muted()
                };
                (fill, Color32::TRANSPARENT, text)
            }
            ButtonKind::Secondary => {
                let fill = if !self.enabled {
                    c(theme.tokens.surface).gamma_multiply(0.6)
                } else if pressed || hovered {
                    c(theme.tokens.surface_overlay)
                } else {
                    c(theme.tokens.surface_raised)
                };
                let stroke = if hovered {
                    c(theme.tokens.border_strong)
                } else {
                    c(theme.tokens.border)
                };
                let text = if self.enabled {
                    theme.text()
                } else {
                    theme.text_muted()
                };
                (fill, stroke, text)
            }
            ButtonKind::Ghost => {
                let fill = if hovered {
                    c(theme.tokens.surface_raised)
                } else {
                    Color32::TRANSPARENT
                };
                let text = if self.enabled {
                    theme.text_secondary()
                } else {
                    theme.text_muted()
                };
                (fill, Color32::TRANSPARENT, text)
            }
            ButtonKind::Danger => {
                let base = c(theme.tokens.danger);
                let fill = if !self.enabled {
                    theme.surface_raised()
                } else if pressed {
                    base.gamma_multiply(0.85)
                } else if hovered {
                    base.gamma_multiply(1.1)
                } else {
                    base
                };
                (
                    fill,
                    Color32::TRANSPARENT,
                    Color32::from_rgb(0x1a, 0x08, 0x0a),
                )
            }
        };

        let painter = ui.painter();
        let rounding = corner(theme.tokens.radius.md);
        painter.rect_filled(rect, rounding, fill);
        if stroke_color != Color32::TRANSPARENT {
            painter.rect_stroke(rect, rounding, Stroke::new(1.0_f32, stroke_color));
        }
        if focused {
            painter.rect_stroke(
                rect.expand(2.0),
                corner(theme.tokens.radius.md + 2.0),
                Stroke::new(2.0_f32, c(theme.tokens.focus_ring)),
            );
        }

        // Lay the icon and label out as one centred group.
        let content_w = galley.size().x + icon_w;
        let mut x = rect.center().x - content_w / 2.0;
        if let Some(icon) = self.icon {
            let icon_rect =
                Rect::from_min_size(egui::pos2(x, rect.center().y - 9.0), Vec2::splat(18.0));
            icon.paint(ui, icon_rect, text_color, 1.6);
            x += icon_w;
        }
        let text_pos = egui::pos2(x, rect.center().y - galley.size().y / 2.0);
        ui.painter().galley(text_pos, galley, text_color);

        if hovered && self.enabled {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        response
    }
}

/// A labelled text field with an optional hint, an error message, and a
/// password mask. Returns the inner response so callers can request focus.
pub struct Field<'a> {
    value: &'a mut String,
    hint: &'a str,
    label: Option<&'a str>,
    error: Option<&'a str>,
    password: bool,
    icon: Option<&'static Icon>,
}

impl<'a> Field<'a> {
    pub fn new(value: &'a mut String, hint: &'a str) -> Self {
        Self {
            value,
            hint,
            label: None,
            error: None,
            password: false,
            icon: None,
        }
    }
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
    pub fn error(mut self, error: Option<&'a str>) -> Self {
        self.error = error;
        self
    }
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
    pub fn icon(mut self, icon: &'static Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn show(self, ui: &mut Ui, theme: &Theme) -> Response {
        let fonts = crate::theme::Fonts::from(&theme.tokens);
        let space = theme.tokens.space;

        if let Some(label) = self.label {
            ui.label(
                RichText::new(label)
                    .font(fonts.label())
                    .color(theme.text_secondary()),
            );
            ui.add_space(space.xs);
        }

        let has_error = self.error.is_some();
        let id = ui.next_auto_id();
        let height = 40.0;
        let icon_w = if self.icon.is_some() { 26.0 } else { 0.0 };
        let inner_pad = space.md + icon_w;

        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());

        let painter = ui.painter();
        let rounding = corner(theme.tokens.radius.md);
        painter.rect_filled(rect, rounding, c(theme.tokens.canvas));

        let mut child_ui =
            ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(Vec2::new(inner_pad, 0.0))));
        child_ui.vertical_centered_justified(|ui| {
            ui.add_space((height - fonts.body().size) / 2.0 - 4.0);
            let mut edit = TextEdit::singleline(self.value)
                .hint_text(self.hint)
                .font(fonts.body())
                .desired_width(f32::INFINITY)
                .frame(false)
                .id(id);
            if self.password {
                edit = edit.password(true);
            }
            ui.add(edit)
        });

        // Border and focus ring are drawn after the inner widget so the field's
        // own frame never paints over them.
        let focused = ui.memory(|m| m.has_focus(id));
        let border = if has_error {
            theme.danger()
        } else if focused {
            c(theme.tokens.accent)
        } else {
            theme.border()
        };
        ui.painter().rect_stroke(
            rect,
            rounding,
            Stroke::new(if focused || has_error { 1.6 } else { 1.0_f32 }, border),
        );

        if let Some(icon) = self.icon {
            let icon_rect = Rect::from_min_size(
                egui::pos2(rect.min.x + space.md, rect.center().y - 9.0),
                Vec2::splat(18.0),
            );
            icon.paint(
                ui,
                icon_rect,
                if focused {
                    theme.text_secondary()
                } else {
                    theme.text_muted()
                },
                1.6,
            );
        }

        if let Some(err) = self.error {
            ui.add_space(space.xs);
            ui.label(
                RichText::new(err)
                    .font(fonts.caption())
                    .color(theme.danger()),
            );
        }

        ui.interact(rect, id, Sense::click())
    }
}

/// A rounded surface that groups related content.
pub fn card<R>(ui: &mut Ui, theme: &Theme, elevated: bool, add: impl FnOnce(&mut Ui) -> R) -> R {
    let space = theme.tokens.space;
    let fill = if elevated {
        c(theme.tokens.surface_raised)
    } else {
        c(theme.tokens.surface)
    };
    let rounding = corner(theme.tokens.radius.lg);
    egui::Frame::none()
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, theme.border()))
        .rounding(rounding)
        .inner_margin(egui::Margin::same(space.lg))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// A small pill, for status and counts.
pub fn pill(ui: &mut Ui, theme: &Theme, text: &str, color: Color32) {
    let fonts = crate::theme::Fonts::from(&theme.tokens);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), fonts.caption(), color);
    let pad = Vec2::new(9.0, 3.0);
    let size = galley.size() + pad * 2.0;
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let bg = color.gamma_multiply(0.16);
    ui.painter()
        .rect_filled(rect, corner(theme.tokens.radius.pill), bg);
    ui.painter().galley(rect.min + pad, galley, color);
}

/// A status dot with a label, used in the panel and in Settings.
pub fn status_dot(ui: &mut Ui, theme: &Theme, color: Color32, text: &str) {
    let fonts = crate::theme::Fonts::from(&theme.tokens);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, color);
        ui.label(
            RichText::new(text)
                .font(fonts.caption())
                .color(theme.text_secondary()),
        );
    });
}

/// A titled section with a rule under the heading.
pub fn section(ui: &mut Ui, theme: &Theme, title: &str, add: impl FnOnce(&mut Ui)) {
    let fonts = crate::theme::Fonts::from(&theme.tokens);
    let space = theme.tokens.space;
    ui.label(
        RichText::new(title)
            .font(fonts.heading())
            .color(theme.text())
            .strong(),
    );
    ui.add_space(space.sm);
    add(ui);
    ui.add_space(space.lg);
}

/// A switch. Returns true when the value changed.
pub fn toggle(ui: &mut Ui, theme: &Theme, on: &mut bool) -> bool {
    let desired = Vec2::new(44.0, 24.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click());
    let hovered = response.hovered();
    let t = ui.ctx().animate_bool_responsive(response.id, *on);
    let track = if *on {
        c(theme.tokens.accent).lerp_to_gamma(c(theme.tokens.accent_hover), t)
    } else {
        c(theme.tokens.border_strong)
    };
    ui.painter()
        .rect_filled(rect, corner(theme.tokens.radius.pill), track);
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            corner(theme.tokens.radius.pill + 2.0),
            Stroke::new(2.0_f32, c(theme.tokens.focus_ring)),
        );
    }
    let knob_x = egui::lerp((rect.left() + 13.0)..=(rect.right() - 13.0), t);
    let knob = egui::pos2(knob_x, rect.center().y);
    ui.painter().circle_filled(knob, 9.0, Color32::WHITE);
    if response.clicked() {
        *on = !*on;
        return true;
    }
    false
}

/// A labelled toggle row: title, description, and the switch on the right.
pub fn toggle_row(
    ui: &mut Ui,
    theme: &Theme,
    title: &str,
    description: &str,
    value: &mut bool,
) -> bool {
    let fonts = crate::theme::Fonts::from(&theme.tokens);
    let space = theme.tokens.space;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).font(fonts.body()).color(theme.text()));
            ui.label(
                RichText::new(description)
                    .font(fonts.caption())
                    .color(theme.text_muted()),
            );
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(space.sm);
            changed = toggle(ui, theme, value);
        });
    });
    changed
}

/// A progress bar in the accent colour. `fraction` of `None` draws an
/// indeterminate sweep.
pub fn progress(ui: &mut Ui, theme: &Theme, fraction: Option<f32>, height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let rounding = corner(height / 2.0);
    ui.painter()
        .rect_filled(rect, rounding, c(theme.tokens.surface_overlay));
    match fraction {
        Some(f) => {
            let w = (rect.width() * f.clamp(0.0, 1.0)).max(height);
            let filled = Rect::from_min_size(rect.min, Vec2::new(w, height));
            ui.painter()
                .rect_filled(filled, rounding, c(theme.tokens.accent));
        }
        None => {
            let t = ui.input(|i| i.time) as f32;
            let phase = (t * 0.6).fract();
            let w = rect.width() * 0.28;
            let x = rect.left() - w + phase * (rect.width() + w);
            let seg = Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::new(w, height))
                .intersect(rect);
            if seg.width() > 0.0 {
                ui.painter()
                    .rect_filled(seg, rounding, c(theme.tokens.accent));
            }
            ui.ctx().request_repaint();
        }
    }
}

/// A password strength meter: four segments, coloured by the score.
pub fn strength_meter(ui: &mut Ui, theme: &Theme, score: u8) {
    let labels = ["Molto debole", "Debole", "Discreta", "Buona", "Ottima"];
    let colors = [
        theme.danger(),
        theme.danger(),
        theme.warning(),
        theme.success(),
        theme.success(),
    ];
    let space = theme.tokens.space;
    let fonts = crate::theme::Fonts::from(&theme.tokens);
    ui.horizontal(|ui| {
        for i in 0..4u8 {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(46.0, 5.0), Sense::hover());
            let filled = i < score;
            let color = if filled {
                colors[score.min(4) as usize]
            } else {
                c(theme.tokens.surface_overlay)
            };
            ui.painter().rect_filled(rect, corner(2.5), color);
        }
        ui.add_space(space.sm);
        ui.label(
            RichText::new(labels[score.min(4) as usize])
                .font(fonts.caption())
                .color(colors[score.min(4) as usize]),
        );
    });
}

/// An empty state: an icon, a title, and a sentence about what to do.
pub fn empty_state(ui: &mut Ui, theme: &Theme, icon: &Icon, title: &str, body: &str) {
    let fonts = crate::theme::Fonts::from(&theme.tokens);
    let space = theme.tokens.space;
    ui.add_space(space.xxl);
    ui.vertical_centered(|ui| {
        icon.show(ui, 40.0, theme.text_muted());
        ui.add_space(space.md);
        ui.label(
            RichText::new(title)
                .font(fonts.heading())
                .color(theme.text_secondary()),
        );
        ui.add_space(space.xs);
        ui.label(
            RichText::new(body)
                .font(fonts.label())
                .color(theme.text_muted()),
        );
    });
    ui.add_space(space.xxl);
}

/// A full-surface background: a soft two-tone wash with a faint accent glow,
/// generated from the palette rather than shipped as a bitmap.
pub fn wallpaper(ui: &Ui, theme: &Theme, style: libonui_core::WallpaperStyle) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    let top = c(theme.tokens.wallpaper_top);
    let bottom = c(theme.tokens.wallpaper_bottom);
    let steps = 48;
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let y0 = egui::lerp(rect.top()..=rect.bottom(), t);
        let y1 = egui::lerp(rect.top()..=rect.bottom(), (i + 1) as f32 / steps as f32);
        let band = Rect::from_min_max(egui::pos2(rect.left(), y0), egui::pos2(rect.right(), y1));
        painter.rect_filled(band, 0.0, top.lerp_to_gamma(bottom, t));
    }
    match style {
        libonui_core::WallpaperStyle::Solid => {}
        libonui_core::WallpaperStyle::Aurora => {
            let center = egui::pos2(
                rect.right() - rect.width() * 0.28,
                rect.top() + rect.height() * 0.22,
            );
            painter.circle_filled(
                center,
                rect.width() * 0.32,
                theme.accent().gamma_multiply(0.10),
            );
            let center2 = egui::pos2(
                rect.left() + rect.width() * 0.18,
                rect.bottom() - rect.height() * 0.18,
            );
            painter.circle_filled(
                center2,
                rect.width() * 0.26,
                theme.accent_alt().gamma_multiply(0.07),
            );
        }
        libonui_core::WallpaperStyle::Mesh => {
            let t = ui.input(|i| i.time) as f32;
            for k in 0..5 {
                let f = k as f32;
                let x = rect.left() + rect.width() * (0.5 + 0.42 * (t * 0.05 + f * 1.7).sin());
                let y = rect.top() + rect.height() * (0.5 + 0.38 * (t * 0.04 + f * 2.3).cos());
                let color = if k % 2 == 0 {
                    theme.accent()
                } else {
                    theme.accent_alt()
                };
                painter.circle_filled(
                    egui::pos2(x, y),
                    rect.width() * 0.22,
                    color.gamma_multiply(0.055),
                );
            }
            ui.ctx().request_repaint();
        }
    }
}
