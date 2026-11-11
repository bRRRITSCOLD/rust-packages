//! Avatar: core-react's `avatar.tsx`, plain styled elements over GPUI's `img`.
//!
//! Base UI's Avatar shows the fallback until the image has loaded, and keeps it when the image
//! fails; GPUI's `img` does the same through `with_loading` and `with_fallback`. An avatar's
//! `size` reaches its fallback and badge (`group-data-[size=*]/avatar:` in core-react) because
//! [`Avatar`] takes its parts as typed builders and hands the size down; an [`AvatarGroup`]
//! hands its avatars' size to its count the same way.
//!
//! The ring (`after:border after:border-border`) is blended in core-react
//! (`mix-blend-darken`, `dark:mix-blend-lighten`), which GPUI can't do. The blend hides the
//! ring wherever the image is darker (light mode) or lighter (dark mode) than `border`, which
//! is nearly every photo, and leaves it plain over the `muted` fallback. So core-gpui rings
//! the fallback only: over an image whose edge is paler than `border` (in light mode), core-react
//! shows a ring core-gpui doesn't. Not mirrored either: the badge's `bg-blend-color`.

use brrritscold_core_gpui_tokens::primitive::{font_size, space};
use gpui::{
    App, ImageSource, IntoElement, ObjectFit, ParentElement as _, Pixels, Rems, RenderOnce,
    SharedString, StyleRefinement, Styled, StyledImage as _, Window, div, img,
    prelude::FluentBuilder as _,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use crate::icons::{Icon, IconName};
use crate::theme::ActiveTheme as _;

/// Penpot variant property `size`. Cases are its values in PascalCase; [`Display`] gives the
/// Penpot value back (`AvatarSize::Lg` → `lg`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum AvatarSize {
    /// `default`: 32px.
    #[default]
    Default,
    /// `sm`: 24px.
    Sm,
    /// `lg`: 40px.
    Lg,
}

impl AvatarSize {
    /// `size-8`; `data-[size=sm]:size-6`, `data-[size=lg]:size-10`. A group count follows it
    /// too (`group-has-data-[size=*]/avatar-group:size-*`).
    fn side(self) -> Pixels {
        match self {
            AvatarSize::Default => space::S8,
            AvatarSize::Sm => space::S6,
            AvatarSize::Lg => space::S10,
        }
    }

    /// The fallback's `text-sm`; `group-data-[size=sm]/avatar:text-xs`.
    fn fallback_text(self) -> Rems {
        match self {
            AvatarSize::Sm => font_size::XS,
            AvatarSize::Default | AvatarSize::Lg => font_size::SM,
        }
    }

    /// The badge's side and its icon's: `size-2` with the icon hidden at `sm`, `size-2.5` and a
    /// `size-2` icon at `default`, `size-3` and a `size-2` icon at `lg`.
    fn badge(self) -> (Pixels, Option<Pixels>) {
        match self {
            AvatarSize::Sm => (space::S2, None),
            AvatarSize::Default => (space::S2_5, Some(space::S2)),
            AvatarSize::Lg => (space::S3, Some(space::S2)),
        }
    }

    /// A group count's icon: `[&>svg]:size-4`; `size-5` beside `lg` avatars, `size-3` beside `sm`.
    fn count_icon(self) -> Pixels {
        match self {
            AvatarSize::Default => space::S4,
            AvatarSize::Sm => space::S3,
            AvatarSize::Lg => space::S5,
        }
    }
}

/// An avatar, styled like core-react's `Avatar`. Its parts are builders:
///
/// ```no_run
/// use brrritscold_core_gpui::{Avatar, AvatarFallback, AvatarImage, AvatarSize};
///
/// let ada = Avatar::new()
///     .size(AvatarSize::Lg)
///     .image(AvatarImage::new("https://example.com/ada.png"))
///     .fallback(AvatarFallback::new("AL"));
/// ```
#[derive(IntoElement)]
pub struct Avatar {
    size: AvatarSize,
    image: Option<AvatarImage>,
    fallback: Option<AvatarFallback>,
    badge: Option<AvatarBadge>,
    /// `*:data-[slot=avatar]:ring-2 *:data-[slot=avatar]:ring-background`, set by a group.
    in_group: bool,
    style: StyleRefinement,
}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}

