use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::application;
use crate::application::{cl_leaves, cv_leaves};
use crate::format;
use crate::measure;
use crate::pdf;
use crate::public;
use crate::render::{Compiler, DocumentSpec};
use crate::skills;
use crate::styles;
use crate::workspace::Workspace;

pub fn run(workspace: &Workspace) -> Result<()> {
    run_with_artifacts(workspace, None)
}

/// Retain the first, verified PDF for independent checks in this invocation.
/// The destination must be new; existing files are never accepted as evidence.
pub fn run_with_artifacts(workspace: &Workspace, artifacts: Option<&Path>) -> Result<()> {
    if let Some(path) = artifacts {
        ensure!(
            !path.exists(),
            "artifact directory already exists: {}",
            path.display()
        );
    }
    validate_manifest(workspace)?;
    validate_correspondence(workspace)?;
    validate_styles(workspace)?;
    application::validate_profiles(workspace)?;
    application::validate_station_files(workspace)?;
    application::validate_all(workspace)?;
    skills::validate(workspace)?;
    public::validate_repository(workspace)?;
    format::format_typst(workspace, true)?;
    validate_embedded_fonts(workspace)?;
    render_and_verify(workspace, artifacts)
}

fn validate_correspondence(workspace: &Workspace) -> Result<()> {
    let root = workspace.path(".agent/typst/letter");
    let manifest = workspace.read_json(".agent/typst/letter/source.json")?;
    let files = manifest
        .get("files")
        .and_then(Value::as_object)
        .context("correspondence source manifest needs file hashes")?;
    ensure!(!files.is_empty(), "correspondence source manifest is empty");
    for (relative, hash) in files {
        ensure!(
            Path::new(relative)
                .components()
                .all(|component| matches!(component, std::path::Component::Normal(_))),
            "invalid correspondence source path: {relative}"
        );
        let path = workspace.existing_inside(root.join(relative))?;
        ensure!(
            path.starts_with(&root),
            "correspondence source escapes vendor tree: {relative}"
        );
        let actual = format!("{:x}", Sha256::digest(fs::read(&path)?));
        ensure!(
            hash.as_str() == Some(&actual),
            "vendored correspondence source changed: {relative}; update it upstream and re-vendor"
        );
    }
    for entry in walkdir::WalkDir::new(&root) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let relative = entry
                .path()
                .strip_prefix(&root)?
                .to_string_lossy()
                .replace('\\', "/");
            ensure!(
                ["README.md", "source.json"].contains(&relative.as_str())
                    || files.contains_key(&relative),
                "unrecorded correspondence source: {relative}"
            );
        }
    }
    Ok(())
}

