// Vendored from gpui-component 0.9.1: crates/ui/src/menu/popup_menu.rs
//
// Modified for core-gpui:
// - The items are the dropdown menu's parts (`MenuEntry`), and those parts draw every row; the
//   menu only says which row is highlighted. Its surface is the dropdown menu's.
// - Gone: action dispatch and the action context, links, custom element items, key-binding
//   hints (the shortcut is a part), icons and the check side, sizes, scrolling and scrolling to
//   the checked item, `rebuild`, and building from `OwnedMenuItem`s.
// - Added: checkbox items toggle and radio items check in place, without closing the menu;
//   the submenu opens at core-react's offsets (`sideOffset={0}`, `alignOffset={-3}`).
// - Actions come from `gpui-base`; `init` is called from `core_gpui::init`.

use gpui::{
    Anchor, App, AppContext as _, Bounds, ClickEvent, Context, DismissEvent, Edges, Entity,
    EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement, KeyBinding,
    MouseDownEvent, ParentElement, Pixels, Point, Render, Role, StatefulInteractiveElement as _,
    StyleRefinement, Styled, WeakEntity, Window, anchored, deferred, div, prelude::FluentBuilder,
    px,
};
use gpui_base::actions::{Cancel, Confirm, SelectDown, SelectLeft, SelectRight, SelectUp};
use gpui_base::{ElementExt as _, StyledExt as _};

use super::menu_item::MenuItemElement;
use super::{
    DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel,
    DropdownMenuRadioItem, DropdownMenuSeparator, DropdownMenuSubTrigger, MenuEntry, surface,
};

const CONTEXT: &str = "PopupMenu";

/// Distance kept between a submenu and the window edge.
const EDGE_PADDING: Pixels = px(4.);

/// `alignOffset={-3}`: a submenu's top sits 3px above its trigger's.
const SUBMENU_ALIGN_OFFSET: Pixels = px(-3.);

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", Confirm { secondary: false }, Some(CONTEXT)),
        KeyBinding::new("space", Confirm { secondary: false }, Some(CONTEXT)),
        KeyBinding::new("escape", Cancel, Some(CONTEXT)),
        KeyBinding::new("up", SelectUp, Some(CONTEXT)),
        KeyBinding::new("down", SelectDown, Some(CONTEXT)),
        KeyBinding::new("left", SelectLeft, Some(CONTEXT)),
        KeyBinding::new("right", SelectRight, Some(CONTEXT)),
    ]);
}

/// An menu item in a popup menu.
pub(crate) enum PopupMenuItem {
    /// A menu separator item.
    Separator(DropdownMenuSeparator),
    /// A non-interactive label item.
    Label(DropdownMenuLabel),
    /// A standard menu item.
    Item(DropdownMenuItem),
    /// An item toggled in place.
    CheckboxItem(DropdownMenuCheckboxItem),
    /// One of the menu's radio items, checked in place.
    RadioItem(DropdownMenuRadioItem),
    /// A submenu item that opens another popup menu.
    Submenu {
        trigger: DropdownMenuSubTrigger,
        menu: Entity<PopupMenu>,
    },
}

impl PopupMenuItem {
    #[inline]
    fn is_clickable(&self) -> bool {
        match self {
            PopupMenuItem::Item(item) => !item.disabled,
            PopupMenuItem::CheckboxItem(item) => !item.disabled,
            PopupMenuItem::RadioItem(item) => !item.disabled,
            PopupMenuItem::Submenu { trigger, .. } => !trigger.disabled,
            PopupMenuItem::Separator(_) | PopupMenuItem::Label(_) => false,
        }
    }

    fn is_checked(&self) -> Option<bool> {
        match self {
            PopupMenuItem::CheckboxItem(item) => Some(item.checked),
            PopupMenuItem::RadioItem(item) => Some(item.checked),
            _ => None,
        }
    }

    fn a11y_label(&self) -> Option<gpui::SharedString> {
        match self {
            PopupMenuItem::Item(item) => Some(item.label.clone()),
            PopupMenuItem::CheckboxItem(item) => Some(item.label.clone()),
            PopupMenuItem::RadioItem(item) => Some(item.label.clone()),
            PopupMenuItem::Submenu { trigger, .. } => Some(trigger.label.clone()),
            PopupMenuItem::Separator(_) | PopupMenuItem::Label(_) => None,
        }
    }
}

