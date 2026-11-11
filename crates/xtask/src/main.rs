//! Workspace tasks for rust-packages, run through the cargo aliases in `.cargo/config.toml`.
//!
//! `cargo xtask test <tier> [--ci] [-- <nextest args>]` runs one test tier.

mod test_chain;

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
        Some(other) => bail!("unknown task `{other}`"),
        None => bail!("usage: cargo xtask <task>; tasks: test"),
    }
}
