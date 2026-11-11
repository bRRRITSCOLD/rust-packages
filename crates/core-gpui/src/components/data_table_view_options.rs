//! Data Table View Options: core-react's `data-table-view-options.tsx`, a data table's column
//! visibility menu on core-gpui's [`DropdownMenu`] and [`Button`]. Plain props, as core-react's:
//! the app keeps which columns show and rebuilds the menu when that changes; no table model.

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{
    App, ElementId, IntoElement, RenderOnce, SharedString, StyleRefinement, Styled, Window,
};
use gpui_base::StyledExt as _;

use super::button::{Button, ButtonSize, ButtonVariant};
use super::dropdown_menu::{
    DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuLabel,
    DropdownMenuSeparator,
};
use crate::icons::IconName;

/// A column the menu can show or hide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTableViewColumn {
    /// The column's id, given back to [`DataTableViewOptions::on_visibility_change`].
    pub id: SharedString,
    /// What the menu calls it.
    pub label: SharedString,
    /// Whether it shows.
    pub visible: bool,
}

impl DataTableViewColumn {
    /// A column `id`, labelled `label`, showing or not.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>, visible: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            visible,
        }
    }
}

type VisibilityHandler = Rc<dyn Fn(&SharedString, bool, &mut Window, &mut App)>;

/// A data table's view options: an outline "View" button (`Settings2`) opening "Toggle
/// columns", a checkbox item per column, checked while it shows. A pick keeps the menu open.
///
/// ```no_run
/// use brrritscold_core_gpui::{DataTableViewColumn, DataTableViewOptions};
///
/// let view = DataTableViewOptions::new("view")
///     .columns([
///         DataTableViewColumn::new("email", "Email", true),
///         DataTableViewColumn::new("status", "Status", false),
///     ])
///     .on_visibility_change(|id, visible, _, _| println!("{id}: {visible}"));
/// ```
#[derive(IntoElement)]
pub struct DataTableViewOptions {
    id: ElementId,
    columns: Vec<DataTableViewColumn>,
    on_visibility_change: Option<VisibilityHandler>,
    style: StyleRefinement,
}

impl DataTableViewOptions {
    /// A menu with no columns. `id` keys its state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            columns: Vec::new(),
            on_visibility_change: None,
            style: StyleRefinement::default(),
        }
    }

    /// The columns that can hide, in order.
    pub fn columns(mut self, columns: impl IntoIterator<Item = DataTableViewColumn>) -> Self {
        self.columns = columns.into_iter().collect();
        self
    }

    /// Runs when a column is shown or hidden, with its id and whether it now shows.
    pub fn on_visibility_change(
        mut self,
        handler: impl Fn(&SharedString, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_visibility_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for DataTableViewOptions {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let on_change = self.on_visibility_change;
        // <DropdownMenuContent align="end" className="w-36"><DropdownMenuGroup>
        // <DropdownMenuLabel>Toggle columns</DropdownMenuLabel><DropdownMenuSeparator />
        let content = self.columns.into_iter().fold(
            DropdownMenuContent::new()
                .w(space::S36)
                .label(DropdownMenuLabel::new("Toggle columns"))
                .separator(DropdownMenuSeparator::new()),
            |content, column| {
                let on_change = on_change.clone();
                let id = column.id;
                content.checkbox_item(
                    DropdownMenuCheckboxItem::new(column.label)
                        .checked(column.visible)
                        .on_checked_change(move |visible, window, cx| {
                            if let Some(on_change) = &on_change {
                                on_change(&id, visible, window, cx);
                            }
                        }),
                )
            },
        );
        // <Button data-slot="data-table-view-options" variant="outline" size="sm"
        // className="ml-auto"><Settings2Icon data-icon="inline-start" />View</Button>
        let trigger = Button::new(ElementId::NamedChild(
            std::sync::Arc::new(self.id.clone()),
            "trigger".into(),
        ))
        .variant(ButtonVariant::Outline)
        .size(ButtonSize::Sm)
        .icon(IconName::Settings2)
        .label("View")
        .ml_auto()
        .refine_style(&self.style);
        DropdownMenu::new(self.id)
            .trigger(trigger)
            .content(content)
            .align_end()
    }
}

impl_styled!(DataTableViewOptions);

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::ParentElement as _;
    use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext, div, point, px};

    use super::*;
    use crate::components::test_support::render;

    type Log = Rc<RefCell<Vec<(SharedString, bool)>>>;

    /// Room left of the trigger, where the end-aligned menu opens.
    const INSET: Pixels = px(200.);

    fn columns() -> [DataTableViewColumn; 3] {
        [
            DataTableViewColumn::new("email", "Email", true),
            DataTableViewColumn::new("amount", "Amount", true),
            DataTableViewColumn::new("status", "Status", false),
        ]
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    fn open(cx: &mut TestAppContext, log: Log) -> (&mut VisualTestContext, Bounds<Pixels>) {
        cx.update(crate::init);
        let (cx, page) = render(cx, move || {
            let log = log.clone();
            div().flex().pl(INSET).child(
                DataTableViewOptions::new("view")
                    .columns(columns())
                    .on_visibility_change(move |id, visible, _, _| {
                        log.borrow_mut().push((id.clone(), visible))
                    }),
            )
        });
        let trigger = Bounds::new(
            page.origin + point(INSET, px(0.)),
            gpui::size(page.size.width - INSET, page.size.height),
        );
        cx.simulate_click(trigger.center(), Modifiers::default());
        draw(cx);
        (cx, trigger)
    }

    mod view_options {
        use super::*;

        #[gpui::test]
        fn should_be_32px_tall(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || DataTableViewOptions::new("view"));
            assert_eq!(bounds.size.height, px(32.));
        }

        #[gpui::test]
        fn should_open_under_its_trigger_aligned_to_its_end(cx: &mut TestAppContext) {
            let (cx, trigger) = open(cx, Log::default());
            let menu = cx
                .debug_bounds("dropdown-menu-content")
                .expect("the menu opened");
            assert_eq!(menu.size.width, px(144.));
            assert_eq!(menu.right(), trigger.right());
        }

        #[gpui::test]
        fn should_toggle_a_column_and_stay_open(cx: &mut TestAppContext) {
            let log = Log::default();
            let (cx, _) = open(cx, log.clone());
            let menu = cx
                .debug_bounds("dropdown-menu-content")
                .expect("the menu opened");
            // Amount: under the 4px padding, the 28px label, the 9px separator and Email's 33px row.
            let amount = point(menu.center().x, menu.top() + px(4. + 28. + 9. + 33. + 16.));
            cx.simulate_click(amount, Modifiers::default());
            draw(cx);
            assert_eq!(*log.borrow(), [(SharedString::from("amount"), false)]);
            assert!(cx.debug_bounds("dropdown-menu-content").is_some());
        }
    }
}
