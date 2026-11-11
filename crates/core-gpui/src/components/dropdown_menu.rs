//! Dropdown Menu: core-react's `dropdown-menu.tsx`. `gpui-base` has no menu, so its behaviour
//! (the highlight, arrow keys, submenus and their focus, dismissing the whole chain) is
//! gpui-component's popup menu, vendored into `popup_menu` and `menu_item` and drawn with
//! the parts here; [`DropdownMenu`] opens it under its trigger through `gpui-base`'s
//! [`gpui_base::Popover`].
//!
//! Every part renders on its own, the way the gallery and captures show them:
//! [`DropdownMenuContent`] lists its items in order, and the open menu draws the same parts,
//! highlighting the one under the pointer or the arrow keys.
//!
//! The shortcut's `tracking-widest` has no GPUI target: it is a row of one element per glyph,
//! spaced by the `letter-spacing.widest` token.
//!
//! Not mirrored: the open and close animations, `max-h-(--available-height)` (the menu grows to
//! fit), the content's `w-(--anchor-width)` (it is as wide as its widest item unless the caller
//! sets a width) and the `inset` prop, which no Penpot variant property names.

mod menu_item;
pub(crate) mod popup_menu;

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::{
    font_size, font_weight, letter_spacing, opacity, shadow, space,
};
use gpui::{
    Anchor, App, ClickEvent, Context, DismissEvent, ElementId, Entity, Focusable as _, Hsla,
    IntoElement, ParentElement as _, RenderOnce, SharedString, StyleRefinement, Styled, Window,
    div, prelude::FluentBuilder as _, px,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use super::button::Button;
use crate::icons::{Icon, IconName};
use crate::theme::{ActiveTheme as _, Theme};
use popup_menu::PopupMenu;

/// Penpot variant property `variant` of `DropdownMenuItem`. Cases are its values in PascalCase;
/// [`Display`] gives the Penpot value back.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum DropdownMenuItemVariant {
    /// `default`
    #[default]
    Default,
    /// `destructive`: the item, and its icon, in `destructive`.
    Destructive,
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type CheckedHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A dropdown menu: a trigger button that opens [`DropdownMenuContent`] below it.
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     Button, ButtonVariant, DropdownMenu, DropdownMenuContent, DropdownMenuItem,
///     DropdownMenuItemVariant, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuShortcut,
/// };
///
/// let account = DropdownMenu::new("account")
///     .trigger(Button::new("open").variant(ButtonVariant::Outline).label("Open menu"))
///     .content(
///         DropdownMenuContent::new()
///             .label(DropdownMenuLabel::new("My account"))
///             .item(
///                 DropdownMenuItem::new("Profile")
///                     .shortcut(DropdownMenuShortcut::new("⇧⌘P"))
///                     .on_click(|_, _, _| println!("profile")),
///             )
///             .separator(DropdownMenuSeparator::new())
///             .item(
///                 DropdownMenuItem::new("Delete account")
///                     .variant(DropdownMenuItemVariant::Destructive),
///             ),
///     );
/// ```
#[derive(IntoElement)]
pub struct DropdownMenu {
    id: ElementId,
    trigger: Option<Button>,
    content: DropdownMenuContent,
    anchor: Anchor,
}

impl DropdownMenu {
    /// A closed menu. `id` keys its state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: DropdownMenuContent::new(),
            anchor: Anchor::TopLeft,
        }
    }

    /// The button that opens it (core-react's `DropdownMenuTrigger`).
    pub fn trigger(mut self, trigger: Button) -> Self {
        self.trigger = Some(trigger);
        self
    }

    /// The menu shown while open.
    pub fn content(mut self, content: DropdownMenuContent) -> Self {
        self.content = content;
        self
    }

    /// Lines the menu's right edge up with the trigger's (`align="end"`); by default their left
    /// edges line up (`align="start"`). Not a Penpot property, so it stays inside the crate.
    pub(crate) fn align_end(mut self) -> Self {
        self.anchor = Anchor::TopRight;
        self
    }
}

/// The open menu, kept between frames; rebuilt from the content each time it opens.
#[derive(Default)]
struct DropdownMenuState {
    menu: Option<Entity<PopupMenu>>,
}

