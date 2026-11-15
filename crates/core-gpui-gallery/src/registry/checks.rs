//! `Checks`: Penpot property `state`, each with its story's and Penpot's label.

use brrritscold_core_gpui::{Checks, ChecksState};
use gpui::IntoElement as _;
use strum::IntoEnumIterator as _;

use super::{Entry, Variant};

/// The story named for `state`, and the label it and Penpot's main show.
fn sample(state: ChecksState) -> (&'static str, &'static str) {
    match state {
        ChecksState::Success => ("components-checks--success", "passing"),
        ChecksState::Pending => ("components-checks--pending", "running"),
        ChecksState::Failure => ("components-checks--failure", "failing"),
    }
}

pub(super) fn entry() -> Entry {
    Entry {
        name: "Checks",
        properties: &["state"],
        variants: || {
            ChecksState::iter()
                .map(|state| {
                    let (story, label) = sample(state);
                    Variant {
                        props: vec![("state", state.into())],
                        story: Some(story),
                        render: Box::new(move |_| {
                            Checks::new(state).label(label).into_any_element()
                        }),
                    }
                })
                .collect()
        },
    }
}
