//! Discover independent document styles and their own substyles and locales.
//! This module knows the workspace protocol, never a particular page design.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::workspace::Workspace;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    pub style: String,
    pub substyle: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Definition {
    pub id: String,
    api: u64,
    documents: Vec<String>,
    pub supports_locales: Vec<String>,
    pub pages: Vec<usize>,
    pub default_pages: usize,
    pub default_substyle: String,
    pub substyles: Vec<String>,
    pub defaults: Option<String>,
    #[serde(default)]
    pub fonts: Vec<String>,
}

/// A document/style/substyle/language/country render entry point.
#[derive(Clone, Debug)]
pub struct StyleLeaf {
    pub document: &'static str,
    pub style: String,
    pub substyle: String,
    pub locale: String,
    pub dir: PathBuf,
    pub pages: Vec<usize>,
    pub default_pages: usize,
    pub defaults: Option<PathBuf>,
    pub contract: Value,
}

impl StyleLeaf {
    #[must_use]
    pub fn selection(&self) -> Selection {
        Selection {
            style: self.style.clone(),
            substyle: self.substyle.clone(),
        }
    }

    #[must_use]
    pub fn content(&self) -> PathBuf {
        self.dir.join("content.toml")
    }

    #[must_use]
    pub fn strings(&self) -> PathBuf {
        self.dir.join("strings.toml")
    }

    #[must_use]
    pub fn adapter(&self) -> PathBuf {
        self.dir
            .join("typst")
            .join(format!("{}.typ", self.document))
    }

    #[must_use]
    pub fn substyle_file(&self) -> PathBuf {
        self.dir
            .ancestors()
            .nth(2)
            .expect("leaf has a substyle parent")
            .join("substyle.toml")
    }

    #[must_use]
    pub fn style_dir(&self) -> &Path {
        self.dir
            .ancestors()
            .nth(3)
            .expect("leaf has a style parent")
    }

    #[must_use]
    pub fn output(&self, pages: usize) -> PathBuf {
        let name = if self.document == "cl" && pages == self.default_pages {
            "cl.pdf".to_owned()
        } else {
            format!("{}-{pages}.pdf", self.document)
        };
        self.dir.join("pdf").join(name)
    }
}

fn atom(value: &str, label: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_'),
        "{label}: expected a lowercase name containing letters, numbers, hyphens or underscores"
    );
    Ok(())
}

pub fn normalize_locale(value: &str) -> Result<String> {
    let value = value.to_ascii_lowercase();
    let value = match value.as_str() {
        "de" => "de-ch",
        "en" => "en-ch",
        _ => &value,
    };
    let (language, country) = value
        .split_once('-')
        .context("locale must be language-country")?;
    ensure!(
        (2..=3).contains(&language.len())
            && country.len() == 2
            && language.bytes().all(|c| c.is_ascii_lowercase())
            && country.bytes().all(|c| c.is_ascii_lowercase()),
        "unsupported locale: {value}; expected language-country"
    );
    Ok(value.to_owned())
}

fn manifest_key(document: &str) -> Result<&'static str> {
    match document {
        "cv" => Ok("cv"),
        "cl" => Ok("cover_letter"),
        _ => anyhow::bail!("unknown document: {document}"),
    }
}

pub fn root(workspace: &Workspace, document: &str) -> Result<PathBuf> {
    let manifest = workspace.read_json("ccvl.json")?;
    let key = manifest_key(document)?;
    let path = manifest
        .pointer(&format!("/documents/{key}/root"))
        .and_then(Value::as_str)
        .context("document discovery root is missing")?;
    workspace.existing_inside(path)
}

pub fn default_style(workspace: &Workspace, document: &str) -> Result<String> {
    let manifest = workspace.read_json("ccvl.json")?;
    let key = manifest_key(document)?;
    let name = manifest
        .pointer(&format!("/documents/{key}/default_style"))
        .and_then(Value::as_str)
        .context("document default_style is missing")?;
    definition(workspace, document, name)?;
    Ok(name.to_owned())
}