impl RenderOnce for DropdownMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state =
            window.use_keyed_state(self.id.clone(), cx, |_, _| DropdownMenuState::default());
        let content = self.content;
        let trigger = self.trigger;
        // side="bottom" align="start" (or "end") sideOffset={4}
        gpui_base::Popover::new(self.id)
            .anchor(self.anchor)
            .offset(space::S1)
            // The menu dismisses itself on a press outside, so a press on a submenu isn't one.
            .overlay_closable(false)
            .when_some(trigger, |p, trigger| {
                p.trigger_with(move |_, _, _| trigger.into_any_element())
            })
            .content(
                move |_, window, cx: &mut Context<gpui_base::PopoverState>| {
                    if let Some(menu) = state.read(cx).menu.clone() {
                        return menu;
                    }
                    let menu = PopupMenu::build(content, window, cx);
                    state.update(cx, |state, _| state.menu = Some(menu.clone()));
                    menu.focus_handle(cx).focus(window, cx);
                    // The menu's dismissal closes the popover and drops the menu, so it's rebuilt
                    // fresh on the next open. Weak handles: the state owns the menu.
                    let popover = cx.entity().downgrade();
                    let menu_state = state.downgrade();
                    window
                        .subscribe(&menu, cx, move |_, _: &DismissEvent, window, cx| {
                            if let Some(popover) = popover.upgrade() {
                                popover.update(cx, |popover, cx| popover.dismiss(window, cx));
                            }
                            _ = menu_state.update(cx, |state, _| state.menu = None);
                        })
                        .detach();
                    menu
                },
            )
    }
}

/// A row of a menu, in the order its content lists them.
#[derive(Clone)]
pub(crate) enum MenuEntry {
    Label(DropdownMenuLabel),
    Item(DropdownMenuItem),
    CheckboxItem(DropdownMenuCheckboxItem),
    RadioItem(DropdownMenuRadioItem),
    Separator(DropdownMenuSeparator),
    Sub(DropdownMenuSubTrigger, Box<DropdownMenuSubContent>),
}

/// The surface every menu draws on: core-react's content and sub-content classes.
/// `rounded-md bg-popover p-1 text-popover-foreground ring-1 ring-foreground/10`, with
/// `shadow-md` (content) or `shadow-lg` (sub-content).
pub(crate) fn surface(sub: bool, cx: &App) -> gpui::Div {
    let s = cx.theme().semantic();
    let mut shadows = vec![super::ring(1., s.foreground.opacity(0.1))];
    shadows.extend_from_slice(if sub { shadow::LG } else { shadow::MD });
    div()
        .flex()
        .flex_col()
        // min-w-32; the sub-content's min-w-24
        .min_w(if sub { space::S24 } else { space::S32 })
        .rounded(s.radius_md)
        .bg(s.popover)
        .p(space::S1)
        .text_color(s.popover_foreground)
        .shadow(shadows)
}

/// The dropdown menu's `Content` part: its labels, items and separators, in order.
#[derive(IntoElement, Clone)]
pub struct DropdownMenuContent {
    pub(crate) entries: Vec<MenuEntry>,
    pub(crate) style: StyleRefinement,
}

impl Default for DropdownMenuContent {
    fn default() -> Self {
        Self::new()
    }
}

impl DropdownMenuContent {
    /// An empty menu.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// A `Label` part: a heading.
    pub fn label(mut self, label: DropdownMenuLabel) -> Self {
        self.entries.push(MenuEntry::Label(label));
        self
    }

    /// An `Item` part.
    pub fn item(mut self, item: DropdownMenuItem) -> Self {
        self.entries.push(MenuEntry::Item(item));
        self
    }

    /// A `CheckboxItem` part.
    pub fn checkbox_item(mut self, item: DropdownMenuCheckboxItem) -> Self {
        self.entries.push(MenuEntry::CheckboxItem(item));
        self
    }

    /// A `RadioItem` part; the radio items of one menu check one at a time.
    pub fn radio_item(mut self, item: DropdownMenuRadioItem) -> Self {
        self.entries.push(MenuEntry::RadioItem(item));
        self
    }

    /// A `Separator` part.
    pub fn separator(mut self, separator: DropdownMenuSeparator) -> Self {
        self.entries.push(MenuEntry::Separator(separator));
        self
    }

    /// A submenu: its `SubTrigger` part, which opens its `SubContent` beside it.
    pub fn sub(mut self, trigger: DropdownMenuSubTrigger, content: DropdownMenuSubContent) -> Self {
        self.entries
            .push(MenuEntry::Sub(trigger, Box::new(content)));
        self
    }
}

