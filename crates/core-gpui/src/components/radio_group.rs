//! RadioGroup: core-react's `radio-group.tsx` on `gpui-base`'s unstyled
//! [`gpui_base::RadioGroup`] and [`gpui_base::Radio`], which own the group's role, focus,
//! Space and click activation, disabled inertness and accessibility.
//!
//! Items are controlled: each shows [`RadioGroupItem::checked`] and asks to be selected
//! through [`RadioGroupItem::on_select`]; the app keeps the group's value. A group holds open
//! content (an item usually sits in a [`crate::Label`] beside its text), so items are its
//! children rather than typed parts.
//!
//! Not mirrored:
//!
//! - Arrow keys between items: `gpui-base`'s group has no roving focus, so Tab visits each.
//! - A click on the label text selecting its item (a `<label>` does that in core-react).
//! - `after:-inset-x-3 after:-inset-y-2`, the enlarged hit area.
//! - The `aria-invalid` arms (not a Penpot variant property of the item) and the
//!   `group-has-…/field-label` arms (CSS relations GPUI has no equivalent of).

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::{border_radius, space};
use gpui::{
    AnyElement, App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
    transparent_black,
};
use gpui_base::StyledExt as _;

use super::control::{Outline, dimmed, on_background, shadows};
use crate::theme::{ActiveTheme as _, Theme};

/// A radio group, styled like core-react's `RadioGroup`: its children stacked 12px apart.
///
/// ```no_run
/// use brrritscold_core_gpui::{Label, RadioGroup, RadioGroupItem};
/// use gpui::ParentElement as _;
///
/// let plan = "Starter";
/// let group = RadioGroup::new("plan").children(["Starter", "Pro"].map(|name| {
///     Label::new()
///         .child(RadioGroupItem::new(name).checked(name == plan))
///         .child(name)
/// }));
/// ```
#[derive(IntoElement)]
pub struct RadioGroup {
    id: ElementId,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl RadioGroup {
    /// An empty group. `id` names it to assistive technology.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // grid w-full gap-3
        gpui_base::RadioGroup::new(self.id)
            .flex()
            .flex_col()
            .w_full()
            .gap(space::S3)
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl_styled!(RadioGroup);
impl_parent!(RadioGroup);

type SelectHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// The group's `Item` part: one radio, styled like core-react's `RadioGroupItem`.
#[derive(IntoElement)]
pub struct RadioGroupItem {
    id: ElementId,
    checked: bool,
    disabled: bool,
    accessibility_label: Option<SharedString>,
    on_select: Option<SelectHandler>,
    style: StyleRefinement,
}

impl RadioGroupItem {
    /// An unchecked item. `id` keys its focus state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            disabled: false,
            accessibility_label: None,
            on_select: None,
            style: StyleRefinement::default(),
        }
    }

    /// Whether this is the group's selected item: a `primary` circle with a dot.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Penpot property `disabled`: dims the item to 50% and makes it inert.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The name assistive technology reads, when no label text names the item.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Runs on a click or Space on an unchecked item: it asks to become the group's value.
    /// Never while disabled or already checked.
    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

/// The item's colours for one state, resolved for the active mode.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ItemStyle {
    fill: Hsla,
    border: Hsla,
    dot: Option<Hsla>,
    outline: Outline,
}

impl ItemStyle {
    fn new(theme: &Theme, checked: bool) -> Self {
        let s = theme.semantic();
        let outline = Outline::new(theme, false);
        if checked {
            // data-checked:border-primary data-checked:bg-primary dark:data-checked:bg-primary;
            // the indicator's dot bg-primary-foreground
            Self {
                fill: s.primary,
                border: s.primary,
                dot: Some(s.primary_foreground),
                outline,
            }
        } else {
            Self {
                // dark:bg-input/30, made opaque for the focus ring under it
                fill: on_background(
                    theme,
                    theme.mode().pick(transparent_black(), s.input.opacity(0.3)),
                ),
                border: outline.border,
                dot: None,
                outline,
            }
        }
    }
}

