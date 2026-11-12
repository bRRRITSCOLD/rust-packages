//! Sidebar: core-react's `sidebar.tsx`, its static parts as plain styled elements and the menu
//! button on `gpui-base`'s unstyled [`gpui_base::Button`].
//!
//! A static, non-collapsible sidebar at 256px on the `sidebar` fill, as core-react's is: no
//! provider, trigger, rail or mobile sheet. Its containers ([`Sidebar`], [`SidebarHeader`],
//! [`SidebarContent`], [`SidebarGroup`], [`SidebarMenu`], [`SidebarFooter`]) hold open content.
//! A row's look depends on its siblings in core-react (`group-has-[badge]/menu-item:pr-8`,
//! `peer-data-[active=true]/menu-button:`), so [`SidebarMenuItem`] takes its button and badge as
//! typed builders and hands each the other's state.
//!
//! Not mirrored:
//!
//! - `render` (Base UI's, on the group label and the menu button): an app navigates from the
//!   button's [`on_click`](SidebarMenuButton::on_click).
//! - The `--sidebar-width` variable: the width is `space.64`, and the caller's own `.w(..)` wins,
//!   as a class or `style` does in core-react.
//! - `select-none` and `pointer-events-none` on the badge: GPUI text isn't selectable, and the
//!   badge takes no pointer events, so a click on it reaches the button beneath.
//! - The row's `outline-hidden`: GPUI draws no outline.

use std::sync::Arc;

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, space};
use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontFeatures, FontWeight, Hsla,
    InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, SharedString,
    StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use gpui_base::StyledExt as _;

use crate::components::separator::Separator;
use crate::icons::{Icon, IconName};
use crate::theme::{ActiveTheme as _, Theme};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// Group name a row's badge reads for `peer-hover/menu-button:`; the nearest row wins.
const ITEM_GROUP: &str = "brrritscold-core-gpui-sidebar-menu-item";

/// Declares an open-content part: a `children` list and a style, built empty by `new()`.
macro_rules! open_part {
    ($(#[$doc:meta])* $part:ident, $new_doc:literal) => {
        $(#[$doc])*
        #[derive(IntoElement)]
        pub struct $part {
            children: Vec<AnyElement>,
            style: StyleRefinement,
        }

        impl Default for $part {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $part {
            #[doc = $new_doc]
            pub fn new() -> Self {
                Self {
                    children: Vec::new(),
                    style: StyleRefinement::default(),
                }
            }
        }
    };
}

open_part! {
    /// A static sidebar, styled like core-react's `Sidebar`: a 256px column on the `sidebar`
    /// fill, as tall as its container. Its children are its parts:
    ///
    /// ```no_run
    /// use brrritscold_core_gpui::{
    ///     IconName, Sidebar, SidebarContent, SidebarGroup, SidebarGroupLabel, SidebarMenu,
    ///     SidebarMenuBadge, SidebarMenuButton, SidebarMenuItem,
    /// };
    /// use gpui::ParentElement as _;
    ///
    /// let sidebar = Sidebar::new().child(
    ///     SidebarContent::new().child(
    ///         SidebarGroup::new()
    ///             .child(SidebarGroupLabel::new("Workspace"))
    ///             .child(
    ///                 SidebarMenu::new().child(
    ///                     SidebarMenuItem::new()
    ///                         .button(
    ///                             SidebarMenuButton::new("inbox")
    ///                                 .icon(IconName::Inbox)
    ///                                 .label("Inbox")
    ///                                 .is_active(true)
    ///                                 .on_click(|_, _, _| println!("inbox")),
    ///                         )
    ///                         .badge(SidebarMenuBadge::new("12")),
    ///                 ),
    ///             ),
    ///     ),
    /// );
    /// ```
    Sidebar,
    "An empty sidebar; its children are its parts."
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // flex h-full w-(--sidebar-width) flex-col bg-sidebar text-sidebar-foreground
        // [--sidebar-width:var(--spacing-64)]
        div()
            .flex()
            .h_full()
            .w(space::S64)
            .flex_col()
            .bg(s.sidebar)
            .text_color(s.sidebar_foreground)
            .children(self.children)
            .refine_style(&self.style)
    }
}

open_part! {
    /// The sidebar's `Header` part: open content (its Penpot `Body` slot) above the content.
    SidebarHeader,
    "An empty header; its children are its content."
}

impl RenderOnce for SidebarHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex flex-col gap-2 p-2
        div()
            .flex()
            .flex_col()
            .gap(space::S2)
            .p(space::S2)
            .children(self.children)
            .refine_style(&self.style)
    }
}

