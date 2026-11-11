//! Select: core-react's `select.tsx` on `gpui-base`'s unstyled [`gpui_base::Select`], which owns
//! the trigger's combobox role, opening from the keyboard and focus moving into the list and
//! back. The list (highlight, arrow keys, choosing) is drawn and driven here, anchored by
//! `gpui-base`'s [`gpui_base::Popup`].
//!
//! [`SelectTrigger`] and [`SelectContent`] are surfaces that render on their own; [`Select`]
//! wires them together and keeps the chosen value, its own by default.
//!
//! Not mirrored:
//! - `alignItemWithTrigger`: Base UI lays the list over the trigger, the chosen item on it; here
//!   the list opens below the trigger, 4px away, at least as wide as it.
//! - The trigger's transparent fill (`bg-transparent`, `dark:bg-input/30`) is drawn as it
//!   shows on `background`, opaque, as Input's is: GPUI paints `shadow-xs` and the invalid ring
//!   under the whole box, so they'd show through (see `control`). On another surface (a
//!   popover, a card in dark mode) the trigger keeps `background`'s colour.
//! - The scroll arrows and `max-h-(--available-height)`: the list grows to fit its items.
//! - The open and close animations.

use std::cell::Cell;
use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::{font_size, opacity, space};
use gpui::{
    Anchor, App, ElementId, FocusHandle, Hsla, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, Pixels, RenderOnce, SharedString, StatefulInteractiveElement as _,
    StyleRefinement, Styled, Window, canvas, div, prelude::FluentBuilder as _, px,
    transparent_black,
};
use gpui_base::StyledExt as _;
use gpui_base::actions::{Cancel, Confirm, SelectDown, SelectUp};
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use super::control::{Outline, dimmed, dimmed_shadows, on_background, shadows};
use crate::icons::{Icon, IconName};
use crate::theme::{ActiveTheme as _, Theme};

/// Penpot variant property `size` of `SelectTrigger`. Cases are its values in PascalCase;
/// [`Display`] gives the Penpot value back (`SelectTriggerSize::Sm` → `sm`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SelectTriggerSize {
    /// `default`: 36px tall.
    #[default]
    Default,
    /// `sm`: 32px tall.
    Sm,
}

type ChangeHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// A select: a [`SelectTrigger`] showing the chosen item, which opens [`SelectContent`] to
/// choose another.
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectTrigger,
/// };
///
/// let produce = Select::new("produce")
///     .trigger(SelectTrigger::new().placeholder("Pick one"))
///     .content(
///         SelectContent::new().group(
///             SelectGroup::new()
///                 .label(SelectLabel::new("Fruits"))
///                 .item(SelectItem::new("apple", "Apple"))
///                 .item(SelectItem::new("banana", "Banana")),
///         ),
///     )
///     .on_value_change(|value, _, _| println!("{value}"));
/// ```
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    trigger: SelectTrigger,
    content: SelectContent,
    value: Option<Option<SharedString>>,
    default_value: Option<SharedString>,
    on_value_change: Option<ChangeHandler>,
}

impl Select {
    /// A closed select with nothing chosen. `id` keys its state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: SelectTrigger::new(),
            content: SelectContent::new(),
            value: None,
            default_value: None,
            on_value_change: None,
        }
    }

    /// The `Trigger` part: the field showing the chosen item.
    pub fn trigger(mut self, trigger: SelectTrigger) -> Self {
        self.trigger = trigger;
        self
    }

    /// The `Content` part: the list to choose from.
    pub fn content(mut self, content: SelectContent) -> Self {
        self.content = content;
        self
    }

    /// The chosen item's value, owned by the caller (`None`: nothing chosen); it then changes
    /// only when the caller passes another.
    pub fn value(mut self, value: Option<impl Into<SharedString>>) -> Self {
        self.value = Some(value.map(Into::into));
        self
    }

    /// The item chosen at first, when the select keeps its own value.
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Runs when an item is chosen, with its value.
    pub fn on_value_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}

