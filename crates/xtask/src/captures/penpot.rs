//! The Penpot column: each variant's main component, exported by the Penpot exporter at 2x
//! as the automation profile.
//!
//! Credentials come from the process env (`PENPOT_URL`, `PENPOT_AUTOMATION_EMAIL`,
//! `PENPOT_AUTOMATION_PASSWORD`; names only in `.env.example`). The exporter takes only a
//! session cookie, so this logs in with `login-with-password`. It has no theme parameter
//! either: it exports whatever `Mode` the library file stores, and `--penpot-theme` labels
//! which one that is. The library has no dark boards, so a dark run means toggling the
//! library's `Mode` over MCP first, and back afterwards.

use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use super::Capture;

/// The env the Penpot column needs.
const ENV: [&str; 3] = [
    "PENPOT_URL",
    "PENPOT_AUTOMATION_EMAIL",
    "PENPOT_AUTOMATION_PASSWORD",
];

/// The vendored library snapshot: component ids and their variant properties.
const SNAPSHOT_FILE: &str = "design/library.snapshot.json";

/// The exporter's scale, matching the GPUI captures.
const SCALE: f32 = 2.;

/// Exports the main component of every `theme` capture into `out`, under the same file name.
pub fn export(root: &Path, captures: &[Capture], theme: &str, out: &Path) -> Result<()> {
    let [url, email, password] = env()?;
    let snapshot: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(SNAPSHOT_FILE))
            .context("reading the library snapshot")?,
    )?;
    let file_id = snapshot["libraryFileId"]
        .as_str()
        .context("library snapshot has no libraryFileId")?;

    let session = Session::login(&url, &email, &password)?;
    let file = session.rpc("get-file", json!({ "id": file_id }))?;
    for capture in captures.iter().filter(|c| c.theme == theme) {
        let component_id = component_id(&snapshot, capture)?;
        let main = &file["data"]["components"][component_id];
        let (Some(page), Some(object)) = (
            main["mainInstancePage"].as_str(),
            main["mainInstanceId"].as_str(),
        ) else {
            bail!(
                "get-file has no main instance for {component_id} ({})",
                capture.file
            );
        };
        let png = session.export(file_id, page, object)?;
        std::fs::write(out.join(&capture.file), png)?;
    }
    Ok(())
}

/// The three env values, or which are missing.
fn env() -> Result<[String; 3]> {
    let values = ENV.map(|name| std::env::var(name).ok().filter(|v| !v.is_empty()));
    let missing: Vec<&str> = ENV
        .iter()
        .zip(&values)
        .filter(|(_, value)| value.is_none())
        .map(|(name, _)| *name)
        .collect();
    if !missing.is_empty() {
        bail!(
            "set {} in the environment (the Penpot automation profile; names in .env.example) \
             and make sure Penpot is running",
            missing.join(", ")
        );
    }
    Ok(values.map(|value| value.unwrap_or_default()))
}

/// The snapshot's id for the library component with this capture's name and variant props.
fn component_id<'a>(snapshot: &'a Value, capture: &Capture) -> Result<&'a str> {
    let components = snapshot["components"]
        .as_object()
        .context("library snapshot has no components")?;
    components
        .iter()
        .find(|(_, component)| {
            let properties = &component["variantProperties"];
            component["name"] == capture.component.as_str()
                && properties.as_object().map(|p| p.len()) == Some(capture.props.len())
                && capture
                    .props
                    .iter()
                    .all(|(property, value)| properties[property] == value.as_str())
        })
        .map(|(id, _)| id.as_str())
        .with_context(|| {
            format!(
                "no library component {} with {:?} in the snapshot",
                capture.component, capture.props
            )
        })
}

/// A logged-in automation-profile session.
struct Session {
    url: String,
    cookie: String,
    profile_id: String,
}

