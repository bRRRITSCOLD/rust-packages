//! The active theme: which [`Mode`] is showing, and what the app asked for.
//!
//! There is one way to change the mode: [`Theme::set_preference`] (or an OS appearance change
//! while the preference is [`ThemePreference::System`]). Both go through one path, which writes
//! this crate's [`Theme`] and its projection onto `gpui-base`'s theme, then redraws every
//! window, so styled components and `gpui-base` behaviour never disagree.

use brrritscold_core_gpui_tokens::{Semantic, primitive};
use gpui::{App, Global, Hsla, WindowAppearance, px};

/// The active theme, `Light` or `Dark`. Named after the Penpot library's `Mode` theme group.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Mode {
    /// The `light` token set.
    #[default]
    Light,
    /// The `dark` token set.
    Dark,
}

impl Mode {
    /// Picks the value for this mode. Components use it for core-react's dark-only arms
    /// (`dark:` classes), which aren't tokens: `mode.pick(s.accent, s.input.opacity(0.5))`.
    pub fn pick<T>(self, light: T, dark: T) -> T {
        match self {
            Mode::Light => light,
            Mode::Dark => dark,
        }
    }

    /// The semantic token set for this mode.
    pub fn semantic(self) -> &'static Semantic {
        self.pick(&Semantic::LIGHT, &Semantic::DARK)
    }

    /// The mode matching an OS appearance.
    pub fn from_appearance(appearance: WindowAppearance) -> Mode {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Mode::Dark,
            WindowAppearance::Light | WindowAppearance::VibrantLight => Mode::Light,
        }
    }
}

/// What the app asks for. [`ThemePreference::System`] follows the OS appearance.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ThemePreference {
    /// Follow the OS appearance.
    #[default]
    System,
    /// Always [`Mode::Light`].
    Light,
    /// Always [`Mode::Dark`].
    Dark,
}

impl ThemePreference {
    /// The mode this preference shows under an OS appearance.
    pub fn resolve(self, appearance: WindowAppearance) -> Mode {
        match self {
            ThemePreference::System => Mode::from_appearance(appearance),
            ThemePreference::Light => Mode::Light,
            ThemePreference::Dark => Mode::Dark,
        }
    }
}

/// The app-wide theme. Installed by [`crate::init`]; read it with [`ActiveTheme::theme`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    preference: ThemePreference,
    mode: Mode,
}

impl Global for Theme {}

impl Theme {
    /// The active mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// What the app asked for.
    pub fn preference(&self) -> ThemePreference {
        self.preference
    }

    /// The semantic tokens of the active mode.
    pub fn semantic(&self) -> &'static Semantic {
        self.mode.semantic()
    }

    /// The focus ring colour: `ring` at 50% opacity. Twin of core-react's `outline-ring/50`
    /// base style in `styles.ts`, which isn't a token.
    pub fn focus_ring(&self) -> Hsla {
        self.semantic().ring.opacity(primitive::opacity::O50)
    }

    /// Sets the preference and shows the mode it resolves to.
    pub fn set_preference(preference: ThemePreference, cx: &mut App) {
        let mode = preference.resolve(cx.window_appearance());
        cx.global_mut::<Theme>().preference = preference;
        set_mode(mode, cx);
    }

    /// An OS appearance change. Only a [`ThemePreference::System`] preference follows it.
    pub(crate) fn appearance_changed(appearance: WindowAppearance, cx: &mut App) {
        let preference = cx.global::<Theme>().preference;
        set_mode(preference.resolve(appearance), cx);
    }
}

/// Read access to the active [`Theme`].
pub trait ActiveTheme {
    /// The active theme. Panics if [`crate::init`] hasn't run.
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        self.try_global::<Theme>()
            .expect("brrritscold_core_gpui::init must run before the theme is read")
    }
}

pub(crate) fn init(cx: &mut App) {
    let preference = ThemePreference::default();
    let mode = preference.resolve(cx.window_appearance());
    cx.set_global(Theme { preference, mode });
    *gpui_base::Theme::global_mut(cx) = base_theme(mode);
}

/// The one path that changes the mode: our theme, then `gpui-base`'s, then a redraw.
fn set_mode(mode: Mode, cx: &mut App) {
    if cx.global::<Theme>().mode == mode {
        return;
    }
    cx.global_mut::<Theme>().mode = mode;
    *gpui_base::Theme::global_mut(cx) = base_theme(mode);
    cx.refresh_windows();
}

