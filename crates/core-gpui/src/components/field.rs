//! Field: core-react's `field.tsx`, plain styled elements that lay a form out: a [`Field`]
//! (label, control, description, error) in a [`FieldGroup`], groups in a [`FieldSet`] under a
//! [`FieldLegend`].
//!
//! A field holds open content in the caller's order (a vertical field leads with its label, a
//! horizontal one with its control), so [`Field`] takes children. Where core-react styles a
//! child by what it is, the child comes through a typed builder instead: [`Field::content`]
//! for `has-[>[data-slot=field-content]]:items-start`, and [`FieldSet::legend`] /
//! [`FieldSet::description`] for the legend's spacing.
//!
//! Not mirrored:
//!
//! - `responsive`'s container query (`@min-[28rem]/field-group:` switches it to a row): GPUI
//!   has no container queries, so `responsive` lays out as `vertical`, which is what it shows
//!   in a group narrower than 448px.
//! - `data-invalid` (a field's text in `destructive`) and `data-disabled` (its label dimmed):
//!   neither is a Penpot variant property.
//! - The choice-card arms of FieldLabel (`has-[>[data-slot=field]]:…`, a field inside a label)
//!   and the checked-control tints (`has-data-checked:…`), which read the label's children
//!   through CSS relations.
//! - `*:data-[slot=field-label]:flex-auto`, `[&>[role=checkbox],[role=radio]]:mt-px` and
//!   FieldDescription's `nth-last-2:-mt-1` inside a field: they depend on a child's kind or
//!   place, which a field's open children don't carry.
//! - `text-balance`: GPUI wraps greedily.
//! - FieldError's `errors` list: an error takes its message.

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, line_height, space};
use gpui::{
    AnyElement, App, IntoElement, ParentElement as _, Pixels, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _, px, relative,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use super::label::Label;
use super::separator::Separator;
use crate::theme::ActiveTheme as _;

/// Penpot variant property `orientation` of [`Field`]. Cases are its values in PascalCase;
/// [`Display`] gives the Penpot value back (`FieldOrientation::Horizontal` → `horizontal`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum FieldOrientation {
    /// `vertical`: label, control and description stacked, each full width.
    #[default]
    Vertical,
    /// `horizontal`: a row, the control beside its label.
    Horizontal,
    /// `responsive`: `vertical`; core-react turns it into a row in a group at least 448px
    /// wide, which GPUI can't query.
    Responsive,
}

/// A field, styled like core-react's `Field`: a control with its label, description and
/// error, 12px apart.
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Field, FieldContent, FieldDescription, FieldLabel, FieldOrientation, Switch,
/// };
/// use gpui::ParentElement as _;
///
/// let marketing = Field::new()
///     .orientation(FieldOrientation::Horizontal)
///     .child(Switch::new("marketing"))
///     .content(
///         FieldContent::new()
///             .child(FieldLabel::new().child("Marketing emails"))
///             .child(FieldDescription::new("Product news, at most once a month.")),
///     );
/// ```
#[derive(IntoElement)]
pub struct Field {
    orientation: FieldOrientation,
    has_content: bool,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for Field {
    fn default() -> Self {
        Self::new()
    }
}

impl Field {
    /// An empty `vertical` field.
    pub fn new() -> Self {
        Self {
            orientation: FieldOrientation::default(),
            has_content: false,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `orientation`.
    pub fn orientation(mut self, orientation: FieldOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// A [`FieldContent`] after the children so far. A horizontal field with one aligns its
    /// row to the top, so the control sits beside the label rather than the block's middle.
    pub fn content(mut self, content: FieldContent) -> Self {
        self.has_content = true;
        self.children.push(content.into_any_element());
        self
    }
}

impl RenderOnce for Field {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // group/field flex w-full gap-3
        div()
            .flex()
            .w_full()
            .gap(space::S3)
            .map(|f| match self.orientation {
                // horizontal: flex-row items-center has-[>[data-slot=field-content]]:items-start
                FieldOrientation::Horizontal => f.flex_row().map(|f| {
                    if self.has_content {
                        f.items_start()
                    } else {
                        f.items_center()
                    }
                }),
                // vertical: flex-col *:w-full (the column stretches its children)
                FieldOrientation::Vertical | FieldOrientation::Responsive => f.flex_col(),
            })
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl_styled!(Field);
impl_parent!(Field);

/// Builds a part that is a styled container of open content.
macro_rules! container {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(IntoElement)]
        pub struct $name {
            children: Vec<AnyElement>,
            style: StyleRefinement,
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            /// An empty part.
            pub fn new() -> Self {
                Self {
                    children: Vec::new(),
                    style: StyleRefinement::default(),
                }
            }
        }

        impl_styled!($name);
        impl_parent!($name);
    };
}

container!(
    /// A group of fields, styled like core-react's `FieldGroup`: stacked 28px apart.
    FieldGroup
);

impl RenderOnce for FieldGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // group/field-group flex w-full flex-col gap-7
        div()
            .flex()
            .flex_col()
            .w_full()
            .gap(space::S7)
            .children(self.children)
            .refine_style(&self.style)
    }
}

container!(
    /// The `Content` part of a horizontal field: its label over its description, styled like
    /// core-react's `FieldContent`. Add it with [`Field::content`].
    FieldContent
);

impl RenderOnce for FieldContent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // group/field-content flex flex-1 flex-col gap-1 leading-snug
        div()
            .flex()
            .flex_1()
            .flex_col()
            .gap(space::S1)
            .line_height(relative(line_height::SNUG))
            .children(self.children)
            .refine_style(&self.style)
    }
}

