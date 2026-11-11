//! `Tabs` (Penpot property `orientation`) and its parts (`TabsList` `variant`, `TabsTrigger`
//! `disabled`), with the story's settings tabs. The stories' play functions select Password,
//! so the entries show it selected; Penpot's main components show Account.

use brrritscold_core_gpui::{
    Tabs, TabsContent, TabsList, TabsListVariant, TabsOrientation, TabsTrigger,
};
use gpui::{IntoElement as _, ParentElement as _};
use strum::IntoEnumIterator as _;

use super::support::{BOOLS, value};
use super::{Entry, Variant, one};

const TABS: [(&str, &str, &str); 3] = [
    ("account", "Account", "Change your name and email address."),
    ("password", "Password", "Change your password."),
    ("billing", "Billing", "Manage your plan and invoices."),
];

/// What the play functions leave selected.
const SELECTED: &str = "password";

fn list(variant: TabsListVariant) -> TabsList {
    TABS.into_iter().fold(
        TabsList::new().variant(variant),
        |list, (value, label, _)| list.trigger(TabsTrigger::new(value, label)),
    )
}

fn tabs(id: gpui::ElementId, orientation: TabsOrientation, variant: TabsListVariant) -> Tabs {
    TABS.into_iter().fold(
        Tabs::new(id)
            .orientation(orientation)
            .default_value(SELECTED)
            .list(list(variant)),
        |tabs, (value, _, body)| tabs.content(TabsContent::new(value).child(body)),
    )
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Tabs",
            properties: &["orientation"],
            variants: || {
                TabsOrientation::iter()
                    .map(|orientation| Variant {
                        props: vec![("orientation", orientation.into())],
                        story: Some(match orientation {
                            TabsOrientation::Horizontal => "components-tabs--default",
                            TabsOrientation::Vertical => "components-tabs--vertical",
                        }),
                        render: Box::new(move |id| {
                            tabs(id, orientation, TabsListVariant::Default).into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "TabsList",
            properties: &["variant"],
            // A list shows its selection through its tabs, so it is drawn in its Tabs, with no
            // panel. The story column shows the whole story, panel included.
            variants: || {
                TabsListVariant::iter()
                    .map(|variant| Variant {
                        props: vec![("variant", variant.into())],
                        story: Some(match variant {
                            TabsListVariant::Default => "components-tabs--default",
                            TabsListVariant::Line => "components-tabs--line",
                        }),
                        render: Box::new(move |id| {
                            Tabs::new(id)
                                .default_value(SELECTED)
                                .list(list(variant))
                                .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "TabsTrigger",
            properties: &["disabled"],
            // A tab at rest, as Penpot's main component shows it.
            variants: || {
                BOOLS
                    .into_iter()
                    .map(|disabled| Variant {
                        props: vec![("disabled", value(disabled))],
                        story: None,
                        render: Box::new(move |_| {
                            TabsTrigger::new("account", "Account")
                                .disabled(disabled)
                                .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        Entry {
            name: "TabsContent",
            properties: &[],
            variants: || {
                one(None, || {
                    TabsContent::new("password")
                        .child("Change your password.")
                        .into_any_element()
                })
            },
        },
    ]
}