/// Projects a mode onto `gpui-base`'s theme, so its behaviour-layer defaults (scrollbars,
/// resizable handles, text selection) use our tokens.
pub(crate) fn base_theme(mode: Mode) -> gpui_base::Theme {
    let s = mode.semantic();
    let mut theme = gpui_base::Theme::default();
    theme.appearance = mode.pick(
        gpui_base::ThemeAppearance::Light,
        gpui_base::ThemeAppearance::Dark,
    );

    let colors = &mut theme.tokens.colors;
    colors.background = s.background;
    colors.foreground = s.foreground;
    colors.surface = s.card;
    colors.surface_foreground = s.card_foreground;
    colors.primary = s.primary;
    colors.primary_foreground = s.primary_foreground;
    colors.secondary = s.secondary;
    colors.secondary_foreground = s.secondary_foreground;
    colors.muted = s.muted;
    colors.muted_foreground = s.muted_foreground;
    colors.accent = s.accent;
    colors.accent_foreground = s.accent_foreground;
    colors.destructive = s.destructive;
    // core-react has no destructive-foreground token; its destructive button text is white.
    colors.destructive_foreground = primitive::color::WHITE;
    colors.border = s.border;
    colors.input = s.input;
    colors.ring = s.ring;

    let radius = &mut theme.tokens.radius;
    radius.none = px(0.);
    radius.sm = s.radius_sm;
    radius.md = s.radius_md;
    radius.lg = s.radius_lg;
    radius.xl = s.radius_xl;
    radius.full = primitive::border_radius::FULL;

    let typography = &mut theme.tokens.typography;
    typography.sans = primitive::font_family::SANS.family.into();
    typography.mono = primitive::font_family::MONO.family.into();
    theme
}

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;

    use super::*;

    mod mode {
        use super::*;

        #[test]
        fn should_pick_the_value_for_its_mode() {
            assert_eq!(Mode::Light.pick(1, 2), 1);
            assert_eq!(Mode::Dark.pick(1, 2), 2);
        }

        #[test]
        fn should_follow_dark_and_vibrant_dark_appearances() {
            assert_eq!(
                Mode::from_appearance(WindowAppearance::VibrantDark),
                Mode::Dark
            );
            assert_eq!(
                Mode::from_appearance(WindowAppearance::VibrantLight),
                Mode::Light
            );
        }
    }

    mod preference {
        use super::*;

        #[test]
        fn should_ignore_the_appearance_unless_system() {
            assert_eq!(
                ThemePreference::Light.resolve(WindowAppearance::Dark),
                Mode::Light
            );
            assert_eq!(
                ThemePreference::Dark.resolve(WindowAppearance::Light),
                Mode::Dark
            );
            assert_eq!(
                ThemePreference::System.resolve(WindowAppearance::Dark),
                Mode::Dark
            );
        }
    }

    mod theme {
        use super::*;

        fn base(cx: &TestAppContext) -> gpui_base::Theme {
            cx.read(gpui_base::Theme::global)
        }

        #[gpui::test]
        fn should_start_on_system_and_project_onto_gpui_base(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let theme = cx.read(|cx| *cx.theme());
            assert_eq!(theme.preference(), ThemePreference::System);
            assert_eq!(theme.mode(), Mode::Light);
            assert_eq!(
                base(cx).tokens.colors.background,
                Semantic::LIGHT.background
            );
        }

        #[gpui::test]
        fn should_switch_both_themes_when_the_preference_changes(cx: &mut TestAppContext) {
            cx.update(crate::init);
            cx.update(|cx| Theme::set_preference(ThemePreference::Dark, cx));
            assert_eq!(cx.read(|cx| cx.theme().mode()), Mode::Dark);
            let base = base(cx);
            assert_eq!(base.appearance, gpui_base::ThemeAppearance::Dark);
            assert_eq!(base.tokens.colors.background, Semantic::DARK.background);
            assert_eq!(base.tokens.radius.lg, Semantic::DARK.radius_lg);
        }

        #[gpui::test]
        fn should_follow_the_os_appearance_on_system(cx: &mut TestAppContext) {
            cx.update(crate::init);
            cx.update(|cx| Theme::appearance_changed(WindowAppearance::Dark, cx));
            assert_eq!(cx.read(|cx| cx.theme().mode()), Mode::Dark);
        }

        #[gpui::test]
        fn should_not_follow_the_os_appearance_on_an_explicit_preference(cx: &mut TestAppContext) {
            cx.update(crate::init);
            cx.update(|cx| Theme::set_preference(ThemePreference::Light, cx));
            cx.update(|cx| Theme::appearance_changed(WindowAppearance::Dark, cx));
            assert_eq!(cx.read(|cx| cx.theme().mode()), Mode::Light);
        }

        #[gpui::test]
        fn should_make_the_focus_ring_half_opacity_ring(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let ring = cx.read(|cx| cx.theme().focus_ring());
            assert_eq!(ring, Semantic::LIGHT.ring.opacity(0.5));
        }
    }
}
