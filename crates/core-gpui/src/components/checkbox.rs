//! Checkbox: core-react's `checkbox.tsx` on `gpui-base`'s unstyled [`gpui_base::Checkbox`],
//! which owns focus, Space and click activation, disabled inertness and accessibility.
//!
//! The checkbox is controlled: it shows [`Checkbox::checked`] and reports the next value to
//! [`Checkbox::on_change`], as core-react's `checked` / `onCheckedChange` do.
//!
//! Not mirrored:
//!
//! - `after:-inset-x-3 after:-inset-y-2`: the hit area stays the 16px box.
//! - The `group-has-…/field` and `/field-label` arms, which read the checkbox's place in a
//!   `Field` through CSS relations GPUI has no equivalent of.
//! - The indeterminate state: core-react draws no indeterminate mark and Penpot has no value
//!   for it.

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{
    App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StyleRefinement, Styled, Window, prelude::FluentBuilder as _, px,
    transparent_black,
};
use gpui_base::{CheckboxState, StyledExt as _};

use super::control::{Outline, dimmed, dimmed_shadows, on_background, shadows};
use crate::icons::{Icon, IconName};
use crate::theme::{ActiveTheme as _, Theme};

type ChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A checkbox, styled like core-react's `Checkbox`. One builder per Penpot variant property
/// ([`checked`](Self::checked), [`aria_invalid`](Self::aria_invalid),
/// [`disabled`](Self::disabled)):
///
/// ```no_run
/// use brrritscold_core_gpui::Checkbox;
///
/// let terms = Checkbox::new("terms")
///     .checked(true)
///     .accessibility_label("Accept terms")
///     .on_change(|checked, _, _| println!("accepted: {checked}"));
/// ```
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    aria_invalid: bool,
    disabled: bool,
    accessibility_label: Option<SharedString>,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
}

impl Checkbox {
    /// An unchecked checkbox. `id` keys its focus state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            aria_invalid: false,
            disabled: false,
            accessibility_label: None,
            on_change: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `checked`: fills the box in `primary` and draws the check.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Penpot property `aria-invalid`: a `destructive` border and ring.
    pub fn aria_invalid(mut self, invalid: bool) -> Self {
        self.aria_invalid = invalid;
        self
    }

    /// Penpot property `disabled`: dims the box to 50% and makes it inert.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The name assistive technology reads, for a checkbox drawn without text beside it.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Runs on a click or Space with the value the checkbox asks to change to. Never while
    /// disabled.
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

/// The box's colours for one state, resolved for the active mode.
#[derive(Debug, Clone, Copy, PartialEq)]
struct BoxStyle {
    fill: Hsla,
    border: Hsla,
    /// The check's colour.
    text: Hsla,
    outline: Outline,
}

impl BoxStyle {
    fn new(theme: &Theme, checked: bool, invalid: bool, disabled: bool) -> Self {
        let style = Self::enabled(theme, checked, invalid);
        if !disabled {
            return style;
        }
        // data-disabled:opacity-50, layer by layer: the border over the fill, the check over it.
        Self {
            fill: dimmed(theme, style.fill),
            border: dimmed(theme, style.fill.blend(style.border)),
            text: dimmed(theme, style.fill.blend(style.text)),
            ..style
        }
    }

