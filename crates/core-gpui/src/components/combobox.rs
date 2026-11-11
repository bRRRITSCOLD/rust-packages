//! Combobox: core-react's `combobox.tsx` on `gpui-base`'s unstyled [`gpui_base::Combobox`],
//! which owns the combobox role and its open state's semantics. Filtering, the highlight, the
//! arrow keys and choosing are driven here, and the list is anchored by `gpui-base`'s
//! [`gpui_base::Popup`].
//!
//! The text being typed lives in an [`InputState`] the app creates and keeps, as for
//! [`Input`](crate::Input); its placeholder is the state's. The input keeps keyboard focus
//! while the list is open: Up and Down move the highlight, Enter chooses it, Escape closes the
//! list, and Backspace in an empty multiple-choice input removes the last chip.
//!
//! [`ComboboxInput`], [`ComboboxChips`], [`ComboboxChip`] and [`ComboboxContent`] with its parts
//! are surfaces that render on their own; [`Combobox`] wires them together and keeps the
//! chosen values, its own.
//!
//! Not mirrored:
//! - `showTrigger` on the input and `showRemove` on a chip: no Penpot variant property names
//!   them, so the trigger and the remove button always show.
//! - `text-base md:text-sm` on the input: it is `text-sm`, as Input's is (see
//!   [`Input`](crate::Input)); the published Storybook's 640px viewport shows 16px text.
//! - The input's and the chips' transparent fill is drawn as it shows on `background`, as
//!   Input's is (see `control`).
//! - The chips' `aria-invalid` arms, an input inside the list, groups of items given as data
//!   (`items` / `ComboboxCollection`), `autoHighlight` and wrapping from the last item to the
//!   first.
//! - Resetting the typed text to the chosen item's when the input loses focus.
//! - The list's `max-h` and scrolling, and the open and close animations.

use std::cell::Cell;
use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, opacity, shadow, space};
use gpui::{
    Anchor, App, Bounds, ElementId, Entity, Focusable as _, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, RenderOnce, SharedString, StatefulInteractiveElement as _,
    StyleRefinement, Styled, Subscription, Window, canvas, div, prelude::FluentBuilder as _, px,
};
use gpui_base::StyledExt as _;
use gpui_base::input::InputState;
use gpui_base::input::{Backspace, Enter, Escape, InputEvent, MoveDown, MoveUp};

use super::button::{Button, ButtonSize, ButtonVariant};
use super::control::{Outline, on_background, shadows};
use super::input_group::{InputGroup, InputGroupButton};
use crate::icons::{Icon, IconName};
use crate::theme::ActiveTheme as _;

type ChangeHandler = Rc<dyn Fn(&[SharedString], &mut Window, &mut App)>;
type ValueAction = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type SetOpen = Rc<dyn Fn(bool, &mut App)>;
type Highlight = Rc<dyn Fn(SharedString, &mut App)>;

/// What a combobox types into: one value's [`ComboboxInput`], or [`ComboboxChips`] for many.
enum Field {
    Input(ComboboxInput),
    Chips(ComboboxChips),
}

impl Field {
    fn state(&self) -> &Entity<InputState> {
        match self {
            Field::Input(input) => &input.state,
            Field::Chips(chips) => &chips.state,
        }
    }

    fn disabled(&self) -> bool {
        match self {
            Field::Input(input) => input.disabled,
            Field::Chips(_) => false,
        }
    }
}

/// A combobox: a field to type into, which filters a [`ComboboxContent`] list to choose from.
///
/// Its field decides how many values it keeps: a [`ComboboxInput`] keeps one and shows its
/// label as the text, [`ComboboxChips`] keep any number (core-react's `multiple`), one chip
/// each.
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Combobox, ComboboxContent, ComboboxEmpty, ComboboxInput, ComboboxItem, InputState,
/// };
/// use gpui::{AppContext as _, Context, Entity, IntoElement, Render, Window};
///
/// struct Project {
///     framework: Entity<InputState>,
/// }
///
/// impl Project {
///     fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
///         let framework = cx.new(|cx| InputState::new(window, cx).placeholder("Pick a framework"));
///         Self { framework }
///     }
/// }
///
/// impl Render for Project {
///     fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
///         Combobox::new("framework")
///             .input(ComboboxInput::new(&self.framework))
///             .content(
///                 ComboboxContent::new()
///                     .empty(ComboboxEmpty::new("No frameworks found."))
///                     .item(ComboboxItem::new("next", "Next.js"))
///                     .item(ComboboxItem::new("sveltekit", "SvelteKit")),
///             )
///             .on_value_change(|values, _, _| println!("{values:?}"))
///     }
/// }
/// ```
#[derive(IntoElement)]
pub struct Combobox {
    id: ElementId,
    field: Option<Field>,
    content: ComboboxContent,
    default_values: Vec<SharedString>,
    on_value_change: Option<ChangeHandler>,
    style: StyleRefinement,
}

