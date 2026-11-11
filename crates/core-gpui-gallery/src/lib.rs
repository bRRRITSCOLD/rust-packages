//! The core-gpui gallery: the component registry, a window to browse it, and headless
//! captures. Internal; never released.
//!
//! - `cargo run -p brrritscold-core-gpui-gallery` opens the gallery window.
//! - `cargo xtask captures` writes one PNG per variant × theme (see [`capture`]).
//! - `cargo xtask drift` checks the library against the code manifest (see [`manifest`]).

pub mod capture;
pub mod gallery;
pub mod manifest;
pub mod registry;
