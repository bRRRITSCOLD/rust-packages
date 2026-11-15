//! `StateDot`: Penpot property `state`, each from its own story.

use brrritscold_core_gpui::{StateDot, StateDotState};
use gpui::IntoElement as _;
use strum::IntoEnumIterator as _;

use super::{Entry, Variant};

/// The story named for `state`.
fn story(state: StateDotState) -> &'static str {
    match state {
        StateDotState::Working => "components-statedot--working",
        StateDotState::Blocked => "components-statedot--blocked",
        StateDotState::Done => "components-statedot--done",
        StateDotState::Idle => "components-statedot--idle",
        StateDotState::Unknown => "components-statedot--unknown",
        StateDotState::Exited => "components-statedot--exited",
        StateDotState::ExitedError => "components-statedot--exited-error",
    }
}

pub(super) fn entry() -> Entry {
    Entry {
        name: "StateDot",
        properties: &["state"],
        variants: || {
            StateDotState::iter()
                .map(|state| Variant {
                    props: vec![("state", state.into())],
                    story: Some(story(state)),
                    render: Box::new(move |_| StateDot::new(state).into_any_element()),
                })
                .collect()
        },
    }
}