impl Combobox {
    /// A closed combobox with nothing chosen. `id` keys its state. Give it a field with
    /// [`input`](Self::input) or [`chips`](Self::chips): one rendered without either panics.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            field: None,
            content: ComboboxContent::new(),
            default_values: Vec::new(),
            on_value_change: None,
            style: StyleRefinement::default(),
        }
    }

    /// The `Input` part: the field for a combobox that keeps one value.
    pub fn input(mut self, input: ComboboxInput) -> Self {
        self.field = Some(Field::Input(input));
        self
    }

    /// The `Chips` part: the field for a combobox that keeps many values, a chip each. The
    /// chosen values draw the chips; chips added to `chips` itself are left out.
    pub fn chips(mut self, chips: ComboboxChips) -> Self {
        self.field = Some(Field::Chips(chips));
        self
    }

    /// The `Content` part: the list to choose from.
    pub fn content(mut self, content: ComboboxContent) -> Self {
        self.content = content;
        self
    }

    /// The item chosen at first, for a combobox that keeps one value.
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_values = vec![value.into()];
        self
    }

    /// The items chosen at first, for a combobox with [`chips`](Self::chips).
    pub fn default_values(
        mut self,
        values: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_values = values.into_iter().map(Into::into).collect();
        self
    }

    /// Runs when the chosen values change, with all of them: one or none for a combobox that
    /// keeps one value.
    pub fn on_value_change(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}

/// A combobox's state between frames.
struct ComboboxState {
    open: bool,
    values: Vec<SharedString>,
    highlighted: Option<SharedString>,
    /// The field's bounds last frame: the list is as wide, and a press inside it isn't one
    /// outside the list.
    field: Rc<Cell<Bounds<Pixels>>>,
    _typing: Subscription,
}

/// `sideOffset={6}`: the gap between the field and the list.
const SIDE_OFFSET: Pixels = space::S1_5;

