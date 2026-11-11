//! `Alert` (Penpot property `variant`) and its parts, with the story's "Payment successful"
//! content. Penpot's main component also shows the `Action` part; the story the variants are
//! captured from has none, so neither has the entry.

use brrritscold_core_gpui::{
    Alert, AlertAction, AlertDescription, AlertTitle, AlertVariant, Button, ButtonSize, IconName,
};
use gpui::{IntoElement as _, ParentElement as _, Styled as _};
use strum::IntoEnumIterator as _;

use super::{Entry, STORY_WIDTH, Variant, one};

fn title() -> AlertTitle {
    AlertTitle::new("Payment successful")
}

fn description() -> AlertDescription {
    AlertDescription::new("Your payment of $29.99 has been processed.")
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Alert",
            properties: &["variant"],
            variants: || {
                AlertVariant::iter()
                    .map(|variant| Variant {
                        props: vec![("variant", variant.into())],
                        story: Some("components-alert--default"),
                        render: Box::new(move |_| {
                            Alert::new()
                                .variant(variant)
                                .w(STORY_WIDTH)
                                .icon(IconName::CircleCheck)
                                .title(title())
                                .description(description())
                                .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "AlertTitle",
            properties: &[],
            variants: || one(None, || title().into_any_element()),
        },
        Entry {
            name: "AlertDescription",
            properties: &[],
            variants: || one(None, || description().into_any_element()),
        },
        Entry {
            name: "AlertAction",
            properties: &[],
            // Alone, the action has no alert corner to pin to: it stays where it is laid out.
            variants: || {
                one(None, || {
                    AlertAction::new()
                        .relative()
                        .top_0()
                        .right_0()
                        .child(Button::new("install").size(ButtonSize::Xs).label("Install"))
                        .into_any_element()
                })
            },
        },
    ]
}
