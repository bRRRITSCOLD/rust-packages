//! `SheetContent` (Penpot property `side`) and its parts, with the story's "Edit profile"
//! content. Penpot's main component holds a slot placeholder where the story has its name
//! field.

use brrritscold_core_gpui::{
    Button, ButtonVariant, DialogClose, SheetContent, SheetContentSide, SheetDescription,
    SheetFooter, SheetHeader, SheetTitle,
};
use brrritscold_core_gpui_tokens::primitive::{size, space};
use gpui::{IntoElement as _, ParentElement as _, Styled as _, div, px};
use strum::IntoEnumIterator as _;

use super::stand_in::{self, Field};
use super::{Entry, Variant, one};

/// The story's viewport, which a sheet fills along its edge.
const VIEWPORT_WIDTH: gpui::Pixels = px(640.);
const VIEWPORT_HEIGHT: gpui::Pixels = px(320.);

fn title() -> SheetTitle {
    SheetTitle::new("Edit profile")
}

fn description() -> SheetDescription {
    SheetDescription::new("Make changes to your profile here.")
}

fn header() -> SheetHeader {
    SheetHeader::new().title(title()).description(description())
}

fn footer() -> SheetFooter {
    SheetFooter::new()
        .child(Button::new("save").label("Save changes"))
        .child(DialogClose::new(
            Button::new("cancel")
                .variant(ButtonVariant::Outline)
                .label("Cancel"),
        ))
}

/// A surface's sample content, pinned to `side`.
pub(super) fn content(side: SheetContentSide) -> SheetContent {
    SheetContent::new()
        .side(side)
        .header(header())
        .child(
            div()
                .px(space::S4)
                .child(stand_in::field(Field::new("Name"))),
        )
        .footer(footer())
}

/// A part at a left or right sheet's width.
fn in_sheet(part: impl gpui::IntoElement) -> gpui::AnyElement {
    div().w(size::SM).child(part).into_any_element()
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "SheetContent",
            properties: &["side"],
            variants: || {
                SheetContentSide::iter()
                    .map(|side| Variant {
                        props: vec![("side", side.into())],
                        story: Some("components-sheet--right"),
                        // Its size in the story's viewport: 384px by the full height at the
                        // left or right, the full width at the top or bottom.
                        render: Box::new(move |_| {
                            let sheet = content(side);
                            match side {
                                SheetContentSide::Left | SheetContentSide::Right => {
                                    sheet.w(size::SM).h(VIEWPORT_HEIGHT)
                                }
                                SheetContentSide::Top | SheetContentSide::Bottom => {
                                    sheet.w(VIEWPORT_WIDTH)
                                }
                            }
                            .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "SheetHeader",
            properties: &[],
            variants: || one(None, || in_sheet(header())),
        },
        Entry {
            name: "SheetTitle",
            properties: &[],
            variants: || one(None, || title().into_any_element()),
        },
        Entry {
            name: "SheetDescription",
            properties: &[],
            variants: || one(None, || description().into_any_element()),
        },
        Entry {
            name: "SheetFooter",
            properties: &[],
            variants: || one(None, || in_sheet(footer())),
        },
    ]
}