/// A select's state between frames.
struct SelectState {
    open: bool,
    value: Option<SharedString>,
    highlighted: Option<SharedString>,
    trigger_focus: FocusHandle,
    content_focus: FocusHandle,
    /// The trigger's width last frame: the list is at least as wide (`w-(--anchor-width)`).
    trigger_width: Rc<Cell<Pixels>>,
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let default_value = self.default_value;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| SelectState {
            open: false,
            value: default_value,
            highlighted: None,
            trigger_focus: cx.focus_handle(),
            content_focus: cx.focus_handle(),
            trigger_width: Rc::new(Cell::new(px(0.))),
        });
        if let Some(value) = self.value {
            state.update(cx, |state, _| state.value = value);
        }
        let (open, value, highlighted, trigger_focus, content_focus, trigger_width) = {
            let s = state.read(cx);
            (
                s.open,
                s.value.clone(),
                s.highlighted.clone(),
                s.trigger_focus.clone(),
                s.content_focus.clone(),
                s.trigger_width.clone(),
            )
        };
        let items = self.content.enabled_values();
        let label = value
            .as_ref()
            .and_then(|value| self.content.label_of(value));
        let disabled = self.trigger.disabled;

        let set_open = {
            let state = state.clone();
            let items = items.clone();
            Rc::new(move |open: bool, cx: &mut App| {
                state.update(cx, |s, cx| {
                    s.open = open;
                    // Opening highlights the chosen item, else the first.
                    s.highlighted = open
                        .then(|| s.value.clone().filter(|v| items.contains(v)))
                        .flatten()
                        .or_else(|| open.then(|| items.first().cloned()).flatten());
                    cx.notify();
                })
            })
        };
        let choose = {
            let state = state.clone();
            let on_change = self.on_value_change.clone();
            let trigger_focus = trigger_focus.clone();
            Rc::new(
                move |value: SharedString, window: &mut Window, cx: &mut App| {
                    state.update(cx, |s, cx| {
                        s.value = Some(value.clone());
                        s.open = false;
                        cx.notify();
                    });
                    trigger_focus.focus(window, cx);
                    if let Some(on_change) = &on_change {
                        on_change(&value, window, cx);
                    }
                },
            )
        };
        let move_highlight = {
            let state = state.clone();
            let items = items.clone();
            move |step: isize, cx: &mut App| {
                state.update(cx, |s, cx| {
                    let at = s
                        .highlighted
                        .as_ref()
                        .and_then(|h| items.iter().position(|v| v == h));
                    let next = match at {
                        None => 0,
                        Some(at) => {
                            (at as isize + step).clamp(0, items.len() as isize - 1) as usize
                        }
                    };
                    s.highlighted = items.get(next).cloned();
                    cx.notify();
                })
            }
        };

        let focus_visible = trigger_focus.is_focused(window) && window.last_input_was_keyboard();
        let trigger = gpui_base::Select::new("select-trigger")
            .open(open)
            .disabled(disabled)
            .focus_handle(&trigger_focus)
            .content_focus_handle(&content_focus)
            .when_some(label.clone(), |s, label| s.accessibility_value(label))
            .on_open_change({
                let set_open = set_open.clone();
                move |open, _, cx| set_open(open, cx)
            })
            .child(self.trigger.label(label).focus_visible(focus_visible))
            .child(
                // Notes the trigger's width for the list.
                canvas(
                    move |bounds, _, _| trigger_width.set(bounds.size.width),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full()
                .top_0()
                .left_0(),
            )
            .relative();

        let popup = gpui_base::Popup::new(self.id, trigger)
            .anchor(Anchor::TopLeft)
            .offset(SIDE_OFFSET)
            .when(!disabled, |p| {
                let set_open = set_open.clone();
                let trigger_focus = trigger_focus.clone();
                let content_focus = content_focus.clone();
                p.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    set_open(!open, cx);
                    let focus = if open { &trigger_focus } else { &content_focus };
                    focus.focus(window, cx);
                })
            });
        if !open {
            return popup;
        }
        let min_width = state.read(cx).trigger_width.get().max(space::S36);
        let context = Rc::new(SelectContext {
            selected: value,
            highlighted,
            choose: choose.clone(),
            highlight: Rc::new({
                let state = state.clone();
                move |value, cx| {
                    state.update(cx, |s, cx| {
                        if s.highlighted.as_ref() != Some(&value) {
                            s.highlighted = Some(value);
                            cx.notify();
                        }
                    })
                }
            }),
        });
        let close = {
            let set_open = set_open.clone();
            let trigger_focus = trigger_focus.clone();
            move |window: &mut Window, cx: &mut App| {
                set_open(false, cx);
                trigger_focus.focus(window, cx);
            }
        };
        let list = div()
            .id("select-list")
            .track_focus(&content_focus)
            .key_context("Select")
            .on_action({
                let move_highlight = move_highlight.clone();
                move |_: &SelectUp, _, cx| move_highlight(-1, cx)
            })
            .on_action(move |_: &SelectDown, _, cx| move_highlight(1, cx))
            .on_action({
                let state = state.clone();
                let choose = choose.clone();
                move |_: &Confirm, window, cx| {
                    if let Some(value) = state.read(cx).highlighted.clone() {
                        choose(value, window, cx);
                    }
                }
            })
            .on_action({
                let close = close.clone();
                move |_: &Cancel, window, cx| close(window, cx)
            })
            .on_mouse_down_out(move |_, window, cx| {
                // A press on the trigger toggles it closed by itself.
                set_open(false, cx);
                window.refresh();
            })
            .child(self.content.context(context).min_w(min_width));
        popup.content(list)
    }
}

