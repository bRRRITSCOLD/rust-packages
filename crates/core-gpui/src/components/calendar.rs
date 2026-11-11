//! Calendar: core-react's `calendar.tsx` (react-day-picker in core-react's classes). The shown
//! month, its weeks and the pick live in `gpui-base`'s [`gpui_base::CalendarState`], kept per
//! calendar, and disabled days are `gpui-base`'s [`Matcher`] (react-day-picker's matchers: days
//! of the week, before or after a date, a range, or a function). The drawing is ours:
//! `gpui-base`'s own `Calendar` lays its header out as month and year toggles, where core-react
//! shows one centred caption between the month buttons.
//!
//! The picked day is the caller's ([`Calendar::selected`]) or the calendar's own, and
//! [`Calendar::on_select`] reports each pick. Days are [`CalendarDayButton`]s on core-gpui's
//! [`Button`], as core-react's are on its `Button`.
//!
//! Not mirrored:
//! - Arrow-key, Page Up / Down movement through the grid and its single Tab stop: each day is
//!   its own Tab stop, picked with Enter or Space.
//! - `mode="range"`, `numberOfMonths`, `captionLayout="dropdown"` and week numbers, which no
//!   Penpot variant property names: one month, single date.
//! - Ghost's `hover:text-foreground` (and `dark:hover:text-foreground`) on a muted day: the
//!   button keeps the day's colour under the pointer, over ghost's `hover:bg-muted`.
//! - The day's `[&>span]` sub-label and the locale: the caption and weekdays are English, weeks
//!   start on Sunday.

use std::rc::Rc;

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, opacity, space};
use chrono::{Datelike as _, NaiveDate};
use gpui::{
    App, ElementId, Entity, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};
pub use gpui_base::Matcher;
use gpui_base::{CalendarState, Date, StyledExt as _};

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::icons::IconName;
use crate::theme::ActiveTheme as _;

type SelectHandler = Rc<dyn Fn(Option<NaiveDate>, &mut Window, &mut App)>;

/// The months' names, January first, as the caption reads them.
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// The weekday headings, Sunday first: react-day-picker's two-letter `cccccc`.
const WEEKDAYS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

/// The English name of `month`, 1-based.
pub(crate) fn month_name(month: u32) -> &'static str {
    MONTHS[(month as usize + 11) % 12]
}

/// A month calendar: a caption between the previous and next month buttons, the weekdays, and a
/// grid of days, the picked one `primary`, today `muted`, the months either side's
/// `muted-foreground`.
///
/// ```no_run
/// use brrritscold_core_gpui::Calendar;
/// use chrono::NaiveDate;
///
/// let due = Calendar::new("due")
///     .default_month(NaiveDate::from_ymd_opt(2026, 11, 1).unwrap())
///     .disabled_days(vec![0, 6]) // weekends
///     .on_select(|date, _, _| println!("{date:?}"));
/// ```
#[derive(IntoElement)]
pub struct Calendar {
    id: ElementId,
    selected: Option<Option<NaiveDate>>,
    default_month: Option<NaiveDate>,
    today: Option<NaiveDate>,
    disabled_days: Option<Rc<Matcher>>,
    on_select: Option<SelectHandler>,
    style: StyleRefinement,
}

