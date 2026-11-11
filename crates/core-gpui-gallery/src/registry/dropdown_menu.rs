//! The dropdown menu's parts: `DropdownMenuItem` (Penpot property `variant`),
//! `DropdownMenuCheckboxItem` (`checked`) and the rest, with the stories' content. The
//! `Default` story spreads its args over its first item, Profile, so the item variants are
//! Profile.

use brrritscold_core_gpui::{
    DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuItem, DropdownMenuItemVariant,
    DropdownMenuLabel, DropdownMenuRadioItem, DropdownMenuSeparator, DropdownMenuShortcut,
    DropdownMenuSubContent, DropdownMenuSubTrigger, IconName,
};
use gpui::{AnyElement, IntoElement, ParentElement as _, Styled as _, div, px};
use strum::IntoEnumIterator as _;

use super::{Entry, Variant, one};

const STORY: &str = "components-dropdownmenu--default";

/// The `Default` story's rows: its menu is as wide as "Delete account" with its icon, 147px,
/// less its 4px padding each side. (The published stories have no `w-56`.)
const DEFAULT_ROW: f32 = 139.;

/// The other stories' rows: their menus are `min-w-32`, 128px, less the padding.
const MIN_ROW: f32 = 120.;

/// A row at a story menu's content width.
fn in_menu(width: f32, part: impl IntoElement) -> AnyElement {
    div().w(px(width)).child(part).into_any_element()
}

fn profile() -> DropdownMenuItem {
    DropdownMenuItem::new("Profile").shortcut(DropdownMenuShortcut::new("⇧⌘P"))
}

/// The `Default` story's menu.
pub(super) fn content() -> DropdownMenuContent {
    DropdownMenuContent::new()
        .label(DropdownMenuLabel::new("My account"))
        .item(profile())
        .item(DropdownMenuItem::new("Settings").shortcut(DropdownMenuShortcut::new("⌘,")))
        .item(DropdownMenuItem::new("Billing").disabled(true))
        .separator(DropdownMenuSeparator::new())
        .item(
            DropdownMenuItem::new("Delete account")
                .variant(DropdownMenuItemVariant::Destructive)
                .icon(IconName::Trash),
        )
}

/// The `Submenu` story's submenu.
pub(super) fn sub_content() -> DropdownMenuSubContent {
    DropdownMenuSubContent::new()
        .item(DropdownMenuItem::new("Email"))
        .item(DropdownMenuItem::new("Message"))
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "DropdownMenuContent",
            properties: &[],
            variants: || one(Some(STORY), || content().into_any_element()),
        },
        Entry {
            name: "DropdownMenuItem",
            properties: &["variant"],
            variants: || {
                DropdownMenuItemVariant::iter()
                    .map(|variant| Variant {
                        props: vec![("variant", variant.into())],
                        story: Some(STORY),
                        render: Box::new(move |_| in_menu(DEFAULT_ROW, profile().variant(variant))),
                    })
                    .collect()
            },
        },
        Entry {
            name: "DropdownMenuLabel",
            properties: &[],
            variants: || {
                one(Some(STORY), || {
                    in_menu(DEFAULT_ROW, DropdownMenuLabel::new("My account"))
                })
            },
        },
        Entry {
            name: "DropdownMenuShortcut",
            properties: &[],
            variants: || {
                one(Some(STORY), || {
                    DropdownMenuShortcut::new("⇧⌘P").into_any_element()
                })
            },
        },
        Entry {
            name: "DropdownMenuSeparator",
            properties: &[],
            variants: || {
                one(Some(STORY), || {
                    in_menu(DEFAULT_ROW, DropdownMenuSeparator::new())
                })
            },
        },
        Entry {
            name: "DropdownMenuCheckboxItem",
            properties: &["checked"],
            variants: || {
                [false, true]
                    .into_iter()
                    .map(|checked| Variant {
                        props: vec![("checked", if checked { "true" } else { "false" })],
                        // The story's first checkbox item, Status bar, is checked.
                        story: checked.then_some("components-dropdownmenu--checkboxes"),
                        render: Box::new(move |_| {
                            in_menu(
                                MIN_ROW,
                                DropdownMenuCheckboxItem::new("Status bar").checked(checked),
                            )
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "DropdownMenuRadioItem",
            properties: &[],
            // The story's first radio item, Top, which its play function checks (and Penpot's
            // main component shows checked).
            variants: || {
                one(Some("components-dropdownmenu--radio-items"), || {
                    in_menu(MIN_ROW, DropdownMenuRadioItem::new("Top").checked(true))
                })
            },
        },
        Entry {
            name: "DropdownMenuSubTrigger",
            properties: &[],
            // The story shoots it with its submenu open, highlighted; here it's at rest.
            variants: || {
                one(Some("components-dropdownmenu--submenu"), || {
                    in_menu(MIN_ROW, DropdownMenuSubTrigger::new("Share"))
                })
            },
        },
        Entry {
            name: "DropdownMenuSubContent",
            properties: &[],
            variants: || {
                one(Some("components-dropdownmenu--submenu"), || {
                    sub_content().into_any_element()
                })
            },
        },
    ]
}
