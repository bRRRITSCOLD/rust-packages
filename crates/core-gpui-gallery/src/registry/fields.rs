//! The fields the overlay stories put in their surfaces: a name field (Dialog, Sheet) and a
//! width field (Popover).

use brrritscold_core_gpui::{Field, FieldLabel, FieldOrientation, Input, InputState};
use gpui::{AnyElement, Context, ElementId, IntoElement as _, ParentElement as _, Window};

use super::support::{Stateful, child};

fn empty(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
    InputState::new(window, cx)
}

fn hundred_percent(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
    InputState::new(window, cx).default_value("100%")
}

fn input(
    id: ElementId,
    init: fn(&mut Window, &mut Context<InputState>) -> InputState,
) -> AnyElement {
    Stateful::new(id, init, |state| Input::new(state).into_any_element()).into_any_element()
}

/// `Field` > `FieldLabel` "Name" + an empty `Input`, stacked.
pub(super) fn name(id: &ElementId) -> AnyElement {
    Field::new()
        .child(FieldLabel::new().child("Name"))
        .child(input(child(id, 0), empty))
        .into_any_element()
}

/// `Field orientation="horizontal"` > `FieldLabel` "Width" + an `Input` reading "100%".
pub(super) fn width(id: &ElementId) -> AnyElement {
    Field::new()
        .orientation(FieldOrientation::Horizontal)
        .child(FieldLabel::new().child("Width"))
        .child(input(child(id, 0), hundred_percent))
        .into_any_element()
}
