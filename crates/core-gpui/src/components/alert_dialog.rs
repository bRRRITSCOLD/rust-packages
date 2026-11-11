//! Alert Dialog: core-react's `alert-dialog.tsx` on `gpui-base`'s unstyled
//! [`gpui_base::AlertDialog`]: a dialog that asks for an answer, so a backdrop press doesn't
//! dismiss it; Escape and its [`AlertDialogCancel`] do, and its [`AlertDialogAction`] confirms.
//!
//! The content's `size` reaches its header, media, title and footer through
//! `group-data-[size=…]/alert-dialog-content:` in core-react; here [`AlertDialogContent`] takes
//! its parts as typed builders and hands its size down. The header's grid (with media: the
//! media spanning both rows at the left, the title and description beside it) is a row of the
//! media and a text column.
//!
//! Not mirrored: the open and close animations, `text-balance` / `md:text-pretty` (GPUI wraps
//! greedily), the description's link styles, and the narrow-screen layouts below `sm:` (a
//! desktop window is always past the `sm` breakpoint).

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, size, space};
use gpui::{
    AnyElement, App, ClickEvent, ElementId, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use super::button::{Button, ButtonSize, ButtonVariant};
use super::dialog::DialogClose;
pub use super::modal::Overlay as AlertDialogOverlay;
use super::modal::{self, DialogHandle, ModalState};
use crate::icons::{Icon, IconName};
use crate::theme::ActiveTheme as _;

/// Penpot variant property `size` of `AlertDialogContent`. Cases are its values in PascalCase;
/// [`Display`] gives the Penpot value back (`AlertDialogContentSize::Sm` → `sm`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum AlertDialogContentSize {
    /// `default`: 512px wide, the header left-aligned and the buttons at the right.
    #[default]
    Default,
    /// `sm`: 320px wide, the header centred and the buttons sharing the row.
    Sm,
}

impl AlertDialogContentSize {
    /// `data-[size=default]:sm:max-w-lg`, `data-[size=sm]:max-w-xs`.
    fn width(self) -> gpui::Pixels {
        match self {
            AlertDialogContentSize::Default => size::LG,
            AlertDialogContentSize::Sm => size::XS,
        }
    }
}

/// A modal alert dialog: a trigger button that opens [`AlertDialogContent`] over an
/// [`AlertDialogOverlay`].
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent,
///     AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, Button,
///     ButtonVariant,
/// };
/// use gpui::ParentElement as _;
///
/// let confirm = AlertDialog::new("delete-account")
///     .trigger(Button::new("open").variant(ButtonVariant::Outline).label("Delete account"))
///     .content(
///         AlertDialogContent::new()
///             .header(
///                 AlertDialogHeader::new()
///                     .title(AlertDialogTitle::new("Delete your account?"))
///                     .description(AlertDialogDescription::new("This cannot be undone.")),
///             )
///             .footer(
///                 AlertDialogFooter::new()
///                     .child(AlertDialogCancel::new("cancel", "Cancel"))
///                     .child(
///                         AlertDialogAction::new("delete", "Delete")
///                             .variant(ButtonVariant::Destructive)
///                             .on_click(|_, _, _| println!("deleted")),
///                     ),
///             ),
///     );
/// ```
#[derive(IntoElement)]
pub struct AlertDialog {
    id: ElementId,
    trigger: Option<Button>,
    content: Option<AlertDialogContent>,
    handle: Option<DialogHandle>,
    default_open: bool,
}

impl AlertDialog {
    /// A closed alert dialog. `id` keys its open state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: None,
            handle: None,
            default_open: false,
        }
    }

    /// The button that opens it (core-react's `AlertDialogTrigger`), after its own `on_click`.
    pub fn trigger(mut self, trigger: Button) -> Self {
        self.trigger = Some(trigger);
        self
    }

    /// The surface shown while open.
    pub fn content(mut self, content: AlertDialogContent) -> Self {
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

impl RenderOnce for AlertDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = ModalState::use_keyed(&self.id, self.handle, self.default_open, window, cx);
        ModalState::sync_focus(&state, window, cx);
        let (handle, focus) = {
            let state = state.read(cx);
            (state.handle(), state.focus())
        };
        div()
            .children(self.trigger.map(|t| modal::trigger(t, handle.clone())))
            .child(
                gpui_base::AlertDialog::new(cx)
                    .focus_handle(focus)
                    .handle(handle)
                    .backdrop(AlertDialogOverlay::new())
                    .popup(gpui_base::AlertDialogPopup::new().children(self.content)),
            )
    }
}

