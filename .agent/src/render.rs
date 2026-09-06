use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use ctypst::{CompileRequest, Document, Engine, PageConstraint};
use serde_json::Value;

use crate::application::{resolve_style, validate_record};
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
) -> Result<DocumentSpec> {
    stations::validate_interview(workspace, true)?;
    let locale = normalize_locale(locale_value)?;
    let application = workspace.path(format!("cvl/{locale}/application.toml"));
    let profile = workspace.path("cvl/profile.toml");
    cv_spec(
        workspace,
        locale,
        pages,
        &application,
        &profile,
        &workspace.path(format!("cvl/{locale}/output/cv-{pages}.pdf")),
    )
}

pub fn cvl_cl_spec(workspace: &Workspace, locale_value: &str) -> Result<DocumentSpec> {
    let locale = normalize_locale(locale_value)?;
    let application = workspace.path(format!("cvl/{locale}/application.toml"));
    let profile = workspace.path("cvl/profile.toml");
    cl_spec(
        workspace,
        locale,
        &application,
        &profile,
        &workspace.path(format!("cvl/{locale}/output/cl.pdf")),
    )
}

pub fn cv_spec(
    workspace: &Workspace,
    locale: &str,
    pages: usize,
    application: &Path,
    profile: &Path,
    output: &Path,
) -> Result<DocumentSpec> {
    ensure!(
        (2..=4).contains(&pages),
        "CV pages must be 2, 3, or 4: {pages}"
    );
    Ok(DocumentSpec {
        name: format!("CV {locale}"),
        kind: DocumentKind::Cv,
        source: workspace.path(format!("cvl/{locale}/cv.typ")),
        output: output.to_path_buf(),
        inputs: BTreeMap::from([
            ("application".to_owned(), workspace.typst_path(application)?),
            ("cv-pages".to_owned(), pages.to_string()),
            ("profile".to_owned(), workspace.typst_path(profile)?),
            ("style".to_owned(), record_style(workspace, application)?),
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
) -> Result<DocumentSpec> {
    Ok(DocumentSpec {
        name: format!("cover letter {locale}"),
        kind: DocumentKind::CoverLetter,
        source: workspace.path(format!("cvl/{locale}/cl.typ")),
        output: output.to_path_buf(),
        inputs: BTreeMap::from([
            ("application".to_owned(), workspace.typst_path(application)?),
            ("profile".to_owned(), workspace.typst_path(profile)?),
            ("style".to_owned(), record_style(workspace, application)?),
        ]),
        expected_pages: 1,
    })
}

/// Resolve the render style for the record at `application` and surface it
/// as the `style` Typst input. Unknown names fail here — before the Typst
/// compile — with the available styles; records without `options.style`
/// render with the manifest default (`harvard`).
fn record_style(workspace: &Workspace, application: &Path) -> Result<String> {
    let relative = workspace.relative(&workspace.existing_inside(application)?)?;
    let document = workspace.read_toml_value(&relative)?;
    resolve_style(workspace, &document, &relative.display().to_string())
}

pub fn render_cvl(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let compiler = Compiler::new(workspace)?;
    let mut outputs = Vec::new();
    for locale in ["de-ch", "en-ch"] {
        for pages in [2, 3, 4] {
            outputs.push(compiler.render(workspace, &cvl_cv_spec(workspace, locale, pages)?)?);
        }
        outputs.push(compiler.render(workspace, &cvl_cl_spec(workspace, locale)?)?);
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
    let output = application
        .parent()
        .context("application record has no parent")?
        .join("output");
    let profile = workspace.path("cvl/profile.toml");
    let mut specs = vec![cv_spec(
        workspace,
        locale,
        pages,
        &application,
        &profile,
        &output.join("cv.pdf"),
    )?];
    specs[0].name = format!("CV {organisation}/{position}");
    if cover_enabled {
        let mut spec = cl_spec(
            workspace,
            locale,
            &application,
            &profile,
            &output.join("cl.pdf"),
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

/// Locale selected by one keyed opportunity record, without building its
/// full render specs. Lets the opportunity watcher scope its digest to the
/// record's own locale templates.
pub fn opportunity_locale(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
) -> Result<&'static str> {
    let application = opportunity::record_path(workspace, organisation, position, true)?;
    let document = workspace.read_toml_value(workspace.relative(&application)?)?;
    Ok(opportunity_options(&document)?.locale)
}

pub fn render_opportunity(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
) -> Result<Vec<PathBuf>> {
    let specs = opportunity_specs(workspace, organisation, position)?;
    let output_dir = opportunity::record_path(workspace, organisation, position, true)?
        .parent()
        .context("record has no parent")?
        .join("output");
    remove_stale_cover_letter(
        &output_dir,
        specs
            .iter()
            .any(|spec| spec.kind == DocumentKind::CoverLetter),
    )?;
    let compiler = Compiler::new(workspace)?;
    let mut outputs = Vec::new();
    for spec in &specs {
        outputs.push(compiler.render(workspace, spec)?);
    }
    // Emit the resolved customization copies beside the PDFs only after the
    // PDFs render, so the .typ files always describe the PDFs next to them.
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
    let destination = spec
        .output
        .parent()
        .context("opportunity output has no parent")?
        .join(name);
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
    for key in ["application", "profile", "cv-pages", "style"] {
        if let Some(default) = spec.inputs.get(key) {
            resolved = rewrite_input_default(&resolved, key, default);
        }
    }
    let mut provenance = format!("Template: {template_display}");
    for key in ["application", "profile", "cv-pages", "style"] {
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

fn remove_stale_cover_letter(output_dir: &Path, cover_enabled: bool) -> Result<()> {
    if !cover_enabled {
        for stale in ["cl.pdf", "cl.typ"] {
            let stale = output_dir.join(stale);
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
        for pages in [2, 3, 4] {
            let spec = cvl_cv_spec(&workspace, "en-ch", pages).unwrap();
            assert_eq!(
                spec.output,
                workspace.path(format!("cvl/en-ch/output/cv-{pages}.pdf"))
            );
        }
        assert!(cvl_cv_spec(&workspace, "en-ch", 1).is_err());
        assert!(cvl_cv_spec(&workspace, "en-ch", 5).is_err());
    }

    #[test]
    fn opportunity_record_selects_its_locale_pages_and_documents() {
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let mut document = workspace
            .read_toml_value("cvl/en-ch/application.toml")
            .unwrap();
        document["options"]["language"] = "en-CH".into();
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
        let stale_pdf = directory.path().join("cl.pdf");
        let stale_typ = directory.path().join("cl.typ");
        fs::write(&stale_pdf, b"stale").unwrap();
        fs::write(&stale_typ, b"stale").unwrap();
        remove_stale_cover_letter(directory.path(), false).unwrap();
        assert!(!stale_pdf.exists());
        assert!(!stale_typ.exists());

        fs::write(&stale_pdf, b"current").unwrap();
        fs::write(&stale_typ, b"current").unwrap();
        remove_stale_cover_letter(directory.path(), true).unwrap();
        assert!(stale_pdf.exists());
        assert!(stale_typ.exists());
    }

    #[test]
    fn input_default_rewrite_points_copy_at_record() {
        let cv = "#let cv-pages = int(sys.inputs.at(\"cv-pages\", default: \"4\"))\n\
                  #let application-path = sys.inputs.at(\"application\", default: \"/cvl/en-ch/application.toml\")\n";
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
        assert!(!resolved.contains("/cvl/en-ch/application.toml"));

        let cl = "#let application-path = sys.inputs.at(\"application\", default: \"/cvl/de-ch/application.toml\")\n";
        let resolved = rewrite_input_default(
            cl,
            "application",
            "/opportunities/acme/lead/application.toml",
        );
        assert!(resolved.contains(
            "sys.inputs.at(\"application\", default: \"/opportunities/acme/lead/application.toml\")"
        ));
        let styled = "#let style-input = sys.inputs.at(\"style\", default: \"\")\n";
        assert_eq!(
            rewrite_input_default(styled, "style", "harvard"),
            "#let style-input = sys.inputs.at(\"style\", default: \"harvard\")\n"
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
        let source = workspace.path("cvl/en-ch/cv.typ");
        let output = tempdir().unwrap().path().join("output").join("cv.pdf");
        let mut spec = cv_spec(
            &workspace,
            "en-ch",
            3,
            &workspace.path("cvl/en-ch/application.toml"),
            &workspace.path("cvl/profile.toml"),
            &output,
        )
        .unwrap();
        spec.inputs.insert(
            "application".to_owned(),
            "/opportunities/acme/lead/application.toml".to_owned(),
        );
        let template = fs::read_to_string(&source).unwrap();
        let text = resolved_typ_text(&template, &spec, "cvl/en-ch/cv.typ", "acme", "lead");
        assert!(text.starts_with(
            "// Resolved customization copy emitted by `ccvl build-opportunity acme lead`."
        ));
        assert!(text.contains("Template: cvl/en-ch/cv.typ"));
        assert!(text.contains("application: /opportunities/acme/lead/application.toml"));
        assert!(text.contains("cv-pages: 3"));
        assert!(text.contains("| style: "));
        assert!(!text.contains("sys.inputs.at(\"style\", default: \"\")"));
        assert!(text.ends_with('\n'));
        assert!(!text.contains("default: \"/cvl/en-ch/application.toml\""));
        assert!(text.contains("sys.inputs.at(\"cv-pages\", default: \"3\")"));
    }

    #[test]
    fn emitted_copy_compiles_inputs_standalone() {
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let directory = tempdir().unwrap();
        let output = directory.path().join("output").join("cv.pdf");
        let spec = cv_spec(
            &workspace,
            "en-ch",
            3,
            &workspace.path("cvl/en-ch/application.toml"),
            &workspace.path("cvl/profile.toml"),
            &output,
        )
        .unwrap();
        let typ = emit_resolved_typ(&workspace, &spec, "acme", "lead").unwrap();
        assert_eq!(typ, directory.path().join("output").join("cv.typ"));
        let text = fs::read_to_string(&typ).unwrap();
        assert!(text.contains("| application: /cvl/en-ch/application.toml |"));
        assert!(
            !text
                .lines()
                .any(|line| line.ends_with(' ') || line.ends_with('\t'))
        );
    }

    #[test]
    fn cvl_specs_carry_the_resolved_default_style() {
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        for pages in [2, 3, 4] {
            let spec = cvl_cv_spec(&workspace, "en-ch", pages).unwrap();
            assert_eq!(
                spec.inputs.get("style").map(String::as_str),
                Some("harvard")
            );
        }
        let spec = cvl_cl_spec(&workspace, "de-ch").unwrap();
        assert_eq!(
            spec.inputs.get("style").map(String::as_str),
            Some("harvard")
        );
    }

    #[test]
    fn back_compat_shim_renders_the_harvard_default() {
        // document.typ only re-exports harvard.typ, and harvard-compact.typ
        // wraps the same renderer with compact defaults. A document going
        // through either — defaulted or with an explicit named style — must
        // compile to one page. Note: `style` travels by name everywhere;
        // positional arguments cannot reach past a defaulted parameter.
        let engine = ctypst::Engine::builder()
            .root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .fonts(ctypst::fonts::documents())
            .build()
            .unwrap();
        for (module, locale, style) in [
            ("document", "de-ch", "harvard"),
            ("harvard-compact", "en-ch", "harvard-compact"),
        ] {
            for with in [
                format!("locale: \"{locale}\""),
                format!("locale: \"{locale}\", style: load-style(\"{style}\")"),
            ] {
                let source = format!(
                    "#import \"/.agent/typst/styles/{module}.typ\": document-style, load-style\n\
                     #show: document-style.with({with})\n\
                     Hello, Harvard.\n"
                );
                engine
                    .compile(
                        ctypst::CompileRequest::new("shim.typ")
                            .source_file("shim.typ", source)
                            .pages(ctypst::PageConstraint::Exactly(1)),
                    )
                    .unwrap();
            }
        }
    }

    #[test]
    fn unknown_style_fails_with_a_clear_error() {
        let engine = ctypst::Engine::builder()
            .root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .fonts(ctypst::fonts::documents())
            .build()
            .unwrap();
        let source =
            "#import \"/.agent/typst/styles/harvard.typ\": load-style\n#load-style(\"nope\")\n";
        let error = engine
            .compile(
                ctypst::CompileRequest::new("unknown-style.typ")
                    .source_file("unknown-style.typ", source),
            )
            .err()
            .expect("unknown style must fail")
            .to_string();
        assert!(error.contains("unknown style"), "unexpected error: {error}");
        assert!(
            error.contains("harvard-compact"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn harvard_compact_passes_the_same_measurement_gates() {
        use crate::measure::{document_metrics, line_failure, summary_failures};

        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let compiler = Compiler::new(&workspace).unwrap();
        // Two-page CV exercises pages 1-2; the cover letter exercises the
        // vertical-rhythm gate. Both compile under an exact page constraint.
        let mut cv = cvl_cv_spec(&workspace, "en-ch", 2).unwrap();
        cv.inputs
            .insert("line-contracts".to_owned(), "report".to_owned());
        cv.inputs
            .insert("style".to_owned(), "harvard-compact".to_owned());
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

        let mut cl = cvl_cl_spec(&workspace, "en-ch").unwrap();
        cl.inputs
            .insert("line-contracts".to_owned(), "report".to_owned());
        cl.inputs
            .insert("style".to_owned(), "harvard-compact".to_owned());
        let document = compiler.compile(&workspace, &cl).unwrap();
        let metrics = document_metrics(&workspace, &cl, &document).unwrap();
        for (index, metric) in metrics.iter().enumerate() {
            assert!(
                line_failure(&cl, index, metric).unwrap().is_none(),
                "compact cover-letter failure at #{index}: {metric}"
            );
        }
    }

    #[test]
    fn compact_variant_keeps_horizontal_measure_and_accents() {
        // Horizontal measure feeds every fill percentage, so the compact
        // example may only tighten vertical whitespace. Pin the invariant in
        // both knob files; the gates above prove the result still passes.
        let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let harvard = workspace
            .read_toml_value(".agent/typst/styles/harvard.toml")
            .unwrap();
        let compact = workspace
            .read_toml_value(".agent/typst/styles/harvard-compact.toml")
            .unwrap();
        for pointer in ["/page", "/text", "/accents"] {
            assert_eq!(
                harvard.pointer(pointer),
                compact.pointer(pointer),
                "style-invariant section changed: {pointer}"
            );
        }
        for pointer in [
            "/cv/bullet_indent_pt",
            "/cover/highlight_inset_pt",
            "/cover/highlight_number_width_mm",
        ] {
            assert_eq!(
                harvard.pointer(pointer),
                compact.pointer(pointer),
                "horizontal knob changed: {pointer}"
            );
        }
        let fill = |style: &serde_json::Value, pointer: &str| {
            style.pointer(pointer).and_then(serde_json::Value::as_f64)
        };
        assert!(
            fill(&compact, "/cv/entry_spacing_pt") < fill(&harvard, "/cv/entry_spacing_pt"),
            "compact must tighten vertical whitespace"
        );
        assert_ne!(harvard, compact);
    }
}
