//! `Field` (Penpot property `orientation`) and its parts, with the stories' content: a
//! username field, a marketing-email switch, a notification set and a separated pair.
//! Penpot's main `Field` shows an input in every orientation; the horizontal story, which the
//! entry follows, shows a switch beside its label and description.

use brrritscold_core_gpui::{
    Checkbox, Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel,
    FieldLegend, FieldLegendVariant, FieldOrientation, FieldSeparator, FieldSet, FieldTitle, Input,
    InputState, Switch,
};
use brrritscold_core_gpui_tokens::primitive::{size, space};
use gpui::{AnyElement, Context, ElementId, IntoElement, ParentElement as _, Styled as _, Window};
use strum::IntoEnumIterator as _;

use super::support::{Stateful, child};
use super::{Entry, Variant, one};

/// The story's input after its play function typed `ada` into it.
fn ada(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
    InputState::new(window, cx)
        .placeholder("ada")
        .default_value("ada")
}

/// An empty input.
fn empty(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
    InputState::new(window, cx)
}

fn input(
    id: impl Into<ElementId>,
    init: fn(&mut Window, &mut Context<InputState>) -> InputState,
) -> AnyElement {
    Stateful::new(id, init, |state| Input::new(state).into_any_element()).into_any_element()
}

fn label(text: &'static str) -> FieldLabel {
    FieldLabel::new().child(text)
}

/// The stories' `FieldGroup className="w-96"`.
fn group() -> FieldGroup {
    FieldGroup::new().w(size::SM)
}

/// A part on its own, at the stories' width.
fn at_story_width(part: impl IntoElement) -> AnyElement {
    gpui::div().w(size::SM).child(part).into_any_element()
}

fn field(id: ElementId, orientation: FieldOrientation) -> Field {
    let field = Field::new().orientation(orientation);
    match orientation {
        // The story's play function turns the switch on.
        FieldOrientation::Horizontal => field.child(Switch::new(id).checked(true)).content(
            FieldContent::new()
                .child(label("Marketing emails"))
                .child(FieldDescription::new("Product news, at most once a month.")),
        ),
        FieldOrientation::Vertical | FieldOrientation::Responsive => field
            .child(label("Username"))
            .child(input(id, ada))
            .child(FieldDescription::new("Choose a unique username.")),
    }
}

/// The Legend story's set: two checkbox fields under "Notifications", both checked once its
/// play function has clicked "SMS".
fn notifications(id: ElementId) -> FieldSet {
    let choice = |index: usize, text: &'static str, checked: bool| {
        Field::new()
            .orientation(FieldOrientation::Horizontal)
            .child(Checkbox::new(child(&id, index)).checked(checked))
            .child(label(text))
    };
    FieldSet::new()
        .w(size::SM)
        .legend(FieldLegend::new("Notifications"))
        .description(FieldDescription::new("Where we reach you."))
        .child(
            FieldGroup::new()
                .gap(space::S3)
                .child(choice(0, "Email", true))
                .child(choice(1, "SMS", true)),
        )
}

/// The Separator story's group: two name fields either side of "Or".
fn names(id: ElementId) -> FieldGroup {
    let name = |index: usize, text: &'static str| {
        Field::new()
            .child(label(text))
            .child(input(child(&id, index), empty))
    };
    group()
        .child(name(0, "First name"))
        .child(FieldSeparator::new().text("Or"))
        .child(name(1, "Last name"))
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Field",
            properties: &["orientation"],
            variants: || {
                FieldOrientation::iter()
                    .map(|orientation| Variant {
                        props: vec![("orientation", orientation.into())],
                        story: Some(match orientation {
                            FieldOrientation::Vertical => "components-field--vertical",
                            FieldOrientation::Horizontal => "components-field--horizontal",
                            FieldOrientation::Responsive => "components-field--responsive",
                        }),
                        render: Box::new(move |id| {
                            group().child(field(id, orientation)).into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "FieldLabel",
            properties: &[],
            variants: || one(None, || label("Username").into_any_element()),
        },
        Entry {
            name: "FieldTitle",
            properties: &[],
            variants: || {
                one(None, || {
                    FieldTitle::new()
                        .child("Marketing emails")
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "FieldDescription",
            properties: &[],
            variants: || {
                one(None, || {
                    at_story_width(FieldDescription::new("Choose a unique username."))
                })
            },
        },
        Entry {
            name: "FieldError",
            properties: &[],
            variants: || {
                one(None, || {
                    at_story_width(FieldError::new("Enter a valid email."))
                })
            },
        },
        Entry {
            name: "FieldContent",
            properties: &[],
            variants: || {
                one(None, || {
                    at_story_width(
                        FieldContent::new()
                            .child(label("Marketing emails"))
                            .child(FieldDescription::new("Product news, at most once a month.")),
                    )
                })
            },
        },
        Entry {
            name: "FieldLegend",
            properties: &["variant"],
            variants: || {
                FieldLegendVariant::iter()
                    .map(|variant| Variant {
                        props: vec![("variant", variant.into())],
                        story: None,
                        render: Box::new(move |_| {
                            FieldLegend::new("Notifications")
                                .variant(variant)
                                .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "FieldSet",
            properties: &[],
            variants: || {
                vec![Variant {
                    props: Vec::new(),
                    story: Some("components-field--legend"),
                    render: Box::new(|id| notifications(id).into_any_element()),
                }]
            },
        },
        Entry {
            name: "FieldGroup",
            properties: &[],
            variants: || {
                vec![Variant {
                    props: Vec::new(),
                    story: Some("components-field--separator"),
                    render: Box::new(|id| names(id).into_any_element()),
                }]
            },
        },
        Entry {
            name: "FieldSeparator",
            properties: &[],
            // Alone, without the -my-2 that tucks it between two fields.
            variants: || {
                one(None, || {
                    at_story_width(FieldSeparator::new().text("Or").my(gpui::px(0.)))
                })
            },
        },
    ]
}
