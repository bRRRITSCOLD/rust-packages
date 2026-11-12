//! `Sidebar` and its parts (`SidebarMenuButton` with Penpot property `isActive`), with the
//! stories' content: an "Acme" header, a "Workspace" group of Inbox, Calendar, Search and
//! Settings, a separator and an Account row in the footer.
//!
//! Each part is shot on its own inside the story's sidebar (`storybook.cjs`'s `INSIDE`), so a
//! part renders here in a sidebar as wide as the room it has in the story (the rows' 240px,
//! inside the group's `p-2`) and on its fill. Penpot's parts hold an empty `Body` slot and
//! its `MenuItem` an inactive row; the stories' first item and badge are on the active Inbox
//! row of `WithBadges`, so `SidebarMenuItem` is too. `SidebarMenuBadge` alone has no story
//! that shows it on an inactive row, so it has no core-react column; `SidebarMenuItem` shows
//! it against core-react. `SidebarSeparator` renders in the whole 256px sidebar, so its 8px
//! insets show; core-react's column crops to the 240px rule. `SidebarMenu` is code-only
//! (core-react allowlists it): no entry.

use brrritscold_core_gpui::{
    IconName, Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel,
    SidebarHeader, SidebarMenu, SidebarMenuBadge, SidebarMenuButton, SidebarMenuItem,
    SidebarSeparator,
};
use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, space};
use gpui::{AnyElement, IntoElement, ParentElement as _, Pixels, Styled as _, div, px};

use super::support::{BOOLS, value};
use super::{Entry, Variant, one};

const DEFAULT: Option<&str> = Some("components-sidebar--default");
const ACTIVE_ITEM: Option<&str> = Some("components-sidebar--active-item");
const WITH_BADGES: Option<&str> = Some("components-sidebar--with-badges");

/// A row's width: the 256px sidebar less its group's `p-2`.
const ROW_WIDTH: Pixels = px(240.);

const ITEMS: [(&str, IconName); 4] = [
    ("Inbox", IconName::Inbox),
    ("Calendar", IconName::Calendar),
    ("Search", IconName::Search),
    ("Settings", IconName::Settings),
];

fn button(label: &'static str, icon: IconName) -> SidebarMenuButton {
    SidebarMenuButton::new(label).icon(icon).label(label)
}

fn header() -> SidebarHeader {
    // <span className="px-2 text-sm font-medium">Acme</span>
    SidebarHeader::new().child(
        div()
            .px(space::S2)
            .text_size(font_size::SM)
            .font_weight(font_weight::MEDIUM)
            .child("Acme"),
    )
}

fn group_label() -> SidebarGroupLabel {
    SidebarGroupLabel::new("Workspace")
}

fn group() -> SidebarGroup {
    let menu = ITEMS
        .into_iter()
        .fold(SidebarMenu::new(), |menu, (label, icon)| {
            menu.child(SidebarMenuItem::new().button(button(label, icon)))
        });
    SidebarGroup::new().child(group_label()).child(menu)
}

fn content() -> SidebarContent {
    SidebarContent::new().child(group())
}

fn footer() -> SidebarFooter {
    SidebarFooter::new().child(
        SidebarMenu::new().child(SidebarMenuItem::new().button(button("Account", IconName::User))),
    )
}

/// `WithBadges`' first item: the active Inbox row and its count.
fn item() -> SidebarMenuItem {
    SidebarMenuItem::new()
        .button(button("Inbox", IconName::Inbox).is_active(true))
        .badge(SidebarMenuBadge::new("12"))
}

/// A part in a sidebar as wide as the room it has in the story, on the sidebar's fill.
fn inside(width: Pixels, part: impl IntoElement) -> AnyElement {
    Sidebar::new()
        .w(width)
        .h_auto()
        .child(part)
        .into_any_element()
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Sidebar",
            properties: &[],
            variants: || {
                one(DEFAULT, || {
                    Sidebar::new()
                        .h_auto()
                        .child(header())
                        .child(content())
                        .child(SidebarSeparator::new())
                        .child(footer())
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "SidebarHeader",
            properties: &[],
            variants: || one(DEFAULT, || inside(space::S64, header())),
        },
        Entry {
            name: "SidebarContent",
            properties: &[],
            variants: || one(DEFAULT, || inside(space::S64, content())),
        },
        Entry {
            name: "SidebarGroup",
            properties: &[],
            variants: || one(DEFAULT, || inside(space::S64, group())),
        },
        Entry {
            name: "SidebarGroupLabel",
            properties: &[],
            variants: || one(DEFAULT, || inside(ROW_WIDTH, group_label())),
        },
        Entry {
            name: "SidebarMenuItem",
            properties: &[],
            variants: || one(WITH_BADGES, || inside(ROW_WIDTH, item())),
        },
        Entry {
            name: "SidebarMenuButton",
            properties: &["isActive"],
            variants: || {
                BOOLS
                    .into_iter()
                    .map(|is_active| Variant {
                        props: vec![("isActive", value(is_active))],
                        // The first row of each: Default's Inbox is inactive, ActiveItem's
                        // active (its play function's focus is blurred before the shot).
                        story: if is_active { ACTIVE_ITEM } else { DEFAULT },
                        render: Box::new(move |id| {
                            inside(
                                ROW_WIDTH,
                                SidebarMenuButton::new(id)
                                    .icon(IconName::Inbox)
                                    .label("Inbox")
                                    .is_active(is_active),
                            )
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "SidebarMenuBadge",
            properties: &[],
            variants: || {
                one(None, || {
                    // On the sidebar's fill, as wide as the badge.
                    Sidebar::new()
                        .w_auto()
                        .h_auto()
                        .child(SidebarMenuBadge::new("12"))
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "SidebarSeparator",
            properties: &[],
            variants: || one(DEFAULT, || inside(space::S64, SidebarSeparator::new())),
        },
        Entry {
            name: "SidebarFooter",
            properties: &[],
            variants: || one(DEFAULT, || inside(space::S64, footer())),
        },
    ]
}
