//! The LibonOS design system, expressed as data so every surface — shell,
//! installer, terminal, and the CLI — reads the same values.
//!
//! Palette: a deep violet-black canvas with a single violet-to-cyan accent
//! pair. Colour carries meaning here (action, focus, status), never decoration.

/// A colour in sRGB with straight alpha, kept independent from any UI toolkit
/// so the design tokens stay testable and portable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Linear interpolation in sRGB space, `t` clamped to `0.0..=1.0`.
    pub fn mix(self, other: Rgba, t: f32) -> Rgba {
        let t = t.clamp(0.0, 1.0);
        let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
        Rgba {
            r: lerp(self.r, other.r),
            g: lerp(self.g, other.g),
            b: lerp(self.b, other.b),
            a: lerp(self.a, other.a),
        }
    }

    pub fn to_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Relative luminance per WCAG 2.1.
    pub fn luminance(self) -> f32 {
        fn channel(c: u8) -> f32 {
            let c = c as f32 / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(self.r) + 0.7152 * channel(self.g) + 0.0722 * channel(self.b)
    }

    /// WCAG contrast ratio against another colour, in `1.0..=21.0`.
    pub fn contrast(self, other: Rgba) -> f32 {
        let (a, b) = (self.luminance(), other.luminance());
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }
}

/// Typography scale. Sizes are logical pixels at a 1.0 scale factor.
#[derive(Clone, Copy, Debug)]
pub struct TypeScale {
    pub display: f32,
    pub title: f32,
    pub heading: f32,
    pub body: f32,
    pub label: f32,
    pub caption: f32,
}

impl TypeScale {
    pub const fn desktop() -> Self {
        Self {
            display: 34.0,
            title: 24.0,
            heading: 18.0,
            body: 14.5,
            label: 13.0,
            caption: 11.5,
        }
    }
}

/// Spacing rhythm — a 4px base grid. Tight groups, generous separation.
#[derive(Clone, Copy, Debug)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl Spacing {
    pub const fn grid() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 18.0,
            xl: 28.0,
            xxl: 44.0,
        }
    }
}

/// Corner radii. Cards stay 12–16px; pills only for small controls.
#[derive(Clone, Copy, Debug)]
pub struct Radii {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub pill: f32,
}

impl Radii {
    pub const fn desktop() -> Self {
        Self {
            sm: 6.0,
            md: 10.0,
            lg: 14.0,
            pill: 999.0,
        }
    }
}

/// The complete token set for one theme.
#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    // Surfaces, from the canvas up to the most elevated layer.
    pub canvas: Rgba,
    pub surface: Rgba,
    pub surface_raised: Rgba,
    pub surface_overlay: Rgba,
    pub border: Rgba,
    pub border_strong: Rgba,
    // Text.
    pub text: Rgba,
    pub text_secondary: Rgba,
    pub text_muted: Rgba,
    // Action and focus.
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub accent_soft: Rgba,
    pub on_accent: Rgba,
    pub accent_alt: Rgba,
    pub focus_ring: Rgba,
    // Status.
    pub success: Rgba,
    pub warning: Rgba,
    pub danger: Rgba,
    pub info: Rgba,
    // Selection and the shell chrome.
    pub selection: Rgba,
    pub wallpaper_top: Rgba,
    pub wallpaper_bottom: Rgba,
    // Structure.
    pub type_scale: TypeScale,
    pub space: Spacing,
    pub radius: Radii,
}

impl Tokens {
    /// The default LibonOS look: dark, calm, violet. Chosen for the use scene —
    /// a personal computer used late, in low ambient light — not from category
    /// habit.
    pub const fn dark() -> Self {
        Self {
            canvas: Rgba::new(0x0D, 0x0B, 0x16),
            surface: Rgba::new(0x16, 0x14, 0x22),
            surface_raised: Rgba::new(0x1E, 0x1B, 0x2E),
            surface_overlay: Rgba::new(0x26, 0x22, 0x39),
            border: Rgba::new(0x33, 0x2E, 0x4A),
            border_strong: Rgba::new(0x46, 0x3F, 0x63),
            text: Rgba::new(0xF3, 0xF1, 0xFA),
            text_secondary: Rgba::new(0xBD, 0xB7, 0xD6),
            text_muted: Rgba::new(0x8B, 0x84, 0xA8),
            // Bright enough to carry dark ink, so a filled button keeps AA
            // contrast without a second colour for its label.
            accent: Rgba::new(0xA7, 0x8B, 0xFA),
            accent_hover: Rgba::new(0xB7, 0xA3, 0xFF),
            accent_soft: Rgba::new(0x2A, 0x22, 0x4D),
            on_accent: Rgba::new(0x0D, 0x0B, 0x16),
            accent_alt: Rgba::new(0x5F, 0xD8, 0xEC),
            focus_ring: Rgba::new(0xB7, 0xA3, 0xFF),
            success: Rgba::new(0x4A, 0xD9, 0x9B),
            warning: Rgba::new(0xF2, 0xB9, 0x4B),
            danger: Rgba::new(0xF2, 0x6D, 0x6D),
            info: Rgba::new(0x63, 0xB6, 0xF5),
            selection: Rgba::new(0x8B, 0x5C, 0xF6).with_alpha(0x59),
            wallpaper_top: Rgba::new(0x1A, 0x12, 0x33),
            wallpaper_bottom: Rgba::new(0x08, 0x08, 0x12),
            type_scale: TypeScale::desktop(),
            space: Spacing::grid(),
            radius: Radii::desktop(),
        }
    }