pub fn definition(workspace: &Workspace, document: &str, name: &str) -> Result<Definition> {
    atom(name, "style")?;
    let directory = root(workspace, document)?.join(name);
    let path = directory.join("style.toml");
    let text = fs::read_to_string(&path).with_context(|| {
        format!(
            "unknown {document} style {name:?}: missing {}",
            path.display()
        )
    })?;
    let definition: Definition =
        toml::from_str(&text).with_context(|| format!("invalid {}", path.display()))?;
    ensure!(
        definition.id == name && definition.api == 1,
        "{}: id must match the directory and api must be 1",
        path.display()
    );
    ensure!(
        definition.documents == [document],
        "{}: documents must contain only {document}",
        path.display()
    );
    ensure!(
        !definition.pages.is_empty()
            && definition.pages.iter().all(|p| *p > 0)
            && definition.pages.contains(&definition.default_pages),
        "{}: invalid page presets or default_pages",
        path.display()
    );
    ensure!(
        !definition.substyles.is_empty()
            && definition.substyles.contains(&definition.default_substyle),
        "{}: default substyle is not listed",
        path.display()
    );
    for name in &definition.substyles {
        atom(name, "substyle")?;
    }
    ensure!(
        !definition.supports_locales.is_empty(),
        "{}: supports_locales is empty",
        path.display()
    );
    for locale in &definition.supports_locales {
        ensure!(
            normalize_locale(locale)? == *locale,
            "{}: locale must be canonical: {locale}",
            path.display()
        );
    }
    ensure!(
        definition.substyles.iter().collect::<BTreeSet<_>>().len() == definition.substyles.len()
            && definition
                .supports_locales
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                == definition.supports_locales.len()
            && definition.pages.iter().collect::<BTreeSet<_>>().len() == definition.pages.len(),
        "{}: duplicate substyles, locales or page presets",
        path.display()
    );
    for relative in definition.defaults.iter().chain(definition.fonts.iter()) {
        workspace.existing_inside(directory.join(relative))?;
    }
    Ok(definition)
}

pub fn definitions(workspace: &Workspace, document: &str) -> Result<Vec<Definition>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(root(workspace, document)?)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            names.push(
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| anyhow::anyhow!("style directory is not UTF-8"))?,
            );
        }
    }
    names.sort();
    ensure!(!names.is_empty(), "no {document} styles found");
    names
        .iter()
        .map(|name| definition(workspace, document, name))
        .collect()
}

pub fn contract(workspace: &Workspace, document: &str, style: &str) -> Result<Value> {
    let path = root(workspace, document)?.join(style).join("contract.toml");
    if path.is_file() {
        let value = workspace.read_toml_value(workspace.relative(&path)?)?;
        validate_contract(&value).with_context(|| format!("invalid {}", path.display()))?;
        Ok(value)
    } else {
        Ok(serde_json::json!({}))
    }
}

fn validate_contract(value: &Value) -> Result<()> {
    for key in [
        "content_fields",
        "metric_rules",
        "shared_pages",
        "paragraphs",
        "paragraph_regions",
        "source_files",
    ] {
        if let Some(value) = value.get(key) {
            ensure!(value.is_array(), "{key} must be an array");
        }
    }
    for key in ["content_fields", "source_files"] {
        if let Some(values) = value.get(key).and_then(Value::as_array) {
            ensure!(
                values.iter().all(Value::is_string),
                "{key} must contain strings"
            );
        }
    }
    if let Some(pages) = value.get("shared_pages").and_then(Value::as_array) {
        ensure!(
            pages.iter().all(|p| p.as_u64().is_some_and(|p| p > 0)),
            "shared_pages must contain positive page numbers"
        );
    }
    if let Some(rules) = value.get("metric_rules").and_then(Value::as_array) {
        for rule in rules {
            ensure!(
                rule.get("kind")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| !kind.is_empty()),
                "metric rule must have a kind"
            );
            for key in ["minimum", "maximum"] {
                if let Some(count) = rule.get(key) {
                    ensure!(
                        count.as_u64().is_some(),
                        "metric {key} must be a non-negative count"
                    );
                }
            }
            let minimum = rule.get("minimum").and_then(Value::as_u64).unwrap_or(0);
            ensure!(
                rule.get("maximum")
                    .and_then(Value::as_u64)
                    .is_none_or(|maximum| maximum >= minimum),
                "metric count bounds are reversed"
            );
        }
    }
    if let Some(policy) = value.get("pdf") {
        ensure!(policy.is_object(), "pdf must be a table");
        if let Some(size) = policy.get("size_pt") {
            let size = size.as_array().context("size_pt must be an array")?;
            ensure!(
                size.len() == 2
                    && size
                        .iter()
                        .all(|n| n.as_f64().is_some_and(|n| n.is_finite() && n > 0.0)),
                "size_pt must contain positive width and height"
            );
        }
        if let Some(pattern) = policy.get("font_pattern") {
            regex::Regex::new(pattern.as_str().context("font_pattern must be text")?)?;
        }
        if let Some(required) = policy.get("required_profile_fields") {
            ensure!(
                required
                    .as_array()
                    .is_some_and(|fields| fields.iter().all(Value::is_string)),
                "required_profile_fields must contain field names"
            );
        }
        if let Some(required) = policy.get("require_image") {
            ensure!(required.is_boolean(), "require_image must be boolean");
        }
        if let Some(minimum) = policy.get("minimum_text_chars") {
            ensure!(
                minimum.as_u64().is_some_and(|n| n > 0),
                "minimum_text_chars must be positive"
            );
        }
    }
    Ok(())
}

