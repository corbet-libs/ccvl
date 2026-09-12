//! Discover independent document styles and their own substyles and locales.
//! This module knows the workspace protocol, never a particular page design.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::workspace::Workspace;

mod contract;
use contract::validate_contract;

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
    pub settings_adapter: Option<String>,
    pub paper: Option<crate::paper::Registry>,
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
    pub fonts: Vec<PathBuf>,
    pub settings_adapter: Option<String>,
    pub paper: Option<crate::paper::Registry>,
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
        self.output_for_paper(pages, None)
    }

    #[must_use]
    pub fn output_for_paper(&self, pages: usize, paper: Option<&str>) -> PathBuf {
        let name = if self.document == "cl" && pages == self.default_pages {
            "cl.pdf".to_owned()
        } else {
            format!("{}-{pages}.pdf", self.document)
        };
        let name = if let Some(paper) = paper.filter(|id| {
            self.paper
                .as_ref()
                .and_then(|registry| registry.defaults.get(&self.locale))
                .map(String::as_str)
                != Some(*id)
        }) {
            format!(
                "{}-{paper}.pdf",
                name.strip_suffix(".pdf").expect("PDF name")
            )
        } else {
            name
        };
        self.dir.join("pdf").join(name)
    }
}

pub(crate) fn atom(value: &str, label: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_'),
        "{label}: expected a lowercase name containing letters, numbers, hyphens or underscores"
    );
    Ok(())
}

pub use ccvl_core::normalize_locale;

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
    let text = workspace.read_text(&path).with_context(|| {
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
    ensure!(
        definition
            .settings_adapter
            .as_deref()
            .is_none_or(|value| value == "document-v1"),
        "{}: unknown settings_adapter; expected document-v1 or omit for a custom renderer",
        path.display()
    );
    if let Some(paper) = &definition.paper {
        paper
            .validate(&definition.supports_locales)
            .with_context(|| format!("invalid {}", path.display()))?;
    }
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
    if workspace.input_is_file(&path) {
        let value = workspace.read_toml_value(workspace.relative(&path)?)?;
        validate_contract(&value).with_context(|| format!("invalid {}", path.display()))?;
        Ok(value)
    } else {
        Ok(serde_json::json!({}))
    }
}

/// One resolved style supplies either its requested leaf or its complete set.
/// Resolving a document never opens another style's definition or assets.
struct ResolvedStyle {
    document: &'static str,
    definition: Definition,
    directory: PathBuf,
    defaults: Option<PathBuf>,
    fonts: Vec<PathBuf>,
    contract: Value,
}

impl ResolvedStyle {
    fn new(workspace: &Workspace, document: &'static str, definition: Definition) -> Result<Self> {
        let directory = root(workspace, document)?.join(&definition.id);
        let defaults = definition
            .defaults
            .as_ref()
            .map(|path| workspace.existing_inside(directory.join(path)))
            .transpose()?;
        let fonts = definition
            .fonts
            .iter()
            .map(|path| workspace.existing_inside(directory.join(path)))
            .collect::<Result<BTreeSet<_>>>()?
            .into_iter()
            .collect();
        let contract = contract(workspace, document, &definition.id)?;
        if definition.paper.is_some() {
            ensure!(
                contract.pointer("/pdf/size_pt").is_none()
                    && contract
                        .pointer("/pdf/by_locale")
                        .and_then(Value::as_object)
                        .is_none_or(|locales| locales
                            .values()
                            .all(|policy| policy.get("size_pt").is_none())),
                "paper preset size_pt owns geometry; remove duplicate PDF size_pt from contract.toml"
            );
        }
        Ok(Self {
            document,
            definition,
            directory,
            defaults,
            fonts,
            contract,
        })
    }

    fn leaf(&self, workspace: &Workspace, substyle: &str, locale: &str) -> Result<StyleLeaf> {
        let style = &self.definition;
        ensure!(
            style.substyles.iter().any(|name| name == substyle)
                && style.supports_locales.iter().any(|name| name == locale),
            "no {} leaf for {}/{substyle}/{locale}",
            self.document,
            style.id
        );
        let substyle_file =
            workspace.existing_inside(self.directory.join(substyle).join("substyle.toml"))?;
        workspace.read_toml_value(workspace.relative(&substyle_file)?)?;
        let (language, country) = locale.split_once('-').expect("validated locale");
        let mut resolved_contract = self.contract.clone();
        if let Some(overrides) = self.contract.pointer("/pdf/by_locale") {
            let policy = overrides
                .get(locale)
                .and_then(Value::as_object)
                .with_context(|| format!("missing PDF policy for {locale}"))?;
            let pdf = resolved_contract
                .get_mut("pdf")
                .and_then(Value::as_object_mut)
                .context("PDF policy must be a table")?;
            pdf.remove("by_locale");
            pdf.extend(policy.clone());
        }
        let mut leaf = StyleLeaf {
            document: self.document,
            style: style.id.clone(),
            substyle: substyle.to_owned(),
            locale: locale.to_owned(),
            dir: self.directory.join(substyle).join(language).join(country),
            pages: style.pages.clone(),
            default_pages: style.default_pages,
            defaults: self.defaults.clone(),
            fonts: self.fonts.clone(),
            settings_adapter: style.settings_adapter.clone(),
            paper: style.paper.clone(),
            contract: resolved_contract,
        };
        leaf.contract = crate::paper::contract(
            &leaf,
            crate::paper::select(leaf.paper.as_ref(), locale, None)?,
        );
        for path in [leaf.content(), leaf.strings(), leaf.adapter()] {
            workspace.existing_inside(&path).with_context(|| {
                format!(
                    "incomplete {}/{}/{substyle}/{locale} leaf",
                    self.document, style.id
                )
            })?;
        }
        Ok(leaf)
    }
}

pub fn leaves(workspace: &Workspace, document: &'static str) -> Result<Vec<StyleLeaf>> {
    let mut leaves = Vec::new();
    for definition in definitions(workspace, document)? {
        let style = ResolvedStyle::new(workspace, document, definition)?;
        for substyle in &style.definition.substyles {
            for locale in &style.definition.supports_locales {
                leaves.push(style.leaf(workspace, substyle, locale)?);
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
    let definition = definition(workspace, document, &selection.style)?;
    let requested = cletter::normalize_locale_id(locale);
    // Language shorthand is a selection among this style's declared leaves,
    // never a global country default or a correspondence-table fallback.
    let locale = if requested.contains('-') {
        normalize_locale(&requested)?
    } else {
        let mut matches = definition.supports_locales.iter().filter(|locale| {
            locale.split_once('-').map(|(language, _)| language) == Some(requested.as_str())
        });
        let resolved = matches.next().with_context(|| {
            format!(
                "style {} has no locale for language {requested}",
                definition.id
            )
        })?;
        ensure!(
            matches.next().is_none(),
            "ambiguous language {requested} for style {}; select an explicit locale from {}",
            definition.id,
            definition.supports_locales.join(", ")
        );
        resolved.clone()
    };
    ResolvedStyle::new(workspace, document, definition)?.leaf(
        workspace,
        &selection.substyle,
        &locale,
    )
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

#[cfg(test)]
mod tests;
