//! Data Table Pagination: core-react's `data-table-pagination.tsx`, a data table's page controls on
//! core-gpui's [`Pagination`] parts and [`Select`]. Plain props, as core-react's: the app keeps
//! the page and the page size and rebuilds the controls when they change; no table model.
//!
//! Previous on the first page and Next on the last are faded and inert, as core-react's
//! `aria-disabled` links are; here they also leave the Tab order, as a disabled [`Button`]
//! does.
//!
//! [`Button`]: super::button::Button

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{
    App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString, StyleRefinement,
    Styled, Window, div, prelude::FluentBuilder as _,
};
use gpui_base::StyledExt as _;

use super::label::Label;
use super::pagination::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationLink, PaginationNext,
    PaginationPrevious,
};
use super::select::{Select, SelectContent, SelectItem, SelectTrigger, SelectTriggerSize};

type PageHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The page indexes to show, in order: the first, the last, the current one and its
/// neighbours; `None` is a gap.
fn page_range(page_index: usize, page_count: usize) -> Vec<Option<usize>> {
    let mut shown: Vec<usize> = [Some(0), page_index.checked_sub(1), Some(page_index)]
        .into_iter()
        .flatten()
        .chain([page_index + 1, page_count.saturating_sub(1)])
        .filter(|&page| page < page_count)
        .collect();
    shown.sort_unstable();
    shown.dedup();
    let mut range = Vec::new();
    let mut previous: Option<usize> = None;
    for page in shown {
        let gap = page - previous.map_or(0, |p| p + 1);
        // A gap of one page shows that page: an ellipsis would take the same room.
        if gap == 1 {
            range.push(Some(page - 1));
        } else if gap > 1 {
            range.push(None);
        }
        range.push(Some(page));
        previous = Some(page);
    }
    range
}

/// A data table's pagination: Previous, the first and last pages, the current one and its
/// neighbours (an ellipsis for each gap), Next, and, with
/// [`on_page_size_change`](Self::on_page_size_change), a rows-per-page select before them.
///
/// ```no_run
/// use brrritscold_core_gpui::DataTablePagination;
///
/// let pages = DataTablePagination::new("pages")
///     .page_index(0)
///     .page_count(10)
///     .on_page_change(|page, _, _| println!("page {page}"));
/// ```
#[derive(IntoElement)]
pub struct DataTablePagination {
    id: ElementId,
    page_index: usize,
    page_count: usize,
    on_page_change: Option<PageHandler>,
    page_size: Option<usize>,
    page_size_options: Vec<usize>,
    on_page_size_change: Option<PageHandler>,
    style: StyleRefinement,
}

impl DataTablePagination {
    /// The first of one page. `id` keys its controls.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            page_index: 0,
            page_count: 1,
            on_page_change: None,
            page_size: None,
            page_size_options: vec![10, 20, 30, 40, 50],
            on_page_size_change: None,
            style: StyleRefinement::default(),
        }
    }

    /// The current page, from 0.
    pub fn page_index(mut self, page_index: usize) -> Self {
        self.page_index = page_index;
        self
    }

    /// How many pages there are, at least 1.
    pub fn page_count(mut self, page_count: usize) -> Self {
        self.page_count = page_count.max(1);
        self
    }

    /// Runs with the page index to show.
    pub fn on_page_change(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_page_change = Some(Rc::new(handler));
        self
    }

    /// Rows per page, shown in the rows-per-page select.
    pub fn page_size(mut self, page_size: usize) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// The rows-per-page choices; 10 to 50 in tens unless set.
    pub fn page_size_options(mut self, options: impl IntoIterator<Item = usize>) -> Self {
        self.page_size_options = options.into_iter().collect();
        self
    }

    /// Runs with the rows per page picked. Without it there is no rows-per-page select.
    pub fn on_page_size_change(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_page_size_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for DataTablePagination {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let child = |name: &str| {
            ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), name.to_owned().into())
        };
        let page_index = self.page_index.min(self.page_count - 1);
        let first = page_index == 0;
        let last = page_index + 1 >= self.page_count;
        let go = |page: usize| {
            let on_page_change = self.on_page_change.clone();
            move |_: &gpui::ClickEvent, window: &mut Window, cx: &mut App| {
                if let Some(on_page_change) = &on_page_change {
                    on_page_change(page, window, cx);
                }
            }
        };

        // <div className="flex items-center gap-2"><Label>Rows per page</Label><Select …>
        // <SelectTrigger size="sm">
        let page_size = self.on_page_size_change.map(|on_page_size_change| {
            let content =
                self.page_size_options
                    .iter()
                    .fold(SelectContent::new(), |content, size| {
                        let size = SharedString::from(size.to_string());
                        content.item(SelectItem::new(size.clone(), size))
                    });
            let select = Select::new(child("page-size"))
                .trigger(SelectTrigger::new().size(SelectTriggerSize::Sm))
                .content(content)
                .value(self.page_size.map(|size| size.to_string()))
                .on_value_change(move |size, window, cx| {
                    if let Ok(size) = size.parse() {
                        on_page_size_change(size, window, cx);
                    }
                });
            div()
                .flex()
                .items_center()
                .gap(space::S2)
                .child(Label::new().child("Rows per page"))
                .child(select)
        });

        let pages = page_range(page_index, self.page_count)
            .into_iter()
            .enumerate()
            .map(|(slot, page)| match page {
                None => PaginationEllipsis::new().into_any_element(),
                Some(page) => PaginationLink::new(child(&format!("page-{slot}-{page}")))
                    .label((page + 1).to_string())
                    .is_active(page == page_index)
                    .on_click(go(page))
                    .into_any_element(),
            });
        // <Pagination className="mr-0 w-auto">
        let pagination = Pagination::new().w_auto().mx_0().child(
            PaginationContent::new()
                .child(
                    PaginationPrevious::new(child("previous"))
                        .disabled(first)
                        .when(!first, |p| p.on_click(go(page_index.saturating_sub(1)))),
                )
                .children(pages)
                .child(
                    PaginationNext::new(child("next"))
                        .disabled(last)
                        .when(!last, |p| p.on_click(go(page_index + 1))),
                ),
        );
        // flex flex-wrap items-center gap-4
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(space::S4)
            .children(page_size)
            .child(pagination)
            .refine_style(&self.style)
    }
}

