//! Lucide icons, vendored from `lucide-static` at the exact version core-react pins.
//!
//! Every canonical Lucide icon is here, named as lucide-react names it: core-react's
//! `PlusIcon` is [`IconName::Plus`]. The SVGs are embedded, so there's no `AssetSource` to set
//! up. Adding an icon on the Rust side is nothing: it arrives with the next core-react pin.

#[rustfmt::skip]
mod generated;

pub use generated::IconName;

use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    Pixels, Refineable as _, StyleRefinement, Styled, Svg, Window, rems, svg,
};

/// One Lucide icon. `1rem` square by default, in the inherited text colour; style it like
/// any element (`.size_4()`, `.text_color(..)`).
pub struct Icon {
    name: IconName,
    style: StyleRefinement,
}

impl Icon {
    /// An icon in its default style.
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            style: StyleRefinement::default(),
        }
    }
}

impl From<IconName> for Icon {
    fn from(name: IconName) -> Self {
        Icon::new(name)
    }
}

impl Styled for Icon {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl IntoElement for Icon {
    type Element = IconElement;

    fn into_element(self) -> IconElement {
        let mut svg = svg().data(self.name.svg()).size(rems(1.)).flex_none();
        svg.style().refine(&self.style);
        IconElement {
            svg,
            inherit_color: self.style.text.color.is_none(),
        }
    }
}

/// The element behind [`Icon`]. GPUI's SVG paints only its own text colour, so this fills
/// it from the inherited text style at paint time, when the parents' styles are in place.
pub struct IconElement {
    svg: Svg,
    inherit_color: bool,
}

impl IntoElement for IconElement {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for IconElement {
    type RequestLayoutState = <Svg as Element>::RequestLayoutState;
    type PrepaintState = <Svg as Element>::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        self.svg.id()
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.svg.source_location()
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.svg.request_layout(id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.svg
            .prepaint(id, inspector_id, bounds, request_layout, window, cx)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.inherit_color {
            self.svg.style().text.color = Some(window.text_style().color);
        }
        self.svg.paint(
            id,
            inspector_id,
            bounds,
            request_layout,
            prepaint,
            window,
            cx,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod icon_name {
        use super::*;

        #[test]
        fn should_cover_every_vendored_icon() {
            assert!(IconName::ALL.len() > 1900);
            assert_eq!(IconName::Plus.file_name(), "plus");
        }

        #[test]
        fn should_embed_svg_sources() {
            for name in IconName::ALL {
                let svg = std::str::from_utf8(name.svg()).unwrap();
                assert!(svg.contains("<svg"), "{} isn't an SVG", name.file_name());
            }
        }
    }
}
