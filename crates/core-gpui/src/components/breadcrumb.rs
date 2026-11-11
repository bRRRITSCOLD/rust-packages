//! Breadcrumb: core-react's `breadcrumb.tsx`, plain styled elements; a link is `gpui-base`'s
//! unstyled [`gpui_base::Link`], which owns focus, activation and accessibility.
//!
//! The list's items are its children directly. core-react wraps each in a `BreadcrumbItem`
//! `<li>` (`inline-flex items-center gap-1.5`) for list semantics; GPUI has no list element, and
//! the item has no Penpot twin (core-react allowlists it), so core-gpui has no item part.
//!
//! The list uses core-react's `sm:gap-2.5`: a desktop window is wider than the 640px `sm`
//! breakpoint, and Penpot's list uses the same 10px gap.

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, space};
use gpui::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement as _, IntoElement,
    ParentElement as _, RenderOnce, SharedString, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use gpui_base::StyledExt as _;

use crate::icons::{Icon, IconName};
use crate::theme::ActiveTheme as _;

/// A breadcrumb trail, styled like core-react's `Breadcrumb`. It holds a [`BreadcrumbList`]:
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Breadcrumb, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator,
/// };
/// use gpui::ParentElement as _;
///
/// let trail = Breadcrumb::new().child(
///     BreadcrumbList::new()
///         .child(BreadcrumbLink::new("home").label("Home").on_click(|_, _, _| {}))
///         .child(BreadcrumbSeparator::new())
///         .child(BreadcrumbPage::new("Settings")),
/// );
/// ```
#[derive(IntoElement)]
pub struct Breadcrumb {
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for Breadcrumb {
    fn default() -> Self {
        Self::new()
    }
}

impl Breadcrumb {
    /// An empty trail; its child is its `List` part.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for Breadcrumb {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // <nav aria-label="breadcrumb">, unstyled.
        div().children(self.children).refine_style(&self.style)
    }
}

/// The breadcrumb's `List` part: links, separators and the current page, in a muted row.
#[derive(IntoElement)]
pub struct BreadcrumbList {
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for BreadcrumbList {
    fn default() -> Self {
        Self::new()
    }
}

impl BreadcrumbList {
    /// An empty list; its children are its items.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for BreadcrumbList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // flex flex-wrap items-center gap-1.5 text-sm wrap-break-word text-muted-foreground
        // sm:gap-2.5
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(space::S2_5)
            .text_size(font_size::SM)
            .text_color(cx.theme().semantic().muted_foreground)
            .children(self.children)
            .refine_style(&self.style)
    }
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// The breadcrumb's `Link` part: a muted link that turns `foreground` on hover.
#[derive(IntoElement)]
pub struct BreadcrumbLink {
    id: ElementId,
    label: Option<SharedString>,
    on_click: Option<ClickHandler>,
    style: StyleRefinement,
}

impl BreadcrumbLink {
    /// A link with no label. `id` keys its focus state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            on_click: None,
            style: StyleRefinement::default(),
        }
    }

    /// The `Text` layer: the link's text.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Runs on a pointer click or Enter: core-react's `href`, as the app navigates.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for BreadcrumbLink {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let foreground = cx.theme().semantic().foreground;
        // transition-colors hover:text-foreground
        gpui_base::Link::new(self.id)
            .hover(move |s| s.text_color(foreground))
            .when_some(self.on_click, |link, on_click| {
                link.on_activate(move |event, window, cx| on_click(event, window, cx))
            })
            .when_some(self.label, |link, label| link.child(label))
            .refine_style(&self.style)
    }
}

/// The breadcrumb's `Page` part: the current page, in `foreground`.
#[derive(IntoElement)]
pub struct BreadcrumbPage {
    text: SharedString,
    style: StyleRefinement,
}

impl BreadcrumbPage {
    /// The current page, reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for BreadcrumbPage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // font-normal text-foreground
        div()
            .font_weight(font_weight::NORMAL)
            .text_color(cx.theme().semantic().foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The breadcrumb's `Separator` part: a 14px chevron, or another icon.
#[derive(IntoElement)]
pub struct BreadcrumbSeparator {
    icon: IconName,
    style: StyleRefinement,
}

impl Default for BreadcrumbSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl BreadcrumbSeparator {
    /// A `ChevronRight` separator.
    pub fn new() -> Self {
        Self {
            icon: IconName::ChevronRight,
            style: StyleRefinement::default(),
        }
    }

    /// The `Icon` part, replacing the chevron at the same size.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = icon;
        self
    }
}

impl RenderOnce for BreadcrumbSeparator {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // [&>svg]:size-3.5
        div()
            .flex()
            .child(Icon::new(self.icon).size(space::S3_5))
            .refine_style(&self.style)
    }
}

/// The breadcrumb's `Ellipsis` part: a collapsed run of links.
#[derive(IntoElement)]
pub struct BreadcrumbEllipsis {
    style: StyleRefinement,
}

impl Default for BreadcrumbEllipsis {
    fn default() -> Self {
        Self::new()
    }
}

impl BreadcrumbEllipsis {
    /// An ellipsis.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for BreadcrumbEllipsis {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex size-5 items-center justify-center [&>svg]:size-4
        div()
            .flex()
            .size(space::S5)
            .items_center()
            .justify_center()
            .child(Icon::new(IconName::Ellipsis).size(space::S4))
            .refine_style(&self.style)
    }
}

impl_styled!(
    Breadcrumb,
    BreadcrumbList,
    BreadcrumbLink,
    BreadcrumbPage,
    BreadcrumbSeparator,
    BreadcrumbEllipsis
);
impl_parent!(Breadcrumb, BreadcrumbList);

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{Modifiers, TestAppContext, px};

    use super::*;
    use crate::components::test_support::render;

    mod link {
        use super::*;

        #[gpui::test]
        fn should_run_the_handler_on_click(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let clicks = Rc::new(Cell::new(0));
            let (cx, bounds) = render(cx, {
                let clicks = clicks.clone();
                move || {
                    let clicks = clicks.clone();
                    BreadcrumbLink::new("home")
                        .label("Home")
                        .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                }
            });
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert_eq!(clicks.get(), 1);
        }
    }

    mod list {
        use super::*;

        #[gpui::test]
        fn should_size_the_separator_to_14px_and_the_ellipsis_to_20px(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, separator) = render(cx, BreadcrumbSeparator::new);
            let (_, ellipsis) = render(cx, BreadcrumbEllipsis::new);
            assert_eq!(separator.size.width, px(14.));
            assert_eq!(ellipsis.size.width, px(20.));
        }

        #[gpui::test]
        fn should_set_a_row_on_one_text_sm_line(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || {
                BreadcrumbList::new()
                    .child(BreadcrumbLink::new("home").label("Home"))
                    .child(BreadcrumbSeparator::new())
                    .child(BreadcrumbPage::new("Breadcrumb"))
            });
            assert_eq!(bounds.size.height, px(21.));
        }
    }
}
