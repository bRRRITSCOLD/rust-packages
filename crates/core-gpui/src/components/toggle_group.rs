//! ToggleGroup: core-react's `toggle-group.tsx` on `gpui-base`'s unstyled
//! [`gpui_base::ToggleGroup`] (the group's role and orientation) and [`crate::Toggle`], which
//! draws each item through core-react's `toggleVariants`.
//!
//! core-react hands the group's `variant`, `size`, `spacing` and orientation to its items
//! through a React context; here [`ToggleGroup`] takes its items as typed builders and hands
//! them down, with each item's place (first, last) for `spacing` 0, which joins the items.
//! Items are controlled: each shows [`ToggleGroupItem::pressed`] and reports to
//! [`ToggleGroupItem::on_change`]; the app keeps the group's value, single or multiple.
//!
//! Not mirrored:
//!
//! - Arrow keys between items: `gpui-base`'s group has no roving focus, so Tab visits each.
//! - `focus:z-10`: GPUI paints siblings in order, so a joined item's focus ring sits under
//!   the next item.

use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{
    App, Axis, ElementId, IntoElement, ParentElement as _, Pixels, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, prelude::FluentBuilder as _,
};
use gpui_base::StyledExt as _;
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use super::control::shadows;
use super::toggle::{Joined, Toggle, ToggleSize, ToggleVariant};
use crate::icons::IconName;
use crate::theme::ActiveTheme as _;

/// Penpot variant property `orientation`. Cases are its values in PascalCase; [`Display`]
/// gives the Penpot value back (`ToggleGroupOrientation::Vertical` → `vertical`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum ToggleGroupOrientation {
    /// `horizontal`: items in a row.
    #[default]
    Horizontal,
    /// `vertical`: items stacked, stretched to the widest.
    Vertical,
}

/// Penpot variant property `spacing`, the gap between items. Its values are numbers, so the
/// cases spell them; [`Display`] gives the Penpot value back (`ToggleGroupSpacing::Zero` →
/// `0`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr,
)]
pub enum ToggleGroupSpacing {
    /// `0`: no gap; the items join, sharing borders and rounding only the group's ends.
    #[strum(serialize = "0")]
    Zero,
    /// `1`: `gap-1`.
    #[strum(serialize = "1")]
    One,
    /// `2`: `gap-2`, core-react's default.
    #[default]
    #[strum(serialize = "2")]
    Two,
}

impl ToggleGroupSpacing {
    fn gap(self) -> Pixels {
        match self {
            ToggleGroupSpacing::Zero => space::S0,
            ToggleGroupSpacing::One => space::S1,
            ToggleGroupSpacing::Two => space::S2,
        }
    }
}

/// A toggle group, styled like core-react's `ToggleGroup`. One builder per Penpot variant
/// property ([`orientation`](Self::orientation), [`spacing`](Self::spacing),
/// [`variant`](Self::variant)); its items are [`item`](Self::item)s:
///
/// ```no_run
/// use brrritscold_core_gpui::{
///     IconName, ToggleGroup, ToggleGroupItem, ToggleGroupSpacing, ToggleVariant,
/// };
///
/// let alignment = "center";
/// let group = ToggleGroup::new("alignment")
///     .variant(ToggleVariant::Outline)
///     .spacing(ToggleGroupSpacing::Zero)
///     .items([
///         ("left", IconName::TextAlignStart),
///         ("center", IconName::TextAlignCenter),
///         ("right", IconName::TextAlignEnd),
///     ]
///     .map(|(value, icon)| {
///         ToggleGroupItem::new(value)
///             .icon(icon)
///             .pressed(value == alignment)
///             .on_change(move |_, _, _| println!("align {value}"))
///     }));
/// ```
#[derive(IntoElement)]
pub struct ToggleGroup {
    id: ElementId,
    orientation: ToggleGroupOrientation,
    spacing: ToggleGroupSpacing,
    variant: Option<ToggleVariant>,
    size: Option<ToggleSize>,
    disabled: bool,
    items: Vec<ToggleGroupItem>,
    style: StyleRefinement,
}

impl ToggleGroup {
    /// An empty `horizontal` group, `spacing` 2. `id` names it to assistive technology.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            orientation: ToggleGroupOrientation::default(),
            spacing: ToggleGroupSpacing::default(),
            variant: None,
            size: None,
            disabled: false,
            items: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Penpot property `orientation`.
    pub fn orientation(mut self, orientation: ToggleGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Penpot property `spacing`.
    pub fn spacing(mut self, spacing: ToggleGroupSpacing) -> Self {
        self.spacing = spacing;
        self
    }

    /// Penpot property `variant`, which every item takes over its own.
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    /// The items' `size`, which every item takes over its own.
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = Some(size);
        self
    }

    /// Disables every item.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// An `Item` part, after those already added.
    pub fn item(mut self, item: ToggleGroupItem) -> Self {
        self.items.push(item);
        self
    }

    /// Several `Item` parts, after those already added.
    pub fn items(mut self, items: impl IntoIterator<Item = ToggleGroupItem>) -> Self {
        self.items.extend(items);
        self
    }
}

