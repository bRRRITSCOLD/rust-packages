//! The core-gpui gallery: the component registry, a window to browse it, and headless
//! captures. Internal; never released.
//!
//! - `cargo run -p brrritscold-core-gpui-gallery` opens the gallery window.
//! - `cargo xtask captures` writes one PNG per variant × theme (see [`capture`]).

pub mod capture;
pub mod gallery;
pub mod registry;
