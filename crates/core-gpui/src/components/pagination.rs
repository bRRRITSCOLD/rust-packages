//! Pagination: core-react's `pagination.tsx`, on core-gpui's [`Button`] as core-react's is on
//! its `Button`. Plain links with no state of their own: the app keeps the current page and
//! rebuilds the links when it changes.
//!
//! Not mirrored: `PaginationItem`, a bare `<li>` with no styles and no Penpot twin (core-react
//! allowlists it); the content's children are its items. Previous and Next always show their
//! text: core-react hides it below the 640px `sm` breakpoint, narrower than a desktop window.

use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StyleRefinement, Styled, Window, div,
};
use gpui_base::StyledExt as _;

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::icons::{Icon, IconName};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// Page navigation, styled like core-react's `Pagination`. It holds a [`PaginationContent`]:
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Pagination, PaginationContent, PaginationLink, PaginationNext, PaginationPrevious,
/// };
/// use gpui::ParentElement as _;
///
/// let pages = Pagination::new().child(
///     PaginationContent::new()
///         .child(PaginationPrevious::new("previous"))
///         .child(PaginationLink::new("page-1").label("1"))
///         .child(PaginationLink::new("page-2").label("2").is_active(true))
///         .child(PaginationNext::new("next")),
/// );
/// ```
#[derive(IntoElement)]
pub struct Pagination {
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new()
    }
}

impl Pagination {
    /// Empty navigation; its child is its `Content` part.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for Pagination {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // mx-auto flex w-full justify-center
        div()
            .mx_auto()
            .flex()
            .w_full()
            .justify_center()
            .children(self.children)
            .refine_style(&self.style)
    }
}

/// The pagination's `Content` part: its children, the links, in a row.
#[derive(IntoElement)]
pub struct PaginationContent {
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for PaginationContent {
    fn default() -> Self {
        Self::new()
    }
}

impl PaginationContent {
    /// An empty row; its children are its items.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for PaginationContent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex items-center gap-1
        div()
            .flex()
            .items_center()
            .gap(space::S1)
            .children(self.children)
            .refine_style(&self.style)
    }
}

/// The pagination's `Link` part: a page number, an `outline` button when it is the current
/// page and a `ghost` one otherwise.
#[derive(IntoElement)]
pub struct PaginationLink {
    id: ElementId,
    label: Option<SharedString>,
    is_active: bool,
    size: ButtonSize,
    disabled: bool,
    on_click: Option<ClickHandler>,
    style: StyleRefinement,
}

impl PaginationLink {
    /// An inactive, `icon`-sized link with no label. `id` keys its focus state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            is_active: false,
            size: ButtonSize::Icon,
            disabled: false,
            on_click: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `isActive`: whether this is the current page.
    pub fn is_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// The `Label` layer: the page number.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The button size; `icon` (a 36px square) unless set.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Runs on a pointer click, Enter or Space: core-react's `href`, as the app navigates.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// `variant={isActive ? 'outline' : 'ghost'}`
    fn variant(&self) -> ButtonVariant {
        if self.is_active {
            ButtonVariant::Outline
        } else {
            ButtonVariant::Ghost
        }
    }

    /// `aria-disabled:pointer-events-none aria-disabled:opacity-50`, as core-react's
    /// `DataTablePagination` fades Previous on the first page and Next on the last. Not a
    /// Penpot property of the link, so it stays inside the crate.
    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn button(self) -> Button {
        let variant = self.variant();
        let mut button = Button::new(self.id)
            .variant(variant)
            .size(self.size)
            .disabled(self.disabled);
        if let Some(label) = self.label {
            button = button.label(label);
        }
        if let Some(on_click) = self.on_click {
            button = button.on_click(move |event, window, cx| on_click(event, window, cx));
        }
        button.refine_style(&self.style)
    }
}

impl RenderOnce for PaginationLink {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.button()
    }
}

