//! StateDot: core-react's `state-dot.tsx`, a plain styled element. An agent's state as a 16px
//! mark: a Lucide icon, or an 8px dot, in the state's colour token. `blocked` and
//! `exited-error` share `destructive` on purpose; the shape tells them apart.
//!
//! The styles mirror core-react's cva table arm by arm; each arm names the Tailwind classes it
//! stands for. `working`'s `LoaderCircle` spins (`animate-spin`); Penpot draws it at rest.
//! A turning icon redraws its window, so it steps at a capped frame rate, not every frame,
//! and [`StateDot::spin`] holds it still: an app stops it while its window is inactive.
//!
//! Not mirrored: `aria-hidden` on the icon and the caller's `role="img"` / `aria-label`, since
//! GPUI elements carry no ARIA attributes.

use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{
    App, Hsla, IntoElement, ParentElement as _, RenderOnce, StyleRefinement, Styled, Window, div,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use crate::icons::{Icon, IconName};
use crate::theme::{ActiveTheme as _, Theme};

/// Penpot variant property `state`. Cases are its values in PascalCase; [`Display`] gives the
/// Penpot value back (`StateDotState::ExitedError` → `exited-error`). Required: the library
/// has no `default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum StateDotState {
    /// `working`: a spinning `LoaderCircle` in `info`.
    Working,
    /// `blocked`: the 8px dot in `destructive`.
    Blocked,
    /// `done`: a `CircleCheck` in `success`.
    Done,
    /// `idle`: the 8px dot in `muted-foreground`.
    Idle,
    /// `unknown`: a `Circle` in `muted-foreground`.
    Unknown,
    /// `exited`, with exit code 0 or none read: an `X` in `muted-foreground`.
    Exited,
    /// `exited-error`, any other exit code: an `OctagonX` in `destructive`.
    ExitedError,
}

/// What one `state` arm draws, resolved for the active mode.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Mark {
    /// `None` draws the dot.
    icon: Option<IconName>,
    /// `animate-spin`.
    spins: bool,
    /// The text colour: the icon's stroke, or the dot's fill (`bg-current`).
    color: Hsla,
}

impl StateDotState {
    /// core-react's `state` arms and its `marks` table.
    fn mark(self, theme: &Theme) -> Mark {
        let s = theme.semantic();
        let icon = |icon, color| Mark {
            icon: Some(icon),
            spins: false,
            color,
        };
        let dot = |color| Mark {
            icon: None,
            spins: false,
            color,
        };
        match self {
            // text-info, <LoaderCircleIcon className="size-4 animate-spin" />
            StateDotState::Working => Mark {
                spins: true,
                ..icon(IconName::LoaderCircle, s.info)
            },
            // text-destructive, the dot
            StateDotState::Blocked => dot(s.destructive),
            // text-success, <CircleCheckIcon />
            StateDotState::Done => icon(IconName::CircleCheck, s.success),
            // text-muted-foreground, the dot
            StateDotState::Idle => dot(s.muted_foreground),
            // text-muted-foreground, <CircleIcon />
            StateDotState::Unknown => icon(IconName::Circle, s.muted_foreground),
            // text-muted-foreground, <XIcon />
            StateDotState::Exited => icon(IconName::X, s.muted_foreground),
            // text-destructive, <OctagonXIcon />
            StateDotState::ExitedError => icon(IconName::OctagonX, s.destructive),
        }
    }
}

/// An agent's state as a 16px mark, styled like core-react's `StateDot`. The state is
/// required, so it's the constructor's:
///
/// ```no_run
/// use brrritscold_core_gpui::{StateDot, StateDotState};
///
/// let working = StateDot::new(StateDotState::Working);
/// let failed = StateDot::new(StateDotState::ExitedError);
/// ```
///
/// `working`'s icon turns until [`spin`](Self::spin) says otherwise; hold it still while the
/// window is inactive, so an idle app doesn't keep redrawing:
///
/// ```no_run
/// use brrritscold_core_gpui::{StateDot, StateDotState};
/// # fn mark(window: &gpui::Window) -> StateDot {
/// StateDot::new(StateDotState::Working).spin(window.is_window_active())
/// # }
/// ```
#[derive(IntoElement)]
pub struct StateDot {
    state: StateDotState,
    spin: bool,
    style: StyleRefinement,
}

