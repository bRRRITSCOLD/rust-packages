//! Alert: core-react's `alert.tsx`, plain styled elements.
//!
//! core-react lays an alert out as a grid: with an icon (`has-[>svg]`), the icon takes a column
//! spanning both rows and the text the rest. Here that is a row: the icon, then a text column
//! holding the title over the description. The `variant` reaches the description
//! (`*:data-[slot=alert-description]:text-destructive/90`) because [`Alert`] takes its parts as
//! typed builders and hands the variant down.
//!
//! Not mirrored: `text-balance` / `md:text-pretty` (GPUI wraps greedily), the link styles inside
//! a title or description (`[&_a]:underline`), and `[&_p:not(:last-child)]:mb-4`.

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, space};
use gpui::{
    AnyElement, App, Hsla, IntoElement, ParentElement as _, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use crate::icons::{Icon, IconName};
use crate::theme::{ActiveTheme as _, Theme};

/// Penpot variant property `variant`. Cases are its values in PascalCase; [`Display`] gives
/// the Penpot value back (`AlertVariant::Destructive` → `destructive`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum AlertVariant {
    /// `default`
    #[default]
    Default,
    /// `destructive`: title, icon and description in `destructive`.
    Destructive,
}

/// The colours of one `variant` arm.
#[derive(Debug, Clone, Copy, PartialEq)]
struct VariantStyle {
    fill: Hsla,
    /// The alert's text colour, which its title and icon (`text-current`) inherit.
    text: Hsla,
    description: Hsla,
}

impl AlertVariant {
    /// core-react's `variant` arms, over the description's own `text-muted-foreground`.
    fn style(self, theme: &Theme) -> VariantStyle {
        let s = theme.semantic();
        match self {
            // bg-card text-card-foreground
            AlertVariant::Default => VariantStyle {
                fill: s.card,
                text: s.card_foreground,
                description: s.muted_foreground,
            },
            // bg-card text-destructive *:data-[slot=alert-description]:text-destructive/90
            AlertVariant::Destructive => VariantStyle {
                fill: s.card,
                text: s.destructive,
                description: s.destructive.opacity(0.9),
            },
        }
    }
}

/// An alert, styled like core-react's `Alert`. Its parts are builders:
///
/// ```no_run
/// use brrritscold_core_gpui::{Alert, AlertDescription, AlertTitle, AlertVariant, IconName};
///
/// let failed = Alert::new()
///     .variant(AlertVariant::Destructive)
///     .icon(IconName::CircleX)
///     .title(AlertTitle::new("Payment failed"))
///     .description(AlertDescription::new("Your card was declined."));
/// ```
#[derive(IntoElement)]
pub struct Alert {
    variant: AlertVariant,
    icon: Option<IconName>,
    title: Option<AlertTitle>,
    description: Option<AlertDescription>,
    action: Option<AlertAction>,
    style: StyleRefinement,
}

impl Default for Alert {
    fn default() -> Self {
        Self::new()
    }
}

