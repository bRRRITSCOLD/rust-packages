//! Button: core-react's `button.tsx` on `gpui-base`'s unstyled [`gpui_base::Button`], which
//! owns focus, keyboard activation, disabled inertness and accessibility.
//!
//! The styles mirror core-react's cva table arm by arm; each arm names the Tailwind classes
//! it stands for. Not mirrored, because GPUI or the Penpot library can't carry them yet:
//!
//! - `active:translate-y-px`: a `Div` has no transform.
//! - `underline-offset-4`: GPUI draws underlines at a fixed offset.
//! - The `aria-invalid` and `aria-expanded` arms: neither is a Penpot variant property, so
//!   the Button has no builder for them until one is.

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, opacity, shadow, space};
use gpui::{
    App, ClickEvent, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, Rems, RenderOnce, SharedString, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _, px, transparent_black,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use crate::icons::{Icon, IconName};
use crate::theme::{ActiveTheme as _, Theme};

/// Penpot variant property `variant`. Cases are its values in PascalCase; [`Display`] gives
/// the Penpot value back (`ButtonVariant::Outline` → `outline`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum ButtonVariant {
    /// `default`: the primary action.
    #[default]
    Default,
    /// `outline`
    Outline,
    /// `secondary`
    Secondary,
    /// `ghost`
    Ghost,
    /// `destructive`
    Destructive,
    /// `link`
    Link,
}

/// Penpot variant property `size`. Cases are its values in PascalCase; [`Display`] gives the
/// Penpot value back (`ButtonSize::IconSm` → `icon-sm`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum ButtonSize {
    /// `default`
    #[default]
    Default,
    /// `xs`
    Xs,
    /// `sm`
    Sm,
    /// `lg`
    Lg,
    /// `icon`: a square for an icon-only button.
    Icon,
    /// `icon-xs`
    IconXs,
    /// `icon-sm`
    IconSm,
    /// `icon-lg`
    IconLg,
}

impl ButtonSize {
    /// Whether this is a square, icon-only size (`icon`, `icon-xs`, …).
    pub fn is_icon(self) -> bool {
        matches!(
            self,
            ButtonSize::Icon | ButtonSize::IconXs | ButtonSize::IconSm | ButtonSize::IconLg
        )
    }
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// A button, styled like core-react's `Button`.
///
/// One builder per Penpot variant property ([`variant`](Self::variant),
/// [`size`](Self::size)) and one per part ([`label`](Self::label) for the `Label` layer,
/// [`icon`](Self::icon) for the `Icon` part):
///
/// ```no_run
/// use brrritscold_core_gpui::{Button, ButtonSize, ButtonVariant, IconName};
///
/// let save = Button::new("save").label("Save");
/// let add = Button::new("add")
///     .variant(ButtonVariant::Outline)
///     .size(ButtonSize::IconSm)
///     .icon(IconName::Plus)
///     .on_click(|_, _, _| println!("add"));
/// ```
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    variant: ButtonVariant,
    size: ButtonSize,
    label: Option<SharedString>,
    icon: Option<IconName>,
    icon_end: Option<IconName>,
    disabled: bool,
    on_click: Option<ClickHandler>,
    style: StyleRefinement,
}

