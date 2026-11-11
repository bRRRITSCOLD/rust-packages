//! `cargo xtask captures …`: local, on-demand screenshots for a person to judge. Never run in
//! CI, never committed (they land under `target/`), and nothing compares their pixels.
//!
//! - `captures [--out <dir>]` runs the gallery's headless `capture`: one 2x PNG per registry
//!   variant × theme plus `index.json`, into `target/captures/` by default.
//! - `captures compare [--penpot-theme light|dark]` captures again, then builds
//!   `target/captures/compare/index.html`: one row per component × variant × theme, columns
//!   core-react (the published Storybook, the reference) | GPUI | Penpot. A column whose source
//!   can't be reached says why in the sheet instead of failing the run.

mod penpot;
mod storybook;

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::tokens::workspace_root;

/// Where captures land unless `--out` says otherwise, relative to the workspace root.
const DEFAULT_OUT: &str = "target/captures";

/// `cargo xtask captures <[--out <dir>] | compare [--penpot-theme <theme>]>`.
pub fn run(args: impl Iterator<Item = String>) -> Result<()> {
    let root = workspace_root();
    match parse(&args.collect::<Vec<_>>())? {
        Task::Capture { out } => {
            let out = out.unwrap_or_else(|| root.join(DEFAULT_OUT));
            capture(&root, &out)?;
            println!("captures written to {}", out.display());
            Ok(())
        }
        Task::Compare { penpot_theme } => compare(&root, &penpot_theme),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Task {
    Capture { out: Option<PathBuf> },
    Compare { penpot_theme: String },
}

const USAGE: &str =
    "usage: cargo xtask captures [--out <dir>] | captures compare [--penpot-theme light|dark]";

fn parse(args: &[String]) -> Result<Task> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        [] => Ok(Task::Capture { out: None }),
        ["--out", out] => Ok(Task::Capture {
            out: Some(PathBuf::from(out)),
        }),
        ["compare"] => Ok(Task::Compare {
            penpot_theme: "light".into(),
        }),
        ["compare", "--penpot-theme", theme @ ("light" | "dark")] => Ok(Task::Compare {
            penpot_theme: (*theme).into(),
        }),
        _ => bail!(USAGE),
    }
}

/// Runs the gallery's headless capture into `out`.
fn capture(root: &Path, out: &Path) -> Result<()> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .current_dir(root)
        .args([
            "run",
            "--quiet",
            "--locked",
            "--package",
            "brrritscold-core-gpui-gallery",
            "--",
            "capture",
            "--out",
        ])
        .arg(out)
        .status()
        .context("running the gallery's capture")?;
    if !status.success() {
        bail!("the gallery's capture failed ({status}); it needs a Vulkan driver, e.g. lavapipe");
    }
    Ok(())
}

/// One GPUI capture, as the gallery's `index.json` lists it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Capture {
    pub component: String,
    /// `(Penpot property, value)` in the registry's order.
    pub props: Vec<(String, String)>,
    pub theme: String,
    pub file: String,
    pub story: Option<String>,
    /// The theme's `background` token, `#rrggbb`.
    pub background: String,
}

fn read_index(path: &Path) -> Result<Vec<Capture>> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    parse_index(&text)
}

fn parse_index(text: &str) -> Result<Vec<Capture>> {
    let index: Value = serde_json::from_str(text)?;
    let string = |c: &Value, key: &str| -> Result<String> {
        c[key]
            .as_str()
            .map(str::to_owned)
            .with_context(|| format!("capture index entry has no `{key}`"))
    };
    index["captures"]
        .as_array()
        .context("capture index has no `captures`")?
        .iter()
        .map(|c| {
            let props = c["props"]
                .as_object()
                .context("capture index entry has no `props`")?
                .iter()
                .map(|(property, value)| {
                    Ok((
                        property.clone(),
                        value.as_str().context("a prop value")?.to_owned(),
                    ))
                })
                .collect::<Result<_>>()?;
            Ok(Capture {
                component: string(c, "component")?,
                props,
                theme: string(c, "theme")?,
                file: string(c, "file")?,
                story: c["story"].as_str().map(str::to_owned),
                background: string(c, "background")?,
            })
        })
        .collect()
}