impl Alert {
    /// A `default` alert with no parts. It fills its container's width.
    pub fn new() -> Self {
        Self {
            variant: AlertVariant::default(),
            icon: None,
            title: None,
            description: None,
            action: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `variant`.
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The `Icon` part, in the alert's text colour, beside the title and description.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// The `Title` part.
    pub fn title(mut self, title: AlertTitle) -> Self {
        self.title = Some(title);
        self
    }

    /// The `Description` part, under the title.
    pub fn description(mut self, description: AlertDescription) -> Self {
        self.description = Some(description);
        self
    }

    /// The `Action` part, pinned top right; the alert makes room for it.
    pub fn action(mut self, action: AlertAction) -> Self {
        self.action = Some(action);
        self
    }
}

impl RenderOnce for Alert {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let s = theme.semantic();
        let v = self.variant.style(theme);
        // relative grid w-full gap-0.5 rounded-lg border px-4 py-3 text-left text-sm
        // has-[>svg]:gap-x-2.5 has-data-[slot=alert-action]:pr-18
        div()
            .relative()
            .w_full()
            .flex()
            .items_start()
            .gap(space::S2_5)
            .rounded(s.radius_lg)
            .border_1()
            .border_color(s.border)
            .px(space::S4)
            .py(space::S3)
            .when(self.action.is_some(), |d| d.pr(space::S18))
            .text_size(font_size::SM)
            .bg(v.fill)
            .text_color(v.text)
            // *:[svg]:row-span-2 *:[svg]:translate-y-0.5 *:[svg]:text-current size-4. The
            // translate moves the icon without taking space, as a top margin does here: the
            // icon is shorter than the text column beside it.
            .when_some(self.icon, |d, icon| {
                d.child(Icon::new(icon).size(space::S4).mt(px(2.)))
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(space::S0_5)
                    .children(self.title)
                    .children(
                        self.description
                            .map(|description| description.color(v.description)),
                    ),
            )
            .children(self.action)
            .refine_style(&self.style)
    }
}

/// The alert's `Title` part, in the alert's text colour.
#[derive(IntoElement)]
pub struct AlertTitle {
    text: SharedString,
    style: StyleRefinement,
}

impl AlertTitle {
    /// A title reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for AlertTitle {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // font-medium group-has-[>svg]/alert:col-start-2
        div()
            .font_weight(font_weight::MEDIUM)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The alert's `Description` part.
#[derive(IntoElement)]
pub struct AlertDescription {
    text: SharedString,
    /// Set by the alert's `variant`; `None` alone is `text-muted-foreground`.
    color: Option<Hsla>,
    style: StyleRefinement,
}

impl AlertDescription {
    /// A description reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            color: None,
            style: StyleRefinement::default(),
        }
    }

    fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl RenderOnce for AlertDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // text-sm text-muted-foreground
        let color = self.color.unwrap_or(cx.theme().semantic().muted_foreground);
        div()
            .text_size(font_size::SM)
            .text_color(color)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The alert's `Action` part: open content pinned to the alert's top right.
#[derive(IntoElement)]
pub struct AlertAction {
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for AlertAction {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertAction {
    /// An empty action; its children are its content.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for AlertAction {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // absolute top-2.5 right-3
        div()
            .absolute()
            .top(space::S2_5)
            .right(space::S3)
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl_styled!(Alert, AlertTitle, AlertDescription, AlertAction);
impl_parent!(AlertAction);

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::Button;
    use crate::components::test_support::{render, theme};
    use crate::theme::Mode;

    mod variant {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&'static str> = AlertVariant::iter().map(Into::into).collect();
            assert_eq!(values, ["default", "destructive"]);
        }

        #[gpui::test]
        fn should_turn_the_description_destructive_at_90_percent(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Dark);
            let s = theme.semantic();
            let style = AlertVariant::Destructive.style(&theme);
            assert_eq!(style.text, s.destructive);
            assert_eq!(style.description, s.destructive.opacity(0.9));
            assert_eq!(
                AlertVariant::Default.style(&theme).description,
                s.muted_foreground
            );
        }
    }

    mod layout {
        use super::*;

        fn alert() -> Alert {
            Alert::new()
                .w(px(400.))
                .title(AlertTitle::new("Heads up!"))
                .description(AlertDescription::new("Something happened."))
        }

        #[gpui::test]
        fn should_stack_title_over_description_inside_its_padding(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, alert);
            // border + py-3 + two 21px lines + gap-0.5 + py-3 + border
            assert_eq!(bounds.size.height, px(1. + 12. + 21. + 2. + 21. + 12. + 1.));
        }

        #[gpui::test]
        fn should_keep_its_height_with_an_action_pinned_in_the_corner(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, plain) = render(cx, alert);
            let (_, with_action) = render(cx, || {
                alert().action(AlertAction::new().child(Button::new("b").label("Install")))
            });
            assert_eq!(with_action.size.height, plain.size.height);
        }
    }
}
