//! Input: core-react's `input.tsx` on `gpui-base`'s unstyled single-line editor. The text,
//! caret, selection, keyboard editing and placeholder live in an [`InputState`] entity the app
//! creates and keeps, as `gpui-base` and gpui-component do; [`Input`] draws core-react's frame
//! around it.
//!
//! The `Placeholder` layer is the state's: `InputState::new(window, cx).placeholder(..)`.
//!
//! Not mirrored:
//!
//! - `text-base md:text-sm`: an input is `text-sm`, core-react's size at desktop widths. The
//!   published Storybook's 640px capture viewport sits below `md`, so its column shows 16px
//!   text.
//! - The `file:` arms: GPUI has no file input.
//! - `cursor-not-allowed` while disabled.

use brrritscold_core_gpui_tokens::primitive::{font_size, opacity, space};
use gpui::{
    App, Entity, EntityId, Focusable as _, IntoElement, ParentElement as _, RenderOnce,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _, transparent_black,
};
use gpui_base::StyledExt as _;
use gpui_base::input::InputBase;
pub use gpui_base::input::InputState;

use super::control::{Outline, dimmed, dimmed_shadows, on_background, shadows};
use crate::theme::ActiveTheme as _;

/// A text input, styled like core-react's `Input`. One builder per Penpot variant property
/// ([`aria_invalid`](Self::aria_invalid), [`disabled`](Self::disabled)) over the
/// [`InputState`] that holds its text:
///
/// ```no_run
/// use brrritscold_core_gpui::{Input, InputState};
/// use gpui::{AppContext as _, Context, Entity, IntoElement, Render, Window};
///
/// struct SignIn {
///     email: Entity<InputState>,
/// }
///
/// impl SignIn {
///     fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
///         let email = cx.new(|cx| InputState::new(window, cx).placeholder("you@example.com"));
///         Self { email }
///     }
/// }
///
/// impl Render for SignIn {
///     fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
///         let invalid = !self.email.read(cx).value().contains('@');
///         Input::new(&self.email).aria_invalid(invalid)
///     }
/// }
/// ```
#[derive(IntoElement)]
pub struct Input {
    state: Entity<InputState>,
    aria_invalid: bool,
    disabled: bool,
    style: StyleRefinement,
}

impl Input {
    /// An input showing and editing `state`.
    pub fn new(state: &Entity<InputState>) -> Self {
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

    /// Penpot property `disabled`: dims the input to 50% and stops it taking focus or text.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.disabled;
        self.state
            .update(cx, |state, cx| state.set_disabled(disabled, cx));
        let focused = self.state.read(cx).focus_handle(cx).is_focused(window);
        // h-9 w-full min-w-0 px-2.5 py-1, and its text vertically centred like a native input
        frame(
            self.state.entity_id(),
            gpui_base::input::Input::new(&self.state),
            focused,
            self.aria_invalid,
            disabled,
            cx,
        )
        .flex()
        .items_center()
        .h(space::S9)
        .py(space::S1)
        .refine_style(&self.style)
    }
}

impl_styled!(Input);

/// The frame Input and Textarea share around `editor` (whose state `editor_id` names):
/// `w-full min-w-0 rounded-md border border-input bg-transparent px-2.5 shadow-xs md:text-sm`,
/// the focus and invalid outline, `dark:bg-input/30` and `disabled:opacity-50`.
///
/// The frame is dimmed layer by layer (see `control::dimmed`) and the text through an
/// opacity on its own, so the frame's ring and shadow don't show through a half-transparent
/// fill.
pub(super) fn frame(
    editor_id: EntityId,
    editor: impl IntoElement,
    focused: bool,
    invalid: bool,
    disabled: bool,
    cx: &App,
) -> InputBase {
    let theme = cx.theme();
    let s = theme.semantic();
    let outline = Outline::new(theme, invalid);
    // A text field is focus-visible whenever it has focus, from the keyboard or the pointer.
    let (mut border, ring) = match outline.focus.filter(|_| focused && !disabled) {
        Some((border, ring)) => (border, Some(ring)),
        None => (outline.border, outline.ring),
    };
    // bg-transparent dark:bg-input/30, made opaque: it sits on a ring and shadow-xs
    let mut fill = on_background(
        theme,
        theme.mode().pick(transparent_black(), s.input.opacity(0.3)),
    );
    let mut shadow = shadows(ring, true);
    if disabled {
        border = dimmed(theme, fill.blend(border));
        fill = dimmed(theme, fill);
        shadow = dimmed_shadows(shadow);
    }
    InputBase::new(("brrritscold-core-gpui-input", editor_id))
        .focused(focused)
        .disabled(disabled)
        .w_full()
        .min_w_0()
        .rounded(s.radius_md)
        .border_1()
        .border_color(border)
        .bg(fill)
        .px(space::S2_5)
        .text_size(font_size::SM)
        .shadow(shadow)
        .child(
            div()
                .flex()
                .flex_1()
                .min_w_0()
                .when(disabled, |d| d.opacity(opacity::O50))
                .child(editor),
        )
}

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, VisualTestContext, px};

    use super::*;
    use crate::components::test_support::{render, state};

    /// Renders an input 320px wide on a fresh state and returns the state with the window.
    fn input(
        cx: &mut TestAppContext,
        disabled: bool,
    ) -> (Entity<InputState>, &mut VisualTestContext) {
        let state = state(cx, |window, cx| {
            InputState::new(window, cx).placeholder("you@example.com")
        });
        let shown = state.clone();
        let (cx, _) = render(cx, move || {
            Input::new(&shown).disabled(disabled).w(px(320.))
        });
        (state, cx)
    }

    mod input {
        use super::*;

        #[gpui::test]
        fn should_be_36px_tall(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let state = state(cx, InputState::new);
            let (_, bounds) = render(cx, move || Input::new(&state));
            assert_eq!(bounds.size.height, px(36.));
        }

        #[gpui::test]
        fn should_take_typed_text(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (state, cx) = input(cx, false);
            cx.update(|window, cx| state.update(cx, |state, cx| state.focus(window, cx)));
            cx.simulate_input("ada@example.com");
            assert_eq!(cx.read(|cx| state.read(cx).value()), "ada@example.com");
        }

        #[gpui::test]
        fn should_take_no_text_while_disabled(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (state, cx) = input(cx, true);
            cx.update(|window, cx| state.update(cx, |state, cx| state.focus(window, cx)));
            cx.simulate_input("ada");
            assert_eq!(cx.read(|cx| state.read(cx).value()), "");
        }
    }
}
