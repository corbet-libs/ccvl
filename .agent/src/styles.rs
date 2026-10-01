//! Discover independent document styles and their own substyles and locales.
//! This module knows the workspace protocol, never a particular page design.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
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

/// The reserved selection name for the manifest default style or the
/// selected style's first substyle. No real style or substyle may use it.
const DEFAULT: &str = "default";

/// The only keys of an empty style slot's `style.toml`.
const EMPTY_STYLE_KEYS: [&str; 5] = ["id", "api", "documents", "empty", "substyles"];

#[derive(Clone, Debug, Deserialize)]
pub struct Definition {
    pub id: String,
    api: u64,
    documents: Vec<String>,
    #[serde(default)]
    pub supports_locales: Vec<String>,
    #[serde(default)]
    pub pages: Vec<usize>,
    #[serde(default)]
    pub default_pages: usize,
    /// Ordered positions; the first is the default substyle.
    pub substyles: Vec<String>,
    /// Positions of `substyles` without a design yet, each named `slot-<n>`.
    #[serde(default)]
    pub empty_substyles: Vec<String>,
    /// An undesigned style slot: no locales, pages, leaves or assets.
    #[serde(default)]
    pub empty: bool,
    pub defaults: Option<String>,
    pub settings_adapter: Option<String>,
    pub paper: Option<crate::paper::Registry>,
    #[serde(default)]
    pub fonts: Vec<String>,
}

impl Definition {
    /// The first substyle is the default; validation keeps it designed.
    #[must_use]
    pub fn default_substyle(&self) -> &str {
        &self.substyles[0]
    }

    #[must_use]
    pub fn is_empty_slot(&self, substyle: &str) -> bool {
        self.empty || self.empty_substyles.iter().any(|name| name == substyle)
    }

    /// Substyles with leaves, in declared order.
    pub fn designed_substyles(&self) -> impl Iterator<Item = &String> {
        self.substyles
            .iter()
            .filter(|name| !self.is_empty_slot(name))
    }
}

/// The position of a `slot-<n>` name; these names are reserved for empty slots.
fn slot_number(name: &str) -> Option<usize> {
    let number = name.strip_prefix("slot-")?.parse::<usize>().ok()?;
    (number > 0 && format!("slot-{number}") == name).then_some(number)
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
    ensure!(
        !definition(workspace, document, name)?.empty,
        "document default_style {name:?} is an empty style slot"
    );
    Ok(name.to_owned())
}