fn validate_manifest(workspace: &Workspace) -> Result<()> {
    let manifest = workspace.read_json("ccvl.json")?;
    ensure!(
        manifest.get("format") == Some(&Value::String("ccvl-workspace".to_owned()))
            && manifest.get("schema_version") == Some(&Value::from(8)),
        "ccvl.json: unsupported workspace format or schema version"
    );
    let expected_groups = json!({
        "interview": {"root": "interview", "stations": "interview/stations.toml"},
        "cvl": {"root": "cvl", "profile": "cvl/profile.toml", "cv": "cvl/cv", "cl": "cvl/cl"},
        "opportunities": {"root": "opportunities", "path": "opportunities/<organisation-key>/<position-key>", "record": "application.toml", "pdfs": "pdfs", "typst": "typst"}
    });
    ensure!(
        manifest.get("workspace_groups") == Some(&expected_groups),
        "ccvl.json: workspace groups must be interview, cvl, and keyed opportunities"
    );
    let documents = manifest
        .get("documents")
        .and_then(Value::as_object)
        .context("ccvl.json has no documents")?;
    ensure!(
        documents.len() == 2,
        "ccvl.json must declare CV and cover-letter discovery"
    );
    for (key, document) in [("cv", "cv"), ("cover_letter", "cl")] {
        ensure!(
            documents.get(key).and_then(|value| value.get("root"))
                == Some(&Value::String(format!("cvl/{document}"))),
            "ccvl.json: {key} must use its document root"
        );
        styles::default_style(workspace, document)?;
    }
    let mut required = vec![
        "cvl/profile.toml".to_owned(),
        "interview/stations.toml".to_owned(),
        "cvl/README.md".to_owned(),
        "interview/README.md".to_owned(),
        "opportunities/README.md".to_owned(),
        ".agent/schemas/review-result.schema.json".to_owned(),
    ];
    for leaf in cv_leaves(workspace)?
        .into_iter()
        .chain(cl_leaves(workspace)?)
    {
        required.push(workspace.relative(&leaf.content())?.display().to_string());
        required.push(workspace.relative(&leaf.strings())?.display().to_string());
        required.push(workspace.relative(&leaf.adapter())?.display().to_string());
        required.push(
            workspace
                .relative(&leaf.substyle_file())?
                .display()
                .to_string(),
        );
    }
    for relative in required {
        ensure!(
            workspace.path(&relative).is_file(),
            "ccvl.json: missing referenced file {relative}"
        );
    }
    for legacy in [
        ".agents",
        ".claude",
        ".vscode",
        ".zed",
        ".agent/scaffolds/opportunity/application.json",
        ".agent/scaffolds/interview/profile.json",
        ".agent/scaffolds/interview/stations.json",
        ".agent/typst/styles/harvard.typ",
        ".agent/typst/styles/harvard.toml",
        ".agent/typst/styles/harvard-compact.typ",
        ".agent/typst/styles/harvard-compact.toml",
        ".agent/typst/styles/document.typ",
        "docs",
        "schemas",
        "scripts",
        "src",
        "targets",
        "templates",
        "tests",
        "cvl/general",
        "cvl/imports",
        "cvl/evidence",
        "cvl/profile.json",
        "cvl/de-ch",
        "cvl/en-ch",
        "cvl/de-ch/application.json",
        "cvl/en-ch/application.json",
        "interview/stations.json",
    ] {
        ensure!(
            !path_has_content(&workspace.path(legacy))?,
            "legacy workspace path must be removed: {legacy}"
        );
    }
    for entry in fs::read_dir(workspace.root())? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // Hidden directories are tooling scratch (test tempdirs, editor
        // state), never project content.
        if name.starts_with('.') {
            continue;
        }
        ensure!(
            [
                ".agent",
                ".ci",
                ".crow",
                ".git",
                ".github",
                "LICENSES",
                "cvl",
                "interview",
                "opportunities",
                "target",
            ]
            .contains(&name.as_ref()),
            "unexpected top-level directory: {name}"
        );
    }
    // Generic single-home rule: every concrete application record must live
    // under opportunities/<organisation>/<position>/. Any application.toml
    // elsewhere (a second applications tree, a backup copy, ...) fails
    // closed no matter what the stray directory is called. The only
    // exception is the neutral scaffold template shipped upstream.
    for entry in walkdir::WalkDir::new(workspace.root())
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            // Hidden directories are tooling scratch (test tempdirs, editor
            // state), never project content — same rule as the top-level
            // layout check above. Pruning here keeps parallel test tempdirs
            // with fixture records from tripping the single-home rule.
            !(name.starts_with('.') && entry.file_type().is_dir())
                && name != ".git"
                && name != "target"
        })
    {
        let entry = entry?;
        if !entry.file_type().is_file() || entry.file_name().to_string_lossy() != "application.toml"
        {
            continue;
        }
        let relative = workspace
            .relative(entry.path())?
            .display()
            .to_string()
            .replace('\\', "/");
        if relative == ".agent/scaffolds/opportunity/application.toml"
            || relative.starts_with("opportunities/")
        {
            continue;
        }
        ensure!(
            false,
            "application record outside opportunities/: {relative}; all applications must live under opportunities/<organisation>/<position>/"
        );
    }
    Ok(())
}

/// Validate the style registries below the cv/cl discovery roots: family
/// identity, supported locales, page presets, and one knob delta per listed
/// substyle with the configured default among them.
fn validate_styles(workspace: &Workspace) -> Result<()> {
    for document in ["cv", "cl"] {
        styles::leaves(workspace, document)?;
    }
    Ok(())
}