container!(
    /// A field's `Label` part, styled like core-react's `FieldLabel`: a [`Label`] at
    /// `leading-snug`. Its children are its text and anything beside it.
    FieldLabel
);

impl RenderOnce for FieldLabel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // group/field-label peer/field-label flex w-fit gap-2 leading-snug
        Label::new()
            .line_height(relative(line_height::SNUG))
            .children(self.children)
            .refine_style(&self.style)
    }
}

container!(
    /// A field's title, styled like core-react's `FieldTitle`: a label's look for text that
    /// labels nothing.
    FieldTitle
);

impl RenderOnce for FieldTitle {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex w-fit items-center gap-2 text-sm font-medium
        div()
            .flex()
            .items_center()
            .gap(space::S2)
            .text_size(font_size::SM)
            .font_weight(font_weight::MEDIUM)
            .children(self.children)
            .refine_style(&self.style)
    }
}

/// A field's `Description` part, styled like core-react's `FieldDescription`.
#[derive(IntoElement)]
pub struct FieldDescription {
    text: SharedString,
    margin_top: Pixels,
    style: StyleRefinement,
}

impl FieldDescription {
    /// A description reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            margin_top: px(0.),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for FieldDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // text-left text-sm leading-normal font-normal text-muted-foreground
        div()
            .mt(self.margin_top)
            .text_size(font_size::SM)
            .line_height(relative(line_height::NORMAL))
            .font_weight(font_weight::NORMAL)
            .text_color(cx.theme().semantic().muted_foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

impl_styled!(FieldDescription);

/// A field's `Error` part, styled like core-react's `FieldError`.
#[derive(IntoElement)]
pub struct FieldError {
    text: SharedString,
    style: StyleRefinement,
}

impl FieldError {
    /// An error reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for FieldError {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // text-sm font-normal text-destructive
        div()
            .text_size(font_size::SM)
            .font_weight(font_weight::NORMAL)
            .text_color(cx.theme().semantic().destructive)
            .child(self.text)
            .refine_style(&self.style)
    }
}

impl_styled!(FieldError);

/// Penpot variant property `variant` of [`FieldLegend`]. Cases are its values in
/// PascalCase; [`Display`] gives the Penpot value back (`FieldLegendVariant::Label` →
/// `label`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum FieldLegendVariant {
    /// `legend`: `text-base`, a set's heading.
    #[default]
    Legend,
    /// `label`: `text-sm`, a set's label.
    Label,
}

/// A field set's `Legend` part, styled like core-react's `FieldLegend`.
#[derive(IntoElement)]
pub struct FieldLegend {
    variant: FieldLegendVariant,
    text: SharedString,
    style: StyleRefinement,
}

