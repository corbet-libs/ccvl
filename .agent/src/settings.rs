//! Optional document-v1 adapter settings. Independent renderers keep their own schemas.
use anyhow::{Context, Result, ensure};
use serde_json::{Map, Value, json};

use crate::{styles::StyleLeaf, workspace::Workspace};

const SCHEMA: &str = ".agent/typst/document-settings.json";

/// Explain the family → substyle → locale → paper merge used by document-v1.
/// Component-level Typst overrides are intentionally outside this result.
pub fn explain(workspace: &Workspace, leaf: &StyleLeaf, requested: Option<&str>) -> Result<Value> {
    let record = crate::content::read_record(workspace, leaf.content())?;
    let recorded = crate::paper::record_choice(&record, leaf.document)?;
    let mut result = resolve_with_paper(workspace, leaf, requested.or(recorded))?;
    if result["paper"].is_object() {
        result["paper"]["selection_source"] = if requested.is_some() {
            "CLI --paper".into()
        } else if recorded.is_some() {
            format!(
                "{}#options.{}_paper",
                workspace
                    .relative(&leaf.content())?
                    .to_string_lossy()
                    .replace('\\', "/"),
                leaf.document
            )
            .into()
        } else {
            format!(
                "{}#paper.defaults.{}",
                workspace
                    .relative(&leaf.style_dir().join("style.toml"))?
                    .to_string_lossy()
                    .replace('\\', "/"),
                leaf.locale
            )
            .into()
        };
    }
    Ok(result)
}

pub fn resolve(workspace: &Workspace, leaf: &StyleLeaf) -> Result<Value> {
    resolve_with_paper(workspace, leaf, None)
}

pub fn resolve_with_paper(
    workspace: &Workspace,
    leaf: &StyleLeaf,
    requested: Option<&str>,
) -> Result<Value> {
    ensure!(
        leaf.settings_adapter.as_deref() == Some("document-v1"),
        "{}/{} does not declare settings_adapter = document-v1; its renderer owns settings resolution",
        leaf.document,
        leaf.style
    );
    let schema = workspace.read_json(SCHEMA)?;
    let mut paths = Vec::new();
    if let Some(defaults) = &leaf.defaults {
        paths.push(defaults.clone());
    }
    paths.push(leaf.substyle_file());
    let layout = leaf.dir.join("layout.toml");
    if workspace.input_is_file(&layout) {
        paths.push(layout);
    }
    let mut layers = Vec::new();
    let mut settings = json!({});
    for path in paths {
        let path = workspace.existing_inside(path)?;
        let path = workspace.relative(&path)?;
        let source = path.to_string_lossy().replace('\\', "/");
        let value = workspace.read_toml_value(&path)?;
        if leaf.paper.is_some() {
            for key in ["paper", "width_mm", "height_mm"] {
                ensure!(
                    value.pointer(&format!("/page/{key}")).is_none(),
                    "{source}: paper presets own page.{key}; remove the duplicate geometry setting"
                );
            }
        }
        validate(&value, &schema, false)
            .with_context(|| format!("invalid settings in {source}"))?;
        merge(&mut settings, &value);
        layers.push((source, value));
    }
    let selected = crate::paper::select(leaf.paper.as_ref(), &leaf.locale, requested)?;
    if let Some((id, preset)) = selected {
        validate(&preset.settings, &schema, false)
            .with_context(|| format!("invalid paper preset {id}"))?;
        merge(&mut settings, &preset.settings);
        let path = workspace.relative(&leaf.style_dir().join("style.toml"))?;
        layers.push((
            format!(
                "{}#paper.sizes.{id}.settings",
                path.to_string_lossy().replace('\\', "/")
            ),
            preset.settings.clone(),
        ));
    }
    validate(&settings, &schema, true).with_context(|| {
        format!(
            "invalid merged settings for {}/{}/{}",
            leaf.style, leaf.substyle, leaf.locale
        )
    })?;
    let locale = format!(
        "{}-{}",
        settings["text"]["lang"].as_str().unwrap(),
        settings["text"]["region"]
            .as_str()
            .unwrap()
            .to_ascii_lowercase()
    );
    ensure!(
        locale == leaf.locale,
        "settings text.lang/text.region resolve to {locale}, expected {}",
        leaf.locale
    );
    let mut origins = Map::new();
    origins_for(&settings, "", &layers, &mut origins);
    Ok(json!({
        "document": leaf.document, "style": leaf.style, "substyle": leaf.substyle,
        "locale": leaf.locale, "adapter": "document-v1",
        "paper": selected.map(|(id, preset)| json!({"id": id, "label": preset.label, "size_pt": preset.size_pt, "selection_source": if requested.is_some() { "explicit" } else { "style locale default" }})),
        "scope": "Merged adapter inputs; renderers may apply component-level overrides",
        "sources": layers.iter().map(|(path, _)| path).collect::<Vec<_>>(),
        "settings": settings, "origins": origins,
    }))
}

fn merge(base: &mut Value, delta: &Value) {
    if let (Some(base), Some(delta)) = (base.as_object_mut(), delta.as_object()) {
        for (key, value) in delta {
            merge(base.entry(key).or_insert(Value::Null), value);
        }
    } else {
        *base = delta.clone();
    }
}