impl RenderOnce for Combobox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let field = self
            .field
            .unwrap_or_else(|| panic!("Combobox {:?} needs an input or chips field", self.id));
        let multiple = matches!(field, Field::Chips(_));
        let input = field.state().clone();
        let disabled = field.disabled();
        let default_values = self.default_values;
        let default_label = default_values
            .first()
            .filter(|_| !multiple)
            .and_then(|value| self.content.label_of(value));
        let state = window.use_keyed_state(self.id.clone(), cx, |window, cx| {
            if let Some(label) = default_label {
                input.update(cx, |input, cx| input.set_value(label, window, cx));
            }
            // Typing opens the list and clears the highlight; leaving the input closes it.
            let typing = cx.subscribe(&input, |state: &mut ComboboxState, _, event, cx| {
                match event {
                    InputEvent::Change => {
                        state.open = true;
                        state.highlighted = None;
                    }
                    InputEvent::Blur => state.open = false,
                    _ => return,
                }
                cx.notify();
            });
            ComboboxState {
                open: false,
                values: default_values,
                highlighted: None,
                field: Rc::new(Cell::new(Bounds::default())),
                _typing: typing,
            }
        });
        let (open, values, highlighted, field_bounds) = {
            let s = state.read(cx);
            (
                s.open && !disabled,
                s.values.clone(),
                s.highlighted.clone(),
                s.field.clone(),
            )
        };

        // Base UI's filter: the items whose label contains the typed text, ignoring case. The
        // chosen item's own label, shown as the text, filters nothing.
        let query = input.read(cx).value();
        let chosen_label = values
            .first()
            .filter(|_| !multiple)
            .and_then(|value| self.content.label_of(value));
        let filter = (!query.is_empty() && chosen_label.as_ref() != Some(&query))
            .then(|| query.to_lowercase());
        let content = self.content.filtered(filter.as_deref());
        let items = content.enabled_values();
        let highlighted = highlighted.filter(|h| items.contains(h));

        let change = {
            let state = state.clone();
            let on_change = self.on_value_change.clone();
            Rc::new(
                move |values: Vec<SharedString>, window: &mut Window, cx: &mut App| {
                    state.update(cx, |s, cx| {
                        s.values = values.clone();
                        cx.notify();
                    });
                    if let Some(on_change) = &on_change {
                        on_change(&values, window, cx);
                    }
                },
            )
        };
        let set_open: SetOpen = Rc::new({
            let state = state.clone();
            move |open: bool, cx: &mut App| {
                state.update(cx, |s, cx| {
                    if s.open != open {
                        s.open = open;
                        // Base UI highlights the chosen item on opening.
                        s.highlighted = open.then(|| s.values.first().cloned()).flatten();
                        cx.notify();
                    }
                })
            }
        });
        let choose: ValueAction = Rc::new({
            let input = input.clone();
            let change = change.clone();
            let set_open = set_open.clone();
            let values = values.clone();
            let label_of = content.labels();
            move |value: SharedString, window: &mut Window, cx: &mut App| {
                let next = if multiple {
                    // Choosing a chosen item again lets it go.
                    let mut next = values.clone();
                    match next.iter().position(|v| v == &value) {
                        Some(at) => {
                            next.remove(at);
                        }
                        None => next.push(value),
                    }
                    next
                } else {
                    vec![value]
                };
                // One value shows its label as the text; chips clear it for the next pick.
                let text = if multiple {
                    SharedString::default()
                } else {
                    label_of(&next[0]).unwrap_or_default()
                };
                input.update(cx, |input, cx| input.set_value(text, window, cx));
                set_open(false, cx);
                change(next, window, cx);
            }
        });
        let remove: ValueAction = Rc::new({
            let change = change.clone();
            let values = values.clone();
            move |value, window, cx| {
                let next = values.iter().filter(|v| **v != value).cloned().collect();
                change(next, window, cx);
            }
        });
        let move_highlight = {
            let state = state.clone();
            let items = items.clone();
            let values = values.clone();
            move |step: isize, cx: &mut App| {
                state.update(cx, |s, cx| {
                    let at = s
                        .highlighted
                        .as_ref()
                        .filter(|_| s.open)
                        .and_then(|h| items.iter().position(|v| v == h));
                    let last = items.len() as isize - 1;
                    let next = match at {
                        Some(at) => Some((at as isize + step).clamp(0, last) as usize),
                        // Opening from the keyboard highlights the chosen item, else an end.
                        None => values
                            .iter()
                            .find_map(|v| items.iter().position(|i| i == v))
                            .or_else(|| {
                                (last >= 0).then_some(if step > 0 { 0 } else { last as usize })
                            }),
                    };
                    s.open = true;
                    s.highlighted = next.and_then(|n| items.get(n).cloned());
                    cx.notify();
                })
            }
        };

        let focus = input.read(cx).focus_handle(cx);
        let field = match field {
            Field::Input(field) => {
                let has_value = !query.is_empty() || !values.is_empty();
                field
                    .has_value(has_value)
                    .on_press({
                        let set_open = set_open.clone();
                        move |_, cx| set_open(true, cx)
                    })
                    .on_toggle({
                        let set_open = set_open.clone();
                        let focus = focus.clone();
                        move |window, cx| {
                            set_open(!open, cx);
                            focus.focus(window, cx);
                        }
                    })
                    .on_clear({
                        let input = input.clone();
                        let change = change.clone();
                        let focus = focus.clone();
                        move |window, cx| {
                            input.update(cx, |input, cx| input.set_value("", window, cx));
                            change(Vec::new(), window, cx);
                            focus.focus(window, cx);
                        }
                    })
                    .into_any_element()
            }
            Field::Chips(chips) => {
                let chosen = values.iter().map(|value| {
                    let label = content.label_of(value).unwrap_or_else(|| value.clone());
                    let remove = remove.clone();
                    let value = value.clone();
                    ComboboxChip::new(label)
                        .on_remove(move |window, cx| remove(value.clone(), window, cx))
                });
                chips
                    .replace_chips(chosen)
                    .on_press({
                        let set_open = set_open.clone();
                        let focus = focus.clone();
                        move |window, cx| {
                            set_open(true, cx);
                            focus.focus(window, cx);
                        }
                    })
                    .into_any_element()
            }
        };

        let root = gpui_base::Combobox::new("combobox")
            .open(open)
            .disabled(disabled)
            .content_focus_handle(&focus)
            .on_open_change({
                let set_open = set_open.clone();
                move |open, _, cx| set_open(open, cx)
            })
            .w_full()
            .relative()
            .child(field)
            .child(
                // Notes the field's bounds for the list.
                canvas(
                    move |bounds, _, _| field_bounds.set(bounds),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full()
                .top_0()
                .left_0(),
            );
        let popup = gpui_base::Popup::new(self.id, root)
            .anchor(Anchor::TopLeft)
            .offset(SIDE_OFFSET)
            .w_full();
        let popup = if open {
            let field = state.read(cx).field.get();
            // w-(--anchor-width) min-w-[calc(var(--anchor-width)+var(--spacing-7))]: Base UI
            // anchors the list to the text box, which is the field less its border and the
            // 28px addon, so the list is the field less its border, inset by it. Under chips
            // (data-[chips=true]:min-w-(--anchor-width)) it is the chips' own width.
            let (inset, width) = if multiple {
                (px(0.), field.size.width)
            } else {
                (px(1.), field.size.width - px(2.))
            };
            let context = Rc::new(ComboboxContext {
                selected: values.clone(),
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
            let field_bounds = state.read(cx).field.clone();
            let close = set_open.clone();
            popup.content(
                div()
                    .id("combobox-list")
                    .on_mouse_down_out(move |event, window, cx| {
                        // A press in the field is the field's own (typing, the trigger).
                        if !field_bounds.get().contains(&event.position) {
                            close(false, cx);
                            window.refresh();
                        }
                    })
                    .ml(inset)
                    .child(content.context(context).w(width)),
            )
        } else {
            popup
        };

        div()
            .flex()
            .when(!disabled, |d| {
                let down = move_highlight.clone();
                let up = move_highlight;
                let state_enter = state.clone();
                let state_escape = state.clone();
                let set_open = set_open.clone();
                let remove = remove.clone();
                let empty = query.is_empty();
                let last = values.last().cloned();
                d.capture_action(move |_: &MoveDown, _, cx| {
                    down(1, cx);
                    cx.stop_propagation();
                })
                .capture_action(move |_: &MoveUp, _, cx| {
                    up(-1, cx);
                    cx.stop_propagation();
                })
                .capture_action(move |_: &Enter, window, cx| {
                    let s = state_enter.read(cx);
                    if let Some(value) = s.highlighted.clone().filter(|_| s.open) {
                        choose(value, window, cx);
                        cx.stop_propagation();
                    }
                })
                .capture_action(move |_: &Escape, _, cx| {
                    if state_escape.read(cx).open {
                        set_open(false, cx);
                        cx.stop_propagation();
                    }
                })
                .capture_action(move |_: &Backspace, window, cx| {
                    // An empty chips input gives back its last chip.
                    if let Some(last) = last.clone().filter(|_| multiple && empty) {
                        remove(last, window, cx);
                        cx.stop_propagation();
                    }
                })
            })
            .child(popup)
            .refine_style(&self.style)
    }
}

/// What a list's items need from the combobox they're in.
struct ComboboxContext {
    selected: Vec<SharedString>,
    highlighted: Option<SharedString>,
    choose: ValueAction,
    highlight: Highlight,
}

type Press = Box<dyn Fn(&mut Window, &mut App)>;

/// The combobox's `Input` part: the text box with a chevron, or a clear button once there is
/// a value (`showClear`).
#[derive(IntoElement)]
pub struct ComboboxInput {
    state: Entity<InputState>,
    disabled: bool,
    show_clear: bool,
    has_value: Option<bool>,
    on_press: Option<Press>,
    on_toggle: Option<Press>,
    on_clear: Option<Press>,
    style: StyleRefinement,
}

impl ComboboxInput {
    /// An input showing and editing `state`.
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            disabled: false,
            show_clear: false,
            has_value: None,
            on_press: None,
            on_toggle: None,
            on_clear: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `disabled`: fades the text and the chevron and keeps the list closed.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Penpot property `showClear`: a clear button, in the chevron's place, while there is a
    /// value.
    pub fn show_clear(mut self, show_clear: bool) -> Self {
        self.show_clear = show_clear;
        self
    }

    /// Whether the combobox has a value or typed text; on its own, the input's text decides.
    fn has_value(mut self, has_value: bool) -> Self {
        self.has_value = Some(has_value);
        self
    }

    fn on_press(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Box::new(handler));
        self
    }

    fn on_toggle(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    fn on_clear(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ComboboxInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let has_value = self
            .has_value
            .unwrap_or_else(|| !self.state.read(cx).value().is_empty());
        // ComboboxClear shows while there is a value, and the trigger hides beside it
        // (group-has-data-[slot=combobox-clear]/input-group:hidden).
        let button = if self.show_clear && has_value {
            // XIcon in an icon-xs InputGroupButton: size-4 (InputGroupButton's own icon-xs arm
            // sets no icon size, so Button's base size-4 shows)
            InputGroupButton::new("combobox-clear", IconName::X, space::S4, "Clear")
                .disabled(self.disabled)
                .when_some(self.on_clear, |b, clear| {
                    b.on_click(move |_, window, cx| clear(window, cx))
                })
        } else {
            // ComboboxTrigger: ChevronDownIcon pointer-events-none size-4 text-muted-foreground,
            // tabIndex={-1}
            InputGroupButton::new(
                "combobox-trigger",
                IconName::ChevronDown,
                space::S4,
                "Show options",
            )
            .focusable(false)
            .disabled(self.disabled)
            .when_some(self.on_toggle, |b, toggle| {
                b.on_click(move |_, window, cx| toggle(window, cx))
            })
        };
        // InputGroup w-auto
        InputGroup::new(&self.state)
            .disabled(self.disabled)
            .addon(button)
            .when_some(self.on_press, |g, press| g.on_input_press(press))
            .refine_style(&self.style)
    }
}

/// The combobox's `Chips` part: the field of a combobox that keeps many values, a
/// [`ComboboxChip`] each, then the text box (core-react's `ComboboxChipsInput`).
#[derive(IntoElement)]
pub struct ComboboxChips {
    state: Entity<InputState>,
    chips: Vec<ComboboxChip>,
    on_press: Option<Press>,
    style: StyleRefinement,
}

impl ComboboxChips {
    /// Chips around a text box showing and editing `state`.
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            chips: Vec::new(),
            on_press: None,
            style: StyleRefinement::default(),
        }
    }

    /// A `Chip` part, before the text box: for chips drawn on their own. Inside a
    /// [`Combobox`], its chosen values draw the chips.
    pub fn chip(mut self, chip: ComboboxChip) -> Self {
        self.chips.push(chip);
        self
    }

    fn replace_chips(mut self, chips: impl IntoIterator<Item = ComboboxChip>) -> Self {
        self.chips = chips.into_iter().collect();
        self
    }

    fn on_press(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ComboboxChips {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focused = self.state.read(cx).focus_handle(cx).is_focused(window);
        let theme = cx.theme();
        let s = theme.semantic();
        // border-input; focus-within:border-ring focus-within:ring-3 focus-within:ring-ring/50
        let outline = Outline::new(theme, false);
        let (border, ring) = match outline.focus.filter(|_| focused) {
            Some((border, ring)) => (border, Some(ring)),
            None => (outline.border, outline.ring),
        };
        // bg-input-background, made opaque: it sits on shadow-xs and the ring
        let fill = on_background(theme, s.input_background);
        let has_chips = !self.chips.is_empty();
        // ComboboxChipsInput: min-w-16 flex-1 outline-none
        let input = div()
            .id("combobox-chip-input")
            .flex()
            .flex_1()
            .min_w(space::S16)
            .items_center()
            .child(gpui_base::input::Input::new(&self.state));
        // flex min-h-9 flex-wrap items-center gap-1.5 rounded-md border border-input px-2.5
        // py-1.5 text-sm shadow-xs has-data-[slot=combobox-chip]:px-1.5
        gpui_base::input::InputBase::new((
            "brrritscold-core-gpui-combobox-chips",
            self.state.entity_id(),
        ))
        .focused(focused)
        .flex()
        .flex_wrap()
        .items_center()
        .gap(space::S1_5)
        .min_h(space::S9)
        .w_full()
        .rounded(s.radius_md)
        .border_1()
        .border_color(border)
        .bg(fill)
        .shadow(shadows(ring, true))
        .px(if has_chips { space::S1_5 } else { space::S2_5 })
        .py(space::S1_5)
        .text_size(font_size::SM)
        // A press anywhere but on a remove button is the text box's (Base UI focuses it).
        .when_some(self.on_press, |d, press| {
            d.on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                press(window, cx)
            })
        })
        .children(self.chips)
        .child(input)
        .refine_style(&self.style)
    }
}