/// `sideOffset={4}`: the gap between the trigger and the list.
const SIDE_OFFSET: Pixels = space::S1;

type Choose = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type Highlight = Rc<dyn Fn(SharedString, &mut App)>;

/// What a list's items need from the select they're in.
struct SelectContext {
    selected: Option<SharedString>,
    highlighted: Option<SharedString>,
    choose: Choose,
    highlight: Highlight,
}

/// The select's `Trigger` part: the field showing the chosen item, or a placeholder, and a
/// chevron.
#[derive(IntoElement)]
pub struct SelectTrigger {
    size: SelectTriggerSize,
    aria_invalid: bool,
    disabled: bool,
    placeholder: Option<SharedString>,
    label: Option<SharedString>,
    focus_visible: bool,
    style: StyleRefinement,
}

impl Default for SelectTrigger {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectTrigger {
    /// A `default` trigger with no placeholder.
    pub fn new() -> Self {
        Self {
            size: SelectTriggerSize::default(),
            aria_invalid: false,
            disabled: false,
            placeholder: None,
            label: None,
            focus_visible: false,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `size`.
    pub fn size(mut self, size: SelectTriggerSize) -> Self {
        self.size = size;
        self
    }

    /// Penpot property `aria-invalid`: draws the trigger destructive.
    pub fn aria_invalid(mut self, invalid: bool) -> Self {
        self.aria_invalid = invalid;
        self
    }

    /// Penpot property `disabled`: dims the trigger and keeps the list closed.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The text shown, muted, while nothing is chosen (`SelectValue`'s `placeholder`).
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    fn label(mut self, label: Option<SharedString>) -> Self {
        self.label = label;
        self
    }

    fn focus_visible(mut self, focus_visible: bool) -> Self {
        self.focus_visible = focus_visible;
        self
    }
}

/// The trigger's resolved colours.
#[derive(Debug, Clone, PartialEq)]
struct TriggerStyle {
    fill: Hsla,
    hover_fill: Option<Hsla>,
    border: Hsla,
    shadows: Vec<gpui::BoxShadow>,
}

impl SelectTrigger {
    fn colors(&self, theme: &Theme) -> TriggerStyle {
        let s = theme.semantic();
        let m = theme.mode();
        // border-input, the focus-visible and aria-invalid arms: Input's outline.
        let outline = Outline::new(theme, self.aria_invalid);
        let (mut border, ring) = match outline.focus.filter(|_| self.focus_visible) {
            Some((border, ring)) => (border, Some(ring)),
            None => (outline.border, outline.ring),
        };
        // bg-transparent dark:bg-input/30 dark:hover:bg-input/50, made opaque: it sits on
        // shadow-xs and the ring
        let mut fill = on_background(theme, m.pick(transparent_black(), s.input.opacity(0.3)));
        let hover_fill = m.pick(None, Some(on_background(theme, s.input.opacity(0.5))));
        let mut shadows = shadows(ring, true);
        if self.disabled {
            // disabled:opacity-50, layer by layer
            border = dimmed(theme, fill.blend(border));
            fill = dimmed(theme, fill);
            shadows = dimmed_shadows(shadows);
        }
        TriggerStyle {
            fill,
            hover_fill: hover_fill.filter(|_| !self.disabled),
            border,
            shadows,
        }
    }
}

impl RenderOnce for SelectTrigger {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let s = theme.semantic();
        let c = self.colors(theme);
        let placeholder = self.label.is_none();
        let text = self.label.or(self.placeholder);
        // flex w-fit items-center justify-between gap-1.5 rounded-md border py-2 pr-2 pl-2.5
        // text-sm whitespace-nowrap data-[size=default]:h-9 data-[size=sm]:h-8
        div()
            .id("select-trigger-surface")
            .flex()
            .items_center()
            .justify_between()
            .gap(space::S1_5)
            .h(match self.size {
                SelectTriggerSize::Default => space::S9,
                SelectTriggerSize::Sm => space::S8,
            })
            .rounded(s.radius_md)
            .border_1()
            .border_color(c.border)
            .bg(c.fill)
            .when_some(c.hover_fill, |d, fill| d.hover(move |style| style.bg(fill)))
            .shadow(c.shadows)
            .py(space::S2)
            .pr(space::S2)
            .pl(space::S2_5)
            .text_size(font_size::SM)
            .whitespace_nowrap()
            // *:data-[slot=select-value]:line-clamp-1; data-placeholder:text-muted-foreground
            .child(
                div()
                    .flex_1()
                    .line_clamp(1)
                    .when(placeholder, |d| d.text_color(s.muted_foreground))
                    .when(self.disabled, |d| d.opacity(opacity::O50))
                    .children(text),
            )
            // ChevronDownIcon: pointer-events-none size-4 text-muted-foreground
            .child(
                Icon::new(IconName::ChevronDown)
                    .size(space::S4)
                    .text_color(s.muted_foreground)
                    .when(self.disabled, |icon| icon.opacity(opacity::O50)),
            )
            .refine_style(&self.style)
    }
}

enum SelectChild {
    Group(SelectGroup),
    Item(SelectItem),
    Separator(SelectSeparator),
}

/// The select's `Content` part: the list of its groups, items and separators.
#[derive(IntoElement)]
pub struct SelectContent {
    children: Vec<SelectChild>,
    context: Option<Rc<SelectContext>>,
    style: StyleRefinement,
}

impl Default for SelectContent {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectContent {
    /// An empty list.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            context: None,
            style: StyleRefinement::default(),
        }
    }

