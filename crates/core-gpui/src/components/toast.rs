//! Toast: core-react's `toast.tsx` (Base UI Toast, not Sonner) on `gpui-base`'s unstyled toast
//! parts: [`gpui_base::ToastManager`] keeps each toast's lifecycle and auto-hide timer,
//! [`gpui_base::ToastStack`] stacks them and expands the stack on hover, and
//! [`gpui_base::Toast`] gives each the alert role.
//!
//! [`Toast`] is the surface and renders on its own. [`crate::init`] registers a root plugin that
//! hosts every window's stack at its bottom-right corner (core-react's `Toaster` viewport:
//! `bottom-4 right-4`, `max-w-sm`), so a window wrapped in [`crate::Root`] shows toasts with no
//! wiring; [`ToastExt::push_toast`] adds one:
//!
//! ```no_run
//! use brrritscold_core_gpui::{Toast, ToastDescription, ToastExt as _, ToastTitle, ToastType};
//! # fn saved(window: &mut gpui::Window, cx: &mut gpui::App) {
//! window.push_toast(
//!     Toast::new()
//!         .toast_type(ToastType::Success)
//!         .title(ToastTitle::new("Event created"))
//!         .description(ToastDescription::new("Sunday, December 3 at 9:00 AM")),
//!     cx,
//! );
//! # }
//! ```
//!
//! Not mirrored: the enter and exit slides, swipe-to-dismiss, and the loading icon's spin.

use std::rc::Rc;
use std::time::Duration;

use brrritscold_core_gpui_tokens::primitive::{font_size, font_weight, shadow, size, space};
use gpui::{
    Anchor, App, ClickEvent, Context, ElementId, Empty, FocusHandle, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, RenderOnce, SharedString, StyleRefinement, Styled,
    Task, Window, div, prelude::FluentBuilder as _,
};
use gpui_base::{
    RootPlugin, StyledExt as _, ToastManager, ToastMotion, ToastOptions, ToastStack,
    ToastStackState,
};
use strum::{AsRefStr, Display, EnumIter, IntoStaticStr};

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::icons::{Icon, IconName};
use crate::theme::ActiveTheme as _;

/// How long a toast stays before it hides itself, unless [`Toast::timeout`] says otherwise
/// (Base UI's default).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// The kind of a toast (core-react's `type` option), which picks its icon. Not a Penpot
/// variant property: the library draws the icon as its `Icon` part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum ToastType {
    /// A `CircleCheck`.
    Success,
    /// An `Info`.
    Info,
    /// A `TriangleExclamationPoint`.
    Warning,
    /// An `OctagonX` in `destructive`: what a failed action reports.
    Error,
    /// A `LoaderCircle`.
    Loading,
}

impl ToastType {
    fn icon(self) -> IconName {
        match self {
            ToastType::Success => IconName::CircleCheck,
            ToastType::Info => IconName::Info,
            ToastType::Warning => IconName::TriangleExclamationPoint,
            ToastType::Error => IconName::OctagonX,
            ToastType::Loading => IconName::LoaderCircle,
        }
    }
}

/// A toast's surface (core-react's `Toast` with its `ToastContent`): an icon, its title over
/// its description, an action and a close button.
#[derive(IntoElement, Clone)]
pub struct Toast {
    toast_type: Option<ToastType>,
    title: Option<ToastTitle>,
    description: Option<ToastDescription>,
    action: Option<ToastAction>,
    close: ToastClose,
    timeout: Option<Duration>,
    style: StyleRefinement,
}

impl Default for Toast {
    fn default() -> Self {
        Self::new()
    }
}

impl Toast {
    /// An empty toast, hiding itself after [`DEFAULT_TIMEOUT`].
    pub fn new() -> Self {
        Self {
            toast_type: None,
            title: None,
            description: None,
            action: None,
            close: ToastClose::new(),
            timeout: Some(DEFAULT_TIMEOUT),
            style: StyleRefinement::default(),
        }
    }

    /// Its kind, which shows an icon (the `Icon` part).
    pub fn toast_type(mut self, toast_type: ToastType) -> Self {
        self.toast_type = Some(toast_type);
        self
    }

    /// The `Title` part.
    pub fn title(mut self, title: ToastTitle) -> Self {
        self.title = Some(title);
        self
    }