    fn enabled(theme: &Theme, checked: bool, invalid: bool) -> Self {
        let s = theme.semantic();
        let outline = Outline::new(theme, invalid);
        if checked {
            // data-checked:border-primary data-checked:bg-primary
            // data-checked:text-primary-foreground dark:data-checked:bg-primary
            // aria-invalid:aria-checked:border-primary
            Self {
                fill: s.primary,
                border: s.primary,
                text: s.primary_foreground,
                outline,
            }
        } else {
            Self {
                // dark:bg-input/30, made opaque: the box sits on shadow-xs and maybe a ring
                fill: on_background(
                    theme,
                    theme.mode().pick(transparent_black(), s.input.opacity(0.3)),
                ),
                border: outline.border,
                text: s.foreground,
                outline,
            }
        }
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let b = BoxStyle::new(theme, self.checked, self.aria_invalid, self.disabled);
        let mut resting = shadows(b.outline.ring, true);
        if self.disabled {
            resting = dimmed_shadows(resting);
        }
        let checked = self.checked;
        // focus-visible:border-ring, except that data-checked:border-primary comes later.
        let focus = b.outline.focus.map(|(border, ring)| {
            let border = if checked { b.border } else { border };
            (border, shadows(Some(ring), true))
        });

        // peer relative flex size-4 shrink-0 items-center justify-center rounded-sm border
        // border-input shadow-xs data-disabled:opacity-50
        gpui_base::Checkbox::new(self.id)
            .checked(checked)
            .disabled(self.disabled)
            .relative()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(space::S4)
            .rounded(theme.semantic().radius_sm)
            .border_1()
            .border_color(b.border)
            .bg(b.fill)
            .text_color(b.text)
            .shadow(resting)
            .when_some(focus, |c, (border, shadow)| {
                c.focus_visible(move |s| s.border_color(border).shadow(shadow))
            })
            .when_some(self.accessibility_label, |c, label| {
                c.accessibility_label(label)
            })
            .when_some(self.on_change, |c, on_change| {
                c.on_change(move |state, _, window, cx| {
                    on_change(state == CheckboxState::Checked, window, cx)
                })
            })
            // The indicator: [&>svg]:size-3.5, in the box's text colour.
            .when(checked, |c| {
                c.child(Icon::new(IconName::Check).size(px(14.)))
            })
            .refine_style(&self.style)
    }
}

impl_styled!(Checkbox);

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{Modifiers, TestAppContext};

    use super::*;
    use crate::components::test_support::{render, theme};
    use crate::theme::Mode;

    mod style {
        use super::*;

        #[gpui::test]
        fn should_fill_a_checked_box_in_primary(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Light);
            let s = theme.semantic();
            let checked = BoxStyle::new(&theme, true, false, false);
            assert_eq!((checked.fill, checked.border), (s.primary, s.primary));
            assert_eq!(checked.text, s.primary_foreground);
        }

        #[gpui::test]
        fn should_keep_the_primary_border_when_checked_and_invalid(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Light);
            let style = BoxStyle::new(&theme, true, true, false);
            assert_eq!(style.border, theme.semantic().primary);
            assert_eq!(
                style.outline.ring,
                Some(theme.semantic().destructive.opacity(0.2))
            );
        }

        #[gpui::test]
        fn should_fill_an_unchecked_box_with_input_at_30_percent_in_dark(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let light = BoxStyle::new(&theme(cx, Mode::Light), false, false, false);
            let dark = BoxStyle::new(&theme(cx, Mode::Dark), false, false, false);
            let dark_tokens = Mode::Dark.semantic();
            assert_eq!(light.fill, Mode::Light.semantic().background);
            assert_eq!(
                dark.fill,
                dark_tokens.background.blend(dark_tokens.input.opacity(0.3))
            );
        }
    }

    mod disabled {
        use super::*;

        #[gpui::test]
        fn should_dim_a_checked_box_as_one_so_its_border_matches_its_fill(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Light);
            let style = BoxStyle::new(&theme, true, false, true);
            let half_primary = theme
                .semantic()
                .background
                .blend(theme.semantic().primary.opacity(0.5));
            assert_eq!(style.fill, half_primary);
            assert_eq!(style.border, half_primary);
        }
    }

    mod checkbox {
        use super::*;

        #[gpui::test]
        fn should_be_a_16px_square(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || Checkbox::new("c").checked(true));
            assert_eq!(bounds.size, gpui::size(px(16.), px(16.)));
        }

        /// A checkbox whose `checked` lives in a cell its handler writes, as an app's state.
        fn controlled(disabled: bool) -> (impl Fn() -> Checkbox, Rc<Cell<bool>>) {
            let checked = Rc::new(Cell::new(false));
            let make = {
                let checked = checked.clone();
                move || {
                    let write = checked.clone();
                    Checkbox::new("c")
                        .checked(checked.get())
                        .disabled(disabled)
                        .on_change(move |value, _, _| write.set(value))
                }
            };
            (make, checked)
        }

        #[gpui::test]
        fn should_check_then_uncheck_on_clicks(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (checkbox, checked) = controlled(false);
            let (cx, bounds) = render(cx, checkbox);
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert!(checked.get());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert!(!checked.get());
        }

        #[gpui::test]
        fn should_ignore_clicks_while_disabled(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (checkbox, checked) = controlled(true);
            let (cx, bounds) = render(cx, checkbox);
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert!(!checked.get());
        }
    }
}
