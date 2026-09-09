//! Optional style-owned paper presets; names and renderer settings are opaque.
use crate::styles::StyleLeaf;
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub defaults: BTreeMap<String, String>,
    pub sizes: BTreeMap<String, Preset>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub size_pt: [f64; 2],
    pub label: String,
    pub settings: Value,
}

impl Registry {
    pub fn validate(&self, locales: &[String]) -> Result<()> {
        ensure!(
            !self.sizes.is_empty(),
            "paper.sizes must declare at least one preset"
        );
        ensure!(
            self.defaults.keys().collect::<BTreeSet<_>>()
                == locales.iter().collect::<BTreeSet<_>>(),
            "paper.defaults must declare exactly the supported locales"
        );
        for (id, preset) in &self.sizes {
            crate::styles::atom(id, "paper preset")?;
            ensure!(
                preset.size_pt.iter().all(|n| n.is_finite() && *n > 0.0),
                "paper.sizes.{id}.size_pt must contain positive finite dimensions"
            );
            ensure!(
                !preset.label.trim().is_empty(),
                "paper.sizes.{id}.label must not be empty"
            );
            ensure!(
                preset.settings.is_object(),
                "paper.sizes.{id}.settings must be a table"
            );
        }
        for id in self.defaults.values() {
            ensure!(
                self.sizes.contains_key(id),
                "paper.defaults selects unsupported paper {id:?}"
            );
        }
        Ok(())
    }
}

pub fn record_choice<'a>(record: &'a Value, document: &str) -> Result<Option<&'a str>> {
    record
        .pointer(&format!("/options/{document}_paper"))
        .map(|value| {
            value
                .as_str()
                .with_context(|| format!("options.{document}_paper must be a preset name"))
        })
        .transpose()
}

pub fn select<'a>(
    registry: Option<&'a Registry>,
    locale: &str,
    requested: Option<&str>,
) -> Result<Option<(&'a str, &'a Preset)>> {
    let Some(registry) = registry else {
        ensure!(
            requested.is_none(),
            "this style has fixed geometry and does not declare paper presets"
        );
        return Ok(None);
    };
    let id = requested
        .or_else(|| registry.defaults.get(locale).map(String::as_str))
        .with_context(|| format!("paper.defaults has no default for {locale}"))?;
    let (id, preset) = registry.sizes.get_key_value(id).with_context(|| {
        format!(
            "unsupported paper {id:?}; expected {:?}",
            registry.sizes.keys().collect::<Vec<_>>()
        )
    })?;
    Ok(Some((id.as_str(), preset)))
}

#[must_use]
pub fn contract(leaf: &StyleLeaf, selected: Option<(&str, &Preset)>) -> Value {
    let mut contract = leaf.contract.clone();
    if let Some((_, preset)) = selected {
        if contract.get("pdf").is_none() {
            contract["pdf"] = json!({});
        }
        contract["pdf"]["size_pt"] = json!(preset.size_pt);
    }
    contract
}

#[cfg(test)]
mod tests;
