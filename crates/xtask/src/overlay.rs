//! Applies OpenAPI Overlay 1.0 documents (`spec/overlays/*.yaml`) to the pristine
//! upstream spec (`spec/openapi.yaml`), writing the patched spec used by the generator
//! to `target/codegen/openapi.yaml`.
//!
//! Semantics follow <https://spec.openapis.org/overlay/v1.0.0.html>:
//! - overlay files are applied in file-name order, actions sequentially, each on the
//!   result of the previous one;
//! - `update` on an object is merged recursively (primitives and nested arrays are
//!   replaced), `update` on an array appends the value as a new entry;
//! - `remove: true` removes the selected node from its parent (and ignores `update`).

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use serde_json::{Map, Value};
use serde_json_path::{JsonPath, PathElement};
use std::fs;
use std::path::{Path, PathBuf};

const SPEC_PATH: &str = "spec/openapi.yaml";
const OVERLAYS_DIR: &str = "spec/overlays";
const OUTPUT_PATH: &str = "target/codegen/openapi.yaml";

#[derive(Debug, Deserialize)]
struct Overlay {
    overlay: String,
    info: Info,
    actions: Vec<Action>,
}

#[derive(Debug, Deserialize)]
struct Info {
    title: String,
}

#[derive(Debug, Deserialize)]
struct Action {
    target: String,
    #[serde(default)]
    update: Option<Value>,
    #[serde(default)]
    remove: bool,
}

/// Owned copy of a normalized-path segment, so the document can be mutated after querying.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Segment {
    Name(String),
    Index(usize),
}

pub fn apply_overlays() -> Result<()> {
    let spec_src =
        fs::read_to_string(SPEC_PATH).with_context(|| format!("cannot read {SPEC_PATH}"))?;
    let mut doc: Value =
        serde_norway::from_str(&spec_src).with_context(|| format!("cannot parse {SPEC_PATH}"))?;

    let files = overlay_files(Path::new(OVERLAYS_DIR))?;
    let mut total_actions = 0;
    for file in &files {
        let src =
            fs::read_to_string(file).with_context(|| format!("cannot read {}", file.display()))?;
        let overlay: Overlay = serde_norway::from_str(&src)
            .with_context(|| format!("cannot parse overlay {}", file.display()))?;
        total_actions += apply_overlay(&mut doc, &overlay)
            .with_context(|| format!("overlay {} failed", file.display()))?;
    }

    let out = Path::new(OUTPUT_PATH);
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let rendered = serde_norway::to_string(&doc).context("cannot serialize patched spec")?;
    fs::write(out, rendered).with_context(|| format!("cannot write {OUTPUT_PATH}"))?;
    println!(
        "applied {} overlay(s), {} action(s): {SPEC_PATH} -> {OUTPUT_PATH}",
        files.len(),
        total_actions
    );
    Ok(())
}