    /// A group of items under a label (code-only, as core-react's `SelectGroup`).
    pub fn group(mut self, group: SelectGroup) -> Self {
        self.children.push(SelectChild::Group(group));
        self
    }

    /// An item outside any group.
    pub fn item(mut self, item: SelectItem) -> Self {
        self.children.push(SelectChild::Item(item));
        self
    }

    /// The `Separator` part, between groups.
    pub fn separator(mut self, separator: SelectSeparator) -> Self {
        self.children.push(SelectChild::Separator(separator));
        self
    }

    fn items(&self) -> impl Iterator<Item = &SelectItem> {
        self.children.iter().flat_map(|child| match child {
            SelectChild::Group(group) => group.items.iter().collect::<Vec<_>>(),
            SelectChild::Item(item) => vec![item],
            SelectChild::Separator(_) => Vec::new(),
        })
    }

    /// The values that can be chosen, in order.
    fn enabled_values(&self) -> Rc<Vec<SharedString>> {
        Rc::new(
            self.items()
                .filter(|item| !item.disabled)
                .map(|item| item.value.clone())
                .collect(),
        )
    }

    fn label_of(&self, value: &SharedString) -> Option<SharedString> {
        self.items()
            .find(|item| &item.value == value)
            .map(|item| item.label.clone())
    }

