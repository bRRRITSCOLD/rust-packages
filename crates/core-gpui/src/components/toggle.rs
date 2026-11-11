//! Toggle: core-react's `toggle.tsx` on `gpui-base`'s unstyled [`gpui_base::Toggle`], which
//! owns focus, Space and click activation, disabled inertness and accessibility.
//!
//! The toggle is controlled: it shows [`Toggle::pressed`] and reports the next value to
//! [`Toggle::on_change`], as core-react's `pressed` / `onPressedChange` do. The styles mirror
//! core-react's `toggleVariants` arm by arm; [`crate::ToggleGroupItem`] draws through them too.
//!
//! Not mirrored:
//!
//! - The `aria-invalid` arms: not a Penpot variant property of Toggle.
//! - `has-data-[icon=inline-end]:pr-*`: a toggle draws its icon before its label only.

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, opacity, space};
use gpui::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, Pixels, RenderOnce,
    SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use super::control::{dimmed, dimmed_shadows, shadows};
use crate::icons::{Icon, IconName};
use crate::theme::ActiveTheme as _;

/// Penpot variant property `variant`. Cases are its values in PascalCase; [`Display`] gives
/// the Penpot value back (`ToggleVariant::Outline` → `outline`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum ToggleVariant {
    /// `default`: no border, filled only when pressed or hovered.
    #[default]
    Default,
    /// `outline`: an `input` border and `shadow-xs`.
    Outline,
}

/// Penpot variant property `size`. Cases are its values in PascalCase; [`Display`] gives the
/// Penpot value back (`ToggleSize::Lg` → `lg`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum ToggleSize {
    /// `default`: 36px tall.
    #[default]
    Default,
    /// `sm`: 32px tall.
    Sm,
    /// `lg`: 40px tall.
    Lg,
}

/// The geometry of one `size` arm.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SizeStyle {
    /// `h-*`, and the `min-w-*` that keeps an icon-only toggle square.
    height: Pixels,
    /// `has-data-[icon=inline-start]:pl-*`: the left padding when an icon leads a label.
    padding_left_with_icon: Pixels,
}

impl ToggleSize {
    fn style(self) -> SizeStyle {
        match self {
            // h-9 min-w-9 px-2.5 has-data-[icon=inline-start]:pl-2
            ToggleSize::Default => SizeStyle {
                height: space::S9,
                padding_left_with_icon: space::S2,
            },
            // h-8 min-w-8 px-2.5 has-data-[icon=inline-start]:pl-1.5
            ToggleSize::Sm => SizeStyle {
                height: space::S8,
                padding_left_with_icon: space::S1_5,
            },
            // h-10 min-w-10 px-2.5 has-data-[icon=inline-start]:pl-2
            ToggleSize::Lg => SizeStyle {
                height: space::S10,
                padding_left_with_icon: space::S2,
            },
        }
    }
}

/// Where a toggle sits in a `spacing` 0 [`crate::ToggleGroup`], which joins its items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Joined {
    pub(crate) vertical: bool,
    pub(crate) first: bool,
    pub(crate) last: bool,
}

type ChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A toggle button, styled like core-react's `Toggle`. One builder per Penpot variant
/// property ([`variant`](Self::variant), [`size`](Self::size), [`pressed`](Self::pressed),
/// [`disabled`](Self::disabled)) and one per part ([`label`](Self::label) for the `Label`
/// layer, [`icon`](Self::icon) for an icon):
///
/// ```no_run
/// use brrritscold_core_gpui::{IconName, Toggle, ToggleVariant};
///
/// let bold = Toggle::new("bold")
///     .variant(ToggleVariant::Outline)
///     .icon(IconName::LetterB)
///     .accessibility_label("Bold")
///     .pressed(true)
///     .on_change(|pressed, _, _| println!("bold: {pressed}"));
/// ```
#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    variant: ToggleVariant,
    size: ToggleSize,
    pressed: bool,
    disabled: bool,
    label: Option<SharedString>,
    icon: Option<IconName>,
    accessibility_label: Option<SharedString>,
    on_change: Option<ChangeHandler>,
    joined: Option<Joined>,
    style: StyleRefinement,
}

impl Toggle {
    /// A `default` / `default`, unpressed toggle with no content. `id` keys its focus state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            pressed: false,
            disabled: false,
            label: None,
            icon: None,
            accessibility_label: None,
            on_change: None,
            joined: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `variant`.
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Penpot property `size`.
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    /// Penpot property `pressed`: filled in `muted`.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    /// Penpot property `disabled`: dims the toggle to 50% and makes it inert.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The `Label` layer: the toggle's text.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// An icon, drawn 16px before the label in the toggle's text colour.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// The name assistive technology reads, for an icon-only toggle.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Runs on a click or Space with the pressed value the toggle asks to change to. Never
    /// while disabled.
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Draws the toggle as an item of a `spacing` 0 group.
    pub(crate) fn joined(mut self, joined: Joined) -> Self {
        self.joined = Some(joined);
        self
    }
}

