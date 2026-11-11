//! Date Picker: core-react's `date-picker.tsx`, a composition with one export: an outline
//! [`Button`] that opens a popover holding a [`Calendar`]. `gpui-base`'s
//! [`gpui_base::Popover`] owns opening from the trigger (pointer, Enter or Space), dismissal on
//! Escape or a press outside, and focus moving into the popup and back to the trigger.
//!
//! The picked date is the caller's ([`DatePicker::value`]) or the picker's own; a pick sets it,
//! closes the popup and runs [`DatePicker::on_value_change`].
//!
//! Not mirrored: the popup's open and close animations, and focus landing on the picked day
//! when it opens (the calendar has no grid focus; see [`Calendar`]).

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::{font_weight, shadow, space};
use chrono::{Datelike as _, NaiveDate};
use gpui::{
    Anchor, App, Context, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
    transparent_black,
};
use gpui_base::StyledExt as _;

use super::button::{Button, ButtonVariant};
use super::calendar::{Calendar, Matcher, month_name};
use crate::icons::IconName;
use crate::theme::ActiveTheme as _;

type ValueChangeHandler = Rc<dyn Fn(Option<NaiveDate>, &mut Window, &mut App)>;
type Format = Rc<dyn Fn(NaiveDate) -> SharedString>;

/// date-fns's `PPP`, core-react's default format: `November 9th, 2026`.
fn format_ppp(date: NaiveDate) -> SharedString {
    let day = date.day();
    let suffix = match (day % 10, day % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!(
        "{} {day}{suffix}, {}",
        month_name(date.month()),
        date.year()
    )
    .into()
}

/// A date picker: a trigger showing the picked date, or a placeholder, that opens a calendar to
/// pick another.
///
/// ```no_run
/// use brrritscold_core_gpui::DatePicker;
/// use chrono::NaiveDate;
///
/// let due = DatePicker::new("due")
///     .default_value(NaiveDate::from_ymd_opt(2026, 11, 9).unwrap())
///     .disabled_days(vec![0, 6]) // weekends
///     .on_value_change(|date, _, _| println!("{date:?}"));
/// ```
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    value: Option<Option<NaiveDate>>,
    default_value: Option<NaiveDate>,
    placeholder: SharedString,
    disabled: bool,
    disabled_days: Option<Rc<Matcher>>,
    format_value: Format,
    on_value_change: Option<ValueChangeHandler>,
    style: StyleRefinement,
}

impl DatePicker {
    /// A closed picker with nothing picked. `id` keys its state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            default_value: None,
            placeholder: "Pick a date".into(),
            disabled: false,
            disabled_days: None,
            format_value: Rc::new(format_ppp),
            on_value_change: None,
            style: StyleRefinement::default(),
        }
    }

    /// The picked date, owned by the caller (`None`: nothing picked); it then changes only when
    /// the caller passes another.
    pub fn value(mut self, value: Option<NaiveDate>) -> Self {
        self.value = Some(value);
        self
    }

    /// The date picked at first, when the picker keeps its own value.
    pub fn default_value(mut self, value: NaiveDate) -> Self {
        self.default_value = Some(value);
        self
    }

    /// What the trigger shows, muted, while nothing is picked; `Pick a date` unless set.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Penpot property `disabled`: dims the trigger to 50% and keeps the picker closed.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Days that can't be picked.
    pub fn disabled_days(mut self, matcher: impl Into<Matcher>) -> Self {
        self.disabled_days = Some(Rc::new(matcher.into()));
        self
    }

    /// How the trigger shows the picked date; date-fns's `PPP` (`November 9th, 2026`) unless
    /// set.
    pub fn format_value(mut self, format: impl Fn(NaiveDate) -> SharedString + 'static) -> Self {
        self.format_value = Rc::new(format);
        self
    }

    /// Runs on a pick with the picked date, or `None` when the picked day is picked again.
    pub fn on_value_change(
        mut self,
        handler: impl Fn(Option<NaiveDate>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let default_value = self.default_value;
        let state = window.use_keyed_state(
            ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "value".into()),
            cx,
            move |_, _| default_value,
        );
        if let Some(value) = self.value {
            state.update(cx, |picked, _| *picked = value);
        }
        let value = *state.read(cx);
        let s = cx.theme().semantic();

        // <Button variant="outline" className="w-72 justify-start font-normal
        // data-[empty=true]:text-muted-foreground"><CalendarIcon data-icon="inline-start" />
        let trigger = Button::new(ElementId::NamedChild(
            std::sync::Arc::new(self.id.clone()),
            "trigger".into(),
        ))
        .variant(ButtonVariant::Outline)
        .icon(IconName::Calendar)
        .label(match value {
            Some(date) => (self.format_value)(date),
            None => self.placeholder,
        })
        .disabled(self.disabled)
        .w(space::S72)
        .justify_start()
        .font_weight(font_weight::NORMAL)
        .when(value.is_none(), |b| b.text_color(s.muted_foreground))
        .refine_style(&self.style);

        let on_value_change = self.on_value_change;
        let disabled_days = self.disabled_days;
        // PopoverContent align="start" (side="bottom", sideOffset={4}) className="w-auto p-0"
        gpui_base::Popover::new(self.id)
            .anchor(Anchor::TopLeft)
            .offset(space::S1)
            .trigger_with(move |_, _, _| trigger.into_any_element())
            .when(!self.disabled, |p| {
                p.content(move |_, _, cx: &mut Context<gpui_base::PopoverState>| {
                    let popover = cx.entity().downgrade();
                    let s = cx.theme().semantic();
                    // rounded-md bg-popover text-popover-foreground shadow-md ring-1
                    // ring-foreground/10, at w-auto p-0
                    let mut shadows = vec![super::ring(1., s.foreground.opacity(0.1))];
                    shadows.extend_from_slice(shadow::MD);
                    // The calendar on a popover: in-data-[slot=popover-content]:bg-transparent
                    let calendar = Calendar::new("date-picker-calendar")
                        .selected(value)
                        .disabled_days_shared(disabled_days)
                        .bg(transparent_black())
                        .on_select(move |date, window, cx| {
                            state.update(cx, |picked, cx| {
                                *picked = date;
                                cx.notify();
                            });
                            if let Some(on_value_change) = &on_value_change {
                                on_value_change(date, window, cx);
                            }
                            _ = popover.update(cx, |popover, cx| popover.dismiss(window, cx));
                        });
                    div()
                        .debug_selector(|| "date-picker-content".into())
                        .rounded(s.radius_md)
                        .bg(s.popover)
                        .text_color(s.popover_foreground)
                        .shadow(shadows)
                        .child(calendar)
                })
            })
    }
}