impl Button {
    /// A `default` / `default` button with no content. `id` keys its focus state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            label: None,
            icon: None,
            icon_end: None,
            disabled: false,
            on_click: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `variant`.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Penpot property `size`.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// The `Label` layer: the button's text.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The `Icon` part, drawn before the label in the button's text colour.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Dims the button to 50% and makes it inert (`disabled:opacity-50
    /// disabled:pointer-events-none`); it also leaves the Tab order.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Runs on a pointer click, Enter or Space. Never while disabled.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

/// The colours of one `variant` arm, resolved for the active mode.
#[derive(Debug, Clone, Copy, PartialEq)]
struct VariantStyle {
    fill: Option<Hsla>,
    text: Hsla,
    /// `None` keeps the base `border-transparent`.
    border: Option<Hsla>,
    shadow_xs: bool,
    hover_fill: Option<Hsla>,
    hover_underline: bool,
    focus_border: Hsla,
    focus_ring: Hsla,
}

impl ButtonVariant {
    /// core-react's `variant` arms. Dark-only arms go through [`crate::Mode::pick`].
    fn style(self, theme: &Theme) -> VariantStyle {
        let s = theme.semantic();
        let m = theme.mode();
        // Base: focus-visible:border-ring focus-visible:ring-ring/50
        let base = VariantStyle {
            fill: None,
            text: s.foreground,
            border: None,
            shadow_xs: false,
            hover_fill: None,
            hover_underline: false,
            focus_border: s.ring,
            focus_ring: theme.focus_ring(),
        };
        match self {
            // bg-primary text-primary-foreground hover:bg-primary/80
            ButtonVariant::Default => VariantStyle {
                fill: Some(s.primary),
                text: s.primary_foreground,
                hover_fill: Some(s.primary.opacity(0.8)),
                ..base
            },
            // border-border bg-background shadow-xs hover:bg-muted hover:text-foreground
            ButtonVariant::Outline => VariantStyle {
                fill: Some(m.pick(s.background, s.input.opacity(0.3))), // dark:bg-input/30
                border: Some(m.pick(s.border, s.input)),                // dark:border-input
                shadow_xs: true,
                hover_fill: Some(m.pick(s.muted, s.input.opacity(0.5))), // dark:hover:bg-input/50
                ..base
            },
            // bg-secondary text-secondary-foreground hover:bg-secondary/80
            ButtonVariant::Secondary => VariantStyle {
                fill: Some(s.secondary),
                text: s.secondary_foreground,
                hover_fill: Some(s.secondary.opacity(0.8)),
                ..base
            },
            // hover:bg-muted hover:text-foreground
            ButtonVariant::Ghost => VariantStyle {
                hover_fill: Some(m.pick(s.muted, s.muted.opacity(0.5))), // dark:hover:bg-muted/50
                ..base
            },
            // bg-destructive/10 text-destructive hover:bg-destructive/20
            // focus-visible:border-destructive/40 focus-visible:ring-destructive/20
            ButtonVariant::Destructive => VariantStyle {
                fill: Some(s.destructive.opacity(m.pick(opacity::O10, 0.2))), // dark:bg-destructive/20
                text: s.destructive,
                hover_fill: Some(s.destructive.opacity(m.pick(0.2, 0.3))), // dark:hover:bg-destructive/30
                focus_border: s.destructive.opacity(0.4),
                focus_ring: s.destructive.opacity(m.pick(0.2, 0.4)), // dark:focus-visible:ring-destructive/40
                ..base
            },
            // text-primary underline-offset-4 hover:underline
            ButtonVariant::Link => VariantStyle {
                text: s.primary,
                hover_underline: true,
                ..base
            },
        }
    }
}

/// The geometry of one `size` arm.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SizeStyle {
    /// `h-*`, or both sides of a square `size-*`.
    height: Pixels,
    square: bool,
    padding_x: Pixels,
    /// `has-data-[icon=inline-start]:pl-*`: the left padding when an icon leads a label.
    padding_left_with_icon: Pixels,
    /// `has-data-[icon=inline-end]:pr-*`: the right padding when an icon trails a label.
    padding_right_with_icon: Pixels,
    gap: Pixels,
    text: Rems,
    /// `[&_svg:not([class*='size-'])]:size-*`
    icon: Pixels,
}

