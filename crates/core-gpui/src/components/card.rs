//! Card: core-react's `card.tsx`, plain styled elements.
//!
//! A card's `size` reaches its parts through `group-data-[size=sm]/card:` in core-react; here
//! [`Card`] takes its parts as typed builders and hands its size down, and [`CardHeader`] does
//! the same for its title. The header's CSS grid (title and description stacked, the action
//! spanning both rows at the right) is a row: a text column that grows, then the action.
//!
//! Not mirrored: the image rules (`has-[>img:first-child]:pt-0`, rounded image corners), since a
//! card takes no image part, and the ruled header and footer (`[.border-b]:pb-6`,
//! `[.border-t]:pt-6`), which hang off a class the caller adds.

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, line_height, shadow, space};
use gpui::{
    AnyElement, App, IntoElement, ParentElement as _, Pixels, Rems, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, div, relative,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use crate::theme::ActiveTheme as _;

/// Penpot variant property `size`. Cases are its values in PascalCase; [`Display`] gives the
/// Penpot value back (`CardSize::Sm` → `sm`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum CardSize {
    /// `default`: 24px padding and gaps.
    #[default]
    Default,
    /// `sm`: 16px padding and gaps, a `text-sm` title.
    Sm,
}

impl CardSize {
    /// The card's spacing step: its vertical padding, the gap between parts and each part's
    /// horizontal padding (`gap-6 py-6`, `px-6`; at `sm` `gap-4 py-4`, `px-4`).
    fn spacing(self) -> Pixels {
        match self {
            CardSize::Default => space::S6,
            CardSize::Sm => space::S4,
        }
    }

    /// `text-base`; at `sm` `group-data-[size=sm]/card:text-sm`.
    fn title_text(self) -> Rems {
        match self {
            CardSize::Default => font_size::BASE,
            CardSize::Sm => font_size::SM,
        }
    }
}

/// A card, styled like core-react's `Card`. Its parts are builders, in Penpot's order:
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Button, Card, CardContent, CardDescription, CardFooter, CardHeader, CardSize, CardTitle,
/// };
/// use gpui::ParentElement as _;
///
/// let card = Card::new()
///     .size(CardSize::Sm)
///     .header(
///         CardHeader::new()
///             .title(CardTitle::new("Team plan"))
///             .description(CardDescription::new("Billed monthly.")),
///     )
///     .content(CardContent::new().child("Five seats."))
///     .footer(CardFooter::new().child(Button::new("upgrade").label("Upgrade")));
/// ```
#[derive(IntoElement)]
pub struct Card {
    size: CardSize,
    header: Option<CardHeader>,
    content: Option<CardContent>,
    footer: Option<CardFooter>,
    style: StyleRefinement,
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Card {
    /// A `default` card with no parts.
    pub fn new() -> Self {
        Self {
            size: CardSize::default(),
            header: None,
            content: None,
            footer: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `size`.
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = size;
        self
    }

    /// The `Header` part.
    pub fn header(mut self, header: CardHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// The `Content` part.
    pub fn content(mut self, content: CardContent) -> Self {
        self.content = Some(content);
        self
    }

    /// The `Footer` part.
    pub fn footer(mut self, footer: CardFooter) -> Self {
        self.footer = Some(footer);
        self
    }
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let size = self.size;
        // ring-1 ring-foreground/10 over shadow-xs: the ring is a spread shadow, outside the box.
        let mut shadows = vec![super::ring(1., s.foreground.opacity(0.1))];
        shadows.extend_from_slice(shadow::XS);
        // flex flex-col gap-6 overflow-hidden rounded-xl bg-card py-6 text-sm
        // text-card-foreground shadow-xs ring-1 ring-foreground/10 data-[size=sm]:gap-4
        // data-[size=sm]:py-4
        div()
            .flex()
            .flex_col()
            .gap(size.spacing())
            .overflow_hidden()
            .rounded(s.radius_xl)
            .bg(s.card)
            .py(size.spacing())
            .text_size(font_size::SM)
            .text_color(s.card_foreground)
            .shadow(shadows)
            .children(self.header.map(|header| header.card_size(size)))
            .children(self.content.map(|content| content.card_size(size)))
            .children(self.footer.map(|footer| footer.card_size(size)))
            .refine_style(&self.style)
    }
}

/// The card's `Header` part: a title and description stacked, an action at the right.
#[derive(IntoElement)]
pub struct CardHeader {
    size: CardSize,
    title: Option<CardTitle>,
    description: Option<CardDescription>,
    action: Option<CardAction>,
    style: StyleRefinement,
}

impl Default for CardHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl CardHeader {
    /// An empty header, spaced for a `default` card until a [`Card`] renders it.
    pub fn new() -> Self {
        Self {
            size: CardSize::default(),
            title: None,
            description: None,
            action: None,
            style: StyleRefinement::default(),
        }
    }

    /// The `Title` part.
    pub fn title(mut self, title: CardTitle) -> Self {
        self.title = Some(title);
        self
    }

    /// The `Description` part, under the title.
    pub fn description(mut self, description: CardDescription) -> Self {
        self.description = Some(description);
        self
    }

    /// The `Action` part, at the header's right edge.
    pub fn action(mut self, action: CardAction) -> Self {
        self.action = Some(action);
        self
    }

    fn card_size(mut self, size: CardSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for CardHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let size = self.size;
        // grid auto-rows-min items-start gap-1 rounded-t-xl px-6
        // group-data-[size=sm]/card:px-4
        div()
            .flex()
            .items_start()
            .gap(space::S1)
            .px(size.spacing())
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(space::S1)
                    .children(self.title.map(|title| title.card_size(size)))
                    .children(self.description),
            )
            // col-start-2 row-span-2 row-start-1 self-start justify-self-end
            .children(self.action)
            .refine_style(&self.style)
    }
}

/// The card's `Title` part.
#[derive(IntoElement)]
pub struct CardTitle {
    size: CardSize,
    text: SharedString,
    style: StyleRefinement,
}

impl CardTitle {
    /// A title reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            size: CardSize::default(),
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }

    fn card_size(mut self, size: CardSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for CardTitle {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // text-base leading-normal font-medium group-data-[size=sm]/card:text-sm
        div()
            .text_size(self.size.title_text())
            .line_height(relative(line_height::NORMAL))
            .font_weight(font_weight::MEDIUM)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The card's `Description` part.
#[derive(IntoElement)]
pub struct CardDescription {
    text: SharedString,
    style: StyleRefinement,
}

impl CardDescription {
    /// A description reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for CardDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // text-sm text-muted-foreground
        div()
            .text_size(font_size::SM)
            .text_color(cx.theme().semantic().muted_foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The card header's `Action` part: open content (its Penpot `Body` slot) at the right edge.
#[derive(IntoElement)]
pub struct CardAction {
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for CardAction {
    fn default() -> Self {
        Self::new()
    }
}

impl CardAction {
    /// An empty action; its children are its content.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

/// The card's `Content` part: open content (its Penpot `Body` slot).
#[derive(IntoElement)]
pub struct CardContent {
    size: CardSize,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for CardContent {
    fn default() -> Self {
        Self::new()
    }
}

impl CardContent {
    /// An empty content part; its children are its content.
    pub fn new() -> Self {
        Self {
            size: CardSize::default(),
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    fn card_size(mut self, size: CardSize) -> Self {
        self.size = size;
        self
    }
}

/// The card's `Footer` part: open content (its Penpot `Body` slot) in a row.
#[derive(IntoElement)]
pub struct CardFooter {
    size: CardSize,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for CardFooter {
    fn default() -> Self {
        Self::new()
    }
}

impl CardFooter {
    /// An empty footer; its children are its content.
    pub fn new() -> Self {
        Self {
            size: CardSize::default(),
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    fn card_size(mut self, size: CardSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for CardAction {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex_none()
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl RenderOnce for CardContent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex flex-col gap-3 px-6 group-data-[size=sm]/card:px-4
        div()
            .flex()
            .flex_col()
            .gap(space::S3)
            .px(self.size.spacing())
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl RenderOnce for CardFooter {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex items-center rounded-b-xl px-6 group-data-[size=sm]/card:px-4
        div()
            .flex()
            .items_center()
            .px(self.size.spacing())
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl_styled!(
    Card,
    CardHeader,
    CardTitle,
    CardDescription,
    CardAction,
    CardContent,
    CardFooter
);
impl_parent!(CardAction, CardContent, CardFooter);

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, px};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::render;

    fn card(size: CardSize) -> Card {
        Card::new()
            .size(size)
            .w(px(384.))
            .header(CardHeader::new().title(CardTitle::new("Team plan")))
            .content(CardContent::new().child(div().h(px(20.))))
    }

    mod size {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&'static str> = CardSize::iter().map(Into::into).collect();
            assert_eq!(values, ["default", "sm"]);
        }

        #[gpui::test]
        fn should_space_a_default_card_by_24px(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || card(CardSize::Default));
            // py-6 + a 16px × 1.5 title + gap-6 + 20px of content + py-6
            assert_eq!(bounds.size.height, px(24. + 24. + 24. + 20. + 24.));
        }

        #[gpui::test]
        fn should_space_a_small_card_by_16px_with_a_text_sm_title(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || card(CardSize::Sm));
            // py-4 + a 14px × 1.5 title + gap-4 + 20px of content + py-4
            assert_eq!(bounds.size.height, px(16. + 21. + 16. + 20. + 16.));
        }
    }
}