impl RenderOnce for Toggle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let s = theme.semantic();
        let z = self.size.style();
        let outline = self.variant == ToggleVariant::Outline;
        let enabled = !self.disabled;
        let radius = s.radius_md; // rounded-md
        let icon_leads_label = self.icon.is_some() && self.label.is_some();
        let focus_border = s.ring;
        let focus_ring = shadows(Some(theme.focus_ring()), outline && self.joined.is_none());
        let (muted, foreground) = (s.muted, s.foreground);
        // bg-transparent, or aria-pressed:bg-muted. A transparent toggle under a ring or
        // shadow-xs takes `background` instead, which GPUI would otherwise paint the shadow
        // through (see `control`).
        let fill = if self.pressed { muted } else { s.background };
        let filled = self.pressed || outline;
        let mut border = s.input;
        let mut resting = shadows(None, outline && self.joined.is_none());
        let mut resting_fill = fill;
        if self.disabled {
            // disabled:opacity-50, layer by layer (see `control::dimmed`): the border over the
            // fill, the content through an opacity of its own.
            border = dimmed(theme, fill.blend(border));
            resting_fill = dimmed(theme, fill);
            resting = dimmed_shadows(resting);
        }

        // group/toggle inline-flex items-center justify-center gap-1 rounded-md text-sm
        // font-medium whitespace-nowrap hover:bg-muted hover:text-foreground
        // focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50
        // disabled:pointer-events-none disabled:opacity-50 aria-pressed:bg-muted
        gpui_base::Toggle::new(self.id)
            .pressed(self.pressed)
            .disabled(self.disabled)
            .flex_shrink_0()
            .h(z.height)
            .min_w(z.height)
            .text_size(font_size::SM)
            .font_weight(font_weight::MEDIUM)
            .whitespace_nowrap()
            .map(|t| match self.joined {
                None => t
                    .rounded(radius)
                    .px(space::S2_5)
                    .when(icon_leads_label, |t| t.pl(z.padding_left_with_icon)),
                // group-data-[spacing=0]/toggle-group:rounded-none …:px-2 …:shadow-none
                // …:has-data-[icon=inline-start]:pl-1.5, then first: / last: round the ends.
                Some(j) => t
                    .px(space::S2)
                    .when(icon_leads_label, |t| t.pl(space::S1_5))
                    .when(j.first, |t| {
                        if j.vertical {
                            t.rounded_t(radius)
                        } else {
                            t.rounded_l(radius)
                        }
                    })
                    .when(j.last, |t| {
                        if j.vertical {
                            t.rounded_b(radius)
                        } else {
                            t.rounded_r(radius)
                        }
                    }),
            })
            // outline: border border-input bg-transparent shadow-xs hover:bg-muted
            .when(outline, |t| {
                t.border_1()
                    .border_color(border)
                    .shadow(resting)
                    // …:data-[variant=outline]:border-l-0 (border-t-0 vertical), first: keeps it
                    .when_some(self.joined.filter(|j| !j.first), |t, j| {
                        if j.vertical {
                            t.border_t_0()
                        } else {
                            t.border_l_0()
                        }
                    })
            })
            .when(filled, |t| t.bg(resting_fill))
            .when(enabled, |t| {
                t.hover(move |h| h.bg(muted).text_color(foreground))
            })
            .focus_visible(move |f| f.bg(fill).border_color(focus_border).shadow(focus_ring))
            .when_some(self.accessibility_label, |t, label| {
                t.accessibility_label(label)
            })
            .when_some(self.on_change, |t, on_change| {
                t.on_change(move |pressed, _, window, cx| on_change(pressed, window, cx))
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(space::S1)
                    .when(self.disabled, |c| c.opacity(opacity::O50))
                    // [&_svg:not([class*='size-'])]:size-4
                    .children(self.icon.map(|icon| Icon::new(icon).size(space::S4)))
                    .children(self.label),
            )
            .refine_style(&self.style)
    }
}

impl_styled!(Toggle);

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{Modifiers, TestAppContext, px, size};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::render;

    mod values {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let variants: Vec<&'static str> = ToggleVariant::iter().map(Into::into).collect();
            let sizes: Vec<&'static str> = ToggleSize::iter().map(Into::into).collect();
            assert_eq!(variants, ["default", "outline"]);
            assert_eq!(sizes, ["default", "sm", "lg"]);
        }
    }

    mod size {
        use super::*;

        #[gpui::test]
        fn should_size_an_icon_only_toggle_by_its_height_and_padding(cx: &mut TestAppContext) {
            cx.update(crate::init);
            // px-2.5 around a 16px icon is 36px wide, which `min-w-*` widens to the height.
            for (toggle_size, width, height) in [
                (ToggleSize::Default, 36., 36.),
                (ToggleSize::Sm, 36., 32.),
                (ToggleSize::Lg, 40., 40.),
            ] {
                let (_, bounds) = render(cx, move || {
                    Toggle::new("t").size(toggle_size).icon(IconName::LetterB)
                });
                assert_eq!(bounds.size, size(px(width), px(height)), "{toggle_size}");
            }
        }
    }

    mod press {
        use super::*;

        fn controlled(disabled: bool) -> (impl Fn() -> Toggle, Rc<Cell<bool>>) {
            let pressed = Rc::new(Cell::new(false));
            let make = {
                let pressed = pressed.clone();
                move || {
                    let write = pressed.clone();
                    Toggle::new("t")
                        .icon(IconName::LetterB)
                        .pressed(pressed.get())
                        .disabled(disabled)
                        .on_change(move |value, _, _| write.set(value))
                }
            };
            (make, pressed)
        }

        #[gpui::test]
        fn should_press_then_release_on_clicks(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (toggle, pressed) = controlled(false);
            let (cx, bounds) = render(cx, toggle);
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert!(pressed.get());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert!(!pressed.get());
        }

        #[gpui::test]
        fn should_ignore_clicks_while_disabled(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (toggle, pressed) = controlled(true);
            let (cx, bounds) = render(cx, toggle);
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert!(!pressed.get());
        }
    }
}
