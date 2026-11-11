//! Skeleton: core-react's `skeleton.tsx`, a plain styled element. A `muted` placeholder whose
//! size is the caller's, pulsing like Tailwind's `animate-pulse`.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, App, IntoElement, RenderOnce, StyleRefinement, Styled, Window,
    bounce, div, ease_in_out,
};
use gpui_base::StyledExt as _;

use crate::theme::ActiveTheme as _;

/// A loading placeholder, styled like core-react's `Skeleton`. Size it like any element:
///
/// ```no_run
/// use brrritscold_core_gpui::Skeleton;
/// use gpui::{Styled as _, px};
///
/// let line = Skeleton::new().h_4().w(px(288.));
/// let avatar = Skeleton::new().size_12().rounded_full();
/// ```
#[derive(IntoElement)]
pub struct Skeleton {
    style: StyleRefinement,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl Skeleton {
    /// A skeleton with no size of its own.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
        }
    }
}

/// `animate-pulse`: opacity 1 → 0.5 → 1 over 2s. `delta` runs 0 → 1 → 0 (`bounce`).
fn pulse_opacity(delta: f32) -> f32 {
    1. - 0.5 * delta
}

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // animate-pulse rounded-md bg-muted
        div()
            .rounded(s.radius_md)
            .bg(s.muted)
            .refine_style(&self.style)
            .with_animation(
                "skeleton-pulse",
                Animation::new(Duration::from_secs(2))
                    .repeat()
                    .with_easing(bounce(ease_in_out)),
                |skeleton, delta| skeleton.opacity(pulse_opacity(delta)),
            )
    }
}

impl_styled!(Skeleton);

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, px};

    use super::*;
    use crate::components::test_support::render;

    mod skeleton {
        use super::*;

        #[gpui::test]
        fn should_take_the_callers_size(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || Skeleton::new().h(px(16.)).w(px(288.)));
            assert_eq!(bounds.size.width, px(288.));
            assert_eq!(bounds.size.height, px(16.));
        }

        #[test]
        fn should_pulse_from_opaque_to_half_and_back() {
            let easing = bounce(ease_in_out);
            assert_eq!(pulse_opacity(easing(0.)), 1.);
            assert_eq!(pulse_opacity(easing(0.5)), 0.5);
        }
    }
}