/// The combobox's `Chip` part: one chosen value, with a button that lets it go.
#[derive(IntoElement)]
pub struct ComboboxChip {
    label: SharedString,
    on_remove: Option<Press>,
    style: StyleRefinement,
}

impl ComboboxChip {
    /// A chip reading `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            on_remove: None,
            style: StyleRefinement::default(),
        }
    }

    fn on_remove(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ComboboxChip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // ComboboxChipRemove: a ghost icon-xs Button, -ml-1 opacity-50 hover:opacity-100
        let remove = div()
            .id("combobox-chip-remove")
            .ml(-space::S1)
            .opacity(opacity::O50)
            .hover(|style| style.opacity(1.))
            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                Button::new("remove")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconXs)
                    .icon(IconName::X)
                    .when_some(self.on_remove, |b, remove| {
                        b.on_click(move |_, window, cx| remove(window, cx))
                    }),
            );
        // flex h-5.5 w-fit items-center justify-center gap-1 rounded-sm bg-muted px-1.5
        // text-xs font-medium whitespace-nowrap text-foreground
        // has-data-[slot=combobox-chip-remove]:pr-0
        div()
            .id(ElementId::Name(self.label.clone()))
            .debug_selector(|| "combobox-chip".into())
            .flex()
            .flex_none()
            .h(space::S5_5)
            .items_center()
            .justify_center()
            .gap(space::S1)
            .rounded(s.radius_sm)
            .bg(s.muted)
            .pl(space::S1_5)
            .text_size(font_size::XS)
            .font_weight(font_weight::MEDIUM)
            .whitespace_nowrap()
            .text_color(s.foreground)
            .child(self.label)
            .child(remove)
            .refine_style(&self.style)
    }
}

