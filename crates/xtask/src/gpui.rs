//! `cargo xtask gpui update <version|latest> [--body <file>]`: moves the GPUI pin.
//!
//! The *GPUI pin* is the gpui-kit version, `gpui-base = "=<kit>"`. Every `gpui-pre*` line in
//! `[workspace.dependencies]` is derived from that `gpui-base` release's own `=` requirements
//! and never chosen on its own. With `--body`, it also writes the bump PR's body, listing the
//! vendored gpui-component files whose upstream source changed (information only).

use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use toml_edit::{DocumentMut, Item, Value, value};

use crate::tokens::{extract_where, workspace_root, write};

const KIT: &str = "gpui-base";
const COMPONENT: &str = "gpui-component";
const USER_AGENT: &str = "rust-packages-xtask (https://github.com/bRRRITSCOLD/rust-packages)";
/// A vendored file starts with this, then `<version>: <upstream path>`.
pub const VENDORED_HEADER: &str = "// Vendored from gpui-component ";

/// `cargo xtask gpui update <version|latest> [--body <file>]`.
pub fn run(mut args: impl Iterator<Item = String>) -> Result<()> {
    let usage = "usage: cargo xtask gpui update <version|latest> [--body <file>]";
    if args.next().as_deref() != Some("update") {
        bail!(usage);
    }
    let spec = args.next().context(usage)?;
    let body = match (args.next().as_deref(), args.next()) {
        (Some("--body"), Some(path)) => Some(PathBuf::from(path)),
        (None, _) => None,
        _ => bail!(usage),
    };
    update(&spec, body.as_deref())
}

fn update(spec: &str, body: Option<&Path>) -> Result<()> {
    let root = workspace_root();
    let manifest_path = root.join("Cargo.toml");
    let mut manifest: DocumentMut = std::fs::read_to_string(&manifest_path)?.parse()?;
    let old = current_kit(&manifest)?;
    let new = match spec {
        "latest" => latest_version(KIT)?,
        version => version.to_owned(),
    };
    let requirements = exact_requirements(&get_json(&format!(
        "https://crates.io/api/v1/crates/{KIT}/{new}/dependencies"
    ))?)?;
    let changed = rewrite_pin(&mut manifest, &new, &requirements)?;
    write(&manifest_path, &manifest.to_string())?;
    println!("GPUI pin: gpui-kit {old} -> {new}");
    for line in &changed {
        println!("  {line}");
    }
    if let Some(body) = body {
        let vendored = vendored_files(&root)?;
        let changes = if vendored.is_empty() || old == new {
            Vec::new()
        } else {
            upstream_changes(&vendored, &old, &new)?
        };
        write(body, &pr_body(&old, &new, &changed, &vendored, &changes))?;
    }
    Ok(())
}

pub(crate) fn current_kit(manifest: &DocumentMut) -> Result<String> {
    let requirement = manifest["workspace"]["dependencies"][KIT]
        .as_str()
        .context("[workspace.dependencies] has no `gpui-base = \"=<version>\"`")?;
    Ok(requirement.trim_start_matches('=').to_owned())
}

/// `gpui-pre*` crate → its `=` version, from a crates.io dependencies response. Dev
/// dependencies count: `-platform` and `-reqwest-client` are pinned there.
fn exact_requirements(response: &serde_json::Value) -> Result<BTreeMap<String, String>> {
    let mut pins = BTreeMap::new();
    for dep in response["dependencies"]
        .as_array()
        .context("crates.io returned no dependencies")?
    {
        let (Some(name), Some(req)) = (dep["crate_id"].as_str(), dep["req"].as_str()) else {
            continue;
        };
        let Some(version) = req
            .strip_prefix('=')
            .filter(|_| name.starts_with("gpui-pre"))
        else {
            continue;
        };
        if let Some(previous) = pins.insert(name.to_owned(), version.to_owned())
            && previous != version
        {
            bail!("{KIT} pins {name} at both ={previous} and ={version}");
        }
    }
    if !pins.contains_key("gpui-pre") {
        bail!("{KIT} doesn't pin gpui-pre exactly");
    }
    Ok(pins)
}

