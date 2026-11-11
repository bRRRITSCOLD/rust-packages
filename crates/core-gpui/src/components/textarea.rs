//! Textarea: core-react's `textarea.tsx` on `gpui-base`'s unstyled multi-line editor. As with
//! [`crate::Input`], the text lives in a [`TextareaState`] entity the app creates and keeps,
//! and [`Textarea`] draws core-react's frame (shared with Input) around it.
//!
//! The `Placeholder` layer is the state's: `TextareaState::new(window, cx).placeholder(..)`.
//! `field-sizing-content` (growing with its text) is the state's too:
//! `TextareaState::new(window, cx).auto_grow(min_rows, max_rows)`.
//!
//! Not mirrored: `text-base md:text-sm` and `cursor-not-allowed`, as for Input.

use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{App, Entity, Focusable as _, IntoElement, RenderOnce, StyleRefinement, Styled, Window};
use gpui_base::StyledExt as _;
pub use gpui_base::input::TextareaState;

use super::input::frame;

/// A multi-line text input, styled like core-react's `Textarea`. One builder per Penpot
/// variant property ([`aria_invalid`](Self::aria_invalid), [`disabled`](Self::disabled)) over
/// the [`TextareaState`] that holds its text:
///
/// ```no_run
/// use brrritscold_core_gpui::{Textarea, TextareaState};
/// use gpui::{AppContext as _, Context, Entity, IntoElement, Render, Window};
///
/// struct Feedback {
///     message: Entity<TextareaState>,
/// }
///
/// impl Feedback {
///     fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
///         let message =
///             cx.new(|cx| TextareaState::new(window, cx).placeholder("Type your message here."));
///         Self { message }
///     }
/// }
///
/// impl Render for Feedback {
///     fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
///         Textarea::new(&self.message)
///     }
/// }
/// ```
#[derive(IntoElement)]
pub struct Textarea {
    state: Entity<TextareaState>,
    aria_invalid: bool,
    disabled: bool,
    style: StyleRefinement,
}

impl Textarea {
    /// A textarea showing and editing `state`.
    pub fn new(state: &Entity<TextareaState>) -> Self {
        Self {
            state: state.clone(),
            aria_invalid: false,
            disabled: false,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `aria-invalid`: a `destructive` border and ring.
    pub fn aria_invalid(mut self, invalid: bool) -> Self {
        self.aria_invalid = invalid;
        self
    }

    /// Penpot property `disabled`: dims the textarea to 50% and stops it taking focus or text.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for Textarea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.disabled;
        self.state
            .update(cx, |state, cx| state.set_disabled(disabled, cx));
        let focused = self.state.read(cx).focus_handle(cx).is_focused(window);
        // flex field-sizing-content min-h-16 w-full py-2
        frame(
            self.state.entity_id(),
            gpui_base::input::Textarea::new(&self.state),
            focused,
            self.aria_invalid,
            disabled,
            cx,
        )
        .flex()
        .min_h(space::S16)
        .py(space::S2)
        .refine_style(&self.style)
    }
}

impl_styled!(Textarea);

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, px};

    use super::*;
    use crate::components::test_support::{render, state};

    mod textarea {
        use super::*;

        #[gpui::test]
        fn should_be_at_least_64px_tall(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let state = state(cx, TextareaState::new);
            let (_, bounds) = render(cx, move || Textarea::new(&state).w(px(320.)));
            assert_eq!(bounds.size.height, px(64.));
        }

        #[gpui::test]
        fn should_keep_typed_lines(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let state = state(cx, TextareaState::new);
            let shown = state.clone();
            let (cx, _) = render(cx, move || Textarea::new(&shown).w(px(320.)));
            cx.update(|window, cx| state.update(cx, |state, cx| state.focus(window, cx)));
            cx.simulate_input("Hello");
            cx.simulate_keystrokes("enter");
            cx.simulate_input("world");
            assert_eq!(cx.read(|cx| state.read(cx).value()), "Hello\nworld");
        }
    }
}
