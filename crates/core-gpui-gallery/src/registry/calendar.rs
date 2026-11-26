//! `Calendar` and `CalendarDayButton`, with the stories' November 2026, the 9th as today. The
//! Default story's play function picks the 12th, so the calendar shows it picked.

use brrritscold_core_gpui::{Calendar, CalendarDayButton};
use gpui::IntoElement as _;
use jiff::civil;

use super::{Entry, one};

pub(super) fn november(day: i8) -> civil::Date {
    civil::date(2026, 11, day)
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Calendar",
            properties: &[],
            variants: || {
                one(Some("components-calendar--default"), || {
                    Calendar::new("calendar")
                        .default_month(november(1))
                        .today(november(9))
                        .selected(Some(november(12)))
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "CalendarDayButton",
            properties: &[],
            variants: || {
                one(None, || {
                    CalendarDayButton::new(november(12)).into_any_element()
                })
            },
        },
    ]
}