    /// The `Description` part, under the title.
    pub fn description(mut self, description: ToastDescription) -> Self {
        self.description = Some(description);
        self
    }

    /// The `Action` part, beside the text.
    pub fn action(mut self, action: ToastAction) -> Self {
        self.action = Some(action);
        self
    }

    /// How long it stays before hiding itself, while the pointer isn't over the stack;
    /// `None` keeps it until it is closed (core-react's `timeout: 0`).
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    fn on_close(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.close.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Toast {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        let icon = self.toast_type.map(|kind| {
            // [&_svg:not([class*='size-'])]:size-4; the error icon is text-destructive
            Icon::new(kind.icon())
                .size(space::S4)
                .flex_none()
                .when(kind == ToastType::Error, |icon| {
                    icon.text_color(s.destructive)
                })
        });
        // Toast: w-full rounded-xl border bg-popover text-popover-foreground shadow-lg
        // ToastContent: flex h-full items-center gap-3 overflow-hidden p-4
        div()
            .debug_selector(|| "toast".into())
            .w_full()
            .flex()
            .items_center()
            .gap(space::S3)
            .overflow_hidden()
            .p(space::S4)
            .rounded(s.radius_xl)
            .border_1()
            .border_color(s.border)
            .bg(s.popover)
            .text_color(s.popover_foreground)
            .shadow(shadow::LG.to_vec())
            .children(icon)
            // flex min-w-0 flex-1 flex-col gap-1
            .child(
                div()
                    .flex()
                    .min_w_0()
                    .flex_1()
                    .flex_col()
                    .gap(space::S1)
                    .children(self.title)
                    .children(self.description),
            )
            .children(self.action)
            .child(self.close)
            .refine_style(&self.style)
    }
}

/// The toast's `Title` part.
#[derive(IntoElement, Clone)]
pub struct ToastTitle {
    text: SharedString,
    style: StyleRefinement,
}

impl ToastTitle {
    /// A title reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for ToastTitle {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // text-sm font-medium
        div()
            .text_size(font_size::SM)
            .font_weight(font_weight::MEDIUM)
            .child(self.text)
            .refine_style(&self.style)
    }
}

/// The toast's `Description` part.
#[derive(IntoElement, Clone)]
pub struct ToastDescription {
    text: SharedString,
    style: StyleRefinement,
}

impl ToastDescription {
    /// A description reading `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for ToastDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // text-sm text-muted-foreground
        div()
            .text_size(font_size::SM)
            .text_color(cx.theme().semantic().muted_foreground)
            .child(self.text)
            .refine_style(&self.style)
    }
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// The toast's `Action` part: an `outline` `sm` button beside the text.
#[derive(IntoElement, Clone)]
pub struct ToastAction {
    label: SharedString,
    on_click: Option<ClickHandler>,
}

impl ToastAction {
    /// An action reading `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
        }
    }

    /// Runs when the action is pressed (core-react's `actionProps.onClick`).
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ToastAction {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // render = <Button variant="outline" size="sm" />, shrink-0
        Button::new("toast-action")
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Sm)
            .label(self.label)
            .when_some(self.on_click, |b, on_click| {
                b.on_click(move |event, window, cx| on_click(event, window, cx))
            })
    }
}

/// The toast's `Close` part: a `ghost` `icon-sm` button with an `X`, in `muted-foreground`.
#[derive(IntoElement, Clone)]
pub struct ToastClose {
    on_click: Option<ClickHandler>,
}

impl Default for ToastClose {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastClose {
    /// A close button; in a window's stack it dismisses its toast.
    pub fn new() -> Self {
        Self { on_click: None }
    }
}

impl RenderOnce for ToastClose {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = cx.theme().semantic();
        // render = <Button variant="ghost" size="icon-sm" />, relative shrink-0
        // text-muted-foreground hover:text-foreground
        Button::new("toast-close")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .icon(IconName::X)
            .text_color(s.muted_foreground)
            .when_some(self.on_click, |b, on_click| {
                b.on_click(move |event, window, cx| on_click(event, window, cx))
            })
    }
}

impl_styled!(Toast, ToastTitle, ToastDescription);

