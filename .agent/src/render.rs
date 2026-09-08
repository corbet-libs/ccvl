use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use ctypst::{CompileRequest, Document, Engine, PageConstraint};
use serde_json::Value;

use crate::application::validate_record;
use crate::opportunity;
use crate::stations;
use crate::styles::{self, Selection, StyleLeaf};
use crate::workspace::Workspace;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentKind {
    Cv,
    CoverLetter,
}

#[derive(Clone, Debug)]
pub struct DocumentSpec {
    pub name: String,
    pub kind: DocumentKind,
    pub source: PathBuf,
    pub output: PathBuf,
    pub inputs: BTreeMap<String, String>,
    pub expected_pages: usize,
    pub selection: Selection,
    pub contract: Value,
}

pub struct Compiler {
    engine: Engine,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OpportunityOptions {
    locale: String,
    pages: usize,
    cover_letter: bool,
}

impl Compiler {
    pub fn new(workspace: &Workspace) -> Result<Self> {
        let additional_fonts = styles::font_paths(workspace)?
            .iter()
            .map(fs::read)
            .collect::<std::io::Result<Vec<_>>>()?;
        let engine = Engine::builder()
            .root(workspace.root())
            .fonts(ctypst::fonts::documents())
            .fonts(additional_fonts)
            .build()
            .context("cannot initialize embedded Typst engine")?;
        Ok(Self { engine })
    }

    pub fn compile(&self, workspace: &Workspace, spec: &DocumentSpec) -> Result<Document> {
        let source = workspace.relative(&workspace.existing_inside(&spec.source)?)?;
        let source = source.to_string_lossy().replace('\\', "/");
        self.engine
            .compile(
                CompileRequest::new(source)
                    .inputs(spec.inputs.clone())
                    .pages(PageConstraint::Exactly(spec.expected_pages)),
            )
            .map(|output| output.document)
            .with_context(|| format!("cannot compile {}", spec.name))
    }

    pub fn render(&self, workspace: &Workspace, spec: &DocumentSpec) -> Result<PathBuf> {
        let document = self.compile(workspace, spec)?;
        self.export(spec, &document)
    }