impl StateDot {
    /// A mark for Penpot property `state`.
    pub fn new(state: StateDotState) -> Self {
        Self {
            state,
            spin: true,
            style: StyleRefinement::default(),
        }
    }

    /// Whether `working`'s `LoaderCircle` turns: `true` by default. `false` draws it at rest
    /// and schedules no redraws. Other states never turn.
    pub fn spin(mut self, spin: bool) -> Self {
        self.spin = spin;
        self
    }
}

impl RenderOnce for StateDot {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mark = self.state.mark(cx.theme());
        let content = match mark.icon {
            // size-4
            Some(name) if mark.spins && self.spin => {
                Icon::new(name).size(space::S4).spin().into_any_element()
            }
            Some(name) => Icon::new(name).size(space::S4).into_any_element(),
            // size-2 rounded-full bg-current
            None => div()
                .size(space::S2)
                .rounded_full()
                .bg(mark.color)
                .into_any_element(),
        };
        // inline-flex size-4 shrink-0 items-center justify-center
        div()
            .flex()
            .size(space::S4)
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .text_color(mark.color)
            .child(content)
            .refine_style(&self.style)
    }
}

impl_styled!(StateDot);

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
            let values: Vec<&'static str> = StateDotState::iter().map(Into::into).collect();
            assert_eq!(
                values,
                [
                    "working",
                    "blocked",
                    "done",
                    "idle",
                    "unknown",
                    "exited",
                    "exited-error"
                ]
            );
        }

        #[gpui::test]
        fn should_draw_each_states_mark_in_its_token_in_both_modes(cx: &mut TestAppContext) {
            cx.update(crate::init);
            for mode in [Mode::Light, Mode::Dark] {
                let s = mode.semantic();
                let theme = theme(cx, mode);
                for (state, icon, color) in [
                    (StateDotState::Working, Some(IconName::LoaderCircle), s.info),
                    (StateDotState::Blocked, None, s.destructive),
                    (StateDotState::Done, Some(IconName::CircleCheck), s.success),
                    (StateDotState::Idle, None, s.muted_foreground),
                    (
                        StateDotState::Unknown,
                        Some(IconName::Circle),
                        s.muted_foreground,
                    ),
                    (StateDotState::Exited, Some(IconName::X), s.muted_foreground),
                    (
                        StateDotState::ExitedError,
                        Some(IconName::OctagonX),
                        s.destructive,
                    ),
                ] {
                    let mark = state.mark(&theme);
                    assert_eq!(mark.icon, icon, "{state} {mode:?}");
                    assert_eq!(mark.color, color, "{state} {mode:?}");
                }
            }
        }

        #[gpui::test]
        fn should_spin_only_working(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Light);
            for state in StateDotState::iter() {
                let spins = state.mark(&theme).spins;
                assert_eq!(spins, state == StateDotState::Working, "{state}");
            }
        }
    }

    mod spin {
        use super::*;
        use crate::components::test_support::redraws_in_one_second;

        #[gpui::test]
        fn should_turn_by_default(cx: &mut TestAppContext) {
            cx.update(crate::init);
            assert!(redraws_in_one_second(cx, || StateDot::new(StateDotState::Working)) > 0);
        }

        #[gpui::test]
        fn should_hold_still_without_redrawing_when_off(cx: &mut TestAppContext) {
            cx.update(crate::init);
            assert_eq!(
                redraws_in_one_second(cx, || StateDot::new(StateDotState::Working).spin(false)),
                0
            );
        }

        #[gpui::test]
        fn should_keep_its_size_when_off(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, turning) = render(cx, || StateDot::new(StateDotState::Working));
            let (_, still) = render(cx, || StateDot::new(StateDotState::Working).spin(false));
            assert_eq!(still.size, turning.size);
        }
    }

    mod layout {
        use super::*;

        #[gpui::test]
        fn should_be_16px_square_for_every_state(cx: &mut TestAppContext) {
            cx.update(crate::init);
            for state in StateDotState::iter() {
                let (_, bounds) = render(cx, move || StateDot::new(state));
                assert_eq!(bounds.size.width, px(16.), "{state}");
                assert_eq!(bounds.size.height, px(16.), "{state}");
            }
        }

        #[gpui::test]
        fn should_take_a_callers_style_over_its_own(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || StateDot::new(StateDotState::Idle).size(px(24.)));
            assert_eq!(bounds.size.width, px(24.));
        }
    }
}