/// Adds toasts to a window's stack.
pub trait ToastExt {
    /// Shows `toast` at the bottom-right of this window, newest in front. The window's root
    /// must be a [`crate::Root`] (as [`crate::init`] expects of every window).
    fn push_toast(&mut self, toast: Toast, cx: &mut App);
}

impl ToastExt for Window {
    fn push_toast(&mut self, toast: Toast, cx: &mut App) {
        let plugin = gpui_base::Root::read(self, cx)
            .plugin::<ToastPlugin>()
            .expect("core_gpui::init registers the toast plugin before the window opens");
        plugin.update(cx, |plugin, cx| plugin.push(toast, self, cx));
    }
}

pub(crate) fn register(cx: &mut App) {
    gpui_base::Root::register_plugin(cx, ToastPlugin::new);
}

/// How often the stack's clock advances while it holds toasts.
const TICK: Duration = Duration::from_millis(50);

/// The window's toast stack: a `gpui-base` root plugin over the window's content.
pub(crate) struct ToastPlugin {
    manager: ToastManager<u64, Toast>,
    stack: ToastStackState,
    focus: FocusHandle,
    next_id: u64,
    clock: Option<Task<()>>,
}

/// Base UI's stack geometry: the gap and peek are `var(--spacing-3)`, each layer behind is
/// 10% narrower (`scale(max(0, 1 - index * 0.1))`).
fn motion() -> ToastMotion {
    ToastMotion {
        collapsed_peek: space::S3,
        expanded_gap: space::S3,
        collapsed_scale_step: 0.1,
        ..ToastMotion::sonner()
    }
}

impl ToastPlugin {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            manager: ToastManager::new(motion()),
            stack: ToastStackState::default(),
            focus: cx.focus_handle(),
            next_id: 0,
            clock: None,
        }
    }

    fn push(&mut self, toast: Toast, window: &mut Window, cx: &mut Context<Self>) {
        let id = self.next_id;
        self.next_id += 1;
        let options = ToastOptions {
            timeout: toast.timeout,
        };
        let now = cx.background_executor().now();
        self.manager.push(id, toast, options, now);
        if self.clock.is_none() {
            self.clock = Some(cx.spawn_in(window, async move |this, cx| {
                loop {
                    cx.background_executor().timer(TICK).await;
                    let running = this
                        .update(cx, |this, cx| this.advance(cx))
                        .unwrap_or(false);
                    if !running {
                        break;
                    }
                }
            }));
        }
        cx.notify();
    }

    /// Moves the toasts' clock on; returns whether any are left to keep it going.
    fn advance(&mut self, cx: &mut Context<Self>) -> bool {
        let now = cx.background_executor().now();
        let advance = self.manager.advance(now, self.stack.is_expanded());
        if advance.changed {
            cx.notify();
        }
        let running = !self.manager.is_empty();
        if !running {
            self.clock = None;
        }
        running
    }

    fn dismiss(&mut self, id: u64, cx: &mut Context<Self>) {
        let now = cx.background_executor().now();
        if self.manager.dismiss(&id, now) {
            cx.notify();
        }
    }
}

impl Render for ToastPlugin {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.manager.is_empty() {
            return Empty.into_any_element();
        }
        let toasts: Vec<(u64, Toast, gpui_base::ToastTransitionStatus)> = self
            .manager
            .iter()
            .map(|(id, toast, status)| (*id, toast.clone(), status))
            .collect();
        let stack = toasts.into_iter().fold(
            ToastStack::new("toasts", self.stack.clone())
                .motion(motion())
                .placement(Anchor::BottomRight)
                .focus_handle(self.focus.clone()),
            |stack, (id, toast, status)| {
                let dismiss = cx.listener(move |this, _: &ClickEvent, _, cx| this.dismiss(id, cx));
                let item = gpui_base::Toast::new(ElementId::Integer(id))
                    .transition_status(status)
                    // An ending toast fades while it waits to unmount.
                    .when(status == gpui_base::ToastTransitionStatus::Ending, |t| {
                        t.opacity(0.)
                    })
                    .child(toast.on_close(dismiss));
                stack.item(ElementId::Integer(id), item)
            },
        );
        // ToastViewport: pointer-events-none fixed right-4 bottom-4 w-full max-w-sm
        div()
            .absolute()
            .right(space::S4)
            .bottom(space::S4)
            .w(size::SM)
            .occlude()
            .child(stack)
            .into_any_element()
    }
}