/// The sidebar's `Content` part: open content (its Penpot `Body` slot) that takes the height
/// left between the header and the footer, and scrolls when its groups don't fit.
#[derive(IntoElement)]
pub struct SidebarContent {
    id: ElementId,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Default for SidebarContent {
    fn default() -> Self {
        Self::new()
    }
}

impl SidebarContent {
    /// An empty content part; its children are its groups.
    pub fn new() -> Self {
        Self {
            id: "sidebar-content".into(),
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Keys the scroll position, `sidebar-content` unless set. Give each sidebar's content its
    /// own when two sidebars share a parent, so they don't share a scroll position.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for SidebarContent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex min-h-0 flex-1 flex-col gap-2 overflow-auto
        div()
            .id(self.id)
            .flex()
            .min_h_0()
            .flex_1()
            .flex_col()
            .gap(space::S2)
            .overflow_y_scroll()
            .children(self.children)
            .refine_style(&self.style)
    }
}

open_part! {
    /// The sidebar's `Group` part: a [`SidebarGroupLabel`] and a [`SidebarMenu`] (its Penpot
    /// `Body` slot), in that order.
    SidebarGroup,
    "An empty group; its children are its label and menu."
}

impl RenderOnce for SidebarGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // relative flex w-full min-w-0 flex-col p-2
        div()
            .relative()
            .flex()
            .w_full()
            .min_w_0()
            .flex_col()
            .p(space::S2)
            .children(self.children)
            .refine_style(&self.style)
    }
}

/// The sidebar's `GroupLabel` part: a group's muted, 32px heading.
#[derive(IntoElement)]
pub struct SidebarGroupLabel {
    text: SharedString,
    style: StyleRefinement,
}

impl SidebarGroupLabel {
    /// A label reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for SidebarGroupLabel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // flex h-8 shrink-0 items-center px-2 text-xs font-medium text-muted-foreground
        div()
            .flex()
            .h(space::S8)
            .flex_shrink_0()
            .items_center()
            .px(space::S2)
            .text_size(font_size::XS)
            .font_weight(font_weight::MEDIUM)
            .text_color(cx.theme().semantic().muted_foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

open_part! {
    /// A group's list of [`SidebarMenuItem`]s, 4px apart: core-react's `SidebarMenu`. A
    /// code-only part: the library's rows sit straight in `Sidebar/Group`.
    SidebarMenu,
    "An empty menu; its children are its items."
}

impl RenderOnce for SidebarMenu {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex w-full min-w-0 flex-col gap-1
        div()
            .flex()
            .w_full()
            .min_w_0()
            .flex_col()
            .gap(space::S1)
            .children(self.children)
            .refine_style(&self.style)
    }
}

/// The sidebar's `MenuItem` part: a row's [`SidebarMenuButton`] and, after it, an optional
/// [`SidebarMenuBadge`] at its right edge.
#[derive(IntoElement)]
pub struct SidebarMenuItem {
    button: Option<SidebarMenuButton>,
    badge: Option<SidebarMenuBadge>,
    style: StyleRefinement,
}

impl Default for SidebarMenuItem {
    fn default() -> Self {
        Self::new()
    }
}

impl SidebarMenuItem {
    /// An empty row.
    pub fn new() -> Self {
        Self {
            button: None,
            badge: None,
            style: StyleRefinement::default(),
        }
    }

    /// The `MenuButton` part.
    pub fn button(mut self, button: SidebarMenuButton) -> Self {
        self.button = Some(button);
        self
    }

    /// The `MenuBadge` part: a count over the button's right end, which the button leaves
    /// room for.
    pub fn badge(mut self, badge: SidebarMenuBadge) -> Self {
        self.badge = Some(badge);
        self
    }
}

impl RenderOnce for SidebarMenuItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let has_badge = self.badge.is_some();
        let is_active = self.button.as_ref().is_some_and(|b| b.is_active);
        // group/menu-item relative
        div()
            .group(ITEM_GROUP)
            .relative()
            .children(self.button.map(|button| button.has_badge(has_badge)))
            .children(self.badge.map(|badge| badge.in_row(is_active)))
            .refine_style(&self.style)
    }
}

/// What a menu button paints, resolved from the theme and `isActive`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct MenuButtonPaint {
    fill: Option<Hsla>,
    text: Hsla,
    weight: FontWeight,
    hover_fill: Hsla,
    hover_text: Hsla,
}

