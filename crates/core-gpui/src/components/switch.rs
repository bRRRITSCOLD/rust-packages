//! Switch: core-react's `switch.tsx` on `gpui-base`'s unstyled [`gpui_base::Switch`], which
//! owns focus, Space and click activation, disabled inertness and accessibility.
//!
//! The switch is controlled: it shows [`Switch::checked`] and reports the next value to
//! [`Switch::on_change`], as core-react's `checked` / `onCheckedChange` do. The thumb's
//! `translate-x-*` is a left margin.
//!
//! Not mirrored:
//!
//! - `after:-inset-x-3 after:-inset-y-2`: the hit area stays the track.
//! - The `aria-invalid` arms: not a Penpot variant property of Switch.
//! - The `group-has-…/field-label` arms, CSS relations GPUI has no equivalent of.
//! - `transition-all` / `transition-transform`: the thumb moves without animating.

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::{border_radius, space};
use gpui::{
    App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    RenderOnce, SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
    transparent_black,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use super::control::{dimmed, dimmed_shadows, on_background, shadows};
use crate::theme::{ActiveTheme as _, Theme};

/// Penpot variant property `size`. Cases are its values in PascalCase; [`Display`] gives the
/// Penpot value back (`SwitchSize::Sm` → `sm`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SwitchSize {
    /// `default`: a 32×18 track.
    #[default]
    Default,
    /// `sm`: a 24×14 track.
    Sm,
}

/// The geometry of one `size` arm.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SizeStyle {
    width: Pixels,
    height: Pixels,
    thumb: Pixels,
    /// How far the thumb moves when checked.
    travel: Pixels,
}

impl SwitchSize {
    fn style(self) -> SizeStyle {
        match self {
            // data-[size=default]:h-4.5 data-[size=default]:w-8
            // group-data-[size=default]/switch:size-4 …:data-checked:translate-x-3.5
            SwitchSize::Default => SizeStyle {
                width: space::S8,
                height: space::S4_5,
                thumb: space::S4,
                travel: space::S3_5,
            },
            // data-[size=sm]:h-3.5 data-[size=sm]:w-6
            // group-data-[size=sm]/switch:size-3 …:data-checked:translate-x-2.5
            SwitchSize::Sm => SizeStyle {
                width: space::S6,
                height: space::S3_5,
                thumb: space::S3,
                travel: space::S2_5,
            },
        }
    }
}

type ChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A switch, styled like core-react's `Switch`. One builder per Penpot variant property
/// ([`size`](Self::size), [`checked`](Self::checked), [`disabled`](Self::disabled)):
///
/// ```no_run
/// use brrritscold_core_gpui::{Switch, SwitchSize};
///
/// let airplane = Switch::new("airplane")
///     .size(SwitchSize::Sm)
///     .checked(true)
///     .accessibility_label("Airplane mode")
///     .on_change(|on, _, _| println!("airplane mode: {on}"));
/// ```
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    size: SwitchSize,
    checked: bool,
    disabled: bool,
    accessibility_label: Option<SharedString>,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
}

impl Switch {
    /// A `default`, unchecked switch. `id` keys its focus state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            size: SwitchSize::default(),
            checked: false,
            disabled: false,
            accessibility_label: None,
            on_change: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `size`.
    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    /// Penpot property `checked`: a `primary` track with the thumb at the end.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Penpot property `disabled`: dims the switch to 50% and makes it inert.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The name assistive technology reads, for a switch drawn without text beside it.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Runs on a click or Space with the value the switch asks to change to. Never while
    /// disabled.
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

/// The track and thumb colours for one state, resolved for the active mode.
#[derive(Debug, Clone, Copy, PartialEq)]
struct StateStyle {
    track: Hsla,
    thumb: Hsla,
}

impl StateStyle {
    fn new(theme: &Theme, checked: bool) -> Self {
        let s = theme.semantic();
        let m = theme.mode();
        if checked {
            // data-checked:bg-primary; thumb bg-background dark:data-checked:bg-primary-foreground
            Self {
                track: s.primary,
                thumb: m.pick(s.background, s.primary_foreground),
            }
        } else {
            // data-unchecked:bg-input dark:data-unchecked:bg-input/80;
            // thumb bg-background dark:data-unchecked:bg-foreground
            Self {
                // made opaque: the track sits on shadow-xs
                track: on_background(theme, m.pick(s.input, s.input.opacity(0.8))),
                thumb: m.pick(s.background, s.foreground),
            }
        }
    }
}

impl RenderOnce for Switch {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let mut colors = StateStyle::new(theme, self.checked);
        let mut resting = shadows(None, true);
        if self.disabled {
            // data-disabled:opacity-50, layer by layer: the thumb over the track.
            colors = StateStyle {
                track: dimmed(theme, colors.track),
                thumb: dimmed(theme, colors.thumb),
            };
            resting = dimmed_shadows(resting);
        }
        let z = self.size.style();
        let focus_ring = shadows(Some(theme.focus_ring()), true);
        let focus_border = theme.semantic().ring;

