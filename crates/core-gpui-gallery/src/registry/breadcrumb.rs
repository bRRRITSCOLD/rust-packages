//! `Breadcrumb` and its parts, with the story's Home › Components › Breadcrumb trail. Penpot's
//! main component also collapses a run into an `Ellipsis`; the story it is captured from
//! doesn't, so neither does the entry.

use brrritscold_core_gpui::{
    Breadcrumb, BreadcrumbEllipsis, BreadcrumbLink, BreadcrumbList, BreadcrumbPage,
    BreadcrumbSeparator,
};
use gpui::{AnyElement, IntoElement, ParentElement as _, Styled as _};

use super::{Entry, STORY_WIDTH, one};

fn list() -> BreadcrumbList {
    BreadcrumbList::new()
        .child(BreadcrumbLink::new("home").label("Home"))
        .child(BreadcrumbSeparator::new())
        .child(BreadcrumbLink::new("components").label("Components"))
        .child(BreadcrumbSeparator::new())
        .child(BreadcrumbPage::new("Breadcrumb"))
}

/// A part on its own, in a list so it takes the list's muted `text-sm`.
fn listed(part: impl IntoElement) -> AnyElement {
    BreadcrumbList::new().child(part).into_any_element()
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Breadcrumb",
            properties: &[],
            variants: || {
                one(Some("components-breadcrumb--default"), || {
                    Breadcrumb::new()
                        .w(STORY_WIDTH)
                        .child(list())
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "BreadcrumbList",
            properties: &[],
            variants: || one(None, || list().into_any_element()),
        },
        Entry {
            name: "BreadcrumbLink",
            properties: &[],
            variants: || one(None, || listed(BreadcrumbLink::new("home").label("Home"))),
        },
        Entry {
            name: "BreadcrumbPage",
            properties: &[],
            variants: || one(None, || listed(BreadcrumbPage::new("Breadcrumb"))),
        },
        Entry {
            name: "BreadcrumbSeparator",
            properties: &[],
            variants: || one(None, || listed(BreadcrumbSeparator::new())),
        },
        Entry {
            name: "BreadcrumbEllipsis",
            properties: &[],
            variants: || one(None, || listed(BreadcrumbEllipsis::new())),
        },
    ]
}
