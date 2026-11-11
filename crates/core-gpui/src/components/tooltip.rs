//! Tooltip: core-react's `tooltip.tsx`. Its popup is `gpui-base`'s unstyled
//! [`gpui_base::Tooltip`] (the tooltip role), anchored by `gpui-base`'s [`gpui_base::Popup`].
//!
//! [`TooltipContent`] is the surface, arrow included, and renders on its own; [`Tooltip`] shows
//! it beside its trigger, 4px away, while the pointer is over the trigger. core-react's
//! `TooltipProvider` sets no delay (`delay = 0`), so neither does this.
//!
//! The arrow is a rotated square in core-react (`size-2.5 rotate-45`, half under the surface);
//! a `Div` can't rotate, so the part that shows, a triangle as deep as half the square's
//! diagonal less its offset, is painted as a path. Not mirrored: the arrow's rounded tip
//! (`rounded-xs`), the open and close animations, and the `kbd` styles.

use brrritscold_core_gpui_tokens::primitive::{font_size, space};
use gpui::{
    Anchor, AnyElement, App, Bounds, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, PathBuilder, Pixels, RenderOnce, SharedString,
    StatefulInteractiveElement as _, StyleRefinement, Styled, Window, canvas, div, point,
    prelude::FluentBuilder as _, px,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use crate::theme::ActiveTheme as _;

/// Penpot variant property `side` of `TooltipContent`: the side of its trigger it shows on.
/// Cases are its values in PascalCase; [`Display`] gives the Penpot value back.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum TooltipContentSide {
    /// `top`, core-react's default.
    #[default]
    Top,
    /// `right`
    Right,
    /// `bottom`
    Bottom,
    /// `left`
    Left,
}

impl TooltipContentSide {
    /// The popup's point that meets its trigger.
    fn anchor(self) -> Anchor {
        match self {
            TooltipContentSide::Top => Anchor::BottomCenter,
            TooltipContentSide::Right => Anchor::LeftCenter,
            TooltipContentSide::Bottom => Anchor::TopCenter,
            TooltipContentSide::Left => Anchor::RightCenter,
        }
    }

    /// How far the arrow reaches past the surface: the 10px square's half-diagonal (7.07px)
    /// less how far its centre sits inside the edge, 2px (`-bottom-2` / `top-0.5`, then
    /// `-translate-y-1/2`) above and below, 1px (`-left-1` / `-right-1`) at the sides.
    fn arrow_depth(self) -> Pixels {
        match self {
            TooltipContentSide::Top | TooltipContentSide::Bottom => px(5.07),
            TooltipContentSide::Right | TooltipContentSide::Left => px(6.07),
        }
    }
}

/// A tooltip: shows [`TooltipContent`] beside its trigger while the pointer is over it.
///
/// ```no_run
/// use brrritscold_core_gpui::{Button, ButtonSize, ButtonVariant, IconName, Tooltip, TooltipContent};
///
/// let add = Tooltip::new("add")
///     .trigger(Button::new("add").variant(ButtonVariant::Outline).size(ButtonSize::Icon).icon(IconName::Plus))
///     .content(TooltipContent::new("Add to library"));
/// ```
#[derive(IntoElement)]
pub struct Tooltip {
    id: ElementId,
    trigger: Option<AnyElement>,
    content: Option<TooltipContent>,
}

impl Tooltip {
    /// A tooltip with no trigger yet. `id` keys its hover state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: None,
        }
    }

    /// The element it describes (core-react's `TooltipTrigger`).
    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
        self
    }

    /// The surface shown while the trigger is hovered.
    pub fn content(mut self, content: TooltipContent) -> Self {
        self.content = Some(content);
        self
    }
}

/// `sideOffset={4}`: the gap between the trigger and the surface.
const SIDE_OFFSET: Pixels = space::S1;

impl RenderOnce for Tooltip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let hovered = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        let open = *hovered.read(cx);
        let side = self.content.as_ref().map(|c| c.side).unwrap_or_default();
        let popup = gpui_base::Popup::new(self.id, div().children(self.trigger))
            .anchor(side.anchor())
            .offset(SIDE_OFFSET)
            .on_hover(move |is_hovered, _, cx| {
                hovered.update(cx, |hovered, cx| {
                    *hovered = *is_hovered;
                    cx.notify();
                })
            });
        popup.when_some(self.content.filter(|_| open), |popup, content| {
            popup.content(gpui_base::Tooltip::new("tooltip").child(content))
        })
    }
}

