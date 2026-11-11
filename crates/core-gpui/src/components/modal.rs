//! What the modal overlays (Dialog, Alert Dialog, Sheet) share: their open state and the focus
//! that moves into them while open, and the backdrop core-react calls their `Overlay`.
//!
//! `gpui-base` owns the modal behaviour (focus trap, Escape, backdrop dismissal); its hosts
//! leave the open state and focus movement to their caller. Here that state lives in element
//! state keyed by the overlay's id, so an overlay is uncontrolled by default, as Base UI's is;
//! a caller that opens one from elsewhere passes its own [`DialogHandle`].

use gpui::{
    App, ElementId, Entity, FocusHandle, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, StyleRefinement, Styled, Window, div,
};
pub use gpui_base::DialogHandle;
use gpui_base::StyledExt as _;

use crate::components::button::Button;
use crate::icons::IconName;
use crate::theme::ActiveTheme as _;

/// A modal's open state and the focus it takes while open.
pub(crate) struct ModalState {
    handle: DialogHandle,
    focus: FocusHandle,
    /// What had focus when the modal opened, given back when it closes.
    previous: Option<FocusHandle>,
    shown: bool,
}

impl ModalState {
    /// The modal keyed `id` in the current element: created closed (or `default_open`) on
    /// first use; a caller's `handle` replaces the stored one.
    pub(crate) fn use_keyed(
        id: &ElementId,
        handle: Option<DialogHandle>,
        default_open: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<ModalState> {
        let state = window.use_keyed_state(id.clone(), cx, |_, cx| ModalState {
            handle: DialogHandle::new(default_open),
            focus: cx.focus_handle(),
            previous: None,
            shown: false,
        });
        if let Some(handle) = handle {
            state.update(cx, |state, _| state.handle = handle);
        }
        state
    }

    pub(crate) fn handle(&self) -> DialogHandle {
        self.handle.clone()
    }

    pub(crate) fn focus(&self) -> FocusHandle {
        self.focus.clone()
    }

    /// Moves focus into the modal when it has just opened, however it was opened, and back to
    /// where it was when it has just closed. Runs once per render; the focus change waits
    /// until the frame is drawn.
    pub(crate) fn sync_focus(state: &Entity<Self>, window: &mut Window, cx: &mut App) {
        let (open, shown) = {
            let s = state.read(cx);
            (s.handle.is_open(), s.shown)
        };
        if open == shown {
            return;
        }
        let previous = open.then(|| window.focused(cx)).flatten();
        let focus = state.update(cx, |s, _| {
            s.shown = open;
            if open {
                s.previous = previous;
                Some(s.focus.clone())
            } else {
                s.previous.take()
            }
        });
        window.defer(cx, move |window, cx| match focus {
            Some(focus) => focus.focus(window, cx),
            None if !open => window.blur(cx),
            None => {}
        });
    }
}

/// A trigger button that opens `handle` when clicked, after its own handler.
pub(crate) fn trigger(button: Button, handle: DialogHandle) -> Button {
    button.then_on_click(move |_, window, cx| handle.open(window, cx))
}

/// The close button in a dialog's or sheet's corner (`absolute top-4 right-4`, a `ghost`
/// `icon-sm` button with an `X`), dispatching `gpui-base`'s cancel from inside the modal.
pub(crate) fn corner_close() -> impl IntoElement {
    use crate::components::button::{ButtonSize, ButtonVariant};
    use brrritscold_core_gpui_tokens::primitive::space;
    div()
        .id("corner-close")
        .absolute()
        .top(space::S4)
        .right(space::S4)
        .child(gpui_base::DialogClose::new().trigger(|base| {
            Button::new("close")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .icon(IconName::X)
                .on_base(base)
        }))
}

/// A modal's backdrop, exported as `DialogOverlay` and `AlertDialogOverlay` (core-react's
/// overlays, `SheetOverlay` too, share their classes). It fills its container.
///
/// Not mirrored: `supports-backdrop-filter:backdrop-blur-xs`, since GPUI has no backdrop
/// filter, and the fade-in / fade-out.
#[derive(IntoElement)]
pub struct Overlay {
    style: StyleRefinement,
}

impl Default for Overlay {
    fn default() -> Self {
        Self::new()
    }
}

impl Overlay {
    /// A backdrop filling its container.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for Overlay {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let s = theme.semantic();
        // fixed inset-0 bg-foreground/10 dark:bg-background/10
        div()
            .absolute()
            .inset_0()
            .bg(theme.mode().pick(s.foreground, s.background).opacity(0.1))
            .refine_style(&self.style)
    }
}

impl_styled!(Overlay);
