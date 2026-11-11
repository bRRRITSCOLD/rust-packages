//! `DatePicker` (Penpot property `disabled`) in the stories' "Due date" field. The Default
//! story's play function picks the 20th; the Disabled story keeps the 9th. Penpot's main
//! component shows the placeholder.

use brrritscold_core_gpui::{DatePicker, Field, FieldLabel};
use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{IntoElement as _, ParentElement as _, Styled as _};

use super::calendar::november;
use super::support::{BOOLS, value};
use super::{Entry, Variant};

pub(super) fn entry() -> Entry {
    Entry {
        name: "DatePicker",
        properties: &["disabled"],
        variants: || {
            BOOLS
                .into_iter()
                .map(|disabled| Variant {
                    props: vec![("disabled", value(disabled))],
                    story: Some(if disabled {
                        "components-datepicker--disabled"
                    } else {
                        "components-datepicker--default"
                    }),
                    // <Field className="w-72"><FieldLabel>Due date</FieldLabel><DatePicker /></Field>
                    render: Box::new(move |id| {
                        Field::new()
                            .w(space::S72)
                            .child(FieldLabel::new().child("Due date"))
                            .child(
                                DatePicker::new(id)
                                    .default_value(november(if disabled { 9 } else { 20 }))
                                    .disabled(disabled),
                            )
                            .into_any_element()
                    }),
                })
                .collect()
        },
    }
}