fn path_has_content(path: &std::path::Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    if !path.is_dir() {
        return Ok(true);
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() || path_has_content(&entry.path())? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn validate_embedded_fonts(workspace: &Workspace) -> Result<()> {
    ensure!(
        ctypst::fonts::documents().len() == 16,
        "the Rust binary must embed all 16 declared fonts"
    );
    let expected = [
        "Archivo-Bold.ttf",
        "Archivo-Italic.ttf",
        "Archivo-Medium.ttf",
        "Archivo-Regular.ttf",
    ];
    for name in expected {
        let bytes = fs::read(workspace.path(format!(".agent/typst/fonts/{name}")))?;
        ensure!(
            matches!(
                bytes.get(..4),
                Some(b"\x00\x01\x00\x00" | b"OTTO" | b"true" | b"typ1")
            ),
            "bundled font is missing or invalid: {name}"
        );
    }
    Ok(())
}

fn render_and_verify(workspace: &Workspace, artifacts: Option<&Path>) -> Result<()> {
    let profile = workspace.read_toml_value("cvl/profile.toml")?;
    let temporary = TempDir::new()?;
    let compiler = Compiler::new(workspace)?;
    let mut checked_pdfs = Vec::new();
    for leaf in cv_leaves(workspace)?
        .into_iter()
        .chain(cl_leaves(workspace)?)
    {
        let showcase = crate::render::cvl_spec(workspace, &leaf, leaf.default_pages)?;
        let showcase_paper = showcase.inputs.get("paper").map(String::as_str);
        let choices = leaf.paper.as_ref().map_or_else(
            || vec![None],
            |registry| registry.sizes.keys().map(|id| Some(id.as_str())).collect(),
        );
        for paper in choices {
            let mut verified = Vec::new();
            let selected = crate::paper::select(leaf.paper.as_ref(), &leaf.locale, paper)?;
            let is_default = paper == showcase_paper;
            let contract = crate::paper::contract(&leaf, selected);
            let policy = contract.get("pdf").cloned().unwrap_or_else(|| json!({}));
            let contacts = policy
                .get("required_profile_fields")
                .and_then(Value::as_array)
                .map(|fields| {
                    fields
                        .iter()
                        .map(|field| {
                            let key = field
                                .as_str()
                                .context("required_profile_fields must contain names")?;
                            profile
                                .get(key)
                                .and_then(Value::as_str)
                                .map(str::to_owned)
                                .with_context(|| format!("profile has no {key}"))
                        })
                        .collect::<Result<Vec<_>>>()
                })
                .transpose()?
                .unwrap_or_default();
            for pages in &leaf.pages {
                let spec = crate::render::cvl_spec_with_paper(workspace, &leaf, *pages, paper)?;
                let label = format!(
                    "{}-{}-{}-{}-{pages}",
                    leaf.document, leaf.style, leaf.substyle, leaf.locale
                );
                let label = if is_default {
                    label
                } else {
                    format!("{label}-{}", paper.unwrap())
                };
                let first = render_pair(
                    workspace,
                    &compiler,
                    &spec,
                    &temporary.path().join(format!("{label}-first.pdf")),
                    &temporary.path().join(format!("{label}-second.pdf")),
                    &format!("{} is not byte-reproducible", spec.name),
                    true,
                )?;
                if is_default {
                    require_semantic_pdf_match(
                        &first,
                        &spec.output,
                        &format!("{} output", spec.name),
                    )?;
                }
                verified.push((*pages, pdf::verify(&first, *pages, &contacts, &policy)?));
                checked_pdfs.push((first, format!("{label}.pdf")));
            }
            if let Some(pages) = leaf.contract.get("shared_pages").and_then(Value::as_array) {
                for page in pages {
                    let page = u32::try_from(
                        page.as_u64()
                            .context("shared_pages must contain page numbers")?,
                    )?;
                    ensure!(
                        page > 0 && verified.iter().all(|(count, _)| *count >= page as usize),
                        "shared page is absent from a preset"
                    );
                    let baseline = verified[0].1.page_content(page)?;
                    for (_, pdf) in &verified[1..] {
                        ensure!(
                            pdf.page_content(page)? == baseline,
                            "shared page {page} changed across presets: {}",
                            leaf.dir.display()
                        );
                    }
                }
            }
        }
    }
    if let Some(destination) = artifacts {
        // Publish only after every document, shared-page and repeat-render check
        // succeeds. create_dir rejects an existing destination, including races.
        fs::create_dir(destination).with_context(|| {
            format!(
                "cannot create new artifact directory {}",
                destination.display()
            )
        })?;
        for (source, name) in checked_pdfs {
            fs::copy(source, destination.join(name))?;
        }
    }
    Ok(())
}

/// Render one spec twice for byte-reproducibility. When `measured`, the first
/// build runs in report mode so its document serves both the line-measurement
/// gate and the first PDF; the enforce-mode second build stays the backstop
/// and proves both modes emit identical bytes. Unmeasured presets compile in
/// enforce mode twice, as before.
fn render_pair(
    workspace: &Workspace,
    compiler: &Compiler,
    spec: &DocumentSpec,
    first: &Path,
    second: &Path,
    repro_label: &str,
    measured: bool,
) -> Result<PathBuf> {
    let mut one = spec.clone();
    one.output = first.to_path_buf();
    let mut two = spec.clone();
    two.output = second.to_path_buf();
    if measured {
        let mut report = one.clone();
        report
            .inputs
            .insert("line-contracts".to_owned(), "report".to_owned());
        let document = compiler.compile(workspace, &report)?;
        let metrics = measure::document_metrics(workspace, &one, &document)?;
        let mut failures = measure::summary_failures(workspace, &one, &metrics)?;
        for (index, metric) in metrics.iter().enumerate() {
            if let Some(failure) = measure::line_failure(&one, index, metric)? {
                failures.push(failure);
            }
        }
        let advisories = measure::preference_warnings(workspace, &one, &metrics)?;
        for advisory in &advisories {
            println!("WARN {advisory}");
        }
        ensure!(
            failures.is_empty(),
            "line measurement failed: {}",
            failures.join("; ")
        );
        compiler.export(&one, &document)?;
    } else {
        compiler.render(workspace, &one)?;
    }
    compiler.render(workspace, &two)?;
    ensure!(
        fs::read(&one.output)? == fs::read(&two.output)?,
        "{repro_label}"
    );
    Ok(one.output.clone())
}

fn require_semantic_pdf_match(
    generated: &std::path::Path,
    tracked: &std::path::Path,
    label: &str,
) -> Result<()> {
    ensure!(
        pdf::semantic_signature(generated)? == pdf::semantic_signature(tracked)?,
        "tracked {label} is stale or platform-dependent"
    );
    Ok(())
}

#[cfg(test)]
mod tests;
