//! `Separator`: Penpot property `orientation`.

use brrritscold_core_gpui::{Separator, SeparatorOrientation};
use gpui::{IntoElement as _, ParentElement as _, Styled as _, div, px};
use strum::IntoEnumIterator as _;

use super::{Entry, Variant};

pub(super) fn entry() -> Entry {
    Entry {
        name: "Separator",
        properties: &["orientation"],
        variants: || {
            SeparatorOrientation::iter()
                .map(|orientation| Variant {
                    props: vec![("orientation", orientation.into())],
                    story: Some(match orientation {
                        SeparatorOrientation::Horizontal => "components-separator--horizontal",
                        SeparatorOrientation::Vertical => "components-separator--vertical",
                    }),
                    // The story's box: flex h-10 w-64 items-center justify-center.
                    render: Box::new(move |_| {
                        div()
                            .flex()
                            .h(px(40.))
                            .w(px(256.))
                            .items_center()
                            .justify_center()
                            .child(Separator::new().orientation(orientation))
                            .into_any_element()
                    }),
                })
                .collect()
        },
    }
}
