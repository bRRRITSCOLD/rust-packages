//! Live overlays for the gallery: a trigger that really opens what an entry shows inline.
//! Registry cells render surfaces only (a capture is a settled frame); opening, focus and
//! dismissal are seen here, by using them.

use brrritscold_core_gpui::{
    AlertDialog, AlertDialogContentSize, Button, ButtonSize, ButtonVariant, Dialog, DropdownMenu,
    DropdownMenuCheckboxItem, DropdownMenuRadioItem, DropdownMenuSubTrigger, IconName, Popover,
    Select, SelectTrigger, Sheet, SheetContentSide, Toast, ToastAction, ToastDescription,
    ToastExt as _, ToastTitle, ToastType, Tooltip, TooltipContent, TooltipContentSide,
};
use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{AnyElement, ElementId, IntoElement as _, ParentElement as _, Styled as _, div};
use strum::IntoEnumIterator as _;

use super::{alert_dialog, dialog, dropdown_menu, popover, select, sheet};

fn outline(id: impl Into<ElementId>, label: &'static str) -> Button {
    Button::new(id).variant(ButtonVariant::Outline).label(label)
}

fn row(children: impl IntoIterator<Item = AnyElement>) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(space::S4)
        .children(children)
        .into_any_element()
}

/// The live overlay for the registry entry `name`, if it is an overlay's surface.
pub fn live(name: &str) -> Option<AnyElement> {
    let element = match name {
        "DialogContent" => Dialog::new("live-dialog")
            .trigger(outline("live-dialog-open", "Edit profile"))
            .content(dialog::content(&"live-dialog-content".into()))
            .into_any_element(),
        "AlertDialogContent" => row(AlertDialogContentSize::iter().map(|size| {
            AlertDialog::new(("live-alert-dialog", size as usize))
                .trigger(outline(
                    ("live-alert-dialog-open", size as usize),
                    "Delete account",
                ))
                .content(alert_dialog::content(size))
                .into_any_element()
        })),
        "SheetContent" => row(SheetContentSide::iter().map(|side| {
            let label: &'static str = side.into();
            Sheet::new(("live-sheet", side as usize))
                .trigger(outline(("live-sheet-open", side as usize), label))
                .content(sheet::content(
                    &("live-sheet-content", side as usize).into(),
                    side,
                ))
                .into_any_element()
        })),
        "PopoverContent" => Popover::new("live-popover")
            .trigger(outline("live-popover-open", "Dimensions"))
            .content(popover::content(&"live-popover-content".into()))
            .into_any_element(),
        "TooltipContent" => row(TooltipContentSide::iter().map(|side| {
            Tooltip::new(("live-tooltip", side as usize))
                .trigger(
                    Button::new(("live-tooltip-trigger", side as usize))
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .icon(IconName::Plus),
                )
                .content(TooltipContent::new("Add to library").side(side))
                .into_any_element()
        })),
        "SelectTrigger" | "SelectContent" => Select::new("live-select")
            .trigger(SelectTrigger::new().placeholder("Pick one"))
            .content(select::content())
            .into_any_element(),
        "DropdownMenuContent" => DropdownMenu::new("live-dropdown-menu")
            .trigger(outline("live-dropdown-menu-open", "Open menu"))
            .content(
                dropdown_menu::content()
                    .separator(Default::default())
                    .checkbox_item(DropdownMenuCheckboxItem::new("Status bar").checked(true))
                    .radio_item(DropdownMenuRadioItem::new("Top"))
                    .radio_item(DropdownMenuRadioItem::new("Bottom").checked(true))
                    .sub(
                        DropdownMenuSubTrigger::new("Share"),
                        dropdown_menu::sub_content(),
                    ),
            )
            .into_any_element(),
        "Toast" => row(ToastType::iter().map(|kind| {
            let label: &'static str = kind.into();
            Button::new(("live-toast", kind as usize))
                .variant(ButtonVariant::Outline)
                .label(label)
                .on_click(move |_, window, cx| {
                    window.push_toast(
                        Toast::new()
                            .toast_type(kind)
                            .title(ToastTitle::new("Message archived"))
                            .description(ToastDescription::new("It has moved to the archive."))
                            .action(ToastAction::new("Undo")),
                        cx,
                    )
                })
                .into_any_element()
        })),
        _ => return None,
    };
    Some(element)
}

#[cfg(test)]
mod tests {
    use brrritscold_core_gpui::Root;
    use gpui::{
        AppContext as _, Context, IntoElement, Modifiers, Render, TestAppContext,
        VisualTestContext, Window, point, px,
    };

    use super::*;

    struct Page(&'static str);

    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().p_4().child(live(self.0).expect("a live overlay"))
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    /// Opens the page of `name`'s live overlay and presses (or, for a tooltip, points at) its
    /// first trigger, 16px in from the corner.
    fn open<'a>(cx: &'a mut TestAppContext, name: &'static str) -> &'a mut VisualTestContext {
        cx.update(brrritscold_core_gpui::init);
        let window = cx.add_window(move |window, cx| {
            let page = cx.new(|_| Page(name));
            Root::new(page, window, cx)
        });
        let cx = VisualTestContext::from_window(*window, cx).into_mut();
        draw(cx);
        let trigger = point(px(24.), px(24.));
        if name == "TooltipContent" {
            cx.simulate_mouse_move(trigger, None, Modifiers::default());
        } else {
            cx.simulate_click(trigger, Modifiers::default());
        }
        draw(cx);
        cx
    }

    mod live {
        use super::*;

        #[gpui::test]
        fn should_open_and_close_every_modal_without_panicking(cx: &mut TestAppContext) {
            for name in ["DialogContent", "AlertDialogContent", "SheetContent"] {
                let cx = open(cx, name);
                cx.simulate_keystrokes("escape");
                draw(cx);
            }
        }

        #[gpui::test]
        fn should_open_the_popover_and_select(cx: &mut TestAppContext) {
            for name in ["PopoverContent", "SelectContent"] {
                let cx = open(cx, name);
                cx.simulate_keystrokes("escape");
                draw(cx);
            }
        }

        #[gpui::test]
        fn should_show_the_tooltip(cx: &mut TestAppContext) {
            let cx = open(cx, "TooltipContent");
            assert!(cx.debug_bounds("tooltip-content").is_some());
        }

        #[gpui::test]
        fn should_open_the_dropdown_menu_and_its_submenu(cx: &mut TestAppContext) {
            let cx = open(cx, "DropdownMenuContent");
            let menu = cx
                .debug_bounds("dropdown-menu-content")
                .expect("the menu opened");
            // Share, the last row, opens its submenu when pointed at.
            let share = point(menu.center().x, menu.bottom() - px(20.));
            cx.simulate_mouse_move(share, None, Modifiers::default());
            draw(cx);
            assert!(cx.debug_bounds("dropdown-menu-sub-content").is_some());
        }

        #[gpui::test]
        fn should_push_a_toast(cx: &mut TestAppContext) {
            let cx = open(cx, "Toast");
            assert!(cx.debug_bounds("toast").is_some());
        }
    }
}