impl RenderOnce for DropdownMenuContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        surface(false, cx)
            .children(self.entries.into_iter().map(MenuEntry::into_any_element))
            .refine_style(&self.style)
    }
}

impl MenuEntry {
    fn into_any_element(self) -> gpui::AnyElement {
        match self {
            MenuEntry::Label(label) => label.into_any_element(),
            MenuEntry::Item(item) => item.into_any_element(),
            MenuEntry::CheckboxItem(item) => item.into_any_element(),
            MenuEntry::RadioItem(item) => item.into_any_element(),
            MenuEntry::Separator(separator) => separator.into_any_element(),
            MenuEntry::Sub(trigger, _) => trigger.into_any_element(),
        }
    }
}

/// The dropdown menu's `SubContent` part: a submenu's items, beside its trigger.
#[derive(IntoElement, Clone)]
pub struct DropdownMenuSubContent {
    pub(crate) content: DropdownMenuContent,
}

impl Default for DropdownMenuSubContent {
    fn default() -> Self {
        Self::new()
    }
}

impl DropdownMenuSubContent {
    /// An empty submenu.
    pub fn new() -> Self {
        Self {
            content: DropdownMenuContent::new(),
        }
    }

    /// An `Item` part.
    pub fn item(mut self, item: DropdownMenuItem) -> Self {
        self.content = self.content.item(item);
        self
    }

    /// A `Separator` part.
    pub fn separator(mut self, separator: DropdownMenuSeparator) -> Self {
        self.content = self.content.separator(separator);
        self
    }
}

impl RenderOnce for DropdownMenuSubContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        surface(true, cx)
            .children(
                self.content
                    .entries
                    .into_iter()
                    .map(MenuEntry::into_any_element),
            )
            .refine_style(&self.content.style)
    }
}

/// An item's colours, resolved for its variant and whether it's highlighted.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RowStyle {
    fill: Option<Hsla>,
    text: Option<Hsla>,
}

impl DropdownMenuItemVariant {
    fn style(self, highlighted: bool, theme: &Theme) -> RowStyle {
        let s = theme.semantic();
        match (self, highlighted) {
            (DropdownMenuItemVariant::Default, false) => RowStyle {
                fill: None,
                text: None,
            },
            // focus:bg-accent focus:text-accent-foreground
            (DropdownMenuItemVariant::Default, true) => RowStyle {
                fill: Some(s.accent),
                text: Some(s.accent_foreground),
            },
            // data-[variant=destructive]:text-destructive
            (DropdownMenuItemVariant::Destructive, false) => RowStyle {
                fill: None,
                text: Some(s.destructive),
            },
            // data-[variant=destructive]:focus:bg-destructive/10
            // dark:data-[variant=destructive]:focus:bg-destructive/20
            (DropdownMenuItemVariant::Destructive, true) => RowStyle {
                fill: Some(s.destructive.opacity(theme.mode().pick(0.1, 0.2))),
                text: Some(s.destructive),
            },
        }
    }
}

/// A row: `relative flex items-center gap-2 rounded-sm py-1.5 text-sm`, `px-2` unless a row
/// with a trailing indicator sets its own padding. `data-disabled:opacity-50`.
fn row(style: RowStyle, disabled: bool, cx: &App) -> gpui::Div {
    let s = cx.theme().semantic();
    div()
        .relative()
        .flex()
        .items_center()
        .gap(space::S2)
        .rounded(s.radius_sm)
        .px(space::S2)
        .py(space::S1_5)
        .text_size(font_size::SM)
        .whitespace_nowrap()
        .when_some(style.fill, |d, fill| d.bg(fill))
        .when_some(style.text, |d, text| d.text_color(text))
        .when(disabled, |d| d.opacity(opacity::O50))
}

/// A trailing check (`pointer-events-none absolute right-2 flex items-center justify-center`).
fn check_indicator() -> gpui::Div {
    div()
        .absolute()
        .right(space::S2)
        .flex()
        .items_center()
        .justify_center()
        .child(Icon::new(IconName::Check).size(space::S4))
}

/// The dropdown menu's `Item` part: an icon, its label and a shortcut.
#[derive(IntoElement, Clone)]
pub struct DropdownMenuItem {
    pub(crate) label: SharedString,
    variant: DropdownMenuItemVariant,
    icon: Option<IconName>,
    shortcut: Option<DropdownMenuShortcut>,
    pub(crate) disabled: bool,
    pub(crate) highlighted: bool,
    pub(crate) on_click: Option<ClickHandler>,
}

