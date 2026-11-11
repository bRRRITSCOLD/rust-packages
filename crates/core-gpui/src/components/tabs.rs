//! Tabs: core-react's `tabs.tsx` on `gpui-base`'s unstyled [`gpui_base::Tabs`] (the tab list's
//! role) and [`gpui_base::Tab`] (each tab's role, selected state, disabled inertness and pointer
//! activation).
//!
//! [`Tabs`] keeps the selected tab, its own by default (Base UI's `defaultValue`) or the
//! caller's ([`Tabs::value`]); it hands its orientation and selection to its [`TabsList`], which
//! hands them and its variant to each [`TabsTrigger`], and shows only the selected
//! [`TabsContent`]. core-react reaches those through `group-data-*`; GPUI has no such
//! selectors, so the parts are typed builders on their parent.
//!
//! Not mirrored:
//! - Arrow-key roving focus: `gpui-base`'s tabs have none. Each tab is its own Tab stop and Enter
//!   or Space selects it.
//! - The `has-data-[icon=…]` padding and `[&_svg]` rules: a trigger takes a label only.
//! - The line underline's opacity transition: it shows at once.

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::{
    font_size, font_weight, line_height, opacity, shadow, space,
};
use gpui::{
    AnyElement, App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
    px, relative, transparent_black,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use crate::theme::{ActiveTheme as _, Theme};

/// Penpot variant property `orientation` of `Tabs`. Cases are its values in PascalCase;
/// [`Display`] gives the Penpot value back.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum TabsOrientation {
    /// `horizontal`: the list above the panel.
    #[default]
    Horizontal,
    /// `vertical`: the list beside the panel, its tabs stacked.
    Vertical,
}

/// Penpot variant property `variant` of `TabsList`. Cases are its values in PascalCase;
/// [`Display`] gives the Penpot value back.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum TabsListVariant {
    /// `default`: a `muted` track, the selected tab lifted onto `background`.
    #[default]
    Default,
    /// `line`: no track, the selected tab underlined.
    Line,
}

type ValueChangeHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// Tabs: a [`TabsList`] of triggers and a [`TabsContent`] per tab, the selected one showing.
///
/// ```no_run
/// use brrritscold_core_gpui::{Tabs, TabsContent, TabsList, TabsTrigger};
/// use gpui::ParentElement as _;
///
/// let settings = Tabs::new("settings")
///     .default_value("account")
///     .list(
///         TabsList::new()
///             .trigger(TabsTrigger::new("account", "Account"))
///             .trigger(TabsTrigger::new("password", "Password")),
///     )
///     .content(TabsContent::new("account").child("Change your name and email address."))
///     .content(TabsContent::new("password").child("Change your password."))
///     .on_value_change(|tab, _, _| println!("{tab}"));
/// ```
#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    orientation: TabsOrientation,
    list: Option<TabsList>,
    contents: Vec<TabsContent>,
    value: Option<SharedString>,
    default_value: Option<SharedString>,
    on_value_change: Option<ValueChangeHandler>,
    style: StyleRefinement,
}

impl Tabs {
    /// Horizontal tabs with nothing selected. `id` keys the selection.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            orientation: TabsOrientation::default(),
            list: None,
            contents: Vec::new(),
            value: None,
            default_value: None,
            on_value_change: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `orientation`.
    pub fn orientation(mut self, orientation: TabsOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// The `List` part.
    pub fn list(mut self, list: TabsList) -> Self {
        self.list = Some(list);
        self
    }

    /// A `Content` part; it shows while its tab is selected.
    pub fn content(mut self, content: TabsContent) -> Self {
        self.contents.push(content);
        self
    }

    /// The selected tab's value, owned by the caller; it then changes only when the caller
    /// passes another.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// The tab selected at first, when the tabs keep their own value.
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Runs when a tab is selected, with its value.
    pub fn on_value_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let default_value = self.default_value;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| default_value);
        if let Some(value) = self.value {
            state.update(cx, |selected, _| *selected = Some(value));
        }
        let selected = state.read(cx).clone();
        let on_select: ValueChangeHandler = Rc::new({
            let on_value_change = self.on_value_change;
            move |value, window, cx| {
                state.update(cx, |selected, cx| {
                    *selected = Some(value.clone());
                    cx.notify();
                });
                if let Some(on_value_change) = &on_value_change {
                    on_value_change(value, window, cx);
                }
            }
        });
        let orientation = self.orientation;
        let list = self.list.map(|list| {
            list.orientation(orientation)
                .selected(selected.clone())
                .on_select(on_select)
        });
        let content = self
            .contents
            .into_iter()
            .find(|content| Some(&content.value) == selected.as_ref());
        // group/tabs flex gap-2 data-[orientation=horizontal]:flex-col
        div()
            .id(self.id)
            .flex()
            .gap(space::S2)
            .when(orientation == TabsOrientation::Horizontal, |d| d.flex_col())
            .children(list)
            .children(content)
            .refine_style(&self.style)
    }
}