impl ButtonSize {
    /// core-react's `size` arms, over the base `text-sm` and `size-4` icons.
    fn style(self) -> SizeStyle {
        let base = SizeStyle {
            height: space::S9,
            square: false,
            padding_x: space::S2_5,
            padding_left_with_icon: space::S2,
            padding_right_with_icon: space::S2,
            gap: space::S1_5,
            text: font_size::SM,
            icon: space::S4,
        };
        let square = |side| SizeStyle {
            height: side,
            square: true,
            padding_x: px(0.),
            padding_left_with_icon: px(0.),
            padding_right_with_icon: px(0.),
            gap: px(0.),
            ..base
        };
        match self {
            // h-9 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2
            ButtonSize::Default => base,
            // h-6 gap-1 px-2 text-xs has-data-[icon=inline-end]:pr-1.5
            // has-data-[icon=inline-start]:pl-1.5 [&_svg]:size-3
            ButtonSize::Xs => SizeStyle {
                height: space::S6,
                padding_x: space::S2,
                padding_left_with_icon: space::S1_5,
                padding_right_with_icon: space::S1_5,
                gap: space::S1,
                text: font_size::XS,
                icon: space::S3,
                ..base
            },
            // h-8 gap-1 px-2.5 has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5
            ButtonSize::Sm => SizeStyle {
                height: space::S8,
                padding_left_with_icon: space::S1_5,
                padding_right_with_icon: space::S1_5,
                gap: space::S1,
                ..base
            },
            // h-10 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2
            ButtonSize::Lg => SizeStyle {
                height: space::S10,
                ..base
            },
            // size-9
            ButtonSize::Icon => square(space::S9),
            // size-6 [&_svg]:size-3
            ButtonSize::IconXs => SizeStyle {
                icon: space::S3,
                ..square(space::S6)
            },
            // size-8
            ButtonSize::IconSm => square(space::S8),
            // size-10
            ButtonSize::IconLg => square(space::S10),
        }
    }
}

/// Group name the fill layer's hover reads; the nearest button wins.
const GROUP: &str = "brrritscold-core-gpui-button";

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let v = self.variant.style(theme);
        let z = self.size.style();
        let radius = theme.semantic().radius_md; // rounded-md
        let enabled = !self.disabled;
        let icon_leads_label = self.icon.is_some() && self.label.is_some();
        let icon_trails_label = self.icon_end.is_some() && self.label.is_some();

        // ring-3 is a 3px spread shadow; in CSS it stacks over the variant's shadow-xs.
        let mut focus_shadow = vec![super::ring(3., v.focus_ring)];
        if v.shadow_xs {
            focus_shadow.extend_from_slice(shadow::XS);
        }

        // `bg-clip-padding`: the fill stops inside the 1px border, transparent or not. GPUI
        // paints a background under the border, so the fill is its own layer.
        let fill = (v.fill.is_some() || v.hover_fill.is_some()).then(|| {
            div()
                .absolute()
                .inset_0()
                .rounded(radius - px(1.))
                .when_some(v.fill, |d, fill| d.bg(fill))
                .when_some(v.hover_fill.filter(|_| enabled), |d, hover| {
                    d.group_hover(GROUP, move |s| s.bg(hover))
                })
        });

        gpui_base::Button::new(self.id)
            .disabled(self.disabled)
            .group(GROUP)
            .relative()
            .flex_shrink_0()
            .h(z.height)
            .map(|b| {
                if z.square {
                    b.w(z.height)
                } else {
                    b.px(z.padding_x)
                        .gap(z.gap)
                        .when(icon_leads_label, |b| b.pl(z.padding_left_with_icon))
                        .when(icon_trails_label, |b| b.pr(z.padding_right_with_icon))
                }
            })
            .rounded(radius)
            .border_1()
            .border_color(v.border.unwrap_or(transparent_black()))
            .text_size(z.text)
            .font_weight(font_weight::MEDIUM)
            .text_color(v.text)
            .whitespace_nowrap()
            .when(v.shadow_xs, |b| b.shadow(shadow::XS.to_vec()))
            .when(v.hover_underline && enabled, |b| b.hover(|s| s.underline()))
            .focus_visible(move |s| s.border_color(v.focus_border).shadow(focus_shadow))
            .when(self.disabled, |b| b.opacity(opacity::O50))
            .when_some(self.on_click, |b, on_click| {
                b.on_click(move |event, window, cx| on_click(event, window, cx))
            })
            .children(fill)
            .when_some(self.icon, |b, icon| b.child(Icon::new(icon).size(z.icon)))
            .when_some(self.label, |b, label| b.child(label))
            .when_some(self.icon_end, |b, icon| {
                b.child(Icon::new(icon).size(z.icon))
            })
            .refine_style(&self.style)
    }
}