/// The sidebar's `MenuButton` part: a 32px row with an icon and a label, on `sidebar-accent`
/// when it is the current one.
#[derive(IntoElement)]
pub struct SidebarMenuButton {
    id: ElementId,
    icon: Option<IconName>,
    label: Option<SharedString>,
    is_active: bool,
    has_badge: bool,
    on_click: Option<ClickHandler>,
    style: StyleRefinement,
}

impl SidebarMenuButton {
    /// An inactive row with no icon or label. `id` keys its focus state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            label: None,
            is_active: false,
            has_badge: false,
            on_click: None,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `isActive`: whether this is the current row. It only styles the row,
    /// as core-react's does.
    pub fn is_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// The `Icon` layer, before the label.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// The `Label` layer.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Runs on a pointer click, Enter or Space: where the app navigates.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// `group-has-data-[slot=sidebar-menu-badge]/menu-item:pr-8`, from its item.
    fn has_badge(mut self, has_badge: bool) -> Self {
        self.has_badge = has_badge;
        self
    }

    fn paint(&self, theme: &Theme) -> MenuButtonPaint {
        let s = theme.semantic();
        // hover:bg-sidebar-accent hover:text-sidebar-accent-foreground
        let (hover_fill, hover_text) = (s.sidebar_accent, s.sidebar_accent_foreground);
        if self.is_active {
            // data-[active=true]:bg-sidebar-accent data-[active=true]:font-medium
            // data-[active=true]:text-sidebar-accent-foreground
            MenuButtonPaint {
                fill: Some(s.sidebar_accent),
                text: s.sidebar_accent_foreground,
                weight: font_weight::MEDIUM,
                hover_fill,
                hover_text,
            }
        } else {
            // text-sidebar-foreground, no fill
            MenuButtonPaint {
                fill: None,
                text: s.sidebar_foreground,
                weight: font_weight::NORMAL,
                hover_fill,
                hover_text,
            }
        }
    }
}

impl RenderOnce for SidebarMenuButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let s = theme.semantic();
        let paint = self.paint(theme);
        // GPUI paints a box shadow under the whole box, so a transparent row would show the
        // ring through as a wash: a focused row paints the sidebar fill under its own.
        let focus_fill = paint.fill.unwrap_or(s.sidebar);
        let ring = super::ring(2., s.sidebar_ring);
        // peer/menu-button flex h-8 w-full items-center gap-2 rounded-md px-2 text-left
        // text-sm text-sidebar-foreground outline-hidden ring-sidebar-ring
        gpui_base::Button::new(self.id)
            .flex()
            .h(space::S8)
            .w_full()
            .items_center()
            .justify_start()
            .gap(space::S2)
            .rounded(s.radius_md)
            .px(space::S2)
            .when(self.has_badge, |b| b.pr(space::S8))
            .text_size(font_size::SM)
            .font_weight(paint.weight)
            .text_color(paint.text)
            .when_some(paint.fill, |b, fill| b.bg(fill))
            // focus-visible:ring-2
            .focus_visible(move |style| style.bg(focus_fill).shadow(vec![ring]))
            .hover(move |style| style.bg(paint.hover_fill).text_color(paint.hover_text))
            .when_some(self.on_click, |b, on_click| {
                b.on_click(move |event, window, cx| on_click(event, window, cx))
            })
            // [&>svg]:size-4 [&>svg]:shrink-0
            .when_some(self.icon, |b, icon| {
                b.child(div().flex_shrink_0().child(Icon::new(icon).size(space::S4)))
            })
            .when_some(self.label, |b, label| b.child(label))
            .refine_style(&self.style)
    }
}

/// The sidebar's `MenuBadge` part: a count at a row's right edge, in `muted-foreground`, or
/// `sidebar-accent-foreground` on a hovered or current row.
#[derive(IntoElement)]
pub struct SidebarMenuBadge {
    text: SharedString,
    in_row: Option<bool>,
    style: StyleRefinement,
}

impl SidebarMenuBadge {
    /// A badge reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            in_row: None,
            style: StyleRefinement::default(),
        }
    }

    /// Placed in a [`SidebarMenuItem`] whose button is active or not.
    fn in_row(mut self, is_active: bool) -> Self {
        self.in_row = Some(is_active);
        self
    }

    fn color(&self, theme: &Theme) -> Hsla {
        let s = theme.semantic();
        match self.in_row {
            // peer-data-[active=true]/menu-button:text-sidebar-accent-foreground
            Some(true) => s.sidebar_accent_foreground,
            _ => s.muted_foreground,
        }
    }
}

