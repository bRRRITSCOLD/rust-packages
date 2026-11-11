//! `cargo xtask tokens …`: vendor core-react's tokens and library snapshot from npm at one
//! exact version (the *core-react pin*) and generate `core-gpui-tokens/src/generated.rs`. The
//! Lucide icons ride the same pin: core-react names the exact `lucide-react` version.

pub mod generate;
pub mod icons;

use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use toml_edit::{DocumentMut, Item, Table, value};

const PACKAGE: &str = "@brrritscold/core-react";
/// The tokens crate's manifest, which holds the core-react pin and the drift checker pin.
pub(crate) const TOKENS_CARGO_TOML: &str = "crates/core-gpui-tokens/Cargo.toml";
pub(crate) const TOKENS_FILE: &str = "crates/core-gpui-tokens/tokens.json";
const GENERATED_FILE: &str = "crates/core-gpui-tokens/src/generated.rs";
/// The library snapshot rides the same pin; the drift check reads it.
pub(crate) const SNAPSHOT_FILE: &str = "design/library.snapshot.json";

/// The core-react pin recorded under `[package.metadata.core-react]` in the tokens crate.
#[derive(Debug, PartialEq, Eq)]
pub struct Pin {
    /// Exact npm version of `@brrritscold/core-react`.
    pub version: String,
    /// sha256 of the vendored `tokens.json`.
    pub tokens_sha256: String,
    /// sha256 of the vendored `library.snapshot.json`.
    pub snapshot_sha256: String,
    /// The exact `lucide-react` version core-react depends on; icons come from `lucide-static`.
    pub lucide: String,
}

/// `cargo xtask tokens <update <version|latest> | generate>`.
pub fn run(mut args: impl Iterator<Item = String>) -> Result<()> {
    match args.next().as_deref() {
        Some("update") => {
            let spec = args
                .next()
                .context("usage: cargo xtask tokens update <version|latest>")?;
            update(&spec)
        }
        Some("generate") => regenerate(),
        _ => bail!("usage: cargo xtask tokens <update <version|latest> | generate>"),
    }
}

/// Downloads core-react at `spec`, vendors its tokens and snapshot, records the pin and
/// regenerates `generated.rs`.
fn update(spec: &str) -> Result<()> {
    let root = workspace_root();
    let (version, bytes) = npm_tarball(PACKAGE, spec)?;
    let version = version.as_str();
    let tokens = extract(&bytes, "package/dist/tokens.json")?;
    let snapshot = extract(&bytes, "package/dist/library.snapshot.json")?;
    let package: serde_json::Value =
        serde_json::from_str(&extract(&bytes, "package/package.json")?)?;
    let lucide = exact_dependency(&package, "dependencies", "lucide-react")?;
    // The drift checker core-react itself ran at this release: the pair moves with the pin.
    let checker_node =
        exact_dependency(&package, "devDependencies", "@brrritscold/design-core-node")?;
    let checker_js = exact_dependency(&package, "devDependencies", "@brrritscold/design-core-js")?;

    write(&root.join(TOKENS_FILE), &tokens)?;
    write(&root.join(SNAPSHOT_FILE), &snapshot)?;
    write_pin(
        &root,
        &Pin {
            version: version.to_owned(),
            tokens_sha256: sha256(tokens.as_bytes()),
            snapshot_sha256: sha256(snapshot.as_bytes()),
            lucide: lucide.clone(),
        },
    )?;
    write_checker_pin(&root, &checker_node, &checker_js)?;
    icons::vendor(&root, &lucide)?;
    let generated = generate::generate(&tokens, version)?;
    write(&root.join(GENERATED_FILE), &generated)?;
    println!(
        "core-react pin: {version} (tokens, snapshot, lucide-static {lucide}, \
         design-core-node {checker_node} + design-core-js {checker_js})"
    );
    Ok(())
}

/// Regenerates both generated modules from the vendored files, for a generator change.
fn regenerate() -> Result<()> {
    let root = workspace_root();
    let pin = read_pin(&root)?;
    let tokens = std::fs::read_to_string(root.join(TOKENS_FILE))?;
    write(
        &root.join(GENERATED_FILE),
        &generate::generate(&tokens, &pin.version)?,
    )?;
    icons::regenerate(&root, &pin.lucide)
}