/// The tabs' `List` part: the row (or, vertical, the column) of triggers.
#[derive(IntoElement)]
pub struct TabsList {
    variant: TabsListVariant,
    triggers: Vec<TabsTrigger>,
    orientation: TabsOrientation,
    selected: Option<SharedString>,
    on_select: Option<ValueChangeHandler>,
    style: StyleRefinement,
}

impl Default for TabsList {
    fn default() -> Self {
        Self::new()
    }
}

impl TabsList {
    /// A `default` list with no triggers.
    pub fn new() -> Self {
        Self {
            variant: TabsListVariant::default(),
            triggers: Vec::new(),
            orientation: TabsOrientation::default(),
            selected: None,
            on_select: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `variant`.
    pub fn variant(mut self, variant: TabsListVariant) -> Self {
        self.variant = variant;
        self
    }

    /// A `Trigger` part, after the ones already added.
    pub fn trigger(mut self, trigger: TabsTrigger) -> Self {
        self.triggers.push(trigger);
        self
    }

    fn orientation(mut self, orientation: TabsOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    fn selected(mut self, selected: Option<SharedString>) -> Self {
        self.selected = selected;
        self
    }

    fn on_select(mut self, on_select: ValueChangeHandler) -> Self {
        self.on_select = Some(on_select);
        self
    }
}

impl RenderOnce for TabsList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let vertical = self.orientation == TabsOrientation::Vertical;
        let count = self.triggers.len();
        let (variant, orientation, selected, on_select) = (
            self.variant,
            self.orientation,
            self.selected,
            self.on_select,
        );
        let triggers = self
            .triggers
            .into_iter()
            .enumerate()
            .map(|(index, trigger)| {
                let active = selected.as_ref() == Some(&trigger.value);
                trigger
                    .variant(variant)
                    .orientation(orientation)
                    .active(active)
                    .position(index + 1, count)
                    .when_some(on_select.clone(), |t, on_select| t.on_select(on_select))
            });
        // inline-flex w-fit items-center justify-center rounded-lg p-1 text-muted-foreground
        // group-data-[orientation=horizontal]/tabs:h-9
        // group-data-[orientation=vertical]/tabs:h-fit group-data-[orientation=vertical]/tabs:flex-col
        gpui_base::Tabs::new("tabs-list")
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(s.radius_lg)
            .p(space::S1)
            .text_color(s.muted_foreground)
            .map(|d| {
                if vertical {
                    d.flex_col()
                } else {
                    d.h(space::S9)
                }
            })
            .map(|d| match variant {
                // bg-muted
                TabsListVariant::Default => d.bg(s.muted),
                // gap-1 bg-transparent data-[variant=line]:rounded-none
                TabsListVariant::Line => d.gap(space::S1).rounded(px(0.)),
            })
            .children(triggers)
            .refine_style(&self.style)
    }
}

/// The tabs' `Trigger` part: a tab, lifted (or, in a `line` list, underlined) while selected.
#[derive(IntoElement)]
pub struct TabsTrigger {
    value: SharedString,
    label: SharedString,
    disabled: bool,
    variant: TabsListVariant,
    orientation: TabsOrientation,
    active: bool,
    position: Option<(usize, usize)>,
    on_select: Option<ValueChangeHandler>,
    style: StyleRefinement,
}

impl TabsTrigger {
    /// A tab reading `label`; `value` names it among its tabs.
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            variant: TabsListVariant::default(),
            orientation: TabsOrientation::default(),
            active: false,
            position: None,
            on_select: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `disabled`: dims the tab to 50% and keeps it from being selected.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn variant(mut self, variant: TabsListVariant) -> Self {
        self.variant = variant;
        self
    }