    fn context(mut self, context: Rc<SelectContext>) -> Self {
        self.context = Some(context);
        self
    }
}

impl RenderOnce for SelectContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let context = self.context;
        let mut shadows = vec![super::ring(1., s.foreground.opacity(0.1))];
        shadows.extend_from_slice(brrritscold_core_gpui_tokens::primitive::shadow::MD);
        // relative isolate min-w-36 overflow-x-hidden overflow-y-auto rounded-md bg-popover
        // text-popover-foreground shadow-md ring-1 ring-foreground/10
        div()
            .relative()
            .flex()
            .flex_col()
            .min_w(space::S36)
            .rounded(s.radius_md)
            .bg(s.popover)
            .text_color(s.popover_foreground)
            .shadow(shadows)
            .children(self.children.into_iter().map(|child| match child {
                SelectChild::Group(group) => group.context(context.clone()).into_any_element(),
                SelectChild::Item(item) => item.context(context.clone()).into_any_element(),
                SelectChild::Separator(separator) => separator.into_any_element(),
            }))
            .refine_style(&self.style)
    }
}

/// A group of items under an optional label: core-react's `SelectGroup`. A code-only part.
#[derive(IntoElement)]
pub struct SelectGroup {
    label: Option<SelectLabel>,
    items: Vec<SelectItem>,
    context: Option<Rc<SelectContext>>,
}

impl Default for SelectGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectGroup {
    /// An empty group.
    pub fn new() -> Self {
        Self {
            label: None,
            items: Vec::new(),
            context: None,
        }
    }

    /// The `Label` part, over the items.
    pub fn label(mut self, label: SelectLabel) -> Self {
        self.label = Some(label);
        self
    }

    /// An `Item` part.
    pub fn item(mut self, item: SelectItem) -> Self {
        self.items.push(item);
        self
    }

    fn context(mut self, context: Option<Rc<SelectContext>>) -> Self {
        self.context = context;
        self
    }
}

impl RenderOnce for SelectGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let context = self.context;
        // scroll-my-1 p-1
        div()
            .flex()
            .flex_col()
            .p(space::S1)
            .children(self.label)
            .children(
                self.items
                    .into_iter()
                    .map(|item| item.context(context.clone())),
            )
    }
}

/// The select's `Label` part: a group's heading.
#[derive(IntoElement)]
pub struct SelectLabel {
    text: SharedString,
    style: StyleRefinement,
}

impl SelectLabel {
    /// A label reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for SelectLabel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // px-2 py-1.5 text-xs text-muted-foreground
        div()
            .px(space::S2)
            .py(space::S1_5)
            .text_size(font_size::XS)
            .text_color(cx.theme().semantic().muted_foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The select's `Item` part: a choice, with a check (its `Indicator` part) once chosen.
#[derive(IntoElement)]
pub struct SelectItem {
    value: SharedString,
    label: SharedString,
    disabled: bool,
    selected: bool,
    context: Option<Rc<SelectContext>>,
    style: StyleRefinement,
}

impl SelectItem {
    /// An item for `value`, reading `label`.
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            selected: false,
            context: None,
            style: StyleRefinement::default(),
        }
    }

