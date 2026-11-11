//! Accordion: core-react's `accordion.tsx` on `gpui-base`'s unstyled
//! [`gpui_base::AccordionTrigger`] and [`gpui_base::AccordionPanel`], which own the trigger's
//! button role and expanded state and the panel's region role and mounting.
//!
//! [`Accordion`] keeps which items are open, its own by default (Base UI's `defaultValue`) or the
//! caller's ([`Accordion::value`]), and hands each [`AccordionItem`] its open state, whether it is
//! the last item (the last has no rule under it) and what a press on its trigger does. Each item
//! is a typed builder on the root: GPUI has no context for an item to find its accordion through.
//!
//! Not mirrored:
//! - The panel's height animation and the chevron's swap animation: a panel shows or hides at
//!   once.
//! - The content's `[&_a]` and `[&_p]` rules: GPUI has no links or paragraphs to style.
//! - Arrow-key movement between triggers: Base UI 1.9 has none either; each trigger is its own
//!   Tab stop, as in core-react.

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, opacity, space};
use gpui::{
    AnyElement, App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
    transparent_black,
};
use gpui_base::StyledExt as _;

use crate::icons::{Icon, IconName};
use crate::theme::ActiveTheme as _;

type ValueChangeHandler = Rc<dyn Fn(&[SharedString], &mut Window, &mut App)>;
type ToggleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// An accordion: a stack of items, each a trigger that shows or hides its content.
///
/// ```no_run
/// use brrritscold_core_gpui::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
/// use gpui::ParentElement as _;
///
/// let faq = Accordion::new("faq")
///     .item(
///         AccordionItem::new("shipping")
///             .trigger(AccordionTrigger::new("What are your shipping options?"))
///             .content(AccordionContent::new().child("Standard shipping takes three to five days.")),
///     )
///     .item(
///         AccordionItem::new("returns")
///             .trigger(AccordionTrigger::new("What is your return policy?"))
///             .content(AccordionContent::new().child("Return any unused item within 30 days.")),
///     )
///     .on_value_change(|open, _, _| println!("{open:?}"));
/// ```
#[derive(IntoElement)]
pub struct Accordion {
    id: ElementId,
    items: Vec<AccordionItem>,
    multiple: bool,
    value: Option<Vec<SharedString>>,
    default_value: Vec<SharedString>,
    on_value_change: Option<ValueChangeHandler>,
    style: StyleRefinement,
}

impl Accordion {
    /// An accordion with every item closed. `id` keys which items are open.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            multiple: false,
            value: None,
            default_value: Vec::new(),
            on_value_change: None,
            style: StyleRefinement::default(),
        }
    }

    /// An `Item` part, after the ones already added.
    pub fn item(mut self, item: AccordionItem) -> Self {
        self.items.push(item);
        self
    }

    /// Lets several items stay open at once; by default opening one closes the others.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// The open items' values, owned by the caller; they then change only when the caller
    /// passes others.
    pub fn value(mut self, value: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.value = Some(value.into_iter().map(Into::into).collect());
        self
    }

    /// The items open at first, when the accordion keeps its own value.
    pub fn default_value(
        mut self,
        value: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_value = value.into_iter().map(Into::into).collect();
        self
    }

    /// Runs when an item opens or closes, with the values of every item now open.
    pub fn on_value_change(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}

/// The open items after `item` is opened or closed: opening one closes the others unless
/// `multiple`.
fn toggled(
    open: &[SharedString],
    item: &SharedString,
    opening: bool,
    multiple: bool,
) -> Vec<SharedString> {
    if !opening {
        return open.iter().filter(|v| *v != item).cloned().collect();
    }
    let mut next: Vec<SharedString> = if multiple { open.to_vec() } else { Vec::new() };
    if !next.contains(item) {
        next.push(item.clone());
    }
    next
}

impl RenderOnce for Accordion {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let default_value = self.default_value;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| default_value);
        if let Some(value) = self.value {
            state.update(cx, |open, _| *open = value);
        }
        let open = state.read(cx).clone();
        let count = self.items.len();
        let multiple = self.multiple;
        let on_value_change = self.on_value_change;
        let items = self.items.into_iter().enumerate().map(|(index, item)| {
            let value = item.value.clone();
            let on_toggle: ToggleHandler = Rc::new({
                let state = state.clone();
                let on_value_change = on_value_change.clone();
                move |opening, window, cx| {
                    let next = state.update(cx, |open, cx| {
                        *open = toggled(open, &value, opening, multiple);
                        cx.notify();
                        open.clone()
                    });
                    if let Some(on_value_change) = &on_value_change {
                        on_value_change(&next, window, cx);
                    }
                }
            });
            let is_open = open.contains(&item.value);
            item.open(is_open)
                .last(index + 1 == count)
                .on_toggle(on_toggle)
        });
        // flex w-full flex-col
        gpui_base::Accordion::new(self.id)
            .flex()
            .w_full()
            .flex_col()
            .children(items)
            .refine_style(&self.style)
    }
}

