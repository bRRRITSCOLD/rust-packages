//! The gallery window: a sidebar of registry entries plus Icons, one page each, and a
//! light/dark toggle.

use brrritscold_core_gpui::{
    ActiveTheme as _, Button, ButtonSize, ButtonVariant, Icon, IconName, Theme, ThemePreference,
};
use brrritscold_core_gpui_tokens::primitive::font_weight;
use gpui::{
    App, Context, InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div, px, uniform_list,
};

use crate::registry::{Entry, Variant, registry};

/// Which page the gallery shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    /// The registry entry at this index.
    Component(usize),
    /// Every icon.
    Icons,
}

/// The gallery's root view. Wrap it in a [`brrritscold_core_gpui::Root`].
pub struct Gallery {
    entries: Vec<Entry>,
    page: Page,
}

impl Default for Gallery {
    fn default() -> Self {
        Self::new()
    }
}

impl Gallery {
    /// Opens on the first registry entry.
    pub fn new() -> Self {
        Self {
            entries: registry(),
            page: Page::Component(0),
        }
    }

    /// The page showing.
    pub fn page(&self) -> Page {
        self.page
    }

    /// Shows another page.
    pub fn show(&mut self, page: Page, cx: &mut Context<Self>) {
        self.page = page;
        cx.notify();
    }

    fn title(&self) -> &'static str {
        match self.page {
            Page::Component(index) => self.entries[index].name,
            Page::Icons => "Icons",
        }
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let s = cx.theme().semantic();
        let pages = (0..self.entries.len())
            .map(Page::Component)
            .chain([Page::Icons]);
        let items = pages.enumerate().map(|(index, page)| {
            let name = match page {
                Page::Component(entry) => self.entries[entry].name,
                Page::Icons => "Icons",
            };
            let variant = if page == self.page {
                ButtonVariant::Secondary
            } else {
                ButtonVariant::Ghost
            };
            Button::new(("nav", index))
                .variant(variant)
                .size(ButtonSize::Sm)
                .label(name)
                .on_click(cx.listener(move |this, _, _, cx| this.show(page, cx)))
        });
        div()
            .w(px(224.))
            .flex_none()
            .h_full()
            .flex()
            .flex_col()
            .items_start()
            .gap_1()
            .p_3()
            .border_r_1()
            .border_color(s.border)
            .child(
                div()
                    .px_2p5()
                    .py_2()
                    .text_sm()
                    .font_weight(font_weight::SEMIBOLD)
                    .child("core-gpui"),
            )
            .children(items)
    }

    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let s = theme.semantic();
        let mode = theme.mode();
        div()
            .h(px(56.))
            .flex_none()
            .px_6()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(s.border)
            .child(
                div()
                    .text_lg()
                    .font_weight(font_weight::SEMIBOLD)
                    .child(self.title()),
            )
            .child(
                Button::new("theme-toggle")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::IconSm)
                    .icon(mode.pick(IconName::Moon, IconName::Sun))
                    .on_click(|_, _, cx| toggle_theme(cx)),
            )
    }
}

/// Switches between light and dark, leaving the OS appearance behind.
pub fn toggle_theme(cx: &mut App) {
    let next = cx
        .theme()
        .mode()
        .pick(ThemePreference::Dark, ThemePreference::Light);
    Theme::set_preference(next, cx);
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = match self.page {
            Page::Component(index) => component_page(&self.entries[index], cx).into_any_element(),
            Page::Icons => icons_page(cx).into_any_element(),
        };
        div().size_full().flex().child(self.sidebar(cx)).child(
            div()
                .flex_1()
                .min_w_0()
                .h_full()
                .flex()
                .flex_col()
                .child(self.header(cx))
                .child(page),
        )
    }
}

const CELL_WIDTH: Pixels = px(160.);

