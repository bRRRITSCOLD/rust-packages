//! The outline core-react's form controls share: Input, Textarea, Checkbox and
//! RadioGroupItem all carry the same `border-input`, `focus-visible:` and `aria-invalid:`
//! classes, so their border and rings resolve here once.
//!
//! GPUI paints a box shadow under the whole box, where CSS paints it only outside the border.
//! A control with a ring or `shadow-xs` and a transparent or translucent fill would show the
//! shadow through, so its fill and border are made opaque with [`on_background`].

use brrritscold_core_gpui_tokens::primitive::{opacity, shadow};
use gpui::{BoxShadow, Hsla};

use crate::theme::Theme;

/// A control's border and rings, resolved for the active mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Outline {
    /// `border-input`, or the invalid arm's border.
    pub(crate) border: Hsla,
    /// The ring an invalid control shows at rest (`aria-invalid:ring-3`).
    pub(crate) ring: Option<Hsla>,
    /// The border and ring focus adds (`focus-visible:border-ring focus-visible:ring-3
    /// focus-visible:ring-ring/50`). `None` while invalid: the `aria-invalid:` arms come later
    /// in core-react's stylesheet and win.
    pub(crate) focus: Option<(Hsla, Hsla)>,
}

impl Outline {
    /// The outline of a control that is, or isn't, `aria-invalid`.
    pub(crate) fn new(theme: &Theme, invalid: bool) -> Self {
        let s = theme.semantic();
        let m = theme.mode();
        if invalid {
            Self {
                // aria-invalid:border-destructive dark:aria-invalid:border-destructive/50
                border: on_background(theme, m.pick(s.destructive, s.destructive.opacity(0.5))),
                // aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40
                ring: Some(s.destructive.opacity(m.pick(0.2, 0.4))),
                focus: None,
            }
        } else {
            Self {
                border: s.input,
                ring: None,
                focus: Some((s.ring, theme.focus_ring())),
            }
        }
    }
}

/// A transparent or translucent fill as it shows on `background`, the surface core-react's
/// stories and Penpot's library put a control on. Opaque, so no shadow shows through it.
pub(crate) fn on_background(theme: &Theme, fill: Hsla) -> Hsla {
    theme.semantic().background.blend(fill)
}

/// `disabled:opacity-50` for one layer of a control drawn in stacked layers: an opaque `color`
/// as it shows at 50% on `background`.
///
/// GPUI applies an element's opacity to each layer it paints, so a dimmed layer shows the
/// dimmed layer under it through (a checked box's border over its fill, a switch's thumb over
/// its track), where CSS dims the finished control as one. A disabled Checkbox, Switch or
/// RadioGroupItem dims each layer's colour with this instead of setting an opacity.
pub(crate) fn dimmed(theme: &Theme, color: Hsla) -> Hsla {
    theme
        .semantic()
        .background
        .blend(color.opacity(opacity::O50))
}

/// Shadows at half strength, for a control [`dimmed`] layer by layer.
pub(crate) fn dimmed_shadows(shadows: Vec<BoxShadow>) -> Vec<BoxShadow> {
    shadows
        .into_iter()
        .map(|shadow| BoxShadow {
            color: shadow.color.opacity(opacity::O50),
            ..shadow
        })
        .collect()
}

/// `ring-3` in `ring` over `shadow-xs`, as CSS stacks them; either may be absent.
pub(crate) fn shadows(ring: Option<Hsla>, shadow_xs: bool) -> Vec<BoxShadow> {
    let mut shadows: Vec<BoxShadow> = ring.map(|ring| super::ring(3., ring)).into_iter().collect();
    if shadow_xs {
        shadows.extend_from_slice(shadow::XS);
    }
    shadows
}

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;

    use super::*;
    use crate::components::test_support::theme;
    use crate::theme::Mode;

    mod outline {
        use super::*;

        #[gpui::test]
        fn should_ring_focus_with_the_focus_ring_when_valid(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Light);
            let outline = Outline::new(&theme, false);
            assert_eq!(outline.border, theme.semantic().input);
            assert_eq!(outline.ring, None);
            assert_eq!(
                outline.focus,
                Some((theme.semantic().ring, theme.focus_ring()))
            );
        }

        #[gpui::test]
        fn should_ring_an_invalid_control_at_rest_and_ignore_focus(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let light = Outline::new(&theme(cx, Mode::Light), true);
            let dark = Outline::new(&theme(cx, Mode::Dark), true);
            let destructive = |mode: Mode| mode.semantic().destructive;
            assert_eq!(light.border, destructive(Mode::Light));
            assert_eq!(light.ring, Some(destructive(Mode::Light).opacity(0.2)));
            assert_eq!(light.focus, None);
            assert_eq!(
                dark.border,
                Mode::Dark
                    .semantic()
                    .background
                    .blend(destructive(Mode::Dark).opacity(0.5))
            );
            assert_eq!(dark.ring, Some(destructive(Mode::Dark).opacity(0.4)));
        }
    }
}