/// Fetches an npm package's manifest at `spec` and downloads its tarball.
pub(crate) fn npm_tarball(package: &str, spec: &str) -> Result<(String, Vec<u8>)> {
    let manifest: serde_json::Value = ureq::get(format!(
        "https://registry.npmjs.org/{}/{spec}",
        package.replace('/', "%2f")
    ))
    .call()
    .with_context(|| format!("fetching {package}@{spec} from npm"))?
    .body_mut()
    .read_json()?;
    let version = manifest["version"]
        .as_str()
        .context("npm manifest has no version")?
        .to_owned();
    let tarball = manifest["dist"]["tarball"]
        .as_str()
        .context("npm manifest has no tarball")?;
    let bytes = ureq::get(tarball)
        .call()
        .with_context(|| format!("downloading {tarball}"))?
        .body_mut()
        .with_config()
        .limit(128 * 1024 * 1024)
        .read_to_vec()?;
    Ok((version, bytes))
}

/// A dependency's version from a package.json section, which must be exact (no range).
fn exact_dependency(package: &serde_json::Value, section: &str, name: &str) -> Result<String> {
    let version = package[section][name]
        .as_str()
        .with_context(|| format!("core-react's {section} has no `{name}`"))?;
    if !version.starts_with(|c: char| c.is_ascii_digit()) {
        bail!("core-react's `{name}` is `{version}`, not an exact version");
    }
    Ok(version.to_owned())
}

/// Every file in a tarball whose path passes `wanted`, keyed by path.
pub(crate) fn extract_where(
    tarball: &[u8],
    wanted: impl Fn(&str) -> bool,
) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tarball));
    let mut files = BTreeMap::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().into_owned();
        if wanted(&path) {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            files.insert(path, bytes);
        }
    }
    Ok(files)
}

fn extract(tarball: &[u8], wanted: &str) -> Result<String> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tarball));
    for entry in archive.entries()? {
        let mut entry = entry?;
        if entry.path()?.as_ref() == Path::new(wanted) {
            let mut text = String::new();
            entry.read_to_string(&mut text)?;
            return Ok(text);
        }
    }
    bail!("the tarball has no `{wanted}`")
}

pub(crate) fn write(path: &Path, contents: &str) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, contents).with_context(|| format!("writing {}", path.display()))
}

fn manifest_path(root: &Path) -> PathBuf {
    root.join(TOKENS_CARGO_TOML)
}

/// Reads the core-react pin from the tokens crate's manifest.
pub fn read_pin(root: &Path) -> Result<Pin> {
    parse_pin(&std::fs::read_to_string(manifest_path(root))?)
}

/// The core-react pin in the text of the tokens crate's manifest.
pub(crate) fn parse_pin(cargo_toml: &str) -> Result<Pin> {
    let doc: DocumentMut = cargo_toml.parse()?;
    let table = doc["package"]["metadata"]["core-react"]
        .as_table()
        .context("the tokens crate has no [package.metadata.core-react]")?;
    let field = |key: &str| {
        table
            .get(key)
            .and_then(Item::as_str)
            .map(str::to_owned)
            .with_context(|| format!("[package.metadata.core-react] has no `{key}`"))
    };
    Ok(Pin {
        version: field("version")?,
        tokens_sha256: field("tokens-sha256")?,
        snapshot_sha256: field("snapshot-sha256")?,
        lucide: field("lucide")?,
    })
}

fn write_pin(root: &Path, pin: &Pin) -> Result<()> {
    let path = manifest_path(root);
    let mut doc: DocumentMut = std::fs::read_to_string(&path)?.parse()?;
    let package = doc["package"].as_table_mut().context("no [package]")?;
    let metadata = package
        .entry("metadata")
        .or_insert_with(|| {
            let mut table = Table::new();
            table.set_implicit(true);
            Item::Table(table)
        })
        .as_table_mut()
        .context("[package.metadata] is not a table")?;
    let mut table = Table::new();
    table.decor_mut().set_prefix(
        "\n# The core-react pin: tokens.json and library.snapshot.json are vendored from this exact\n\
         # npm version by `cargo xtask tokens update`. Don't edit by hand.\n",
    );
    table.insert("version", value(&pin.version));
    table.insert("tokens-sha256", value(&pin.tokens_sha256));
    table.insert("snapshot-sha256", value(&pin.snapshot_sha256));
    table.insert("lucide", value(&pin.lucide));
    metadata.insert("core-react", Item::Table(table));
    write(&path, &doc.to_string())
}

/// Moves the drift checker pin, `[package.metadata.design-core-node]`, editing the two values
/// in place so the table's comments survive.
fn write_checker_pin(root: &Path, node: &str, js: &str) -> Result<()> {
    let path = manifest_path(root);
    let mut doc: DocumentMut = std::fs::read_to_string(&path)?.parse()?;
    set_checker_pin(&mut doc, node, js)?;
    write(&path, &doc.to_string())
}