pub fn leaves(workspace: &Workspace, document: &'static str) -> Result<Vec<StyleLeaf>> {
    let mut leaves = Vec::new();
    for style in definitions(workspace, document)? {
        let directory = root(workspace, document)?.join(&style.id);
        let contract = contract(workspace, document, &style.id)?;
        let defaults = style
            .defaults
            .as_ref()
            .map(|path| workspace.existing_inside(directory.join(path)))
            .transpose()?;
        for substyle in &style.substyles {
            workspace.read_toml_value(
                workspace.relative(&directory.join(substyle).join("substyle.toml"))?,
            )?;
            for locale in &style.supports_locales {
                let (language, country) = locale.split_once('-').expect("validated locale");
                let leaf = StyleLeaf {
                    document,
                    style: style.id.clone(),
                    substyle: substyle.clone(),
                    locale: locale.clone(),
                    dir: directory.join(substyle).join(language).join(country),
                    pages: style.pages.clone(),
                    default_pages: style.default_pages,
                    defaults: defaults.clone(),
                    contract: contract.clone(),
                };
                for path in [leaf.content(), leaf.strings(), leaf.adapter()] {
                    workspace.existing_inside(&path).with_context(|| {
                        format!(
                            "incomplete {document}/{}/{substyle}/{locale} leaf",
                            style.id
                        )
                    })?;
                }
                leaves.push(leaf);
            }
        }
    }
    Ok(leaves)
}

pub fn leaf(
    workspace: &Workspace,
    document: &'static str,
    locale: &str,
    selection: &Selection,
) -> Result<StyleLeaf> {
    let locale = normalize_locale(locale)?;
    leaves(workspace, document)?
        .into_iter()
        .find(|leaf| {
            leaf.style == selection.style
                && leaf.substyle == selection.substyle
                && leaf.locale == locale
        })
        .with_context(|| {
            format!(
                "no {document} leaf for {}/{}/{locale}",
                selection.style, selection.substyle
            )
        })
}

pub fn selection(
    workspace: &Workspace,
    document: &str,
    style: Option<&str>,
    substyle: Option<&str>,
) -> Result<Selection> {
    let style = match style {
        Some(name) => name.to_owned(),
        None => default_style(workspace, document)?,
    };
    let definition = definition(workspace, document, &style)?;
    let substyle = substyle.unwrap_or(&definition.default_substyle);
    ensure!(
        definition.substyles.iter().any(|name| name == substyle),
        "unknown {document} substyle {substyle:?} for style {style:?}; expected one of {}",
        definition.substyles.join(", ")
    );
    Ok(Selection {
        style,
        substyle: substyle.to_owned(),
    })
}

pub fn record_selection(
    workspace: &Workspace,
    document: &str,
    record: &Value,
    location: &str,
) -> Result<Selection> {
    let name = |suffix: &str| -> Result<Option<&str>> {
        let key = format!("/options/{document}_{suffix}");
        record
            .pointer(&key)
            .map(|value| {
                value
                    .as_str()
                    .with_context(|| format!("{location}{key} must be a {suffix} name"))
            })
            .transpose()
            .map(|name| name.filter(|name| !name.is_empty()))
    };
    selection(workspace, document, name("style")?, name("substyle")?)
}

pub fn font_paths(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut paths = BTreeSet::new();
    for document in ["cv", "cl"] {
        for style in definitions(workspace, document)? {
            let directory = root(workspace, document)?.join(&style.id);
            for font in style.fonts {
                paths.insert(workspace.existing_inside(directory.join(font))?);
            }
        }
    }
    Ok(paths.into_iter().collect())
}

#[cfg(test)]
mod tests;
