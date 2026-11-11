//! `Avatar` (Penpot property `size`) and its parts, with the story's portrait and "AL"
//! initials.

use std::sync::Arc;

use brrritscold_core_gpui::{
    Avatar, AvatarBadge, AvatarFallback, AvatarGroupCount, AvatarImage, AvatarSize, IconName,
};
use gpui::{Image, ImageFormat, IntoElement as _, ParentElement as _, Styled as _, div, px};
use strum::IntoEnumIterator as _;

use super::{Entry, Variant, one};

/// The stories' local portrait: a grey square with a light head. GPUI rasterises an SVG image
/// at its intrinsic size, so this one is 80px (a 2x `lg` avatar) where the story's has none.
const PORTRAIT: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="80" viewBox="0 0 8 8"><rect width="8" height="8" fill="#6b7280"/><circle cx="4" cy="3" r="2" fill="#e5e7eb"/></svg>"##;

fn image() -> AvatarImage {
    AvatarImage::new(Arc::new(Image::from_bytes(
        ImageFormat::Svg,
        PORTRAIT.as_bytes().to_vec(),
    )))
}

fn fallback() -> AvatarFallback {
    AvatarFallback::new("AL")
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Avatar",
            properties: &["size"],
            variants: || {
                AvatarSize::iter()
                    .map(|size| Variant {
                        props: vec![("size", size.into())],
                        story: Some("components-avatar--default"),
                        render: Box::new(move |_| {
                            Avatar::new()
                                .size(size)
                                .image(image())
                                .fallback(fallback())
                                .into_any_element()
                        }),
                    })
                    .collect()
            },
        },
        // The parts fill or pin to their avatar, so each sits in a default avatar's 32px box.
        Entry {
            name: "AvatarImage",
            properties: &[],
            variants: || {
                one(None, || {
                    div().size(px(32.)).child(image()).into_any_element()
                })
            },
        },
        Entry {
            name: "AvatarFallback",
            properties: &[],
            variants: || {
                one(None, || {
                    div().size(px(32.)).child(fallback()).into_any_element()
                })
            },
        },
        Entry {
            name: "AvatarBadge",
            properties: &[],
            variants: || {
                one(None, || {
                    div()
                        .relative()
                        .size(px(10.))
                        .child(AvatarBadge::new())
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "AvatarGroupCount",
            properties: &[],
            variants: || {
                one(None, || {
                    AvatarGroupCount::new()
                        .icon(IconName::Plus)
                        .into_any_element()
                })
            },
        },
    ]
}