/// Rewrites `gpui-base` and every `gpui-pre*` line to the new release. Returns what changed.
fn rewrite_pin(
    manifest: &mut DocumentMut,
    kit: &str,
    requirements: &BTreeMap<String, String>,
) -> Result<Vec<String>> {
    let deps = manifest["workspace"]["dependencies"]
        .as_table_like_mut()
        .context("no [workspace.dependencies]")?;
    let mut changed = Vec::new();
    let old_kit = deps
        .get(KIT)
        .and_then(Item::as_str)
        .unwrap_or_default()
        .to_owned();
    let new_kit = format!("={kit}");
    if old_kit != new_kit {
        changed.push(format!("{KIT} {old_kit} -> {new_kit}"));
    }
    // Edit in place: `insert` would move the key to the end and drop the comment above it.
    match deps.get_mut(KIT).and_then(Item::as_value_mut) {
        Some(existing) => *existing = new_kit.into(),
        None => {
            deps.insert(KIT, value(new_kit));
        }
    }

    let keys: Vec<String> = deps.iter().map(|(key, _)| key.to_owned()).collect();
    for key in keys {
        let item = deps.get_mut(&key).expect("key listed above");
        let Some(table) = item.as_inline_table_mut() else {
            continue;
        };
        let Some(package) = table
            .get("package")
            .and_then(Value::as_str)
            .map(str::to_owned)
        else {
            continue;
        };
        if !package.starts_with("gpui-pre") {
            continue;
        }
        let new = requirements.get(&package).with_context(|| {
            format!("{KIT} {kit} doesn't pin {package}; drop or re-derive `{key}`")
        })?;
        let old = table
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let new = format!("={new}");
        if old != new {
            changed.push(format!("{package} {old} -> {new}"));
        }
        table.insert("version", new.into());
    }
    Ok(changed)
}

/// `(local path, upstream version, upstream path)` for every vendored gpui-component file.
fn vendored_files(root: &Path) -> Result<Vec<(String, String, String)>> {
    let mut found = Vec::new();
    let mut stack = vec![root.join("crates")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let mut first = String::new();
                std::fs::File::open(&path)?
                    .take(512)
                    .read_to_string(&mut first)
                    .ok();
                if let Some((version, upstream)) = parse_header(&first) {
                    let local = path.strip_prefix(root)?.to_string_lossy().into_owned();
                    found.push((local, version, upstream));
                }
            }
        }
    }
    found.sort();
    Ok(found)
}

/// `// Vendored from gpui-component 0.9.1: src/menu/popup_menu.rs`.
fn parse_header(text: &str) -> Option<(String, String)> {
    let line = text.lines().next()?.strip_prefix(VENDORED_HEADER)?;
    let (version, upstream) = line.split_once(": ")?;
    Some((version.trim().to_owned(), upstream.trim().to_owned()))
}

/// Upstream paths whose source differs between the two gpui-component versions.
fn upstream_changes(
    vendored: &[(String, String, String)],
    old: &str,
    new: &str,
) -> Result<Vec<String>> {
    let wanted: Vec<&str> = vendored.iter().map(|(_, _, up)| up.as_str()).collect();
    let fetch = |version: &str| -> Result<BTreeMap<String, Vec<u8>>> {
        let bytes = get_bytes(&format!(
            "https://crates.io/api/v1/crates/{COMPONENT}/{version}/download"
        ))?;
        let prefix = format!("{COMPONENT}-{version}/");
        let files = extract_where(&bytes, |path| {
            path.strip_prefix(&prefix)
                .is_some_and(|rel| wanted.contains(&rel))
        })?;
        Ok(files
            .into_iter()
            .map(|(path, contents)| (path[prefix.len()..].to_owned(), contents))
            .collect())
    };
    let (before, after) = (fetch(old)?, fetch(new)?);
    Ok(wanted
        .into_iter()
        .filter(|path| before.get(*path) != after.get(*path))
        .map(str::to_owned)
        .collect())
}

fn pr_body(
    old: &str,
    new: &str,
    changed: &[String],
    vendored: &[(String, String, String)],
    upstream_changed: &[String],
) -> String {
    let mut body = format!(
        "Moves the GPUI pin **gpui-kit {old} → {new}**. Every `gpui-pre*` line is derived from \
         `gpui-base {new}`'s own `=` requirements.\n\n\
         Advisory: merge when green and checked; it may stay open, and versions may be skipped. \
         Fix any breakage on this branch.\n\n\
         **Required before merge:** a local `cargo xtask captures compare`, linked here. A GPUI \
         bump can always change rendering.\n\n## Lines\n\n"
    );
    for line in changed {
        body.push_str(&format!("- `{line}`\n"));
    }
    body.push_str("\n## Vendored gpui-component files\n\n");
    if vendored.is_empty() {
        body.push_str("None yet.\n");
    }
    for (local, version, upstream) in vendored {
        let mark = if upstream_changed.contains(upstream) {
            "⚠️ upstream changed"
        } else {
            "unchanged upstream"
        };
        body.push_str(&format!(
            "- `{local}` (from {version}, `{upstream}`): {mark}\n"
        ));
    }
    body.push_str(
        "\nInformation only: re-sync stays manual, and a file's header version changes only \
         when someone re-syncs it.\n",
    );
    body
}

fn latest_version(name: &str) -> Result<String> {
    let response = get_json(&format!("https://crates.io/api/v1/crates/{name}"))?;
    response["crate"]["max_stable_version"]
        .as_str()
        .map(str::to_owned)
        .with_context(|| format!("crates.io has no stable {name}"))
}