impl RootPlugin for ToastPlugin {}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use gpui::{AppContext as _, Modifiers, TestAppContext, VisualTestContext};
    use strum::IntoEnumIterator as _;

    use super::*;

    struct Blank;

    impl Render for Blank {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    fn window(cx: &mut TestAppContext) -> &mut VisualTestContext {
        cx.update(crate::init);
        let window = cx.add_window(|window, cx| {
            let blank = cx.new(|_| Blank);
            crate::Root::new(blank, window, cx)
        });
        let cx = VisualTestContext::from_window(*window, cx).into_mut();
        draw(cx);
        cx
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    fn event_created() -> Toast {
        Toast::new().title(ToastTitle::new("Event created"))
    }

    fn showing(cx: &mut VisualTestContext) -> bool {
        cx.debug_bounds("toast").is_some()
    }

    /// Lets `time` pass on the test clock, then draws.
    fn wait(cx: &mut VisualTestContext, time: Duration) {
        cx.executor().advance_clock(time);
        draw(cx);
    }

    mod toast_type {
        use super::*;

        #[test]
        fn should_name_its_kinds_as_core_react_does() {
            let kinds: Vec<&str> = ToastType::iter().map(Into::into).collect();
            assert_eq!(kinds, ["success", "info", "warning", "error", "loading"]);
        }
    }

    mod push_toast {
        use super::*;

        #[gpui::test]
        fn should_show_the_toast_in_the_window(cx: &mut TestAppContext) {
            let cx = window(cx);
            assert!(!showing(cx));
            cx.update(|window, cx| window.push_toast(event_created(), cx));
            draw(cx);
            assert!(showing(cx));
            let toast = cx.debug_bounds("toast").expect("shown");
            let viewport = cx.update(|window, _| window.viewport_size());
            assert!(
                toast.right() <= viewport.width - space::S4,
                "at the right edge"
            );
        }

        #[gpui::test]
        fn should_hide_it_after_its_timeout(cx: &mut TestAppContext) {
            let cx = window(cx);
            let timeout = Duration::from_secs(1);
            cx.update(|window, cx| window.push_toast(event_created().timeout(Some(timeout)), cx));
            wait(cx, Duration::from_millis(500));
            assert!(showing(cx));
            // The entry transition, the timeout, then the exit transition.
            wait(cx, motion().duration + timeout + motion().exit_duration);
            assert!(!showing(cx));
        }

        #[gpui::test]
        fn should_keep_a_toast_without_a_timeout(cx: &mut TestAppContext) {
            let cx = window(cx);
            cx.update(|window, cx| window.push_toast(event_created().timeout(None), cx));
            wait(cx, Duration::from_secs(60));
            assert!(showing(cx));
        }

        #[gpui::test]
        fn should_dismiss_it_from_its_close_button(cx: &mut TestAppContext) {
            let cx = window(cx);
            cx.update(|window, cx| window.push_toast(event_created().timeout(None), cx));
            wait(cx, motion().duration);
            let toast = cx.debug_bounds("toast").expect("shown");
            // The close button: the toast's last child, 16px in from its right edge.
            let close = gpui::point(toast.right() - space::S4 - gpui::px(16.), toast.center().y);
            cx.simulate_click(close, Modifiers::default());
            wait(cx, motion().exit_duration + TICK);
            assert!(!showing(cx));
        }

        #[gpui::test]
        fn should_run_its_action(cx: &mut TestAppContext) {
            let cx = window(cx);
            let undone = Rc::new(Cell::new(0));
            let toast = event_created()
                .timeout(None)
                .action(ToastAction::new("Undo").on_click({
                    let undone = undone.clone();
                    move |_, _, _| undone.set(undone.get() + 1)
                }));
            cx.update(|window, cx| window.push_toast(toast, cx));
            wait(cx, motion().duration);
            let toast = cx.debug_bounds("toast").expect("shown");
            // The action sits left of the 32px close button and the 12px gap.
            let action = gpui::point(
                toast.right() - gpui::px(16. + 32. + 12. + 20.),
                toast.center().y,
            );
            cx.simulate_click(action, Modifiers::default());
            assert_eq!(undone.get(), 1);
        }
    }
}