impl FieldLegend {
    /// A `legend` reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            variant: FieldLegendVariant::default(),
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `variant`.
    pub fn variant(mut self, variant: FieldLegendVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl RenderOnce for FieldLegend {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // mb-3 font-medium data-[variant=label]:text-sm data-[variant=legend]:text-base
        div()
            .mb(space::S3)
            .font_weight(font_weight::MEDIUM)
            .text_size(match self.variant {
                FieldLegendVariant::Legend => font_size::BASE,
                FieldLegendVariant::Label => font_size::SM,
            })
            .child(self.text)
            .refine_style(&self.style)
    }
}

impl_styled!(FieldLegend);

/// A set of fields under a legend, styled like core-react's `FieldSet`. Its `Legend` and
/// `Description` parts are builders; its fields (usually a [`FieldGroup`]) are its children:
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Checkbox, Field, FieldDescription, FieldGroup, FieldLabel, FieldLegend,
///     FieldOrientation, FieldSet,
/// };
/// use gpui::ParentElement as _;
///
/// let notifications = FieldSet::new()
///     .legend(FieldLegend::new("Notifications"))
///     .description(FieldDescription::new("Where we reach you."))
///     .child(FieldGroup::new().child(
///         Field::new()
///             .orientation(FieldOrientation::Horizontal)
///             .child(Checkbox::new("email").checked(true))
///             .child(FieldLabel::new().child("Email")),
///     ));
/// ```
#[derive(IntoElement)]
pub struct FieldSet {
    legend: Option<FieldLegend>,
    description: Option<FieldDescription>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for FieldSet {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldSet {
    /// An empty set.
    pub fn new() -> Self {
        Self {
            legend: None,
            description: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// The `Legend` part, at the top.
    pub fn legend(mut self, legend: FieldLegend) -> Self {
        self.legend = Some(legend);
        self
    }

    /// The `Description` part, under the legend and before the fields.
    pub fn description(mut self, description: FieldDescription) -> Self {
        self.description = Some(description);
        self
    }
}

impl RenderOnce for FieldSet {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let after_legend = self
            .legend
            .as_ref()
            .is_some_and(|legend| legend.variant == FieldLegendVariant::Legend);
        let description = self.description.map(|mut description| {
            description.margin_top = if after_legend {
                // [[data-variant=legend]+&]:-mt-1.5
                -space::S1_5
            } else if self.children.len() == 1 {
                // nth-last-2:-mt-1
                -space::S1
            } else {
                px(0.)
            };
            description
        });
        // A <legend> sits above the fieldset's flex content, so its mb-3 isn't a flex gap:
        // the legend, then the fields in the set's flex flex-col gap-6.
        div()
            .flex()
            .flex_col()
            .children(self.legend)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(space::S6)
                    .children(description)
                    .children(self.children),
            )
            .refine_style(&self.style)
    }
}

impl_styled!(FieldSet);
impl_parent!(FieldSet);

/// A rule between fields, styled like core-react's `FieldSeparator`, with optional text
/// (`Or`) on it.
#[derive(IntoElement)]
pub struct FieldSeparator {
    text: Option<SharedString>,
    style: StyleRefinement,
}

impl Default for FieldSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldSeparator {
    /// A bare rule.
    pub fn new() -> Self {
        Self {
            text: None,
            style: StyleRefinement::default(),
        }
    }

    /// The `Text` layer: words centred on the rule.
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
        self
    }
}

impl RenderOnce for FieldSeparator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // relative -my-2 h-5 text-sm
        div()
            .relative()
            .flex()
            .justify_center()
            .my(-space::S2)
            .h(space::S5)
            .text_size(font_size::SM)
            // The Separator: absolute inset-0 top-1/2
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(space::S5 / 2.)
                    .child(Separator::new()),
            )
            // relative mx-auto block w-fit bg-background px-2 text-muted-foreground
            .when_some(self.text, |d, text| {
                d.child(
                    div()
                        .relative()
                        .bg(s.background)
                        .px(space::S2)
                        .text_color(s.muted_foreground)
                        .child(text),
                )
            })
            .refine_style(&self.style)
    }
}

impl_styled!(FieldSeparator);

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::switch::Switch;
    use crate::components::test_support::render;

    mod field {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&'static str> = FieldOrientation::iter().map(Into::into).collect();
            assert_eq!(values, ["vertical", "horizontal", "responsive"]);
        }

        #[gpui::test]
        fn should_stack_a_vertical_field_12px_apart(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || {
                Field::new()
                    .w(px(384.))
                    .child(FieldLabel::new().child("Username"))
                    .child(FieldDescription::new("Choose a unique username."))
            });
            // A 14px label at leading-snug (19.25px, which GPUI lays out on a 20px line), 12px,
            // a 20px description.
            assert_eq!(bounds.size.height, px(20. + 12. + 20.));
        }

        #[gpui::test]
        fn should_top_align_a_horizontal_field_with_content(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || {
                Field::new()
                    .orientation(FieldOrientation::Horizontal)
                    .w(px(384.))
                    .child(Switch::new("s"))
                    .content(
                        FieldContent::new()
                            .child(FieldLabel::new().child("Marketing emails"))
                            .child(FieldDescription::new("Product news.")),
                    )
            });
            // The content block sets the row's height: a 20px label line, 4px, a 20px
            // description.
            assert_eq!(bounds.size.height, px(20. + 4. + 20.));
        }
    }

    mod legend {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&'static str> = FieldLegendVariant::iter().map(Into::into).collect();
            assert_eq!(values, ["legend", "label"]);
        }
    }

    mod separator {
        use super::*;

        #[gpui::test]
        fn should_take_20px_less_its_negative_margins(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || FieldSeparator::new().text("Or").w(px(384.)));
            assert_eq!(bounds.size.height, px(20.));
        }
    }
}
