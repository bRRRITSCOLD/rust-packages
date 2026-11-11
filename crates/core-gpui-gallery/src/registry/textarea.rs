//! `Textarea`: Penpot properties `aria-invalid` × `disabled`, with the stories' content. The
//! Default story's play function types two lines, so an enabled textarea shows them; a
//! disabled one is captured from the Disabled story, which shows the placeholder.

use brrritscold_core_gpui::{Textarea, TextareaState};
use gpui::{Context, IntoElement as _, Styled as _, Window};

use super::support::{BOOLS, Stateful, value};
use super::{Entry, STORY_WIDTH, Variant};

/// The stories' empty textarea: `placeholder: 'Type your message here.'`.
fn message(window: &mut Window, cx: &mut Context<TextareaState>) -> TextareaState {
    TextareaState::new(window, cx).placeholder("Type your message here.")
}

/// The Default story's textarea after its play function typed `Hello{enter}world`.
fn typed(window: &mut Window, cx: &mut Context<TextareaState>) -> TextareaState {
    message(window, cx).default_value("Hello\nworld")
}

pub(super) fn entry() -> Entry {
    Entry {
        name: "Textarea",
        properties: &["aria-invalid", "disabled"],
        variants: || {
            BOOLS
                .into_iter()
                .flat_map(|invalid| BOOLS.map(move |disabled| (invalid, disabled)))
                .map(|(invalid, disabled)| Variant {
                    props: vec![
                        ("aria-invalid", value(invalid)),
                        ("disabled", value(disabled)),
                    ],
                    story: Some(if disabled {
                        "components-textarea--disabled"
                    } else {
                        "components-textarea--default"
                    }),
                    // The story's w-full textarea, at the width it fills.
                    render: Box::new(move |id| {
                        let init = if disabled { message } else { typed };
                        Stateful::new(id, init, move |state| {
                            Textarea::new(state)
                                .aria_invalid(invalid)
                                .disabled(disabled)
                                .w(STORY_WIDTH)
                                .into_any_element()
                        })
                        .into_any_element()
                    }),
                })
                .collect()
        },
    }
}