enum ComboboxChild {
    Group(ComboboxGroup),
    Item(ComboboxItem),
    Separator(ComboboxSeparator),
}

impl ComboboxChild {
    fn has_items(&self) -> bool {
        match self {
            ComboboxChild::Group(group) => !group.items.is_empty(),
            ComboboxChild::Item(_) => true,
            ComboboxChild::Separator(_) => false,
        }
    }
}

/// The combobox's `Content` part: the list of its groups, items and separators, or its
/// `Empty` part when nothing matches.
#[derive(IntoElement)]
pub struct ComboboxContent {
    children: Vec<ComboboxChild>,
    empty: Option<ComboboxEmpty>,
    context: Option<Rc<ComboboxContext>>,
    style: StyleRefinement,
}

impl Default for ComboboxContent {
    fn default() -> Self {
        Self::new()
    }
}

impl ComboboxContent {
    /// An empty list.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            empty: None,
            context: None,
            style: StyleRefinement::default(),
        }
    }

    /// A group of items under a label (code-only, as core-react's `ComboboxGroup`).
    pub fn group(mut self, group: ComboboxGroup) -> Self {
        self.children.push(ComboboxChild::Group(group));
        self
    }

    /// An item outside any group.
    pub fn item(mut self, item: ComboboxItem) -> Self {
        self.children.push(ComboboxChild::Item(item));
        self
    }

    /// The `Separator` part, between groups.
    pub fn separator(mut self, separator: ComboboxSeparator) -> Self {
        self.children.push(ComboboxChild::Separator(separator));
        self
    }

    /// The `Empty` part, shown in place of the list when no item matches.
    pub fn empty(mut self, empty: ComboboxEmpty) -> Self {
        self.empty = Some(empty);
        self
    }

    fn items(&self) -> impl Iterator<Item = &ComboboxItem> {
        self.children.iter().flat_map(|child| match child {
            ComboboxChild::Group(group) => group.items.iter().collect::<Vec<_>>(),
            ComboboxChild::Item(item) => vec![item],
            ComboboxChild::Separator(_) => Vec::new(),
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

    /// Every item's label by value, kept after the list itself is drawn.
    fn labels(&self) -> impl Fn(&SharedString) -> Option<SharedString> + 'static {
        let labels: Vec<(SharedString, SharedString)> = self
            .items()
            .map(|item| (item.value.clone(), item.label.clone()))
            .collect();
        move |value| {
            labels
                .iter()
                .find(|(v, _)| v == value)
                .map(|(_, label)| label.clone())
        }
    }

    /// Keeps the items whose label contains `query` (already lowercase), the groups left with
    /// any, and the separators left between two of those.
    fn filtered(mut self, query: Option<&str>) -> Self {
        let Some(query) = query else {
            return self;
        };
        let matches = |item: &ComboboxItem| item.label.to_lowercase().contains(query);
        let mut kept: Vec<ComboboxChild> = Vec::new();
        for child in self.children {
            let child = match child {
                ComboboxChild::Group(mut group) => {
                    group.items.retain(|item| matches(item));
                    ComboboxChild::Group(group)
                }
                ComboboxChild::Item(item) if !matches(&item) => continue,
                other => other,
            };
            let keep = match &child {
                ComboboxChild::Separator(_) => kept.last().is_some_and(ComboboxChild::has_items),
                child => child.has_items(),
            };
            if keep {
                kept.push(child);
            }
        }
        if matches!(kept.last(), Some(ComboboxChild::Separator(_))) {
            kept.pop();
        }
        self.children = kept;
        self
    }

    fn context(mut self, context: Rc<ComboboxContext>) -> Self {
        self.context = Some(context);
        self
    }
}

impl RenderOnce for ComboboxContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let context = self.context;
        let empty = !self.children.iter().any(ComboboxChild::has_items);
        let mut shadows = vec![super::ring(1., s.foreground.opacity(0.1))];
        shadows.extend_from_slice(shadow::MD);
        // relative overflow-hidden rounded-md bg-popover text-popover-foreground shadow-md
        // ring-1 ring-foreground/10
        div()
            .debug_selector(|| "combobox-content".into())
            .relative()
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(s.radius_md)
            .bg(s.popover)
            .text_color(s.popover_foreground)
            .shadow(shadows)
            // ComboboxEmpty: hidden … group-data-empty/combobox-content:flex
            .when(empty, |d| d.children(self.empty))
            // ComboboxList: p-1 data-empty:p-0
            .child(
                div()
                    .flex()
                    .flex_col()
                    .when(!empty, |d| d.p(space::S1))
                    .children(self.children.into_iter().map(|child| match child {
                        ComboboxChild::Group(group) => {
                            group.context(context.clone()).into_any_element()
                        }
                        ComboboxChild::Item(item) => {
                            item.context(context.clone()).into_any_element()
                        }
                        ComboboxChild::Separator(separator) => separator.into_any_element(),
                    })),
            )
            .refine_style(&self.style)
    }
}

