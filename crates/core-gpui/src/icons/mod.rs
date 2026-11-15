//! Lucide icons, vendored from `lucide-static` at the exact version core-react pins.
//!
//! Every canonical Lucide icon is here, named as lucide-react names it: core-react's
//! `PlusIcon` is [`IconName::Plus`]. The SVGs are embedded, so there's no `AssetSource` to set
//! up. Adding an icon on the Rust side is nothing: it arrives with the next core-react pin.

#[rustfmt::skip]
mod generated;

pub use generated::IconName;

use std::collections::HashSet;
use std::f32::consts::TAU;
use std::time::{Duration, Instant};

use gpui::{
    App, Bounds, Element, ElementId, EntityId, Global, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, Pixels, Radians, Refineable as _, RenderOnce, StyleRefinement, Styled,
    Svg, Transformation, Window, radians, rems, svg,
};

/// How many times a second a spinning icon steps, and so redraws its window, at most.
const SPIN_FPS: u32 = 15;

/// One Lucide icon. `1rem` square by default, in the inherited text colour; style it like
/// any element (`.size_4()`, `.text_color(..)`).
pub struct Icon {
    name: IconName,
    rotation: Option<Radians>,
    style: StyleRefinement,
}

impl Icon {
    /// An icon in its default style.
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            rotation: None,
            style: StyleRefinement::default(),
        }
    }

    /// Turns the drawn icon about its centre; layout is unchanged.
    fn rotate(mut self, angle: Radians) -> Self {
        self.rotation = Some(angle);
        self
    }

    /// Tailwind's `animate-spin`: one clockwise turn a second, linear, forever. Drawn at rest
    /// when the app reduces motion.
    pub(crate) fn spin(self) -> Spin {
        Spin(self)
    }
}

/// `@keyframes spin { to { transform: rotate(360deg) } }`: `delta` runs 0 → 1 each second.
fn spin_angle(delta: f32) -> Radians {
    radians(delta * TAU)
}

/// A turning [`Icon`], from [`Icon::spin`].
///
/// GPUI redraws a whole window for any change in it, so the turn steps [`SPIN_FPS`] times a
/// second rather than every frame. The steps are the app's [`SpinClock`] ticks, shared by
/// every turning icon: GPUI's `Animation::with_max_fps` times each animation from its own
/// last frame, so icons that start apart redraw apart, and fifty of them redraw every frame.
#[derive(IntoElement)]
pub(crate) struct Spin(Icon);

impl RenderOnce for Spin {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if cx.reduce_motion() {
            return self.0;
        }
        let step = SpinClock::wake_on_next_tick(window.current_view(), cx);
        self.0.rotate(spin_angle(step as f32 / SPIN_FPS as f32))
    }
}

/// The ticks every turning icon steps on, counted from the app's first spin.
#[derive(Default)]
struct SpinClock {
    start: Option<Instant>,
    /// The views a timer will wake on the next tick, so a view that draws more than once
    /// between ticks starts no second timer.
    waking: HashSet<EntityId>,
}

impl Global for SpinClock {}

impl SpinClock {
    /// Returns the current tick's step within the turn (0 to [`SPIN_FPS`] - 1), and wakes
    /// `view` on the next tick unless that's in hand.
    fn wake_on_next_tick(view: EntityId, cx: &mut App) -> u32 {
        let now = cx.background_executor().now();
        let clock = cx.default_global::<SpinClock>();
        let start = *clock.start.get_or_insert(now);
        // Whole nanoseconds, so the next tick always lies after `now`.
        let tick = (now - start).as_nanos() * u128::from(SPIN_FPS) / NANOS_PER_SEC;
        if clock.waking.insert(view) {
            let next = ((tick + 1) * NANOS_PER_SEC).div_ceil(u128::from(SPIN_FPS));
            let delay = start + Duration::from_nanos(next as u64) - now;
            let timer = cx.background_executor().timer(delay);
            cx.spawn(async move |cx| {
                timer.await;
                cx.update(|cx| {
                    cx.global_mut::<SpinClock>().waking.remove(&view);
                    cx.notify(view);
                });
            })
            .detach();
        }
        (tick % u128::from(SPIN_FPS)) as u32
    }
}

const NANOS_PER_SEC: u128 = 1_000_000_000;

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
        if let Some(angle) = self.rotation {
            svg = svg.with_transformation(Transformation::rotate(angle));
        }
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

    mod spin {
        use std::cell::Cell;
        use std::f32::consts::PI;
        use std::rc::Rc;

        use gpui::{ParentElement as _, TestAppContext, div, prelude::FluentBuilder as _};

        use super::*;
        use crate::components::test_support::{play_frame, redraws_in_one_second, render};

        #[test]
        fn should_turn_once_clockwise_each_cycle() {
            assert_eq!(spin_angle(0.), radians(0.));
            assert_eq!(spin_angle(0.25), radians(PI / 2.));
            assert_eq!(spin_angle(1.), radians(TAU));
        }

        #[gpui::test]
        fn should_redraw_its_window_at_most_spin_fps_times_a_second(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let redraws = redraws_in_one_second(cx, || Icon::new(IconName::LoaderCircle).spin());
            assert!(redraws > 0, "it stopped turning");
            assert!(redraws <= SPIN_FPS as usize, "{redraws} redraws");
        }

        #[gpui::test]
        fn should_hold_still_when_the_app_reduces_motion(cx: &mut TestAppContext) {
            cx.update(|cx| {
                crate::init(cx);
                cx.set_reduce_motion(true);
            });
            let redraws = redraws_in_one_second(cx, || Icon::new(IconName::LoaderCircle).spin());
            assert_eq!(redraws, 0);
        }

        #[gpui::test]
        fn should_keep_icons_that_start_apart_on_the_same_redraws(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let second = Rc::new(Cell::new(false));
            let renders = Rc::new(Cell::new(0));
            let (cx, _) = render(cx, {
                let (second, renders) = (second.clone(), renders.clone());
                move || {
                    renders.set(renders.get() + 1);
                    div()
                        .flex()
                        .child(Icon::new(IconName::LoaderCircle).spin())
                        .when(second.get(), |row| {
                            row.child(Icon::new(IconName::LoaderCircle).spin())
                        })
                }
            });
            // A frame in, something else redraws the window and a second icon starts turning.
            play_frame(cx);
            second.set(true);
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
            let drawn = renders.get();
            for _ in 0..60 {
                play_frame(cx);
            }
            let redraws = renders.get() - drawn;
            assert!(redraws <= SPIN_FPS as usize, "{redraws} redraws");
        }
    }
}
