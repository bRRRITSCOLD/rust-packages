//! Badge: core-react's `badge.tsx`, a plain styled element.
//!
//! The styles mirror core-react's cva table arm by arm; each arm names the Tailwind classes it
//! stands for. Not mirrored:
//!
//! - The `[a]:hover:*` arms: they apply only when `render` swaps the `<span>` for a link, and a
//!   GPUI badge has no `render`.
//! - `focus-visible:*` and `aria-invalid:*`: a badge isn't focusable, and `aria-invalid` is no
//!   Penpot variant property.
//! - `underline-offset-4`: GPUI draws underlines at a fixed offset.

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, opacity, space};
use gpui::{
    App, Hsla, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _, transparent_black,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use crate::icons::{Icon, IconName};
use crate::theme::{ActiveTheme as _, Theme};

/// Penpot variant property `variant`. Cases are its values in PascalCase; [`Display`] gives
/// the Penpot value back (`BadgeVariant::Outline` → `outline`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum BadgeVariant {
    /// `default`
    #[default]
    Default,
    /// `secondary`
    Secondary,
    /// `destructive`
    Destructive,
    /// `outline`
    Outline,
    /// `ghost`
    Ghost,
    /// `link`
    Link,
}

/// A badge, styled like core-react's `Badge`.
///
/// One builder per Penpot variant property ([`variant`](Self::variant)) and one per layer
/// ([`label`](Self::label) for `Label`, [`icon`](Self::icon) for a leading icon):
///
/// ```no_run
/// use brrritscold_core_gpui::{Badge, BadgeVariant, IconName};
///
/// let new = Badge::new().label("New");
/// let verified = Badge::new()
///     .variant(BadgeVariant::Secondary)
///     .icon(IconName::BadgeCheck)
///     .label("Verified");
/// ```
#[derive(IntoElement)]
pub struct Badge {
    variant: BadgeVariant,
    label: Option<SharedString>,
    icon: Option<IconName>,
    style: StyleRefinement,
}

impl Default for Badge {
    fn default() -> Self {
        Self::new()
    }
}

impl Badge {
    /// A `default` badge with no content.
    pub fn new() -> Self {
        Self {
            variant: BadgeVariant::default(),
            label: None,
            icon: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `variant`.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The `Label` layer: the badge's text.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// An icon before the label (`data-icon="inline-start"`), 12px in the text colour.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
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
    hover_fill: Option<Hsla>,
    hover_text: Option<Hsla>,
    hover_underline: bool,
}

impl BadgeVariant {
    /// core-react's `variant` arms. Dark-only arms go through [`crate::Mode::pick`].
    fn style(self, theme: &Theme) -> VariantStyle {
        let s = theme.semantic();
        let m = theme.mode();
        let base = VariantStyle {
            fill: None,
            text: s.foreground,
            border: None,
            hover_fill: None,
            hover_text: None,
            hover_underline: false,
        };
        match self {
            // bg-primary text-primary-foreground
            BadgeVariant::Default => VariantStyle {
                fill: Some(s.primary),
                text: s.primary_foreground,
                ..base
            },
            // bg-secondary text-secondary-foreground
            BadgeVariant::Secondary => VariantStyle {
                fill: Some(s.secondary),
                text: s.secondary_foreground,
                ..base
            },
            // bg-destructive/10 text-destructive
            BadgeVariant::Destructive => VariantStyle {
                fill: Some(s.destructive.opacity(m.pick(opacity::O10, 0.2))), // dark:bg-destructive/20
                text: s.destructive,
                ..base
            },
            // border-border text-foreground
            BadgeVariant::Outline => VariantStyle {
                border: Some(s.border),
                ..base
            },
            // hover:bg-muted hover:text-muted-foreground
            BadgeVariant::Ghost => VariantStyle {
                hover_fill: Some(m.pick(s.muted, s.muted.opacity(0.5))), // dark:hover:bg-muted/50
                hover_text: Some(s.muted_foreground),
                ..base
            },
            // text-primary hover:underline
            BadgeVariant::Link => VariantStyle {
                text: s.primary,
                hover_underline: true,
                ..base
            },
        }
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let v = self.variant.style(cx.theme());
        let icon_leads_label = self.icon.is_some() && self.label.is_some();
        // inline-flex h-5 w-fit shrink-0 items-center justify-center gap-1 overflow-hidden
        // rounded-full border border-transparent px-2 py-0.5 text-xs font-medium whitespace-nowrap
        // has-data-[icon=inline-start]:pl-1.5. No bg-clip-padding: the fill runs under the
        // border, as GPUI paints it.
        div()
            .flex()
            .h(space::S5)
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .gap(space::S1)
            .overflow_hidden()
            .rounded_full()
            .border_1()
            .border_color(v.border.unwrap_or(transparent_black()))
            .px(space::S2)
            .py(space::S0_5)
            .when(icon_leads_label, |d| d.pl(space::S1_5))
            .text_size(font_size::XS)
            .font_weight(font_weight::MEDIUM)
            .whitespace_nowrap()
            .text_color(v.text)
            .when_some(v.fill, |d, fill| d.bg(fill))
            .when(
                v.hover_fill.is_some() || v.hover_text.is_some() || v.hover_underline,
                |d| {
                    d.hover(move |s| {
                        let s = if v.hover_underline { s.underline() } else { s };
                        let s = match v.hover_fill {
                            Some(fill) => s.bg(fill),
                            None => s,
                        };
                        match v.hover_text {
                            Some(text) => s.text_color(text),
                            None => s,
                        }
                    })
                },
            )
            // [&>svg]:size-3!
            .when_some(self.icon, |d, icon| {
                d.child(Icon::new(icon).size(space::S3))
            })
            .when_some(self.label, |d, label| d.child(label))
            .refine_style(&self.style)
    }
}

impl_styled!(Badge);

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, px};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::{render, theme};
    use crate::theme::Mode;

    mod variant {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&'static str> = BadgeVariant::iter().map(Into::into).collect();
            assert_eq!(
                values,
                [
                    "default",
                    "secondary",
                    "destructive",
                    "outline",
                    "ghost",
                    "link"
                ]
            );
        }

        #[gpui::test]
        fn should_tint_destructive_at_10_percent_light_and_20_percent_dark(
            cx: &mut TestAppContext,
        ) {
            cx.update(crate::init);
            let light = BadgeVariant::Destructive.style(&theme(cx, Mode::Light));
            let dark = BadgeVariant::Destructive.style(&theme(cx, Mode::Dark));
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
        fn should_border_only_outline(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Light);
            for variant in BadgeVariant::iter() {
                let border = variant.style(&theme).border;
                let expected =
                    (variant == BadgeVariant::Outline).then_some(theme.semantic().border);
                assert_eq!(border, expected, "{variant}");
            }
        }
    }

    mod layout {
        use super::*;

        #[gpui::test]
        fn should_be_20px_tall(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || Badge::new().label("Badge"));
            assert_eq!(bounds.size.height, px(20.));
        }

        #[gpui::test]
        fn should_take_a_callers_style_over_its_own(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || Badge::new().label("Badge").h(px(28.)));
            assert_eq!(bounds.size.height, px(28.));
        }
    }
}