        // peer group/switch relative inline-flex shrink-0 items-center rounded-full border
        // border-transparent shadow-xs focus-visible:border-ring focus-visible:ring-3
        // focus-visible:ring-ring/50 data-disabled:opacity-50
        gpui_base::Switch::new(self.id)
            .checked(self.checked)
            .disabled(self.disabled)
            .relative()
            .flex()
            .flex_shrink_0()
            .items_center()
            .w(z.width)
            .h(z.height)
            .rounded(border_radius::FULL)
            .border_1()
            .border_color(transparent_black())
            .bg(colors.track)
            .shadow(resting)
            .focus_visible(move |s| s.border_color(focus_border).shadow(focus_ring))
            .when_some(self.accessibility_label, |s, label| {
                s.accessibility_label(label)
            })
            .when_some(self.on_change, |s, on_change| {
                s.on_change(move |checked, _, window, cx| on_change(checked, window, cx))
            })
            // The thumb: pointer-events-none block rounded-full bg-background
            .child(
                div()
                    .flex_shrink_0()
                    .size(z.thumb)
                    .rounded(border_radius::FULL)
                    .bg(colors.thumb)
                    .when(self.checked, |t| t.ml(z.travel)),
            )
            .refine_style(&self.style)
    }
}

impl_styled!(Switch);

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{Modifiers, TestAppContext, px, size};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::{render, theme};
    use crate::theme::Mode;

    mod size {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&'static str> = SwitchSize::iter().map(Into::into).collect();
            assert_eq!(values, ["default", "sm"]);
        }

        #[gpui::test]
        fn should_lay_the_tracks_out_at_32_by_18_and_24_by_14(cx: &mut TestAppContext) {
            cx.update(crate::init);
            for (switch_size, width, height) in
                [(SwitchSize::Default, 32., 18.), (SwitchSize::Sm, 24., 14.)]
            {
                let (_, bounds) = render(cx, move || Switch::new("s").size(switch_size));
                assert_eq!(bounds.size, size(px(width), px(height)), "{switch_size}");
            }
        }

        #[test]
        fn should_carry_the_thumb_to_the_far_edge_when_checked() {
            for switch_size in SwitchSize::iter() {
                let z = switch_size.style();
                // The thumb ends 1px (the transparent border) inside the track.
                assert_eq!(z.travel + z.thumb + px(2.), z.width, "{switch_size}");
            }
        }
    }

    mod state {
        use super::*;

        #[gpui::test]
        fn should_swap_the_thumb_colours_in_dark(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let dark = Mode::Dark.semantic();
            let on = StateStyle::new(&theme(cx, Mode::Dark), true);
            let off = StateStyle::new(&theme(cx, Mode::Dark), false);
            assert_eq!(on.thumb, dark.primary_foreground);
            assert_eq!(off.thumb, dark.foreground);
            assert_eq!(off.track, dark.background.blend(dark.input.opacity(0.8)));
        }
    }

    mod toggle {
        use super::*;

        fn controlled(disabled: bool) -> (impl Fn() -> Switch, Rc<Cell<bool>>) {
            let checked = Rc::new(Cell::new(false));
            let make = {
                let checked = checked.clone();
                move || {
                    let write = checked.clone();
                    Switch::new("s")
                        .checked(checked.get())
                        .disabled(disabled)
                        .on_change(move |value, _, _| write.set(value))
                }
            };
            (make, checked)
        }

        #[gpui::test]
        fn should_turn_on_then_off_on_clicks(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (switch, checked) = controlled(false);
            let (cx, bounds) = render(cx, switch);
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert!(checked.get());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert!(!checked.get());
        }

        #[gpui::test]
        fn should_ignore_clicks_while_disabled(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (switch, checked) = controlled(true);
            let (cx, bounds) = render(cx, switch);
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert!(!checked.get());
        }
    }
}
