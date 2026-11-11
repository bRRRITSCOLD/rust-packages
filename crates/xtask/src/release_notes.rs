//! `cargo xtask release-notes`: reads one rendered changelog entry on stdin and writes it back
//! with the GPUI pin and the core-react pin under its heading.
//!
//! release-plz runs it as a changelog postprocessor (`[changelog]` in `release-plz.toml`), so
//! every release's `CHANGELOG.md` entry states both pins, and so does its GitHub release, whose
//! body release-plz copies from that entry. The pins are read from the checkout: the GPUI pin
//! from `[workspace.dependencies]`, the core-react pin and the drift checker pin from the tokens
//! crate's `[package.metadata]`.

use std::io::Read as _;

use anyhow::{Context, Result, bail};
use toml_edit::{DocumentMut, Value};

use crate::drift::read_checker_pin;
use crate::gpui::current_kit;
use crate::tokens::{TOKENS_CARGO_TOML, parse_pin, workspace_root};

/// `cargo xtask release-notes < entry.md`.
pub fn run(mut args: impl Iterator<Item = String>) -> Result<()> {
    if args.next().is_some() {
        bail!("usage: cargo xtask release-notes < <changelog entry>");
    }
    let root = workspace_root();
    let mut entry = String::new();
    std::io::stdin()
        .read_to_string(&mut entry)
        .context("reading the changelog entry from stdin")?;
    let workspace = std::fs::read_to_string(root.join("Cargo.toml"))?;
    let tokens = std::fs::read_to_string(root.join(TOKENS_CARGO_TOML))?;
    print!("{}", with_pins(&entry, &workspace, &tokens)?);
    Ok(())
}

/// `entry` with the two pin paragraphs inserted after its `## [<version>]` heading.
fn with_pins(entry: &str, workspace_toml: &str, tokens_toml: &str) -> Result<String> {
    let (heading_end, version) =
        release_heading(entry).context("the changelog entry has no `## [<version>]` heading")?;
    let gpui = gpui_pin(&workspace_toml.parse()?, version)?;
    let core_react = core_react_pin(tokens_toml)?;
    Ok(format!(
        "{}\n{gpui}\n\n{core_react}\n{}",
        &entry[..heading_end],
        &entry[heading_end..]
    ))
}

/// Where the first `## [<version>]` line ends (after its newline), and the version.
fn release_heading(entry: &str) -> Option<(usize, &str)> {
    let mut end = 0;
    for line in entry.split_inclusive('\n') {
        end += line.len();
        let version = line
            .strip_prefix("## [")
            .and_then(|rest| rest.split_once(']'))
            .map(|(version, _)| version);
        if let Some(version) = version.filter(|v| v.starts_with(|c: char| c.is_ascii_digit())) {
            return Some((end, version));
        }
    }
    None
}

/// The GPUI pin paragraph:`gpui-base` and every `gpui-pre*` line, as apps copy them.
fn gpui_pin(workspace: &DocumentMut, version: &str) -> Result<String> {
    let kit = current_kit(workspace)?;
    let deps = workspace["workspace"]["dependencies"]
        .as_table_like()
        .context("no [workspace.dependencies]")?;
    // (manifest line, package, `=` version) for each `gpui-pre*` dependency, in manifest order.
    let mut lines = Vec::new();
    for (key, item) in deps.iter() {
        let Some(table) = item.as_inline_table() else {
            continue;
        };
        let field = |name: &str| table.get(name).and_then(Value::as_str);
        if let (Some(package), Some(requirement)) = (field("package"), field("version"))
            && package.starts_with("gpui-pre")
        {
            lines.push((
                format!("`{key} = {{ package = \"{package}\", version = \"{requirement}\" }}`"),
                package,
                requirement,
            ));
        }
    }
    let at = lines
        .iter()
        .position(|(_, package, _)| *package == "gpui-pre")
        .context("[workspace.dependencies] has no `gpui-pre` (the `gpui` line)")?;
    let (gpui, _, pre) = lines.remove(at);
    let derived = if lines.iter().all(|(_, _, requirement)| *requirement == pre) {
        format!("{gpui} and its `gpui-pre-*` siblings at `{pre}`")
    } else {
        let mut all: Vec<String> = std::iter::once(gpui)
            .chain(lines.into_iter().map(|(line, _, _)| line))
            .collect();
        let last = all.pop().expect("gpui-pre is in the list");
        format!("{} and {last}", all.join(", "))
    };
    Ok(format!(
        "**GPUI pin:** gpui-kit {kit} (`gpui-base = \"={kit}\"`, {derived}). \
         Apps copy these lines next to the `v{version}` tag."
    ))
}

