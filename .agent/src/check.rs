use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::application;
use crate::application::{cl_leaves, cv_leaves};
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
    validate_styles(workspace)?;
    validate_contracts(workspace)?;
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
            && manifest.get("schema_version") == Some(&Value::from(7)),
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
    // Style-major layout: the manifest only names the cv/cl discovery roots.
    // Templates, contracts, and substyle registries live below those roots.
    ensure!(
        manifest.get("documents")
            == Some(&json!({"cv": {"root": "cvl/cv"}, "cover_letter": {"root": "cvl/cl"}})),
        "ccvl.json: documents must be the cv/cl discovery roots"
    );
    ensure!(
        manifest.get("styles").is_none(),
        "ccvl.json: styles moved to cvl/cv/style.toml and cvl/cl/style.toml"
    );
    let mut required = vec![
        "cvl/profile.toml".to_owned(),
        "interview/stations.toml".to_owned(),
        "cvl/README.md".to_owned(),
        "interview/README.md".to_owned(),
        "opportunities/README.md".to_owned(),
        "cvl/cv/style.toml".to_owned(),
        "cvl/cv/contract.toml".to_owned(),
        "cvl/cl/style.toml".to_owned(),
        "cvl/cl/contract.toml".to_owned(),
        "cvl/shared/style.toml".to_owned(),
        "cvl/shared/defaults.toml".to_owned(),
        "cvl/shared/style.typ".to_owned(),
        "cvl/cv/src/cv.typ".to_owned(),
        "cvl/cv/src/entries-de.typ".to_owned(),
        "cvl/cv/src/entries-en.typ".to_owned(),
        "cvl/cl/src/cl.typ".to_owned(),
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
        ".crow",
        ".vscode",
        ".zed",
        ".agent/schemas",
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
    Ok(())
}

/// Validate the style registries below the cv/cl discovery roots: family
/// identity, supported locales, page presets, and one knob delta per listed
/// substyle with the configured default among them.
fn validate_styles(workspace: &Workspace) -> Result<()> {
    for (root, document, pages) in [
        ("cvl/cv", "cv", json!([2, 3, 4])),
        ("cvl/cl", "cl", json!([1])),
    ] {
        let style = workspace.read_toml_value(format!("{root}/style.toml"))?;
        ensure!(
            style.get("id") == Some(&Value::String("harvard".to_owned())),
            "{root}/style.toml: style family must be harvard"
        );
        ensure!(
            style.get("documents") == Some(&Value::from(vec![Value::from(document)])),
            "{root}/style.toml: documents must be [{document}]"
        );
        ensure!(
            style.get("supports_locales") == Some(&json!(["de-ch", "en-ch"])),
            "{root}/style.toml: supported locales must be de-ch and en-ch"
        );
        ensure!(
            style.get("pages") == Some(&pages),
            "{root}/style.toml: page presets changed"
        );
        let substyles = style
            .get("substyles")
            .and_then(Value::as_array)
            .context(format!("{root}/style.toml has no substyles"))?;
        ensure!(
            !substyles.is_empty(),
            "{root}/style.toml: substyles must not be empty"
        );
        let default = style
            .get("default_substyle")
            .and_then(Value::as_str)
            .context(format!("{root}/style.toml has no default_substyle"))?;
        ensure!(
            substyles.contains(&Value::String(default.to_owned())),
            "{root}/style.toml: default substyle {default:?} is not listed"
        );
        for name in substyles {
            let name = name
                .as_str()
                .context(format!("{root}/style.toml substyles must be names"))?;
            let relative = format!("{root}/{name}/substyle.toml");
            ensure!(
                workspace.path(&relative).is_file(),
                "{root}/style.toml: substyle {name} is missing {relative}"
            );
        }
    }
    Ok(())
}

