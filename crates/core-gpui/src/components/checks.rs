//! Checks: core-react's `checks.tsx`, a plain styled element. A PR's checks as a 14px Lucide
//! icon and an optional label, both in the state's colour token.
//!
//! The styles mirror core-react's cva table arm by arm; each arm names the Tailwind classes it
//! stands for. `pending`'s `LoaderCircle` spins (`animate-spin`); Penpot draws it at rest.
//! A turning icon redraws its window, so it steps at a capped frame rate, not every frame,
//! and [`Checks::spin`] holds it still: an app stops it while its window is inactive.
//! The label inherits the root surface's line height (1.5, an 18px row), as core-react's does;
//! Penpot's `Label` keeps its 1.2.
//!
//! Not mirrored: `aria-hidden` on the icon and the caller's `role="img"` / `aria-label`, since
//! GPUI elements carry no ARIA attributes.

use brrritscold_core_gpui_tokens::primitive::{font_size, space};
use gpui::{
    App, Hsla, IntoElement, ParentElement as _, RenderOnce, SharedString, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder as _,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use crate::icons::{Icon, IconName};
use crate::theme::{ActiveTheme as _, Theme};

/// Penpot variant property `state`, as GitHub's checks rollup names it. Cases are its values in
/// PascalCase; [`Display`] gives the Penpot value back. Required: the library has no `default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum ChecksState {
    /// `success`: a `CircleCheck` in `success`.
    Success,
    /// `pending`: a spinning `LoaderCircle` in `info`.
    Pending,
    /// `failure`: an `OctagonX` in `destructive`.
    Failure,
}

/// What one `state` arm draws, resolved for the active mode.
#[derive(Debug, Clone, Copy, PartialEq)]
struct StateStyle {
    icon: IconName,
    /// `animate-spin`.
    spins: bool,
    /// The text colour: the icon's stroke and the label's fill.
    color: Hsla,
}

impl ChecksState {
    /// core-react's `state` arms and its `icons` table.
    fn style(self, theme: &Theme) -> StateStyle {
        let s = theme.semantic();
        let style = |icon, color| StateStyle {
            icon,
            spins: false,
            color,
        };
        match self {
            // text-success, <CircleCheckIcon className="size-3.5" />
            ChecksState::Success => style(IconName::CircleCheck, s.success),
            // text-info, <LoaderCircleIcon className="size-3.5 animate-spin" />
            ChecksState::Pending => StateStyle {
                spins: true,
                ..style(IconName::LoaderCircle, s.info)
            },
            // text-destructive, <OctagonXIcon className="size-3.5" />
            ChecksState::Failure => style(IconName::OctagonX, s.destructive),
        }
    }
}

/// A PR's checks, styled like core-react's `Checks`. The state is required, so it's the
/// constructor's; [`label`](Self::label) is the `Label` layer, and without one it's the icon
/// alone:
///
/// ```no_run
/// use brrritscold_core_gpui::{Checks, ChecksState};
///
/// let passing = Checks::new(ChecksState::Success).label("passing");
/// let icon_only = Checks::new(ChecksState::Pending);
/// ```
///
/// `pending`'s icon turns until [`spin`](Self::spin) says otherwise; hold it still while the
/// window is inactive, so an idle app doesn't keep redrawing:
///
/// ```no_run
/// use brrritscold_core_gpui::{Checks, ChecksState};
/// # fn checks(window: &gpui::Window) -> Checks {
/// Checks::new(ChecksState::Pending).spin(window.is_window_active())
/// # }
/// ```
#[derive(IntoElement)]
pub struct Checks {
    state: ChecksState,
    label: Option<SharedString>,
    spin: bool,
    style: StyleRefinement,
}

impl Checks {
    /// Checks for Penpot property `state`, with no label.
    pub fn new(state: ChecksState) -> Self {
        Self {
            state,
            label: None,
            spin: true,
            style: StyleRefinement::default(),
        }
    }

    /// The `Label` layer: the word beside the icon (core-react's children).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Whether `pending`'s `LoaderCircle` turns: `true` by default. `false` draws it at rest
    /// and schedules no redraws. Other states never turn.
    pub fn spin(mut self, spin: bool) -> Self {
        self.spin = spin;
        self
    }
}

impl RenderOnce for Checks {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let v = self.state.style(cx.theme());
        // size-3.5
        let icon = Icon::new(v.icon).size(space::S3_5);
        let icon = if v.spins && self.spin {
            icon.spin().into_any_element()
        } else {
            icon.into_any_element()
        };
        // inline-flex items-center gap-1 text-xs
        div()
            .flex()
            .items_center()
            .gap(space::S1)
            .text_size(font_size::XS)
            .text_color(v.color)
            .child(icon)
            .when_some(self.label, |d, label| d.child(label))
            .refine_style(&self.style)
    }
}

impl_styled!(Checks);

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, px};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::{render, theme};
    use crate::theme::Mode;

    mod state {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&'static str> = ChecksState::iter().map(Into::into).collect();
            assert_eq!(values, ["success", "pending", "failure"]);
        }

        #[gpui::test]
        fn should_draw_each_states_icon_in_its_token_in_both_modes(cx: &mut TestAppContext) {
            cx.update(crate::init);
            for mode in [Mode::Light, Mode::Dark] {
                let s = mode.semantic();
                let theme = theme(cx, mode);
                for (state, icon, color) in [
                    (ChecksState::Success, IconName::CircleCheck, s.success),
                    (ChecksState::Pending, IconName::LoaderCircle, s.info),
                    (ChecksState::Failure, IconName::OctagonX, s.destructive),
                ] {
                    let style = state.style(&theme);
                    assert_eq!(style.icon, icon, "{state} {mode:?}");
                    assert_eq!(style.color, color, "{state} {mode:?}");
                }
            }
        }

        #[gpui::test]
        fn should_spin_only_pending(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Light);
            for state in ChecksState::iter() {
                let spins = state.style(&theme).spins;
                assert_eq!(spins, state == ChecksState::Pending, "{state}");
            }
        }
    }

    mod spin {
        use super::*;
        use crate::components::test_support::redraws_in_one_second;

        #[gpui::test]
        fn should_turn_by_default(cx: &mut TestAppContext) {
            cx.update(crate::init);
            assert!(redraws_in_one_second(cx, || Checks::new(ChecksState::Pending)) > 0);
        }

        #[gpui::test]
        fn should_hold_still_without_redrawing_when_off(cx: &mut TestAppContext) {
            cx.update(crate::init);
            assert_eq!(
                redraws_in_one_second(cx, || Checks::new(ChecksState::Pending).spin(false)),
                0
            );
        }

        #[gpui::test]
        fn should_keep_its_size_when_off(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, turning) = render(cx, || Checks::new(ChecksState::Pending));
            let (_, still) = render(cx, || Checks::new(ChecksState::Pending).spin(false));
            assert_eq!(still.size, turning.size);
        }
    }

    mod label {
        use super::*;

        #[gpui::test]
        fn should_be_the_14px_icon_alone_without_one(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || Checks::new(ChecksState::Success));
            assert_eq!(bounds.size.width, px(14.));
        }

        #[gpui::test]
        fn should_follow_the_icon_4px_apart_on_an_18px_row(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || Checks::new(ChecksState::Success).label("passing"));
            // the 14px icon, gap-1, then the word; text-xs on the root's 1.5 line height
            assert!(bounds.size.width > px(14.) + space::S1, "{bounds:?}");
            assert_eq!(bounds.size.height, px(18.));
        }
    }
}
