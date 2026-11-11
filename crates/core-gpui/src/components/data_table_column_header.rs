//! Data Table Column Header: core-react's `data-table-column-header.tsx`, a data table's column
//! title that sorts, on core-gpui's [`DropdownMenu`] and [`Button`]. Plain props, as core-react's:
//! the app keeps the sort order and rebuilds the header when it changes; no table model.
//!
//! Not mirrored: `aria-sort`, which core-react puts on the `th` around the header and GPUI's
//! accessibility tree has no table header to carry.

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{
    App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString, StyleRefinement,
    Styled, Window, div,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use super::button::{Button, ButtonSize, ButtonVariant};
use super::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuRadioItem,
    DropdownMenuSeparator,
};
use crate::icons::IconName;

/// Penpot variant property `sort` of `DataTableColumnHeader`: a column's sort order (core-react's
/// `DataTableSort`). Cases are its values in PascalCase; [`Display`] gives the Penpot value back.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum DataTableSort {
    /// `none`: unsorted, `ChevronsUpDown`.
    #[default]
    None,
    /// `asc`: ascending, `ArrowUp`.
    Asc,
    /// `desc`: descending, `ArrowDown`.
    Desc,
}

impl DataTableSort {
    /// `SORT_ICON`: the trigger's icon, and the menu item's.
    fn icon(self) -> IconName {
        match self {
            DataTableSort::None => IconName::ChevronsUpDown,
            DataTableSort::Asc => IconName::ArrowUp,
            DataTableSort::Desc => IconName::ArrowDown,
        }
    }
}

type SortChangeHandler = Rc<dyn Fn(DataTableSort, &mut Window, &mut App)>;
type HideHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A data table's column header: its title as a ghost button opening a menu of sort orders
/// (Asc, Desc, None; the current one checked) and, with [`on_hide`](Self::on_hide), Hide.
/// Without [`on_sort_change`](Self::on_sort_change) the column can't sort and the header is its
/// title.
///
/// ```no_run
/// use brrritscold_core_gpui::{DataTableColumnHeader, DataTableSort};
///
/// let email = DataTableColumnHeader::new("email", "Email")
///     .sort(DataTableSort::Asc)
///     .on_sort_change(|sort, _, _| println!("{sort}"));
/// ```
#[derive(IntoElement)]
pub struct DataTableColumnHeader {
    id: ElementId,
    title: SharedString,
    sort: DataTableSort,
    on_sort_change: Option<SortChangeHandler>,
    on_hide: Option<HideHandler>,
    style: StyleRefinement,
}

impl DataTableColumnHeader {
    /// An unsorted header reading `title`. `id` keys its menu.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            sort: DataTableSort::default(),
            on_sort_change: None,
            on_hide: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `sort`.
    pub fn sort(mut self, sort: DataTableSort) -> Self {
        self.sort = sort;
        self
    }

    /// Runs with the order picked from the menu. Without it the column can't sort.
    pub fn on_sort_change(
        mut self,
        handler: impl Fn(DataTableSort, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_sort_change = Some(Rc::new(handler));
        self
    }

    /// Hides the column; adds a Hide item to the menu.
    pub fn on_hide(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_hide = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for DataTableColumnHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let Some(on_sort_change) = self.on_sort_change else {
            return div().child(self.title).refine_style(&self.style);
        };
        let choice = |sort: DataTableSort, label: &'static str| {
            let on_sort_change = on_sort_change.clone();
            DropdownMenuRadioItem::new(label)
                .icon(sort.icon())
                .checked(self.sort == sort)
                .close_on_click(true)
                .on_select(move |_, window, cx| on_sort_change(sort, window, cx))
        };
        let mut menu = DropdownMenuContent::new()
            .radio_item(choice(DataTableSort::Asc, "Asc"))
            .radio_item(choice(DataTableSort::Desc, "Desc"))
            .radio_item(choice(DataTableSort::None, "None"));
        if let Some(on_hide) = self.on_hide {
            menu = menu.separator(DropdownMenuSeparator::new()).item(
                DropdownMenuItem::new("Hide")
                    .icon(IconName::EyeOff)
                    .on_click(move |_, window, cx| on_hide(window, cx)),
            );
        }
        // <Button variant="ghost" size="sm" className="-ml-3"><span>{title}</span><SortIcon /></Button>:
        // the icon has no data-icon, so the button keeps px-2.5 on both sides.
        let trigger = Button::new(ElementId::NamedChild(
            std::sync::Arc::new(self.id.clone()),
            "trigger".into(),
        ))
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .label(self.title)
        .icon_end(self.sort.icon())
        .pr(space::S2_5)
        .ml(-space::S3);
        // flex items-center gap-2
        div()
            .flex()
            .items_center()
            .gap(space::S2)
            .child(DropdownMenu::new(self.id).trigger(trigger).content(menu))
            .refine_style(&self.style)
    }
}