fn compare(root: &Path, penpot_theme: &str) -> Result<()> {
    let captures_dir = root.join(DEFAULT_OUT);
    capture(root, &captures_dir)?;
    let captures = read_index(&captures_dir.join("index.json"))?;

    let dir = captures_dir.join("compare");
    if dir.exists() {
        std::fs::remove_dir_all(&dir).with_context(|| format!("clearing {}", dir.display()))?;
    }
    let storybook_dir = dir.join("storybook");
    let penpot_dir = dir.join("penpot");
    std::fs::create_dir_all(&storybook_dir)?;
    std::fs::create_dir_all(&penpot_dir)?;

    println!("core-react column: capturing the published Storybook …");
    let storybook =
        storybook::capture(root, &captures, &storybook_dir).map_err(|e| format!("{e:#}"));
    println!("Penpot column: exporting the library ({penpot_theme}) …");
    let penpot =
        penpot::export(root, &captures, penpot_theme, &penpot_dir).map_err(|e| format!("{e:#}"));
    for (column, status) in [("core-react", &storybook), ("Penpot", &penpot)] {
        if let Err(reason) = status {
            eprintln!("xtask: {column} column unavailable: {reason}");
        }
    }

    let sheet = Sheet {
        captures: &captures,
        storybook: storybook.as_ref().err().map(String::as_str),
        penpot: penpot.as_ref().err().map(String::as_str),
        penpot_theme,
    };
    let html = sheet.html(|column, file| dir.join(column).join(file).exists());
    let index = dir.join("index.html");
    std::fs::write(&index, html).with_context(|| format!("writing {}", index.display()))?;
    println!("comparison sheet: {}", index.display());
    Ok(())
}

/// The comparison page. A column is `Some(reason)` when its source couldn't be reached.
struct Sheet<'a> {
    captures: &'a [Capture],
    storybook: Option<&'a str>,
    penpot: Option<&'a str>,
    penpot_theme: &'a str,
}

/// One cell of the sheet: an image (relative to `index.html`) or why there is none.
#[derive(Debug, PartialEq, Eq)]
enum Cell {
    Image(String),
    Note(String),
}

impl Sheet<'_> {
    fn cells(&self, capture: &Capture, exists: &impl Fn(&str, &str) -> bool) -> [Cell; 3] {
        let image = |column: &str| {
            if exists(column, &capture.file) {
                Cell::Image(format!("{column}/{}", capture.file))
            } else {
                Cell::Note("not captured; see the run's output".into())
            }
        };
        let storybook = match (self.storybook, &capture.story) {
            (Some(_), _) => Cell::Note("column unavailable (see above)".into()),
            (None, None) => Cell::Note("no Storybook story shows this variant".into()),
            (None, Some(_)) => image("storybook"),
        };
        let penpot = if self.penpot.is_some() {
            Cell::Note("column unavailable (see above)".into())
        } else if capture.theme != self.penpot_theme {
            Cell::Note(format!(
                "this run exported Penpot's stored theme as {}; for {}, toggle the library's \
                 Mode and run with --penpot-theme {}",
                self.penpot_theme, capture.theme, capture.theme
            ))
        } else {
            image("penpot")
        };
        [
            storybook,
            Cell::Image(format!("../{}", capture.file)),
            penpot,
        ]
    }

    fn html(&self, exists: impl Fn(&str, &str) -> bool) -> String {
        let mut banners = String::new();
        for (column, reason) in [("core-react", self.storybook), ("Penpot", self.penpot)] {
            if let Some(reason) = reason {
                banners.push_str(&format!(
                    "<p class=\"banner\"><b>{column} column unavailable.</b> {}</p>\n",
                    escape(reason)
                ));
            }
        }
        let mut rows = String::new();
        for capture in self.captures {
            let props: Vec<String> = capture
                .props
                .iter()
                .map(|(property, value)| format!("{property}={value}"))
                .collect();
            rows.push_str(&format!(
                "<tr><th>{}<br><code>{}</code><br>{}</th>",
                escape(&capture.component),
                escape(&props.join(" ")),
                escape(&capture.theme)
            ));
            for cell in self.cells(capture, &exists) {
                match cell {
                    Cell::Image(src) => rows.push_str(&format!(
                        "<td style=\"background:{}\"><img src=\"{}\" alt=\"\"></td>",
                        escape(&capture.background),
                        escape(&src)
                    )),
                    Cell::Note(note) => {
                        rows.push_str(&format!("<td class=\"note\">{}</td>", escape(&note)))
                    }
                }
            }
            rows.push_str("</tr>\n");
        }
        format!(
            "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
             <title>core-gpui design comparison</title>\n<style>\n{STYLE}</style></head><body>\n\
             <h1>core-gpui design comparison</h1>\n\
             <p>Every image is 2x. core-react is the reference: fix GPUI where it differs from \
             core-react; where GPUI matches core-react but not Penpot, note it against ts-packages.</p>\n\
             {banners}<table>\n<thead><tr><th></th><th>core-react (Storybook)</th><th>GPUI</th>\
             <th>Penpot</th></tr></thead>\n<tbody>\n{rows}</tbody></table></body></html>\n"
        )
    }
}