pub(crate) struct PopupMenu {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) menu_items: Vec<PopupMenuItem>,
    /// The focus to restore on dismiss.
    pub(crate) previous_focus_handle: Option<FocusHandle>,
    selected_index: Option<usize>,
    bounds: Bounds<Pixels>,
    /// The content's own style refinements (its width, typically).
    style: StyleRefinement,
    /// Whether this is a submenu, drawn as the sub-content.
    sub: bool,

    /// The parent menu of this menu, if this is a submenu
    parent_menu: Option<WeakEntity<Self>>,
    // This will update on render
    submenu_anchor: (Anchor, Pixels),

    /// Paint priority for this menu layer. The top-level menu starts at 1 and
    /// each nested submenu increments it, so deeper levels are always drawn on
    /// top of shallower ones.
    ///
    /// The top-level menu relies on its container (the `Popover`) to
    /// `deferred`-draw it, and each submenu is deferred once in `render_item`
    /// with `priority + 1`. Keeping a single deferred layer per level matters
    /// because GPUI caps nested deferred depth (see `prepaint_deferred_draws`).
    priority: usize,
}

impl PopupMenu {
    fn new(cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            previous_focus_handle: None,
            parent_menu: None,
            menu_items: Vec::new(),
            selected_index: None,
            bounds: Bounds::default(),
            style: StyleRefinement::default(),
            sub: false,
            submenu_anchor: (Anchor::TopLeft, Pixels::ZERO),
            priority: gpui_base::POPUP_PRIORITY,
        }
    }

    /// The menu for `content`, its submenus built with it; focus goes back to what has it
    /// now when the menu is dismissed.
    pub(crate) fn build(
        content: DropdownMenuContent,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        let previous = window.focused(cx);
        Self::build_level(content, false, previous, None, window, cx)
    }

    fn build_level(
        content: DropdownMenuContent,
        sub: bool,
        previous: Option<FocusHandle>,
        parent: Option<(WeakEntity<Self>, usize)>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let mut menu = Self::new(cx);
            menu.style = content.style;
            menu.sub = sub;
            menu.previous_focus_handle = previous.clone();
            if let Some((parent, priority)) = parent {
                menu.parent_menu = Some(parent);
                menu.priority = priority + 1;
            }
            let this = cx.entity().downgrade();
            for entry in content.entries {
                let item = match entry {
                    MenuEntry::Label(label) => PopupMenuItem::Label(label),
                    MenuEntry::Item(item) => PopupMenuItem::Item(item),
                    MenuEntry::CheckboxItem(item) => PopupMenuItem::CheckboxItem(item),
                    MenuEntry::RadioItem(item) => PopupMenuItem::RadioItem(item),
                    MenuEntry::Separator(separator) => PopupMenuItem::Separator(separator),
                    MenuEntry::Sub(trigger, sub_content) => PopupMenuItem::Submenu {
                        trigger,
                        menu: Self::build_level(
                            sub_content.content,
                            true,
                            previous.clone(),
                            Some((this.clone(), menu.priority)),
                            window,
                            cx,
                        ),
                    },
                };
                menu.menu_items.push(item);
            }
            menu
        })
    }

    pub(crate) fn active_submenu(&self) -> Option<Entity<PopupMenu>> {
        if let Some(ix) = self.selected_index {
            if let Some(item) = self.menu_items.get(ix) {
                return match item {
                    PopupMenuItem::Submenu { menu, .. } => Some(menu.clone()),
                    _ => None,
                };
            }
        }

        None
    }

    fn clickable_menu_items(&self) -> impl Iterator<Item = (usize, &PopupMenuItem)> {
        self.menu_items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.is_clickable())
    }

    fn on_click(
        &mut self,
        ix: usize,
        event: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        window.prevent_default();
        self.selected_index = Some(ix);
        self.activate(event, window, cx);
    }

    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        self.activate(&ClickEvent::default(), window, cx);
    }

    fn activate(&mut self, event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.selected_index else {
            return;
        };
        if !self
            .menu_items
            .get(index)
            .is_some_and(PopupMenuItem::is_clickable)
        {
            return;
        }
        match &mut self.menu_items[index] {
            PopupMenuItem::Item(item) => {
                if let Some(handler) = item.on_click.clone() {
                    handler(event, window, cx);
                }
                self.dismiss(&Cancel, window, cx)
            }
            PopupMenuItem::CheckboxItem(item) => {
                item.checked = !item.checked;
                let checked = item.checked;
                if let Some(handler) = item.on_checked_change.clone() {
                    handler(checked, window, cx);
                }
                cx.notify();
            }
            PopupMenuItem::RadioItem(item) => {
                let handler = item.on_select.clone();
                for (ix, other) in self.menu_items.iter_mut().enumerate() {
                    if let PopupMenuItem::RadioItem(other) = other {
                        other.checked = ix == index;
                    }
                }
                if let Some(handler) = handler {
                    handler(event, window, cx);
                }
                cx.notify();
            }
            PopupMenuItem::Submenu { .. } => {
                self._select_submenu(window, cx);
            }
            PopupMenuItem::Separator(_) | PopupMenuItem::Label(_) => {}
        }
    }

    fn set_selected_index(&mut self, ix: usize, cx: &mut Context<Self>) {
        if self.selected_index != Some(ix) {
            self.selected_index = Some(ix);
            cx.notify();
        }
    }

    fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        let ix = self.selected_index.unwrap_or(0);

        if let Some((prev_ix, _)) = self
            .menu_items
            .iter()
            .enumerate()
            .rev()
            .find(|(i, item)| *i < ix && item.is_clickable())
        {
            self.set_selected_index(prev_ix, cx);
            return;
        }

        let last_clickable_ix = self.clickable_menu_items().last().map(|(ix, _)| ix);
        if let Some(ix) = last_clickable_ix {
            self.set_selected_index(ix, cx);
        }
    }

    fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        let Some(ix) = self.selected_index else {
            let first_clickable_ix = self.clickable_menu_items().next().map(|(ix, _)| ix);
            if let Some(ix) = first_clickable_ix {
                self.set_selected_index(ix, cx);
            }
            return;
        };

        if let Some((next_ix, _)) = self
            .menu_items
            .iter()
            .enumerate()
            .find(|(i, item)| *i > ix && item.is_clickable())
        {
            self.set_selected_index(next_ix, cx);
            return;
        }

        let first_clickable_ix = self.clickable_menu_items().next().map(|(ix, _)| ix);
        if let Some(ix) = first_clickable_ix {
            self.set_selected_index(ix, cx);
        }
    }

    fn select_left(&mut self, _: &SelectLeft, window: &mut Window, cx: &mut Context<Self>) {
        let handled = if matches!(self.submenu_anchor.0, Anchor::TopLeft | Anchor::BottomLeft) {
            self._unselect_submenu(window, cx)
        } else {
            self._select_submenu(window, cx)
        };

        if self.parent_opens_right(cx) {
            self._focus_parent_menu(window, cx);
        }

        if handled {
            return;
        }

        if self.parent_menu.is_none() {
            cx.propagate();
        }
    }

    fn select_right(&mut self, _: &SelectRight, window: &mut Window, cx: &mut Context<Self>) {
        let handled = if matches!(self.submenu_anchor.0, Anchor::TopLeft | Anchor::BottomLeft) {
            self._select_submenu(window, cx)
        } else {
            self._unselect_submenu(window, cx)
        };

        if !self.parent_opens_right(cx) {
            self._focus_parent_menu(window, cx);
        }

        if handled {
            return;
        }

        if self.parent_menu.is_none() {
            cx.propagate();
        }
    }

    fn _select_submenu(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if let Some(active_submenu) = self.active_submenu() {
            // Focus the submenu, so that can be handle the action.
            active_submenu.update(cx, |view, cx| {
                let first_clickable_ix = view.clickable_menu_items().next().map(|(ix, _)| ix);
                if let Some(ix) = first_clickable_ix {
                    view.set_selected_index(ix, cx);
                } else {
                    view.selected_index = None;
                    cx.notify();
                }
                view.focus_handle.focus(window, cx);
            });
            cx.notify();
            return true;
        }

        false
    }

    fn _unselect_submenu(&mut self, _: &mut Window, cx: &mut Context<Self>) -> bool {
        if let Some(active_submenu) = self.active_submenu() {
            active_submenu.update(cx, |view, cx| {
                view.selected_index = None;
                cx.notify();
            });
            return true;
        }

        false
    }

    fn _focus_parent_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(parent) = self.parent_menu.as_ref() else {
            return;
        };
        let Some(parent) = parent.upgrade() else {
            return;
        };

        self.selected_index = None;
        parent.update(cx, |view, cx| {
            view.focus_handle.focus(window, cx);
            cx.notify();
        });
    }

    /// Whether this submenu opened to the right of its parent (the left arrow then goes back).
    fn parent_opens_right(&self, cx: &App) -> bool {
        let Some(parent) = self.parent_menu.as_ref().and_then(WeakEntity::upgrade) else {
            return true;
        };
        matches!(
            parent.read(cx).submenu_anchor.0,
            Anchor::TopLeft | Anchor::BottomLeft
        )
    }

    /// Dismiss the menu and the entire parent chain.
    ///
    /// The submenu is closed together with its parent, same as macOS menus.
    fn dismiss(&mut self, _: &Cancel, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_index = None;
        cx.emit(DismissEvent);

        // Focus back to the previous focused handle, unless the item's click
        // handler has already moved focus elsewhere (e.g. opened a dialog and
        // focused its input) -- stealing focus back would break that.
        let focus_moved_away =
            window.focused(cx).is_some() && !self.focus_handle.contains_focused(window, cx);
        if !focus_moved_away {
            if let Some(handle) = self.previous_focus_handle.as_ref() {
                window.focus(handle, cx);
            }
        }

        let Some(parent_menu) = self.parent_menu.clone() else {
            return;
        };

        // Dismiss parent menu, when this menu is dismissed
        _ = parent_menu.update(cx, |view, cx| {
            view.dismiss(&Cancel, window, cx);
        });
    }

    fn handle_dismiss(
        &mut self,
        position: &Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Do not dismiss, if click inside the parent menu
        if let Some(parent) = self.parent_menu.as_ref() {
            if let Some(parent) = parent.upgrade() {
                if parent.read(cx).bounds.contains(position) {
                    return;
                }
            }
        }

        // Do not dismiss, if there have an active submenu, the click may be
        // inside the submenu, let the submenu to handle it.
        //
        // Otherwise the submenu will be dismissed before its item's `on_click`.
        if self.active_submenu().is_some() {
            return;
        }

        self.dismiss(&Cancel, window, cx);
    }

    fn on_mouse_down_out(
        &mut self,
        e: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.handle_dismiss(&e.position, window, cx);
    }

    /// Calculate the anchor corner and left offset for child submenu
    fn update_submenu_menu_anchor(&mut self, window: &Window, cx: &App) {
        let bounds = self.bounds;
        // Open the submenu on the left only when, placed at this menu's right
        // edge, it would run past the window's. Its width is the one it was
        // last drawn at, or this menu's own until it has been drawn.
        let submenu_width = self
            .selected_index
            .and_then(|ix| match self.menu_items.get(ix) {
                Some(PopupMenuItem::Submenu { menu, .. }) => Some(menu.read(cx).bounds.size.width),
                _ => None,
            })
            .filter(|width| *width > Pixels::ZERO)
            .unwrap_or(bounds.size.width);
        // A row sits inside the menu's 4px padding; `sideOffset={0}` puts the submenu
        // against the trigger row's right edge.
        let padding = brrritscold_core_gpui_tokens::primitive::space::S1;
        let (anchor, left) = if bounds.right() + submenu_width > window.viewport_size().width {
            (Anchor::TopRight, Pixels::ZERO)
        } else {
            (Anchor::TopLeft, bounds.size.width - padding * 2.)
        };

        let is_bottom_pos = bounds.origin.y + bounds.size.height > window.bounds().size.height;
        self.submenu_anchor = if is_bottom_pos {
            (anchor.other_side_along(gpui::Axis::Vertical), left)
        } else {
            (anchor, left)
        };
    }

    fn render_item(
        &self,
        ix: usize,
        item: &PopupMenuItem,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let selected = self.selected_index == Some(ix);
        let is_submenu = matches!(item, PopupMenuItem::Submenu { .. });

        let this =
            MenuItemElement::new(ix)
                .selected(selected)
                .on_hover(cx.listener(move |this, hovered, window, cx| {
                    if *hovered {
                        // Pointing elsewhere closes the open submenu; take back
                        // the focus the keyboard may have moved into it.
                        if this.selected_index != Some(ix)
                            && this.active_submenu().is_some_and(|submenu| {
                                submenu.focus_handle(cx).contains_focused(window, cx)
                            })
                        {
                            this.focus_handle.focus(window, cx);
                        }
                        this.selected_index = Some(ix);
                    } else if !is_submenu
                    && this.selected_index == Some(ix)
                    // A key press ends hover under a still pointer; keep the
                    // highlight so the next arrow key moves on from this item.
                    && !window.last_input_was_keyboard()
                    {
                        this.selected_index = None;
                    }

                    cx.notify();
                }))
                .when_some(item.a11y_label(), |this, label| this.aria_label(label))
                .checked(item.is_checked())
                .disabled(!item.is_clickable())
                .when(item.is_clickable(), |this| {
                    this.on_click(cx.listener(move |this, event, window, cx| {
                        this.on_click(ix, event, window, cx)
                    }))
                });

        match item {
            PopupMenuItem::Separator(separator) => separator.clone().into_any_element(),
            PopupMenuItem::Label(label) => label.clone().into_any_element(),
            PopupMenuItem::Item(item) => this
                .child(item.clone().highlighted(selected))
                .into_any_element(),
            PopupMenuItem::CheckboxItem(item) => this
                .child(item.clone().highlighted(selected))
                .into_any_element(),
            PopupMenuItem::RadioItem(item) => this
                .child(item.clone().highlighted(selected))
                .into_any_element(),
            PopupMenuItem::Submenu { trigger, menu } => this
                .child(trigger.clone().highlighted(selected))
                .when(selected, |this| {
                    this.child({
                        let (anchor, left) = self.submenu_anchor;
                        let is_bottom_pos =
                            matches!(anchor, Anchor::BottomLeft | Anchor::BottomRight);
                        deferred(
                            anchored()
                                .anchor(anchor)
                                .child(
                                    div()
                                        .id("submenu")
                                        .occlude()
                                        .when(is_bottom_pos, |this| this.bottom_0())
                                        .when(!is_bottom_pos, |this| this.top(SUBMENU_ALIGN_OFFSET))
                                        .left(left)
                                        .child(menu.clone()),
                                )
                                .snap_to_window_with_margin(Edges::all(EDGE_PADDING)),
                        )
                        .with_priority(self.priority + 1)
                    })
                })
                .into_any_element(),
        }
    }
}