    fn orientation(mut self, orientation: TabsOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    fn position(mut self, position: usize, count: usize) -> Self {
        self.position = Some((position, count));
        self
    }

    fn on_select(mut self, on_select: ValueChangeHandler) -> Self {
        self.on_select = Some(on_select);
        self
    }
}

/// A trigger's colours, resolved for its list's variant and whether it's selected.
#[derive(Debug, Clone, Copy, PartialEq)]
struct TriggerStyle {
    fill: Option<Hsla>,
    text: Hsla,
    border: Hsla,
    shadow_sm: bool,
    /// The `line` list's underline.
    bar: Option<Hsla>,
}

impl TabsTrigger {
    fn style(&self, theme: &Theme) -> TriggerStyle {
        let s = theme.semantic();
        let m = theme.mode();
        // text-foreground/60 dark:text-muted-foreground; border-transparent
        let resting = TriggerStyle {
            fill: None,
            text: m.pick(s.foreground.opacity(0.6), s.muted_foreground),
            border: transparent_black(),
            shadow_sm: false,
            bar: None,
        };
        if !self.active {
            return resting;
        }
        match self.variant {
            // data-active:bg-background data-active:text-foreground dark:data-active:border-input
            // dark:data-active:bg-input/30 group-data-[variant=default]/tabs-list:data-active:shadow-sm.
            // The dark fill is made opaque on the list's `muted`: GPUI paints shadow-sm under the
            // whole box (see `control`).
            TabsListVariant::Default => TriggerStyle {
                fill: Some(m.pick(s.background, s.muted.blend(s.input.opacity(0.3)))),
                text: s.foreground,
                border: m.pick(transparent_black(), s.input),
                shadow_sm: true,
                bar: None,
            },
            // group-data-[variant=line]/tabs-list:data-active:bg-transparent,
            // dark:…:data-active:border-transparent, …:data-active:shadow-none, and
            // after:bg-foreground …:data-active:after:opacity-100
            TabsListVariant::Line => TriggerStyle {
                text: s.foreground,
                bar: Some(s.foreground),
                ..resting
            },
        }
    }
}

impl RenderOnce for TabsTrigger {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let s = theme.semantic();
        let focus_ring = theme.focus_ring();
        let t = self.style(theme);
        let vertical = self.orientation == TabsOrientation::Vertical;
        let id = ElementId::Name(format!("tabs-trigger-{}", self.value).into());
        let focus = window.use_keyed_state(id.clone(), cx, |_, cx| cx.focus_handle());
        // aria-disabled: a disabled tab stays focusable, as in core-react.
        let focus = focus.read(cx).clone().tab_stop(true);
        let enabled = !self.disabled;
        let foreground = s.foreground;
        let value = self.value;
        // after:absolute after:bg-foreground: inset-x-0 -bottom-1 h-0.5, or (vertical)
        // inset-y-0 -right-1 w-0.5
        let bar = t.bar.map(|bar| {
            div().absolute().bg(bar).map(|d| {
                if vertical {
                    d.top_0().bottom_0().right(-space::S1).w(space::S0_5)
                } else {
                    d.left_0().right_0().bottom(-space::S1).h(space::S0_5)
                }
            })
        });
        gpui_base::Tab::new(id)
            .selected(self.active)
            .disabled(self.disabled)
            .accessibility_label(self.label.clone())
            .when_some(self.position, |tab, (position, count)| {
                tab.set_position(position, count)
            })
            .track_focus(&focus)
            .when_some(self.on_select.filter(|_| enabled), |tab, on_select| {
                tab.on_click(move |_, window, cx| on_select(&value, window, cx))
            })
            // relative inline-flex h-full flex-1 items-center justify-center gap-1.5 rounded-md
            // border border-transparent px-2 py-1 text-sm font-medium whitespace-nowrap
            .relative()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .gap(space::S1_5)
            .rounded(s.radius_md)
            .border_1()
            .border_color(t.border)
            .px(space::S2)
            .py(space::S1)
            .text_size(font_size::SM)
            .font_weight(font_weight::MEDIUM)
            .line_height(relative(line_height::NORMAL))
            .whitespace_nowrap()
            .text_color(t.text)
            // h-full: the horizontal list's 36px less its 4px padding each side.
            // group-data-[orientation=vertical]/tabs:w-full …:justify-start
            .map(|tab| {
                if vertical {
                    tab.w_full().justify_start()
                } else {
                    tab.h(space::S7)
                }
            })
            .when_some(t.fill, |tab, fill| tab.bg(fill))
            .when(t.shadow_sm, |tab| tab.shadow(shadow::SM.to_vec()))
            // hover:text-foreground dark:hover:text-foreground
            .when(enabled, |tab| {
                tab.hover(move |style| style.text_color(foreground))
            })
            // focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50
            .focus_visible(move |style| {
                style
                    .border_color(s.ring)
                    .shadow(vec![super::ring(3., focus_ring)])
            })
            // disabled:opacity-50 aria-disabled:opacity-50
            .when(self.disabled, |tab| tab.opacity(opacity::O50))
            .child(self.label)
            .children(bar)
            .refine_style(&self.style)
    }
}

/// The tabs' `Content` part: a tab's panel (its Penpot `Body` slot).
#[derive(IntoElement)]
pub struct TabsContent {
    value: SharedString,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl TabsContent {
    /// The panel of the tab whose value is `value`.
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for TabsContent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex-1 text-sm outline-none
        div()
            .flex_1()
            .text_size(font_size::SM)
            .debug_selector(|| "tabs-content".into())
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl_styled!(Tabs, TabsList, TabsTrigger, TabsContent);
impl_parent!(TabsContent);

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Modifiers, TestAppContext, VisualTestContext, point};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::{press, render, theme};
    use crate::theme::Mode;

