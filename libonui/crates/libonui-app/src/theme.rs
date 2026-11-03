//! Bridge from the design tokens in `libonui-core` to egui's `Visuals`.
//!
//! Every colour, radius and spacing value the shell draws with comes from here,
//! so a theme change is a single remap of semantic roles rather than a hunt for
//! hard-coded values.

use egui::{Color32, FontFamily, FontId, Rounding, Stroke, TextStyle, Visuals};
use libonui_core::tokens::{Rgba, Tokens};
use std::collections::BTreeMap;

/// Convert a token colour into an egui colour.
pub fn c(rgba: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(rgba.r, rgba.g, rgba.b, rgba.a)
}

/// A resolved palette plus egui `Visuals`, rebuilt whenever the theme changes.
#[derive(Clone)]
pub struct Theme {
    pub tokens: Tokens,
    pub visuals: Visuals,
}

impl Theme {
    pub fn from_tokens(tokens: Tokens) -> Self {
        let visuals = build_visuals(&tokens);
        Self { tokens, visuals }
    }

    /// A translucent surface for the floating dock and panels, derived from the
    /// palette rather than a fixed grey.
    pub fn glass(&self, alpha: u8) -> Color32 {
        c(self.tokens.surface_raised).gamma_multiply(alpha as f32 / 255.0)
    }

    pub fn accent(&self) -> Color32 {
        c(self.tokens.accent)
    }

    pub fn on_accent(&self) -> Color32 {
        c(self.tokens.on_accent)
    }

    pub fn text(&self) -> Color32 {
        c(self.tokens.text)
    }

    pub fn text_secondary(&self) -> Color32 {
        c(self.tokens.text_secondary)
    }

    pub fn text_muted(&self) -> Color32 {
        c(self.tokens.text_muted)
    }

    pub fn success(&self) -> Color32 {
        c(self.tokens.success)
    }

    pub fn warning(&self) -> Color32 {
        c(self.tokens.warning)
    }

    pub fn danger(&self) -> Color32 {
        c(self.tokens.danger)
    }

    pub fn info(&self) -> Color32 {
        c(self.tokens.info)
    }

    pub fn accent_alt(&self) -> Color32 {
        c(self.tokens.accent_alt)
    }

    pub fn surface(&self) -> Color32 {
        c(self.tokens.surface)
    }

    pub fn surface_raised(&self) -> Color32 {
        c(self.tokens.surface_raised)
    }

    pub fn border(&self) -> Color32 {
        c(self.tokens.border)
    }

    pub fn border_strong(&self) -> Color32 {
        c(self.tokens.border_strong)
    }
}

fn build_visuals(t: &Tokens) -> Visuals {
    let mut v = if t.canvas.contrast(t.text) > 3.0 && t.canvas.luminance() < 0.3 {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    let accent = c(t.accent);
    let surface = c(t.surface);
    let raised = c(t.surface_raised);
    let overlay = c(t.surface_overlay);
    let border = c(t.border);
    let border_strong = c(t.border_strong);
    let text = c(t.text);

    v.override_text_color = Some(text);
    v.panel_fill = c(t.canvas);
    v.window_fill = surface;
    v.extreme_bg_color = c(t.canvas);
    v.faint_bg_color = raised;
    v.window_stroke = Stroke::new(1.0_f32, border);
    v.window_rounding = Rounding::same(t.radius.lg);
    v.menu_rounding = Rounding::same(t.radius.md);
    v.hyperlink_color = accent;
    v.warn_fg_color = c(t.warning);
    v.error_fg_color = c(t.danger);
    v.selection.bg_fill = c(t.selection);
    v.selection.stroke = Stroke::new(1.0_f32, text);

    // The shell's own chrome: scrollbars, the caret and the hyperlink underline
    // are themed from the palette so the parts we did not draw still match.
    v.text_cursor.stroke = Stroke::new(2.0_f32, accent);
    v.hyperlink_color = accent;
    v.widgets.hovered.fg_stroke.color = text;
    v.clip_rect_margin = 2.0;
    v.striped = false;

    // Soft, offset shadows — never a zero-offset halo.
    v.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 10.0),
        blur: 34.0,
        spread: 0.0,
        color: Color32::from_black_alpha(140),
    };
    v.popup_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 6.0),
        blur: 20.0,
        spread: 0.0,
        color: Color32::from_black_alpha(120),
    };

    let widgets = &mut v.widgets;
    widgets.noninteractive.bg_fill = surface;
    widgets.noninteractive.weak_bg_fill = surface;
    widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, border);
    widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, text);
    widgets.noninteractive.rounding = Rounding::same(t.radius.md);

    widgets.inactive.bg_fill = raised;
    widgets.inactive.weak_bg_fill = raised;
    widgets.inactive.bg_stroke = Stroke::new(1.0_f32, border);
    widgets.inactive.fg_stroke = Stroke::new(1.0_f32, text);
    widgets.inactive.rounding = Rounding::same(t.radius.md);

    widgets.hovered.bg_fill = overlay;
    widgets.hovered.weak_bg_fill = overlay;
    widgets.hovered.bg_stroke = Stroke::new(1.0_f32, border_strong);
    widgets.hovered.fg_stroke = Stroke::new(1.0_f32, text);
    widgets.hovered.rounding = Rounding::same(t.radius.md);

    widgets.active.bg_fill = overlay;
    widgets.active.weak_bg_fill = overlay;
    widgets.active.bg_stroke = Stroke::new(1.0_f32, accent);
    widgets.active.fg_stroke = Stroke::new(1.0_f32, text);
    widgets.active.rounding = Rounding::same(t.radius.md);

    widgets.open.bg_fill = overlay;
    widgets.open.weak_bg_fill = overlay;
    widgets.open.bg_stroke = Stroke::new(1.0_f32, border_strong);
    widgets.open.fg_stroke = Stroke::new(1.0_f32, text);
    widgets.open.rounding = Rounding::same(t.radius.md);

    v
}