const STYLE: &str =
    "body { font: 14px system-ui, sans-serif; margin: 24px; background: #888; color: #111; }
table { border-collapse: separate; border-spacing: 8px; }
th { text-align: left; font-weight: 500; vertical-align: middle; }
td { padding: 8px; vertical-align: middle; text-align: center; min-width: 120px; }
td.note { background: #ddd; font-size: 12px; max-width: 220px; }
.banner { background: #ffe9a8; padding: 8px 12px; max-width: 900px; }
";

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture(theme: &str, story: Option<&str>) -> Capture {
        Capture {
            component: "Button".into(),
            props: vec![
                ("variant".into(), "outline".into()),
                ("size".into(), "sm".into()),
            ],
            theme: theme.into(),
            file: format!("button--variant-outline--size-sm--{theme}.png"),
            story: story.map(str::to_owned),
            background: "#ffffff".into(),
        }
    }

    mod parse {
        use super::*;

        fn args(args: &[&str]) -> Vec<String> {
            args.iter().map(|a| (*a).to_owned()).collect()
        }

        #[test]
        fn should_default_to_capturing_into_target() {
            assert_eq!(parse(&[]).unwrap(), Task::Capture { out: None });
        }

        #[test]
        fn should_take_an_out_dir() {
            assert_eq!(
                parse(&args(&["--out", "/tmp/x"])).unwrap(),
                Task::Capture {
                    out: Some("/tmp/x".into())
                }
            );
        }

        #[test]
        fn should_compare_penpot_light_by_default() {
            assert_eq!(
                parse(&args(&["compare"])).unwrap(),
                Task::Compare {
                    penpot_theme: "light".into()
                }
            );
        }

        #[test]
        fn should_reject_an_unknown_penpot_theme() {
            assert!(parse(&args(&["compare", "--penpot-theme", "sepia"])).is_err());
        }
    }

    mod index {
        use super::*;

        #[test]
        fn should_read_props_in_registry_order() {
            let text = r##"{"captures":[{"component":"Button","props":{"variant":"outline","size":"sm"},
                "theme":"light","file":"button--variant-outline--size-sm--light.png",
                "story":"components-button--default","background":"#ffffff"}]}"##;
            let captures = parse_index(text).unwrap();
            assert_eq!(
                captures,
                vec![capture("light", Some("components-button--default"))]
            );
        }
    }

    mod sheet {
        use super::*;

        fn sheet<'a>(
            captures: &'a [Capture],
            storybook: Option<&'a str>,
            penpot: Option<&'a str>,
        ) -> Sheet<'a> {
            Sheet {
                captures,
                storybook,
                penpot,
                penpot_theme: "light",
            }
        }

        #[test]
        fn should_put_core_react_gpui_and_penpot_side_by_side() {
            let captures = [capture("light", Some("components-button--default"))];
            let cells = sheet(&captures, None, None).cells(&captures[0], &|_, _| true);
            let file = &captures[0].file;
            assert_eq!(
                cells,
                [
                    Cell::Image(format!("storybook/{file}")),
                    Cell::Image(format!("../{file}")),
                    Cell::Image(format!("penpot/{file}")),
                ]
            );
        }

        #[test]
        fn should_explain_a_dark_row_on_a_light_penpot_run() {
            let captures = [capture("dark", Some("components-button--default"))];
            let [_, _, penpot] = sheet(&captures, None, None).cells(&captures[0], &|_, _| true);
            assert!(matches!(penpot, Cell::Note(note) if note.contains("--penpot-theme dark")));
        }

        #[test]
        fn should_show_why_an_unreachable_column_is_empty() {
            let captures = [capture("light", Some("components-button--default"))];
            let html = sheet(&captures, None, Some("set PENPOT_URL")).html(|_, _| true);
            assert!(html.contains("Penpot column unavailable.</b> set PENPOT_URL"));
            assert!(!html.contains("penpot/button"));
            assert!(html.contains("storybook/button"));
        }
    }
}