fn origins_for(
    value: &Value,
    pointer: &str,
    layers: &[(String, Value)],
    origins: &mut Map<String, Value>,
) {
    if let Some(fields) = value.as_object().filter(|fields| !fields.is_empty()) {
        for (key, value) in fields {
            let escaped = key.replace('~', "~0").replace('/', "~1");
            origins_for(value, &format!("{pointer}/{escaped}"), layers, origins);
        }
    } else if let Some((source, _)) = layers
        .iter()
        .rev()
        .find(|(_, layer)| layer.pointer(pointer).is_some())
    {
        origins.insert(pointer.to_owned(), source.clone().into());
    }
}

fn validate(settings: &Value, schema: &Value, complete: bool) -> Result<()> {
    let settings = settings.as_object().context("settings must be a table")?;
    for (group, rules) in schema
        .as_object()
        .context("adapter schema must be an object")?
    {
        let Some(values) = settings.get(group) else {
            ensure!(!complete, "missing {group} settings");
            continue;
        };
        let values = values
            .as_object()
            .with_context(|| format!("{group} must be a table"))?;
        let rules = rules
            .as_object()
            .context("adapter schema group must be an object")?;
        for (key, value) in values {
            let field = format!("{group}.{key}");
            let rule = rules
                .get(key)
                .with_context(|| format!("unknown adapter setting {field}"))?;
            validate_field(value, rule).with_context(|| format!("{field} = {value}"))?;
        }
        if complete {
            for (key, rule) in rules {
                ensure!(
                    rule["optional"] == true || values.contains_key(key),
                    "missing {group}.{key}"
                );
            }
        }
    }
    if complete {
        let page = &settings["page"];
        if page["paper"] == "custom" {
            ensure!(
                page.get("width_mm").is_some() && page.get("height_mm").is_some(),
                "custom paper requires page.width_mm and page.height_mm"
            );
        } else {
            ensure!(
                page.get("width_mm").is_none() && page.get("height_mm").is_none(),
                "page.width_mm/height_mm are unused unless page.paper = custom"
            );
        }
        let paragraph = &settings["paragraph"];
        for (min, max) in [
            ("word_spacing_min_percent", "word_spacing_max_percent"),
            ("tracking_min_pt", "tracking_max_pt"),
        ] {
            ensure!(
                paragraph[min].as_f64() <= paragraph[max].as_f64(),
                "paragraph.{min} exceeds paragraph.{max}"
            );
        }
    }
    Ok(())
}

fn color(value: &Value) -> bool {
    value.as_str().is_some_and(|value| {
        let digits = value.strip_prefix('#').unwrap_or(value);
        [3, 4, 6, 8].contains(&digits.len()) && digits.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn nonempty_text(value: &Value) -> bool {
    value.as_str().is_some_and(|value| !value.trim().is_empty())
}

fn number(value: &Value) -> bool {
    value.as_f64().is_some_and(f64::is_finite)
}

fn tags(value: &Value, features: bool) -> bool {
    value.as_object().is_some_and(|values| {
        values.iter().all(|(key, value)| {
            key.len() == 4
                && key.bytes().all(|byte| byte.is_ascii_graphic())
                && if features {
                    value.is_boolean() || value.as_i64().is_some()
                } else {
                    number(value)
                }
        })
    })
}

fn validate_field(value: &Value, rule: &Value) -> Result<()> {
    let kind = rule["type"]
        .as_str()
        .context("adapter field type is missing")?;
    let valid = match kind {
        "boolean" => value.is_boolean(),
        "boolean-auto" => value.is_boolean() || value == "auto",
        "number" => number(value),
        "integer" => value.as_i64().is_some(),
        "string" => nonempty_text(value),
        "color" => color(value),
        "color-auto" => color(value) || value == "auto",
        "font" => {
            nonempty_text(value)
                || value
                    .as_array()
                    .is_some_and(|fonts| !fonts.is_empty() && fonts.iter().all(nonempty_text))
        }
        "features" => {
            tags(value, true)
                || value.as_array().is_some_and(|tags| {
                    tags.iter().all(|tag| {
                        tag.as_str().is_some_and(|tag| {
                            tag.len() == 4 && tag.bytes().all(|byte| byte.is_ascii_graphic())
                        })
                    })
                })
        }
        "variations" => tags(value, false),
        _ => anyhow::bail!("unknown adapter schema type {kind}"),
    };
    ensure!(valid, "expected {kind}");
    if let Some(allowed) = rule["enum"].as_array() {
        ensure!(allowed.contains(value), "expected one of {}", rule["enum"]);
    }
    if let Some(actual) = value.as_f64() {
        for (bound, okay) in [
            (
                "minimum",
                rule["minimum"].as_f64().is_none_or(|min| actual >= min),
            ),
            (
                "exclusive_minimum",
                rule["exclusive_minimum"]
                    .as_f64()
                    .is_none_or(|min| actual > min),
            ),
            (
                "maximum",
                rule["maximum"].as_f64().is_none_or(|max| actual <= max),
            ),
        ] {
            ensure!(okay, "violates {bound} {}", rule[bound]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
