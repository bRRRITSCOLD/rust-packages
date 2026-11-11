//! Popover: core-react's `popover.tsx` on `gpui-base`'s unstyled [`gpui_base::Popover`], which
//! owns the trigger's interaction, the open state, outside-press and Escape dismissal, and focus
//! moving in and back out.
//!
//! [`PopoverContent`] is the surface and renders on its own; [`Popover`] anchors it under its
//! trigger, centred, 4px away (core-react's `side="bottom"`, `align="center"`,
//! `sideOffset={4}`), flipping above when there's no room below.
//!
//! Not mirrored: the open and close animations, and the `side` / `align` props, which no Penpot
//! variant property names (the popover always opens below).

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, shadow, space};
use gpui::{
    Anchor, AnyElement, App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};
use gpui_base::StyledExt as _;

use super::button::Button;
use crate::theme::ActiveTheme as _;

/// A popover: a trigger button that toggles [`PopoverContent`] below it.
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Button, ButtonVariant, Popover, PopoverContent, PopoverDescription, PopoverHeader,
///     PopoverTitle,
/// };
/// use gpui::ParentElement as _;
///
/// let dimensions = Popover::new("dimensions")
///     .trigger(Button::new("open").variant(ButtonVariant::Outline).label("Dimensions"))
///     .content(
///         PopoverContent::new()
///             .header(
///                 PopoverHeader::new()
///                     .title(PopoverTitle::new("Dimensions"))
///                     .description(PopoverDescription::new("Set the dimensions for the layer.")),
///             )
///             .child("…"),
///     );
/// ```
#[derive(IntoElement)]
pub struct Popover {
    id: ElementId,
    trigger: Option<Button>,
    content: Option<PopoverContent>,
    default_open: bool,
}

impl Popover {
    /// A closed popover. `id` keys its open state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: None,
            default_open: false,
        }
    }

    /// The button that toggles it (core-react's `PopoverTrigger`).
    pub fn trigger(mut self, trigger: Button) -> Self {
        self.trigger = Some(trigger);
        self
    }

    /// The surface shown while open.
    pub fn content(mut self, content: PopoverContent) -> Self {
        self.content = Some(content);
        self
    }

    /// Starts open.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }
}

/// `sideOffset={4}`: the gap between the trigger and the surface.
const SIDE_OFFSET: gpui::Pixels = space::S1;

impl RenderOnce for Popover {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let trigger = self.trigger;
        let content = self.content;
        gpui_base::Popover::new(self.id)
            .default_open(self.default_open)
            .anchor(Anchor::TopCenter)
            .offset(SIDE_OFFSET)
            .when_some(trigger, |p, trigger| {
                p.trigger_with(move |_, _, _| trigger.into_any_element())
            })
            .when_some(content, |p, content| p.content(move |_, _, _| content))
    }
}

/// The popover's surface (core-react's `PopoverContent`): a header and open content (its
/// Penpot `Body` slot), 288px wide.
#[derive(IntoElement)]
pub struct PopoverContent {
    header: Option<PopoverHeader>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for PopoverContent {
    fn default() -> Self {
        Self::new()
    }
}

impl PopoverContent {
    /// An empty surface.
    pub fn new() -> Self {
        Self {
            header: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// The `Header` part.
    pub fn header(mut self, header: PopoverHeader) -> Self {
        self.header = Some(header);
        self
    }
}

impl RenderOnce for PopoverContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // flex w-72 flex-col gap-4 rounded-md bg-popover p-4 text-sm text-popover-foreground
        // shadow-md ring-1 ring-foreground/10
        let mut shadows = vec![super::ring(1., s.foreground.opacity(0.1))];
        shadows.extend_from_slice(shadow::MD);
        div()
            .w(space::S72)
            .flex()
            .flex_col()
            .gap(space::S4)
            .rounded(s.radius_md)
            .bg(s.popover)
            .p(space::S4)
            .text_size(font_size::SM)
            .text_color(s.popover_foreground)
            .shadow(shadows)
            .children(self.header)
            .children(self.children)
            .refine_style(&self.style)
    }
}

/// The popover's `Header` part: its title over its description.
#[derive(IntoElement)]
pub struct PopoverHeader {
    title: Option<PopoverTitle>,
    description: Option<PopoverDescription>,
    style: StyleRefinement,
}

impl Default for PopoverHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl PopoverHeader {
    /// An empty header.
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            style: StyleRefinement::default(),
        }
    }

    /// The `Title` part.
    pub fn title(mut self, title: PopoverTitle) -> Self {
        self.title = Some(title);
        self
    }

    /// The `Description` part, under the title.
    pub fn description(mut self, description: PopoverDescription) -> Self {
        self.description = Some(description);
        self
    }
}

