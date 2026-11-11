//! Design tokens for core-gpui, generated from core-react's DTCG `tokens.json`.
//!
//! core-react's `tokens.json` is the single token source. `cargo xtask tokens update <version>`
//! vendors it from one exact npm version (the *core-react pin*, recorded in this crate's
//! `Cargo.toml`) and regenerates [`generated.rs`](self). Primitives live in [`primitive`];
//! components read [`Semantic::LIGHT`] or [`Semantic::DARK`] for the active mode.
//!
//! Not tokens, on purpose: the 16px rem root (font sizes are already [`gpui::Rems`]) and
//! core-react's `outline-ring/50` base style (core-gpui's focus ring is `ring` at
//! `primitive::opacity::O50`).

#[rustfmt::skip]
mod generated;

pub use generated::*;

use gpui::FontFallbacks;

/// A CSS font stack as GPUI needs it: one family plus fallbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontStack {
    /// The family GPUI loads first.
    pub family: &'static str,
    /// Families GPUI tries, in order, for glyphs `family` lacks. Missing ones are skipped.
    pub fallbacks: &'static [&'static str],
}

impl FontStack {
    /// The fallbacks as GPUI's [`FontFallbacks`].
    pub fn font_fallbacks(&self) -> FontFallbacks {
        FontFallbacks::from_fonts(self.fallbacks.iter().map(|f| (*f).to_owned()).collect())
    }
}

/// The platform's monospace font, standing in for the CSS `ui-monospace` / `monospace`
/// generics. GPUI has no alias for it, and no mono font is bundled.
#[cfg(target_os = "macos")]
pub const PLATFORM_MONO: &str = "Menlo";
/// The platform's monospace font, standing in for the CSS `ui-monospace` / `monospace`
/// generics. GPUI has no alias for it, and no mono font is bundled.
#[cfg(not(target_os = "macos"))]
pub const PLATFORM_MONO: &str = "DejaVu Sans Mono";

#[cfg(test)]
mod tests {
    use gpui::{Hsla, px, rems, rgb, rgba};

    use super::*;

    mod semantic {
        use super::*;

        #[test]
        fn should_match_gpui_colour_conversion_exactly() {
            assert_eq!(Semantic::LIGHT.background, Hsla::from(rgb(0xffffff)));
            assert_eq!(Semantic::LIGHT.foreground, Hsla::from(rgb(0x0a0a0a)));
            assert_eq!(Semantic::LIGHT.destructive, Hsla::from(rgb(0xb91c1c)));
            assert_eq!(Semantic::DARK.destructive, Hsla::from(rgb(0xf87171)));
        }

        #[test]
        fn should_carry_the_radius_scale() {
            assert_eq!(Semantic::LIGHT.radius, px(10.));
            assert_eq!(Semantic::LIGHT.radius_xs, px(2.));
            assert_eq!(Semantic::DARK.radius_xl, px(14.));
        }

        #[test]
        fn should_differ_between_modes() {
            assert_ne!(Semantic::LIGHT, Semantic::DARK);
        }
    }

    mod primitives {
        use super::*;

        #[test]
        fn should_emit_font_sizes_as_rems() {
            assert_eq!(primitive::font_size::SM, rems(0.875));
        }

        #[test]
        fn should_keep_shadow_alpha() {
            assert_eq!(primitive::shadow::XS.len(), 1);
            assert_eq!(primitive::shadow::XS[0].color, Hsla::from(rgba(0x0000000d)));
            assert_eq!(primitive::shadow::SM.len(), 2);
        }

        #[test]
        fn should_resolve_the_sans_stack_to_inter_then_the_system_ui_font() {
            assert_eq!(primitive::font_family::SANS.family, "Inter");
            assert_eq!(primitive::font_family::SANS.fallbacks, &[".SystemUIFont"]);
        }
    }
}
