//! The styled components, one module per core-react component file.
//!
//! Every component and part keeps its caller's style refinements and applies them over its
//! own, so `Card::new().w(px(384.))` overrides the card's width the way a `className` does in
//! core-react. Parts that hold open content (`CardContent`, `TableCell`, …) take children
//! through [`gpui::ParentElement`]; parts whose look depends on their parent (a card's `size`, an
//! alert's `variant`) are typed builders on that parent, which hands its state down.

/// Implements [`gpui::Styled`] over a `style: StyleRefinement` field.
macro_rules! impl_styled {
    ($($component:ty),+ $(,)?) => {$(
        impl gpui::Styled for $component {
            fn style(&mut self) -> &mut gpui::StyleRefinement {
                &mut self.style
            }
        }
    )+};
}

/// Implements [`gpui::ParentElement`] over a `children: Vec<AnyElement>` field.
macro_rules! impl_parent {
    ($($component:ty),+ $(,)?) => {$(
        impl gpui::ParentElement for $component {
            fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
                self.children.extend(elements);
            }
        }
    )+};
}

pub mod alert;
pub mod badge;
pub mod button;
pub mod card;
pub mod label;
pub mod separator;
pub mod skeleton;

/// `ring-<width>`: a solid spread shadow outside the box, which takes no layout space.
fn ring(width: f32, color: gpui::Hsla) -> gpui::BoxShadow {
    gpui::BoxShadow {
        color,
        offset: gpui::point(gpui::px(0.), gpui::px(0.)),
        blur_radius: gpui::px(0.),
        spread_radius: gpui::px(width),
        inset: false,
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    //! Window helpers shared by the components' unit tests.

    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{
        AnyElement, AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels,
        Render, Styled as _, TestAppContext, VisualTestContext, Window, div,
    };

    use crate::theme::{ActiveTheme as _, Mode, Theme, ThemePreference};

    /// Renders the element `make` builds at the window's origin and records its bounds.
    struct Harness {
        make: Box<dyn Fn() -> AnyElement>,
        bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let bounds = self.bounds.clone();
            div()
                .flex()
                .items_start()
                .child((self.make)())
                .on_children_prepainted(move |children, _, _| bounds.set(children.first().copied()))
        }
    }

    /// Opens a [`crate::Root`] window on an app [`crate::init`] has set up, so text inherits
    /// the root surface's style as in an app, renders `make`'s element, draws once and returns
    /// its laid-out bounds.
    pub(crate) fn render<E: IntoElement>(
        cx: &mut TestAppContext,
        make: impl Fn() -> E + 'static,
    ) -> (&mut VisualTestContext, Bounds<Pixels>) {
        let bounds = Rc::new(Cell::new(None));
        let window = cx.add_window({
            let bounds = bounds.clone();
            move |window, cx| {
                let harness = cx.new(|_| Harness {
                    make: Box::new(move || make().into_any_element()),
                    bounds,
                });
                crate::Root::new(harness, window, cx)
            }
        });
        let cx = VisualTestContext::from_window(*window, cx).into_mut();
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let drawn = bounds.get().expect("the element was laid out");
        (cx, drawn)
    }

    /// Switches the app to `mode` and returns the theme.
    pub(crate) fn theme(cx: &mut TestAppContext, mode: Mode) -> Theme {
        let preference = mode.pick(ThemePreference::Light, ThemePreference::Dark);
        cx.update(|cx| Theme::set_preference(preference, cx));
        cx.read(|cx| *cx.theme())
    }
}
