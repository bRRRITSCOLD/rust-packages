//! `Toggle`: Penpot properties `variant` × `size` × `pressed` × `disabled`, with the story's
//! `LetterB` icon. Penpot's main component reads "Label" instead.

use brrritscold_core_gpui::{IconName, Toggle, ToggleSize, ToggleVariant};
use gpui::IntoElement as _;
use strum::IntoEnumIterator as _;

use super::support::{BOOLS, value};
use super::{Entry, Variant};

/// The story a combination is captured from. The stories' toggles are uncontrolled, so a
/// `pressed` arg can't release one the Default story's play function pressed: a pressed
/// toggle comes from the Pressed story, a released one from the Small story, whose play
/// function clicks nothing (the `size` arg overrides its own).
fn story(pressed: bool) -> &'static str {
    if pressed {
        "components-toggle--pressed"
    } else {
        "components-toggle--small"
    }
}

pub(super) fn entry() -> Entry {
    Entry {
        name: "Toggle",
        properties: &["variant", "size", "pressed", "disabled"],
        variants: || {
            ToggleVariant::iter()
                .flat_map(|variant| ToggleSize::iter().map(move |size| (variant, size)))
                .flat_map(|(variant, size)| BOOLS.map(move |pressed| (variant, size, pressed)))
                .flat_map(|(variant, size, pressed)| {
                    BOOLS.map(move |disabled| (variant, size, pressed, disabled))
                })
                .map(|(variant, size, pressed, disabled)| Variant {
                    props: vec![
                        ("variant", variant.into()),
                        ("size", size.into()),
                        ("pressed", value(pressed)),
                        ("disabled", value(disabled)),
                    ],
                    story: Some(story(pressed)),
                    render: Box::new(move |id| {
                        Toggle::new(id)
                            .variant(variant)
                            .size(size)
                            .pressed(pressed)
                            .disabled(disabled)
                            .icon(IconName::LetterB)
                            .accessibility_label("Bold")
                            .into_any_element()
                    }),
                })
                .collect()
        },
    }
}