impl DropdownMenuItem {
    /// An item reading `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: DropdownMenuItemVariant::default(),
            icon: None,
            shortcut: None,
            disabled: false,
            highlighted: false,
            on_click: None,
        }
    }

    /// Penpot property `variant`.
    pub fn variant(mut self, variant: DropdownMenuItemVariant) -> Self {
        self.variant = variant;
        self
    }

    /// An icon before the label, `size-4`.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// The `Shortcut` part, at the right.
    pub fn shortcut(mut self, shortcut: DropdownMenuShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Dims the item and keeps it from being chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Runs when the item is chosen; the menu then closes.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub(crate) fn highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted && !self.disabled;
        self
    }
}

impl RenderOnce for DropdownMenuItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.variant.style(self.highlighted, cx.theme());
        let highlighted = self.highlighted;
        row(style, self.disabled, cx)
            // [&_svg:not([class*='size-'])]:size-4; destructive *:[svg]:text-destructive
            .when_some(self.icon, |d, icon| {
                d.child(Icon::new(icon).size(space::S4))
            })
            .child(self.label)
            .children(
                self.shortcut
                    .map(|shortcut| shortcut.highlighted(highlighted)),
            )
    }
}

/// The dropdown menu's `CheckboxItem` part: an item with a check at the right while checked.
#[derive(IntoElement, Clone)]
pub struct DropdownMenuCheckboxItem {
    pub(crate) label: SharedString,
    pub(crate) checked: bool,
    pub(crate) disabled: bool,
    pub(crate) highlighted: bool,
    pub(crate) on_checked_change: Option<CheckedHandler>,
}

impl DropdownMenuCheckboxItem {
    /// An unchecked item reading `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            checked: false,
            disabled: false,
            highlighted: false,
            on_checked_change: None,
        }
    }

    /// Penpot property `checked`.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Dims the item and keeps it from being toggled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Runs when the item is toggled, with its new state; the menu stays open.
    pub fn on_checked_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_checked_change = Some(Rc::new(handler));
        self
    }

    pub(crate) fn highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted && !self.disabled;
        self
    }
}

impl RenderOnce for DropdownMenuCheckboxItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = DropdownMenuItemVariant::Default.style(self.highlighted, cx.theme());
        // py-1.5 pr-8 pl-2
        row(style, self.disabled, cx)
            .pr(space::S8)
            .child(self.label)
            .when(self.checked, |d| d.child(check_indicator()))
    }
}

/// The dropdown menu's `RadioItem` part: one of a set of choices, checked when chosen.
#[derive(IntoElement, Clone)]
pub struct DropdownMenuRadioItem {
    pub(crate) label: SharedString,
    icon: Option<IconName>,
    pub(crate) close_on_click: bool,
    pub(crate) checked: bool,
    pub(crate) disabled: bool,
    pub(crate) highlighted: bool,
    pub(crate) on_select: Option<ClickHandler>,
}

impl DropdownMenuRadioItem {
    /// An unchecked choice reading `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            close_on_click: false,
            checked: false,
            disabled: false,
            highlighted: false,
            on_select: None,
        }
    }

    /// An icon before the label, `size-4`.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Closes the menu once the item is chosen (Base UI's `closeOnClick`); by default the menu
    /// stays open.
    pub fn close_on_click(mut self, close: bool) -> Self {
        self.close_on_click = close;
        self
    }

    /// Whether it's the chosen one (its `Checked` part shows).
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Dims the item and keeps it from being chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Runs when it's chosen.
    pub fn on_select(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    pub(crate) fn highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted && !self.disabled;
        self
    }
}

impl RenderOnce for DropdownMenuRadioItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = DropdownMenuItemVariant::Default.style(self.highlighted, cx.theme());
        // py-1.5 pr-8 pl-2; [&_svg:not([class*='size-'])]:size-4
        row(style, self.disabled, cx)
            .pr(space::S8)
            .when_some(self.icon, |d, icon| {
                d.child(Icon::new(icon).size(space::S4))
            })
            .child(self.label)
            .when(self.checked, |d| d.child(check_indicator()))
    }
}

/// The dropdown menu's `SubTrigger` part: an item that opens a submenu, with a chevron.
#[derive(IntoElement, Clone)]
pub struct DropdownMenuSubTrigger {
    pub(crate) label: SharedString,
    pub(crate) disabled: bool,
    pub(crate) highlighted: bool,
}

