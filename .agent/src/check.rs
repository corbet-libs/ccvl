use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::application;
use crate::format;
use crate::measure;
use crate::pdf;
use crate::public;
use crate::render::{Compiler, DocumentSpec, cvl_cl_spec, cvl_cv_spec};
use crate::skills;
use crate::stations;
use crate::workspace::Workspace;

pub fn run(workspace: &Workspace) -> Result<()> {
    validate_manifest(workspace)?;
    application::validate_profiles(workspace)?;
    application::validate_station_files(workspace)?;
    application::validate_all(workspace)?;
    skills::validate(workspace)?;
    public::validate_repository(workspace)?;
    format::format_typst(workspace, true)?;
    stations::validate_interview(workspace, true)?;
    validate_embedded_fonts(workspace)?;
    render_and_verify(workspace)
}

fn validate_manifest(workspace: &Workspace) -> Result<()> {
    let manifest = workspace.read_json("ccvl.json")?;
    ensure!(
        manifest.get("format") == Some(&Value::String("ccvl-workspace".to_owned()))
            && manifest.get("schema_version") == Some(&Value::from(5)),
        "ccvl.json: unsupported workspace format or schema version"
    );
    let expected_groups = json!({
        "interview": {"root": "interview", "stations": "interview/stations.toml"},
        "cvl": {"root": "cvl", "profile": "cvl/profile.toml", "de-CH": "cvl/de-ch/application.toml", "en-CH": "cvl/en-ch/application.toml"},
        "opportunities": {"root": "opportunities", "path": "opportunities/<organisation-key>/<position-key>", "record": "application.toml", "output": "output"}
    });
    ensure!(
        manifest.get("workspace_groups") == Some(&expected_groups),
        "ccvl.json: workspace groups must be interview, cvl, and keyed opportunities"
    );
    ensure!(
        manifest.pointer("/documents/cv/de-CH")
            == Some(&Value::String("cvl/de-ch/cv.typ".to_owned()))
            && manifest.pointer("/documents/cv/en-CH")
                == Some(&Value::String("cvl/en-ch/cv.typ".to_owned()))
            && manifest.pointer("/documents/cover_letter/de-CH")
                == Some(&Value::String("cvl/de-ch/cl.typ".to_owned()))
            && manifest.pointer("/documents/cover_letter/en-CH")
                == Some(&Value::String("cvl/en-ch/cl.typ".to_owned())),
        "ccvl.json: document entry points must live below cvl/<locale>"
    );
    for relative in [
        "cvl/profile.toml",
        "interview/stations.toml",
        "cvl/de-ch/application.toml",
        "cvl/en-ch/application.toml",
        "cvl/README.md",
        "interview/README.md",
        "opportunities/README.md",
        "cvl/de-ch/cv.typ",
        "cvl/en-ch/cv.typ",
        "cvl/de-ch/cl.typ",
        "cvl/en-ch/cl.typ",
    ] {
        ensure!(
            workspace.path(relative).is_file(),
            "ccvl.json: missing referenced file {relative}"
        );
    }
    for legacy in [
        ".agents",
        ".claude",
        ".crow",
        ".vscode",
        ".zed",
        ".agent/schemas",
        ".agent/scaffolds/opportunity/application.json",
        ".agent/scaffolds/interview/profile.json",
        ".agent/scaffolds/interview/stations.json",
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
        "cvl/shared",
        "cvl/cv",
        "cvl/cl",
        "cvl/profile.json",
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
        ensure!(
            [
                ".agent",
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
    ensure!(
        manifest.pointer("/documents/cv/presets") == Some(&json!([2, 3, 4])),
        "ccvl.json: CV presets must be [2, 3, 4]"
    );
    let styles = manifest
        .pointer("/styles")
        .context("ccvl.json has no styles section")?;
    ensure!(
        styles.get("default") == Some(&Value::String("harvard".to_owned())),
        "ccvl.json: default style must be harvard"
    );
    let available = styles
        .get("available")
        .and_then(Value::as_array)
        .context("ccvl.json styles.available is missing")?;
    ensure!(
        !available.is_empty() && available.contains(&Value::String("harvard".to_owned())),
        "ccvl.json: styles.available must list harvard"
    );
    for name in available {
        let name = name
            .as_str()
            .context("ccvl.json styles.available must be style names")?;
        for extension in ["typ", "toml"] {
            let relative = format!(".agent/typst/styles/{name}.{extension}");
            ensure!(
                workspace.path(&relative).is_file(),
                "ccvl.json: style {name} is missing {relative}"
            );
        }
    }
    ensure!(
        manifest.pointer("/documents/cv/summary_lines") == Some(&Value::from(5)),
        "ccvl.json: every CV Summary must render to exactly five lines"
    );
    ensure!(
        manifest.pointer("/documents/cv/summary_fill")
            == Some(&json!({"minimum": 95, "target": 97, "maximum": 100})),
        "ccvl.json: CV Summary fill defaults must be 95/97/100"
    );
    ensure!(
        manifest.pointer("/last_line_maximum") == Some(&Value::from(102)),
        "ccvl.json: closing-line maximum must be 102"
    );
    let layout = manifest
        .pointer("/documents/cv/layout_contract")
        .context("ccvl.json has no CV layout contract")?;
    ensure!(
        layout.pointer("/page_1/entries")
            == Some(&json!({"minimum": 6, "target": 7, "maximum": 8})),
        "ccvl.json: page 1 contract changed"
    );
    ensure!(
        layout.pointer("/page_2") == Some(&json!({"entries": 10, "bullets_per_entry": 2})),
        "ccvl.json: page 2 contract changed"
    );
    ensure!(
        layout.pointer("/page_3") == Some(&json!({"entries": 10, "bullets_per_entry": 2})),
        "ccvl.json: page 3 contract changed"
    );
    ensure!(
        layout.pointer("/page_4")
            == Some(&json!({"groups": 3, "entries_per_group": 3, "bullets_per_entry": 3})),
        "ccvl.json: page 4 contract changed"
    );
    ensure!(
        layout.get("verified_only") == Some(&Value::Bool(true))
            && layout.get("unique_fact_assignment") == Some(&Value::Bool(true)),
        "ccvl.json: evidence or MECE guarantees were weakened"
    );
    let cover = manifest
        .pointer("/documents/cover_letter")
        .context("ccvl.json has no cover-letter contract")?;
    ensure!(
        cover.pointer("/body_lines") == Some(&json!({"minimum": 25, "target": 28, "maximum": 28})),
        "ccvl.json: cover-letter body contract changed"
    );
    ensure!(
        cover.pointer("/highlights/count") == Some(&Value::from(5)),
        "ccvl.json: cover letter needs exactly five highlights"
    );
    ensure!(
        cover.pointer("/widow_or_orphan_lines") == Some(&Value::from(0)),
        "ccvl.json: widow/orphan rule changed"
    );
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

fn render_and_verify(workspace: &Workspace) -> Result<()> {
    let profile = workspace.read_toml_value("cvl/profile.toml")?;
    let contacts = ["name", "email", "phone_label"]
        .into_iter()
        .map(|field| {
            profile
                .get(field)
                .and_then(Value::as_str)
                .with_context(|| format!("profile has no {field}"))
                .map(str::to_owned)
        })
        .collect::<Result<Vec<_>>>()?;
    let temporary = TempDir::new()?;
    let first = temporary.path().join("first");
    let second = temporary.path().join("second");
    let compiler = Compiler::new(workspace)?;
    for locale in ["de-ch", "en-ch"] {
        let mut verified = Vec::new();
        for pages in [2, 3, 4] {
            let spec = cvl_cv_spec(workspace, locale, pages)?;
            let first_output = render_pair(
                workspace,
                &compiler,
                &spec,
                &first.join(format!("cv-{locale}-{pages}.pdf")),
                &second.join(format!("cv-{locale}-{pages}.pdf")),
                &format!("CV build is not byte-reproducible: {locale} {pages} pages"),
                pages == 4,
            )?;
            let tracked = workspace.path(format!("cvl/{locale}/output/cv-{pages}.pdf"));
            require_semantic_pdf_match(
                &first_output,
                &tracked,
                &format!("CV output: {locale} {pages} pages"),
            )?;
            verified.push(pdf::verify(&first_output, pages, &contacts, false)?);
        }
        for page in [1, 2] {
            let baseline = verified[0].page_content(page)?;
            ensure!(
                verified[1].page_content(page)? == baseline
                    && verified[2].page_content(page)? == baseline,
                "shared CV page changed across presets: {locale} page {page}"
            );
        }
        let spec = cvl_cl_spec(workspace, locale)?;
        let first_output = render_pair(
            workspace,
            &compiler,
            &spec,
            &first.join(format!("cl-{locale}.pdf")),
            &second.join(format!("cl-{locale}.pdf")),
            &format!("cover-letter build is not byte-reproducible: {locale}"),
            true,
        )?;
        let tracked = workspace.path(format!("cvl/{locale}/output/cl.pdf"));
        require_semantic_pdf_match(
            &first_output,
            &tracked,
            &format!("cover-letter output: {locale}"),
        )?;
        pdf::verify(&first_output, 1, &contacts, true)?;
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
mod tests {
    use super::*;

    #[test]
    fn checked_in_manifest_has_fixed_contract() {
        let workspace = Workspace::discover(None).unwrap();
        validate_manifest(&workspace).unwrap();
    }
}
