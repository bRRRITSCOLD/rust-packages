//! Sheet: core-react's `sheet.tsx` on `gpui-base`'s unstyled [`gpui_base::Sheet`], which owns
//! the focus trap, Escape and backdrop dismissal.
//!
//! [`SheetContent`] is the surface and renders on its own, sized by its `side` (a 3/4-width,
//! full-height panel at `left` / `right`, a full-width one at `top` / `bottom`); [`Sheet`] pins
//! it to that edge of the window over a backdrop while open.
//!
//! Not mirrored: the slide-in and fade transitions, the backdrop blur, and `bg-clip-padding`
//! (the sheet's border is on one edge only and opaque, so the clip draws the same).

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, shadow, size, space};
use gpui::{
    AnyElement, App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StyleRefinement, Styled, Window, deferred, div,
    prelude::FluentBuilder as _, relative,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use super::button::Button;
use super::modal::{self, DialogHandle, ModalState, Overlay};
use crate::theme::ActiveTheme as _;

/// Penpot variant property `side` of `SheetContent`: the window edge it is pinned to. Cases
/// are its values in PascalCase; [`Display`] gives the Penpot value back.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SheetContentSide {
    /// `top`
    Top,
    /// `right`, core-react's default.
    #[default]
    Right,
    /// `bottom`
    Bottom,
    /// `left`
    Left,
}

impl SheetContentSide {
    fn is_horizontal(self) -> bool {
        matches!(self, SheetContentSide::Left | SheetContentSide::Right)
    }
}

/// A modal sheet: a trigger button that opens [`SheetContent`] at an edge of the window.
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Button, ButtonVariant, DialogClose, Sheet, SheetContent, SheetContentSide,
///     SheetDescription, SheetFooter, SheetHeader, SheetTitle,
/// };
/// use gpui::ParentElement as _;
///
/// let sheet = Sheet::new("edit-profile")
///     .trigger(Button::new("open").variant(ButtonVariant::Outline).label("Edit profile"))
///     .content(
///         SheetContent::new()
///             .side(SheetContentSide::Left)
///             .header(SheetHeader::new().title(SheetTitle::new("Edit profile")))
///             .footer(
///                 SheetFooter::new()
///                     .child(Button::new("save").label("Save changes"))
///                     .child(DialogClose::new(
///                         Button::new("cancel").variant(ButtonVariant::Outline).label("Cancel"),
///                     )),
///             ),
///     );
/// ```
#[derive(IntoElement)]
pub struct Sheet {
    id: ElementId,
    trigger: Option<Button>,
    content: Option<SheetContent>,
    handle: Option<DialogHandle>,
    default_open: bool,
}

impl Sheet {
    /// A closed sheet. `id` keys its open state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: None,
            handle: None,
            default_open: false,
        }
    }

    /// The button that opens the sheet (core-react's `SheetTrigger`), after its own
    /// `on_click`.
    pub fn trigger(mut self, trigger: Button) -> Self {
        self.trigger = Some(trigger);
        self
    }

    /// The surface shown while open.
    pub fn content(mut self, content: SheetContent) -> Self {
        self.content = Some(content);
        self
    }

    /// Opens and closes through `handle` instead of its own state.
    pub fn handle(mut self, handle: DialogHandle) -> Self {
        self.handle = Some(handle);
        self
    }

    /// Starts open.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }
}