impl EventEmitter<DismissEvent> for PopupMenu {}
impl Focusable for PopupMenu {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PopupMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.update_submenu_menu_anchor(window, cx);

        let view = cx.entity().clone();
        let items: Vec<gpui::AnyElement> = self
            .menu_items
            .iter()
            .enumerate()
            .map(|(ix, item)| self.render_item(ix, item, cx))
            .collect();

        let slot = if self.sub {
            "dropdown-menu-sub-content"
        } else {
            "dropdown-menu-content"
        };
        surface(self.sub, cx)
            // core-react's `data-slot`, for tests to find it by.
            .debug_selector(move || slot.into())
            .id("popup-menu")
            .role(Role::Menu)
            .key_context(CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_up))
            .on_action(cx.listener(Self::select_down))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::dismiss))
            .on_mouse_down_out(cx.listener(Self::on_mouse_down_out))
            .relative()
            .occlude()
            .children(items)
            .on_prepaint(move |bounds, _, cx| {
                view.update(cx, |menu, cx| {
                    let width_changed = menu.bounds.size.width != bounds.size.width;
                    menu.bounds = bounds;
                    if width_changed {
                        if let Some(parent) = menu
                            .parent_menu
                            .as_ref()
                            .and_then(|parent| parent.upgrade())
                        {
                            parent.update(cx, |_, cx| cx.notify());
                        }
                    }
                });
            })
            .refine_style(&self.style)
    }
}
