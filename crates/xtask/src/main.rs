//! Workspace tasks for rust-packages, run through the cargo aliases in `.cargo/config.toml`.
//!
//! - `cargo xtask drift` runs the drift check: the registry's code manifest against the
//!   vendored library snapshot and tokens, with core-react's checker at its pin.
//! - `cargo xtask captures [--out <dir>]` writes the gallery's headless captures (local only).
//! - `cargo xtask captures compare [--penpot-theme light|dark]` builds the design comparison
//!   sheet: core-react's Storybook | GPUI | Penpot.
//! - `cargo xtask test <tier> [--ci] [-- <nextest args>]` runs one test tier.
//! - `cargo xtask tokens update <version|latest>` vendors core-react's tokens at one exact version.
//! - `cargo xtask tokens generate` regenerates `generated.rs` from the vendored tokens.
//! - `cargo xtask gpui update <version|latest> [--body <file>]` moves the GPUI pin.

mod captures;
mod drift;
mod gpui;
mod test_chain;
mod tokens;

use std::process::ExitCode;

use anyhow::{Context, bail};

use test_chain::Tier;

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("xtask: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> anyhow::Result<ExitCode> {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("test") => {
            let tier: Tier = args
                .next()
                .context("usage: cargo xtask test <tier> [--ci]")?
                .parse()?;
            let rest: Vec<String> = args.collect();
            let ci = rest.iter().any(|a| a == "--ci");
            let passthrough: Vec<String> = rest
                .into_iter()
                .filter(|a| a != "--ci" && a != "--")
                .collect();
            Ok(test_chain::run_tier(tier, ci, &passthrough))
        }
        Some("tokens") => tokens::run(args).map(|()| ExitCode::SUCCESS),
        Some("gpui") => gpui::run(args).map(|()| ExitCode::SUCCESS),
        Some("captures") => captures::run(args).map(|()| ExitCode::SUCCESS),
        Some("drift") => drift::run(args),
        Some(other) => bail!("unknown task `{other}`"),
        None => bail!("usage: cargo xtask <task>; tasks: test, tokens, gpui, captures, drift"),
    }
}