/// The alert dialog's surface (core-react's `AlertDialogContent`): a header and a footer.
#[derive(IntoElement)]
pub struct AlertDialogContent {
    size: AlertDialogContentSize,
    header: Option<AlertDialogHeader>,
    footer: Option<AlertDialogFooter>,
    style: StyleRefinement,
}

impl Default for AlertDialogContent {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertDialogContent {
    /// An empty `default` surface.
    pub fn new() -> Self {
        Self {
            size: AlertDialogContentSize::default(),
            header: None,
            footer: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `size`.
    pub fn size(mut self, size: AlertDialogContentSize) -> Self {
        self.size = size;
        self
    }

    /// The `Header` part.
    pub fn header(mut self, header: AlertDialogHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// The `Footer` part.
    pub fn footer(mut self, footer: AlertDialogFooter) -> Self {
        self.footer = Some(footer);
        self
    }
}

impl RenderOnce for AlertDialogContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let size = self.size;
        // grid w-full gap-6 rounded-xl bg-popover p-6 text-popover-foreground ring-1
        // ring-foreground/10 data-[size=sm]:max-w-xs data-[size=default]:sm:max-w-lg
        div()
            .w(size.width())
            .flex()
            .flex_col()
            .gap(space::S6)
            .rounded(s.radius_xl)
            .bg(s.popover)
            .p(space::S6)
            .text_color(s.popover_foreground)
            .shadow(vec![super::ring(1., s.foreground.opacity(0.1))])
            .children(self.header.map(|header| header.content_size(size)))
            .children(self.footer.map(|footer| footer.content_size(size)))
            .refine_style(&self.style)
    }
}

/// The alert dialog's `Header` part: media, then its title over its description.
#[derive(IntoElement)]
pub struct AlertDialogHeader {
    size: AlertDialogContentSize,
    media: Option<AlertDialogMedia>,
    title: Option<AlertDialogTitle>,
    description: Option<AlertDialogDescription>,
    style: StyleRefinement,
}

impl Default for AlertDialogHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertDialogHeader {
    /// An empty header, laid out for a `default` surface until one renders it.
    pub fn new() -> Self {
        Self {
            size: AlertDialogContentSize::default(),
            media: None,
            title: None,
            description: None,
            style: StyleRefinement::default(),
        }
    }

    /// The `Media` part: an icon beside (`default`) or above (`sm`) the text.
    pub fn media(mut self, media: AlertDialogMedia) -> Self {
        self.media = Some(media);
        self
    }

    /// The `Title` part.
    pub fn title(mut self, title: AlertDialogTitle) -> Self {
        self.title = Some(title);
        self
    }

    /// The `Description` part, under the title.
    pub fn description(mut self, description: AlertDialogDescription) -> Self {
        self.description = Some(description);
        self
    }