    type Log = Rc<RefCell<Vec<SharedString>>>;

    const TABS: [(&str, &str, &str); 3] = [
        ("account", "Account", "Change your name and email address."),
        ("password", "Password", "Change your password."),
        ("billing", "Billing", "Manage your plan and invoices."),
    ];

    /// The story's settings tabs, Billing disabled, logging every selection.
    fn settings(log: Log, orientation: TabsOrientation) -> impl Fn() -> Tabs {
        move || {
            let log = log.clone();
            let list = TABS
                .into_iter()
                .fold(TabsList::new(), |list, (value, label, _)| {
                    list.trigger(TabsTrigger::new(value, label).disabled(value == "billing"))
                });
            TABS.into_iter()
                .fold(
                    Tabs::new("settings")
                        .orientation(orientation)
                        .default_value("account")
                        .list(list),
                    |tabs, (value, _, body)| tabs.content(TabsContent::new(value).child(body)),
                )
                .on_value_change(move |value, _, _| log.borrow_mut().push(value.clone()))
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    fn content(cx: &mut VisualTestContext) -> gpui::Bounds<gpui::Pixels> {
        cx.debug_bounds("tabs-content").expect("a panel shows")
    }

    mod values {
        use super::*;

        #[test]
        fn should_name_orientations_as_penpot_does() {
            let values: Vec<&str> = TabsOrientation::iter().map(Into::into).collect();
            assert_eq!(values, ["horizontal", "vertical"]);
        }

        #[test]
        fn should_name_list_variants_as_penpot_does() {
            let values: Vec<&str> = TabsListVariant::iter().map(Into::into).collect();
            assert_eq!(values, ["default", "line"]);
        }
    }

    mod trigger {
        use super::*;

        #[gpui::test]
        fn should_lift_the_selected_tab_onto_background(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Light);
            let tab = TabsTrigger::new("a", "A").active(true).style(&theme);
            assert_eq!(tab.fill, Some(theme.semantic().background));
            assert!(tab.shadow_sm);
            assert_eq!(tab.bar, None);
        }

        #[gpui::test]
        fn should_underline_the_selected_tab_of_a_line_list(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Dark);
            let tab = TabsTrigger::new("a", "A")
                .variant(TabsListVariant::Line)
                .active(true)
                .style(&theme);
            assert_eq!(tab.fill, None);
            assert_eq!(tab.bar, Some(theme.semantic().foreground));
            assert_eq!(tab.border, transparent_black());
        }

        #[gpui::test]
        fn should_mute_a_resting_tab_in_dark(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = theme(cx, Mode::Dark);
            let tab = TabsTrigger::new("a", "A").style(&theme);
            assert_eq!(tab.text, theme.semantic().muted_foreground);
        }
    }

    mod tabs {
        use super::*;

        #[gpui::test]
        fn should_draw_a_36px_list_above_the_panel(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (cx, tabs) = render(cx, settings(Log::default(), TabsOrientation::Horizontal));
            let panel = content(cx);
            assert_eq!(panel.top(), tabs.top() + px(36. + 8.));
        }

        #[gpui::test]
        fn should_switch_tab_on_a_click(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, tabs) = render(cx, settings(log.clone(), TabsOrientation::Horizontal));
            let before = content(cx);
            // Password, the middle tab: Account's ~75px, then Password's ~80px.
            let password = point(tabs.left() + px(115.), tabs.top() + px(18.));
            cx.simulate_click(password, Modifiers::default());
            draw(cx);
            assert_eq!(*log.borrow(), [SharedString::from("password")]);
            assert_ne!(
                content(cx).size.width,
                before.size.width,
                "the password panel shows"
            );
        }

        #[gpui::test]
        fn should_ignore_a_disabled_tab(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, tabs) = render(cx, settings(log.clone(), TabsOrientation::Vertical));
            // Billing, the third of the stacked 31px tabs.
            let billing = point(tabs.left() + px(20.), tabs.top() + px(4. + 31. * 2. + 15.));
            cx.simulate_click(billing, Modifiers::default());
            draw(cx);
            assert!(log.borrow().is_empty());
        }

        #[gpui::test]
        fn should_put_a_vertical_list_beside_the_panel(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (cx, tabs) = render(cx, settings(Log::default(), TabsOrientation::Vertical));
            let panel = content(cx);
            assert_eq!(panel.top(), tabs.top());
            assert!(panel.left() > tabs.left() + px(60.));
        }

        #[gpui::test]
        fn should_select_from_the_keyboard(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, _) = render(cx, settings(log.clone(), TabsOrientation::Horizontal));
            for _ in 0..2 {
                cx.update(|window, cx| window.focus_next(cx));
                draw(cx);
            }
            press(cx, "space");
            draw(cx);
            assert_eq!(*log.borrow(), [SharedString::from("password")]);
        }
    }
}
