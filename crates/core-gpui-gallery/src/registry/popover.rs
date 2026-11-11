//! `PopoverContent` and its parts, with the story's "Dimensions" content. Penpot's main
//! component holds a slot placeholder where the story has its width field.

use brrritscold_core_gpui::{PopoverContent, PopoverDescription, PopoverHeader, PopoverTitle};
use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{ElementId, IntoElement as _, ParentElement as _, Styled as _, div};

use super::{Entry, Variant, fields, one};

fn title() -> PopoverTitle {
    PopoverTitle::new("Dimensions")
}

fn description() -> PopoverDescription {
    PopoverDescription::new("Set the dimensions for the layer.")
}

fn header() -> PopoverHeader {
    PopoverHeader::new()
        .title(title())
        .description(description())
}

/// The surface's sample content: the story's header and width field.
pub(super) fn content(id: &ElementId) -> PopoverContent {
    PopoverContent::new()
        .header(header())
        .child(fields::width(id))
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "PopoverContent",
            properties: &[],
            variants: || {
                vec![Variant {
                    props: Vec::new(),
                    story: Some("components-popover--default"),
                    render: Box::new(|id| content(&id).into_any_element()),
                }]
            },
        },
        Entry {
            name: "PopoverHeader",
            properties: &[],
            // At the surface's content width: 288px less its 16px padding each side.
            variants: || {
                one(None, || {
                    div()
                        .w(space::S72 - space::S8)
                        .child(header())
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "PopoverTitle",
            properties: &[],
            variants: || one(None, || title().into_any_element()),
        },
        Entry {
            name: "PopoverDescription",
            properties: &[],
            variants: || one(None, || description().into_any_element()),
        },
    ]
}
