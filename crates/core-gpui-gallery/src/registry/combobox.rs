//! `ComboboxInput` (Penpot properties `disabled` × `showClear`, the combinations Penpot draws),
//! the chips and the list's parts, with the stories' framework content.
//!
//! The Default story's play function picks SvelteKit, so an enabled input shows it (with
//! `showClear`, beside the clear button); the Disabled story picks nothing and shows the
//! placeholder. The Multiple story starts on Next.js and its play function adds Nuxt.

use brrritscold_core_gpui::{
    ComboboxChip, ComboboxChips, ComboboxContent, ComboboxEmpty, ComboboxInput, ComboboxItem,
    ComboboxLabel, ComboboxSeparator, InputState,
};
use brrritscold_core_gpui_tokens::primitive::space;
use gpui::{Context, IntoElement as _, ParentElement as _, Styled as _, Window, div, px};

use super::support::{Stateful, value};
use super::{Entry, Variant, one};

/// The stories' frameworks, in their order.
pub(super) const FRAMEWORKS: [&str; 5] = ["Next.js", "SvelteKit", "Nuxt", "Remix", "Astro"];

/// The story's field fills its vertical `Field`: the padded story width less the story's own
/// `p-8`, as Select's trigger does.
pub(super) const FIELD_WIDTH: gpui::Pixels = px(544.);

/// The single-value stories' empty input: `placeholder="Pick a framework"`.
pub(super) fn framework(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
    InputState::new(window, cx).placeholder("Pick a framework")
}

/// The Default story's input once its play function picked SvelteKit.
fn picked(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
    framework(window, cx).default_value("SvelteKit")
}

/// The Multiple story's chips input: `placeholder="Add a framework"`.
pub(super) fn frameworks(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
    InputState::new(window, cx).placeholder("Add a framework")
}

/// The stories' list: an item per framework, and the message when none matches.
pub(super) fn content() -> ComboboxContent {
    FRAMEWORKS.iter().fold(
        ComboboxContent::new().empty(ComboboxEmpty::new("No frameworks found.")),
        |content, framework| content.item(ComboboxItem::new(*framework, *framework)),
    )
}

/// The list's width under the field: the field less its border.
const LIST_WIDTH: gpui::Pixels = px(542.);

/// A list part inside the list's padding.
fn in_list(part: impl gpui::IntoElement) -> gpui::AnyElement {
    div()
        .w(LIST_WIDTH - space::S2)
        .child(part)
        .into_any_element()
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "ComboboxInput",
            properties: &["disabled", "showClear"],
            variants: || {
                // Penpot's combinations: enabled with and without the clear button, and
                // disabled.
                [(false, false), (false, true), (true, false)]
                    .into_iter()
                    .map(|(disabled, show_clear)| Variant {
                        props: vec![
                            ("disabled", value(disabled)),
                            ("showClear", value(show_clear)),
                        ],
                        story: Some(if disabled {
                            "components-combobox--disabled"
                        } else {
                            "components-combobox--default"
                        }),
                        render: Box::new(move |id| {
                            let init = if disabled { framework } else { picked };
                            Stateful::new(id, init, move |state| {
                                ComboboxInput::new(state)
                                    .disabled(disabled)
                                    .show_clear(show_clear)
                                    .w(FIELD_WIDTH)
                                    .into_any_element()
                            })
                            .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "ComboboxChips",
            properties: &[],
            variants: || {
                vec![Variant {
                    props: Vec::new(),
                    story: Some("components-combobox--multiple"),
                    render: Box::new(|id| {
                        Stateful::new(id, frameworks, |state| {
                            ComboboxChips::new(state)
                                .chip(ComboboxChip::new("Next.js"))
                                .chip(ComboboxChip::new("Nuxt"))
                                .w(FIELD_WIDTH)
                                .into_any_element()
                        })
                        .into_any_element()
                    }),
                }]
            },
        },
        Entry {
            name: "ComboboxChip",
            properties: &[],
            variants: || {
                one(Some("components-combobox--multiple"), || {
                    ComboboxChip::new("Next.js").into_any_element()
                })
            },
        },
        Entry {
            name: "ComboboxContent",
            properties: &[],
            // Opened from the Default story's trigger, after its play function picked
            // SvelteKit.
            variants: || {
                one(Some("components-combobox--default"), || {
                    let content = FRAMEWORKS.iter().fold(ComboboxContent::new(), |c, f| {
                        c.item(ComboboxItem::new(*f, *f).selected(*f == "SvelteKit"))
                    });
                    content.w(LIST_WIDTH).into_any_element()
                })
            },
        },
        Entry {
            name: "ComboboxItem",
            properties: &[],
            // Penpot's main component shows its check.
            variants: || {
                one(None, || {
                    in_list(ComboboxItem::new("Next.js", "Next.js").selected(true))
                })
            },
        },
        Entry {
            name: "ComboboxLabel",
            properties: &[],
            variants: || one(None, || in_list(ComboboxLabel::new("Frameworks"))),
        },
        Entry {
            name: "ComboboxSeparator",
            properties: &[],
            // The list's own padding, which the separator's -mx-1 runs into.
            variants: || {
                one(None, || {
                    div()
                        .w(LIST_WIDTH)
                        .p(space::S1)
                        .child(ComboboxSeparator::new())
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "ComboboxEmpty",
            properties: &[],
            // The Empty story's play function types "zzz": the list shows only the message.
            variants: || {
                one(Some("components-combobox--empty"), || {
                    ComboboxContent::new()
                        .empty(ComboboxEmpty::new("No frameworks found."))
                        .w(LIST_WIDTH)
                        .into_any_element()
                })
            },
        },
    ]
}