/// The accordion's `Item` part: its `Trigger`, then its `Content` while open, ruled off from
/// the next item.
#[derive(IntoElement)]
pub struct AccordionItem {
    value: SharedString,
    trigger: Option<AccordionTrigger>,
    content: Option<AccordionContent>,
    disabled: bool,
    open: bool,
    last: bool,
    on_toggle: Option<ToggleHandler>,
    style: StyleRefinement,
}

impl AccordionItem {
    /// A closed item; `value` names it among its accordion's open items.
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            trigger: None,
            content: None,
            disabled: false,
            open: false,
            last: true,
            on_toggle: None,
            style: StyleRefinement::default(),
        }
    }

    /// The `Trigger` part.
    pub fn trigger(mut self, trigger: AccordionTrigger) -> Self {
        self.trigger = Some(trigger);
        self
    }

    /// The `Content` part, shown while the item is open.
    pub fn content(mut self, content: AccordionContent) -> Self {
        self.content = Some(content);
        self
    }

    /// Dims the trigger to 50% and keeps the item from opening or closing.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    fn last(mut self, last: bool) -> Self {
        self.last = last;
        self
    }

    fn on_toggle(mut self, on_toggle: ToggleHandler) -> Self {
        self.on_toggle = Some(on_toggle);
        self
    }
}

impl RenderOnce for AccordionItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let border = cx.theme().semantic().border;
        let trigger = self.trigger.map(|trigger| {
            trigger
                .id(ElementId::Name(
                    format!("accordion-trigger-{}", self.value).into(),
                ))
                .open(self.open)
                .disabled(self.disabled)
                .when_some(self.on_toggle, |t, on_toggle| t.on_toggle(on_toggle))
        });
        let panel = gpui_base::AccordionPanel::new()
            .id(ElementId::Name(
                format!("accordion-content-{}", self.value).into(),
            ))
            .open(self.open)
            .children(self.content);
        // not-last:border-b
        gpui_base::AccordionItem::new()
            .when(!self.last, |d| d.border_b_1().border_color(border))
            .children(trigger)
            .child(panel)
            .refine_style(&self.style)
    }
}

/// The accordion's `Trigger` part: its label and a chevron, down while closed and up while
/// open.
#[derive(IntoElement)]
pub struct AccordionTrigger {
    id: ElementId,
    label: SharedString,
    open: bool,
    disabled: bool,
    on_toggle: Option<ToggleHandler>,
    style: StyleRefinement,
}

impl AccordionTrigger {
    /// A closed trigger reading `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            id: ElementId::Name(label.clone()),
            label,
            open: false,
            disabled: false,
            on_toggle: None,
            style: StyleRefinement::default(),
        }
    }

    fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn on_toggle(mut self, on_toggle: ToggleHandler) -> Self {
        self.on_toggle = Some(on_toggle);
        self
    }
}

impl RenderOnce for AccordionTrigger {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let s = theme.semantic();
        let focus_ring = theme.focus_ring();
        let focus = window.use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle());
        let focus = focus.read(cx).clone().tab_stop(!self.disabled);
        let enabled = !self.disabled;
        // ChevronDownIcon / ChevronUpIcon (group-aria-expanded): ml-auto size-4 shrink-0
        // text-muted-foreground
        let chevron = if self.open {
            IconName::ChevronUp
        } else {
            IconName::ChevronDown
        };
        let trigger = gpui_base::AccordionTrigger::new(self.id)
            .open(self.open)
            .disabled(self.disabled)
            .track_focus(&focus)
            .when_some(self.on_toggle.filter(|_| enabled), |t, on_toggle| {
                t.on_change(move |open, _, window, cx| on_toggle(open, window, cx))
            })
            // relative flex flex-1 items-start justify-between rounded-md border
            // border-transparent py-4 text-left text-sm font-medium hover:underline
            .relative()
            .flex()
            .flex_1()
            .items_start()
            .justify_between()
            .rounded(s.radius_md)
            .border_1()
            .border_color(transparent_black())
            .py(space::S4)
            .text_size(font_size::SM)
            .font_weight(font_weight::MEDIUM)
            .when(enabled, |t| t.hover(|s| s.underline()))
            // focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50
            .focus_visible(move |style| {
                style
                    .border_color(s.ring)
                    .shadow(vec![super::ring(3., focus_ring)])
            })
            // disabled:opacity-50
            .when(self.disabled, |t| t.opacity(opacity::O50))
            .child(self.label)
            .child(
                Icon::new(chevron)
                    .size(space::S4)
                    .flex_shrink_0()
                    .ml_auto()
                    .text_color(s.muted_foreground),
            )
            .refine_style(&self.style);
        // AccordionPrimitive.Header: flex
        gpui_base::AccordionHeader::new(trigger).flex()
    }
}