fn get_json(url: &str) -> Result<serde_json::Value> {
    Ok(ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("GET {url}"))?
        .body_mut()
        .read_json()?)
}

fn get_bytes(url: &str) -> Result<Vec<u8>> {
    Ok(ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("GET {url}"))?
        .body_mut()
        .with_config()
        .limit(64 * 1024 * 1024)
        .read_to_vec()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requirements(deps: serde_json::Value) -> Result<BTreeMap<String, String>> {
        exact_requirements(&serde_json::json!({ "dependencies": deps }))
    }

    mod exact_requirements {
        use super::*;

        #[test]
        fn should_collect_exact_gpui_pre_pins_from_normal_and_dev_dependencies() {
            let pins = requirements(serde_json::json!([
                {"crate_id": "gpui-pre", "req": "=0.3.12", "kind": "normal"},
                {"crate_id": "gpui-pre", "req": "=0.3.12", "kind": "dev"},
                {"crate_id": "gpui-pre-platform", "req": "=0.3.12", "kind": "dev"},
                {"crate_id": "gpui-fast", "req": "^0.1.14", "kind": "dev"},
                {"crate_id": "anyhow", "req": "^1", "kind": "normal"}
            ]))
            .unwrap();
            assert_eq!(pins.len(), 2);
            assert_eq!(pins["gpui-pre-platform"], "0.3.12");
        }

        #[test]
        fn should_fail_when_gpui_pre_is_not_pinned_exactly() {
            let result = requirements(serde_json::json!([
                {"crate_id": "gpui-pre", "req": "^0.3", "kind": "normal"}
            ]));
            assert!(result.is_err());
        }

        #[test]
        fn should_fail_on_two_different_pins_of_one_crate() {
            let result = requirements(serde_json::json!([
                {"crate_id": "gpui-pre", "req": "=0.3.12", "kind": "normal"},
                {"crate_id": "gpui-pre", "req": "=0.3.13", "kind": "dev"}
            ]));
            assert!(result.is_err());
        }
    }

    mod rewrite_pin {
        use super::*;

        const MANIFEST: &str = r#"[workspace.dependencies]
# The GPUI pin.
gpui-base = "=0.9.1"
gpui = { package = "gpui-pre", version = "=0.3.12" }
gpui-platform = { package = "gpui-pre-platform", version = "=0.3.12" }
anyhow = "1"
"#;

        #[test]
        fn should_rewrite_the_kit_and_every_derived_line_and_keep_comments() {
            let mut manifest: DocumentMut = MANIFEST.parse().unwrap();
            let pins = BTreeMap::from([
                ("gpui-pre".to_owned(), "0.3.13".to_owned()),
                ("gpui-pre-platform".to_owned(), "0.3.13".to_owned()),
            ]);
            let changed = rewrite_pin(&mut manifest, "0.9.2", &pins).unwrap();
            let text = manifest.to_string();
            assert!(text.contains("# The GPUI pin."));
            assert!(text.contains(r#"gpui-base = "=0.9.2""#));
            assert!(text.contains(r#"gpui = { package = "gpui-pre", version = "=0.3.13" }"#));
            assert!(text.contains(r#"anyhow = "1""#));
            assert_eq!(changed.len(), 3);
        }

        #[test]
        fn should_fail_when_the_new_kit_no_longer_pins_a_crate_we_use() {
            let mut manifest: DocumentMut = MANIFEST.parse().unwrap();
            let pins = BTreeMap::from([("gpui-pre".to_owned(), "0.3.13".to_owned())]);
            let err = rewrite_pin(&mut manifest, "0.9.2", &pins).unwrap_err();
            assert!(format!("{err:#}").contains("gpui-pre-platform"));
        }
    }

    mod vendored_header {
        use super::*;

        #[test]
        fn should_parse_version_and_upstream_path() {
            let header = "// Vendored from gpui-component 0.9.1: src/menu.rs\n//! docs";
            assert_eq!(
                parse_header(header),
                Some(("0.9.1".into(), "src/menu.rs".into()))
            );
        }

        #[test]
        fn should_ignore_ordinary_files() {
            assert_eq!(parse_header("//! A module.\n"), None);
        }
    }

    mod pr_body {
        use super::*;

        #[test]
        fn should_flag_vendored_files_whose_upstream_changed() {
            let vendored = vec![(
                "crates/core-gpui/src/menu.rs".into(),
                "0.9.1".into(),
                "src/menu.rs".into(),
            )];
            let body = pr_body("0.9.1", "0.9.2", &[], &vendored, &["src/menu.rs".into()]);
            assert!(body.contains("⚠️ upstream changed"));
            assert!(body.contains("captures compare"));
        }
    }
}
