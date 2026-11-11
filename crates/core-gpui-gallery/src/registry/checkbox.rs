//! `Checkbox`: Penpot properties `checked` × `aria-invalid` × `disabled`.

use brrritscold_core_gpui::Checkbox;
use gpui::IntoElement as _;

use super::support::{BOOLS, value};
use super::{Entry, Variant};

/// The story a combination is captured from. The stories' controls are uncontrolled, so a
/// `checked` arg can't uncheck one a play function checked: an unchecked box comes from the
/// Checked story (whose play function unchecks it) or the Disabled one (which clicks nothing),
/// a checked one from the Default story (whose play function checks it).
fn story(checked: bool, disabled: bool) -> &'static str {
    match (checked, disabled) {
        (true, _) => "components-checkbox--default",
        (false, false) => "components-checkbox--checked",
        (false, true) => "components-checkbox--disabled",
    }
}

pub(super) fn entry() -> Entry {
    Entry {
        name: "Checkbox",
        properties: &["checked", "aria-invalid", "disabled"],
        variants: || {
            BOOLS
                .into_iter()
                .flat_map(|checked| BOOLS.map(move |invalid| (checked, invalid)))
                .flat_map(|(checked, invalid)| {
                    BOOLS.map(move |disabled| (checked, invalid, disabled))
                })
                .map(|(checked, invalid, disabled)| Variant {
                    props: vec![
                        ("checked", value(checked)),
                        ("aria-invalid", value(invalid)),
                        ("disabled", value(disabled)),
                    ],
                    story: Some(story(checked, disabled)),
                    render: Box::new(move |id| {
                        Checkbox::new(id)
                            .checked(checked)
                            .aria_invalid(invalid)
                            .disabled(disabled)
                            .accessibility_label("Accept terms")
                            .into_any_element()
                    }),
                })
                .collect()
        },
    }
}
