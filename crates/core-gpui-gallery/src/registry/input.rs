//! `Input`: Penpot properties `aria-invalid` × `disabled`, with the stories' content. The
//! Default story's play function types `ada@example.com`, so an enabled input shows that
//! text; a disabled one is captured from the Disabled story, which types nothing and shows
//! the placeholder.

use brrritscold_core_gpui::{Input, InputState};
use gpui::{Context, IntoElement as _, Styled as _, Window};

use super::support::{BOOLS, Stateful, value};
use super::{Entry, STORY_WIDTH, Variant};

/// The stories' empty input: `placeholder: 'you@example.com'`.
fn email(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
    InputState::new(window, cx).placeholder("you@example.com")
}

/// The Default story's input after its play function typed into it.
fn typed(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
    email(window, cx).default_value("ada@example.com")
}

pub(super) fn entry() -> Entry {
    Entry {
        name: "Input",
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
                        "components-input--disabled"
                    } else {
                        "components-input--default"
                    }),
                    // The story's w-full input, at the width it fills.
                    render: Box::new(move |id| {
                        let init = if disabled { email } else { typed };
                        Stateful::new(id, init, move |state| {
                            Input::new(state)
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