impl_styled!(DataTablePagination);

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Modifiers, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use crate::components::test_support::{press, render};

    type Log = Rc<RefCell<Vec<usize>>>;

    /// The text of each page control between Previous and Next: numbers, `…` for a gap.
    fn pages(page_index: usize, page_count: usize) -> Vec<String> {
        page_range(page_index, page_count)
            .into_iter()
            .map(|page| page.map_or("…".into(), |page| (page + 1).to_string()))
            .collect()
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    /// Ten pages at `page_index`, logging every page change.
    fn ten_pages(log: Log, page_index: usize) -> impl Fn() -> DataTablePagination {
        move || {
            let log = log.clone();
            DataTablePagination::new("pages")
                .page_index(page_index)
                .page_count(10)
                .on_page_change(move |page, _, _| log.borrow_mut().push(page))
        }
    }

    mod page_range {
        use super::*;

        #[test]
        fn should_show_the_first_pages_and_the_last_from_the_start() {
            assert_eq!(pages(0, 10), ["1", "2", "…", "10"]);
            assert_eq!(pages(1, 10), ["1", "2", "3", "…", "10"]);
        }

        #[test]
        fn should_put_a_gap_each_side_in_the_middle() {
            assert_eq!(pages(4, 10), ["1", "…", "4", "5", "6", "…", "10"]);
        }

        #[test]
        fn should_show_a_one_page_gap_as_its_page() {
            assert_eq!(pages(3, 10), ["1", "2", "3", "4", "5", "…", "10"]);
        }

        #[test]
        fn should_end_on_the_last_page() {
            assert_eq!(pages(9, 10), ["1", "…", "9", "10"]);
        }

        #[test]
        fn should_show_a_single_page() {
            assert_eq!(pages(0, 1), ["1"]);
        }
    }

    mod pagination {
        use super::*;

        #[gpui::test]
        fn should_go_to_the_next_page(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, bounds) = render(cx, ten_pages(log.clone(), 0));
            cx.simulate_click(
                point(bounds.right() - px(20.), bounds.center().y),
                Modifiers::default(),
            );
            draw(cx);
            assert_eq!(*log.borrow(), [1]);
        }

        #[gpui::test]
        fn should_change_page_from_a_page_link(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, _) = render(cx, ten_pages(log.clone(), 0));
            // Previous is disabled, so it is no Tab stop: 1, then 2.
            for _ in 0..2 {
                cx.update(|window, cx| window.focus_next(cx));
                draw(cx);
            }
            press(cx, "enter");
            assert_eq!(*log.borrow(), [1]);
        }

        #[gpui::test]
        fn should_ignore_previous_on_the_first_page(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, bounds) = render(cx, ten_pages(log.clone(), 0));
            cx.simulate_click(
                point(bounds.left() + px(20.), bounds.center().y),
                Modifiers::default(),
            );
            draw(cx);
            assert!(log.borrow().is_empty());
        }

        #[gpui::test]
        fn should_ignore_next_on_the_last_page(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, bounds) = render(cx, ten_pages(log.clone(), 9));
            cx.simulate_click(
                point(bounds.right() - px(20.), bounds.center().y),
                Modifiers::default(),
            );
            draw(cx);
            assert!(log.borrow().is_empty());
        }
    }
}