    /// Dims the item and keeps it from being chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Shows the item as chosen outside a [`Select`], as its check does: for an item drawn on
    /// its own. Inside a select, the select's value decides.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn context(mut self, context: Option<Rc<SelectContext>>) -> Self {
        self.context = context;
        self
    }
}

impl RenderOnce for SelectItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let context = self.context;
        let value = self.value;
        let selected = context
            .as_ref()
            .map_or(self.selected, |c| c.selected.as_ref() == Some(&value));
        let highlighted = context
            .as_ref()
            .is_some_and(|c| c.highlighted.as_ref() == Some(&value));
        let enabled = !self.disabled;
        // relative flex w-full items-center gap-2 rounded-sm py-1.5 pr-8 pl-2 text-sm
        // focus:bg-accent focus:text-accent-foreground data-disabled:opacity-50
        div()
            .id(ElementId::Name(value.clone()))
            .relative()
            .flex()
            .w_full()
            .items_center()
            .gap(space::S2)
            .rounded(s.radius_sm)
            .py(space::S1_5)
            .pr(space::S8)
            .pl(space::S2)
            .text_size(font_size::SM)
            .whitespace_nowrap()
            .when(highlighted, |d| {
                d.bg(s.accent).text_color(s.accent_foreground)
            })
            .when(self.disabled, |d| d.opacity(opacity::O50))
            .when_some(context.filter(|_| enabled), |d, context| {
                let hover = context.clone();
                let hover_value = value.clone();
                let click_value = value.clone();
                d.on_mouse_move(move |_, _, cx| (hover.highlight)(hover_value.clone(), cx))
                    .on_click(move |_, window, cx| {
                        (context.choose)(click_value.clone(), window, cx)
                    })
            })
            .child(div().flex_1().child(self.label))
            // ItemIndicator: absolute right-2 flex size-4 items-center justify-center, a check
            .when(selected, |d| {
                d.child(
                    div()
                        .absolute()
                        .right(space::S2)
                        .flex()
                        .size(space::S4)
                        .items_center()
                        .justify_center()
                        .child(Icon::new(IconName::Check).size(space::S4)),
                )
            })
            .refine_style(&self.style)
    }
}

/// The select's `Separator` part: a rule between groups.
#[derive(IntoElement)]
pub struct SelectSeparator {
    style: StyleRefinement,
}

impl Default for SelectSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectSeparator {
    /// A separator.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for SelectSeparator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // pointer-events-none -mx-1 my-1 h-px bg-border; the list has no padding of its own, so
        // the rule runs edge to edge.
        div()
            .my(space::S1)
            .h(px(1.))
            .bg(cx.theme().semantic().border)
            .refine_style(&self.style)
    }
}

