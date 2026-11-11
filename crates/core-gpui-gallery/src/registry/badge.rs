//! `Badge`: Penpot property `variant`.

use brrritscold_core_gpui::{Badge, BadgeVariant};
use gpui::IntoElement as _;
use strum::IntoEnumIterator as _;

use super::{Entry, Variant};

pub(super) fn entry() -> Entry {
    Entry {
        name: "Badge",
        properties: &["variant"],
        variants: || {
            BadgeVariant::iter()
                .map(|variant| Variant {
                    props: vec![("variant", variant.into())],
                    story: Some("components-badge--default"),
                    render: Box::new(move |_| {
                        Badge::new()
                            .variant(variant)
                            .label("Badge")
                            .into_any_element()
                    }),
                })
                .collect()
        },
    }
}