    fn content_size(mut self, size: AlertDialogContentSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for AlertDialogHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let centred = self.size == AlertDialogContentSize::Sm;
        // grid gap-1.5: the title over the description, centred (text-center
        // place-items-center) unless sm:group-data-[size=default]: place-items-start text-left
        let text = div()
            .flex()
            .flex_col()
            .gap(space::S1_5)
            .when(centred, |d| d.items_center().text_center())
            .children(self.title)
            .children(self.description);
        let media_beside = self.media.is_some() && !centred;
        div()
            .flex()
            // With media at default size, media | text (has-data-[slot=alert-dialog-media]:gap-x-6,
            // the media row-span-2); otherwise one centred or left-aligned column.
            .when(media_beside, |d| d.items_start().gap(space::S6))
            .when(!media_beside, |d| {
                d.flex_col()
                    .gap(space::S1_5)
                    .when(centred, |d| d.items_center())
            })
            .children(self.media)
            .child(text.when(media_beside, |d| d.flex_1().min_w_0()))
            .refine_style(&self.style)
    }
}

/// The alert dialog header's `Media` part: an icon on a muted square.
#[derive(IntoElement)]
pub struct AlertDialogMedia {
    icon: IconName,
    style: StyleRefinement,
}

impl AlertDialogMedia {
    /// Media showing `icon` (its `Icon` part).
    pub fn new(icon: IconName) -> Self {
        Self {
            icon,
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for AlertDialogMedia {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // mb-2 inline-flex size-16 items-center justify-center rounded-md bg-muted
        // *:[svg:not([class*='size-'])]:size-8
        div()
            .mb(space::S2)
            .flex()
            .flex_none()
            .size(space::S16)
            .items_center()
            .justify_center()
            .rounded(s.radius_md)
            .bg(s.muted)
            .child(Icon::new(self.icon).size(space::S8))
            .refine_style(&self.style)
    }
}

/// The alert dialog's `Title` part.
#[derive(IntoElement)]
pub struct AlertDialogTitle {
    text: SharedString,
    style: StyleRefinement,
}

impl AlertDialogTitle {
    /// A title reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for AlertDialogTitle {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // text-lg font-medium
        div()
            .text_size(font_size::LG)
            .font_weight(font_weight::MEDIUM)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The alert dialog's `Description` part.
#[derive(IntoElement)]
pub struct AlertDialogDescription {
    text: SharedString,
    style: StyleRefinement,
}

impl AlertDialogDescription {
    /// A description reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for AlertDialogDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // text-sm text-muted-foreground
        div()
            .text_size(font_size::SM)
            .text_color(cx.theme().semantic().muted_foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The alert dialog's `Footer` part: its [`AlertDialogCancel`] and [`AlertDialogAction`].
#[derive(IntoElement)]
pub struct AlertDialogFooter {
    size: AlertDialogContentSize,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for AlertDialogFooter {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertDialogFooter {
    /// An empty footer; its children are its buttons.
    pub fn new() -> Self {
        Self {
            size: AlertDialogContentSize::default(),
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    fn content_size(mut self, size: AlertDialogContentSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for AlertDialogFooter {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let shared_row = self.size == AlertDialogContentSize::Sm;
        // flex gap-2 sm:flex-row sm:justify-end; at sm, grid grid-cols-2: each button fills
        // half the row.
        let footer = div().flex().gap(space::S2);
        if shared_row {
            footer.children(
                self.children
                    .into_iter()
                    .map(|child| div().flex_1().flex().flex_col().child(child)),
            )
        } else {
            footer.justify_end().children(self.children)
        }
        .refine_style(&self.style)
    }
}

/// The alert dialog's `Action` part: a button that confirms and closes it.
#[derive(IntoElement)]
pub struct AlertDialogAction {
    button: Button,
}

impl AlertDialogAction {
    /// A `default` button reading `label`.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            button: Button::new(id).label(label),
        }
    }

    /// The button's `variant`.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.button = self.button.variant(variant);
        self
    }

    /// The button's `size`.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.button = self.button.size(size);
        self
    }

    /// Runs on activation, before the dialog closes.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.button = self.button.on_click(handler);
        self
    }
}

impl RenderOnce for AlertDialogAction {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.button.then_on_click(|_, window, cx| {
            window.dispatch_action(
                Box::new(gpui_base::actions::Confirm { secondary: false }),
                cx,
            )
        })
    }
}

/// The alert dialog's `Cancel` part: an `outline` button that closes it.
#[derive(IntoElement)]
pub struct AlertDialogCancel {
    button: Button,
}

impl AlertDialogCancel {
    /// An `outline` button reading `label`.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            button: Button::new(id).variant(ButtonVariant::Outline).label(label),
        }
    }

    /// The button's `variant`, `outline` unless set.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.button = self.button.variant(variant);
        self
    }

    /// The button's `size`.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.button = self.button.size(size);
        self
    }
}

impl RenderOnce for AlertDialogCancel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        DialogClose::new(self.button)
    }
}