impl Calendar {
    /// A calendar on this month with nothing picked. `id` keys the month shown and the pick.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected: None,
            default_month: None,
            today: None,
            disabled_days: None,
            on_select: None,
            style: StyleRefinement::default(),
        }
    }

    /// The picked day, owned by the caller (`None`: nothing picked); it then changes only when
    /// the caller passes another. The calendar opens on its month.
    pub fn selected(mut self, date: Option<NaiveDate>) -> Self {
        self.selected = Some(date);
        self
    }

    /// The month shown at first, when nothing is picked.
    pub fn default_month(mut self, month: NaiveDate) -> Self {
        self.default_month = Some(month);
        self
    }

    /// The day shown as today; the system's date unless set.
    pub fn today(mut self, today: NaiveDate) -> Self {
        self.today = Some(today);
        self
    }

    /// Days that can't be picked: faded, and inert.
    pub fn disabled_days(mut self, matcher: impl Into<Matcher>) -> Self {
        self.disabled_days = Some(Rc::new(matcher.into()));
        self
    }

    pub(crate) fn disabled_days_shared(mut self, matcher: Option<Rc<Matcher>>) -> Self {
        self.disabled_days = matcher;
        self
    }

    /// Runs on a pick with the picked day, or `None` when the picked day is picked again.
    pub fn on_select(
        mut self,
        handler: impl Fn(Option<NaiveDate>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// The state for this calendar, created on its first render on the picked day's month,
    /// else the default month.
    fn state(&self, window: &mut Window, cx: &mut App) -> Entity<CalendarState> {
        let selected = self.selected.flatten();
        let default_month = self.default_month;
        window.use_keyed_state(self.id.clone(), cx, move |window, cx| {
            let mut state = CalendarState::new(window, cx);
            if let Some(month) = default_month {
                state.select_year(month.year());
                state.select_month(month.month() as u8);
            }
            state.apply_date(Date::Single(selected));
            state
        })
    }
}

impl RenderOnce for Calendar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state(window, cx);
        let matcher = self.disabled_days.clone();
        state.update(cx, |state, _| {
            state.set_disabled_matcher_shared(matcher);
            if let Some(selected) = self.selected
                && state.date() != Date::Single(selected)
            {
                state.apply_date(Date::Single(selected));
            }
        });
        let (year, month, weeks, picked, system_today) = {
            let s = state.read(cx);
            let weeks = s.month_days().into_iter().next().unwrap_or_default();
            let picked = match s.date() {
                Date::Single(date) => date,
                Date::Range(start, _) => start,
            };
            (
                s.current_year(),
                s.current_month() as u32,
                weeks,
                picked,
                s.today(),
            )
        };
        let today = self.today.unwrap_or(system_today);
        let disabled = |date: &NaiveDate| {
            self.disabled_days
                .as_ref()
                .is_some_and(|matcher| matcher.matched(date))
        };
        let s = cx.theme().semantic();

        let step = |id: &'static str, icon: IconName, forward: bool| {
            let state = state.clone();
            // buttonVariants({ variant: 'ghost' }) size-8 p-0, a size-4 chevron
            Button::new(id)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .icon(icon)
                .on_click(move |_, _, cx| {
                    state.update(cx, |state, cx| {
                        if forward {
                            state.next_month();
                        } else {
                            state.prev_month();
                        }
                        cx.notify();
                    })
                })
        };
        // month_caption: flex h-8 w-full items-center justify-center px-8; caption_label:
        // text-sm font-medium. nav: absolute inset-x-0 top-0 flex w-full items-center
        // justify-between gap-1
        let caption = div()
            .relative()
            .flex()
            .h(space::S8)
            .w_full()
            .items_center()
            .justify_center()
            .px(space::S8)
            .text_size(font_size::SM)
            .font_weight(font_weight::MEDIUM)
            .child(format!("{} {year}", month_name(month)))
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(space::S1)
                    .child(step("calendar-previous", IconName::ChevronLeft, false))
                    .child(step("calendar-next", IconName::ChevronRight, true)),
            );
        // weekdays: flex; weekday: flex-1 rounded-md text-xs font-normal text-muted-foreground
        let weekdays = div().flex().children(WEEKDAYS.map(|weekday| {
            div()
                .flex_1()
                .flex()
                .justify_center()
                .text_size(font_size::XS)
                .font_weight(font_weight::NORMAL)
                .text_color(s.muted_foreground)
                .child(weekday)
        }));
        let on_select = self.on_select;
        // week: mt-2 flex w-full
        let weeks = weeks.into_iter().map(|week| {
            div()
                .mt(space::S2)
                .flex()
                .w_full()
                .children(week.into_iter().map(|date| {
                    let state = state.clone();
                    let on_select = on_select.clone();
                    CalendarDayButton::new(date)
                        .selected(picked == Some(date))
                        .today(date == today)
                        .outside(date.month() != month)
                        .disabled(disabled(&date))
                        .on_click(move |window, cx| {
                            let picked = state.update(cx, |state, cx| {
                                // Picking the picked day again clears it, as react-day-picker's
                                // single mode does.
                                let next =
                                    (state.date() != Date::Single(Some(date))).then_some(date);
                                state.apply_date(Date::Single(next));
                                cx.notify();
                                next
                            });
                            if let Some(on_select) = &on_select {
                                on_select(picked, window, cx);
                            }
                        })
                }))
        });
        // root: bg-background p-3 w-fit; months: relative flex flex-col gap-4; month: flex
        // w-full flex-col gap-4
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .flex_none()
            .gap(space::S4)
            .bg(s.background)
            .p(space::S3)
            .w(space::S8 * 7. + space::S3 * 2.)
            .child(caption)
            .child(div().flex().flex_col().child(weekdays).children(weeks))
            .refine_style(&self.style)
    }
}

type ClickHandler = Box<dyn Fn(&mut Window, &mut App)>;

/// A calendar day: a 32px ghost button with the day's number, `primary` when picked, on a
/// `muted` cell when today.
#[derive(IntoElement)]
pub struct CalendarDayButton {
    date: NaiveDate,
    selected: bool,
    today: bool,
    outside: bool,
    disabled: bool,
    on_click: Option<ClickHandler>,
    style: StyleRefinement,
}