impl DropdownMenuSubTrigger {
    /// A submenu trigger reading `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            highlighted: false,
        }
    }

    /// Dims it and keeps its submenu closed.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(crate) fn highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted && !self.disabled;
        self
    }
}

impl RenderOnce for DropdownMenuSubTrigger {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // data-popup-open:bg-accent data-popup-open:text-accent-foreground, as focus:
        let style = DropdownMenuItemVariant::Default.style(self.highlighted, cx.theme());
        row(style, self.disabled, cx)
            .child(self.label)
            // ChevronRightIcon ml-auto size-4
            .child(Icon::new(IconName::ChevronRight).size(space::S4).ml_auto())
    }
}

/// The dropdown menu's `Label` part: a muted heading.
#[derive(IntoElement, Clone)]
pub struct DropdownMenuLabel {
    text: SharedString,
}

impl DropdownMenuLabel {
    /// A label reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for DropdownMenuLabel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // px-2 py-1.5 text-xs font-medium text-muted-foreground
        div()
            .px(space::S2)
            .py(space::S1_5)
            .text_size(font_size::XS)
            .font_weight(font_weight::MEDIUM)
            .text_color(cx.theme().semantic().muted_foreground)
            .child(self.text)
    }
}

/// The dropdown menu's `Separator` part: a rule across the menu.
#[derive(IntoElement, Clone, Default)]
pub struct DropdownMenuSeparator {}

impl DropdownMenuSeparator {
    /// A separator.
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderOnce for DropdownMenuSeparator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // -mx-1 my-1 h-px bg-border
        div()
            .mx(-space::S1)
            .my(space::S1)
            .h(px(1.))
            .bg(cx.theme().semantic().border)
    }
}

/// The dropdown menu's `Shortcut` part: an item's key combination, at its right.
#[derive(IntoElement, Clone)]
pub struct DropdownMenuShortcut {
    keys: SharedString,
    highlighted: bool,
}

impl DropdownMenuShortcut {
    /// A shortcut reading `keys` (`⇧⌘P`).
    pub fn new(keys: impl Into<SharedString>) -> Self {
        Self {
            keys: keys.into(),
            highlighted: false,
        }
    }

    fn highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted;
        self
    }
}

impl RenderOnce for DropdownMenuShortcut {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // ml-auto text-xs tracking-widest text-muted-foreground
        // group-focus/dropdown-menu-item:text-accent-foreground. tracking-widest: one element
        // per glyph, the token apart.
        div()
            .ml_auto()
            .flex()
            .gap(letter_spacing::WIDEST)
            .text_size(font_size::XS)
            .text_color(if self.highlighted {
                s.accent_foreground
            } else {
                s.muted_foreground
            })
            .children(
                self.keys
                    .chars()
                    .map(|glyph| div().child(glyph.to_string())),
            )
    }
}

