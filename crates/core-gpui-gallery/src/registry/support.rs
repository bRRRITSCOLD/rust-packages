//! What several entries share: a boolean property's values, and state for an editor cell.

use gpui::{AnyElement, App, Context, ElementId, Entity, IntoElement, RenderOnce, Window};

/// `false`, then `true`: a boolean Penpot property's values, as the registry lists them.
pub(super) const BOOLS: [bool; 2] = [false, true];

/// A boolean's Penpot value.
pub(super) fn value(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

/// Builds a cell's element from its state.
type Make<S> = Box<dyn FnOnce(&Entity<S>) -> AnyElement>;

/// Renders `make` over a state entity kept per element `id` and started by `init`, so a
/// registry cell (which has no window to create one in) can show an input that remembers its
/// text in the live gallery.
#[derive(IntoElement)]
pub(super) struct Stateful<S: 'static> {
    id: ElementId,
    init: fn(&mut Window, &mut Context<S>) -> S,
    make: Make<S>,
}

impl<S: 'static> Stateful<S> {
    pub(super) fn new(
        id: impl Into<ElementId>,
        init: fn(&mut Window, &mut Context<S>) -> S,
        make: impl FnOnce(&Entity<S>) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            init,
            make: Box::new(make),
        }
    }
}

impl<S: 'static> RenderOnce for Stateful<S> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id, cx, self.init);
        (self.make)(&state)
    }
}

/// The `index`th child of the element `id` names, as an id of its own.
pub(super) fn child(id: &ElementId, index: usize) -> ElementId {
    ElementId::NamedChild(std::sync::Arc::new(id.clone()), index.to_string().into())
}