impl RenderOnce for RadioGroupItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let mut item = ItemStyle::new(theme, self.checked);
        if self.disabled {
            // data-disabled:opacity-50, layer by layer: the border and dot over the fill.
            item = ItemStyle {
                fill: dimmed(theme, item.fill),
                border: dimmed(theme, item.fill.blend(item.border)),
                dot: item.dot.map(|dot| dimmed(theme, dot)),
                ..item
            };
        }
        let checked = self.checked;
        // focus-visible:border-ring, except that data-checked:border-primary comes later.
        let focus = item.outline.focus.map(|(border, ring)| {
            let border = if checked { item.border } else { border };
            (border, shadows(Some(ring), false))
        });

        // relative flex aspect-square size-4 shrink-0 rounded-full border border-input
        // data-disabled:opacity-50
        gpui_base::Radio::new(self.id)
            .checked(checked)
            .disabled(self.disabled)
            .relative()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(space::S4)
            .rounded(border_radius::FULL)
            .border_1()
            .border_color(item.border)
            .bg(item.fill)
            .when_some(focus, |r, (border, shadow)| {
                r.focus_visible(move |s| s.border_color(border).shadow(shadow))
            })
            .when_some(self.accessibility_label, |r, label| {
                r.accessibility_label(label)
            })
            .when_some(self.on_select, |r, on_select| {
                r.on_change(move |_, _, window, cx| on_select(window, cx))
            })
            // The indicator: a centred size-2 dot.
            .when_some(item.dot, |r, dot| {
                r.child(div().size(space::S2).rounded(border_radius::FULL).bg(dot))
            })
            .refine_style(&self.style)
    }
}

impl_styled!(RadioGroupItem);

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{Modifiers, TestAppContext, px, size};

    use super::*;
    use crate::components::label::Label;
    use crate::components::test_support::{render, theme};
    use crate::theme::Mode;

    mod item {
        use super::*;

        #[gpui::test]
        fn should_be_a_16px_circle_with_a_primary_dot_when_checked(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Light);
            let item = ItemStyle::new(&theme, true);
            assert_eq!(item.fill, theme.semantic().primary);
            assert_eq!(item.dot, Some(theme.semantic().primary_foreground));
            let (_, bounds) = render(cx, || RadioGroupItem::new("r").checked(true));
            assert_eq!(bounds.size, size(px(16.), px(16.)));
        }
    }

    mod radio_group {
        use super::*;

        const PLANS: [&str; 3] = ["Starter", "Pro", "Enterprise"];

        /// The story's group: a selection the items' handlers write, as an app's state.
        fn plans(disabled: bool) -> (impl Fn() -> RadioGroup, Rc<Cell<usize>>) {
            let value = Rc::new(Cell::new(0));
            let make = {
                let value = value.clone();
                move || {
                    RadioGroup::new("plan").children(PLANS.iter().enumerate().map(|(i, plan)| {
                        let write = value.clone();
                        Label::new()
                            .child(
                                RadioGroupItem::new(*plan)
                                    .checked(value.get() == i)
                                    .disabled(disabled)
                                    .on_select(move |_, _| write.set(i)),
                            )
                            .child(*plan)
                    }))
                }
            };
            (make, value)
        }

        /// The centre of the `index`th item: 16px rows (the item is the label's tallest
        /// child), 12px apart.
        fn item_center(
            group: gpui::Bounds<gpui::Pixels>,
            index: usize,
        ) -> gpui::Point<gpui::Pixels> {
            let top = px(28.) * index as f32;
            gpui::point(group.origin.x + px(8.), group.origin.y + top + px(8.))
        }

        #[gpui::test]
        fn should_select_the_clicked_item(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (group, value) = plans(false);
            let (cx, bounds) = render(cx, group);
            cx.simulate_click(item_center(bounds, 1), Modifiers::default());
            assert_eq!(value.get(), 1);
        }

        #[gpui::test]
        fn should_keep_the_selection_while_disabled(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (group, value) = plans(true);
            let (cx, bounds) = render(cx, group);
            cx.simulate_click(item_center(bounds, 1), Modifiers::default());
            assert_eq!(value.get(), 0);
        }
    }
}