fn set_checker_pin(doc: &mut DocumentMut, node: &str, js: &str) -> Result<()> {
    let table = doc["package"]["metadata"]["design-core-node"]
        .as_table_like_mut()
        .context("the tokens crate has no [package.metadata.design-core-node]")?;
    for (key, version) in [("version", node), ("design-core-js", js)] {
        match table.get_mut(key).and_then(Item::as_value_mut) {
            Some(existing) => *existing = version.into(),
            None => {
                table.insert(key, value(version));
            }
        }
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The workspace root, two levels above this crate.
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask lives at crates/xtask")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Staleness checks on the checked-in files: a hand edit or a forgotten regeneration fails here.
    mod sourcegen {
        use super::*;

        #[test]
        fn should_match_the_checked_in_generated_rs() {
            let root = workspace_root();
            let pin = read_pin(&root).unwrap();
            let tokens = std::fs::read_to_string(root.join(TOKENS_FILE)).unwrap();
            let checked_in = std::fs::read_to_string(root.join(GENERATED_FILE)).unwrap();
            let fresh = generate::generate(&tokens, &pin.version).unwrap();
            assert!(
                fresh == checked_in,
                "{GENERATED_FILE} is stale or hand-edited: run `cargo xtask tokens generate`"
            );
        }

        #[test]
        fn should_match_the_pinned_sha256_of_the_vendored_files() {
            let root = workspace_root();
            let pin = read_pin(&root).unwrap();
            let tokens = std::fs::read(root.join(TOKENS_FILE)).unwrap();
            let snapshot = std::fs::read(root.join(SNAPSHOT_FILE)).unwrap();
            assert_eq!(
                sha256(&tokens),
                pin.tokens_sha256,
                "{TOKENS_FILE} differs from the pin"
            );
            assert_eq!(
                sha256(&snapshot),
                pin.snapshot_sha256,
                "{SNAPSHOT_FILE} differs from the pin"
            );
        }
    }

    mod checker_pin {
        use super::*;

        const MANIFEST: &str = "[package]\nname = \"t\"\n\n[package.metadata.design-core-node]\n\
            # The checker pin.\nversion = \"0.1.0\"\ndesign-core-js = \"0.1.0\"\n";

        #[test]
        fn should_move_both_versions_and_keep_comments() {
            let mut doc: DocumentMut = MANIFEST.parse().unwrap();
            set_checker_pin(&mut doc, "0.2.0", "0.2.1").unwrap();
            let text = doc.to_string();
            assert!(text.contains("# The checker pin."));
            assert!(text.contains("version = \"0.2.0\""));
            assert!(text.contains("design-core-js = \"0.2.1\""));
        }

        #[test]
        fn should_read_the_pair_from_core_reacts_dev_dependencies() {
            let package = serde_json::json!({
                "dependencies": { "lucide-react": "1.68.0" },
                "devDependencies": { "@brrritscold/design-core-node": "0.1.0" }
            });
            assert_eq!(
                exact_dependency(&package, "devDependencies", "@brrritscold/design-core-node")
                    .unwrap(),
                "0.1.0"
            );
            assert!(
                exact_dependency(&package, "devDependencies", "@brrritscold/design-core-js")
                    .is_err()
            );
        }

        #[test]
        fn should_reject_a_range() {
            let package = serde_json::json!({ "devDependencies": { "x": "^0.1.0" } });
            assert!(exact_dependency(&package, "devDependencies", "x").is_err());
        }
    }

    mod tarball {
        use super::*;

        fn tarball(files: &[(&str, &str)]) -> Vec<u8> {
            let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
                Vec::new(),
                flate2::Compression::fast(),
            ));
            for (path, contents) in files {
                let mut header = tar::Header::new_gnu();
                header.set_size(contents.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                builder
                    .append_data(&mut header, path, contents.as_bytes())
                    .unwrap();
            }
            builder.into_inner().unwrap().finish().unwrap()
        }

        #[test]
        fn should_extract_one_file() {
            let bytes = tarball(&[
                ("package/dist/tokens.json", "{}"),
                ("package/README.md", "x"),
            ]);
            assert_eq!(extract(&bytes, "package/dist/tokens.json").unwrap(), "{}");
        }

        #[test]
        fn should_fail_when_the_file_is_missing() {
            let bytes = tarball(&[("package/README.md", "x")]);
            assert!(extract(&bytes, "package/dist/tokens.json").is_err());
        }
    }
}