impl_styled!(
    SelectTrigger,
    SelectContent,
    SelectLabel,
    SelectItem,
    SelectSeparator
);

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Bounds, Modifiers, TestAppContext, VisualTestContext, point};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::render;

    /// Room left of and above the trigger.
    const INSET: Pixels = px(100.);

    type Chosen = Rc<RefCell<Vec<SharedString>>>;

    /// The story's produce select, inset from the corner, recording what is chosen.
    fn produce(chosen: Chosen) -> impl Fn() -> gpui::Div {
        move || {
            let chosen = chosen.clone();
            div().flex().pl(INSET).pt(INSET).child(
                Select::new("produce")
                    .trigger(SelectTrigger::new().placeholder("Pick one"))
                    .content(
                        SelectContent::new()
                            .group(
                                SelectGroup::new()
                                    .label(SelectLabel::new("Fruits"))
                                    .item(SelectItem::new("apple", "Apple"))
                                    .item(SelectItem::new("banana", "Banana"))
                                    .item(SelectItem::new("grapes", "Grapes").disabled(true)),
                            )
                            .separator(SelectSeparator::new())
                            .group(SelectGroup::new().item(SelectItem::new("leek", "Leek"))),
                    )
                    .on_value_change(move |value, _, _| chosen.borrow_mut().push(value.clone())),
            )
        }
    }

    fn open(cx: &mut TestAppContext) -> (&mut VisualTestContext, Bounds<Pixels>, Chosen) {
        cx.update(crate::init);
        let chosen = Chosen::default();
        let (cx, page) = render(cx, produce(chosen.clone()));
        let trigger = Bounds::new(
            page.origin + point(INSET, INSET),
            gpui::size(page.size.width - INSET, page.size.height - INSET),
        );
        cx.simulate_click(trigger.center(), Modifiers::default());
        draw(cx);
        (cx, trigger, chosen)
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
    }

    /// Row `n` of the open list: 4px under the trigger, the group's 4px padding, the 30px
    /// label, then 33px items (14px text at 1.5, 6px above and below).
    fn row(trigger: Bounds<Pixels>, n: usize) -> gpui::Point<Pixels> {
        let top = trigger.bottom() + px(4. + 4. + 30.) + px(33.) * n as f32;
        point(trigger.left() + px(20.), top + px(16.))
    }

    mod size {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&str> = SelectTriggerSize::iter().map(Into::into).collect();
            assert_eq!(values, ["default", "sm"]);
        }
    }

    mod trigger {
        use super::*;

        #[gpui::test]
        fn should_be_36px_tall_by_default_and_32px_at_sm(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, default) = render(cx, || SelectTrigger::new().placeholder("Pick one"));
            assert_eq!(default.size.height, px(36.));
            let (_, sm) = render(cx, || {
                SelectTrigger::new()
                    .size(SelectTriggerSize::Sm)
                    .placeholder("Pick one")
            });
            assert_eq!(sm.size.height, px(32.));
        }

        #[gpui::test]
        fn should_draw_an_invalid_trigger_destructive_with_a_ring(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = crate::components::test_support::theme(cx, crate::Mode::Light);
            let colors = SelectTrigger::new().aria_invalid(true).colors(&theme);
            let destructive = theme.semantic().destructive;
            assert_eq!(colors.border, destructive);
            assert_eq!(colors.shadows[0].color, destructive.opacity(0.2));
        }
    }

    mod select {
        use super::*;

        #[gpui::test]
        fn should_choose_an_item_with_the_pointer_and_close(cx: &mut TestAppContext) {
            let (cx, trigger, chosen) = open(cx);
            cx.simulate_click(row(trigger, 1), Modifiers::default());
            draw(cx);
            assert_eq!(*chosen.borrow(), ["banana"]);
            // Closed: the same point no longer lands on an item.
            cx.simulate_click(row(trigger, 0), Modifiers::default());
            draw(cx);
            assert_eq!(*chosen.borrow(), ["banana"]);
        }

        #[gpui::test]
        fn should_choose_with_the_arrow_keys_skipping_disabled_items(cx: &mut TestAppContext) {
            let (cx, _, chosen) = open(cx);
            // Apple is highlighted on open; down past Banana skips the disabled Grapes.
            cx.simulate_keystrokes("down down enter");
            draw(cx);
            assert_eq!(*chosen.borrow(), ["leek"]);
        }

        #[gpui::test]
        fn should_close_on_escape_without_choosing(cx: &mut TestAppContext) {
            let (cx, trigger, chosen) = open(cx);
            cx.simulate_keystrokes("escape");
            draw(cx);
            cx.simulate_click(row(trigger, 1), Modifiers::default());
            draw(cx);
            assert!(chosen.borrow().is_empty());
        }

        #[gpui::test]
        fn should_ignore_a_disabled_item(cx: &mut TestAppContext) {
            let (cx, trigger, chosen) = open(cx);
            cx.simulate_click(row(trigger, 2), Modifiers::default());
            draw(cx);
            assert!(chosen.borrow().is_empty());
        }
    }
}
