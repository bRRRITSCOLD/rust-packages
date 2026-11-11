//! `Card` (Penpot property `size`) and its parts, with the story's "Team plan" content.

use brrritscold_core_gpui::{
    Button, ButtonSize, ButtonVariant, Card, CardAction, CardContent, CardDescription, CardFooter,
    CardHeader, CardSize, CardTitle,
};
use brrritscold_core_gpui_tokens::primitive::{font_size, size};
use gpui::{AnyElement, IntoElement, ParentElement as _, Styled as _, div};
use strum::IntoEnumIterator as _;

use super::{Entry, Variant, one};

fn title() -> CardTitle {
    CardTitle::new("Team plan")
}

fn description() -> CardDescription {
    CardDescription::new("Billed monthly, cancel any time.")
}

fn action() -> CardAction {
    CardAction::new().child(
        Button::new("change")
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Sm)
            .label("Change"),
    )
}

fn header() -> CardHeader {
    CardHeader::new()
        .title(title())
        .description(description())
        .action(action())
}

fn content() -> CardContent {
    CardContent::new().child("Five seats, shared projects and priority support.")
}

fn footer() -> CardFooter {
    CardFooter::new().child(Button::new("upgrade").label("Upgrade").w_full())
}

/// A part on its own, at a card's width and `text-sm`.
fn in_card(part: impl IntoElement) -> AnyElement {
    div()
        .w(size::SM)
        .text_size(font_size::SM)
        .child(part)
        .into_any_element()
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Card",
            properties: &["size"],
            variants: || {
                CardSize::iter()
                    .map(|card_size| Variant {
                        props: vec![("size", card_size.into())],
                        story: Some("components-card--default"),
                        // The story's max-w-sm, which its card fills.
                        render: Box::new(move |_| {
                            Card::new()
                                .size(card_size)
                                .w(size::SM)
                                .header(header())
                                .content(content())
                                .footer(footer())
                                .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "CardHeader",
            properties: &[],
            variants: || one(None, || in_card(header())),
        },
        Entry {
            name: "CardTitle",
            properties: &[],
            variants: || one(None, || title().into_any_element()),
        },
        Entry {
            name: "CardDescription",
            properties: &[],
            variants: || one(None, || description().into_any_element()),
        },
        Entry {
            name: "CardAction",
            properties: &[],
            variants: || one(None, || action().into_any_element()),
        },
        Entry {
            name: "CardContent",
            properties: &[],
            variants: || one(None, || in_card(content())),
        },
        Entry {
            name: "CardFooter",
            properties: &[],
            variants: || one(None, || in_card(footer())),
        },
    ]
}
