//! The code manifest: the registry and the icons in core-react's vocabulary, for the drift
//! check (`cargo xtask drift` runs `design-drift check` from `@brrritscold/design-core-node`
//! on it).
//!
//! It matches `codeManifestSchema` in `@brrritscold/design-core-js/code-manifest`:
//!
//! - `exports`: one per registry [`Entry`], named as the Penpot component (= the export). Each
//!   variant property is a literal-union prop whose values are the ones the registry renders
//!   (kebab-case from strum, so `true` / `false` come out as literals, which the check accepts
//!   for a boolean variant value too). Every prop is optional: a builder setter can always be
//!   left out, and leaving it out is what the variant value `default` does.
//! - `icons`: every [`IconName`] as lucide-react exports it, `<Variant>Icon`.
//!
//! Keys and values are sorted in code-unit order, as core-react's manifest is.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Context as _;
use brrritscold_core_gpui::IconName;
use serde_json::{Value, json};

use crate::registry::{Entry, registry};

/// The code manifest for these entries and icons.
pub fn manifest(entries: &[Entry], icons: &[IconName]) -> Value {
    let exports: BTreeMap<&str, Value> = entries
        .iter()
        .map(|entry| {
            let mut values: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
            for variant in entry.variants() {
                for (property, value) in variant.props {
                    values.entry(property).or_default().insert(value);
                }
            }
            let props: BTreeMap<&str, Value> = values
                .into_iter()
                .map(|(property, values)| {
                    let prop = json!({ "type": "literals", "values": values, "optional": true });
                    (property, prop)
                })
                .collect();
            (entry.name, json!({ "props": props }))
        })
        .collect();
    let icons: BTreeSet<String> = icons
        .iter()
        .map(|&icon| format!("{}Icon", <&str>::from(icon)))
        .collect();
    json!({ "exports": exports, "icons": icons })
}

/// Writes the manifest of the whole registry and every icon to `out`.
pub fn write(out: &Path) -> anyhow::Result<()> {
    let manifest = manifest(&registry(), IconName::ALL);
    let text = serde_json::to_string_pretty(&manifest)? + "\n";
    std::fs::write(out, text).with_context(|| format!("writing {}", out.display()))
}

#[cfg(test)]
mod tests {
    use gpui::IntoElement as _;

    use super::*;
    use crate::registry::Variant;

    /// A two-property entry whose variants list some values twice and out of order.
    fn toggle() -> Entry {
        Entry {
            name: "Toggle",
            properties: &["variant", "pressed"],
            variants: || {
                [
                    ("outline", "true"),
                    ("default", "false"),
                    ("outline", "false"),
                ]
                .into_iter()
                .map(|(variant, pressed)| Variant {
                    props: vec![("variant", variant), ("pressed", pressed)],
                    story: None,
                    render: Box::new(|_| gpui::Empty.into_any_element()),
                })
                .collect()
            },
        }
    }

    mod manifest {
        use super::*;

        #[test]
        fn should_list_each_entry_as_an_export_with_one_optional_literal_prop_per_property() {
            assert_eq!(
                manifest(&[toggle()], &[])["exports"],
                json!({
                    "Toggle": {
                        "props": {
                            "pressed": { "type": "literals", "values": ["false", "true"], "optional": true },
                            "variant": { "type": "literals", "values": ["default", "outline"], "optional": true },
                        }
                    }
                })
            );
        }

        #[test]
        fn should_sort_exports_and_props_in_code_unit_order() {
            let mut entries = vec![toggle()];
            entries.extend(registry());
            let manifest = manifest(&entries, &[]);
            let keys = |value: &Value| -> Vec<String> {
                value.as_object().unwrap().keys().cloned().collect()
            };
            let exports = keys(&manifest["exports"]);
            let mut sorted = exports.clone();
            sorted.sort();
            assert_eq!(exports, sorted);
            assert!(exports.contains(&"Toggle".to_owned()));
            assert_eq!(
                keys(&manifest["exports"]["Toggle"]["props"]),
                ["pressed", "variant"]
            );
        }

        #[test]
        fn should_name_icons_as_lucide_react_exports_sorted() {
            assert_eq!(
                manifest(&[], &[IconName::Plus, IconName::ArrowDown01])["icons"],
                json!(["ArrowDown01Icon", "PlusIcon"])
            );
        }

        #[test]
        fn should_list_every_button_variant_and_size_the_builder_takes() {
            use brrritscold_core_gpui::{ButtonSize, ButtonVariant};
            use strum::IntoEnumIterator as _;

            let props = &manifest(&registry(), &[])["exports"]["Button"]["props"];
            let variants: BTreeSet<&str> = ButtonVariant::iter().map(Into::into).collect();
            let sizes: BTreeSet<&str> = ButtonSize::iter().map(Into::into).collect();
            assert_eq!(props["variant"]["values"], json!(variants));
            assert_eq!(props["size"]["values"], json!(sizes));
        }

        #[test]
        fn should_list_a_part_as_an_export_of_its_own() {
            let exports = &manifest(&registry(), &[])["exports"];
            assert_eq!(exports["CardHeader"], json!({ "props": {} }));
            assert_eq!(
                exports["PaginationLink"]["props"]["isActive"]["values"],
                json!(["false", "true"])
            );
        }
    }
}
