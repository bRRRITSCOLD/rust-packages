//! `cargo xtask drift`: the drift check, library → code, with core-react's checker.
//!
//! 1. The gallery writes the code manifest (`manifest --out target/code-manifest.json`): the
//!    registry and the icons in core-react's vocabulary. The gallery owns the registry, so
//!    xtask stays free of GPUI.
//! 2. `design-drift check` from `@brrritscold/design-core-node`, at the exact pin in the
//!    tokens crate's `[package.metadata.design-core-node]`, installed under `target/` on first
//!    use, checks the vendored `library.snapshot.json` and `tokens.json` against it with the
//!    Rust-owned `design/drift.config.json` (allowlist and `pending`).
//!
//! The exit code is the checker's: 1 on drift, 2 on input it can't use. Pending gaps and
//! coverage gaps only warn. In CI the report lands in the step summary.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, bail};
use serde_json::Value;
use toml_edit::{DocumentMut, Item};

use crate::tokens::{SNAPSHOT_FILE, TOKENS_CARGO_TOML, TOKENS_FILE, workspace_root};

const MANIFEST_FILE: &str = "target/code-manifest.json";
const CONFIG_FILE: &str = "design/drift.config.json";
/// Where the checker is installed: an npm prefix of its own, rebuilt when the pin moves.
const CHECKER_PREFIX: &str = "target/design-drift";

/// The checker pin, `[package.metadata.design-core-node]` beside the core-react pin.
#[derive(Debug, PartialEq, Eq)]
struct CheckerPin {
    /// Exact npm version of `@brrritscold/design-core-node`, the `design-drift` CLI.
    version: String,
    /// Exact npm version of `@brrritscold/design-core-js`, the rules themselves.
    /// design-core-node takes it by a caret range, so it's pinned too.
    design_core_js: String,
}

/// `cargo xtask drift`.
pub fn run(mut args: impl Iterator<Item = String>) -> Result<ExitCode> {
    if args.next().is_some() {
        bail!("usage: cargo xtask drift");
    }
    let root = workspace_root();
    let pin = read_checker_pin(&std::fs::read_to_string(root.join(TOKENS_CARGO_TOML))?)?;
    write_manifest(&root)?;
    let cli = install_checker(&root, &pin)?;
    let status = Command::new("node")
        .current_dir(&root)
        .arg(cli)
        .arg("check")
        .args(["--manifest", MANIFEST_FILE])
        .args(["--snapshot", SNAPSHOT_FILE])
        .args(["--tokens", TOKENS_FILE])
        .args(["--config", CONFIG_FILE])
        .status()
        .context("running node (the drift check needs it)")?;
    match status.code() {
        Some(0) => Ok(ExitCode::SUCCESS),
        Some(code) => {
            eprintln!("xtask: design-drift check failed (exit {code})");
            Ok(ExitCode::from(u8::try_from(code).unwrap_or(1)))
        }
        None => bail!("design-drift check was killed ({status})"),
    }
}

fn read_checker_pin(cargo_toml: &str) -> Result<CheckerPin> {
    let doc: DocumentMut = cargo_toml.parse()?;
    let table = doc
        .get("package")
        .and_then(|package| package.get("metadata"))
        .and_then(|metadata| metadata.get("design-core-node"))
        .and_then(Item::as_table)
        .context("the tokens crate has no [package.metadata.design-core-node]")?;
    let field = |key: &str| -> Result<String> {
        let version = table
            .get(key)
            .and_then(Item::as_str)
            .with_context(|| format!("[package.metadata.design-core-node] has no `{key}`"))?;
        if !version.starts_with(|c: char| c.is_ascii_digit()) {
            bail!(
                "[package.metadata.design-core-node] `{key}` is `{version}`, not an exact version"
            );
        }
        Ok(version.to_owned())
    };
    Ok(CheckerPin {
        version: field("version")?,
        design_core_js: field("design-core-js")?,
    })
}

/// Runs the gallery's `manifest` into [`MANIFEST_FILE`].
fn write_manifest(root: &Path) -> Result<()> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .current_dir(root)
        .args(["run", "--quiet", "--locked", "--package"])
        .args(["brrritscold-core-gpui-gallery", "--", "manifest", "--out"])
        .arg(root.join(MANIFEST_FILE))
        .status()
        .context("running the gallery's manifest")?;
    if !status.success() {
        bail!("the gallery's manifest failed ({status})");
    }
    Ok(())
}

/// Installs the pinned checker under [`CHECKER_PREFIX`] unless it's there already, and returns
/// the `design-drift` CLI script.
fn install_checker(root: &Path, pin: &CheckerPin) -> Result<PathBuf> {
    let prefix = root.join(CHECKER_PREFIX);
    let scope = prefix.join("node_modules/@brrritscold");
    let node = scope.join("design-core-node");
    let installed = |dir: &Path| {
        std::fs::read_to_string(dir.join("package.json"))
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .and_then(|package| package["version"].as_str().map(str::to_owned))
    };
    if installed(&node).as_deref() != Some(&pin.version)
        || installed(&scope.join("design-core-js")).as_deref() != Some(&pin.design_core_js)
    {
        if prefix.exists() {
            std::fs::remove_dir_all(&prefix)
                .with_context(|| format!("clearing {}", prefix.display()))?;
        }
        std::fs::create_dir_all(&prefix)?;
        let packages = [
            format!("@brrritscold/design-core-node@{}", pin.version),
            format!("@brrritscold/design-core-js@{}", pin.design_core_js),
        ];
        let npm = Command::new("npm")
            .args(["install", "--no-audit", "--no-fund", "--silent", "--prefix"])
            .arg(&prefix)
            .args(&packages)
            .status()
            .context("running npm (the drift check needs it)")?;
        if !npm.success() {
            bail!("npm install {} failed ({npm})", packages.join(" "));
        }
    }
    // A nested copy means design-core-node didn't accept the pinned design-core-js.
    if node
        .join("node_modules/@brrritscold/design-core-js")
        .exists()
    {
        bail!(
            "@brrritscold/design-core-node@{} doesn't accept @brrritscold/design-core-js@{}: \
             move both pins together",
            pin.version,
            pin.design_core_js
        );
    }
    Ok(node.join("bin/design-drift.mjs"))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod checker_pin {
        use super::*;

        const PIN: &str = "[package]\nname = \"x\"\n\n\
            [package.metadata.design-core-node]\nversion = \"0.1.0\"\ndesign-core-js = \"0.1.2\"\n";

        #[test]
        fn should_read_both_versions() {
            assert_eq!(
                read_checker_pin(PIN).unwrap(),
                CheckerPin {
                    version: "0.1.0".into(),
                    design_core_js: "0.1.2".into(),
                }
            );
        }

        #[test]
        fn should_reject_a_range() {
            let ranged = PIN.replace("\"0.1.0\"", "\"^0.1.0\"");
            let err = read_checker_pin(&ranged).unwrap_err();
            assert!(err.to_string().contains("not an exact version"), "{err}");
        }

        #[test]
        fn should_fail_without_the_table() {
            assert!(read_checker_pin("[package]\nname = \"x\"\n").is_err());
        }

        /// The checked-in pin: a blocking check runs exactly one checker.
        #[test]
        fn should_be_exact_in_the_tokens_crate() {
            let root = workspace_root();
            let text = std::fs::read_to_string(root.join(TOKENS_CARGO_TOML)).unwrap();
            read_checker_pin(&text).unwrap();
        }
    }
}
