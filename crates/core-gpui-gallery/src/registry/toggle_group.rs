//! `ToggleGroup` (Penpot properties `orientation` × `spacing` × `variant`) and its `Item`
//! (`variant` × `pressed`), with the story's three alignment icons. As in Penpot's main
//! component and after the story's play function, the centre item is pressed.

use brrritscold_core_gpui::{
    IconName, ToggleGroup, ToggleGroupItem, ToggleGroupOrientation, ToggleGroupSpacing,
    ToggleVariant,
};
use gpui::{ElementId, IntoElement as _};
use strum::IntoEnumIterator as _;

use super::support::{BOOLS, child, value};
use super::{Entry, Variant};

const ALIGNMENTS: [(&str, IconName); 3] = [
    ("Align left", IconName::TextAlignStart),
    ("Align center", IconName::TextAlignCenter),
    ("Align right", IconName::TextAlignEnd),
];

fn items(id: ElementId) -> impl Iterator<Item = ToggleGroupItem> {
    ALIGNMENTS
        .iter()
        .enumerate()
        .map(move |(index, (label, icon))| {
            ToggleGroupItem::new(child(&id, index))
                .icon(*icon)
                .accessibility_label(*label)
                .pressed(index == 1)
        })
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "ToggleGroup",
            properties: &["orientation", "spacing", "variant"],
            variants: || {
                ToggleGroupOrientation::iter()
                    .flat_map(|o| ToggleGroupSpacing::iter().map(move |s| (o, s)))
                    .flat_map(|(o, s)| ToggleVariant::iter().map(move |v| (o, s, v)))
                    .map(|(orientation, spacing, variant)| Variant {
                        props: vec![
                            ("orientation", orientation.into()),
                            ("spacing", spacing.into()),
                            ("variant", variant.into()),
                        ],
                        story: Some("components-togglegroup--default"),
                        render: Box::new(move |id| {
                            ToggleGroup::new(id.clone())
                                .orientation(orientation)
                                .spacing(spacing)
                                .variant(variant)
                                .items(items(id))
                                .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "ToggleGroupItem",
            properties: &["variant", "pressed"],
            variants: || {
                ToggleVariant::iter()
                    .flat_map(|variant| BOOLS.map(move |pressed| (variant, pressed)))
                    .map(|(variant, pressed)| Variant {
                        props: vec![("variant", variant.into()), ("pressed", value(pressed))],
                        story: None,
                        render: Box::new(move |id| {
                            ToggleGroupItem::new(id)
                                .variant(variant)
                                .pressed(pressed)
                                .icon(IconName::TextAlignCenter)
                                .accessibility_label("Align center")
                                .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
    ]
}