impl CalendarDayButton {
    /// The button for `date`, showing its day of the month.
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            selected: false,
            today: false,
            outside: false,
            disabled: false,
            on_click: None,
            style: StyleRefinement::default(),
        }
    }

    /// Whether it's the picked day: `primary`.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Whether it's today: a `muted` cell behind it.
    pub fn today(mut self, today: bool) -> Self {
        self.today = today;
        self
    }

    /// Whether it belongs to the month before or after the one shown: `muted-foreground`.
    pub fn outside(mut self, outside: bool) -> Self {
        self.outside = outside;
        self
    }

    /// Fades the day and keeps it from being picked.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Runs on a pointer click, Enter or Space. Never while disabled.
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for CalendarDayButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // The cell. today: rounded-md bg-muted text-foreground data-[selected=true]:rounded-none;
        // outside: text-muted-foreground; disabled: text-muted-foreground opacity-50
        let today = self.today && !self.selected;
        let text = if self.selected {
            s.primary_foreground
        } else if self.outside || self.disabled {
            s.muted_foreground
        } else {
            s.foreground
        };
        // Button variant="ghost" size="icon", over it data-[selected-single=true]:bg-primary
        // data-[selected-single=true]:text-primary-foreground: the picked day is the default
        // (primary) variant. size-auto w-full min-w-8 border-0 leading-none font-normal: a 32px
        // square cell.
        let variant = if self.selected {
            ButtonVariant::Default
        } else {
            ButtonVariant::Ghost
        };
        let id = ElementId::Name(format!("calendar-day-{}", self.date).into());
        let on_click = self.on_click;
        let button = Button::new(id)
            .variant(variant)
            .size(ButtonSize::IconSm)
            .label(SharedString::from(self.date.day().to_string()))
            .disabled(self.disabled)
            .when_some(on_click, |b, on_click| {
                b.on_click(move |_, window, cx| on_click(window, cx))
            })
            .font_weight(font_weight::NORMAL)
            .text_color(text);
        div()
            .flex_none()
            .size(space::S8)
            .rounded(s.radius_md)
            .when(today, |d| d.bg(s.muted))
            .when(self.disabled, |d| d.opacity(opacity::O50))
            .child(button)
            .refine_style(&self.style)
    }
}

impl_styled!(Calendar, CalendarDayButton);

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

    /// The story's calendar: November 2026 with the 9th as today, weekends disabled, logging
    /// every pick.
    fn november(log: Log) -> impl Fn() -> Calendar {
        move || {
            let log = log.clone();
            Calendar::new("calendar")
                .default_month(date(11, 1))
                .today(date(11, 9))
                .disabled_days(vec![0, 6])
                .on_select(move |date, _, _| log.borrow_mut().push(date))
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    /// The centre of the day in `week` (from 0) and `weekday` (Sunday 0): 12px padding, the
    /// 32px caption, 16px gap, the 18px weekday row, then 40px weeks (8px apart).
    fn day(
        calendar: gpui::Bounds<gpui::Pixels>,
        week: usize,
        weekday: usize,
    ) -> gpui::Point<gpui::Pixels> {
        let top = calendar.top() + px(12. + 32. + 16. + 18. + 8.) + px(40.) * week as f32;
        point(
            calendar.left() + px(12. + 16.) + px(32.) * weekday as f32,
            top + px(16.),
        )
    }

    mod month_name {
        use super::*;

        #[test]
        fn should_name_months_from_one() {
            assert_eq!(month_name(1), "January");
            assert_eq!(month_name(11), "November");
            assert_eq!(month_name(12), "December");
        }
    }

    mod calendar {
        use super::*;

        #[gpui::test]
        fn should_be_seven_32px_days_wide(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let (_, calendar) = render(cx, november(Log::default()));
            assert_eq!(calendar.size.width, px(32. * 7. + 24.));
        }

        #[gpui::test]
        fn should_pick_a_date_and_clear_it_when_picked_again(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, calendar) = render(cx, november(log.clone()));
            // Thursday the 12th: November 2026 starts on a Sunday.
            cx.simulate_click(day(calendar, 1, 4), Modifiers::default());
            draw(cx);
            cx.simulate_click(day(calendar, 1, 4), Modifiers::default());
            draw(cx);
            assert_eq!(*log.borrow(), [Some(date(11, 12)), None]);
        }

        #[gpui::test]
        fn should_ignore_a_disabled_day(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, calendar) = render(cx, november(log.clone()));
            // Saturday the 14th.
            cx.simulate_click(day(calendar, 1, 6), Modifiers::default());
            draw(cx);
            assert!(log.borrow().is_empty());
        }

        #[gpui::test]
        fn should_step_to_the_next_month(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, calendar) = render(cx, november(log.clone()));
            let next = point(
                calendar.right() - px(12. + 16.),
                calendar.top() + px(12. + 16.),
            );
            cx.simulate_click(next, Modifiers::default());
            draw(cx);
            // December 2026 starts on a Tuesday: its first week's Thursday is the 3rd.
            cx.simulate_click(day(calendar, 0, 4), Modifiers::default());
            draw(cx);
            assert_eq!(*log.borrow(), [Some(date(12, 3))]);
        }

        #[gpui::test]
        fn should_show_the_month_of_a_selected_date(cx: &mut TestAppContext) {
            cx.update(crate::init);
            let log = Log::default();
            let (cx, calendar) = render(cx, {
                let log = log.clone();
                move || {
                    let log = log.clone();
                    Calendar::new("calendar")
                        .default_month(date(1, 1))
                        .selected(Some(date(11, 12)))
                        .on_select(move |date, _, _| log.borrow_mut().push(date))
                }
            });
            // The 12th, selected: picking it again clears it.
            cx.simulate_click(day(calendar, 1, 4), Modifiers::default());
            draw(cx);
            assert_eq!(*log.borrow(), [None]);
        }
    }
}