impl_styled!(
    AlertDialogContent,
    AlertDialogHeader,
    AlertDialogMedia,
    AlertDialogTitle,
    AlertDialogDescription,
    AlertDialogFooter
);
impl_parent!(AlertDialogFooter);

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{AppContext as _, Context, Modifiers, Render, TestAppContext, VisualTestContext};
    use strum::IntoEnumIterator as _;

    use super::*;

    struct Page {
        handle: DialogHandle,
        deleted: Rc<Cell<usize>>,
    }

    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let deleted = self.deleted.clone();
            AlertDialog::new("alert")
                .handle(self.handle.clone())
                .content(
                    AlertDialogContent::new()
                        .size(AlertDialogContentSize::Sm)
                        .header(AlertDialogHeader::new().title(AlertDialogTitle::new("Delete?")))
                        .footer(
                            AlertDialogFooter::new()
                                .child(AlertDialogCancel::new("cancel", "Cancel"))
                                .child(
                                    AlertDialogAction::new("delete", "Delete")
                                        .on_click(move |_, _, _| deleted.set(deleted.get() + 1)),
                                ),
                        ),
                )
        }
    }

    fn open_page(cx: &mut TestAppContext) -> (&mut VisualTestContext, gpui::Entity<Page>) {
        cx.update(crate::init);
        let page = cx.new(|_| Page {
            handle: DialogHandle::new(false),
            deleted: Rc::new(Cell::new(0)),
        });
        let window = cx.add_window({
            let page = page.clone();
            move |window, cx| crate::Root::new(page, window, cx)
        });
        let cx = VisualTestContext::from_window(*window, cx).into_mut();
        let handle = cx.read(|cx| page.read(cx).handle.clone());
        cx.update(|window, cx| handle.open(window, cx));
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

    /// The cancel (left) or action (right) button of the centred 320px surface: 24px padding,
    /// a 28px title, a 24px gap, then the 36px buttons sharing the row, so their centres sit
    /// 70px either side of the window's centre and 26px below it.
    fn button_point(cx: &mut VisualTestContext, right: bool) -> gpui::Point<gpui::Pixels> {
        cx.update(|window, _| {
            let viewport = window.viewport_size();
            let x = viewport.width / 2. + gpui::px(if right { 70. } else { -70. });
            gpui::point(x, viewport.height / 2. + gpui::px(26.))
        })
    }

    mod content_size {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&str> = AlertDialogContentSize::iter().map(Into::into).collect();
            assert_eq!(values, ["default", "sm"]);
        }
    }

    mod alert_dialog {
        use super::*;

        #[gpui::test]
        fn should_stay_open_when_the_backdrop_is_pressed(cx: &mut TestAppContext) {
            let (cx, page) = open_page(cx);
            cx.simulate_click(
                gpui::point(gpui::px(2.), gpui::px(2.)),
                Modifiers::default(),
            );
            draw(cx);
            assert!(is_open(cx, &page));
        }

        #[gpui::test]
        fn should_close_from_cancel_without_running_the_action(cx: &mut TestAppContext) {
            let (cx, page) = open_page(cx);
            let cancel = button_point(cx, false);
            cx.simulate_click(cancel, Modifiers::default());
            draw(cx);
            assert!(!is_open(cx, &page));
            assert_eq!(cx.read(|cx| page.read(cx).deleted.get()), 0);
        }

        #[gpui::test]
        fn should_run_the_action_then_close(cx: &mut TestAppContext) {
            let (cx, page) = open_page(cx);
            let action = button_point(cx, true);
            cx.simulate_click(action, Modifiers::default());
            draw(cx);
            assert!(!is_open(cx, &page));
            assert_eq!(cx.read(|cx| page.read(cx).deleted.get()), 1);
        }
    }
}