/// A group of items under an optional label: core-react's `ComboboxGroup`. A code-only part.
#[derive(IntoElement)]
pub struct ComboboxGroup {
    label: Option<ComboboxLabel>,
    items: Vec<ComboboxItem>,
    context: Option<Rc<ComboboxContext>>,
}

impl Default for ComboboxGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ComboboxGroup {
    /// An empty group.
    pub fn new() -> Self {
        Self {
            label: None,
            items: Vec::new(),
            context: None,
        }
    }

    /// The `Label` part, over the items.
    pub fn label(mut self, label: ComboboxLabel) -> Self {
        self.label = Some(label);
        self
    }

    /// An `Item` part.
    pub fn item(mut self, item: ComboboxItem) -> Self {
        self.items.push(item);
        self
    }

    fn context(mut self, context: Option<Rc<ComboboxContext>>) -> Self {
        self.context = context;
        self
    }
}

impl RenderOnce for ComboboxGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let context = self.context;
        div().flex().flex_col().children(self.label).children(
            self.items
                .into_iter()
                .map(|item| item.context(context.clone())),
        )
    }
}

/// The combobox's `Label` part: a group's heading.
#[derive(IntoElement)]
pub struct ComboboxLabel {
    text: SharedString,
    style: StyleRefinement,
}

impl ComboboxLabel {
    /// A label reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for ComboboxLabel {
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

/// The combobox's `Item` part: a choice, with a check (its `Indicator` part) once chosen.
#[derive(IntoElement)]
pub struct ComboboxItem {
    value: SharedString,
    label: SharedString,
    disabled: bool,
    selected: bool,
    context: Option<Rc<ComboboxContext>>,
    style: StyleRefinement,
}

impl ComboboxItem {
    /// An item for `value`, reading `label`; typing filters on the label.
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

    /// Shows the item as chosen outside a [`Combobox`], as its check does: for an item drawn on
    /// its own. Inside a combobox, its values decide.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn context(mut self, context: Option<Rc<ComboboxContext>>) -> Self {
        self.context = context;
        self
    }
}

impl RenderOnce for ComboboxItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let context = self.context;
        let value = self.value;
        let selected = context
            .as_ref()
            .map_or(self.selected, |c| c.selected.contains(&value));
        let highlighted = context
            .as_ref()
            .is_some_and(|c| c.highlighted.as_ref() == Some(&value));
        let enabled = !self.disabled;
        // relative flex w-full items-center gap-2 rounded-sm py-1.5 pr-8 pl-2 text-sm
        // data-highlighted:bg-accent data-highlighted:text-accent-foreground
        // data-disabled:opacity-50
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
            // ItemIndicator: pointer-events-none absolute right-2 flex size-4 items-center
            // justify-center, a check
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

/// The combobox's `Separator` part: a rule between groups.
#[derive(IntoElement)]
pub struct ComboboxSeparator {
    style: StyleRefinement,
}

impl Default for ComboboxSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl ComboboxSeparator {
    /// A separator.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for ComboboxSeparator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // -mx-1 my-1 h-px bg-border: the list's p-1 undone, so the rule runs edge to edge
        div()
            .mx(-space::S1)
            .my(space::S1)
            .h(px(1.))
            .bg(cx.theme().semantic().border)
            .refine_style(&self.style)
    }
}

/// The combobox's `Empty` part: the message shown when no item matches.
#[derive(IntoElement)]
pub struct ComboboxEmpty {
    text: SharedString,
    style: StyleRefinement,
}