impl RenderOnce for Sheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = ModalState::use_keyed(&self.id, self.handle, self.default_open, window, cx);
        ModalState::sync_focus(&state, window, cx);
        let (handle, focus) = {
            let state = state.read(cx);
            (state.handle(), state.focus())
        };
        let open = handle.is_open();
        let trigger = self.trigger.map(|t| modal::trigger(t, handle.clone()));
        let surface = self.content.filter(|_| open).map(|content| {
            let side = content.side;
            // The edge it is pinned to: right-0 inset-y-0, left-0 inset-y-0, top-0 inset-x-0,
            // bottom-0 inset-x-0.
            div()
                .absolute()
                .inset_0()
                .flex()
                .when(!side.is_horizontal(), |d| d.flex_col())
                .when(
                    matches!(side, SheetContentSide::Right | SheetContentSide::Bottom),
                    |d| d.justify_end(),
                )
                .child(content.occlude())
        });
        div().children(trigger).children(surface.map(|surface| {
            let close = handle.clone();
            deferred(
                gpui_base::Sheet::new(cx)
                    .focus_handle(focus)
                    .request_close(move |window, cx| close.close(window, cx))
                    .overlay(Overlay::new())
                    .surface(surface),
            )
            .with_priority(1)
        }))
    }
}

/// The sheet's surface (core-react's `SheetContent`): a header, open content (its Penpot
/// `Body` slot), a footer at the bottom and a close button in the corner.
#[derive(IntoElement)]
pub struct SheetContent {
    side: SheetContentSide,
    header: Option<SheetHeader>,
    footer: Option<SheetFooter>,
    show_close_button: bool,
    occlude: bool,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for SheetContent {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetContent {
    /// An empty `right` surface with its close button.
    pub fn new() -> Self {
        Self {
            side: SheetContentSide::default(),
            header: None,
            footer: None,
            show_close_button: true,
            occlude: false,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `side`.
    pub fn side(mut self, side: SheetContentSide) -> Self {
        self.side = side;
        self
    }

    /// The `Header` part.
    pub fn header(mut self, header: SheetHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// The `Footer` part, pushed to the bottom.
    pub fn footer(mut self, footer: SheetFooter) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Whether the corner close button (the `Dismiss` part) shows; it does by default.
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }

    /// Keeps presses on the surface off the backdrop behind it.
    fn occlude(mut self) -> Self {
        self.occlude = true;
        self
    }
}

impl RenderOnce for SheetContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let side = self.side;
        // flex flex-col gap-4 bg-popover text-sm text-popover-foreground shadow-lg
        div()
            .id("sheet-content")
            .when(self.occlude, |d| d.occlude())
            .relative()
            .flex()
            .flex_col()
            .gap(space::S4)
            .bg(s.popover)
            .text_size(font_size::SM)
            .text_color(s.popover_foreground)
            .shadow(shadow::LG.to_vec())
            .border_color(s.border)
            .map(|d| match side {
                // h-full w-3/4 border-l sm:max-w-sm (border-r at left)
                SheetContentSide::Right => {
                    d.h_full().w(relative(0.75)).max_w(size::SM).border_l_1()
                }
                SheetContentSide::Left => d.h_full().w(relative(0.75)).max_w(size::SM).border_r_1(),
                // h-auto w-full border-b (border-t at bottom)
                SheetContentSide::Top => d.w_full().border_b_1(),
                SheetContentSide::Bottom => d.w_full().border_t_1(),
            })
            .children(self.header)
            .children(self.children)
            .children(self.footer)
            .when(self.show_close_button, |d| d.child(modal::corner_close()))
            .refine_style(&self.style)
    }
}

/// The sheet's `Header` part: its title over its description.
#[derive(IntoElement)]
pub struct SheetHeader {
    title: Option<SheetTitle>,
    description: Option<SheetDescription>,
    style: StyleRefinement,
}

impl Default for SheetHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetHeader {
    /// An empty header.
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            style: StyleRefinement::default(),
        }
    }

    /// The `Title` part.
    pub fn title(mut self, title: SheetTitle) -> Self {
        self.title = Some(title);
        self
    }

    /// The `Description` part, under the title.
    pub fn description(mut self, description: SheetDescription) -> Self {
        self.description = Some(description);
        self
    }
}

impl RenderOnce for SheetHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex flex-col gap-1.5 p-4
        div()
            .flex()
            .flex_col()
            .gap(space::S1_5)
            .p(space::S4)
            .children(self.title)
            .children(self.description)
            .refine_style(&self.style)
    }
}

