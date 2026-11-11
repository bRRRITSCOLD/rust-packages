//! `Button`: Penpot properties `variant` × `size`.

use brrritscold_core_gpui::{Button, ButtonSize, ButtonVariant, IconName};
use gpui::IntoElement as _;
use strum::IntoEnumIterator as _;

use super::{Entry, Variant};

pub(super) fn entry() -> Entry {
    Entry {
        name: "Button",
        properties: &["variant", "size"],
        variants: || {
            ButtonVariant::iter()
                .flat_map(|variant| ButtonSize::iter().map(move |size| (variant, size)))
                .map(|(variant, size)| Variant {
                    props: vec![("variant", variant.into()), ("size", size.into())],
                    // core-react's icon stories render `<PlusIcon />` as children; the rest render "Button".
                    story: Some(if size.is_icon() {
                        "components-button--icon"
                    } else {
                        "components-button--default"
                    }),
                    render: Box::new(move |id| {
                        let button = Button::new(id).variant(variant).size(size);
                        // Penpot's main components: an `Icon` part (`icon/Plus`) on icon sizes, a
                        // `Label` reading "Button" on the rest.
                        if size.is_icon() {
                            button.icon(IconName::Plus).into_any_element()
                        } else {
                            button.label("Button").into_any_element()
                        }
                    }),
                })
                .collect()
        },
    }
}
