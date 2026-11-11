//! Stand-ins for controls core-gpui doesn't have yet, so an overlay's sample content keeps the
//! story's composition (and height) in the comparison. Each draws core-react's classes for the
//! control at rest; swap it for the real control once core-gpui has it.

use brrritscold_core_gpui::{ActiveTheme as _, Label};
use brrritscold_core_gpui_tokens::primitive::{font_size, line_height, space};
use gpui::{
    AnyElement, App, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _,
    Window, div, relative,
};

/// core-react's `Field` (vertical) holding a `FieldLabel` over an empty `Input`.
#[derive(IntoElement)]
pub(super) struct Field {
    label: SharedString,
    value: Option<SharedString>,
    horizontal: bool,
}

impl Field {
    /// `Field` > `FieldLabel` + `Input`, stacked.
    pub(super) fn new(label: &'static str) -> Self {
        Self {
            label: label.into(),
            value: None,
            horizontal: false,
        }
    }

    /// `orientation="horizontal"`: the label beside the input.
    pub(super) fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }

    /// The input's `defaultValue`.
    pub(super) fn value(mut self, value: &'static str) -> Self {
        self.value = Some(value.into());
        self
    }
}

impl RenderOnce for Field {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let s = theme.semantic();
        // Input: h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-2.5 py-1
        // text-base md:text-sm dark:bg-input/30. Not its shadow-xs: GPUI paints a shadow under
        // the box as well, which shows through a transparent fill.
        let input = div()
            .h(space::S9)
            .w_full()
            .min_w_0()
            .flex()
            .items_center()
            .rounded(s.radius_md)
            .border_1()
            .border_color(s.input)
            .bg(theme
                .mode()
                .pick(gpui::transparent_black(), s.input.opacity(0.3)))
            .px(space::S2_5)
            // text-base: the story's 640px viewport is under `md:`, where text-sm takes over.
            .text_size(font_size::BASE)
            .children(self.value);
        // FieldLabel: Label + leading-snug
        let label = Label::new()
            .line_height(relative(line_height::SNUG))
            .child(self.label);
        // Field: flex w-full gap-3, flex-col (vertical) or flex-row items-center with the
        // label flex-auto (horizontal)
        let field = div().w_full().flex().gap(space::S3);
        if self.horizontal {
            field.items_center().child(label.flex_auto()).child(input)
        } else {
            field.flex_col().child(label).child(input)
        }
    }
}

/// [`Field`] as an [`AnyElement`].
pub(super) fn field(field: Field) -> AnyElement {
    field.into_any_element()
}
