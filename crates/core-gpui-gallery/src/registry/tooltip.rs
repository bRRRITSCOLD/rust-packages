//! `TooltipContent`: Penpot property `side`, with the story's "Add to library" text.

use brrritscold_core_gpui::{TooltipContent, TooltipContentSide};
use gpui::IntoElement as _;
use strum::IntoEnumIterator as _;

use super::{Entry, Variant};

pub(super) fn entries() -> Vec<Entry> {
    vec![Entry {
        name: "TooltipContent",
        properties: &["side"],
        variants: || {
            TooltipContentSide::iter()
                .map(|side| Variant {
                    props: vec![("side", side.into())],
                    story: Some("components-tooltip--top"),
                    render: Box::new(move |_| {
                        TooltipContent::new("Add to library")
                            .side(side)
                            .into_any_element()
                    }),
                })
                .collect()
        },
    }]
}
