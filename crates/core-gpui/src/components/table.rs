//! Table: core-react's `table.tsx`, plain styled elements.
//!
//! GPUI has no table layout, so a row is a flex row and its cells share it: every cell grows
//! from a zero basis by its column span, so columns line up across rows and a cell spanning
//! three columns is exactly three cells wide. Columns are equal unless a cell sets its own
//! width (`.w(..)`, with `.flex_none()`), where an HTML table sizes them to their content.
//!
//! The row borders hang off the row's section in core-react (`[&_tr]:border-b`,
//! `[&_tr:last-child]:border-0`, `[&>tr]:last:border-b-0`); here each section takes its rows as
//! typed builders and tells each one whether it is ruled.
//!
//! Not mirrored: the container's `overflow-x-auto` (a scroll needs state the table doesn't
//! keep: put a wide table in a scrolling element), `align-middle` beyond centring a cell's
//! content, and `[&:has([role=checkbox])]:pr-0`.

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, space};
use gpui::{
    AnyElement, App, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};
use gpui_base::StyledExt as _;

use crate::theme::ActiveTheme as _;

/// A table, styled like core-react's `Table`. Its sections are builders:
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Table, TableBody, TableCaption, TableCell, TableHead, TableHeader, TableRow,
/// };
/// use gpui::{ParentElement as _, Styled as _};
///
/// let invoices = Table::new()
///     .caption(TableCaption::new("Recent invoices."))
///     .header(TableHeader::new().row(
///         TableRow::new()
///             .child(TableHead::new().child("Invoice"))
///             .child(TableHead::new().text_right().child("Amount")),
///     ))
///     .body(TableBody::new().row(
///         TableRow::new()
///             .child(TableCell::new().child("INV001"))
///             .child(TableCell::new().text_right().child("$250.00")),
///     ));
/// ```
#[derive(IntoElement)]
pub struct Table {
    caption: Option<TableCaption>,
    header: Option<TableHeader>,
    body: Option<TableBody>,
    footer: Option<TableFooter>,
    style: StyleRefinement,
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    /// An empty table. It fills its container's width.
    pub fn new() -> Self {
        Self {
            caption: None,
            header: None,
            body: None,
            footer: None,
            style: StyleRefinement::default(),
        }
    }

    /// The `Caption` part, under the table (`caption-bottom`).
    pub fn caption(mut self, caption: TableCaption) -> Self {
        self.caption = Some(caption);
        self
    }

    /// The header rows (Penpot's `Headings` row).
    pub fn header(mut self, header: TableHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// The body rows.
    pub fn body(mut self, body: TableBody) -> Self {
        self.body = Some(body);
        self
    }

    /// The `Footer` part.
    pub fn footer(mut self, footer: TableFooter) -> Self {
        self.footer = Some(footer);
        self
    }
}

impl RenderOnce for Table {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // relative w-full overflow-x-auto > table: w-full caption-bottom text-sm
        div()
            .relative()
            .w_full()
            .flex()
            .flex_col()
            .text_size(font_size::SM)
            .children(self.header)
            .children(self.body)
            .children(self.footer)
            .children(self.caption)
            .refine_style(&self.style)
    }
}

