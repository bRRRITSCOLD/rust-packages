//! `Skeleton`: no variant properties.

use brrritscold_core_gpui::Skeleton;
use gpui::{IntoElement as _, Styled as _, px};

use super::{Entry, one};

pub(super) fn entry() -> Entry {
    Entry {
        name: "Skeleton",
        properties: &[],
        // The story's h-4 w-72.
        variants: || {
            one(Some("components-skeleton--default"), || {
                Skeleton::new().h(px(16.)).w(px(288.)).into_any_element()
            })
        },
    }
}
