//! The core-react column: the **published** Storybook (latest main, not the exact core-react
//! pin; the daily bump keeps that gap small), shot by `storybook.cjs` under Playwright.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value, json};

use super::Capture;

/// core-react's published Storybook.
const STORYBOOK_URL: &str = "https://brrritscold.github.io/ts-packages/core-react";

/// Playwright, at core-react's devDependency version. Installed under `target/` on first use.
const PLAYWRIGHT: &str = "1.64.1";

/// Shoots every capture that names a story into `out`, under the same file name.
pub fn capture(root: &Path, captures: &[Capture], out: &Path) -> Result<()> {
    let node_modules = playwright(root)?;
    let jobs = jobs(captures);
    let jobs_file = out.join("jobs.json");
    std::fs::write(&jobs_file, serde_json::to_string_pretty(&jobs)?)?;
    let script = root.join("crates/xtask/src/captures/storybook.cjs");
    let status = Command::new("node")
        .env("NODE_PATH", &node_modules)
        .arg(script)
        .arg(STORYBOOK_URL)
        .arg(&jobs_file)
        .arg(out)
        .status()
        .context("running node (needed for the core-react column)")?;
    if !status.success() {
        bail!(
            "storybook.cjs failed ({status}). It needs network access to {STORYBOOK_URL} and \
             Playwright's Chromium: {}/.bin/playwright install chromium",
            node_modules.display()
        );
    }
    Ok(())
}

/// One job per capture with a story: the story id, its props as args, the theme and file.
fn jobs(captures: &[Capture]) -> Vec<Value> {
    captures
        .iter()
        .filter_map(|capture| {
            let story = capture.story.as_ref()?;
            let args: Map<String, Value> = capture
                .props
                .iter()
                .map(|(property, value)| (property.clone(), json!(value)))
                .collect();
            Some(json!({
                "story": story,
                "args": args,
                "theme": capture.theme,
                "file": capture.file,
            }))
        })
        .collect()
}

/// Installs Playwright under `target/xtask-node` once and returns its `node_modules`.
fn playwright(root: &Path) -> Result<std::path::PathBuf> {
    let prefix = root.join("target/xtask-node");
    let node_modules = prefix.join("node_modules");
    let installed = node_modules.join("playwright/package.json");
    let version = std::fs::read_to_string(&installed)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|package| package["version"].as_str().map(str::to_owned));
    if version.as_deref() == Some(PLAYWRIGHT) {
        return Ok(node_modules);
    }
    std::fs::create_dir_all(&prefix)?;
    let npm = Command::new("npm")
        .args(["install", "--no-audit", "--no-fund", "--silent", "--prefix"])
        .arg(&prefix)
        .arg(format!("playwright@{PLAYWRIGHT}"))
        .status()
        .context("running npm (needed for the core-react column)")?;
    if !npm.success() {
        bail!("npm install playwright@{PLAYWRIGHT} failed ({npm})");
    }
    let browsers = Command::new(node_modules.join(".bin/playwright"))
        .args(["install", "chromium"])
        .status()
        .context("installing Playwright's Chromium")?;
    if !browsers.success() {
        bail!("playwright install chromium failed ({browsers})");
    }
    Ok(node_modules)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod jobs {
        use super::*;

        #[test]
        fn should_pass_props_as_args_and_skip_captures_without_a_story() {
            let with_story = Capture {
                component: "Button".into(),
                props: vec![
                    ("variant".into(), "ghost".into()),
                    ("size".into(), "icon".into()),
                ],
                theme: "dark".into(),
                file: "b.png".into(),
                story: Some("components-button--icon".into()),
                background: "#0a0a0a".into(),
            };
            let without = Capture {
                story: None,
                ..with_story.clone()
            };
            assert_eq!(
                jobs(&[with_story, without]),
                vec![json!({
                    "story": "components-button--icon",
                    "args": { "variant": "ghost", "size": "icon" },
                    "theme": "dark",
                    "file": "b.png",
                })]
            );
        }
    }
}