/// `*.yaml` / `*.yml` files in `dir`, sorted by name. A missing directory means no overlays.
fn overlay_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("cannot list {}", dir.display()))? {
        let path = entry?.path();
        let is_yaml = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e == "yaml" || e == "yml");
        if path.is_file() && is_yaml {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Applies every action of `overlay` to `doc`, returning the number of actions applied.
fn apply_overlay(doc: &mut Value, overlay: &Overlay) -> Result<usize> {
    if !overlay.overlay.starts_with("1.0.") {
        bail!(
            "unsupported overlay version `{}` (only 1.0.x is supported)",
            overlay.overlay
        );
    }
    if overlay.actions.is_empty() {
        bail!("overlay `{}` has no actions", overlay.info.title);
    }
    for (i, action) in overlay.actions.iter().enumerate() {
        apply_action(doc, action)
            .with_context(|| format!("action #{} (target `{}`)", i + 1, action.target))?;
    }
    Ok(overlay.actions.len())
}

fn apply_action(doc: &mut Value, action: &Action) -> Result<()> {
    let path = JsonPath::parse(&action.target).map_err(|e| anyhow!("invalid JSONPath: {e}"))?;

    let mut locations = Vec::new();
    for node in path.query_located(doc).dedup() {
        if !(node.node().is_object() || node.node().is_array()) {
            bail!(
                "target selects a primitive value at `{}`; select its containing object instead",
                node.location().to_json_pointer()
            );
        }
        let segments: Vec<Segment> = node
            .location()
            .iter()
            .map(|e| match e {
                PathElement::Name(n) => Segment::Name((*n).to_string()),
                PathElement::Index(i) => Segment::Index(*i),
            })
            .collect();
        locations.push(segments);
    }
    if locations.is_empty() {
        bail!("target matches no nodes (spec changed upstream or the fix is no longer needed?)");
    }

    if action.remove {
        // Deepest/highest-index first, so removing one node never shifts another's index.
        locations.sort_unstable_by(|a, b| b.cmp(a));
        for segments in &locations {
            remove_at(doc, segments)?;
        }
        return Ok(());
    }

    let update = action
        .update
        .as_ref()
        .ok_or_else(|| anyhow!("action has neither `update` nor `remove: true`"))?;
    for segments in &locations {
        let node = node_mut(doc, segments)
            .ok_or_else(|| anyhow!("node `{}` disappeared", pointer(segments)))?;
        match (node, update) {
            (Value::Object(target), Value::Object(src)) => merge(target, src),
            (Value::Object(_), _) => bail!(
                "`update` must be an object when the target is an object (`{}`)",
                pointer(segments)
            ),
            (Value::Array(target), value) => target.push(value.clone()),
            _ => bail!(
                "target `{}` is neither an object nor an array",
                pointer(segments)
            ),
        }
    }
    Ok(())
}

fn merge(target: &mut Map<String, Value>, src: &Map<String, Value>) {
    for (key, value) in src {
        match (target.get_mut(key), value) {
            (Some(Value::Object(t)), Value::Object(s)) => merge(t, s),
            _ => {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

fn node_mut<'a>(doc: &'a mut Value, segments: &[Segment]) -> Option<&'a mut Value> {
    segments.iter().try_fold(doc, |node, seg| match seg {
        Segment::Name(n) => node.as_object_mut()?.get_mut(n),
        Segment::Index(i) => node.as_array_mut()?.get_mut(*i),
    })
}

fn remove_at(doc: &mut Value, segments: &[Segment]) -> Result<()> {
    let (last, parent_path) = segments
        .split_last()
        .ok_or_else(|| anyhow!("cannot remove the document root"))?;
    let parent = node_mut(doc, parent_path)
        .ok_or_else(|| anyhow!("parent of `{}` not found", pointer(segments)))?;
    let removed = match (parent, last) {
        (Value::Object(map), Segment::Name(n)) => map.shift_remove(n).is_some(),
        (Value::Array(arr), Segment::Index(i)) if *i < arr.len() => {
            arr.remove(*i);
            true
        }
        _ => false,
    };
    if !removed {
        bail!("cannot remove `{}`", pointer(segments));
    }
    Ok(())
}

fn pointer(segments: &[Segment]) -> String {
    segments
        .iter()
        .map(|s| match s {
            Segment::Name(n) => format!("/{}", n.replace('~', "~0").replace('/', "~1")),
            Segment::Index(i) => format!("/{i}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn overlay(actions: &str) -> Overlay {
        let src =
            format!("overlay: 1.0.0\ninfo: {{ title: t, version: 1.0.0 }}\nactions:\n{actions}");
        serde_norway::from_str(&src).expect("test overlay must parse")
    }

    fn spec() -> Value {
        json!({
            "paths": {
                "/a": { "get": { "parameters": [
                    { "name": "x", "in": "query" },
                    { "name": "legacy", "in": "query" },
                    { "name": "y", "in": "query" }
                ]}}
            },
            "components": { "schemas": {
                "Booking": {
                    "required": ["id", "endsAt"],
                    "properties": {
                        "id": { "type": "integer" },
                        "endsAt": { "type": "string", "format": "date-time" }
                    }
                }
            }}
        })
    }

    #[test]
    fn update_merges_recursively_and_replaces_arrays() {
        let mut doc = spec();
        let o = overlay(
            "  - target: $.components.schemas.Booking\n    update:\n      required: [id]\n      properties:\n        endsAt: { nullable: true }\n",
        );
        apply_overlay(&mut doc, &o).expect("overlay applies");
        let booking = &doc["components"]["schemas"]["Booking"];
        assert_eq!(booking["required"], json!(["id"]));
        assert_eq!(
            booking["properties"]["endsAt"],
            json!({ "type": "string", "format": "date-time", "nullable": true })
        );
        assert_eq!(booking["properties"]["id"], json!({ "type": "integer" }));
    }

    #[test]
    fn update_on_array_appends() {
        let mut doc = spec();
        let o = overlay(
            "  - target: $.paths['/a'].get.parameters\n    update: { name: z, in: header }\n",
        );
        apply_overlay(&mut doc, &o).expect("overlay applies");
        let params = doc["paths"]["/a"]["get"]["parameters"]
            .as_array()
            .expect("parameters is an array");
        assert_eq!(params.len(), 4);
        assert_eq!(params[3], json!({ "name": "z", "in": "header" }));
    }

    #[test]
    fn remove_with_filter_handles_multiple_indices() {
        let mut doc = spec();
        let o =
            overlay("  - target: $.paths['/a'].get.parameters[?@.name != 'x']\n    remove: true\n");
        apply_overlay(&mut doc, &o).expect("overlay applies");
        assert_eq!(
            doc["paths"]["/a"]["get"]["parameters"],
            json!([{ "name": "x", "in": "query" }])
        );
    }

    #[test]
    fn remove_preserves_key_order() {
        let mut doc = json!({ "o": { "a": {}, "b": {}, "c": {} } });
        let o = overlay("  - target: $.o.a\n    remove: true\n");
        apply_overlay(&mut doc, &o).expect("overlay applies");
        let keys: Vec<_> = doc["o"]
            .as_object()
            .expect("object")
            .keys()
            .cloned()
            .collect();
        assert_eq!(keys, ["b", "c"]);
    }

    #[test]
    fn actions_are_sequential() {
        let mut doc = spec();
        let o = overlay(
            "  - target: $.components.schemas.Booking\n    remove: true\n  - target: $.components.schemas\n    update: { Booking: { type: object } }\n",
        );
        apply_overlay(&mut doc, &o).expect("overlay applies");
        assert_eq!(
            doc["components"]["schemas"]["Booking"],
            json!({ "type": "object" })
        );
    }

    #[test]
    fn unmatched_target_is_an_error() {
        let mut doc = spec();
        let o =
            overlay("  - target: $.components.schemas.Missing\n    update: { nullable: true }\n");
        let err = apply_overlay(&mut doc, &o).expect_err("must fail");
        assert!(format!("{err:#}").contains("matches no nodes"));
    }

    #[test]
    fn primitive_target_is_an_error() {
        let mut doc = spec();
        let o = overlay(
            "  - target: $.components.schemas.Booking.properties.id.type\n    update: { x: 1 }\n",
        );
        let err = apply_overlay(&mut doc, &o).expect_err("must fail");
        assert!(format!("{err:#}").contains("primitive"));
    }

    #[test]
    fn action_without_modifier_is_an_error() {
        let mut doc = spec();
        let o = overlay("  - target: $.components\n");
        let err = apply_overlay(&mut doc, &o).expect_err("must fail");
        assert!(format!("{err:#}").contains("neither"));
    }

    #[test]
    fn unsupported_version_is_an_error() {
        let mut doc = spec();
        let mut o = overlay("  - target: $.components\n    remove: true\n");
        o.overlay = "2.0.0".into();
        assert!(apply_overlay(&mut doc, &o).is_err());
    }
}
