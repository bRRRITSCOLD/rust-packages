//! Data Table Toolbar: core-react's `data-table-toolbar.tsx`, a data table's filter on core-gpui's
//! [`Input`] and [`Button`]. No table model: the filter text lives in an [`InputState`] the app
//! creates and keeps, as an [`Input`]'s does, and the app filters its rows by it.
//!
//! The filter's placeholder is the state's, as for any [`Input`]: core-react derives it from
//! `filterLabel` (`Filter payments…`); here the app sets it,
//! `InputState::new(window, cx).placeholder("Filter payments…")`.

use brrritscold_core_gpui_tokens::primitive::{size, space};
use gpui::{
    AnyElement, App, Entity, IntoElement, ParentElement as _, RenderOnce, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder as _,
};
use gpui_base::StyledExt as _;
use gpui_base::input::InputEvent;

use super::button::{Button, ButtonSize, ButtonVariant};
use super::input::{Input, InputState};
use crate::icons::IconName;

/// A data table's toolbar: the filter, a Reset button while the filter has text, and its
/// children at the end (`DataTableViewOptions`, an app's actions).
///
/// The app reads the filter with `filter.read(cx).value()` and hears it change by subscribing to
/// the state's `InputEvent::Change`, which Reset emits too:
///
/// ```no_run
/// use brrritscold_core_gpui::{DataTableToolbar, InputState};
/// use gpui::{AppContext as _, Context, Entity, IntoElement, Render, Window};
///
/// struct Payments {
///     filter: Entity<InputState>,
/// }
///
/// impl Payments {
///     fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
///         let filter = cx.new(|cx| InputState::new(window, cx).placeholder("Filter payments…"));
///         Self { filter }
///     }
/// }
///
/// impl Render for Payments {
///     fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
///         DataTableToolbar::new(&self.filter)
///     }
/// }
/// ```
#[derive(IntoElement)]
pub struct DataTableToolbar {
    filter: Entity<InputState>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl DataTableToolbar {
    /// A toolbar filtering by `filter`'s text.
    pub fn new(filter: &Entity<InputState>) -> Self {
        Self {
            filter: filter.clone(),
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for DataTableToolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let filtered = !self.filter.read(cx).value().is_empty();
        // {filter ? <Button variant="ghost" size="sm" onClick={() => onFilterChange('')}>Reset
        // <XIcon data-icon="inline-end" /></Button> : null}
        let reset = filtered.then(|| {
            let filter = self.filter.clone();
            Button::new(("data-table-toolbar-reset", self.filter.entity_id()))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .label("Reset")
                .icon_end(IconName::X)
                .on_click(move |_, window, cx| {
                    filter.update(cx, |filter, cx| {
                        filter.set_value("", window, cx);
                        cx.emit(InputEvent::Change);
                    })
                })
        });
        let actions = (!self.children.is_empty()).then(|| {
            // flex items-center gap-2
            div()
                .flex()
                .items_center()
                .gap(space::S2)
                .children(self.children)
        });
        // flex items-center justify-between gap-2, the filters flex flex-1 items-center gap-2;
        // the input h-8 w-full max-w-xs
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(space::S2)
            .child(
                div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .gap(space::S2)
                    .child(
                        Input::new(&self.filter)
                            .h(space::S8)
                            .w_full()
                            .max_w(size::XS),
                    )
                    .children(reset),
            )
            .when_some(actions, |d, actions| d.child(actions))
            .refine_style(&self.style)
    }
}

impl_styled!(DataTableToolbar);
impl_parent!(DataTableToolbar);

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{Modifiers, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use crate::components::test_support::{render, state};

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    /// A 608px toolbar on a fresh filter, and the changes the filter emits.
    fn toolbar(
        cx: &mut TestAppContext,
    ) -> (
        Entity<InputState>,
        Rc<RefCell<usize>>,
        &mut VisualTestContext,
    ) {
        cx.update(crate::init);
        let filter = state(cx, |window, cx| {
            InputState::new(window, cx).placeholder("Filter payments…")
        });
        let changes = Rc::new(RefCell::new(0));
        cx.update(|cx| {
            let changes = changes.clone();
            cx.subscribe(&filter, move |_, event: &InputEvent, _| {
                if matches!(event, InputEvent::Change) {
                    *changes.borrow_mut() += 1;
                }
            })
            .detach()
        });
        let shown = filter.clone();
        let (cx, _) = render(cx, move || DataTableToolbar::new(&shown).w(px(608.)));
        (filter, changes, cx)
    }

    mod toolbar {
        use super::*;

        #[gpui::test]
        fn should_take_typed_filter_text(cx: &mut TestAppContext) {
            let (filter, changes, cx) = toolbar(cx);
            cx.update(|window, cx| filter.update(cx, |filter, cx| filter.focus(window, cx)));
            cx.simulate_input("ken");
            draw(cx);
            assert_eq!(cx.read(|cx| filter.read(cx).value()), "ken");
            assert!(*changes.borrow() > 0);
        }

        #[gpui::test]
        fn should_reset_the_filter(cx: &mut TestAppContext) {
            let (filter, changes, cx) = toolbar(cx);
            cx.update(|window, cx| {
                filter.update(cx, |filter, cx| filter.set_value("ken", window, cx))
            });
            draw(cx);
            // Reset sits right of the 320px filter, 8px away.
            cx.simulate_click(point(px(320. + 8. + 20.), px(16.)), Modifiers::default());
            draw(cx);
            assert_eq!(cx.read(|cx| filter.read(cx).value()), "");
            assert_eq!(*changes.borrow(), 1, "Reset tells the app");
        }

        #[gpui::test]
        fn should_be_32px_tall(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let filter = state(cx, InputState::new);
            let (_, bounds) = render(cx, move || DataTableToolbar::new(&filter));
            assert_eq!(bounds.size.height, px(32.));
        }
    }
}