/// The core-react pin paragraph, with the Lucide version and the drift checker it carries.
fn core_react_pin(tokens_toml: &str) -> Result<String> {
    let pin = parse_pin(tokens_toml)?;
    let checker = read_checker_pin(tokens_toml)?;
    let checker = if checker.version == checker.design_core_js {
        format!("design-core-node/js {}", checker.version)
    } else {
        format!(
            "design-core-node {} + design-core-js {}",
            checker.version, checker.design_core_js
        )
    };
    Ok(format!(
        "**core-react pin:** {} (tokens, library snapshot, Lucide {}, drift checker {checker}).",
        pin.version, pin.lucide
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORKSPACE: &str = r#"[workspace.dependencies]
brrritscold-core-gpui = { path = "crates/core-gpui" }
gpui-base = "=0.9.1"
gpui = { package = "gpui-pre", version = "=0.3.12" }
gpui-macros = { package = "gpui-pre-macros", version = "=0.3.12" }
gpui-sum-tree = { package = "gpui-pre-sum-tree", version = "=0.3.12" }
anyhow = "1"
"#;

    const TOKENS: &str = r#"[package]
name = "brrritscold-core-gpui-tokens"

[package.metadata.core-react]
version = "1.1.0"
tokens-sha256 = "aa"
snapshot-sha256 = "bb"
lucide = "1.68.0"

[package.metadata.design-core-node]
version = "0.1.0"
design-core-js = "0.1.0"
"#;

    const ENTRY: &str = "## [0.1.1](https://github.com/bRRRITSCOLD/rust-packages/compare/v0.1.0...v0.1.1) - 2026-11-11

### Added

- *(tokens)* vendor core-react 1.2.0 tokens
";

    mod with_pins {
        use super::*;

        #[test]
        fn should_put_both_pins_between_the_heading_and_the_first_group() {
            let expected = "## [0.1.1](https://github.com/bRRRITSCOLD/rust-packages/compare/v0.1.0...v0.1.1) - 2026-11-11

**GPUI pin:** gpui-kit 0.9.1 (`gpui-base = \"=0.9.1\"`, `gpui = { package = \"gpui-pre\", version = \"=0.3.12\" }` and its `gpui-pre-*` siblings at `=0.3.12`). Apps copy these lines next to the `v0.1.1` tag.

**core-react pin:** 1.1.0 (tokens, library snapshot, Lucide 1.68.0, drift checker design-core-node/js 0.1.0).

### Added

- *(tokens)* vendor core-react 1.2.0 tokens
";
            assert_eq!(with_pins(ENTRY, WORKSPACE, TOKENS).unwrap(), expected);
        }

        #[test]
        fn should_name_each_gpui_pre_sibling_when_their_versions_differ() {
            let workspace = WORKSPACE.replace(
                "gpui-pre-sum-tree\", version = \"=0.3.12\"",
                "gpui-pre-sum-tree\", version = \"=0.3.13\"",
            );
            let notes = with_pins(ENTRY, &workspace, TOKENS).unwrap();
            assert!(notes.contains(
                "`gpui = { package = \"gpui-pre\", version = \"=0.3.12\" }`, \
                 `gpui-macros = { package = \"gpui-pre-macros\", version = \"=0.3.12\" }` and \
                 `gpui-sum-tree = { package = \"gpui-pre-sum-tree\", version = \"=0.3.13\" }`). "
            ));
        }

        #[test]
        fn should_name_each_checker_package_when_their_versions_differ() {
            let tokens = TOKENS.replace("design-core-js = \"0.1.0\"", "design-core-js = \"0.1.2\"");
            let notes = with_pins(ENTRY, WORKSPACE, &tokens).unwrap();
            assert!(
                notes.contains("drift checker design-core-node 0.1.0 + design-core-js 0.1.2).")
            );
        }

        #[test]
        fn should_fail_without_a_release_heading() {
            let err = with_pins("## [Unreleased]\n", WORKSPACE, TOKENS).unwrap_err();
            assert!(err.to_string().contains("no `## [<version>]` heading"));
        }

        #[test]
        fn should_fail_without_gpui_pre() {
            let workspace = "[workspace.dependencies]\ngpui-base = \"=0.9.1\"\n";
            assert!(with_pins(ENTRY, workspace, TOKENS).is_err());
        }

        /// The checked-in manifests: a release PR fails if either pin can't be read.
        #[test]
        fn should_read_the_checked_in_pins() {
            let root = workspace_root();
            let workspace = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
            let tokens = std::fs::read_to_string(root.join(TOKENS_CARGO_TOML)).unwrap();
            let notes = with_pins(ENTRY, &workspace, &tokens).unwrap();
            assert!(notes.contains("**GPUI pin:** gpui-kit "));
            assert!(notes.contains("**core-react pin:** "));
        }
    }
}
