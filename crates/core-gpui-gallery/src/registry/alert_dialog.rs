//! `AlertDialogContent` (Penpot property `size`) and its parts, with the story's "Delete your
//! account?" content. Penpot's main component also shows the `Media` part; the story the
//! variants are captured from has none, so the content entry has none either, and the header
//! entry shows it.

use brrritscold_core_gpui::{
    AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogContentSize,
    AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogMedia,
    AlertDialogOverlay, AlertDialogTitle, ButtonVariant, IconName,
};
use gpui::{IntoElement as _, ParentElement as _, Styled as _, div, px};
use strum::IntoEnumIterator as _;

use super::{Entry, Variant, one};

fn title() -> AlertDialogTitle {
    AlertDialogTitle::new("Delete your account?")
}

fn description() -> AlertDialogDescription {
    AlertDialogDescription::new("This deletes your account and its data, and cannot be undone.")
}

fn header() -> AlertDialogHeader {
    AlertDialogHeader::new()
        .title(title())
        .description(description())
}

fn cancel() -> AlertDialogCancel {
    AlertDialogCancel::new("cancel", "Cancel")
}

fn action() -> AlertDialogAction {
    AlertDialogAction::new("delete", "Delete").variant(ButtonVariant::Destructive)
}

fn footer() -> AlertDialogFooter {
    AlertDialogFooter::new().child(cancel()).child(action())
}

/// A surface's sample content, at `size`.
pub(super) fn content(size: AlertDialogContentSize) -> AlertDialogContent {
    AlertDialogContent::new()
        .size(size)
        .header(header())
        .footer(footer())
}

/// A part at the `default` surface's content width: 512px less its 24px padding each side.
fn in_content(part: impl gpui::IntoElement) -> gpui::AnyElement {
    div().w(px(464.)).child(part).into_any_element()
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "AlertDialogContent",
            properties: &["size"],
            variants: || {
                AlertDialogContentSize::iter()
                    .map(|size| Variant {
                        props: vec![("size", size.into())],
                        story: Some("components-alertdialog--default"),
                        render: Box::new(move |_| content(size).into_any_element()),
                    })
                    .collect()
            },
        },
        Entry {
            name: "AlertDialogHeader",
            properties: &[],
            variants: || {
                one(None, || {
                    in_content(header().media(AlertDialogMedia::new(IconName::Trash)))
                })
            },
        },
        Entry {
            name: "AlertDialogMedia",
            properties: &[],
            variants: || {
                one(None, || {
                    AlertDialogMedia::new(IconName::Trash).into_any_element()
                })
            },
        },
        Entry {
            name: "AlertDialogTitle",
            properties: &[],
            variants: || one(None, || title().into_any_element()),
        },
        Entry {
            name: "AlertDialogDescription",
            properties: &[],
            variants: || one(None, || in_content(description())),
        },
        Entry {
            name: "AlertDialogFooter",
            properties: &[],
            variants: || one(None, || in_content(footer())),
        },
        Entry {
            name: "AlertDialogAction",
            properties: &[],
            variants: || one(None, || action().into_any_element()),
        },
        Entry {
            name: "AlertDialogCancel",
            properties: &[],
            variants: || one(None, || cancel().into_any_element()),
        },
        Entry {
            name: "AlertDialogOverlay",
            properties: &[],
            // The backdrop fills the window; here, a 160x90 one.
            variants: || {
                one(None, || {
                    div()
                        .relative()
                        .w(px(160.))
                        .h(px(90.))
                        .child(AlertDialogOverlay::new())
                        .into_any_element()
                })
            },
        },
    ]
}