impl_styled!(DataTableColumnHeader);

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext, point, px};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::render;

    type Log = Rc<RefCell<Vec<String>>>;

    /// Room left of the header for its -12px margin, and above it.
    const INSET: Pixels = px(40.);

    /// The story's Email header at `sort`, logging picks and hides.
    fn email(log: Log, sort: DataTableSort) -> impl Fn() -> gpui::Div {
        move || {
            let sorted = log.clone();
            let hidden = log.clone();
            div().pl(INSET).pt(INSET).child(
                DataTableColumnHeader::new("email", "Email")
                    .sort(sort)
                    .on_sort_change(move |sort, _, _| sorted.borrow_mut().push(sort.to_string()))
                    .on_hide(move |_, _| hidden.borrow_mut().push("hide".into())),
            )
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    /// Opens the menu from the trigger and returns the trigger's top-left.
    fn open(
        cx: &mut TestAppContext,
        log: Log,
        sort: DataTableSort,
    ) -> (&mut VisualTestContext, Bounds<Pixels>) {
        cx.update(crate::init);
        let (cx, page) = render(cx, email(log, sort));
        let trigger = Bounds::new(
            page.origin + point(INSET - px(12.), INSET),
            gpui::size(px(60.), px(32.)),
        );
        cx.simulate_click(trigger.center(), Modifiers::default());
        draw(cx);
        (cx, trigger)
    }

    /// Menu row `n` (0 = Asc): 4px under the trigger, the menu's 4px padding, then 33px rows;
    /// the separator before Hide takes 9px.
    fn row(trigger: Bounds<Pixels>, n: usize) -> gpui::Point<Pixels> {
        let separator = if n > 2 { 9. } else { 0. };
        let top = trigger.bottom() + px(4. + 4. + separator) + px(33.) * n as f32;
        point(trigger.left() + px(30.), top + px(16.))
    }

    mod sort {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&str> = DataTableSort::iter().map(Into::into).collect();
            assert_eq!(values, ["none", "asc", "desc"]);
        }

        #[test]
        fn should_show_an_icon_per_order() {
            assert_eq!(DataTableSort::None.icon(), IconName::ChevronsUpDown);
            assert_eq!(DataTableSort::Asc.icon(), IconName::ArrowUp);
            assert_eq!(DataTableSort::Desc.icon(), IconName::ArrowDown);
        }
    }

    mod column_header {
        use super::*;

        #[gpui::test]
        fn should_toggle_the_sort_from_the_menu_and_close(cx: &mut TestAppContext) {
            let log = Log::default();
            let (cx, trigger) = open(cx, log.clone(), DataTableSort::None);
            assert!(cx.debug_bounds("dropdown-menu-content").is_some());
            cx.simulate_click(row(trigger, 1), Modifiers::default());
            draw(cx);
            assert_eq!(*log.borrow(), ["desc"]);
            assert!(
                cx.debug_bounds("dropdown-menu-content").is_none(),
                "a pick closes the menu"
            );
        }

        #[gpui::test]
        fn should_hide_the_column(cx: &mut TestAppContext) {
            let log = Log::default();
            let (cx, trigger) = open(cx, log.clone(), DataTableSort::Asc);
            cx.simulate_click(row(trigger, 3), Modifiers::default());
            draw(cx);
            assert_eq!(*log.borrow(), ["hide"]);
        }

        #[gpui::test]
        fn should_be_its_title_when_it_cannot_sort(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (cx, header) = render(cx, || DataTableColumnHeader::new("email", "Email"));
            cx.simulate_click(header.center(), Modifiers::default());
            draw(cx);
            assert!(cx.debug_bounds("dropdown-menu-content").is_none());
            assert!(header.size.height < px(32.), "no button");
        }
    }
}