/// The sheet's `Title` part.
#[derive(IntoElement)]
pub struct SheetTitle {
    text: SharedString,
    style: StyleRefinement,
}

impl SheetTitle {
    /// A title reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for SheetTitle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // font-medium text-foreground
        div()
            .font_weight(font_weight::MEDIUM)
            .text_color(cx.theme().semantic().foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The sheet's `Description` part.
#[derive(IntoElement)]
pub struct SheetDescription {
    text: SharedString,
    style: StyleRefinement,
}

impl SheetDescription {
    /// A description reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for SheetDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // text-sm text-muted-foreground
        div()
            .text_size(font_size::SM)
            .text_color(cx.theme().semantic().muted_foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The sheet's `Footer` part: its buttons stacked, full width.
#[derive(IntoElement)]
pub struct SheetFooter {
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for SheetFooter {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetFooter {
    /// An empty footer; its children are its buttons.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for SheetFooter {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // mt-auto flex flex-col gap-2 p-4
        div()
            .mt_auto()
            .flex()
            .flex_col()
            .gap(space::S2)
            .p(space::S4)
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl_styled!(
    SheetContent,
    SheetHeader,
    SheetTitle,
    SheetDescription,
    SheetFooter
);
impl_parent!(SheetContent, SheetFooter);

#[cfg(test)]
mod tests {
    use gpui::{
        AppContext as _, Context, Modifiers, Render, TestAppContext, VisualTestContext, px,
    };
    use strum::IntoEnumIterator as _;

    use super::*;

    struct Page {
        handle: DialogHandle,
    }

    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            Sheet::new("sheet")
                .handle(self.handle.clone())
                .trigger(Button::new("open").label("Edit profile"))
                .content(
                    SheetContent::new()
                        .header(SheetHeader::new().title(SheetTitle::new("Edit profile"))),
                )
        }
    }

    fn open_page(cx: &mut TestAppContext) -> (&mut VisualTestContext, DialogHandle) {
        cx.update(crate::init);
        let handle = DialogHandle::new(false);
        let page = cx.new({
            let handle = handle.clone();
            |_| Page { handle }
        });
        let window = cx.add_window(move |window, cx| crate::Root::new(page, window, cx));
        let cx = VisualTestContext::from_window(*window, cx).into_mut();
        draw(cx);
        cx.simulate_click(gpui::point(px(10.), px(10.)), Modifiers::default());
        draw(cx);
        (cx, handle)
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
    }

    mod side {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&str> = SheetContentSide::iter().map(Into::into).collect();
            assert_eq!(values, ["top", "right", "bottom", "left"]);
        }
    }

    mod sheet {
        use super::*;

        #[gpui::test]
        fn should_open_from_its_trigger_and_close_on_escape(cx: &mut TestAppContext) {
            let (cx, handle) = open_page(cx);
            assert!(handle.is_open());
            cx.simulate_keystrokes("escape");
            draw(cx);
            assert!(!handle.is_open());
        }

        #[gpui::test]
        fn should_close_when_the_backdrop_left_of_a_right_sheet_is_pressed(
            cx: &mut TestAppContext,
        ) {
            let (cx, handle) = open_page(cx);
            cx.simulate_click(gpui::point(px(5.), px(300.)), Modifiers::default());
            draw(cx);
            assert!(!handle.is_open());
        }

        #[gpui::test]
        fn should_stay_open_when_the_sheet_itself_is_pressed(cx: &mut TestAppContext) {
            let (cx, handle) = open_page(cx);
            let inside = cx.update(|window, _| {
                let viewport = window.viewport_size();
                gpui::point(viewport.width - px(100.), viewport.height / 2.)
            });
            cx.simulate_click(inside, Modifiers::default());
            draw(cx);
            assert!(handle.is_open());
        }
    }
}
