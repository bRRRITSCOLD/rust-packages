//! `Label`: no variant properties. As in the story, the label names an input below it.

use brrritscold_core_gpui::{Input, InputState, Label};
use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{IntoElement as _, ParentElement as _, Styled as _, div};

use super::support::Stateful;
use super::{Entry, STORY_WIDTH, one};

pub(super) fn entry() -> Entry {
    Entry {
        name: "Label",
        properties: &[],
        // The story's flex flex-col gap-2 box, at the width its input fills.
        variants: || {
            one(Some("components-label--default"), || {
                div()
                    .flex()
                    .flex_col()
                    .gap(space::S2)
                    .w(STORY_WIDTH)
                    .child(Label::new().child("Email"))
                    .child(Stateful::new("label-email", InputState::new, |state| {
                        Input::new(state).into_any_element()
                    }))
                    .into_any_element()
            })
        },
    }
}