/// Install the type scale and spacing rhythm on the context's style.
pub fn install_style(ctx: &egui::Context, theme: &Theme) {
    let ts = theme.tokens.type_scale;
    let space = theme.tokens.space;
    let mut style = (*ctx.style()).clone();

    let mut text_styles = BTreeMap::new();
    text_styles.insert(
        TextStyle::Heading,
        FontId::new(ts.title, FontFamily::Proportional),
    );
    text_styles.insert(
        TextStyle::Body,
        FontId::new(ts.body, FontFamily::Proportional),
    );
    text_styles.insert(
        TextStyle::Button,
        FontId::new(ts.label, FontFamily::Proportional),
    );
    text_styles.insert(
        TextStyle::Small,
        FontId::new(ts.caption, FontFamily::Proportional),
    );
    text_styles.insert(
        TextStyle::Monospace,
        FontId::new(ts.body - 0.5, FontFamily::Monospace),
    );
    style.text_styles = text_styles;

    style.spacing.item_spacing = egui::vec2(space.sm, space.sm);
    style.spacing.button_padding = egui::vec2(space.md, space.sm);
    style.spacing.interact_size.y = 30.0;
    style.spacing.scroll.bar_width = 8.0;
    style.visuals = theme.visuals.clone();

    ctx.set_style(style);
}

/// Named font sizes, so screens read from the scale rather than guessing.
pub struct Fonts {
    pub display: FontId,
    pub title: FontId,
    pub heading: FontId,
    pub body: FontId,
    pub label: FontId,
    pub caption: FontId,
    pub mono: FontId,
}

impl Fonts {
    pub fn display(&self) -> FontId {
        self.display.clone()
    }
    pub fn title(&self) -> FontId {
        self.title.clone()
    }
    pub fn heading(&self) -> FontId {
        self.heading.clone()
    }
    pub fn body(&self) -> FontId {
        self.body.clone()
    }
    pub fn label(&self) -> FontId {
        self.label.clone()
    }
    pub fn caption(&self) -> FontId {
        self.caption.clone()
    }
    pub fn mono(&self) -> FontId {
        self.mono.clone()
    }

    pub fn from(t: &Tokens) -> Self {
        let ts = t.type_scale;
        Self {
            display: FontId::new(ts.display, FontFamily::Proportional),
            title: FontId::new(ts.title, FontFamily::Proportional),
            heading: FontId::new(ts.heading, FontFamily::Proportional),
            body: FontId::new(ts.body, FontFamily::Proportional),
            label: FontId::new(ts.label, FontFamily::Proportional),
            caption: FontId::new(ts.caption, FontFamily::Proportional),
            mono: FontId::new(ts.body - 0.5, FontFamily::Monospace),
        }
    }
}

/// Rounding helper for the tokens' radius values.
pub fn corner(px: f32) -> Rounding {
    Rounding::same(px)
}
