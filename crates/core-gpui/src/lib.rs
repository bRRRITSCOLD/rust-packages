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

pub use components::accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
pub use components::alert::{Alert, AlertAction, AlertDescription, AlertTitle, AlertVariant};
pub use components::alert_dialog::{
    AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogContentSize,
    AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogMedia,
    AlertDialogOverlay, AlertDialogTitle,
};
pub use components::avatar::{
    Avatar, AvatarBadge, AvatarFallback, AvatarGroup, AvatarGroupCount, AvatarImage, AvatarSize,
};
pub use components::badge::{Badge, BadgeVariant};
pub use components::breadcrumb::{
    Breadcrumb, BreadcrumbEllipsis, BreadcrumbLink, BreadcrumbList, BreadcrumbPage,
    BreadcrumbSeparator,
};
pub use components::button::{Button, ButtonSize, ButtonVariant};
pub use components::calendar::{Calendar, CalendarDayButton};
pub use components::card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardSize, CardTitle,
};
pub use components::checkbox::Checkbox;
pub use components::combobox::{
    Combobox, ComboboxChip, ComboboxChips, ComboboxContent, ComboboxEmpty, ComboboxGroup,
    ComboboxInput, ComboboxItem, ComboboxLabel, ComboboxSeparator,
};
pub use components::data_table_column_header::{DataTableColumnHeader, DataTableSort};
pub use components::data_table_pagination::DataTablePagination;
pub use components::data_table_toolbar::DataTableToolbar;
pub use components::data_table_view_options::{DataTableViewColumn, DataTableViewOptions};
pub use components::date_picker::DatePicker;
pub use components::dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHandle,
    DialogHeader, DialogOverlay, DialogTitle,
};
pub use components::dropdown_menu::{
    DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuItem,
    DropdownMenuItemVariant, DropdownMenuLabel, DropdownMenuRadioItem, DropdownMenuSeparator,
    DropdownMenuShortcut, DropdownMenuSubContent, DropdownMenuSubTrigger,
};
pub use components::field::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldLegendVariant, FieldOrientation, FieldSeparator, FieldSet, FieldTitle,
};
pub use components::input::{Input, InputState};
pub use components::label::Label;
pub use components::pagination::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationLink, PaginationNext,
    PaginationPrevious,
};
pub use components::popover::{
    Popover, PopoverContent, PopoverDescription, PopoverHeader, PopoverTitle,
};
pub use components::radio_group::{RadioGroup, RadioGroupItem};
pub use components::select::{
    Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectSeparator, SelectTrigger,
    SelectTriggerSize,
};
pub use components::separator::{Separator, SeparatorOrientation};
pub use components::sheet::{
    Sheet, SheetContent, SheetContentSide, SheetDescription, SheetFooter, SheetHeader, SheetTitle,
};
pub use components::skeleton::Skeleton;
pub use components::switch::{Switch, SwitchSize};
pub use components::table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};
pub use components::tabs::{
    Tabs, TabsContent, TabsList, TabsListVariant, TabsOrientation, TabsTrigger,
};
pub use components::textarea::{Textarea, TextareaState};
pub use components::toast::{
    Toast, ToastAction, ToastClose, ToastDescription, ToastExt, ToastTitle, ToastType,
};
pub use components::toggle::{Toggle, ToggleSize, ToggleVariant};
pub use components::toggle_group::{
    ToggleGroup, ToggleGroupItem, ToggleGroupOrientation, ToggleGroupSpacing,
};
pub use components::tooltip::{Tooltip, TooltipContent, TooltipContentSide};
pub use gpui_base::Root;
pub use icons::{Icon, IconName};
pub use theme::{ActiveTheme, Mode, Theme, ThemePreference};

use gpui::App;

/// Sets core-gpui up for this application. Required, once, before the first window opens.
///
/// It initialises `gpui-base`, registers the bundled Inter fonts, installs the [`Theme`]
/// (preference [`ThemePreference::System`]) and registers the root plugins: the one that feeds
/// each window's OS appearance into it, and the one hosting each window's toasts
/// ([`ToastExt::push_toast`]).
pub fn init(cx: &mut App) {
    gpui_base::init(cx);
    fonts::register(cx);
    theme::init(cx);
    root::register(cx);
    components::toast::register(cx);
    components::dropdown_menu::popup_menu::init(cx);
}
