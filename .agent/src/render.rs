use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use ctypst::{CompileRequest, Document, Engine, PageConstraint};
use serde_json::Value;

use crate::application::{
    StyleLeaf, cl_leaves, cv_leaves, default_cl_substyle, default_cv_substyle, resolve_cl_substyle,
    resolve_cv_substyle, validate_record,
};
use crate::opportunity;
use crate::stations;
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
}

pub struct Compiler {
    engine: Engine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OpportunityOptions {
    locale: &'static str,
    pages: usize,
    cover_letter: bool,
}

impl Compiler {
    pub fn new(workspace: &Workspace) -> Result<Self> {
        let engine = Engine::builder()
            .root(workspace.root())
            .fonts(ctypst::fonts::documents())
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

pub fn normalize_locale(value: &str) -> Result<&'static str> {
    match value.to_ascii_lowercase().as_str() {
        "de" | "de-ch" => Ok("de-ch"),
        "en" | "en-ch" => Ok("en-ch"),
        _ => bail!("unsupported locale: {value}"),
    }
}

pub fn cvl_cv_spec(
    workspace: &Workspace,
    locale_value: &str,
    pages: usize,
    substyle: Option<&str>,
) -> Result<DocumentSpec> {
    stations::validate_interview(workspace, true)?;
    let locale = normalize_locale(locale_value)?;
    let name = match substyle {
        Some(name) => name.to_owned(),
        None => default_cv_substyle(workspace)?,
    };
    let leaf = cv_leaf(workspace, locale, &name)?;
    let application = leaf.content();
    let profile = workspace.path("cvl/profile.toml");
    let relative = workspace.relative(&application)?.display().to_string();
    let document = workspace.read_toml_value(&workspace.relative(&application)?)?;
    // The leaf's own selection is normative: an explicit substyle must name
    // this leaf, so a mismatched flag fails here instead of rendering the
    // wrong look.
    let effective = resolve_cv_substyle(workspace, &document, &relative)?;
    ensure!(
        effective == leaf.substyle,
        "{relative}: options.cv_substyle {effective:?} does not match this {} leaf",
        leaf.substyle
    );
    cv_spec(
        workspace,
        locale,
        pages,
        &application,
        &profile,
        &leaf.dir.join("pdf").join(format!("cv-{pages}.pdf")),
        &leaf.substyle,
    )
}

pub fn cvl_cl_spec(
    workspace: &Workspace,
    locale_value: &str,
    substyle: Option<&str>,
) -> Result<DocumentSpec> {
    let locale = normalize_locale(locale_value)?;
    let name = match substyle {
        Some(name) => name.to_owned(),
        None => default_cl_substyle(workspace)?,
    };
    let leaf = cl_leaf(workspace, locale, &name)?;
    let application = leaf.content();
    let profile = workspace.path("cvl/profile.toml");
    let relative = workspace.relative(&application)?.display().to_string();
    let document = workspace.read_toml_value(&workspace.relative(&application)?)?;
    let effective = resolve_cl_substyle(workspace, &document, &relative)?;
    ensure!(
        effective == leaf.substyle,
        "{relative}: options.cl_substyle {effective:?} does not match this {} leaf",
        leaf.substyle
    );
    cl_spec(
        workspace,
        locale,
        &application,
        &profile,
        &leaf.dir.join("pdf").join("cl.pdf"),
        &leaf.substyle,
    )
}

/// Locate one CV leaf by record locale and substyle name. Unknown names fail
/// here — before the Typst compile — with the available substyles.
pub fn cv_leaf(workspace: &Workspace, locale: &str, substyle: &str) -> Result<StyleLeaf> {
    cv_leaves(workspace)?
        .into_iter()
        .find(|leaf| leaf.locale == locale && leaf.substyle == substyle)
        .with_context(|| format!("no CV leaf for locale {locale} substyle {substyle:?}"))
}

/// Locate one cover-letter leaf by record locale and substyle name.
pub fn cl_leaf(workspace: &Workspace, locale: &str, substyle: &str) -> Result<StyleLeaf> {
    cl_leaves(workspace)?
        .into_iter()
        .find(|leaf| leaf.locale == locale && leaf.substyle == substyle)
        .with_context(|| format!("no cover-letter leaf for locale {locale} substyle {substyle:?}"))
}

fn shared_defaults(workspace: &Workspace) -> Result<String> {
    workspace.typst_path(&workspace.path("cvl/shared/defaults.toml"))
}

pub fn cv_spec(
    workspace: &Workspace,
    locale: &str,
    pages: usize,
    application: &Path,
    profile: &Path,
    output: &Path,
    cv_substyle: &str,
) -> Result<DocumentSpec> {
    ensure!(
        (2..=4).contains(&pages),
        "CV pages must be 2, 3, or 4: {pages}"
    );
    let leaf = cv_leaf(workspace, locale, cv_substyle)?;
    // The record's own selection must agree with the requested leaf, so an
    // opportunity record always renders its selected look.
    let relative = workspace.relative(&workspace.existing_inside(application)?)?;
    let document = workspace.read_toml_value(&relative)?;
    let effective = resolve_cv_substyle(workspace, &document, &relative.display().to_string())?;
    ensure!(
        effective == leaf.substyle,
        "{}: options.cv_substyle {effective:?} does not match the requested {} leaf",
        relative.display(),
        leaf.substyle
    );
    Ok(DocumentSpec {
        name: format!("CV {locale}/{} {pages}p", leaf.substyle),
        kind: DocumentKind::Cv,
        source: leaf.adapter(),
        output: output.to_path_buf(),
        inputs: BTreeMap::from([
            ("application".to_owned(), workspace.typst_path(application)?),
            ("cv-pages".to_owned(), pages.to_string()),
            ("profile".to_owned(), workspace.typst_path(profile)?),
            ("strings".to_owned(), workspace.typst_path(&leaf.strings())?),
            (
                "substyle".to_owned(),
                workspace.typst_path(&leaf.substyle_file())?,
            ),
            ("shared-defaults".to_owned(), shared_defaults(workspace)?),
        ]),
        expected_pages: pages,
    })
}

pub fn cl_spec(
    workspace: &Workspace,
    locale: &str,
    application: &Path,
    profile: &Path,
    output: &Path,
    cl_substyle: &str,
) -> Result<DocumentSpec> {
    let leaf = cl_leaf(workspace, locale, cl_substyle)?;
    let relative = workspace.relative(&workspace.existing_inside(application)?)?;
    let document = workspace.read_toml_value(&relative)?;
    let effective = resolve_cl_substyle(workspace, &document, &relative.display().to_string())?;
    ensure!(
        effective == leaf.substyle,
        "{}: options.cl_substyle {effective:?} does not match the requested {} leaf",
        relative.display(),
        leaf.substyle
    );
    Ok(DocumentSpec {
        name: format!("cover letter {locale}/{}", leaf.substyle),
        kind: DocumentKind::CoverLetter,
        source: leaf.adapter(),
        output: output.to_path_buf(),
        inputs: BTreeMap::from([
            ("application".to_owned(), workspace.typst_path(application)?),
            ("profile".to_owned(), workspace.typst_path(profile)?),
            ("strings".to_owned(), workspace.typst_path(&leaf.strings())?),
            (
                "substyle".to_owned(),
                workspace.typst_path(&leaf.substyle_file())?,
            ),
            ("shared-defaults".to_owned(), shared_defaults(workspace)?),
        ]),
        expected_pages: 1,
    })
}

pub fn render_cvl(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let compiler = Compiler::new(workspace)?;
    let mut outputs = Vec::new();
    for leaf in cv_leaves(workspace)? {
        for pages in [2, 3, 4] {
            outputs.push(compiler.render(
                workspace,
                &cvl_cv_spec(workspace, leaf.locale, pages, Some(&leaf.substyle))?,
            )?);
        }
    }
    for leaf in cl_leaves(workspace)? {
        outputs.push(compiler.render(
            workspace,
            &cvl_cl_spec(workspace, leaf.locale, Some(&leaf.substyle))?,
        )?);
    }
    Ok(outputs)
}

pub fn opportunity_specs(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
) -> Result<Vec<DocumentSpec>> {
    stations::validate_interview(workspace, true)?;
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
    let cv_substyle = resolve_cv_substyle(workspace, &document, &relative)?;
    let cl_substyle = resolve_cl_substyle(workspace, &document, &relative)?;
    let parent = application
        .parent()
        .context("application record has no parent")?;
    let pdfs = parent.join("pdfs");
    let profile = workspace.path("cvl/profile.toml");
    let mut specs = vec![cv_spec(
        workspace,
        locale,
        pages,
        &application,
        &profile,
        &pdfs.join("cv.pdf"),
        &cv_substyle,
    )?];
    specs[0].name = format!("CV {organisation}/{position}");
    if cover_enabled {
        let mut spec = cl_spec(
            workspace,
            locale,
            &application,
            &profile,
            &pdfs.join("cl.pdf"),
            &cl_substyle,
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
    pub locale: &'static str,
    pub cv_substyle: String,
    pub cl_substyle: String,
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
        cv_substyle: resolve_cv_substyle(workspace, &document, &relative)?,
        cl_substyle: resolve_cl_substyle(workspace, &document, &relative)?,
    })
}

/// Locale selected by one keyed opportunity record, without building its
/// full render specs.
pub fn opportunity_locale(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
) -> Result<&'static str> {
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
    for key in [
        "application",
        "profile",
        "cv-pages",
        "strings",
        "substyle",
        "shared-defaults",
    ] {
        if let Some(default) = spec.inputs.get(key) {
            resolved = rewrite_input_default(&resolved, key, default);
        }
    }
    let mut provenance = format!("Template: {template_display}");
    for key in [
        "application",
        "profile",
        "cv-pages",
        "strings",
        "substyle",
        "shared-defaults",
    ] {
        if let Some(default) = spec.inputs.get(key) {
            use std::fmt::Write as _;
            let _ = write!(provenance, " | {key}: {default}");
        }
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
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn cvl_outputs_use_numeric_page_names() {
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        for (locale, lang) in [("de-ch", "de"), ("en-ch", "en")] {
            for substyle in ["standard", "compact"] {
                for pages in [2, 3, 4] {
                    let spec = cvl_cv_spec(&workspace, locale, pages, Some(substyle)).unwrap();
                    assert_eq!(
                        spec.output,
                        workspace.path(format!("cvl/cv/{substyle}/{lang}/ch/pdf/cv-{pages}.pdf"))
                    );
                    assert_eq!(
                        spec.source,
                        workspace.path(format!("cvl/cv/{substyle}/{lang}/ch/typst/cv.typ"))
                    );
                }
            }
            for substyle in ["left-rule", "frame"] {
                let spec = cvl_cl_spec(&workspace, locale, Some(substyle)).unwrap();
                assert_eq!(
                    spec.output,
                    workspace.path(format!("cvl/cl/{substyle}/{lang}/ch/pdf/cl.pdf"))
                );
            }
        }
        assert!(cvl_cv_spec(&workspace, "en-ch", 1, None).is_err());
        assert!(cvl_cv_spec(&workspace, "en-ch", 5, None).is_err());
        assert!(cvl_cv_spec(&workspace, "en-ch", 4, Some("nope")).is_err());
        assert!(cvl_cl_spec(&workspace, "en-ch", Some("nope")).is_err());
    }

    #[test]
    fn opportunity_record_selects_its_locale_pages_and_documents() {
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let mut document = workspace
            .read_toml_value("cvl/cv/standard/en/ch/content.toml")
            .unwrap();
        document["options"]["language"] = "en-ch".into();
        document["options"]["pages"] = 3.into();
        document["options"]["generate_cl"] = false.into();
        document.as_object_mut().unwrap().remove("cl");
        assert_eq!(
            opportunity_options(&document).unwrap(),
            OpportunityOptions {
                locale: "en-ch",
                pages: 3,
                cover_letter: false,
            }
        );
    }

    #[test]
    fn disabled_cover_letter_removes_stale_output() {
        let directory = tempdir().unwrap();
        let pdfs = directory.path().join("pdfs");
        let typst = directory.path().join("typst");
        fs::create_dir_all(&pdfs).unwrap();
        fs::create_dir_all(&typst).unwrap();
        let stale_pdf = pdfs.join("cl.pdf");
        let stale_typ = typst.join("cl.typ");
        fs::write(&stale_pdf, b"stale").unwrap();
        fs::write(&stale_typ, b"stale").unwrap();
        remove_stale_cover_letter(&pdfs, &typst, false).unwrap();
        assert!(!stale_pdf.exists());
        assert!(!stale_typ.exists());

        fs::write(&stale_pdf, b"current").unwrap();
        fs::write(&stale_typ, b"current").unwrap();
        remove_stale_cover_letter(&pdfs, &typst, true).unwrap();
        assert!(stale_pdf.exists());
        assert!(stale_typ.exists());
    }

    #[test]
    fn input_default_rewrite_points_copy_at_record() {
        let cv = "#let cv-pages = int(sys.inputs.at(\"cv-pages\", default: \"4\"))\n\
                  #let application-path = sys.inputs.at(\"application\", default: \"/cvl/cv/standard/en/ch/content.toml\")\n";
        let resolved = rewrite_input_default(
            cv,
            "application",
            "/opportunities/acme/lead/application.toml",
        );
        let resolved = rewrite_input_default(&resolved, "cv-pages", "3");
        assert!(resolved.contains(
            "sys.inputs.at(\"application\", default: \"/opportunities/acme/lead/application.toml\")"
        ));
        assert!(resolved.contains("sys.inputs.at(\"cv-pages\", default: \"3\")"));
        assert!(!resolved.contains("/cvl/cv/standard/en/ch/content.toml"));

        let cl = "#let substyle-path = sys.inputs.at(\"substyle\", default: \"/cvl/cl/left-rule/de/ch/substyle.toml\")\n";
        let resolved = rewrite_input_default(cl, "substyle", "/cvl/cl/frame/de/ch/substyle.toml");
        assert!(resolved.contains(
            "sys.inputs.at(\"substyle\", default: \"/cvl/cl/frame/de/ch/substyle.toml\")"
        ));
        let shared = "#let shared-defaults-path = sys.inputs.at(\"shared-defaults\", default: \"/cvl/shared/defaults.toml\")\n";
        assert_eq!(
            rewrite_input_default(shared, "shared-defaults", "/cvl/shared/defaults.toml"),
            shared
        );
        // A template that no longer carries the input survives unchanged.
        assert_eq!(
            rewrite_input_default("#let x = 1\n", "application", "/elsewhere.toml"),
            "#let x = 1\n"
        );
    }

    #[test]
    fn resolved_copy_carries_provenance_and_record_defaults() {
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let leaf = cv_leaf(&workspace, "en-ch", "standard").unwrap();
        let output = tempdir().unwrap().path().join("pdfs").join("cv.pdf");
        let mut spec = cv_spec(
            &workspace,
            "en-ch",
            3,
            &leaf.content(),
            &workspace.path("cvl/profile.toml"),
            &output,
            "standard",
        )
        .unwrap();
        spec.inputs.insert(
            "application".to_owned(),
            "/opportunities/acme/lead/application.toml".to_owned(),
        );
        let template = fs::read_to_string(&leaf.adapter()).unwrap();
        let text = resolved_typ_text(
            &template,
            &spec,
            "cvl/cv/standard/en/ch/typst/cv.typ",
            "acme",
            "lead",
        );
        assert!(text.starts_with(
            "// Resolved customization copy emitted by `ccvl build-opportunity acme lead`."
        ));
        assert!(text.contains("Template: cvl/cv/standard/en/ch/typst/cv.typ"));
        assert!(text.contains("application: /opportunities/acme/lead/application.toml"));
        assert!(text.contains("cv-pages: 3"));
        assert!(text.contains("| strings: "));
        assert!(text.contains("| substyle: "));
        assert!(text.contains("| shared-defaults: "));
        assert!(!text.contains("| style: "));
        assert!(text.ends_with('\n'));
        assert!(!text.contains("default: \"/cvl/cv/standard/en/ch/content.toml\""));
        assert!(text.contains("sys.inputs.at(\"cv-pages\", default: \"3\")"));
    }

    #[test]
    fn emitted_copies_compile_without_cli_inputs() {
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let compiler = Compiler::new(&workspace).unwrap();
        for locale in ["de-ch", "en-ch"] {
            let mut specs = Vec::new();
            for substyle in ["standard", "compact"] {
                specs.push(cvl_cv_spec(&workspace, locale, 3, Some(substyle)).unwrap());
            }
            for substyle in ["left-rule", "frame"] {
                specs.push(cvl_cl_spec(&workspace, locale, Some(substyle)).unwrap());
            }
            for spec in specs {
                let original = compiler.compile(&workspace, &spec).unwrap();
                let template = fs::read_to_string(&spec.source).unwrap();
                let text = resolved_typ_text(&template, &spec, "fixture", "acme", "lead");
                let standalone = compiler
                    .engine
                    .compile(
                        CompileRequest::new("standalone.typ")
                            .source_file("standalone.typ", text)
                            .pages(PageConstraint::Exactly(spec.expected_pages)),
                    )
                    .unwrap();
                assert_eq!(
                    compiler.engine.pdf(&original, 0).unwrap(),
                    compiler.engine.pdf(&standalone.document, 0).unwrap(),
                    "standalone copy differs: {}",
                    spec.name,
                );
            }
        }
    }

    #[test]
    fn cvl_specs_carry_their_leaf_paths() {
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        for pages in [2, 3, 4] {
            let spec = cvl_cv_spec(&workspace, "en-ch", pages, Some("compact")).unwrap();
            assert_eq!(
                spec.inputs.get("strings").map(String::as_str),
                Some("/cvl/cv/compact/en/ch/strings.toml")
            );
            assert_eq!(
                spec.inputs.get("substyle").map(String::as_str),
                Some("/cvl/cv/compact/substyle.toml")
            );
            assert_eq!(
                spec.inputs.get("shared-defaults").map(String::as_str),
                Some("/cvl/shared/defaults.toml")
            );
            assert!(spec.inputs.get("style").is_none());
        }
        // No explicit substyle renders the family default.
        let spec = cvl_cv_spec(&workspace, "en-ch", 4, None).unwrap();
        assert_eq!(
            spec.inputs.get("substyle").map(String::as_str),
            Some("/cvl/cv/standard/substyle.toml")
        );
        let spec = cvl_cl_spec(&workspace, "de-ch", None).unwrap();
        assert_eq!(
            spec.inputs.get("substyle").map(String::as_str),
            Some("/cvl/cl/left-rule/substyle.toml")
        );
        let spec = cvl_cl_spec(&workspace, "de-ch", Some("frame")).unwrap();
        assert_eq!(
            spec.inputs.get("strings").map(String::as_str),
            Some("/cvl/cl/frame/de/ch/strings.toml")
        );
    }

    #[test]
    fn mismatched_record_selection_fails_before_compiling() {
        // A record selecting compact must not render through the standard
        // leaf: the mismatch fails in Rust with both names.
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let leaf = cv_leaf(&workspace, "en-ch", "standard").unwrap();
        let directory = tempfile::tempdir_in(workspace.root()).unwrap();
        let record = directory.path().join("application.toml");
        let text = fs::read_to_string(&leaf.content())
            .unwrap()
            .replace("cv_substyle = \"standard\"", "cv_substyle = \"compact\"");
        fs::write(&record, text).unwrap();
        let output = directory.path().join("cv.pdf");
        let error = cv_spec(
            &workspace,
            "en-ch",
            2,
            &record,
            &workspace.path("cvl/profile.toml"),
            &output,
            "standard",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("compact"), "unexpected error: {error}");
        assert!(error.contains("standard"), "unexpected error: {error}");
    }

    #[test]
    fn style_major_leaves_compile() {
        // The default leaves render end to end through the shared renderers
        // under exact page constraints.
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let compiler = Compiler::new(&workspace).unwrap();
        let cv = cvl_cv_spec(&workspace, "de-ch", 2, None).unwrap();
        compiler.compile(&workspace, &cv).unwrap();
        let cl = cvl_cl_spec(&workspace, "de-ch", None).unwrap();
        compiler.compile(&workspace, &cl).unwrap();
    }

    #[test]
    fn compact_substyles_pass_the_same_measurement_gates() {
        use crate::measure::{document_metrics, line_failure, summary_failures};

        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let compiler = Compiler::new(&workspace).unwrap();
        // Two-page CV exercises pages 1-2; the cover letter exercises the
        // vertical-rhythm gate. Both compile under an exact page constraint.
        let mut cv = cvl_cv_spec(&workspace, "en-ch", 2, Some("compact")).unwrap();
        cv.inputs
            .insert("line-contracts".to_owned(), "report".to_owned());
        let document = compiler.compile(&workspace, &cv).unwrap();
        let metrics = document_metrics(&workspace, &cv, &document).unwrap();
        assert!(
            summary_failures(&workspace, &cv, &metrics)
                .unwrap()
                .is_empty()
        );
        for (index, metric) in metrics.iter().enumerate() {
            assert!(
                line_failure(&cv, index, metric).unwrap().is_none(),
                "compact CV failure at #{index}: {metric}"
            );
        }

        for substyle in ["left-rule", "frame"] {
            let mut cl = cvl_cl_spec(&workspace, "en-ch", Some(substyle)).unwrap();
            cl.inputs
                .insert("line-contracts".to_owned(), "report".to_owned());
            let document = compiler.compile(&workspace, &cl).unwrap();
            let metrics = document_metrics(&workspace, &cl, &document).unwrap();
            for (index, metric) in metrics.iter().enumerate() {
                assert!(
                    line_failure(&cl, index, metric).unwrap().is_none(),
                    "{substyle} cover-letter failure at #{index}: {metric}"
                );
            }
        }
    }

    #[test]
    fn compact_delta_keeps_horizontal_measure_and_accents() {
        // Horizontal measure feeds every fill percentage, so the compact
        // delta may only tighten vertical whitespace. Pin the invariant
        // between the shared base knobs and the delta; the gates above prove
        // the result still passes.
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let base = workspace
            .read_toml_value("cvl/shared/defaults.toml")
            .unwrap();
        let compact = workspace
            .read_toml_value("cvl/cv/compact/substyle.toml")
            .unwrap();
        let delta = compact.as_object().unwrap();
        for forbidden in ["page", "text", "accents"] {
            assert!(
                !delta.contains_key(forbidden),
                "horizontal section changed by the delta: {forbidden}"
            );
        }
        let cv = delta.get("cv").and_then(|value| value.as_object());
        for forbidden in ["bullet_indent_pt"] {
            assert!(
                cv.is_none_or(|table| !table.contains_key(forbidden)),
                "horizontal knob changed by the delta: {forbidden}"
            );
        }
        let fill = |style: &serde_json::Value, pointer: &str| {
            style.pointer(pointer).and_then(serde_json::Value::as_f64)
        };
        assert!(
            fill(&compact, "/cv/entry_spacing_pt") < fill(&base, "/cv/entry_spacing_pt"),
            "compact must tighten vertical whitespace"
        );
        assert!(!delta.is_empty());
    }
}