/// The accordion's `Content` part: an open item's body (its Penpot `Body` slot).
#[derive(IntoElement)]
pub struct AccordionContent {
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for AccordionContent {
    fn default() -> Self {
        Self::new()
    }
}

impl AccordionContent {
    /// An empty body.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for AccordionContent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // The panel: overflow-hidden text-sm; inside it pt-0 pb-4
        div()
            .debug_selector(|| "accordion-content".into())
            .overflow_hidden()
            .text_size(font_size::SM)
            .pb(space::S4)
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl_styled!(Accordion, AccordionItem, AccordionTrigger, AccordionContent);
impl_parent!(AccordionContent);

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Modifiers, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use crate::components::test_support::{press, render};

    type Log = Rc<RefCell<Vec<Vec<SharedString>>>>;

    const QUESTIONS: [(&str, &str); 3] = [
        ("shipping", "What are your shipping options?"),
        ("returns", "What is your return policy?"),
        ("support", "How do I reach support?"),
    ];

    /// The story's FAQ, 448px wide, logging every change; `support` disabled.
    fn faq(log: Log, multiple: bool) -> impl Fn() -> Accordion {
        move || {
            let log = log.clone();
            QUESTIONS
                .into_iter()
                .fold(
                    Accordion::new("faq").multiple(multiple),
                    |accordion, (value, question)| {
                        accordion.item(
                            AccordionItem::new(value)
                                .disabled(value == "support")
                                .trigger(AccordionTrigger::new(question))
                                .content(AccordionContent::new().child("Answer.")),
                        )
                    },
                )
                .w(px(448.))
                .on_value_change(move |open, _, _| log.borrow_mut().push(open.to_vec()))
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    /// A point on the trigger of a closed item `n`, every item above it closed: triggers are
    /// 53px tall (16px padding each side of a 21px line).
    fn trigger(n: usize) -> gpui::Point<gpui::Pixels> {
        point(px(100.), px(26.) + px(54.) * n as f32)
    }

    fn open_panels(cx: &mut VisualTestContext) -> usize {
        cx.debug_bounds("accordion-content").map_or(0, |_| 1)
    }

    mod toggled {
        use super::*;

        fn values(values: &[&str]) -> Vec<SharedString> {
            values.iter().map(|v| SharedString::from(*v)).collect()
        }

        #[test]
        fn should_close_the_others_when_one_opens() {
            let next = toggled(&values(&["a"]), &"b".into(), true, false);
            assert_eq!(next, values(&["b"]));
        }

        #[test]
        fn should_keep_the_others_open_when_multiple() {
            let next = toggled(&values(&["a"]), &"b".into(), true, true);
            assert_eq!(next, values(&["a", "b"]));
        }

        #[test]
        fn should_close_only_the_item_closed() {
            let next = toggled(&values(&["a", "b"]), &"a".into(), false, true);
            assert_eq!(next, values(&["b"]));
        }
    }

    mod accordion {
        use super::*;

        #[gpui::test]
        fn should_expand_an_item_and_collapse_it_again(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, _) = render(cx, faq(log.clone(), false));
            assert_eq!(open_panels(cx), 0);
            cx.simulate_click(trigger(0), Modifiers::default());
            draw(cx);
            assert_eq!(open_panels(cx), 1);
            cx.simulate_click(trigger(0), Modifiers::default());
            draw(cx);
            assert_eq!(open_panels(cx), 0);
            assert_eq!(
                *log.borrow(),
                [vec![SharedString::from("shipping")], vec![]]
            );
        }

        #[gpui::test]
        fn should_close_the_open_item_when_another_opens(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, _) = render(cx, faq(log.clone(), false));
            cx.simulate_click(trigger(1), Modifiers::default());
            draw(cx);
            let returns = cx
                .debug_bounds("accordion-content")
                .expect("returns opened");
            // Shipping's trigger is still above, closed.
            cx.simulate_click(trigger(0), Modifiers::default());
            draw(cx);
            assert_eq!(
                log.borrow().last().unwrap(),
                &[SharedString::from("shipping")]
            );
            let shipping = cx
                .debug_bounds("accordion-content")
                .expect("shipping opened");
            assert!(
                shipping.top() < returns.top(),
                "only shipping's panel shows"
            );
        }

        #[gpui::test]
        fn should_keep_every_opened_item_open_when_multiple(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, _) = render(cx, faq(log.clone(), true));
            cx.simulate_click(trigger(1), Modifiers::default());
            draw(cx);
            cx.simulate_click(trigger(0), Modifiers::default());
            draw(cx);
            assert_eq!(
                log.borrow().last().unwrap(),
                &[
                    SharedString::from("returns"),
                    SharedString::from("shipping")
                ]
            );
        }

        #[gpui::test]
        fn should_ignore_a_disabled_item(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, _) = render(cx, faq(log.clone(), false));
            cx.simulate_click(trigger(2), Modifiers::default());
            draw(cx);
            assert_eq!(open_panels(cx), 0);
            assert!(log.borrow().is_empty());
        }

        #[gpui::test]
        fn should_open_from_the_keyboard(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, _) = render(cx, faq(log.clone(), false));
            cx.update(|window, cx| window.focus_next(cx));
            draw(cx);
            press(cx, "enter");
            draw(cx);
            assert_eq!(*log.borrow(), [vec![SharedString::from("shipping")]]);
        }
    }
}
