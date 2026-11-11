//! `Toast` and its parts, with the `WithAction` story's "Message archived" content. Penpot's
//! main component also shows the `Icon` part; that story's toast has no type, so no icon.

use brrritscold_core_gpui::{Toast, ToastAction, ToastClose, ToastDescription, ToastTitle};
use brrritscold_core_gpui_tokens::primitive::size;
use gpui::{IntoElement as _, Styled as _};

use super::{Entry, one};

const STORY: &str = "components-toast--with-action";

fn title() -> ToastTitle {
    ToastTitle::new("Message archived")
}

fn description() -> ToastDescription {
    ToastDescription::new("It has moved to the archive.")
}

fn action() -> ToastAction {
    ToastAction::new("Undo")
}

/// The toast's sample content, at the viewport's `max-w-sm`.
pub(super) fn toast() -> Toast {
    Toast::new()
        .title(title())
        .description(description())
        .action(action())
        .w(size::SM)
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Toast",
            properties: &[],
            variants: || one(Some(STORY), || toast().into_any_element()),
        },
        Entry {
            name: "ToastTitle",
            properties: &[],
            variants: || one(Some(STORY), || title().into_any_element()),
        },
        Entry {
            name: "ToastDescription",
            properties: &[],
            variants: || one(Some(STORY), || description().into_any_element()),
        },
        Entry {
            name: "ToastAction",
            properties: &[],
            variants: || one(Some(STORY), || action().into_any_element()),
        },
        Entry {
            name: "ToastClose",
            properties: &[],
            variants: || one(Some(STORY), || ToastClose::new().into_any_element()),
        },
    ]
}