impl_styled!(DatePicker);

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Modifiers, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use crate::components::test_support::render;

    type Log = Rc<RefCell<Vec<Option<NaiveDate>>>>;

    fn date(month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, day).unwrap()
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    /// The story's picker on the 9th of November 2026, weekends disabled, logging every pick.
    fn picker(log: Log, disabled: bool) -> impl Fn() -> DatePicker {
        move || {
            let log = log.clone();
            DatePicker::new("due")
                .default_value(date(11, 9))
                .disabled(disabled)
                .disabled_days(vec![0, 6])
                .on_value_change(move |date, _, _| log.borrow_mut().push(date))
        }
    }

    fn is_open(cx: &mut VisualTestContext) -> bool {
        cx.debug_bounds("date-picker-content").is_some()
    }

    /// The centre of the 20th (Friday of the third week) in the open popup under `trigger`.
    fn twentieth(trigger: gpui::Bounds<gpui::Pixels>) -> gpui::Point<gpui::Pixels> {
        let calendar_top = trigger.bottom() + px(4.);
        let top = calendar_top + px(12. + 32. + 16. + 18. + 8.) + px(40.) * 2.;
        point(trigger.left() + px(12. + 16. + 32. * 5.), top + px(16.))
    }

    mod format {
        use super::*;

        #[test]
        fn should_format_as_date_fns_ppp() {
            assert_eq!(format_ppp(date(11, 9)), "November 9th, 2026");
            assert_eq!(format_ppp(date(11, 1)), "November 1st, 2026");
            assert_eq!(format_ppp(date(11, 22)), "November 22nd, 2026");
            assert_eq!(format_ppp(date(11, 23)), "November 23rd, 2026");
            assert_eq!(format_ppp(date(11, 12)), "November 12th, 2026");
        }
    }

    mod date_picker {
        use super::*;

        #[gpui::test]
        fn should_be_288px_wide(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, trigger) = render(cx, picker(Log::default(), false));
            assert_eq!(trigger.size.width, px(288.));
            assert_eq!(trigger.size.height, px(36.));
        }

        #[gpui::test]
        fn should_pick_a_date_and_close(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, trigger) = render(cx, picker(log.clone(), false));
            cx.simulate_click(trigger.center(), Modifiers::default());
            draw(cx);
            assert!(is_open(cx));
            cx.simulate_click(twentieth(trigger), Modifiers::default());
            draw(cx);
            assert_eq!(*log.borrow(), [Some(date(11, 20))]);
            assert!(!is_open(cx), "a pick closes the popup");
        }

        #[gpui::test]
        fn should_close_on_escape_without_picking(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, trigger) = render(cx, picker(log.clone(), false));
            cx.simulate_click(trigger.center(), Modifiers::default());
            draw(cx);
            cx.simulate_keystrokes("escape");
            draw(cx);
            assert!(!is_open(cx));
            assert!(log.borrow().is_empty());
        }

        #[gpui::test]
        fn should_stay_closed_while_disabled(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (cx, trigger) = render(cx, picker(Log::default(), true));
            cx.simulate_click(trigger.center(), Modifiers::default());
            draw(cx);
            assert!(!is_open(cx));
        }
    }
}