impl RenderOnce for ToggleGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let vertical = self.orientation == ToggleGroupOrientation::Vertical;
        let joined = self.spacing == ToggleGroupSpacing::Zero;
        let count = self.items.len();
        let (variant, size, disabled) = (self.variant, self.size, self.disabled);
        let items = self
            .items
            .into_iter()
            .enumerate()
            .map(move |(index, item)| {
                let mut toggle = item.toggle;
                if let Some(variant) = variant {
                    toggle = toggle.variant(variant);
                }
                if let Some(size) = size {
                    toggle = toggle.size(size);
                }
                if disabled {
                    toggle = toggle.disabled(true);
                }
                if joined {
                    toggle = toggle.joined(Joined {
                        vertical,
                        first: index == 0,
                        last: index + 1 == count,
                    });
                }
                // shrink-0
                toggle.flex_shrink_0()
            });

        // group/toggle-group flex w-fit flex-row items-center rounded-md
        // data-[spacing=0]:data-[variant=outline]:shadow-xs
        // data-[orientation=vertical]:flex-col data-[orientation=vertical]:items-stretch
        gpui_base::ToggleGroup::new(self.id)
            .axis(if vertical {
                Axis::Vertical
            } else {
                Axis::Horizontal
            })
            .flex()
            .rounded(s.radius_md)
            .gap(self.spacing.gap())
            .map(|g| {
                if vertical {
                    g.flex_col().items_stretch()
                } else {
                    g.flex_row().items_center()
                }
            })
            .when(joined && variant == Some(ToggleVariant::Outline), |g| {
                g.shadow(shadows(None, true))
            })
            .children(items)
            .refine_style(&self.style)
    }
}

impl_styled!(ToggleGroup);

/// The group's `Item` part: a [`Toggle`] drawn with the group's `variant`, `size` and
/// `spacing`.
#[derive(IntoElement)]
pub struct ToggleGroupItem {
    toggle: Toggle,
}

impl ToggleGroupItem {
    /// An unpressed item with no content. `id` keys its focus state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            toggle: Toggle::new(id),
        }
    }

    /// Penpot property `variant`, used when the group sets none.
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.toggle = self.toggle.variant(variant);
        self
    }

    /// Penpot property `pressed`: filled in `muted`.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.toggle = self.toggle.pressed(pressed);
        self
    }

    /// Dims the item to 50% and makes it inert.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.toggle = self.toggle.disabled(disabled);
        self
    }

    /// The `Label` layer: the item's text.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.toggle = self.toggle.label(label);
        self
    }

    /// An icon, drawn 16px before the label.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.toggle = self.toggle.icon(icon);
        self
    }

    /// The name assistive technology reads, for an icon-only item.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.toggle = self.toggle.accessibility_label(label);
        self
    }

    /// Runs on a click or Space with the pressed value the item asks to change to.
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.toggle = self.toggle.on_change(handler);
        self
    }
}

impl RenderOnce for ToggleGroupItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.toggle
    }
}

impl Styled for ToggleGroupItem {
    fn style(&mut self) -> &mut StyleRefinement {
        self.toggle.style()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{Modifiers, TestAppContext, px};
    use strum::IntoEnumIterator as _;

    use super::*;
    use crate::components::test_support::render;

    const ALIGNMENTS: [IconName; 3] = [
        IconName::TextAlignStart,
        IconName::TextAlignCenter,
        IconName::TextAlignEnd,
    ];

    /// Three outline items `spacing` apart, one pressed value the items' handlers write.
    fn alignment(
        spacing: ToggleGroupSpacing,
        orientation: ToggleGroupOrientation,
    ) -> (impl Fn() -> ToggleGroup, Rc<Cell<Option<usize>>>) {
        let value = Rc::new(Cell::new(None));
        let make = {
            let value = value.clone();
            move || {
                ToggleGroup::new("align")
                    .variant(ToggleVariant::Outline)
                    .spacing(spacing)
                    .orientation(orientation)
                    .items(ALIGNMENTS.iter().enumerate().map(|(index, icon)| {
                        let write = value.clone();
                        ToggleGroupItem::new(index)
                            .icon(*icon)
                            .pressed(value.get() == Some(index))
                            .on_change(move |pressed, _, _| write.set(pressed.then_some(index)))
                    }))
            }
        };
        (make, value)
    }

    mod spacing {
        use super::*;

        #[test]
        fn should_name_values_as_penpot_does() {
            let values: Vec<&'static str> = ToggleGroupSpacing::iter().map(Into::into).collect();
            assert_eq!(values, ["0", "1", "2"]);
        }

        #[gpui::test]
        fn should_set_items_8px_apart_by_default(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (group, _) = alignment(ToggleGroupSpacing::Two, ToggleGroupOrientation::Horizontal);
            let (_, bounds) = render(cx, group);
            // Three outline items (1px borders around px-2.5 and a 16px icon) and two 8px gaps.
            assert_eq!(bounds.size.width, px(38. * 3. + 16.));
        }

        #[gpui::test]
        fn should_join_items_at_spacing_0(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (group, _) =
                alignment(ToggleGroupSpacing::Zero, ToggleGroupOrientation::Horizontal);
            let (_, bounds) = render(cx, group);
            // Three 36px squares (min-w-9) with no gap between them.
            assert_eq!(bounds.size.width, px(36. * 3.));
        }

        #[gpui::test]
        fn should_stack_items_when_vertical(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (group, _) = alignment(ToggleGroupSpacing::Zero, ToggleGroupOrientation::Vertical);
            let (_, bounds) = render(cx, group);
            assert_eq!(bounds.size.height, px(36. * 3.));
        }
    }

    mod press {
        use super::*;

        #[gpui::test]
        fn should_report_the_pressed_item(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (group, value) =
                alignment(ToggleGroupSpacing::Two, ToggleGroupOrientation::Horizontal);
            let (cx, bounds) = render(cx, group);
            let center = gpui::point(bounds.origin.x + px(38. + 8. + 19.), bounds.center().y);
            cx.simulate_click(center, Modifiers::default());
            assert_eq!(value.get(), Some(1));
        }
    }
}