/// Declares a table section: typed rows, each ruled or not by `$ruled(index, len)`.
macro_rules! section {
    ($(#[$doc:meta])* $section:ident, $ruled:expr) => {
        $(#[$doc])*
        #[derive(IntoElement)]
        pub struct $section {
            rows: Vec<TableRow>,
            style: StyleRefinement,
        }

        impl Default for $section {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $section {
            /// An empty section.
            pub fn new() -> Self {
                Self {
                    rows: Vec::new(),
                    style: StyleRefinement::default(),
                }
            }

            /// Adds a row.
            pub fn row(mut self, row: TableRow) -> Self {
                self.rows.push(row);
                self
            }

            /// Adds rows.
            pub fn rows(mut self, rows: impl IntoIterator<Item = TableRow>) -> Self {
                self.rows.extend(rows);
                self
            }

            fn ruled_rows(self) -> impl Iterator<Item = TableRow> {
                let len = self.rows.len();
                let ruled: fn(usize, usize) -> bool = $ruled;
                self.rows
                    .into_iter()
                    .enumerate()
                    .map(move |(index, row)| row.ruled(ruled(index, len)))
            }
        }

        impl_styled!($section);
    };
}

section! {
    /// The table's header rows: `[&_tr]:border-b`, every row ruled. Code only, as in
    /// core-react: Penpot draws the header as a plain `Headings` row.
    TableHeader,
    |_, _| true
}

section! {
    /// The table's body rows: `[&_tr:last-child]:border-0`, every row ruled but the last.
    /// Code only, as in core-react: Penpot draws the body as plain rows.
    TableBody,
    |index, len| index + 1 < len
}

section! {
    /// The table's `Footer` part: ruled above, tinted, in `font-medium`, its last row unruled
    /// (`[&>tr]:last:border-b-0`).
    TableFooter,
    |index, len| index + 1 < len
}

impl RenderOnce for TableHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let style = self.style.clone();
        div()
            .flex()
            .flex_col()
            .children(self.ruled_rows())
            .refine_style(&style)
    }
}

impl RenderOnce for TableBody {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let style = self.style.clone();
        div()
            .flex()
            .flex_col()
            .children(self.ruled_rows())
            .refine_style(&style)
    }
}

impl RenderOnce for TableFooter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let style = self.style.clone();
        // border-t bg-muted/50 font-medium
        div()
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(s.border)
            .bg(s.muted.opacity(0.5))
            .font_weight(font_weight::MEDIUM)
            .children(self.ruled_rows())
            .refine_style(&style)
    }
}

/// The table's `Row` part: its children are its cells.
#[derive(IntoElement)]
pub struct TableRow {
    selected: bool,
    ruled: bool,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for TableRow {
    fn default() -> Self {
        Self::new()
    }
}

impl TableRow {
    /// An empty row, ruled below until its section says otherwise.
    pub fn new() -> Self {
        Self {
            selected: false,
            ruled: true,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Mutes the row (`data-state="selected"`), as a selected row in a data table.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn ruled(mut self, ruled: bool) -> Self {
        self.ruled = ruled;
        self
    }
}

impl RenderOnce for TableRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let hover = s.muted.opacity(0.5);
        // border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted
        div()
            .flex()
            .w_full()
            .when(self.ruled, |d| d.border_b_1().border_color(s.border))
            .map(|d| {
                if self.selected {
                    d.bg(s.muted)
                } else {
                    d.hover(move |style| style.bg(hover))
                }
            })
            .children(self.children)
            .refine_style(&self.style)
    }
}

/// Declares a cell part: open content in a column `span` columns wide.
macro_rules! cell {
    ($(#[$doc:meta])* $cell:ident) => {
        $(#[$doc])*
        #[derive(IntoElement)]
        pub struct $cell {
            span: u16,
            children: Vec<AnyElement>,
            style: StyleRefinement,
        }

        impl Default for $cell {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $cell {
            /// An empty cell, one column wide; its children are its content.
            pub fn new() -> Self {
                Self {
                    span: 1,
                    children: Vec::new(),
                    style: StyleRefinement::default(),
                }
            }

            /// Spans `columns` columns (`colSpan`).
            pub fn col_span(mut self, columns: u16) -> Self {
                self.span = columns.max(1);
                self
            }
        }

        impl_styled!($cell);
        impl_parent!($cell);
    };
}

cell! {
    /// The table's `Head` part: a column heading, 40px tall.
    TableHead
}

cell! {
    /// The table's `Cell` part.
    TableCell
}

/// A cell: its share of the row, then its padded content centred down it (`align-middle`).
///
/// The share is an outer element with no padding, so columns line up whatever a cell's span:
/// a flex item's padding comes off the free space before it is shared out. The content is a
/// column flexbox, which stretches it across the cell, so `text-right` lines text up at the
/// right. A caller's style lands on the outer element; text styles reach the content by
/// inheritance.
fn cell(
    span: u16,
    content: gpui::Div,
    children: Vec<AnyElement>,
    style: &StyleRefinement,
) -> gpui::Div {
    div()
        .flex()
        .flex_grow(f32::from(span))
        .flex_basis(gpui::relative(0.))
        .min_w_0()
        .child(
            content
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .justify_center()
                .whitespace_nowrap()
                .children(children),
        )
        .refine_style(style)
}

impl RenderOnce for TableHead {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // h-10 px-2 text-left align-middle font-medium whitespace-nowrap text-foreground
        let content = div()
            .h(space::S10)
            .px(space::S2)
            .font_weight(font_weight::MEDIUM)
            .text_color(cx.theme().semantic().foreground);
        cell(self.span, content, self.children, &self.style)
    }
}

impl RenderOnce for TableCell {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // p-2 align-middle whitespace-nowrap
        cell(self.span, div().p(space::S2), self.children, &self.style)
    }
}

/// The table's `Caption` part, centred under the table.
#[derive(IntoElement)]
pub struct TableCaption {
    text: SharedString,
    style: StyleRefinement,
}

impl TableCaption {
    /// A caption reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for TableCaption {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // mt-4 text-sm text-muted-foreground; a <caption> centres its text.
        div()
            .mt(space::S4)
            .text_size(font_size::SM)
            .text_color(cx.theme().semantic().muted_foreground)
            .text_center()
            .child(self.text)
            .refine_style(&self.style)
    }
}

impl_styled!(Table, TableRow, TableCaption);
impl_parent!(TableRow);

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{Pixels, TestAppContext, px};

