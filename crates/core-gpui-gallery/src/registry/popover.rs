//! `PopoverContent` and its parts, with the story's "Dimensions" content. Penpot's main
//! component holds a slot placeholder where the story has its width field.

use brrritscold_core_gpui::{PopoverContent, PopoverDescription, PopoverHeader, PopoverTitle};
use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{IntoElement as _, ParentElement as _, Styled as _, div};

use super::stand_in::{self, Field};
use super::{Entry, one};

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
pub(super) fn content() -> PopoverContent {
    PopoverContent::new()
        .header(header())
        .child(stand_in::field(
            Field::new("Width").horizontal().value("100%"),
        ))
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "PopoverContent",
            properties: &[],
            variants: || {
                one(Some("components-popover--default"), || {
                    content().into_any_element()
                })
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
