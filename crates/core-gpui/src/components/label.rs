//! Label: core-react's `label.tsx`, a plain styled element. It names a control; its children
//! are its text and, beside it, the control or an icon.
//!
//! Not mirrored: core-react dims a label to 50% through `peer-disabled` and
//! `group-data-[disabled=true]`, CSS relations GPUI has no equivalent of, and no Penpot variant
//! property names.

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, space};
use gpui::{
    AnyElement, App, IntoElement, ParentElement as _, RenderOnce, StyleRefinement, Styled, Window,
    div, relative,
};
use gpui_base::StyledExt as _;

/// A label, styled like core-react's `Label`. Its children are its content, the `Text` layer
/// first:
///
/// ```no_run
/// use brrritscold_core_gpui::Label;
/// use gpui::ParentElement as _;
///
/// let email = Label::new().child("Email");
/// ```
#[derive(IntoElement)]
pub struct Label {
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for Label {
    fn default() -> Self {
        Self::new()
    }
}

impl Label {
    /// An empty label.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for Label {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex items-center gap-2 text-sm leading-none font-medium select-none
        div()
            .flex()
            .items_center()
            .gap(space::S2)
            .text_size(font_size::SM)
            .line_height(relative(1.))
            .font_weight(font_weight::MEDIUM)
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl_styled!(Label);
impl_parent!(Label);

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, px};

    use super::*;
    use crate::components::test_support::render;

    mod label {
        use super::*;

        #[gpui::test]
        fn should_set_its_line_at_the_font_size(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || Label::new().child("Email"));
            assert_eq!(bounds.size.height, px(14.));
        }
    }
}