impl Avatar {
    /// A `default` avatar with no parts.
    pub fn new() -> Self {
        Self {
            size: AvatarSize::default(),
            image: None,
            fallback: None,
            badge: None,
            in_group: false,
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `size`.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    /// The `Image` part. The fallback shows while it loads, and if it fails.
    pub fn image(mut self, image: AvatarImage) -> Self {
        self.image = Some(image);
        self
    }

    /// The `Fallback` part: initials on `muted`.
    pub fn fallback(mut self, fallback: AvatarFallback) -> Self {
        self.fallback = Some(fallback);
        self
    }

    /// The `Badge` part, on the bottom-right edge.
    pub fn badge(mut self, badge: AvatarBadge) -> Self {
        self.badge = Some(badge);
        self
    }
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let size = self.size;
        let fallback = self.fallback.map(|fallback| fallback.in_avatar(size));
        let content = match (self.image, fallback) {
            (Some(image), fallback) => Some(image.fallback(fallback).into_any_element()),
            (None, Some(fallback)) => Some(fallback.into_any_element()),
            (None, None) => None,
        };
        // relative flex size-8 shrink-0 rounded-full select-none data-[size=lg]:size-10
        // data-[size=sm]:size-6
        div()
            .relative()
            .flex()
            .size(size.side())
            .flex_shrink_0()
            .rounded_full()
            .when(self.in_group, |d| {
                d.shadow(vec![super::ring(2., s.background)])
            })
            .children(content)
            .children(self.badge.map(|badge| badge.avatar_size(size)))
            .refine_style(&self.style)
    }
}

/// The avatar's `Image` part: a round, cover-fitted image.
#[derive(IntoElement)]
pub struct AvatarImage {
    source: ImageSource,
    fallback: Option<AvatarFallback>,
    style: StyleRefinement,
}

impl AvatarImage {
    /// An image from `source`: a URI, a path, or image bytes.
    pub fn new(source: impl Into<ImageSource>) -> Self {
        Self {
            source: source.into(),
            fallback: None,
            style: StyleRefinement::default(),
        }
    }

    fn fallback(mut self, fallback: Option<AvatarFallback>) -> Self {
        self.fallback = fallback;
        self
    }
}

impl RenderOnce for AvatarImage {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // aspect-square size-full rounded-full object-cover
        let image = img(self.source)
            .size_full()
            .rounded_full()
            .object_fit(ObjectFit::Cover)
            .refine_style(&self.style);
        match self.fallback {
            Some(fallback) => {
                let loading = fallback.clone();
                image
                    .with_loading(move || loading.clone().into_any_element())
                    .with_fallback(move || fallback.clone().into_any_element())
            }
            None => image,
        }
    }
}

/// The avatar's `Fallback` part: initials, centred on `muted`.
#[derive(IntoElement, Clone)]
pub struct AvatarFallback {
    size: AvatarSize,
    /// Set by an avatar, which rings its fallback (see the module docs).
    ringed: bool,
    text: SharedString,
    style: StyleRefinement,
}

impl AvatarFallback {
    /// A fallback reading `text`, usually initials.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            size: AvatarSize::default(),
            ringed: false,
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }

    fn in_avatar(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self.ringed = true;
        self
    }
}

impl RenderOnce for AvatarFallback {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // flex size-full items-center justify-center rounded-full bg-muted text-sm
        // text-muted-foreground group-data-[size=sm]/avatar:text-xs
        div()
            .relative()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .rounded_full()
            .bg(s.muted)
            .text_size(self.size.fallback_text())
            .text_color(s.muted_foreground)
            .child(self.text)
            // The avatar's after:absolute after:inset-0 after:rounded-full after:border
            // after:border-border
            .when(self.ringed, |d| {
                d.child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded_full()
                        .border_1()
                        .border_color(s.border),
                )
            })
            .refine_style(&self.style)
    }
}

/// The avatar's `Badge` part: a dot on the bottom-right edge, optionally holding an icon.
#[derive(IntoElement)]
pub struct AvatarBadge {
    size: AvatarSize,
    icon: Option<IconName>,
    style: StyleRefinement,
}

impl Default for AvatarBadge {
    fn default() -> Self {
        Self::new()
    }
}

impl AvatarBadge {
    /// An empty badge.
    pub fn new() -> Self {
        Self {
            size: AvatarSize::default(),
            icon: None,
            style: StyleRefinement::default(),
        }
    }

    /// An icon inside the badge; hidden on an `sm` avatar.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    fn avatar_size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for AvatarBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let (side, icon_side) = self.size.badge();
        // absolute right-0 bottom-0 z-10 inline-flex items-center justify-center rounded-full
        // bg-primary text-primary-foreground ring-2 ring-background
        div()
            .absolute()
            .right_0()
            .bottom_0()
            .flex()
            .items_center()
            .justify_center()
            .size(side)
            .rounded_full()
            .bg(s.primary)
            .text_color(s.primary_foreground)
            .shadow(vec![super::ring(2., s.background)])
            .when_some(self.icon.zip(icon_side), |d, (icon, icon_side)| {
                d.child(Icon::new(icon).size(icon_side))
            })
            .refine_style(&self.style)
    }
}

/// A row of overlapping avatars ending in a count. Code only, as in core-react: the Penpot
/// library has no `AvatarGroup`, only its `GroupCount` part.
///
/// ```no_run
/// use brrritscold_core_gpui::{Avatar, AvatarFallback, AvatarGroup, AvatarGroupCount, IconName};
///
/// let team = AvatarGroup::new()
///     .avatar(Avatar::new().fallback(AvatarFallback::new("A")))
///     .avatar(Avatar::new().fallback(AvatarFallback::new("G")))
///     .count(AvatarGroupCount::new().icon(IconName::Plus));
/// ```
#[derive(IntoElement)]
pub struct AvatarGroup {
    avatars: Vec<Avatar>,
    count: Option<AvatarGroupCount>,
    style: StyleRefinement,
}

