//! `RadioGroup` (no variant properties) and its `Item` (Penpot property `disabled`), with the
//! story's plans. The story's play function selects "Pro", so the group shows it selected.

use brrritscold_core_gpui::{Label, RadioGroup, RadioGroupItem};
use gpui::{IntoElement as _, ParentElement as _, Styled as _};

use super::support::{BOOLS, child, value};
use super::{Entry, STORY_WIDTH, Variant};

const PLANS: [&str; 3] = ["Starter", "Pro", "Enterprise"];

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "RadioGroup",
            properties: &[],
            variants: || {
                vec![Variant {
                    props: Vec::new(),
                    story: Some("components-radiogroup--default"),
                    // The story's w-full group, at the width it fills.
                    render: Box::new(|id| {
                        RadioGroup::new(id.clone())
                            .w(STORY_WIDTH)
                            .children(PLANS.iter().enumerate().map(|(index, plan)| {
                                Label::new()
                                    .child(
                                        RadioGroupItem::new(child(&id, index))
                                            .checked(*plan == "Pro"),
                                    )
                                    .child(*plan)
                            }))
                            .into_any_element()
                    }),
                }]
            },
        },
        Entry {
            name: "RadioGroupItem",
            properties: &["disabled"],
            // Penpot's main component shows the item checked.
            variants: || {
                BOOLS
                    .into_iter()
                    .map(|disabled| Variant {
                        props: vec![("disabled", value(disabled))],
                        story: None,
                        render: Box::new(move |id| {
                            RadioGroupItem::new(id)
                                .checked(true)
                                .disabled(disabled)
                                .accessibility_label("Starter")
                                .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
    ]
}