pub fn definition(workspace: &Workspace, document: &str, name: &str) -> Result<Definition> {
    atom(name, "style")?;
    ensure!(
        name != DEFAULT,
        "style name {DEFAULT:?} is reserved for the workspace default style"
    );
    let directory = root(workspace, document)?.join(name);
    let path = directory.join("style.toml");
    let text = workspace.read_text(&path).with_context(|| {
        format!(
            "unknown {document} style {name:?}: missing {}",
            path.display()
        )
    })?;
    let keys: toml::Table =
        toml::from_str(&text).with_context(|| format!("invalid {}", path.display()))?;
    ensure!(
        !keys.contains_key("default_substyle"),
        "{}: default_substyle was removed; the first substyle is the default",
        path.display()
    );
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
        !definition.substyles.is_empty(),
        "{}: substyles is empty",
        path.display()
    );
    for substyle in &definition.substyles {
        atom(substyle, "substyle")?;
        ensure!(
            substyle != DEFAULT,
            "{}: substyle name {DEFAULT:?} is reserved for the first substyle",
            path.display()
        );
    }
    ensure!(
        definition.substyles.iter().collect::<BTreeSet<_>>().len() == definition.substyles.len(),
        "{}: duplicate substyles",
        path.display()
    );
    if definition.empty {
        validate_empty_style(&definition, &keys, &path)?;
        return Ok(definition);
    }
    ensure!(
        slot_number(name).is_none(),
        "{}: slot-<n> names are reserved for empty style slots; set empty = true or choose a style name",
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
        definition
            .empty_substyles
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            == definition.empty_substyles.len()
            && definition
                .empty_substyles
                .iter()
                .all(|slot| definition.substyles.contains(slot)),
        "{}: empty_substyles must list distinct entries of substyles",
        path.display()
    );
    for (index, substyle) in definition.substyles.iter().enumerate() {
        let slot = format!("slot-{}", index + 1);
        if definition.is_empty_slot(substyle) {
            ensure!(
                index > 0,
                "{}: the first substyle is the default and cannot be an empty slot",
                path.display()
            );
            ensure!(
                *substyle == slot,
                "{}: empty substyle {substyle:?} must be named {slot:?} for its position",
                path.display()
            );
            ensure!(
                !directory.join(substyle).exists(),
                "{}: empty substyle slot {substyle:?} must not have a directory",
                path.display()
            );
        } else {
            ensure!(
                slot_number(substyle).is_none(),
                "{}: substyle {substyle:?} uses a reserved slot name; list it in empty_substyles or rename it",
                path.display()
            );
        }
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
        definition
            .supports_locales
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            == definition.supports_locales.len()
            && definition.pages.iter().collect::<BTreeSet<_>>().len() == definition.pages.len(),
        "{}: duplicate locales or page presets",
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

/// An empty style slot reserves a position: its name and every substyle are
/// `slot-<n>` by position, and it declares nothing that could render.
fn validate_empty_style(definition: &Definition, keys: &toml::Table, path: &Path) -> Result<()> {
    ensure!(
        keys.keys()
            .all(|key| EMPTY_STYLE_KEYS.contains(&key.as_str())),
        "{}: an empty style slot contains only {}",
        path.display(),
        EMPTY_STYLE_KEYS.join(", ")
    );
    ensure!(
        slot_number(&definition.id).is_some(),
        "{}: an empty style slot must be named slot-<n>",
        path.display()
    );
    for (index, substyle) in definition.substyles.iter().enumerate() {
        let slot = format!("slot-{}", index + 1);
        ensure!(
            *substyle == slot,
            "{}: empty style substyle {substyle:?} must be named {slot:?} for its position",
            path.display()
        );
    }
    Ok(())
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

/// The manifest's `slots` fixes how many styles a document offers and how
/// many substyles each style lists. Undesigned positions are empty slots, and
/// empty style slots follow the designed styles by number.
pub fn validate_slots(workspace: &Workspace, document: &str) -> Result<()> {
    let manifest = workspace.read_json("ccvl.json")?;
    let key = manifest_key(document)?;
    let slots = manifest
        .pointer(&format!("/documents/{key}/slots"))
        .and_then(Value::as_object)
        .filter(|slots| slots.len() == 2)
        .with_context(|| {
            format!("ccvl.json: documents.{key}.slots must declare only styles and substyles")
        })?;
    let count = |name: &str| {
        slots
            .get(name)
            .and_then(Value::as_u64)
            .filter(|count| *count > 0)
            .and_then(|count| usize::try_from(count).ok())
            .with_context(|| {
                format!("ccvl.json: documents.{key}.slots.{name} must be a positive integer")
            })
    };
    let (styles, substyles) = (count("styles")?, count("substyles")?);
    let definitions = definitions(workspace, document)?;
    ensure!(
        definitions.len() == styles,
        "{document} must offer exactly {styles} styles, designed or empty slots; found {}",
        definitions.len()
    );
    let designed = definitions.iter().filter(|style| !style.empty).count();
    for definition in &definitions {
        ensure!(
            definition.substyles.len() == substyles,
            "{document} style {} must list exactly {substyles} substyles, designed or empty slots; found {}",
            definition.id,
            definition.substyles.len()
        );
        if definition.empty {
            ensure!(
                slot_number(&definition.id).is_some_and(|number| number > designed),
                "{document} empty style slots must be numbered slot-{} to slot-{styles} after the designed styles; found {}",
                designed + 1,
                definition.id
            );
        }
    }
    Ok(())
}

/// Describe each document's styles in slot order: the default style first,
/// other designed styles alphabetically, then empty style slots by number.
pub fn list_styles(workspace: &Workspace) -> Result<Value> {
    let status = |empty: bool| if empty { "empty" } else { "designed" };
    let mut documents = serde_json::Map::new();
    for document in ["cv", "cl"] {
        let default = selection(workspace, document, None, None)?;
        let mut definitions = definitions(workspace, document)?;
        definitions.sort_by_key(|style| {
            (
                style.empty,
                style.id != default.style,
                slot_number(&style.id),
                style.id.clone(),
            )
        });
        let styles = definitions
            .iter()
            .map(|style| {
                let substyles = style
                    .substyles
                    .iter()
                    .enumerate()
                    .map(|(index, substyle)| {
                        let mut entry = serde_json::json!({
                            "id": substyle,
                            "status": status(style.is_empty_slot(substyle)),
                        });
                        if index == 0 {
                            entry["default"] = true.into();
                        }
                        entry
                    })
                    .collect::<Vec<_>>();
                serde_json::json!({
                    "id": style.id,
                    "status": status(style.empty),
                    "substyles": substyles,
                })
            })
            .collect::<Vec<_>>();
        documents.insert(
            document.to_owned(),
            serde_json::json!({
                "default": {"style": default.style, "substyle": default.substyle},
                "styles": styles,
            }),
        );
    }
    Ok(Value::Object(documents))
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
            style.designed_substyles().any(|name| name == substyle)
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
        if definition.empty {
            continue;
        }
        let style = ResolvedStyle::new(workspace, document, definition)?;
        for substyle in style.definition.designed_substyles() {
            for locale in &style.definition.supports_locales {
                leaves.push(style.leaf(workspace, substyle, locale)?);
            }
        }
    }
    Ok(leaves)
}

/// Repeatable `<doc>/<style>` selections that limit which styles' documents
/// are enumerated for rendering and measurement. An empty filter selects every
/// style. Workspace-wide validation never consults this filter.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StyleFilter {
    selected: BTreeSet<(&'static str, String)>,
    /// Selected empty style slots: valid selections without documents.
    empty: BTreeSet<(&'static str, String)>,
}

impl StyleFilter {
    #[must_use]
    pub fn all() -> Self {
        Self::default()
    }

    /// Parse `<doc>/<style>` values; each must name an existing style. An
    /// empty style slot is a valid selection that contributes no documents.
    pub fn parse<S: AsRef<str>>(workspace: &Workspace, values: &[S]) -> Result<Self> {
        let mut selected = BTreeSet::new();
        let mut empty = BTreeSet::new();
        for value in values {
            let value = value.as_ref();
            let Some((document, style)) = value.split_once('/') else {
                bail!("--style {value:?}: expected <cv|cl>/<style>, for example cv/harvard");
            };
            let document = match document {
                "cv" => "cv",
                "cl" => "cl",
                _ => bail!(
                    "--style {value:?}: unknown document {document:?}; expected cv or cl before the slash"
                ),
            };
            let definition = definition(workspace, document, style)
                .with_context(|| format!("--style {value:?} does not name an existing style"))?;
            if definition.empty {
                empty.insert((document, style.to_owned()));
            }
            selected.insert((document, style.to_owned()));
        }
        Ok(Self { selected, empty })
    }

    #[must_use]
    pub fn is_all(&self) -> bool {
        self.selected.is_empty()
    }

    #[must_use]
    pub fn includes(&self, document: &str, style: &str) -> bool {
        self.is_all()
            || self
                .selected
                .iter()
                .any(|(selected, name)| *selected == document && name == style)
    }

    /// The selected styles' leaves for one document, in discovery order.
    pub fn leaves(&self, workspace: &Workspace, document: &'static str) -> Result<Vec<StyleLeaf>> {
        Ok(leaves(workspace, document)?
            .into_iter()
            .filter(|leaf| self.includes(document, &leaf.style))
            .collect())
    }

    /// The selected CV leaves followed by the selected cover-letter leaves.
    pub fn document_leaves(&self, workspace: &Workspace) -> Result<Vec<StyleLeaf>> {
        let mut leaves = self.leaves(workspace, "cv")?;
        leaves.extend(self.leaves(workspace, "cl")?);
        Ok(leaves)
    }
}

impl fmt::Display for StyleFilter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_all() {
            return formatter.write_str("every style");
        }
        let names = self
            .selected
            .iter()
            .map(|selection| {
                let (document, style) = selection;
                if self.empty.contains(selection) {
                    format!("{document}/{style} (empty style slot without documents)")
                } else {
                    format!("{document}/{style}")
                }
            })
            .collect::<Vec<_>>();
        formatter.write_str(&names.join(", "))
    }
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
        Some(name) if name != DEFAULT => name.to_owned(),
        _ => default_style(workspace, document)?,
    };
    let definition = definition(workspace, document, &style)?;
    ensure!(
        !definition.empty,
        "{document} {style} is an empty style slot; it has no design yet"
    );
    let substyle = match substyle {
        Some(name) if name != DEFAULT => name,
        _ => definition.default_substyle(),
    };
    ensure!(
        definition.substyles.iter().any(|name| name == substyle),
        "unknown {document} substyle {substyle:?} for style {style:?}; expected one of {}",
        definition.substyles.join(", ")
    );
    ensure!(
        !definition.is_empty_slot(substyle),
        "{document} {style}/{substyle} is an empty slot; it has no design yet"
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
