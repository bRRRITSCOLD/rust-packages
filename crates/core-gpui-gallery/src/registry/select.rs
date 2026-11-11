//! `SelectTrigger` (Penpot properties `aria-invalid` × `disabled` × `size`, the combinations
//! Penpot draws) and the list's parts, with the story's produce content.

use brrritscold_core_gpui::{
    SelectContent, SelectGroup, SelectItem, SelectLabel, SelectSeparator, SelectTrigger,
    SelectTriggerSize,
};
use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{IntoElement as _, ParentElement as _, Styled as _, div, px};

use super::{Entry, Variant, one};

/// The trigger stories: `Small` keeps its placeholder (`Default`'s play chooses Banana) and
/// takes the variant's props as args.
const TRIGGER_STORY: &str = "components-select--small";

/// The story's trigger fills its vertical `Field` (`*:w-full`): the padded story width less
/// the story's own `p-8`.
const TRIGGER_WIDTH: gpui::Pixels = px(544.);

/// The story's list: Fruits (Grapes disabled), a separator, Vegetables.
pub(super) fn content() -> SelectContent {
    SelectContent::new()
        .group(
            SelectGroup::new()
                .label(SelectLabel::new("Fruits"))
                .item(SelectItem::new("apple", "Apple"))
                .item(SelectItem::new("banana", "Banana"))
                .item(SelectItem::new("blueberry", "Blueberry"))
                .item(SelectItem::new("grapes", "Grapes").disabled(true)),
        )
        .separator(SelectSeparator::new())
        .group(
            SelectGroup::new()
                .label(SelectLabel::new("Vegetables"))
                .item(SelectItem::new("carrot", "Carrot"))
                .item(SelectItem::new("leek", "Leek")),
        )
}

/// A list part at the story list's width: 144px (`min-w-36`) less the group's 4px padding.
fn in_list(part: impl gpui::IntoElement) -> gpui::AnyElement {
    div()
        .w(space::S36 - space::S2)
        .child(part)
        .into_any_element()
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "SelectTrigger",
            properties: &["aria-invalid", "disabled", "size"],
            variants: || {
                // Penpot's combinations: each state at the default size, and sm.
                [
                    (false, false, SelectTriggerSize::Default),
                    (false, false, SelectTriggerSize::Sm),
                    (false, true, SelectTriggerSize::Default),
                    (true, false, SelectTriggerSize::Default),
                ]
                .into_iter()
                .map(|(invalid, disabled, size)| Variant {
                    props: vec![
                        ("aria-invalid", if invalid { "true" } else { "false" }),
                        ("disabled", if disabled { "true" } else { "false" }),
                        ("size", size.into()),
                    ],
                    story: Some(TRIGGER_STORY),
                    render: Box::new(move |_| {
                        SelectTrigger::new()
                            .aria_invalid(invalid)
                            .disabled(disabled)
                            .size(size)
                            .placeholder("Pick one")
                            .w(TRIGGER_WIDTH)
                            .into_any_element()
                    }),
                })
                .collect()
            },
        },
        Entry {
            name: "SelectContent",
            properties: &[],
            variants: || {
                one(Some("components-select--default"), || {
                    content().into_any_element()
                })
            },
        },
        Entry {
            name: "SelectItem",
            properties: &[],
            // Penpot's main component shows its check.
            variants: || {
                one(None, || {
                    in_list(SelectItem::new("apple", "Apple").selected(true))
                })
            },
        },
        Entry {
            name: "SelectLabel",
            properties: &[],
            variants: || one(None, || in_list(SelectLabel::new("Fruits"))),
        },
        Entry {
            name: "SelectSeparator",
            properties: &[],
            variants: || one(None, || in_list(SelectSeparator::new())),
        },
    ]
}