    use super::*;
    use crate::components::test_support::render;

    /// Content that records its laid-out width, which is its cell's less the padding.
    fn measured(text: &'static str, widths: Rc<RefCell<Vec<Pixels>>>) -> impl IntoElement {
        div()
            .child(div().child(text))
            .on_children_prepainted(move |content, _, _| {
                widths.borrow_mut().push(content[0].size.width);
            })
    }

    /// A 400px table whose body is `rows`.
    fn table(rows: impl IntoIterator<Item = TableRow>) -> Table {
        Table::new().w(px(400.)).body(TableBody::new().rows(rows))
    }

    mod row {
        use super::*;

        #[gpui::test]
        fn should_size_cells_by_their_span(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let widths = Rc::new(RefCell::new(Vec::new()));
            render(cx, {
                let widths = widths.clone();
                move || {
                    table([TableRow::new()
                        .child(
                            TableCell::new()
                                .col_span(3)
                                .child(measured("Total", widths.clone())),
                        )
                        .child(TableCell::new().child(measured("$750.00", widths.clone())))])
                }
            });
            // Three quarters and one quarter of 400px, less p-2 on each side.
            assert_eq!(widths.borrow()[..2], [px(300. - 16.), px(100. - 16.)]);
        }

        #[gpui::test]
        fn should_be_40px_tall_with_heads_and_37px_with_cells(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, head) = render(cx, || {
                table([TableRow::new().child(TableHead::new().child("Invoice"))])
            });
            let (_, cell) = render(cx, || {
                table([TableRow::new().child(TableCell::new().child("INV001"))])
            });
            assert_eq!(head.size.height, px(40.));
            // p-2 around a text-sm line at 1.5
            assert_eq!(cell.size.height, px(8. + 21. + 8.));
        }
    }

    mod sections {
        use super::*;

        fn rows(n: usize) -> impl Iterator<Item = TableRow> {
            (0..n).map(|_| TableRow::new().child(TableCell::new().child("x")))
        }

        #[gpui::test]
        fn should_rule_every_body_row_but_the_last(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || table(rows(3)));
            assert_eq!(bounds.size.height, px(3. * 37. + 2.));
        }

        #[gpui::test]
        fn should_rule_every_header_row(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || Table::new().header(TableHeader::new().rows(rows(2))));
            assert_eq!(bounds.size.height, px(2. * 38.));
        }

        #[gpui::test]
        fn should_rule_the_footer_above_and_not_below(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || Table::new().footer(TableFooter::new().rows(rows(1))));
            assert_eq!(bounds.size.height, px(1. + 37.));
        }
    }
}