    /// Export an already compiled document to the spec output. Lets the check
    /// gate measure metrics off the first compilation and export its PDF from
    /// the same document instead of compiling a third time.
    pub fn export(&self, spec: &DocumentSpec, document: &Document) -> Result<PathBuf> {
        let bytes = self
            .engine
            .pdf(document, source_date_epoch()?)
            .with_context(|| format!("cannot export {}", spec.name))?;
        ensure!(
            bytes.starts_with(b"%PDF-"),
            "Typst did not create a PDF for {}",
            spec.name
        );
        if let Some(parent) = spec.output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        fs::write(&spec.output, bytes)
            .with_context(|| format!("cannot write {}", spec.output.display()))?;
        Ok(spec.output.clone())
    }
}

fn source_date_epoch() -> Result<i64> {
    let raw = std::env::var("SOURCE_DATE_EPOCH").unwrap_or_else(|_| "0".to_owned());
    let epoch = raw
        .parse::<i64>()
        .context("SOURCE_DATE_EPOCH must be a non-negative integer")?;
    ensure!(
        epoch >= 0,
        "SOURCE_DATE_EPOCH must be a non-negative integer"
    );
    Ok(epoch)
}

pub use crate::styles::normalize_locale;

pub fn cv_leaf(workspace: &Workspace, locale: &str, selection: &Selection) -> Result<StyleLeaf> {
    styles::leaf(workspace, "cv", locale, selection)
}

pub fn cl_leaf(workspace: &Workspace, locale: &str, selection: &Selection) -> Result<StyleLeaf> {
    styles::leaf(workspace, "cl", locale, selection)
}

pub fn cvl_cv_spec(
    workspace: &Workspace,
    locale: &str,
    pages: usize,
    selection: Option<&Selection>,
) -> Result<DocumentSpec> {
    let selected = selection
        .cloned()
        .map_or_else(|| styles::selection(workspace, "cv", None, None), Ok)?;
    cvl_spec(workspace, &cv_leaf(workspace, locale, &selected)?, pages)
}

pub fn cvl_cl_spec(
    workspace: &Workspace,
    locale: &str,
    pages: Option<usize>,
    selection: Option<&Selection>,
) -> Result<DocumentSpec> {
    let selected = selection
        .cloned()
        .map_or_else(|| styles::selection(workspace, "cl", None, None), Ok)?;
    let leaf = cl_leaf(workspace, locale, &selected)?;
    cvl_spec(workspace, &leaf, pages.unwrap_or(leaf.default_pages))
}

pub fn cvl_spec(workspace: &Workspace, leaf: &StyleLeaf, pages: usize) -> Result<DocumentSpec> {
    document_spec(
        workspace,
        leaf,
        pages,
        &leaf.content(),
        &workspace.path("cvl/profile.toml"),
        &leaf.output(pages),
    )
}

pub fn cv_spec(
    workspace: &Workspace,
    locale: &str,
    pages: usize,
    application: &Path,
    profile: &Path,
    output: &Path,
    selection: &Selection,
) -> Result<DocumentSpec> {
    document_spec(
        workspace,
        &cv_leaf(workspace, locale, selection)?,
        pages,
        application,
        profile,
        output,
    )
}

pub fn cl_spec(
    workspace: &Workspace,
    locale: &str,
    pages: usize,
    application: &Path,
    profile: &Path,
    output: &Path,
    selection: &Selection,
) -> Result<DocumentSpec> {
    document_spec(
        workspace,
        &cl_leaf(workspace, locale, selection)?,
        pages,
        application,
        profile,
        output,
    )
}

fn document_spec(
    workspace: &Workspace,
    leaf: &StyleLeaf,
    pages: usize,
    application: &Path,
    profile: &Path,
    output: &Path,
) -> Result<DocumentSpec> {
    ensure!(
        leaf.pages.contains(&pages),
        "{}/{}: unsupported page count {pages}; expected {:?}",
        leaf.document,
        leaf.style,
        leaf.pages
    );
    let relative = workspace.relative(&workspace.existing_inside(application)?)?;
    let record = workspace.read_toml_value(&relative)?;
    let location = relative.display().to_string();
    validate_record(workspace, &record, &location, true)?;
    let selected = styles::record_selection(workspace, leaf.document, &record, &location)?;
    ensure!(
        selected == leaf.selection(),
        "{location}: selected {}/{} does not match requested {}/{} leaf",
        selected.style,
        selected.substyle,
        leaf.style,
        leaf.substyle
    );
    ensure!(
        record.pointer("/options/language").and_then(Value::as_str) == Some(leaf.locale.as_str()),
        "{location}: language does not match requested {} leaf",
        leaf.locale
    );
    if leaf.document == "cv" && leaf.contract.get("layout_contract").is_some() {
        stations::validate_style(workspace, &leaf.style, true)?;
    }
    let mut inputs = BTreeMap::from([
        ("application".to_owned(), workspace.typst_path(application)?),
        ("profile".to_owned(), workspace.typst_path(profile)?),
        ("locale".to_owned(), leaf.locale.clone()),
        ("pages".to_owned(), pages.to_string()),
        ("strings".to_owned(), workspace.typst_path(&leaf.strings())?),
        (
            "substyle".to_owned(),
            workspace.typst_path(&leaf.substyle_file())?,
        ),
    ]);
    if let Some(path) = &leaf.defaults {
        inputs.insert("shared-defaults".to_owned(), workspace.typst_path(path)?);
    }
    let contract_path = leaf.style_dir().join("contract.toml");
    if contract_path.is_file() {
        inputs.insert("contract".to_owned(), workspace.typst_path(&contract_path)?);
    }
    Ok(DocumentSpec {
        name: format!(
            "{} {}/{}/{} {pages}p",
            leaf.document, leaf.style, leaf.substyle, leaf.locale
        ),
        kind: if leaf.document == "cv" {
            DocumentKind::Cv
        } else {
            DocumentKind::CoverLetter
        },
        source: leaf.adapter(),
        output: output.to_path_buf(),
        inputs,
        expected_pages: pages,
        selection: leaf.selection(),
        contract: leaf.contract.clone(),
    })
}

pub fn cvl_specs(workspace: &Workspace) -> Result<Vec<DocumentSpec>> {
    let mut specs = Vec::new();
    for leaf in styles::leaves(workspace, "cv")?
        .into_iter()
        .chain(styles::leaves(workspace, "cl")?)
    {
        for pages in &leaf.pages {
            specs.push(cvl_spec(workspace, &leaf, *pages)?);
        }
    }
    Ok(specs)
}

pub fn render_cvl(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let compiler = Compiler::new(workspace)?;
    cvl_specs(workspace)?
        .iter()
        .map(|spec| compiler.render(workspace, spec))
        .collect()
}

pub fn list_documents(workspace: &Workspace) -> Result<Value> {
    let mut entries = Vec::new();
    for leaf in styles::leaves(workspace, "cv")?
        .into_iter()
        .chain(styles::leaves(workspace, "cl")?)
    {
        for pages in &leaf.pages {
            entries.push(serde_json::json!({
                "document": leaf.document, "style": leaf.style, "substyle": leaf.substyle,
                "locale": leaf.locale, "pages": pages,
                "content": workspace.relative(&leaf.content())?, "output": workspace.relative(&leaf.output(*pages))?,
                "shared_pages": leaf.contract.get("shared_pages").unwrap_or(&serde_json::json!([])),
                "require_image": leaf.contract.pointer("/pdf/require_image").and_then(Value::as_bool).unwrap_or(false),
            }));
        }
    }
    Ok(Value::Array(entries))
}

pub fn opportunity_specs(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
) -> Result<Vec<DocumentSpec>> {
    let application = opportunity::record_path(workspace, organisation, position, true)?;
    let document = workspace.read_toml_value(workspace.relative(&application)?)?;
    validate_record(
        workspace,
        &document,
        &workspace.relative(&application)?.display().to_string(),
        true,
    )?;
    let options = opportunity_options(&document)?;
    let locale = options.locale;
    let pages = options.pages;
    let cover_enabled = options.cover_letter;
    let relative = workspace.relative(&application)?.display().to_string();
    let cv_selection = styles::record_selection(workspace, "cv", &document, &relative)?;
    let letter_selection = styles::record_selection(workspace, "cl", &document, &relative)?;
    let parent = application
        .parent()
        .context("application record has no parent")?;
    let pdfs = parent.join("pdfs");
    let profile = workspace.path("cvl/profile.toml");
    let mut specs = vec![cv_spec(
        workspace,
        &locale,
        pages,
        &application,
        &profile,
        &pdfs.join("cv.pdf"),
        &cv_selection,
    )?];
    specs[0].name = format!("CV {organisation}/{position}");
    if cover_enabled {
        let letter_pages = document
            .pointer("/options/cl_pages")
            .and_then(Value::as_u64)
            .map(usize::try_from)
            .transpose()?
            .unwrap_or(styles::definition(workspace, "cl", &letter_selection.style)?.default_pages);
        let mut spec = cl_spec(
            workspace,
            &locale,
            letter_pages,
            &application,
            &profile,
            &pdfs.join("cl.pdf"),
            &letter_selection,
        )?;
        spec.name = format!("cover letter {organisation}/{position}");
        specs.push(spec);
    }
    Ok(specs)
}

fn opportunity_options(document: &Value) -> Result<OpportunityOptions> {
    let locale = normalize_locale(
        document
            .pointer("/options/language")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )?;
    let pages = usize::try_from(
        document
            .pointer("/options/pages")
            .and_then(Value::as_u64)
            .context("options.pages is missing")?,
    )?;
    let cover_enabled = document
        .pointer("/options/generate_cl")
        .and_then(Value::as_bool)
        .context("options.generate_cl is missing")?;
    Ok(OpportunityOptions {
        locale,
        pages,
        cover_letter: cover_enabled,
    })
}

/// Locale and substyles selected by one keyed opportunity record, without
/// building its full render specs. Lets the opportunity watcher scope its
/// digest to the record's own leaf templates.
pub struct OpportunitySelection {
    pub locale: String,
    pub cv: Selection,
    pub cl: Selection,
}

pub fn opportunity_selection(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
) -> Result<OpportunitySelection> {
    let application = opportunity::record_path(workspace, organisation, position, true)?;
    let relative = workspace.relative(&application)?.display().to_string();
    let document = workspace.read_toml_value(workspace.relative(&application)?)?;
    let options = opportunity_options(&document)?;
    Ok(OpportunitySelection {
        locale: options.locale,
        cv: styles::record_selection(workspace, "cv", &document, &relative)?,
        cl: styles::record_selection(workspace, "cl", &document, &relative)?,
    })
}

/// Locale selected by one keyed opportunity record, without building its
/// full render specs.
pub fn opportunity_locale(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
) -> Result<String> {
    Ok(opportunity_selection(workspace, organisation, position)?.locale)
}

pub fn render_opportunity(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
) -> Result<Vec<PathBuf>> {
    let specs = opportunity_specs(workspace, organisation, position)?;
    let parent = opportunity::record_path(workspace, organisation, position, true)?
        .parent()
        .context("record has no parent")?
        .to_path_buf();
    remove_stale_cover_letter(
        &parent.join("pdfs"),
        &parent.join("typst"),
        specs
            .iter()
            .any(|spec| spec.kind == DocumentKind::CoverLetter),
    )?;
    let compiler = Compiler::new(workspace)?;
    let mut outputs = Vec::new();
    for spec in &specs {
        outputs.push(compiler.render(workspace, spec)?);
    }
    // Emit the resolved customization copies in typst/ beside the PDFs only
    // after the PDFs render, so the .typ files always describe the PDFs
    // next to them.
    for spec in &specs {
        outputs.push(emit_resolved_typ(workspace, spec, organisation, position)?);
    }
    Ok(outputs)
}

/// Write the resolved customization copy of one rendered opportunity
/// document: the locale template with its `sys.inputs` defaults pointed at
/// this opportunity's record, so the copy beside the PDFs compiles
/// standalone and reproduces the neighbouring PDF.
fn emit_resolved_typ(
    workspace: &Workspace,
    spec: &DocumentSpec,
    organisation: &str,
    position: &str,
) -> Result<PathBuf> {
    let template = fs::read_to_string(&spec.source)
        .with_context(|| format!("cannot read {}", spec.source.display()))?;
    let template_display = workspace.relative(&spec.source)?.display().to_string();
    let text = resolved_typ_text(&template, spec, &template_display, organisation, position);
    let name = match spec.kind {
        DocumentKind::Cv => "cv.typ",
        DocumentKind::CoverLetter => "cl.typ",
    };
    let pdfs_dir = spec
        .output
        .parent()
        .context("opportunity output has no parent")?;
    let parent = pdfs_dir.parent().context("pdfs dir has no parent")?;
    let destination = parent.join("typst").join(name);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
    }
    fs::write(&destination, text)
        .with_context(|| format!("cannot write {}", destination.display()))?;
    Ok(destination)
}

