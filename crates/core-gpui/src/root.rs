//! The per-window half of the theme: a `gpui-base` root plugin, mounted on every [`Root`]
//! once [`crate::init`] has registered it.
//!
//! `gpui-base`'s [`Root`] has to be the window's root view (its overlays and `Root::read`
//! depend on it), so core-gpui extends it through a plugin rather than wrapping it. The plugin
//! feeds the window's OS appearance into the [`Theme`] and gives the root surface the theme's
//! background, foreground and sans font, at `line-height.normal` (1.5): core-react's text
//! inherits the 1.5 Tailwind's preflight sets on `html`, and GPUI's own default is 1.618.
//!
//! [`Root`]: gpui_base::Root

use brrritscold_core_gpui_tokens::primitive;
use gpui::{
    App, Context, Div, Empty, IntoElement, Render, Stateful, Styled, Subscription, Window, relative,
};
use gpui_base::RootPlugin;

use crate::theme::{ActiveTheme as _, Theme};

pub(crate) fn register(cx: &mut App) {
    gpui_base::Root::register_plugin(cx, ThemePlugin::new);
}

pub(crate) struct ThemePlugin {
    _appearance: Subscription,
}

impl ThemePlugin {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // A window can open after the OS appearance changed; catch up before the first frame.
        Theme::appearance_changed(window.appearance(), cx);
        let appearance = window.observe_window_appearance(|window, cx| {
            Theme::appearance_changed(window.appearance(), cx);
        });
        Self {
            _appearance: appearance,
        }
    }
}

impl Render for ThemePlugin {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

impl RootPlugin for ThemePlugin {
    fn style(&self, surface: &mut Stateful<Div>, _window: &mut Window, cx: &mut App) {
        let s = cx.theme().semantic();
        let sans = primitive::font_family::SANS;
        let style = surface.style();
        style.background = Some(s.background.into());
        style.text.color = Some(s.foreground);
        style.text.font_family = Some(sans.family.into());
        style.text.font_fallbacks = Some(sans.font_fallbacks());
        style.text.line_height = Some(relative(primitive::line_height::NORMAL));
    }
}

#[cfg(test)]
mod tests {
    use gpui::{AppContext as _, TestAppContext, VisualTestContext, div};

    use super::*;
    use crate::theme::Mode;

    struct Blank;

    impl Render for Blank {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    mod theme_plugin {
        use super::*;

        #[gpui::test]
        fn should_mount_on_every_root(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let window = cx.add_window(|window, cx| {
                let blank = cx.new(|_| Blank);
                gpui_base::Root::new(blank, window, cx)
            });
            let cx = &mut VisualTestContext::from_window(*window, cx);
            let mounted = window
                .update(cx, |root, _, _| root.plugin::<ThemePlugin>().is_some())
                .unwrap();
            assert!(mounted);
            cx.run_until_parked();
            assert_eq!(cx.read(|cx| cx.theme().mode()), Mode::Light);
        }
    }
}