/// Pin the frozen measurement contracts in the style tree. Every value here
/// is a product guarantee: weakening one silently reflows measured lines.
fn validate_contracts(workspace: &Workspace) -> Result<()> {
    let cv = application::document_contract(workspace, "cv")?;
    ensure!(
        cv.pointer("/presets") == Some(&json!([2, 3, 4])),
        "CV contract: presets must be [2, 3, 4]"
    );
    ensure!(
        cv.pointer("/summary_lines") == Some(&Value::from(5)),
        "CV contract: every CV Summary must render to exactly five lines"
    );
    ensure!(
        cv.pointer("/summary_fill") == Some(&json!({"minimum": 95, "target": 97, "maximum": 100})),
        "CV contract: Summary fill defaults must be 95/97/100"
    );
    ensure!(
        cv.pointer("/last_line_maximum") == Some(&Value::from(102)),
        "CV contract: closing-line maximum must be 102"
    );
    let layout = cv
        .pointer("/layout_contract")
        .context("CV contract has no layout contract")?;
    ensure!(
        layout.pointer("/page_1/entries")
            == Some(&json!({"minimum": 6, "target": 7, "maximum": 8})),
        "CV contract: page 1 contract changed"
    );
    ensure!(
        layout.pointer("/page_2") == Some(&json!({"entries": 10, "bullets_per_entry": 2})),
        "CV contract: page 2 contract changed"
    );
    ensure!(
        layout.pointer("/page_3") == Some(&json!({"entries": 10, "bullets_per_entry": 2})),
        "CV contract: page 3 contract changed"
    );
    ensure!(
        layout.pointer("/page_4")
            == Some(&json!({"groups": 3, "entries_per_group": 3, "bullets_per_entry": 3})),
        "CV contract: page 4 contract changed"
    );
    ensure!(
        layout.get("verified_only") == Some(&Value::Bool(true))
            && layout.get("unique_fact_assignment") == Some(&Value::Bool(true)),
        "CV contract: evidence or MECE guarantees were weakened"
    );
    let cl = application::document_contract(workspace, "cl")?;
    let paragraphs = cl
        .get("paragraphs")
        .and_then(Value::as_array)
        .context("cover-letter contract has no paragraphs")?;
    // The strict 3|5|5|5|5|3 framework: exactly six paragraphs with exact
    // line budgets.
    ensure!(
        paragraphs.len() == 6,
        "cover-letter contract: paragraph framework changed"
    );
    for (paragraph, (minimum, maximum)) in
        paragraphs
            .iter()
            .zip([(3, 3), (5, 5), (5, 5), (5, 5), (5, 5), (3, 3)])
    {
        ensure!(
            paragraph.pointer("/lines/minimum") == Some(&Value::from(minimum))
                && paragraph.pointer("/lines/maximum") == Some(&Value::from(maximum)),
            "cover-letter contract: paragraph line framework changed"
        );
    }
    ensure!(
        cl.pointer("/body_lines") == Some(&json!({"minimum": 26, "target": 26, "maximum": 26})),
        "cover-letter contract: body contract changed"
    );
    ensure!(
        cl.pointer("/highlights/count") == Some(&Value::from(5)),
        "cover letter needs exactly five highlights"
    );
    ensure!(
        cl.pointer("/line_fill/body")
            == Some(&json!({
                "minimum": 75,
                "non_final_minimum": 95,
                "target": 97,
                "maximum": 100
            })),
        "cover-letter contract: body fill must stay 75/95/97/100; the higher \
         non-final floor is the density gate and may not be weakened"
    );
    ensure!(
        cl.pointer("/line_fill/highlight")
            == Some(&json!({"minimum": 70, "target": 82, "maximum": 100})),
        "cover-letter contract: highlight fill must stay 70/82/100"
    );
    ensure!(
        cl.pointer("/vertical_rhythm/gap_pt")
            == Some(&json!({"minimum": 12, "target": 20, "maximum": 30})),
        "cover-letter contract: vertical rhythm changed"
    );
    ensure!(
        cl.pointer("/vertical_rhythm/highlight_center_percent")
            == Some(&json!({"minimum": 50, "target": 56, "maximum": 60})),
        "cover-letter contract: highlight position changed"
    );
    ensure!(
        cl.pointer("/widow_or_orphan_lines") == Some(&Value::from(0)),
        "cover-letter contract: widow/orphan rule changed"
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
    for leaf in cv_leaves(workspace)? {
        let mut verified = Vec::new();
        for pages in [2, 3, 4] {
            let spec = cvl_cv_spec(workspace, leaf.locale, pages, Some(&leaf.substyle))?;
            let label = format!(
                "CV build is not byte-reproducible: {} {pages} pages",
                spec.name
            );
            let first_output = render_pair(
                workspace,
                &compiler,
                &spec,
                &first.join(format!("cv-{}-{}-{pages}.pdf", leaf.locale, leaf.substyle)),
                &second.join(format!("cv-{}-{}-{pages}.pdf", leaf.locale, leaf.substyle)),
                &label,
                pages == 4,
            )?;
            let tracked = leaf.dir.join("pdf").join(format!("cv-{pages}.pdf"));
            require_semantic_pdf_match(
                &first_output,
                &tracked,
                &format!("CV output: {}", spec.name),
            )?;
            verified.push(pdf::verify(&first_output, pages, &contacts, false)?);
        }
        for page in [1, 2] {
            let baseline = verified[0].page_content(page)?;
            ensure!(
                verified[1].page_content(page)? == baseline
                    && verified[2].page_content(page)? == baseline,
                "shared CV page changed across presets: {} page {page}",
                leaf.dir.display()
            );
        }
    }
    for leaf in cl_leaves(workspace)? {
        let spec = cvl_cl_spec(workspace, leaf.locale, Some(&leaf.substyle))?;
        let first_output = render_pair(
            workspace,
            &compiler,
            &spec,
            &first.join(format!("cl-{}-{}.pdf", leaf.locale, leaf.substyle)),
            &second.join(format!("cl-{}-{}.pdf", leaf.locale, leaf.substyle)),
            &format!("cover-letter build is not byte-reproducible: {}", spec.name),
            true,
        )?;
        let tracked = leaf.dir.join("pdf").join("cl.pdf");
        require_semantic_pdf_match(
            &first_output,
            &tracked,
            &format!("cover-letter output: {}", spec.name),
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
    fn checked_in_manifest_styles_and_contracts_are_fixed() {
        let workspace = Workspace::discover(None).unwrap();
        validate_manifest(&workspace).unwrap();
        validate_styles(&workspace).unwrap();
        validate_contracts(&workspace).unwrap();
    }

    #[test]
    fn cover_letter_contract_rejects_weakened_density_and_closing_spill() {
        let repository = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let temporary = TempDir::new().unwrap();
        // Style validation only needs these references to exist, with real
        // registry and manifest copies. Keep the fixture isolated from
        // private records and local agent directories.
        for relative in [
            "cvl/profile.toml",
            "interview/stations.toml",
            "cvl/README.md",
            "interview/README.md",
            "opportunities/README.md",
            "cvl/shared/style.toml",
            "cvl/shared/defaults.toml",
            "cvl/shared/style.typ",
            "cvl/cv/src/cv.typ",
            "cvl/cv/src/entries-de.typ",
            "cvl/cv/src/entries-en.typ",
            "cvl/cl/src/cl.typ",
        ] {
            let path = temporary.path().join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "").unwrap();
        }
        for relative in [
            "ccvl.json",
            "cvl/cv/style.toml",
            "cvl/cl/style.toml",
            "cvl/cv/contract.toml",
            "cvl/cl/contract.toml",
        ] {
            let path = temporary.path().join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, fs::read(repository.path(relative)).unwrap()).unwrap();
        }
        for leaf in cv_leaves(&repository)
            .unwrap()
            .into_iter()
            .chain(cl_leaves(&repository).unwrap())
        {
            for relative in [
                repository.relative(&leaf.content()).unwrap(),
                repository.relative(&leaf.strings()).unwrap(),
                repository.relative(&leaf.adapter()).unwrap(),
                repository.relative(&leaf.substyle_file()).unwrap(),
            ] {
                let path = temporary.path().join(relative);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, "").unwrap();
            }
        }
        let workspace = Workspace::at(temporary.path()).unwrap();
        validate_manifest(&workspace).unwrap();
        validate_styles(&workspace).unwrap();
        validate_contracts(&workspace).unwrap();

        for (pointer, weakened, message) in [
            ("/line_fill/body/non_final_minimum", 75, "body fill"),
            ("/line_fill/body/minimum", 60, "body fill"),
            ("/line_fill/body/maximum", 102, "body fill"),
            ("/line_fill/highlight/minimum", 60, "highlight fill"),
        ] {
            let contract_path = temporary.path().join("cvl/cl/contract.toml");
            let text = fs::read_to_string(&contract_path).unwrap();
            let mut contract: toml::Value = toml::from_str(&text).unwrap();
            let target = pointer.split('/').filter(|part| !part.is_empty()).fold(
                &mut contract,
                |value, part| {
                    value
                        .as_table_mut()
                        .expect("contract section")
                        .get_mut(part)
                        .expect("contract key")
                },
            );
            *target = toml::Value::Integer(weakened);
            fs::write(&contract_path, toml::to_string(&contract).unwrap()).unwrap();
            let error = validate_contracts(&workspace).unwrap_err().to_string();
            assert!(error.contains(message), "{pointer}: {error}");
            fs::write(&contract_path, text).unwrap();
            validate_contracts(&workspace).unwrap();
        }
    }
}
