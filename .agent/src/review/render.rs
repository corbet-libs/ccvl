use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use super::{
    storage,
    types::{Artifact, Check, Document},
};
use crate::{
    Workspace, content, measure, pdf,
    render::{Compiler, DocumentSpec},
    styles,
};

pub(super) fn resolve(
    workspace: &Workspace,
    request: &Document,
    output: &Path,
) -> Result<(DocumentSpec, Value)> {
    ensure!(
        request.document == "cv" || request.document == "cl",
        "document must be cv or cl"
    );
    ensure!(
        styles::normalize_locale(&request.locale)? == request.locale,
        "locale IDs must be lowercase"
    );
    let record = content::read_record(workspace, &request.application)?;
    let recorded =
        styles::record_selection(workspace, &request.document, &record, "review document")?;
    let selection = styles::Selection {
        style: request.style.clone().unwrap_or(recorded.style),
        substyle: request.substyle.clone().unwrap_or(recorded.substyle),
    };
    let leaf = styles::leaf(
        workspace,
        if request.document == "cv" { "cv" } else { "cl" },
        &request.locale,
        &selection,
    )?;
    let spec = crate::render::document_spec(
        workspace,
        &leaf,
        request.pages,
        &request.application,
        &request.profile,
        Some(output),
        request.paper.as_deref(),
    )?;
    let effective_paper = spec.inputs.get("paper").map(String::as_str);
    let settings = if leaf.settings_adapter.as_deref() == Some("document-v1") {
        crate::settings::resolve_with_paper(workspace, &leaf, effective_paper)?
    } else {
        json!({"adapter": "style-owned", "inputs": spec.inputs})
    };
    let metadata = json!({
        "id": request.id, "document": request.document, "locale": leaf.locale,
        "style": leaf.style, "substyle": leaf.substyle, "pages": spec.expected_pages,
        "paper": effective_paper, "paper_source": if request.paper.is_some() { "review request" } else if crate::paper::record_choice(&record, &request.document)?.is_some() { "record" } else { "style default or fixed geometry" },
        "contract": spec.contract, "settings": settings, "inputs": spec.inputs,
    });
    Ok((spec, metadata))
}

pub(super) fn capture(
    workspace: &Workspace,
    run: &Path,
    revision: u32,
    request: &Document,
    artifacts: &mut Vec<Artifact>,
    checks: &mut Vec<Check>,
) -> Result<Value> {
    let output = storage::revision_path(run, revision).join(format!("{}.pdf", request.id));
    let (mut spec, metadata) = resolve(workspace, request, &output)?;
    let compiler = Compiler::new(workspace)?;
    // Report mode retains a renderable candidate even when line measurement fails.
    spec.inputs.insert("line-contracts".into(), "report".into());
    let document = compiler.compile(workspace, &spec)?;
    checks.push(Check {
        document: request.id.clone(),
        operation: "compile".into(),
        passed: true,
        diagnostics: Vec::new(),
    });
    let measurement = (|| -> Result<Value> {
        let metrics = measure::document_metrics(workspace, &spec, &document)?;
        let mut failures = measure::summary_failures(workspace, &spec, &metrics)?;
        for (index, metric) in metrics.iter().enumerate() {
            if let Some(failure) = measure::line_failure(&spec, index, metric)? {
                failures.push(failure);
            }
        }
        let warnings = measure::preference_warnings(workspace, &spec, &metrics)?;
        checks.push(Check {
            document: request.id.clone(),
            operation: "measure".into(),
            passed: failures.is_empty(),
            diagnostics: failures,
        });
        Ok(json!({"metrics": metrics, "warnings": warnings}))
    })();
    let measurement = match measurement {
        Ok(value) => value,
        Err(error) => {
            checks.push(Check {
                document: request.id.clone(),
                operation: "measure".into(),
                passed: false,
                diagnostics: vec![format!("{error:#}")],
            });
            json!({"error": format!("{error:#}")})
        }
    };
    compiler.export(&spec, &document)?;
    let profile = workspace.read_toml_value(&request.profile)?;
    let policy = spec
        .contract
        .get("pdf")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let contacts = policy
        .get("required_profile_fields")
        .and_then(Value::as_array)
        .map(|fields| {
            fields
                .iter()
                .map(|field| {
                    profile
                        .get(field.as_str().context("invalid profile field")?)
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                        .context("required profile field missing")
                })
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();
    let verification = pdf::verify(&output, spec.expected_pages, &contacts, &policy);
    checks.push(Check {
        document: request.id.clone(),
        operation: "pdf-verify".into(),
        passed: verification.is_ok(),
        diagnostics: verification
            .err()
            .map(|error| vec![format!("{error:#}")])
            .unwrap_or_default(),
    });
    let mut add = |suffix: &str, role: &str, bytes: &[u8], page: Option<usize>| -> Result<()> {
        artifacts.push(storage::freeze(
            run,
            revision,
            &format!("{}:{suffix}", request.id),
            role,
            bytes,
            None,
            Some(request.id.clone()),
            page,
        )?);
        Ok(())
    };
    add("pdf", "pdf", &fs::read(&output)?, None)?;
    add(
        "content",
        "content",
        &serde_json::to_vec_pretty(&content::read_record(workspace, &request.application)?)?,
        None,
    )?;
    add(
        "selection",
        "rule",
        &serde_json::to_vec_pretty(&metadata)?,
        None,
    )?;
    add(
        "correspondence",
        "rule",
        &serde_json::to_vec_pretty(&crate::application::locale_conventions(&request.locale)?)?,
        None,
    )?;
    add(
        "measurement",
        "check",
        &serde_json::to_vec_pretty(&measurement)?,
        None,
    )?;
    let parsed = lopdf::Document::load(&output)?;
    let engine = ctypst::Engine::builder()
        .root(workspace.root())
        .fonts(ctypst::fonts::documents())
        .build()?;
    for page in 1..=spec.expected_pages {
        let text = parsed.extract_text(&[u32::try_from(page)?])?;
        ensure!(
            !text.trim().is_empty(),
            "{} page {page} has no usable text",
            request.id
        );
        add(&format!("text:{page}"), "text", text.as_bytes(), Some(page))?;
        let raster = engine.rasterize(&document, page - 1)?;
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, raster.width, raster.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header()?.write_image_data(&raster.pixels)?;
        }
        add(&format!("page:{page}"), "page", &bytes, Some(page))?;
    }
    checks.push(Check {
        document: request.id.clone(),
        operation: "extract-and-rasterize".into(),
        passed: true,
        diagnostics: Vec::new(),
    });
    Ok(metadata)
}