impl ComboboxEmpty {
    /// A message reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for ComboboxEmpty {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // flex w-full justify-center py-2 text-center text-sm text-muted-foreground
        div()
            .flex()
            .w_full()
            .justify_center()
            .py(space::S2)
            .text_size(font_size::SM)
            .text_color(cx.theme().semantic().muted_foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

impl_styled!(
    Combobox,
    ComboboxInput,
    ComboboxChips,
    ComboboxChip,
    ComboboxContent,
    ComboboxLabel,
    ComboboxItem,
    ComboboxSeparator,
    ComboboxEmpty
);

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Modifiers, TestAppContext, VisualTestContext, point};

    use super::*;
    use crate::components::test_support::{render, state};

    const FRAMEWORKS: [&str; 5] = ["Next.js", "SvelteKit", "Nuxt", "Remix", "Astro"];

    /// The field's width in every test.
    const WIDTH: Pixels = px(320.);

    type Changes = Rc<RefCell<Vec<Vec<SharedString>>>>;

    fn content() -> ComboboxContent {
        FRAMEWORKS.iter().fold(
            ComboboxContent::new().empty(ComboboxEmpty::new("No frameworks found.")),
            |content, framework| content.item(ComboboxItem::new(*framework, *framework)),
        )
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
    }

    /// The story's framework combobox, recording its changes. `chips` draws the multiple-choice
    /// one, starting on Next.js.
    fn framework(
        cx: &mut TestAppContext,
        chips: bool,
        disabled: bool,
    ) -> (&mut VisualTestContext, Entity<InputState>, Changes) {
        cx.update(crate::init);
        let input = state(cx, |window, cx| {
            InputState::new(window, cx).placeholder("Pick a framework")
        });
        let changes = Changes::default();
        let (shown, recorded) = (input.clone(), changes.clone());
        let (cx, _) = render(cx, move || {
            let recorded = recorded.clone();
            let combobox = Combobox::new("framework");
            let combobox = if chips {
                combobox
                    .chips(ComboboxChips::new(&shown))
                    .default_values(["Next.js"])
            } else {
                combobox.input(ComboboxInput::new(&shown).disabled(disabled))
            };
            combobox
                .content(content())
                .on_value_change(move |values, _, _| recorded.borrow_mut().push(values.to_vec()))
                .w(WIDTH)
        });
        (cx, input, changes)
    }

    /// Presses the field's text box and types `text` into it.
    fn type_in(cx: &mut VisualTestContext, text: &str) {
        cx.simulate_click(point(px(160.), px(18.)), Modifiers::default());
        draw(cx);
        cx.simulate_input(text);
        draw(cx);
    }

    /// The open list's height, `None` while it's closed.
    fn list_height(cx: &mut VisualTestContext) -> Option<Pixels> {
        cx.debug_bounds("combobox-content")
            .map(|bounds| bounds.size.height)
    }

    /// The list's height for `rows` items: the list's 4px padding and 33px rows (14px text at
    /// 1.5, 6px above and below).
    fn rows(rows: usize) -> Option<Pixels> {
        Some(px(8. + 33. * rows as f32))
    }

    mod combobox {
        use super::*;

        #[gpui::test]
        fn should_open_on_a_press_showing_every_item(cx: &mut TestAppContext) {
            let (cx, _, _) = framework(cx, false, false);
            cx.simulate_click(point(px(160.), px(18.)), Modifiers::default());
            draw(cx);
            assert_eq!(list_height(cx), rows(5));
        }

        #[gpui::test]
        fn should_filter_the_items_to_the_typed_text(cx: &mut TestAppContext) {
            let (cx, _, _) = framework(cx, false, false);
            type_in(cx, "sv");
            assert_eq!(list_height(cx), rows(1));
        }

        #[gpui::test]
        fn should_choose_with_the_arrow_keys_and_enter(cx: &mut TestAppContext) {
            let (cx, input, changes) = framework(cx, false, false);
            // "n" leaves Next.js and Nuxt; the first Down highlights Next.js.
            type_in(cx, "n");
            assert_eq!(list_height(cx), rows(2));
            cx.simulate_keystrokes("down down enter");
            draw(cx);
            assert_eq!(*changes.borrow(), [["Nuxt"]]);
            assert_eq!(cx.read(|cx| input.read(cx).value()), "Nuxt");
            assert_eq!(list_height(cx), None);
        }

        #[gpui::test]
        fn should_show_every_item_again_once_one_is_chosen(cx: &mut TestAppContext) {
            let (cx, _, _) = framework(cx, false, false);
            type_in(cx, "sv");
            cx.simulate_keystrokes("down enter");
            draw(cx);
            // The chosen label is the text now, and filters nothing.
            cx.simulate_keystrokes("down");
            draw(cx);
            assert_eq!(list_height(cx), rows(5));
        }

        #[gpui::test]
        fn should_choose_an_item_with_the_pointer(cx: &mut TestAppContext) {
            let (cx, _, changes) = framework(cx, false, false);
            type_in(cx, "t");
            let list = cx
                .debug_bounds("combobox-content")
                .expect("the list opened");
            // "t" leaves Next.js, SvelteKit, Nuxt and Astro: Nuxt is the third row.
            cx.simulate_click(
                point(list.center().x, list.top() + px(4. + 33. * 2. + 16.)),
                Modifiers::default(),
            );
            draw(cx);
            assert_eq!(*changes.borrow(), [["Nuxt"]]);
        }