impl RenderOnce for PopoverHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex flex-col gap-1 text-sm
        div()
            .flex()
            .flex_col()
            .gap(space::S1)
            .text_size(font_size::SM)
            .children(self.title)
            .children(self.description)
            .refine_style(&self.style)
    }
}

/// The popover's `Title` part.
#[derive(IntoElement)]
pub struct PopoverTitle {
    text: SharedString,
    style: StyleRefinement,
}

impl PopoverTitle {
    /// A title reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for PopoverTitle {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // font-medium
        div()
            .font_weight(font_weight::MEDIUM)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The popover's `Description` part.
#[derive(IntoElement)]
pub struct PopoverDescription {
    text: SharedString,
    style: StyleRefinement,
}

impl PopoverDescription {
    /// A description reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for PopoverDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // text-muted-foreground
        div()
            .text_color(cx.theme().semantic().muted_foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

impl_styled!(
    PopoverContent,
    PopoverHeader,
    PopoverTitle,
    PopoverDescription
);
impl_parent!(PopoverContent);

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{Modifiers, TestAppContext, VisualTestContext, px};

    use super::*;
    use crate::components::test_support::render;

    /// Room left of and above the trigger, so the surface opens clear of the window's edges.
    const INSET: gpui::Pixels = px(300.);

    /// A popover whose content is one button counting its clicks, inset from the corner.
    fn popover(clicks: Rc<Cell<usize>>) -> impl Fn() -> gpui::Div {
        move || {
            let clicks = clicks.clone();
            div().flex().pl(INSET).pt(INSET).child(
                Popover::new("popover")
                    .trigger(Button::new("open").label("Dimensions"))
                    .content(
                        PopoverContent::new().child(
                            Button::new("inside")
                                .label("Inside")
                                .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                        ),
                    ),
            )
        }
    }

    /// Renders the inset popover and returns its trigger's bounds.
    fn trigger(
        cx: &mut TestAppContext,
        clicks: Rc<Cell<usize>>,
    ) -> (&mut VisualTestContext, gpui::Bounds<gpui::Pixels>) {
        cx.update(crate::init);
        let (cx, page) = render(cx, popover(clicks));
        let origin = page.origin + gpui::point(INSET, INSET);
        let size = gpui::size(page.size.width - INSET, page.size.height - INSET);
        (cx, gpui::Bounds::new(origin, size))
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
    }

    /// The inside button: the 288px surface is centred under the trigger, 4px down, its button
    /// 16px into its padding.
    fn inside(trigger: gpui::Bounds<gpui::Pixels>) -> gpui::Point<gpui::Pixels> {
        let left = trigger.center().x - px(144.);
        gpui::point(left + px(30.), trigger.bottom() + px(4. + 16. + 10.))
    }

    mod popover {
        use super::*;

        #[gpui::test]
        fn should_open_below_its_trigger_and_close_on_escape(cx: &mut TestAppContext) {
            let clicks = Rc::new(Cell::new(0));
            let (cx, trigger) = trigger(cx, clicks.clone());
            cx.simulate_click(trigger.center(), Modifiers::default());
            draw(cx);
            cx.simulate_click(inside(trigger), Modifiers::default());
            draw(cx);
            assert_eq!(clicks.get(), 1, "the content is open and takes clicks");
            cx.simulate_keystrokes("escape");
            draw(cx);
            cx.simulate_click(inside(trigger), Modifiers::default());
            draw(cx);
            assert_eq!(clicks.get(), 1, "the content closed");
        }

        #[gpui::test]
        fn should_close_on_a_press_outside(cx: &mut TestAppContext) {
            let clicks = Rc::new(Cell::new(0));
            let (cx, trigger) = trigger(cx, clicks.clone());
            cx.simulate_click(trigger.center(), Modifiers::default());
            draw(cx);
            cx.simulate_click(gpui::point(px(5.), px(5.)), Modifiers::default());
            draw(cx);
            cx.simulate_click(inside(trigger), Modifiers::default());
            draw(cx);
            assert_eq!(clicks.get(), 0);
        }
    }
}
