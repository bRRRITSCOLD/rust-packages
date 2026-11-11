//! Input Group: core-react's internal `input-group.tsx`, the frame `ComboboxInput` draws. Like
//! core-react's, it isn't exported: Input Group has no library twin, so exporting it is a
//! promotion. Only the parts `ComboboxInput` uses are mirrored: the group, its input
//! (`InputGroupInput`), an `inline-end` addon and an `icon-xs` ghost button
//! (`InputGroupButton`).
//!
//! Not mirrored: the other addon alignments, `InputGroupText`, `InputGroupTextarea`, the
//! `xs` / `sm` / `icon-sm` button sizes and the `aria-invalid` arms, which nothing here uses.
//! The `in-data-[slot=combobox-content]` arms are for a search input inside the list, which
//! core-gpui's combobox doesn't draw.

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, opacity, space};
use gpui::{
    AnyElement, App, ClickEvent, ElementId, Entity, Focusable as _, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, Pixels, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _, transparent_black,
};
use gpui_base::StyledExt as _;
use gpui_base::input::{InputBase, InputState};

use super::control::{Outline, on_background, shadows};
use crate::icons::{Icon, IconName};
use crate::theme::ActiveTheme as _;

type Handler = Box<dyn Fn(&mut Window, &mut App)>;
type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// The group: an input over `state` and an optional `inline-end` addon, in one frame.
#[derive(IntoElement)]
pub(crate) struct InputGroup {
    state: Entity<InputState>,
    disabled: bool,
    addon: Vec<AnyElement>,
    on_input_press: Option<Handler>,
    style: StyleRefinement,
}

impl InputGroup {
    pub(crate) fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            disabled: false,
            addon: Vec::new(),
            on_input_press: None,
            style: StyleRefinement::default(),
        }
    }

    /// Dims the input and stops it taking focus or text. The frame stays as it is: core-react
    /// fades only the input and its buttons.
    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// An element of the `inline-end` addon, after the input.
    pub(crate) fn addon(mut self, element: impl IntoElement) -> Self {
        self.addon.push(element.into_any_element());
        self
    }

    /// Runs on a press in the input itself, not in its addon.
    pub(crate) fn on_input_press(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_input_press = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for InputGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.disabled;
        self.state
            .update(cx, |state, cx| state.set_disabled(disabled, cx));
        let focused = self.state.read(cx).focus_handle(cx).is_focused(window);
        let theme = cx.theme();
        let s = theme.semantic();
        // border-input, and has-[[data-slot=input-group-control]:focus-visible]:border-ring
        // ring-3 ring-ring/50: a text field is focus-visible whenever it has focus.
        let outline = Outline::new(theme, false);
        let (border, ring) = match outline.focus.filter(|_| focused && !disabled) {
            Some((border, ring)) => (border, Some(ring)),
            None => (outline.border, outline.ring),
        };
        // dark:bg-input/30, made opaque: it sits on shadow-xs and the ring
        let fill = on_background(
            theme,
            theme.mode().pick(transparent_black(), s.input.opacity(0.3)),
        );
        let has_addon = !self.addon.is_empty();
        // InputGroupInput: Input's `px-2.5 py-1` with `flex-1 rounded-none border-0
        // bg-transparent shadow-none`, and the group's has-[>[data-align=inline-end]]:[&>input]:pr-1.5
        let input = div()
            .id("input-group-control")
            .flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .items_center()
            .pl(space::S2_5)
            .pr(if has_addon { space::S1_5 } else { space::S2_5 })
            .py(space::S1)
            .when(disabled, |d| d.opacity(opacity::O50))
            .when_some(self.on_input_press.filter(|_| !disabled), |d, handler| {
                d.on_mouse_down(MouseButton::Left, move |_, window, cx| handler(window, cx))
            })
            .child(gpui_base::input::Input::new(&self.state));
        // InputGroupAddon align="inline-end": flex h-auto items-center justify-center gap-2
        // py-1.5 text-sm font-medium text-muted-foreground order-last pr-2 has-[>button]:-mr-1
        let addon = has_addon.then(|| {
            div()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .gap(space::S2)
                .py(space::S1_5)
                .pr(space::S2)
                .mr(-space::S1)
                .font_weight(font_weight::MEDIUM)
                .text_color(s.muted_foreground)
                .children(self.addon)
        });
        // relative flex h-9 w-full min-w-0 items-center rounded-md border border-input
        // shadow-xs
        InputBase::new(("brrritscold-core-gpui-input-group", self.state.entity_id()))
            .focused(focused)
            .disabled(disabled)
            .relative()
            .flex()
            .items_center()
            .h(space::S9)
            .w_full()
            .min_w_0()
            .rounded(s.radius_md)
            .border_1()
            .border_color(border)
            .bg(fill)
            .shadow(shadows(ring, true))
            .text_size(font_size::SM)
            .child(input)
            .children(addon)
            .refine_style(&self.style)
    }
}

/// `InputGroupButton` at `size="icon-xs"`, `variant="ghost"`: a 24px square holding an icon in
/// the addon's text colour.
#[derive(IntoElement)]
pub(crate) struct InputGroupButton {
    id: ElementId,
    icon: IconName,
    icon_size: Pixels,
    label: SharedString,
    disabled: bool,
    focusable: bool,
    on_click: Option<ClickHandler>,
}

impl InputGroupButton {
    /// A button named `label` for assistive technology, showing `icon` at `icon_size`.
    pub(crate) fn new(
        id: impl Into<ElementId>,
        icon: IconName,
        icon_size: Pixels,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon,
            icon_size,
            label: label.into(),
            disabled: false,
            focusable: true,
            on_click: None,
        }
    }

    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Keeps the button out of the Tab order and leaves focus where it is when pressed
    /// (`tabIndex={-1}`).
    pub(crate) fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub(crate) fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for InputGroupButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let s = theme.semantic();
        // ghost: hover:bg-muted hover:text-foreground dark:hover:bg-muted/50
        let hover_fill = theme.mode().pick(s.muted, s.muted.opacity(0.5));
        let foreground = s.foreground;
        // size-6 rounded-sm p-0, disabled:opacity-50
        gpui_base::Button::new(self.id)
            .accessibility_label(self.label)
            .disabled(self.disabled)
            .tab_stop(self.focusable)
            .focusable(self.focusable)
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(space::S6)
            .rounded(s.radius_sm)
            .when(!self.disabled, |b| {
                b.hover(move |style| style.bg(hover_fill).text_color(foreground))
            })
            .when(self.disabled, |b| b.opacity(opacity::O50))
            .when_some(self.on_click, |b, handler| {
                b.on_click(move |event, window, cx| handler(event, window, cx))
            })
            .child(Icon::new(self.icon).size(self.icon_size))
    }
}

impl_styled!(InputGroup);