impl RenderOnce for SidebarMenuBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let hover_text = theme.semantic().sidebar_accent_foreground;
        // pointer-events-none absolute top-1.5 right-1 flex h-5 min-w-5 items-center
        // justify-center gap-1 px-1 text-xs tabular-nums text-muted-foreground select-none
        div()
            .when(self.in_row.is_some(), |d| {
                d.absolute()
                    .top(space::S1_5)
                    .right(space::S1)
                    // peer-hover/menu-button:text-sidebar-accent-foreground
                    .group_hover(ITEM_GROUP, move |style| style.text_color(hover_text))
            })
            .flex()
            .h(space::S5)
            .min_w(space::S5)
            .items_center()
            .justify_center()
            .gap(space::S1)
            .px(space::S1)
            .text_size(font_size::XS)
            .font_features(FontFeatures(Arc::new(vec![("tnum".into(), 1)])))
            .text_color(self.color(theme))
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The sidebar's `Separator` part: a `sidebar-border` rule inset 8px each side.
#[derive(IntoElement)]
pub struct SidebarSeparator {
    style: StyleRefinement,
}

impl Default for SidebarSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl SidebarSeparator {
    /// A separator.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for SidebarSeparator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // <Separator> mx-2 bg-sidebar-border data-[orientation=horizontal]:w-auto
        Separator::new()
            .mx(space::S2)
            .bg(cx.theme().semantic().sidebar_border)
            .w_auto()
            .refine_style(&self.style)
    }
}

open_part! {
    /// The sidebar's `Footer` part: open content (its Penpot `Body` slot) below the content.
    SidebarFooter,
    "An empty footer; its children are its content."
}