impl Session {
    fn login(url: &str, email: &str, password: &str) -> Result<Session> {
        let url = url.trim_end_matches('/').to_owned();
        let mut response = ureq::post(format!("{url}/api/rpc/command/login-with-password"))
            .header("Accept", "application/json")
            .send_json(json!({ "email": email, "password": password }))
            .with_context(|| format!("logging in to {url} as the automation profile"))?;
        let cookie: Vec<String> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|value| value.to_str().ok())
            .filter_map(|value| value.split(';').next())
            .map(str::to_owned)
            .collect();
        let profile: Value = response.body_mut().read_json()?;
        let profile_id = profile["id"]
            .as_str()
            .context("login returned no profile id")?
            .to_owned();
        Ok(Session {
            url,
            cookie: cookie.join("; "),
            profile_id,
        })
    }

    fn rpc(&self, command: &str, body: Value) -> Result<Value> {
        ureq::post(format!("{}/api/rpc/command/{command}", self.url))
            .header("Accept", "application/json")
            .header("Cookie", &self.cookie)
            .send_json(body)
            .with_context(|| format!("Penpot {command}"))?
            .body_mut()
            .with_config()
            .limit(256 * 1024 * 1024)
            .read_json()
            .with_context(|| format!("reading Penpot {command}"))
    }

    /// One shape as a PNG through the exporter.
    fn export(&self, file_id: &str, page_id: &str, object_id: &str) -> Result<Vec<u8>> {
        let body = export_body(&self.profile_id, file_id, page_id, object_id);
        let result = ureq::post(format!("{}/api/export", self.url))
            .header("Content-Type", "application/transit+json")
            .header("Accept", "application/transit+json")
            .header("Cookie", &self.cookie)
            .send(body.to_string())
            .context("Penpot export-shapes")?
            .body_mut()
            .read_to_string()?;
        let uri = asset_uri(&result)
            .with_context(|| format!("no asset uri in the export result: {result:.200}"))?;
        // The asset URL is short-lived: fetch it straight away.
        Ok(ureq::get(&uri)
            .header("Cookie", &self.cookie)
            .call()
            .with_context(|| format!("downloading {uri}"))?
            .body_mut()
            .read_to_vec()?)
    }
}

/// The exporter's `export-shapes` request, in transit+json.
fn export_body(profile_id: &str, file_id: &str, page_id: &str, object_id: &str) -> Value {
    json!({
        "~:wait": true,
        "~:cmd": "~:export-shapes",
        "~:profile-id": format!("~u{profile_id}"),
        "~:exports": [{
            "~:type": "~:png",
            "~:suffix": "",
            "~:scale": SCALE,
            "~:page-id": format!("~u{page_id}"),
            "~:file-id": format!("~u{file_id}"),
            "~:name": "capture",
            "~:object-id": format!("~u{object_id}"),
        }],
    })
}

/// The asset URI in an export result (transit tags a URI string with `~r`), else its first
/// http(s) URL.
fn asset_uri(result: &str) -> Option<String> {
    let Some(key) = result.find("\"~:uri\"") else {
        let start = result.find("http")?;
        let end = start + result[start..].find('"')?;
        return Some(result[start..end].to_owned());
    };
    let after = &result[key + "\"~:uri\"".len()..];
    let start = after.find('"')? + 1;
    let end = start + after[start..].find('"')?;
    let uri = &after[start..end];
    Some(uri.strip_prefix("~r").unwrap_or(uri).to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture(variant: &str, size: &str) -> Capture {
        Capture {
            component: "Button".into(),
            props: vec![
                ("variant".into(), variant.into()),
                ("size".into(), size.into()),
            ],
            theme: "light".into(),
            file: "b.png".into(),
            story: None,
            background: "#ffffff".into(),
        }
    }

    mod component_id {
        use super::*;

        fn snapshot() -> Value {
            json!({ "components": {
                "id-a": { "name": "Button", "variantProperties": { "size": "sm", "variant": "default" } },
                "id-b": { "name": "Button", "variantProperties": { "size": "sm", "variant": "outline" } },
                "id-c": { "name": "Badge", "variantProperties": { "variant": "outline" } },
            }})
        }

        #[test]
        fn should_match_name_and_every_variant_property() {
            assert_eq!(
                component_id(&snapshot(), &capture("outline", "sm")).unwrap(),
                "id-b"
            );
        }

        #[test]
        fn should_fail_when_no_main_component_matches() {
            assert!(component_id(&snapshot(), &capture("ghost", "sm")).is_err());
        }
    }

    mod asset_uri {
        use super::*;

        #[test]
        fn should_read_a_transit_tagged_uri() {
            let result =
                r#"["^ ","~:id","~uabc","~:uri","~rhttp://penpot.localhost/assets/x.png"]"#;
            assert_eq!(
                asset_uri(result).unwrap(),
                "http://penpot.localhost/assets/x.png"
            );
        }
    }

    mod export_body {
        use super::*;

        #[test]
        fn should_tag_ids_as_transit_uuids_at_2x() {
            let body = export_body("p", "f", "pg", "o");
            assert_eq!(body["~:profile-id"], "~up");
            assert_eq!(body["~:exports"][0]["~:object-id"], "~uo");
            assert_eq!(body["~:exports"][0]["~:scale"], 2.0);
        }
    }
}