    /// A light counterpart that keeps the same semantic roles so themes swap by
    /// remapping roles rather than inverting the dark palette mechanically.
    pub const fn light() -> Self {
        Self {
            canvas: Rgba::new(0xF6, 0xF5, 0xFB),
            surface: Rgba::new(0xFF, 0xFF, 0xFF),
            surface_raised: Rgba::new(0xFF, 0xFF, 0xFF),
            surface_overlay: Rgba::new(0xF0, 0xEE, 0xF8),
            border: Rgba::new(0xDD, 0xD8, 0xEC),
            border_strong: Rgba::new(0xC3, 0xBB, 0xDB),
            text: Rgba::new(0x1B, 0x17, 0x2B),
            text_secondary: Rgba::new(0x4A, 0x43, 0x63),
            text_muted: Rgba::new(0x65, 0x5E, 0x7E),
            accent: Rgba::new(0x6D, 0x35, 0xE0),
            accent_hover: Rgba::new(0x5B, 0x25, 0xC8),
            accent_soft: Rgba::new(0xEC, 0xE3, 0xFF),
            on_accent: Rgba::new(0xFF, 0xFF, 0xFF),
            accent_alt: Rgba::new(0x0E, 0x7C, 0x92),
            focus_ring: Rgba::new(0x6D, 0x35, 0xE0),
            success: Rgba::new(0x14, 0x7D, 0x52),
            warning: Rgba::new(0x9A, 0x62, 0x00),
            danger: Rgba::new(0xC0, 0x2E, 0x2E),
            info: Rgba::new(0x14, 0x63, 0xA8),
            selection: Rgba::new(0x6D, 0x35, 0xE0).with_alpha(0x40),
            wallpaper_top: Rgba::new(0xE7, 0xE2, 0xFA),
            wallpaper_bottom: Rgba::new(0xCF, 0xD4, 0xF5),
            type_scale: TypeScale::desktop(),
            space: Spacing::grid(),
            radius: Radii::desktop(),
        }
    }

    /// Text colour that stays readable on a filled accent button.
    pub fn on(&self, background: Rgba) -> Rgba {
        if self.text.contrast(background) >= self.canvas.contrast(background) {
            self.text
        } else {
            self.canvas
        }
    }
}

/// The theme a user selected, persisted in settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeChoice {
    Dark,
    Light,
}

impl ThemeChoice {
    pub fn tokens(self) -> Tokens {
        match self {
            ThemeChoice::Dark => Tokens::dark(),
            ThemeChoice::Light => Tokens::light(),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeChoice::Dark => "Scuro",
            ThemeChoice::Light => "Chiaro",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_text_meets_wcag_aa_on_every_surface() {
        for tokens in [Tokens::dark(), Tokens::light()] {
            for surface in [
                tokens.canvas,
                tokens.surface,
                tokens.surface_raised,
                tokens.surface_overlay,
            ] {
                assert!(
                    tokens.text.contrast(surface) >= 4.5,
                    "primary text contrast {:.2} on {surface:?}",
                    tokens.text.contrast(surface)
                );
                assert!(
                    tokens.text_secondary.contrast(surface) >= 4.5,
                    "secondary text contrast {:.2} on {surface:?}",
                    tokens.text_secondary.contrast(surface)
                );
            }
            assert!(tokens.text_muted.contrast(tokens.canvas) >= 3.0);
            assert!(tokens.on_accent.contrast(tokens.accent) >= 4.5);
            assert!(tokens.accent.contrast(tokens.canvas) >= 3.0);
        }
    }

    #[test]
    fn mix_interpolates_and_clamps() {
        let a = Rgba::new(0, 0, 0);
        let b = Rgba::new(255, 255, 255);
        assert_eq!(a.mix(b, 0.0), a);
        assert_eq!(a.mix(b, 1.0), b);
        assert_eq!(a.mix(b, 2.0), b);
        assert_eq!(a.mix(b, 0.5), Rgba::new(128, 128, 128));
    }
}