/// The tooltip's surface (core-react's `TooltipContent`): its text on the foreground colour,
/// with an arrow pointing at the trigger on its `side`.
#[derive(IntoElement)]
pub struct TooltipContent {
    side: TooltipContentSide,
    text: SharedString,
    style: StyleRefinement,
}

impl TooltipContent {
    /// A `top` tooltip reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            side: TooltipContentSide::default(),
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `side`.
    pub fn side(mut self, side: TooltipContentSide) -> Self {
        self.side = side;
        self
    }
}

impl RenderOnce for TooltipContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let side = self.side;
        let fill = s.foreground;
        // inline-flex w-fit max-w-xs items-center gap-1.5 rounded-md bg-foreground px-3 py-1.5
        // text-xs text-background
        div()
            // core-react's `data-slot`, for tests to find it by.
            .debug_selector(|| "tooltip-content".into())
            .relative()
            .flex()
            .max_w(brrritscold_core_gpui_tokens::primitive::size::XS)
            .items_center()
            .gap(space::S1_5)
            .rounded(s.radius_md)
            .bg(fill)
            .px(space::S3)
            .py(space::S1_5)
            .text_size(font_size::XS)
            .text_color(s.background)
            .child(self.text)
            .child(
                canvas(
                    |_, _, _| {},
                    move |bounds, _, window, _| paint_arrow(bounds, side, fill, window),
                )
                .absolute()
                .inset_0(),
            )
            .refine_style(&self.style)
    }
}

/// The arrow: a triangle on the middle of the surface's edge facing its trigger (the edge
/// opposite `side`), pointing at the trigger.
fn paint_arrow(bounds: Bounds<Pixels>, side: TooltipContentSide, fill: Hsla, window: &mut Window) {
    let depth = side.arrow_depth();
    // The base sinks 1px under the surface, so no seam shows between them.
    let overlap = px(1.);
    let half = depth + overlap;
    let center = bounds.center();
    let [a, b, tip] = match side {
        TooltipContentSide::Top => {
            let y = bounds.bottom();
            [
                point(center.x - half, y - overlap),
                point(center.x + half, y - overlap),
                point(center.x, y + depth),
            ]
        }
        TooltipContentSide::Bottom => {
            let y = bounds.top();
            [
                point(center.x - half, y + overlap),
                point(center.x + half, y + overlap),
                point(center.x, y - depth),
            ]
        }
        TooltipContentSide::Right => {
            let x = bounds.left();
            [
                point(x + overlap, center.y - half),
                point(x + overlap, center.y + half),
                point(x - depth, center.y),
            ]
        }
        TooltipContentSide::Left => {
            let x = bounds.right();
            [
                point(x - overlap, center.y - half),
                point(x - overlap, center.y + half),
                point(x + depth, center.y),
            ]
        }
    };
    let mut path = PathBuilder::fill();
    path.move_to(a);
    path.line_to(b);
    path.line_to(tip);
    path.close();
    if let Ok(path) = path.build() {
        window.paint_path(path, fill);
    }
}

impl_styled!(TooltipContent);

#[cfg(test)]
mod tests {
    use gpui::{Modifiers, TestAppContext, VisualTestContext, size};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::render;

    mod side {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&str> = TooltipContentSide::iter().map(Into::into).collect();
            assert_eq!(values, ["top", "right", "bottom", "left"]);
        }
    }

    mod tooltip {
        use super::*;

        fn showing(cx: &mut VisualTestContext) -> bool {
            cx.debug_bounds("tooltip-content").is_some()
        }

        fn draw(cx: &mut VisualTestContext) {
            cx.run_until_parked();
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.run_until_parked();
            cx.update(|window, cx| window.draw(cx).clear(cx));
        }

        #[gpui::test]
        fn should_show_while_its_trigger_is_hovered(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (cx, trigger) = render(cx, || {
                div().pl(px(200.)).pt(px(200.)).child(
                    Tooltip::new("add")
                        .trigger(div().size(px(36.)))
                        .content(TooltipContent::new("Add to library")),
                )
            });
            let trigger = Bounds::new(
                trigger.origin + point(px(200.), px(200.)),
                size(px(36.), px(36.)),
            );
            assert!(!showing(cx));
            cx.simulate_mouse_move(trigger.center(), None, Modifiers::default());
            draw(cx);
            assert!(showing(cx));
            cx.simulate_mouse_move(point(px(2.), px(2.)), None, Modifiers::default());
            draw(cx);
            assert!(!showing(cx));
        }
    }
}