impl Default for AvatarGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl AvatarGroup {
    /// An empty group.
    pub fn new() -> Self {
        Self {
            avatars: Vec::new(),
            count: None,
            style: StyleRefinement::default(),
        }
    }

    /// Adds an avatar.
    pub fn avatar(mut self, avatar: Avatar) -> Self {
        self.avatars.push(avatar);
        self
    }

    /// The count after the avatars.
    pub fn count(mut self, count: AvatarGroupCount) -> Self {
        self.count = Some(count);
        self
    }
}

impl RenderOnce for AvatarGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // group-has-data-[size=lg] / group-has-data-[size=sm]: the count follows its avatars.
        let sizes: Vec<AvatarSize> = self.avatars.iter().map(|avatar| avatar.size).collect();
        let count_size = [AvatarSize::Lg, AvatarSize::Sm]
            .into_iter()
            .find(|size| sizes.contains(size))
            .unwrap_or_default();
        let avatars = self.avatars.into_iter().map(|mut avatar| {
            avatar.in_group = true;
            avatar.into_any_element()
        });
        let count = self
            .count
            .map(|count| count.group_size(count_size).into_any_element());
        // flex -space-x-2: every item but the first overlaps the one before by 8px.
        div()
            .flex()
            .children(
                avatars
                    .chain(count)
                    .enumerate()
                    .map(|(index, item)| div().when(index > 0, |d| d.ml(-space::S2)).child(item)),
            )
            .refine_style(&self.style)
    }
}

/// The avatar group's `GroupCount` part: an icon or a short label on `muted`.
#[derive(IntoElement)]
pub struct AvatarGroupCount {
    size: AvatarSize,
    icon: Option<IconName>,
    label: Option<SharedString>,
    style: StyleRefinement,
}

impl Default for AvatarGroupCount {
    fn default() -> Self {
        Self::new()
    }
}

impl AvatarGroupCount {
    /// An empty count, sized for `default` avatars until a group renders it.
    pub fn new() -> Self {
        Self {
            size: AvatarSize::default(),
            icon: None,
            label: None,
            style: StyleRefinement::default(),
        }
    }

    /// An icon, such as `Plus`.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A short label, such as `+3`.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn group_size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for AvatarGroupCount {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // relative flex size-8 shrink-0 items-center justify-center rounded-full bg-muted
        // text-sm text-muted-foreground ring-2 ring-background
        div()
            .relative()
            .flex()
            .size(self.size.side())
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .rounded_full()
            .bg(s.muted)
            .text_size(font_size::SM)
            .text_color(s.muted_foreground)
            .shadow(vec![super::ring(2., s.background)])
            .when_some(self.icon, |d, icon| {
                d.child(Icon::new(icon).size(self.size.count_icon()))
            })
            .when_some(self.label, |d, label| d.child(label))
            .refine_style(&self.style)
    }
}

impl_styled!(
    Avatar,
    AvatarImage,
    AvatarFallback,
    AvatarBadge,
    AvatarGroup,
    AvatarGroupCount
);

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, px};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::render;

    mod size {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&'static str> = AvatarSize::iter().map(Into::into).collect();
            assert_eq!(values, ["default", "sm", "lg"]);
        }

        #[gpui::test]
        fn should_be_a_32_24_or_40px_circle(cx: &mut TestAppContext) {
            cx.update(crate::init);
            for (size, side) in [
                (AvatarSize::Default, 32.),
                (AvatarSize::Sm, 24.),
                (AvatarSize::Lg, 40.),
            ] {
                let (_, bounds) = render(cx, move || {
                    Avatar::new().size(size).fallback(AvatarFallback::new("AL"))
                });
                assert_eq!(bounds.size.width, px(side), "{size}");
                assert_eq!(bounds.size.height, px(side), "{size}");
            }
        }

        #[test]
        fn should_hide_the_badge_icon_only_at_sm() {
            assert_eq!(AvatarSize::Sm.badge(), (px(8.), None));
            assert_eq!(AvatarSize::Default.badge(), (px(10.), Some(px(8.))));
            assert_eq!(AvatarSize::Lg.badge(), (px(12.), Some(px(8.))));
        }
    }

    mod group {
        use super::*;

        #[gpui::test]
        fn should_overlap_each_avatar_by_8px(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || {
                AvatarGroup::new()
                    .avatar(Avatar::new().fallback(AvatarFallback::new("A")))
                    .avatar(Avatar::new().fallback(AvatarFallback::new("G")))
                    .count(AvatarGroupCount::new().icon(IconName::Plus))
            });
            assert_eq!(bounds.size.width, px(3. * 32. - 2. * 8.));
            assert_eq!(bounds.size.height, px(32.));
        }

        #[gpui::test]
        fn should_size_the_count_like_large_avatars(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, bounds) = render(cx, || {
                AvatarGroup::new()
                    .avatar(Avatar::new().size(AvatarSize::Lg))
                    .count(AvatarGroupCount::new().label("+3"))
            });
            assert_eq!(bounds.size.height, px(40.));
        }
    }
}
