//! `Label`: no variant properties.

use brrritscold_core_gpui::Label;
use gpui::{IntoElement as _, ParentElement as _};

use super::{Entry, one};

pub(super) fn entry() -> Entry {
    Entry {
        name: "Label",
        properties: &[],
        // The story pairs the label with an Input below it, which core-gpui doesn't have yet.
        variants: || {
            one(Some("components-label--default"), || {
                Label::new().child("Email").into_any_element()
            })
        },
    }
}