        #[gpui::test]
        fn should_show_the_empty_message_when_nothing_matches(cx: &mut TestAppContext) {
            let (cx, _, _) = framework(cx, false, false);
            type_in(cx, "zzz");
            // The message alone: 8px above and below 21px of text, and no list padding.
            assert_eq!(list_height(cx), Some(px(37.)));
        }

        #[gpui::test]
        fn should_close_on_escape_without_choosing(cx: &mut TestAppContext) {
            let (cx, _, changes) = framework(cx, false, false);
            type_in(cx, "n");
            cx.simulate_keystrokes("down escape");
            draw(cx);
            assert_eq!(list_height(cx), None);
            assert!(changes.borrow().is_empty());
        }

        #[gpui::test]
        fn should_stay_closed_while_disabled(cx: &mut TestAppContext) {
            let (cx, input, _) = framework(cx, false, true);
            type_in(cx, "n");
            cx.simulate_keystrokes("down");
            draw(cx);
            assert_eq!(list_height(cx), None);
            assert_eq!(cx.read(|cx| input.read(cx).value()), "");
        }
    }

    mod chips {
        use super::*;

        #[gpui::test]
        fn should_add_a_chip_for_each_choice_and_clear_the_text(cx: &mut TestAppContext) {
            let (cx, input, changes) = framework(cx, true, false);
            type_in(cx, "nu");
            cx.simulate_keystrokes("down enter");
            draw(cx);
            assert_eq!(*changes.borrow(), [["Next.js", "Nuxt"]]);
            assert_eq!(cx.read(|cx| input.read(cx).value()), "");
            assert_eq!(list_height(cx), None);
        }

        #[gpui::test]
        fn should_remove_a_chip_from_its_button(cx: &mut TestAppContext) {
            let (cx, _, changes) = framework(cx, true, false);
            let chip = cx
                .debug_bounds("combobox-chip")
                .expect("Next.js has a chip");
            // The remove button is the chip's last 24px.
            cx.simulate_click(
                point(chip.right() - px(12.), chip.center().y),
                Modifiers::default(),
            );
            draw(cx);
            assert_eq!(*changes.borrow(), [Vec::<SharedString>::new()]);
        }

        #[gpui::test]
        fn should_remove_the_last_chip_on_backspace_in_an_empty_input(cx: &mut TestAppContext) {
            let (cx, _, changes) = framework(cx, true, false);
            type_in(cx, "");
            cx.simulate_keystrokes("backspace");
            draw(cx);
            assert_eq!(*changes.borrow(), [Vec::<SharedString>::new()]);
        }

        #[gpui::test]
        fn should_let_a_chosen_item_go_when_chosen_again(cx: &mut TestAppContext) {
            let (cx, _, changes) = framework(cx, true, false);
            type_in(cx, "next");
            cx.simulate_keystrokes("down enter");
            draw(cx);
            assert_eq!(*changes.borrow(), [Vec::<SharedString>::new()]);
        }
    }

    mod input {
        use super::*;

        #[gpui::test]
        fn should_be_36px_tall(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let input = state(cx, InputState::new);
            let (_, bounds) = render(cx, move || ComboboxInput::new(&input).w(WIDTH));
            assert_eq!(bounds.size.height, px(36.));
        }

        #[gpui::test]
        fn should_clear_the_value_from_its_clear_button(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let input = state(cx, InputState::new);
            let changes = Changes::default();
            let (shown, recorded) = (input.clone(), changes.clone());
            let (cx, field) = render(cx, move || {
                let recorded = recorded.clone();
                Combobox::new("framework")
                    .input(ComboboxInput::new(&shown).show_clear(true))
                    .content(content())
                    .default_value("Astro")
                    .on_value_change(move |values, _, _| {
                        recorded.borrow_mut().push(values.to_vec())
                    })
                    .w(WIDTH)
            });
            assert_eq!(cx.read(|cx| input.read(cx).value()), "Astro");
            // The clear button: 24px, 4px in from the right edge.
            cx.simulate_click(
                point(field.right() - px(16.), field.center().y),
                Modifiers::default(),
            );
            draw(cx);
            assert_eq!(*changes.borrow(), [Vec::<SharedString>::new()]);
            assert_eq!(cx.read(|cx| input.read(cx).value()), "");
        }
    }

    mod chip {
        use super::*;

        #[gpui::test]
        fn should_be_22px_tall(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || ComboboxChip::new("Next.js"));
            assert_eq!(bounds.size.height, px(22.));
        }
    }

    mod content {
        use super::*;

        fn values(content: ComboboxContent) -> Vec<SharedString> {
            content.enabled_values().to_vec()
        }

        #[test]
        fn should_keep_groups_and_separators_with_matching_items_around_them() {
            let grouped = || {
                ComboboxContent::new()
                    .group(
                        ComboboxGroup::new()
                            .label(ComboboxLabel::new("Fruits"))
                            .item(ComboboxItem::new("apple", "Apple"))
                            .item(ComboboxItem::new("banana", "Banana")),
                    )
                    .separator(ComboboxSeparator::new())
                    .group(ComboboxGroup::new().item(ComboboxItem::new("leek", "Leek")))
            };
            assert_eq!(values(grouped().filtered(Some("ee"))), ["leek"]);
            let filtered = grouped().filtered(Some("ee"));
            assert!(
                !filtered
                    .children
                    .iter()
                    .any(|child| matches!(child, ComboboxChild::Separator(_)))
            );
            assert_eq!(values(grouped().filtered(Some("a"))), ["apple", "banana"]);
        }
    }
}
