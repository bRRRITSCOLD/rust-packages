//! Dialog: core-react's `dialog.tsx` on `gpui-base`'s unstyled [`gpui_base::Dialog`], which
//! owns the focus trap, Escape, backdrop dismissal and the dialog role.
//!
//! [`DialogContent`] is the surface and renders on its own, the way the gallery and captures
//! show it; [`Dialog`] puts it over a [`DialogOverlay`], centred in the window, while open. The
//! open state is the [`Dialog`]'s own (keyed by its id), or a [`DialogHandle`] the caller owns.
//!
//! Not mirrored: the open and close animations (`fade-in-0 zoom-in-95`), the description's
//! link styles (`*:[a]:underline`), and the footer's narrow-screen stacking
//! (`flex-col-reverse`, below `sm:`): a desktop window is always past the `sm` breakpoint.

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, size, space};
use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _, relative,
};
use gpui_base::StyledExt as _;

use super::button::{Button, ButtonVariant};
pub use super::modal::DialogHandle;
pub use super::modal::Overlay as DialogOverlay;
use super::modal::{self, ModalState};
use crate::theme::ActiveTheme as _;

/// A modal dialog: a trigger button that opens [`DialogContent`] over a [`DialogOverlay`].
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Button, ButtonVariant, Dialog, DialogClose, DialogContent, DialogDescription,
///     DialogFooter, DialogHeader, DialogTitle,
/// };
/// use gpui::ParentElement as _;
///
/// let dialog = Dialog::new("edit-profile")
///     .trigger(Button::new("open").variant(ButtonVariant::Outline).label("Edit profile"))
///     .content(
///         DialogContent::new()
///             .header(
///                 DialogHeader::new()
///                     .title(DialogTitle::new("Edit profile"))
///                     .description(DialogDescription::new("Make changes to your profile here.")),
///             )
///             .footer(
///                 DialogFooter::new()
///                     .child(DialogClose::new(
///                         Button::new("cancel").variant(ButtonVariant::Outline).label("Cancel"),
///                     ))
///                     .child(Button::new("save").label("Save changes")),
///             ),
///     );
/// ```
#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    trigger: Option<Button>,
    content: Option<DialogContent>,
    handle: Option<DialogHandle>,
    default_open: bool,
}

impl Dialog {
    /// A closed dialog. `id` keys its open state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: None,
            handle: None,
            default_open: false,
        }
    }

    /// The button that opens the dialog (core-react's `DialogTrigger`), after its own
    /// `on_click`.
    pub fn trigger(mut self, trigger: Button) -> Self {
        self.trigger = Some(trigger);
        self
    }

    /// The surface shown while open.
    pub fn content(mut self, content: DialogContent) -> Self {
        self.content = Some(content);
        self
    }

    /// Opens and closes through `handle` instead of the dialog's own state, so the caller can
    /// open it from anywhere.
    pub fn handle(mut self, handle: DialogHandle) -> Self {
        self.handle = Some(handle);
        self
    }

    /// Starts open (Base UI's `defaultOpen`).
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = ModalState::use_keyed(&self.id, self.handle, self.default_open, window, cx);
        ModalState::sync_focus(&state, window, cx);
        let (handle, focus) = {
            let state = state.read(cx);
            (state.handle(), state.focus())
        };
        let title = self.content.as_ref().and_then(DialogContent::title);
        div()
            .children(self.trigger.map(|t| modal::trigger(t, handle.clone())))
            .child(
                gpui_base::Dialog::new(cx)
                    .focus_handle(focus)
                    .handle(handle)
                    .when_some(title, |d, title| d.accessibility_label(title))
                    .backdrop(DialogOverlay::new())
                    .popup(gpui_base::DialogPopup::new().children(self.content)),
            )
    }
}

