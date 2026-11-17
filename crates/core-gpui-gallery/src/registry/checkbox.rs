//! `Checkbox`: Penpot properties `checked` × `aria-invalid` × `disabled` × `indeterminate`.
//! Penpot has one `indeterminate=true` variant (unchecked, enabled, valid); the registry
//! renders it with every other combination, as core-react does.

use brrritscold_core_gpui::Checkbox;
use gpui::IntoElement as _;

use super::support::{BOOLS, value};
use super::{Entry, Variant};

/// The story a combination is captured from. The stories' controls are uncontrolled, so a
/// `checked` arg can't uncheck one a play function checked: an unchecked box comes from the
/// Checked story (whose play function unchecks it) or the Disabled one (which clicks nothing),
/// a checked one from the Default story (whose play function checks it). An indeterminate one
/// comes from the Indeterminate story, which clicks nothing.
fn story(checked: bool, disabled: bool, indeterminate: bool) -> &'static str {
    match (checked, disabled, indeterminate) {
        (_, _, true) => "components-checkbox--indeterminate",
        (true, _, false) => "components-checkbox--default",
        (false, false, false) => "components-checkbox--checked",
        (false, true, false) => "components-checkbox--disabled",
    }
}

pub(super) fn entry() -> Entry {
    Entry {
        name: "Checkbox",
        properties: &["checked", "aria-invalid", "disabled", "indeterminate"],
        variants: || {
            BOOLS
                .into_iter()
                .flat_map(|checked| BOOLS.map(move |invalid| (checked, invalid)))
                .flat_map(|(checked, invalid)| {
                    BOOLS.map(move |disabled| (checked, invalid, disabled))
                })
                .flat_map(|(checked, invalid, disabled)| {
                    BOOLS.map(move |indeterminate| (checked, invalid, disabled, indeterminate))
                })
                .map(|(checked, invalid, disabled, indeterminate)| Variant {
                    props: vec![
                        ("checked", value(checked)),
                        ("aria-invalid", value(invalid)),
                        ("disabled", value(disabled)),
                        ("indeterminate", value(indeterminate)),
                    ],
                    story: Some(story(checked, disabled, indeterminate)),
                    render: Box::new(move |id| {
                        Checkbox::new(id)
                            .checked(checked)
                            .indeterminate(indeterminate)
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
