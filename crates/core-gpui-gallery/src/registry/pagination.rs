//! `Pagination` and its parts (`PaginationLink` with Penpot property `isActive`), with the
//! story's Previous, pages 1–3 (2 current), an ellipsis and Next.

use brrritscold_core_gpui::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationLink, PaginationNext,
    PaginationPrevious,
};
use gpui::{IntoElement as _, ParentElement as _, Styled as _};

use super::{Entry, STORY_WIDTH, Variant, one};

fn content() -> PaginationContent {
    PaginationContent::new()
        .child(PaginationPrevious::new("previous"))
        .child(PaginationLink::new("page-1").label("1"))
        .child(PaginationLink::new("page-2").label("2").is_active(true))
        .child(PaginationLink::new("page-3").label("3"))
        .child(PaginationEllipsis::new())
        .child(PaginationNext::new("next"))
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Pagination",
            properties: &[],
            variants: || {
                one(Some("components-pagination--default"), || {
                    Pagination::new()
                        .w(STORY_WIDTH)
                        .child(content())
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "PaginationContent",
            properties: &[],
            variants: || one(None, || content().into_any_element()),
        },
        Entry {
            name: "PaginationLink",
            properties: &["isActive"],
            variants: || {
                [false, true]
                    .into_iter()
                    .map(|is_active| Variant {
                        props: vec![("isActive", if is_active { "true" } else { "false" })],
                        story: None,
                        render: Box::new(move |id| {
                            PaginationLink::new(id)
                                .label("1")
                                .is_active(is_active)
                                .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "PaginationPrevious",
            properties: &[],
            variants: || {
                one(None, || {
                    PaginationPrevious::new("previous").into_any_element()
                })
            },
        },
        Entry {
            name: "PaginationNext",
            properties: &[],
            variants: || one(None, || PaginationNext::new("next").into_any_element()),
        },
        Entry {
            name: "PaginationEllipsis",
            properties: &[],
            variants: || one(None, || PaginationEllipsis::new().into_any_element()),
        },
    ]
}
