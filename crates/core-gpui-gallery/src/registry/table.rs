//! `Table` and its parts, with the story's invoices. Penpot's main component has three columns
//! and no selected row; the story it is captured from has four (with Method) and selects
//! INV002, and the entry follows the story.

use brrritscold_core_gpui::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};
use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight};
use gpui::{AnyElement, IntoElement, ParentElement as _, Styled as _, div, px};

use super::{Entry, STORY_WIDTH, one};

const INVOICES: [[&str; 4]; 3] = [
    ["INV001", "Paid", "Credit card", "$250.00"],
    ["INV002", "Pending", "PayPal", "$150.00"],
    ["INV003", "Unpaid", "Bank transfer", "$350.00"],
];

fn caption() -> TableCaption {
    TableCaption::new("A list of your recent invoices.")
}

fn headings() -> TableRow {
    TableRow::new()
        .child(TableHead::new().child("Invoice"))
        .child(TableHead::new().child("Status"))
        .child(TableHead::new().child("Method"))
        .child(TableHead::new().text_right().child("Amount"))
}

fn row([invoice, status, method, amount]: [&'static str; 4]) -> TableRow {
    TableRow::new()
        .selected(invoice == "INV002")
        .child(
            TableCell::new()
                .font_weight(font_weight::MEDIUM)
                .child(invoice),
        )
        .child(TableCell::new().child(status))
        .child(TableCell::new().child(method))
        .child(TableCell::new().text_right().child(amount))
}

fn footer() -> TableFooter {
    TableFooter::new().row(
        TableRow::new()
            .child(TableCell::new().col_span(3).child("Total"))
            .child(TableCell::new().text_right().child("$750.00")),
    )
}

/// A part on its own, at a table's width and `text-sm`.
fn in_table(width: f32, part: impl IntoElement) -> AnyElement {
    div()
        .w(px(width))
        .text_size(font_size::SM)
        .child(part)
        .into_any_element()
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Table",
            properties: &[],
            variants: || {
                one(Some("components-table--default"), || {
                    Table::new()
                        .w(STORY_WIDTH)
                        .caption(caption())
                        .header(TableHeader::new().row(headings()))
                        .body(TableBody::new().rows(INVOICES.map(row)))
                        .footer(footer())
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "TableCaption",
            properties: &[],
            variants: || one(None, || in_table(304., caption())),
        },
        Entry {
            name: "TableHead",
            properties: &[],
            variants: || one(None, || in_table(152., TableHead::new().child("Invoice"))),
        },
        Entry {
            name: "TableCell",
            properties: &[],
            variants: || one(None, || in_table(152., TableCell::new().child("INV001"))),
        },
        Entry {
            name: "TableRow",
            properties: &[],
            variants: || one(None, || in_table(608., row(INVOICES[0]))),
        },
        Entry {
            name: "TableFooter",
            properties: &[],
            variants: || one(None, || in_table(608., footer())),
        },
    ]
}
