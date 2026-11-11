//! The GPUI renderer of the shared design library.
//!
//! core-gpui *follows* [`@brrritscold/core-react`](https://github.com/bRRRITSCOLD/ts-packages),
//! the reference renderer: its tokens come from core-react's `tokens.json` at one exact version
//! (see [`brrritscold_core_gpui_tokens`]), and its components arrive only after core-react ships
//! them. The behaviour layer is gpui-kit's unstyled `gpui-base`; this crate is the styled layer.
//!
//! # Setup
//!
//! Call [`init`] once, then give every window a [`Root`]:
//!
//! ```no_run
//! # use gpui::{App, AppContext as _, Application, Render, WindowOptions, div};
//! # struct Workspace;
//! # impl Render for Workspace {
//! #     fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl gpui::IntoElement { div() }
//! # }
//! # fn run(app: Application) {
//! app.run(|cx: &mut App| {
//!     brrritscold_core_gpui::init(cx);
//!     cx.open_window(WindowOptions::default(), |window, cx| {
//!         let workspace = cx.new(|_| Workspace);
//!         cx.new(|cx| brrritscold_core_gpui::Root::new(workspace, window, cx))
//!     })
//!     .unwrap();
//! });
//! # }
//! ```

pub mod components;
mod fonts;
pub mod icons;
mod root;
pub mod theme;

pub use components::alert::{Alert, AlertAction, AlertDescription, AlertTitle, AlertVariant};
pub use components::avatar::{
    Avatar, AvatarBadge, AvatarFallback, AvatarGroup, AvatarGroupCount, AvatarImage, AvatarSize,
};
pub use components::badge::{Badge, BadgeVariant};
pub use components::breadcrumb::{
    Breadcrumb, BreadcrumbEllipsis, BreadcrumbLink, BreadcrumbList, BreadcrumbPage,
    BreadcrumbSeparator,
};
pub use components::button::{Button, ButtonSize, ButtonVariant};
pub use components::card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardSize, CardTitle,
};
pub use components::label::Label;
pub use components::pagination::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationLink, PaginationNext,
    PaginationPrevious,
};
pub use components::separator::{Separator, SeparatorOrientation};
pub use components::skeleton::Skeleton;
pub use components::table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};
pub use gpui_base::Root;
pub use icons::{Icon, IconName};
pub use theme::{ActiveTheme, Mode, Theme, ThemePreference};

use gpui::App;

/// Sets core-gpui up for this application. Required, once, before the first window opens.
///
/// It initialises `gpui-base`, registers the bundled Inter fonts, installs the [`Theme`]
/// (preference [`ThemePreference::System`]) and registers the root plugin that feeds each
/// window's OS appearance into it.
pub fn init(cx: &mut App) {
    gpui_base::init(cx);
    fonts::register(cx);
    theme::init(cx);
    root::register(cx);
}