/// Declares Previous or Next: a `default`-sized ghost link with a chevron and its text.
macro_rules! step {
    ($(#[$doc:meta])* $step:ident, $text:literal) => {
        $(#[$doc])*
        #[derive(IntoElement)]
        pub struct $step {
            id: ElementId,
            text: SharedString,
            disabled: bool,
            on_click: Option<ClickHandler>,
            style: StyleRefinement,
        }

        impl $step {
            #[doc = concat!("A link reading `", $text, "`. `id` keys its focus state.")]
            pub fn new(id: impl Into<ElementId>) -> Self {
                Self {
                    id: id.into(),
                    text: $text.into(),
                    disabled: false,
                    on_click: None,
                    style: StyleRefinement::default(),
                }
            }

            /// The `Label` layer, replacing the default text.
            pub fn text(mut self, text: impl Into<SharedString>) -> Self {
                self.text = text.into();
                self
            }

            /// Runs on a pointer click, Enter or Space.
            pub fn on_click(
                mut self,
                handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
            ) -> Self {
                self.on_click = Some(Box::new(handler));
                self
            }

            /// Fades the link and makes it inert; see [`PaginationLink::disabled`].
            pub(crate) fn disabled(mut self, disabled: bool) -> Self {
                self.disabled = disabled;
                self
            }

            fn link(&mut self) -> PaginationLink {
                let mut link = PaginationLink::new(self.id.clone())
                    .size(ButtonSize::Default)
                    .disabled(self.disabled);
                link.on_click = self.on_click.take();
                link.style = std::mem::take(&mut self.style);
                link
            }
        }

        impl_styled!($step);
    };
}

step! {
    /// The pagination's `Previous` part: `ChevronLeft`, then its text.
    PaginationPrevious,
    "Previous"
}

step! {
    /// The pagination's `Next` part: its text, then `ChevronRight`.
    PaginationNext,
    "Next"
}

impl RenderOnce for PaginationPrevious {
    fn render(mut self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // size="default" pl-2!, <ChevronLeftIcon data-icon="inline-start" /> {text}: the
        // button's inline-start arm is the same pl-2.
        self.link()
            .button()
            .icon(IconName::ChevronLeft)
            .label(self.text)
    }
}

impl RenderOnce for PaginationNext {
    fn render(mut self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // size="default" pr-2!, {text} <ChevronRightIcon data-icon="inline-end" />: the
        // button's inline-end arm is the same pr-2.
        self.link()
            .button()
            .label(self.text)
            .icon_end(IconName::ChevronRight)
    }
}

/// The pagination's `Ellipsis` part: a gap in the pages, the size of a page link.
#[derive(IntoElement)]
pub struct PaginationEllipsis {
    style: StyleRefinement,
}

impl Default for PaginationEllipsis {
    fn default() -> Self {
        Self::new()
    }
}

impl PaginationEllipsis {
    /// An ellipsis.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for PaginationEllipsis {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex size-9 items-center justify-center [&_svg:not([class*='size-'])]:size-4
        div()
            .flex()
            .size(space::S9)
            .items_center()
            .justify_center()
            .child(Icon::new(IconName::Ellipsis).size(space::S4))
            .refine_style(&self.style)
    }
}

impl_styled!(
    Pagination,
    PaginationContent,
    PaginationLink,
    PaginationEllipsis
);
impl_parent!(Pagination, PaginationContent);

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, px};

    use super::*;
    use crate::components::test_support::render;

    mod link {
        use super::*;

        #[test]
        fn should_be_outline_when_active_and_ghost_otherwise() {
            let link = |active| PaginationLink::new("p").is_active(active).variant();
            assert_eq!(link(true), ButtonVariant::Outline);
            assert_eq!(link(false), ButtonVariant::Ghost);
        }

        #[gpui::test]
        fn should_be_a_36px_square(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || PaginationLink::new("p").label("2"));
            assert_eq!(bounds.size.width, px(36.));
            assert_eq!(bounds.size.height, px(36.));
        }
    }

    mod steps {
        use super::*;

        #[gpui::test]
        fn should_mirror_previous_and_next(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, previous) = render(cx, || PaginationPrevious::new("p").text("Step"));
            let (_, next) = render(cx, || PaginationNext::new("n").text("Step"));
            assert_eq!(previous.size, next.size);
            assert_eq!(previous.size.height, px(36.));
        }
    }

    mod ellipsis {
        use super::*;

        #[gpui::test]
        fn should_be_the_size_of_a_page_link(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, PaginationEllipsis::new);
            assert_eq!(bounds.size.width, px(36.));
        }
    }
}
