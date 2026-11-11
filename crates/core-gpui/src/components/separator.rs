//! Separator: core-react's `separator.tsx`, a plain styled element. A hairline in `border`,
//! across its container (`horizontal`) or down it (`vertical`).

use gpui::{
    App, IntoElement, RenderOnce, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use crate::theme::ActiveTheme as _;

/// Penpot variant property `orientation`. Cases are its values in PascalCase; [`Display`]
/// gives the Penpot value back (`SeparatorOrientation::Vertical` → `vertical`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SeparatorOrientation {
    /// `horizontal`: 1px tall, the container's full width.
    #[default]
    Horizontal,
    /// `vertical`: 1px wide, stretched to the row's height.
    Vertical,
}

/// A separator, styled like core-react's `Separator`.
///
/// ```no_run
/// use brrritscold_core_gpui::{Separator, SeparatorOrientation};
///
/// let rule = Separator::new();
/// let divider = Separator::new().orientation(SeparatorOrientation::Vertical);
/// ```
#[derive(IntoElement)]
pub struct Separator {
    orientation: SeparatorOrientation,
    style: StyleRefinement,
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl Separator {
    /// A `horizontal` separator.
    pub fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::default(),
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `orientation`.
    pub fn orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.orientation = orientation;
        self
    }
}

impl RenderOnce for Separator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // shrink-0 bg-border
        div()
            .flex_shrink_0()
            .bg(cx.theme().semantic().border)
            .map(|d| match self.orientation {
                // data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full
                SeparatorOrientation::Horizontal => d.h(px(1.)).w_full(),
                // data-[orientation=vertical]:w-px data-[orientation=vertical]:self-stretch
                SeparatorOrientation::Vertical => d.w(px(1.)).self_stretch(),
            })
            .refine_style(&self.style)
    }
}

impl_styled!(Separator);

#[cfg(test)]
mod tests {
    use gpui::{ParentElement as _, TestAppContext};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::render;

    mod orientation {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&'static str> = SeparatorOrientation::iter().map(Into::into).collect();
            assert_eq!(values, ["horizontal", "vertical"]);
        }

        #[gpui::test]
        fn should_run_a_1px_line_across_a_horizontal_container(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || {
                div().flex().flex_col().w(px(256.)).child(Separator::new())
            });
            assert_eq!(bounds.size.width, px(256.));
            assert_eq!(bounds.size.height, px(1.));
        }

        #[gpui::test]
        fn should_stretch_a_vertical_line_down_its_row(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || {
                div()
                    .flex()
                    .items_center()
                    .h(px(40.))
                    .child(Separator::new().orientation(SeparatorOrientation::Vertical))
            });
            assert_eq!(bounds.size.height, px(40.));
        }
    }
}
