//! The data table parts, with their stories' content: the Email column header in a table's
//! header row (Penpot property `sort`), ten pages with the rows-per-page select, the payments
//! filter reading "ken" (the Default story's play function types it) beside an Export button,
//! and the Email / Amount / Status view options.

use brrritscold_core_gpui::{
    Button, ButtonSize, ButtonVariant, DataTableColumnHeader, DataTablePagination, DataTableSort,
    DataTableToolbar, DataTableViewColumn, DataTableViewOptions, InputState, Table, TableHead,
    TableHeader, TableRow,
};
use gpui::{Context, IntoElement as _, ParentElement as _, Styled as _, Window};
use strum::IntoEnumIterator as _;

use super::support::Stateful;
use super::{Entry, STORY_WIDTH, Variant, one};

/// The story a column header's `sort` is captured from. Each story's play function picks
/// another order (Desc, or Asc from Descending) and the header keeps it, so `asc` is shot from
/// the Descending story and `desc` from the Ascending one. No story ends unsorted.
fn story(sort: DataTableSort) -> Option<&'static str> {
    match sort {
        DataTableSort::None => None,
        DataTableSort::Asc => Some("components-datatablecolumnheader--descending"),
        DataTableSort::Desc => Some("components-datatablecolumnheader--ascending"),
    }
}

/// The Default story's filter after its play function typed into it.
fn ken(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
    InputState::new(window, cx)
        .placeholder("Filter payments…")
        .default_value("ken")
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "DataTableColumnHeader",
            properties: &["sort"],
            variants: || {
                DataTableSort::iter()
                    .map(|sort| Variant {
                        props: vec![("sort", sort.into())],
                        story: story(sort),
                        // The story's header cell: <Table><TableHeader><TableRow><TableHead>
                        render: Box::new(move |id| {
                            let header = DataTableColumnHeader::new(id, "Email")
                                .sort(sort)
                                .on_sort_change(|_, _, _| {})
                                .on_hide(|_, _| {});
                            Table::new()
                                .w(STORY_WIDTH)
                                .header(
                                    TableHeader::new()
                                        .row(TableRow::new().child(TableHead::new().child(header))),
                                )
                                .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "DataTablePagination",
            properties: &[],
            variants: || {
                one(Some("components-datatablepagination--page-size"), || {
                    DataTablePagination::new("pagination")
                        .page_index(0)
                        .page_count(10)
                        .on_page_change(|_, _, _| {})
                        .page_size(10)
                        .on_page_size_change(|_, _, _| {})
                        .w(STORY_WIDTH)
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "DataTableToolbar",
            properties: &[],
            variants: || {
                vec![Variant {
                    props: Vec::new(),
                    story: Some("components-datatabletoolbar--default"),
                    render: Box::new(|id| {
                        Stateful::new(id, ken, |filter| {
                            DataTableToolbar::new(filter)
                                .w(STORY_WIDTH)
                                .child(
                                    Button::new("export")
                                        .variant(ButtonVariant::Outline)
                                        .size(ButtonSize::Sm)
                                        .label("Export"),
                                )
                                .into_any_element()
                        })
                        .into_any_element()
                    }),
                }]
            },
        },
        Entry {
            name: "DataTableViewOptions",
            properties: &[],
            variants: || {
                one(Some("components-datatableviewoptions--default"), || {
                    DataTableViewOptions::new("view-options")
                        .columns([
                            DataTableViewColumn::new("email", "Email", true),
                            DataTableViewColumn::new("amount", "Amount", true),
                            DataTableViewColumn::new("status", "Status", false),
                        ])
                        .into_any_element()
                })
            },
        },
    ]
}