/// Every variant of `entry`, one row per value of its first property, each cell labelled
/// with its values.
pub fn component_page(entry: &Entry, cx: &App) -> impl IntoElement {
    let muted = cx.theme().semantic().muted_foreground;
    let variants = entry.variants();
    let mut rows: Vec<Vec<(usize, &Variant)>> = Vec::new();
    for (index, variant) in variants.iter().enumerate() {
        match rows.last_mut() {
            Some(row) if row[0].1.props.first() == variant.props.first() => {
                row.push((index, variant))
            }
            _ => rows.push(vec![(index, variant)]),
        }
    }
    div()
        .id("component-page")
        .flex_1()
        .overflow_y_scroll()
        .p_6()
        .flex()
        .flex_col()
        .gap_8()
        .children(rows.into_iter().map(|row| {
            div()
                .flex()
                .flex_wrap()
                .items_end()
                .gap_x_4()
                .gap_y_6()
                .children(row.into_iter().map(|(index, variant)| {
                    // A fixed width lines the columns up across rows.
                    div()
                        .w(CELL_WIDTH)
                        .flex()
                        .flex_col()
                        .items_start()
                        .gap_2()
                        .child(variant.render((entry.name, index)))
                        .child(div().text_xs().text_color(muted).child(variant.label()))
                }))
        }))
}

const ICON_COLUMNS: usize = 4;

/// Every icon with its core-react export name (`PlusIcon`).
pub fn icons_page(cx: &App) -> impl IntoElement {
    let muted = cx.theme().semantic().muted_foreground;
    let rows = IconName::ALL.len().div_ceil(ICON_COLUMNS);
    uniform_list("icons-page", rows, move |range, _, _| {
        range
            .map(|row| {
                let start = row * ICON_COLUMNS;
                let end = (start + ICON_COLUMNS).min(IconName::ALL.len());
                div()
                    .px_6()
                    .py_2()
                    .flex()
                    .children(IconName::ALL[start..end].iter().map(|name| {
                        div()
                            .w(px(240.))
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(Icon::new(*name))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted)
                                    .child(format!("{name:?}Icon")),
                            )
                    }))
            })
            .collect()
    })
    .flex_1()
}

#[cfg(test)]
mod tests {
    use brrritscold_core_gpui::{Mode, Root};
    use gpui::{AppContext as _, TestAppContext, VisualTestContext};

    use super::*;

    fn open(cx: &mut TestAppContext) -> (gpui::Entity<Gallery>, &mut VisualTestContext) {
        cx.update(brrritscold_core_gpui::init);
        let gallery = cx.new(|_| Gallery::new());
        let window = cx.add_window({
            let gallery = gallery.clone();
            move |window, cx| Root::new(gallery, window, cx)
        });
        let cx = VisualTestContext::from_window(*window, cx).into_mut();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (gallery, cx)
    }

    mod gallery {
        use super::*;

        #[gpui::test]
        fn should_render_every_page_in_both_themes(cx: &mut TestAppContext) {
            let (gallery, cx) = open(cx);
            let pages: Vec<Page> = (0..registry().len())
                .map(Page::Component)
                .chain([Page::Icons])
                .collect();
            for preference in [ThemePreference::Light, ThemePreference::Dark] {
                cx.update(|_, cx| Theme::set_preference(preference, cx));
                for page in &pages {
                    gallery.update(cx, |gallery, cx| gallery.show(*page, cx));
                    cx.update(|window, cx| window.draw(cx).clear(cx));
                }
            }
        }

        #[gpui::test]
        fn should_toggle_between_light_and_dark(cx: &mut TestAppContext) {
            let (_, cx) = open(cx);
            cx.update(|_, cx| Theme::set_preference(ThemePreference::Light, cx));
            cx.update(|_, cx| toggle_theme(cx));
            assert_eq!(cx.read(|cx| cx.theme().mode()), Mode::Dark);
            cx.update(|_, cx| toggle_theme(cx));
            assert_eq!(cx.read(|cx| cx.theme().mode()), Mode::Light);
        }
    }
}