/// The dialog's surface (core-react's `DialogContent`): a header, open content (its Penpot
/// `Body` slot), a footer and a close button in the corner.
#[derive(IntoElement)]
pub struct DialogContent {
    header: Option<DialogHeader>,
    footer: Option<DialogFooter>,
    show_close_button: bool,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for DialogContent {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogContent {
    /// An empty surface with its close button.
    pub fn new() -> Self {
        Self {
            header: None,
            footer: None,
            show_close_button: true,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// The `Header` part.
    pub fn header(mut self, header: DialogHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// The `Footer` part, after the content.
    pub fn footer(mut self, footer: DialogFooter) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Whether the corner close button (the `Dismiss` part) shows; it does by default.
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }

    fn title(&self) -> Option<SharedString> {
        self.header.as_ref()?.title.as_ref().map(|t| t.text.clone())
    }
}

impl RenderOnce for DialogContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // grid gap-6 rounded-xl bg-popover p-6 text-sm text-popover-foreground ring-1
        // ring-foreground/10 sm:max-w-md (inset-x-4: the window's 1rem gutter)
        div()
            .relative()
            .w(size::MD)
            .flex()
            .flex_col()
            .gap(space::S6)
            .rounded(s.radius_xl)
            .bg(s.popover)
            .p(space::S6)
            .text_size(font_size::SM)
            .text_color(s.popover_foreground)
            .shadow(vec![super::ring(1., s.foreground.opacity(0.1))])
            .children(self.header)
            .children(self.children)
            .children(self.footer)
            .when(self.show_close_button, |d| d.child(modal::corner_close()))
            .refine_style(&self.style)
    }
}

/// The dialog's `Header` part: its title over its description.
#[derive(IntoElement)]
pub struct DialogHeader {
    title: Option<DialogTitle>,
    description: Option<DialogDescription>,
    style: StyleRefinement,
}

impl Default for DialogHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogHeader {
    /// An empty header.
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            style: StyleRefinement::default(),
        }
    }

    /// The `Title` part.
    pub fn title(mut self, title: DialogTitle) -> Self {
        self.title = Some(title);
        self
    }

    /// The `Description` part, under the title.
    pub fn description(mut self, description: DialogDescription) -> Self {
        self.description = Some(description);
        self
    }
}

impl RenderOnce for DialogHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex flex-col gap-2
        div()
            .flex()
            .flex_col()
            .gap(space::S2)
            .children(self.title)
            .children(self.description)
            .refine_style(&self.style)
    }
}

/// The dialog's `Title` part.
#[derive(IntoElement)]
pub struct DialogTitle {
    text: SharedString,
    style: StyleRefinement,
}