impl_styled!(DropdownMenuContent);

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext, point};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::render;

    /// Room left of and above the trigger.
    const INSET: Pixels = px(100.);

    type Log = Rc<RefCell<Vec<String>>>;

    fn logged(log: &Log, entry: &str) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        let log = log.clone();
        let entry = entry.to_owned();
        move |_, _, _| log.borrow_mut().push(entry.clone())
    }

    /// The story's account menu, plus a checkbox item and a submenu, logging what's chosen.
    fn account(log: Log) -> impl Fn() -> gpui::Div {
        move || {
            let checked = log.clone();
            div().flex().pl(INSET).pt(INSET).child(
                DropdownMenu::new("account")
                    .trigger(Button::new("open").label("Open menu"))
                    .content(
                        DropdownMenuContent::new()
                            .w(px(224.))
                            .label(DropdownMenuLabel::new("My account"))
                            .item(
                                DropdownMenuItem::new("Profile").on_click(logged(&log, "profile")),
                            )
                            .item(
                                DropdownMenuItem::new("Settings")
                                    .on_click(logged(&log, "settings")),
                            )
                            .item(DropdownMenuItem::new("Billing").disabled(true))
                            .item(DropdownMenuItem::new("Delete").on_click(logged(&log, "delete")))
                            .checkbox_item(
                                DropdownMenuCheckboxItem::new("Status bar").on_checked_change(
                                    move |on, _, _| {
                                        checked.borrow_mut().push(format!("status bar {on}"))
                                    },
                                ),
                            )
                            .sub(
                                DropdownMenuSubTrigger::new("Share"),
                                DropdownMenuSubContent::new().item(
                                    DropdownMenuItem::new("Email").on_click(logged(&log, "email")),
                                ),
                            ),
                    ),
            )
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    fn open(cx: &mut TestAppContext) -> (&mut VisualTestContext, Bounds<Pixels>, Log) {
        cx.update(crate::init);
        let log = Log::default();
        let (cx, page) = render(cx, account(log.clone()));
        let trigger = Bounds::new(
            page.origin + point(INSET, INSET),
            gpui::size(page.size.width - INSET, page.size.height - INSET),
        );
        cx.simulate_click(trigger.center(), Modifiers::default());
        draw(cx);
        (cx, trigger, log)
    }

    fn is_open(cx: &mut VisualTestContext) -> bool {
        cx.debug_bounds("dropdown-menu-content").is_some()
    }

    /// Item row `n` (0 = Profile): 4px under the trigger, the menu's 4px padding, the 30px
    /// label, then 33px rows.
    fn row(trigger: Bounds<Pixels>, n: usize) -> gpui::Point<Pixels> {
        let top = trigger.bottom() + px(4. + 4. + 30.) + px(33.) * n as f32;
        point(trigger.left() + px(40.), top + px(16.))
    }

    mod variant {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&str> = DropdownMenuItemVariant::iter().map(Into::into).collect();
            assert_eq!(values, ["default", "destructive"]);
        }
    }

    mod dropdown_menu {
        use super::*;

        #[gpui::test]
        fn should_open_from_its_trigger_and_run_the_clicked_item(cx: &mut TestAppContext) {
            let (cx, trigger, log) = open(cx);
            assert!(is_open(cx));
            cx.simulate_click(row(trigger, 1), Modifiers::default());
            draw(cx);
            assert_eq!(*log.borrow(), ["settings"]);
            assert!(!is_open(cx), "choosing an item closes the menu");
        }

        #[gpui::test]
        fn should_move_with_the_arrow_keys_past_disabled_items(cx: &mut TestAppContext) {
            let (cx, _, log) = open(cx);
            // Profile, Settings, then Delete: Billing is disabled.
            cx.simulate_keystrokes("down down down enter");
            draw(cx);
            assert_eq!(*log.borrow(), ["delete"]);
        }

        #[gpui::test]
        fn should_close_on_escape(cx: &mut TestAppContext) {
            let (cx, _, log) = open(cx);
            cx.simulate_keystrokes("escape");
            draw(cx);
            assert!(!is_open(cx));
            assert!(log.borrow().is_empty());
        }

        #[gpui::test]
        fn should_toggle_a_checkbox_item_and_stay_open(cx: &mut TestAppContext) {
            let (cx, trigger, log) = open(cx);
            cx.simulate_click(row(trigger, 4), Modifiers::default());
            draw(cx);
            cx.simulate_click(row(trigger, 4), Modifiers::default());
            draw(cx);
            assert_eq!(*log.borrow(), ["status bar true", "status bar false"]);
            assert!(is_open(cx));
        }

        #[gpui::test]
        fn should_open_a_submenu_and_run_its_item(cx: &mut TestAppContext) {
            let (cx, trigger, log) = open(cx);
            cx.simulate_mouse_move(row(trigger, 5), None, Modifiers::default());
            draw(cx);
            let submenu = cx
                .debug_bounds("dropdown-menu-sub-content")
                .expect("hovering the trigger opens the submenu");
            assert!(submenu.left() >= row(trigger, 5).x, "it opens to the right");
            cx.simulate_click(submenu.center(), Modifiers::default());
            draw(cx);
            assert_eq!(*log.borrow(), ["email"]);
            assert!(!is_open(cx), "the whole chain closes");
        }
    }

    mod shortcut {
        use super::*;

        #[gpui::test]
        fn should_space_its_glyphs_by_the_widest_letter_spacing(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, one) = render(cx, || DropdownMenuShortcut::new("P"));
            let (_, three) = render(cx, || DropdownMenuShortcut::new("PPP"));
            let gaps = f32::from(three.size.width - one.size.width * 3.);
            let expected = f32::from(letter_spacing::WIDEST * 2.);
            // Layout snaps each glyph to whole pixels.
            assert!((gaps - expected).abs() < 0.5, "{gaps} vs {expected}");
        }
    }
}