impl_styled!(Button);

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{Modifiers, TestAppContext};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::{render, theme};
    use crate::theme::Mode;

    mod variant {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<String> = ButtonVariant::iter().map(|v| v.to_string()).collect();
            assert_eq!(
                values,
                [
                    "default",
                    "outline",
                    "secondary",
                    "ghost",
                    "destructive",
                    "link"
                ]
            );
        }

        #[gpui::test]
        fn should_fill_outline_with_input_at_30_percent_in_dark(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let light = ButtonVariant::Outline.style(&theme(cx, Mode::Light));
            let dark = ButtonVariant::Outline.style(&theme(cx, Mode::Dark));
            let s = Mode::Dark.semantic();
            assert_eq!(light.fill, Some(Mode::Light.semantic().background));
            assert_eq!(dark.fill, Some(s.input.opacity(0.3)));
            assert_eq!(dark.border, Some(s.input));
        }

        #[gpui::test]
        fn should_tint_destructive_at_10_percent_light_and_20_percent_dark(
            cx: &mut TestAppContext,
        ) {
            cx.update(crate::init);
            let light = ButtonVariant::Destructive.style(&theme(cx, Mode::Light));
            let dark = ButtonVariant::Destructive.style(&theme(cx, Mode::Dark));
            assert_eq!(
                light.fill,
                Some(Mode::Light.semantic().destructive.opacity(0.1))
            );
            assert_eq!(
                dark.fill,
                Some(Mode::Dark.semantic().destructive.opacity(0.2))
            );
        }

        #[gpui::test]
        fn should_ring_focus_with_half_opacity_ring_except_destructive(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Light);
            assert_eq!(
                ButtonVariant::Ghost.style(&theme).focus_ring,
                theme.focus_ring()
            );
            assert_eq!(
                ButtonVariant::Destructive.style(&theme).focus_ring,
                theme.semantic().destructive.opacity(0.2)
            );
        }
    }

    mod size {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&'static str> = ButtonSize::iter().map(Into::into).collect();
            assert_eq!(
                values,
                [
                    "default", "xs", "sm", "lg", "icon", "icon-xs", "icon-sm", "icon-lg"
                ]
            );
            assert_eq!(ButtonSize::IconSm.as_ref(), "icon-sm");
        }

        #[gpui::test]
        fn should_be_36px_tall_by_default(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, move || Button::new("b").label("Button"));
            assert_eq!(bounds.size.height, px(36.));
        }

        #[gpui::test]
        fn should_size_xs_to_24px_tall(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, move || {
                Button::new("b").size(ButtonSize::Xs).label("Button")
            });
            assert_eq!(bounds.size.height, px(24.));
        }

        #[gpui::test]
        fn should_make_icon_sizes_square(cx: &mut TestAppContext) {
            cx.update(crate::init);
            for (size, side) in [
                (ButtonSize::Icon, 36.),
                (ButtonSize::IconXs, 24.),
                (ButtonSize::IconSm, 32.),
                (ButtonSize::IconLg, 40.),
            ] {
                let (_, bounds) =
                    render(cx, move || Button::new("b").size(size).icon(IconName::Plus));
                assert_eq!(bounds.size.width, px(side), "{size}");
                assert_eq!(bounds.size.height, px(side), "{size}");
            }
        }
    }

    mod click {
        use super::*;

        fn counted(disabled: bool) -> (impl Fn() -> Button, Rc<Cell<usize>>) {
            let clicks = Rc::new(Cell::new(0));
            let make = {
                let clicks = clicks.clone();
                move || {
                    let clicks = clicks.clone();
                    Button::new("b")
                        .label("Button")
                        .disabled(disabled)
                        .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                }
            };
            (make, clicks)
        }

        #[gpui::test]
        fn should_run_the_handler_on_click(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (button, clicks) = counted(false);
            let (cx, bounds) = render(cx, button);
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert_eq!(clicks.get(), 1);
        }

        #[gpui::test]
        fn should_ignore_clicks_while_disabled(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (button, clicks) = counted(true);
            let (cx, bounds) = render(cx, button);
            cx.simulate_click(bounds.center(), Modifiers::default());
            assert_eq!(clicks.get(), 0);
        }
    }
}