impl RenderOnce for SidebarFooter {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // flex flex-col gap-2 p-2
        div()
            .flex()
            .flex_col()
            .gap(space::S2)
            .p(space::S2)
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl_styled!(
    Sidebar,
    SidebarHeader,
    SidebarContent,
    SidebarGroup,
    SidebarGroupLabel,
    SidebarMenu,
    SidebarMenuItem,
    SidebarMenuButton,
    SidebarMenuBadge,
    SidebarSeparator,
    SidebarFooter
);
impl_parent!(
    Sidebar,
    SidebarHeader,
    SidebarContent,
    SidebarGroup,
    SidebarMenu,
    SidebarFooter
);

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, px};

    use super::*;
    use crate::components::test_support::{render, theme};
    use crate::theme::Mode;

    /// The story's row: an icon and a label.
    fn row(id: &'static str, label: &'static str) -> SidebarMenuButton {
        SidebarMenuButton::new(id)
            .icon(IconName::Inbox)
            .label(label)
    }

    /// core-react's `Default` story: a header, a group of four rows, a separator and a footer
    /// row.
    fn story() -> Sidebar {
        let menu = ["Inbox", "Calendar", "Search", "Settings"]
            .into_iter()
            .fold(SidebarMenu::new(), |menu, label| {
                menu.child(SidebarMenuItem::new().button(row(label, label)))
            });
        Sidebar::new()
            .child(
                SidebarHeader::new().child(
                    div()
                        .px(space::S2)
                        .text_size(font_size::SM)
                        .font_weight(font_weight::MEDIUM)
                        .child("Acme"),
                ),
            )
            .child(
                SidebarContent::new().child(
                    SidebarGroup::new()
                        .child(SidebarGroupLabel::new("Workspace"))
                        .child(menu),
                ),
            )
            .child(SidebarSeparator::new())
            .child(SidebarFooter::new().child(
                SidebarMenu::new().child(SidebarMenuItem::new().button(row("account", "Account"))),
            ))
    }

    mod sidebar {
        use super::*;

        #[gpui::test]
        fn should_lay_the_story_out_as_chromium_does(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, story);
            assert_eq!(bounds.size.width, px(256.));
            // header: p-2 around a 14px × 1.5 line; content: a group's p-2 around a 32px label
            // and four 32px rows 4px apart; a 1px separator; footer: p-2 around one row.
            let header = 8. + 21. + 8.;
            let content = 8. + 32. + (4. * 32. + 3. * 4.) + 8.;
            let footer = 8. + 32. + 8.;
            assert_eq!(bounds.size.height, px(header + content + 1. + footer));
        }

        #[gpui::test]
        fn should_take_the_callers_width(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || story().w(px(320.)));
            assert_eq!(bounds.size.width, px(320.));
        }
    }

    mod group_label {
        use super::*;

        #[gpui::test]
        fn should_be_a_32px_row(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || SidebarGroupLabel::new("Workspace"));
            assert_eq!(bounds.size.height, px(32.));
        }
    }

    mod menu_button {
        use super::*;

        #[gpui::test]
        fn should_be_a_32px_row(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || row("inbox", "Inbox"));
            assert_eq!(bounds.size.height, px(32.));
        }

        #[gpui::test]
        fn should_fill_an_active_row_with_sidebar_accent_in_both_modes(cx: &mut TestAppContext) {
            cx.update(crate::init);
            for mode in [Mode::Light, Mode::Dark] {
                let theme = theme(cx, mode);
                let s = theme.semantic();
                let paint = row("inbox", "Inbox").is_active(true).paint(&theme);
                assert_eq!(paint.fill, Some(s.sidebar_accent), "{mode:?}");
                assert_eq!(paint.text, s.sidebar_accent_foreground, "{mode:?}");
                assert_eq!(paint.weight, font_weight::MEDIUM, "{mode:?}");
            }
        }

        #[gpui::test]
        fn should_leave_an_inactive_row_unfilled_until_hovered(cx: &mut TestAppContext) {
            cx.update(crate::init);
            for mode in [Mode::Light, Mode::Dark] {
                let theme = theme(cx, mode);
                let s = theme.semantic();
                let paint = row("inbox", "Inbox").paint(&theme);
                assert_eq!(paint.fill, None, "{mode:?}");
                assert_eq!(paint.text, s.sidebar_foreground, "{mode:?}");
                assert_eq!(paint.weight, font_weight::NORMAL, "{mode:?}");
                assert_eq!(paint.hover_fill, s.sidebar_accent, "{mode:?}");
                assert_eq!(paint.hover_text, s.sidebar_accent_foreground, "{mode:?}");
            }
        }

        #[gpui::test]
        fn should_run_the_handler_on_click(cx: &mut TestAppContext) {
            use std::cell::Cell;
            use std::rc::Rc;

            cx.update(crate::init);
            let clicks = Rc::new(Cell::new(0));
            let (cx, bounds) = render(cx, {
                let clicks = clicks.clone();
                move || {
                    let clicks = clicks.clone();
                    row("inbox", "Inbox")
                        .w(px(240.))
                        .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                }
            });
            cx.simulate_click(bounds.center(), gpui::Modifiers::default());
            assert_eq!(clicks.get(), 1);
        }
    }

    mod menu_badge {
        use super::*;

        #[gpui::test]
        fn should_read_muted_unless_its_row_is_active(cx: &mut TestAppContext) {
            cx.update(crate::init);
            for mode in [Mode::Light, Mode::Dark] {
                let theme = theme(cx, mode);
                let s = theme.semantic();
                let badge = || SidebarMenuBadge::new("12");
                assert_eq!(badge().color(&theme), s.muted_foreground);
                assert_eq!(badge().in_row(false).color(&theme), s.muted_foreground);
                assert_eq!(
                    badge().in_row(true).color(&theme),
                    s.sidebar_accent_foreground
                );
            }
        }

        #[gpui::test]
        fn should_keep_its_row_32px_tall(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || {
                SidebarMenuItem::new()
                    .w(px(240.))
                    .button(row("inbox", "Inbox").is_active(true))
                    .badge(SidebarMenuBadge::new("12"))
            });
            assert_eq!(bounds.size.height, px(32.));
            assert_eq!(bounds.size.width, px(240.));
        }

        #[test]
        fn should_make_room_on_its_button() {
            let item = SidebarMenuItem::new()
                .button(row("inbox", "Inbox"))
                .badge(SidebarMenuBadge::new("12"));
            let has_badge = item.badge.is_some();
            let button = item.button.expect("a button").has_badge(has_badge);
            assert!(button.has_badge);
        }
    }

    mod separator {
        use std::cell::Cell;
        use std::rc::Rc;

        use super::*;

        #[gpui::test]
        fn should_inset_a_1px_rule_8px_each_side(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let rule = Rc::new(Cell::new(None));
            let (_, sidebar) = render(cx, {
                let rule = rule.clone();
                move || {
                    let rule = rule.clone();
                    // The sidebar's column, which records its children's bounds.
                    div()
                        .flex()
                        .flex_col()
                        .w(space::S64)
                        .child(SidebarSeparator::new())
                        .on_children_prepainted(move |children, _, _| {
                            rule.set(children.first().copied())
                        })
                }
            });
            let rule = rule.get().expect("the separator was laid out");
            assert_eq!(rule.size.height, px(1.));
            assert_eq!(rule.left() - sidebar.left(), px(8.));
            assert_eq!(sidebar.right() - rule.right(), px(8.));
        }
    }
}