fn resolved_typ_text(
    template: &str,
    spec: &DocumentSpec,
    template_display: &str,
    organisation: &str,
    position: &str,
) -> String {
    let mut resolved = template.to_owned();
    for (key, default) in &spec.inputs {
        resolved = rewrite_input_default(&resolved, key, default);
    }
    let mut provenance = format!("Template: {template_display}");
    for (key, value) in &spec.inputs {
        write!(provenance, "\n// {key}: {value}").expect("writing to a String is infallible");
    }
    format!(
        "// Resolved customization copy emitted by `ccvl build-opportunity {organisation} {position}`.\n\
         // {provenance}\n\
         // Do not edit: this file is regenerated on every build. It compiles standalone and reproduces the neighbouring PDF.\n\
         {resolved}"
    )
}

/// Point one `sys.inputs.at("<key>", default: "<old>")` default at a resolved
/// value so an emitted copy compiles without `--input` flags. Leaves the
/// source untouched when the template no longer carries that input.
fn rewrite_input_default(source: &str, key: &str, default: &str) -> String {
    let marker = format!("sys.inputs.at(\"{key}\", default: \"");
    let Some(start) = source.find(&marker) else {
        return source.to_owned();
    };
    let value_start = start + marker.len();
    let Some(value_end) = source[value_start..].find('"') else {
        return source.to_owned();
    };
    let mut resolved = String::with_capacity(source.len());
    resolved.push_str(&source[..value_start]);
    resolved.push_str(default);
    resolved.push_str(&source[value_start + value_end..]);
    resolved
}

fn remove_stale_cover_letter(pdfs_dir: &Path, typst_dir: &Path, cover_enabled: bool) -> Result<()> {
    if !cover_enabled {
        for stale in [pdfs_dir.join("cl.pdf"), typst_dir.join("cl.typ")] {
            if stale.is_file() {
                fs::remove_file(stale)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
