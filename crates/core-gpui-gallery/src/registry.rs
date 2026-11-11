//! The registry: one [`Entry`] per core-gpui component, in Penpot's vocabulary.
//!
//! An entry names the Penpot component and its variant properties, and lists every variant
//! combination with the real builder call that renders it. The gallery's pages, the
//! captures and the drift manifest ([`crate::manifest`]) are all generated from this list, so what the
//! gallery shows is what gets checked.
//!
//! Sample content (a Button's `Button` label, its `Plus` icon) mirrors the Penpot main
//! component, so the core-react, GPUI and Penpot columns of a comparison show the same thing.
//! Where the Penpot main component and the Storybook story a variant is captured from
//! disagree (an Alert's action, a Table's columns), the entry follows the story: core-react is
//! the comparison's reference. Its module says what Penpot shows instead.
//!
//! A compound component's parts that are library components of their own (`CardHeader`) are
//! entries of their own, named as their exports. Code-only parts (`AvatarGroup`,
//! `TableHeader`, `TableBody`) are not entries: the library has no twin to check them against.

mod alert;
mod avatar;
mod badge;
mod breadcrumb;
mod button;
mod card;
mod checkbox;
mod field;
mod input;
mod label;
mod pagination;
mod radio_group;
mod separator;
mod skeleton;
mod support;
mod switch;
mod table;
mod textarea;
mod toggle;
mod toggle_group;

use gpui::{AnyElement, ElementId, Pixels, px};

/// The width a full-width story root fills in the published Storybook: its 640px capture
/// viewport less the padded layout's 1rem each side. Entries for `w-full` components render
/// at it, so their captures line up with the core-react column.
const STORY_WIDTH: Pixels = px(608.);

/// One component in the registry.
pub struct Entry {
    /// The Penpot component name, which is also the core-gpui export (`Button`).
    pub name: &'static str,
    /// The Penpot variant property names, in the order every [`Variant::props`] lists them.
    pub properties: &'static [&'static str],
    pub(crate) variants: fn() -> Vec<Variant>,
}

impl Entry {
    /// Every variant combination, in registry order.
    pub fn variants(&self) -> Vec<Variant> {
        (self.variants)()
    }
}

/// One variant combination of a component.
pub struct Variant {
    /// `(Penpot property, Penpot value)` pairs, one per [`Entry::properties`], in that order.
    pub props: Vec<(&'static str, &'static str)>,
    /// The id of core-react's Storybook story this variant is captured from, with
    /// [`props`](Self::props) as its args. `None` when no story can show it.
    pub story: Option<&'static str>,
    pub(crate) render: Box<dyn Fn(ElementId) -> AnyElement>,
}

impl Variant {
    /// Renders this combination with its sample content. `id` keys the element's state.
    pub fn render(&self, id: impl Into<ElementId>) -> AnyElement {
        (self.render)(id.into())
    }

    /// The values, joined for a cell label: `outline / icon-sm`.
    pub fn label(&self) -> String {
        let values: Vec<&str> = self.props.iter().map(|(_, value)| *value).collect();
        values.join(" / ")
    }

    /// A file-name-safe key: `variant-outline--size-icon-sm`, or `default` for a component
    /// with no variant properties.
    pub fn slug(&self) -> String {
        if self.props.is_empty() {
            return "default".into();
        }
        let pairs: Vec<String> = self
            .props
            .iter()
            .map(|(property, value)| format!("{property}-{value}"))
            .collect();
        pairs.join("--")
    }
}

/// The one variant of a component with no variant properties.
fn one(story: Option<&'static str>, render: impl Fn() -> AnyElement + 'static) -> Vec<Variant> {
    vec![Variant {
        props: Vec::new(),
        story,
        render: Box::new(move |_| render()),
    }]
}

/// Every component, in sidebar order: each component, then its parts.
pub fn registry() -> Vec<Entry> {
    let mut entries = vec![
        button::entry(),
        badge::entry(),
        label::entry(),
        separator::entry(),
        skeleton::entry(),
    ];
    entries.extend(card::entries());
    entries.extend(alert::entries());
    entries.extend(avatar::entries());
    entries.extend(breadcrumb::entries());
    entries.extend(table::entries());
    entries.extend(pagination::entries());
    entries.extend([
        input::entry(),
        textarea::entry(),
        checkbox::entry(),
        switch::entry(),
        toggle::entry(),
    ]);
    entries.extend(radio_group::entries());
    entries.extend(toggle_group::entries());
    entries.extend(field::entries());
    entries
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    mod registry {
        use brrritscold_core_gpui::{Root, Theme, ThemePreference};
        use gpui::{
            AppContext as _, Context, IntoElement, ParentElement as _, Render, Styled as _,
            TestAppContext, VisualTestContext, Window, div,
        };

        use super::*;

        #[test]
        fn should_list_every_property_on_every_variant_in_order() {
            for entry in registry() {
                for variant in entry.variants() {
                    let properties: Vec<&str> = variant.props.iter().map(|(p, _)| *p).collect();
                    assert_eq!(properties, entry.properties, "{}", entry.name);
                }
            }
        }

        #[test]
        fn should_list_each_combination_once() {
            for entry in registry() {
                let variants = entry.variants();
                let slugs: HashSet<String> = variants.iter().map(Variant::slug).collect();
                assert_eq!(slugs.len(), variants.len(), "{}", entry.name);
            }
        }

        /// The unit tier's only rendering check: every builder call runs and lays out, in
        /// both themes. It looks at no pixels; captures are for people.
        #[gpui::test]
        fn should_render_every_variant_in_both_themes(cx: &mut TestAppContext) {
            cx.update(brrritscold_core_gpui::init);
            let window = cx.add_window(|window, cx| {
                let board = cx.new(|_| Board);
                Root::new(board, window, cx)
            });
            let cx = VisualTestContext::from_window(*window, cx).into_mut();
            for preference in [ThemePreference::Light, ThemePreference::Dark] {
                cx.update(|window, cx| {
                    Theme::set_preference(preference, cx);
                    window.draw(cx).clear(cx);
                });
            }
        }

        struct Board;

        impl Render for Board {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let cells = registry().into_iter().flat_map(|entry| {
                    entry
                        .variants()
                        .into_iter()
                        .enumerate()
                        .map(move |(index, variant)| variant.render((entry.name, index)))
                });
                div().flex().flex_wrap().children(cells)
            }
        }

        #[test]
        fn should_name_each_component_once() {
            let entries = registry();
            let names: HashSet<&str> = entries.iter().map(|e| e.name).collect();
            assert_eq!(names.len(), entries.len());
        }
    }

    mod variant {
        use gpui::IntoElement as _;

        use super::*;

        fn outline_icon_sm() -> Variant {
            Variant {
                props: vec![("variant", "outline"), ("size", "icon-sm")],
                story: None,
                render: Box::new(|_| gpui::Empty.into_any_element()),
            }
        }

        #[test]
        fn should_label_by_values() {
            assert_eq!(outline_icon_sm().label(), "outline / icon-sm");
        }

        #[test]
        fn should_slug_by_property_and_value() {
            assert_eq!(outline_icon_sm().slug(), "variant-outline--size-icon-sm");
        }

        #[test]
        fn should_slug_a_component_without_properties_as_default() {
            let skeleton = &one(None, || gpui::Empty.into_any_element())[0];
            assert_eq!(skeleton.slug(), "default");
        }
    }
}
