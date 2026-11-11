//! `Switch`: Penpot properties `size` × `checked` × `disabled`.

use brrritscold_core_gpui::{Switch, SwitchSize};
use gpui::IntoElement as _;
use strum::IntoEnumIterator as _;

use super::support::{BOOLS, value};
use super::{Entry, Variant};

/// The story a combination is captured from. The stories' controls are uncontrolled, so a
/// `checked` arg can't turn off one a play function turned on: a switch that is off comes from
/// the Checked story (whose play function turns it off) or the Disabled one (which clicks
/// nothing), one that is on from the Default story (whose play function turns it on).
fn story(checked: bool, disabled: bool) -> &'static str {
    match (checked, disabled) {
        (true, _) => "components-switch--default",
        (false, false) => "components-switch--checked",
        (false, true) => "components-switch--disabled",
    }
}

pub(super) fn entry() -> Entry {
    Entry {
        name: "Switch",
        properties: &["size", "checked", "disabled"],
        variants: || {
            SwitchSize::iter()
                .flat_map(|size| BOOLS.map(move |checked| (size, checked)))
                .flat_map(|(size, checked)| BOOLS.map(move |disabled| (size, checked, disabled)))
                .map(|(size, checked, disabled)| Variant {
                    props: vec![
                        ("size", size.into()),
                        ("checked", value(checked)),
                        ("disabled", value(disabled)),
                    ],
                    story: Some(story(checked, disabled)),
                    render: Box::new(move |id| {
                        Switch::new(id)
                            .size(size)
                            .checked(checked)
                            .disabled(disabled)
                            .accessibility_label("Airplane mode")
                            .into_any_element()
                    }),
                })
                .collect()
        },
    }
}