impl DialogTitle {
    /// A title reading `text`; it also names the dialog for assistive technology.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for DialogTitle {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // leading-none font-medium
        div()
            .line_height(relative(1.))
            .font_weight(font_weight::MEDIUM)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The dialog's `Description` part.
#[derive(IntoElement)]
pub struct DialogDescription {
    text: SharedString,
    style: StyleRefinement,
}

impl DialogDescription {
    /// A description reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for DialogDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // text-sm text-muted-foreground
        div()
            .text_size(font_size::SM)
            .text_color(cx.theme().semantic().muted_foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The dialog's `Footer` part: its buttons in a row at the right.
#[derive(IntoElement)]
pub struct DialogFooter {
    show_close_button: bool,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for DialogFooter {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogFooter {
    /// An empty footer; its children are its buttons.
    pub fn new() -> Self {
        Self {
            show_close_button: false,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Ends the row with an `outline` "Close" button, for a dialog that only informs.
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }
}

impl RenderOnce for DialogFooter {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex flex-col-reverse gap-2 sm:flex-row sm:justify-end
        div()
            .flex()
            .justify_end()
            .gap(space::S2)
            .children(self.children)
            .when(self.show_close_button, |d| {
                d.child(DialogClose::new(
                    Button::new("footer-close")
                        .variant(ButtonVariant::Outline)
                        .label("Close"),
                ))
            })
            .refine_style(&self.style)
    }
}

/// A button that closes the dialog (or sheet) it sits in, as core-react's `DialogClose` with a
/// `render` button does. A code-only part: the library draws it as the button it wraps.
#[derive(IntoElement)]
pub struct DialogClose {
    button: Button,
}

impl DialogClose {
    /// Makes `button` close its dialog, after its own `on_click`.
    pub fn new(button: Button) -> Self {
        Self { button }
    }
}

impl RenderOnce for DialogClose {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // The button has focus when activated, inside the dialog, so the cancel it dispatches
        // reaches the dialog's host as Escape's does.
        self.button.then_on_click(|_, window, cx| {
            window.dispatch_action(Box::new(gpui_base::actions::Cancel), cx)
        })
    }
}

impl_styled!(
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
    DialogFooter
);
impl_parent!(DialogContent, DialogFooter);

#[cfg(test)]
mod tests {
    use gpui::{
        AppContext as _, Context, FocusHandle, InteractiveElement as _, Modifiers, Render,
        TestAppContext, VisualTestContext, px,
    };

    use super::*;

    fn content() -> DialogContent {
        DialogContent::new()
            .header(
                DialogHeader::new()
                    .title(DialogTitle::new("Edit profile"))
                    .description(DialogDescription::new("Make changes to your profile here.")),
            )
            .footer(
                DialogFooter::new()
                    .child(DialogClose::new(
                        Button::new("cancel")
                            .variant(ButtonVariant::Outline)
                            .label("Cancel"),
                    ))
                    .child(Button::new("save").label("Save changes")),
            )
    }

    /// A page with a focusable control outside the dialog, then the dialog's trigger.
    struct Page {
        outside: FocusHandle,
        handle: DialogHandle,
        content: fn() -> DialogContent,
    }

    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .child(
                    div()
                        .id("outside")
                        .size(px(20.))
                        .track_focus(&self.outside.clone().tab_stop(true)),
                )
                .child(
                    Dialog::new("dialog")
                        .handle(self.handle.clone())
                        .trigger(Button::new("open").label("Edit profile"))
                        .content((self.content)()),
                )
        }
    }

    /// Opens the page in a [`crate::Root`] window and draws it.
    fn page(cx: &mut TestAppContext) -> (&mut VisualTestContext, gpui::Entity<Page>) {
        page_with(cx, content)
    }

    fn page_with(
        cx: &mut TestAppContext,
        content: fn() -> DialogContent,
    ) -> (&mut VisualTestContext, gpui::Entity<Page>) {
        cx.update(crate::init);
        let page = cx.new(|cx| Page {
            outside: cx.focus_handle(),
            handle: DialogHandle::new(false),
            content,
        });
        let window = cx.add_window({
            let page = page.clone();
            move |window, cx| crate::Root::new(page, window, cx)
        });
        let cx = VisualTestContext::from_window(*window, cx).into_mut();
        draw(cx);
        (cx, page)
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
    }

    fn is_open(cx: &mut VisualTestContext, page: &gpui::Entity<Page>) -> bool {
        cx.read(|cx| page.read(cx).handle.is_open())
    }

    /// Clicks the trigger, which sits right of the 20px outside control.
    fn open(cx: &mut VisualTestContext) {
        cx.simulate_click(gpui::point(px(40.), px(10.)), Modifiers::default());
        draw(cx);
    }

    /// Focuses the control outside, then opens the dialog through its handle.
    fn focus_outside_then_open(
        cx: &mut VisualTestContext,
        page: &gpui::Entity<Page>,
    ) -> FocusHandle {
        let (outside, handle) = cx.read(|cx| {
            let page = page.read(cx);
            (page.outside.clone(), page.handle.clone())
        });
        cx.update(|window, cx| outside.focus(window, cx));
        draw(cx);
        cx.update(|window, cx| handle.open(window, cx));
        draw(cx);
        outside
    }

    mod dialog {
        use super::*;

        #[gpui::test]
        fn should_open_from_its_trigger_and_close_on_escape(cx: &mut TestAppContext) {
            let (cx, page) = page(cx);
            assert!(!is_open(cx, &page));
            open(cx);
            assert!(is_open(cx, &page));
            cx.simulate_keystrokes("escape");
            draw(cx);
            assert!(!is_open(cx, &page));
        }

        #[gpui::test]
        fn should_close_when_the_backdrop_is_pressed(cx: &mut TestAppContext) {
            let (cx, page) = page(cx);
            open(cx);
            cx.simulate_click(gpui::point(px(2.), px(500.)), Modifiers::default());
            draw(cx);
            assert!(!is_open(cx, &page));
        }

        #[gpui::test]
        fn should_close_from_a_dialog_close_button(cx: &mut TestAppContext) {
            let (cx, page) = page_with(cx, || {
                DialogContent::new()
                    .show_close_button(false)
                    .child(DialogClose::new(
                        Button::new("close").label("Close").w_full().h(px(200.)),
                    ))
            });
            open(cx);
            let center = cx.update(|window, _| {
                let viewport = window.viewport_size();
                gpui::point(viewport.width / 2., viewport.height / 2.)
            });
            cx.simulate_click(center, Modifiers::default());
            draw(cx);
            assert!(!is_open(cx, &page));
        }

        #[gpui::test]
        fn should_keep_tab_inside_the_open_dialog(cx: &mut TestAppContext) {
            let (cx, page) = page(cx);
            let outside = focus_outside_then_open(cx, &page);
            for _ in 0..10 {
                cx.simulate_keystrokes("tab");
                draw(cx);
                assert!(cx.update(|window, _| !outside.is_focused(window)));
            }
            assert!(is_open(cx, &page));
        }

        #[gpui::test]
        fn should_give_focus_back_when_closed(cx: &mut TestAppContext) {
            let (cx, page) = page(cx);
            let outside = focus_outside_then_open(cx, &page);
            assert!(cx.update(|window, _| !outside.is_focused(window)));
            cx.simulate_keystrokes("escape");
            draw(cx);
            assert!(cx.update(|window, _| outside.is_focused(window)));
        }
    }
}
