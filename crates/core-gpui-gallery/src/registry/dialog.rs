//! `DialogContent` and its parts, with the story's "Edit profile" content. Penpot's main
//! component holds a slot placeholder where the story has its name field.

use brrritscold_core_gpui::{
    Button, ButtonVariant, DialogClose, DialogContent, DialogDescription, DialogFooter,
    DialogHeader, DialogOverlay, DialogTitle,
};
use gpui::{IntoElement as _, ParentElement as _, Styled as _, div, px};

use super::stand_in::{self, Field};
use super::{Entry, one};

fn title() -> DialogTitle {
    DialogTitle::new("Edit profile")
}

fn description() -> DialogDescription {
    DialogDescription::new("Make changes to your profile here.")
}

fn header() -> DialogHeader {
    DialogHeader::new()
        .title(title())
        .description(description())
}

fn footer() -> DialogFooter {
    DialogFooter::new()
        .child(DialogClose::new(
            Button::new("cancel")
                .variant(ButtonVariant::Outline)
                .label("Cancel"),
        ))
        .child(Button::new("save").label("Save changes"))
}

/// The surface's sample content: the story's header, name field and footer.
pub(super) fn content() -> DialogContent {
    DialogContent::new()
        .header(header())
        .child(stand_in::field(Field::new("Name")))
        .footer(footer())
}

/// A part at the surface's content width: 448px less its 24px padding each side.
fn in_content(part: impl gpui::IntoElement) -> gpui::AnyElement {
    div().w(px(400.)).child(part).into_any_element()
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "DialogContent",
            properties: &[],
            variants: || {
                one(Some("components-dialog--default"), || {
                    content().into_any_element()
                })
            },
        },
        Entry {
            name: "DialogHeader",
            properties: &[],
            variants: || one(None, || in_content(header())),
        },
        Entry {
            name: "DialogTitle",
            properties: &[],
            variants: || one(None, || title().into_any_element()),
        },
        Entry {
            name: "DialogDescription",
            properties: &[],
            variants: || one(None, || description().into_any_element()),
        },
        Entry {
            name: "DialogFooter",
            properties: &[],
            variants: || one(None, || in_content(footer())),
        },
        Entry {
            name: "DialogOverlay",
            properties: &[],
            // The backdrop fills the window; here, a 160x90 one.
            variants: || {
                one(None, || {
                    div()
                        .relative()
                        .w(px(160.))
                        .h(px(90.))
                        .child(DialogOverlay::new())
                        .into_any_element()
                })
            },
        },
    ]
}
