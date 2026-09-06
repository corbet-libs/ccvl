use std::collections::HashMap;
use std::sync::OnceLock;

use crate::render::{
    Compiler, DocumentKind, DocumentSpec, cvl_cl_spec, cvl_cv_spec, opportunity_specs,
};
use crate::workspace::Workspace;
use anyhow::{Context, Result, ensure};
use cgreet::recipient_salutation_warning;
use ctypst::Document;
use serde_json::Value;

pub fn cvl_specs(workspace: &Workspace) -> Result<Vec<DocumentSpec>> {
    let mut specs = Vec::new();
    for locale in ["de-ch", "en-ch"] {
        let mut cv = cvl_cv_spec(workspace, locale, 4)?;
        cv.inputs
            .insert("line-contracts".to_owned(), "report".to_owned());
        specs.push(cv);
        let mut cl = cvl_cl_spec(workspace, locale)?;
        cl.inputs
            .insert("line-contracts".to_owned(), "report".to_owned());
        specs.push(cl);
    }
    Ok(specs)
}

pub fn keyed_specs(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
) -> Result<Vec<DocumentSpec>> {
    let mut specs = opportunity_specs(workspace, organisation, position)?;
    for spec in &mut specs {
        spec.inputs
            .insert("line-contracts".to_owned(), "report".to_owned());
    }
    Ok(specs)
}

pub fn evaluate(workspace: &Workspace, spec: &DocumentSpec) -> Result<Vec<Value>> {
    evaluate_with(workspace, &Compiler::new(workspace)?, spec)
}

fn evaluate_with(
    workspace: &Workspace,
    compiler: &Compiler,
    spec: &DocumentSpec,
) -> Result<Vec<Value>> {
    let document = compiler.compile(workspace, spec)?;
    document_metrics(workspace, spec, &document)
}

/// Query the line/layout metrics of an already compiled document and enforce
/// the structural metric set. Lets callers reuse one compilation for both
/// measurement and PDF export instead of compiling twice.
pub fn document_metrics(
    workspace: &Workspace,
    spec: &DocumentSpec,
    document: &Document,
) -> Result<Vec<Value>> {
    let mut metrics = Vec::new();
    for label_name in ["ccvl-line", "ccvl-layout"] {
        metrics.extend(
            ctypst::query_json(document, label_name)
                .with_context(|| format!("cannot query {} metrics", spec.name))?,
        );
    }
    validate_metric_set(workspace, spec, &metrics)?;
    Ok(metrics)
}

/// A summary line is counsel, not verdict: the count rule owns the gate,
/// density only advises (or fails thin lines without an explicit override).
/// Shared by the standalone `measure` command and the fused check gate.
fn is_summary(metric: &Value) -> bool {
    metric.get("kind").and_then(Value::as_str) == Some("cv-summary")
}

/// Format the fill violation of one measured line, if any. Shared by the
/// standalone `measure` command and the fused check gate so both report the
/// same failure text. Summary lines never fail here; see `summary_failures`.
pub fn line_failure(spec: &DocumentSpec, index: usize, metric: &Value) -> Result<Option<String>> {
    if is_summary(metric) {
        return Ok(None);
    }
    violation(metric).map(|state| {
        state.map(|state| {
            format!(
                "{} #{} {state}: {:.1} outside {}–{}",
                spec.name,
                index + 1,
                number_field(metric, "actual_fill").expect("violation implies numeric actual_fill"),
                number_field(metric, "min_fill").expect("violation implies numeric min_fill"),
                number_field(metric, "max_fill").expect("violation implies numeric max_fill"),
            )
        })
    })
}

/// Soll inputs for summary counsel: the record's explicit thin allowance
/// plus the contract's density floor and uniform closing-line maximum.
struct SummaryPolicy {
    allow_thin: bool,
    floor: f64,
    edge: f64,
    last_max: f64,
}

fn summary_policy(workspace: &Workspace, spec: &DocumentSpec) -> Result<SummaryPolicy> {
    let contract = workspace.read_json("ccvl.json")?;
    let fill = contract
        .pointer("/documents/cv/summary_fill")
        .context("ccvl.json has no summary fill contract")?;
    let floor = fill
        .get("minimum")
        .and_then(Value::as_f64)
        .context("summary fill contract has no minimum")?;
    let edge = fill
        .get("maximum")
        .and_then(Value::as_f64)
        .context("summary fill contract has no maximum")?;
    let last_max = contract
        .pointer("/last_line_maximum")
        .and_then(Value::as_f64)
        .context("ccvl.json has no closing-line maximum")?;
    let typst_path = spec
        .inputs
        .get("application")
        .context("summary spec has no application input")?;
    let record = workspace.read_toml_value(
        workspace.relative(&workspace.existing_inside(typst_path.trim_start_matches('/'))?)?,
    )?;
    let allow_thin = record
        .pointer("/cv/allow_thin")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Ok(SummaryPolicy {
        allow_thin,
        floor,
        edge,
        last_max,
    })
}

/// Diagnose summary lines against the counsel rules. The line COUNT stays
/// hard in `validate_metric_set`; here thin lines fail unless the record
/// explicitly wants them, and overflow past the closing-line maximum fails
/// while invisible spill only counsels.
pub fn summary_failures(
    workspace: &Workspace,
    spec: &DocumentSpec,
    metrics: &[Value],
) -> Result<Vec<String>> {
    if spec.kind != DocumentKind::Cv {
        return Ok(Vec::new());
    }
    let policy = summary_policy(workspace, spec)?;
    let mut failures = Vec::new();
    for (index, metric) in metrics
        .iter()
        .enumerate()
        .filter(|(_, metric)| is_summary(metric))
    {
        let actual = number_field(metric, "actual_fill")?;
        if actual < policy.floor && !policy.allow_thin {
            failures.push(format!(
                "{} #{} too short: {:.1} below {} (set cv.allow_thin to keep a thin line explicitly)",
                spec.name,
                index + 1,
                actual,
                policy.floor
            ));
        } else if actual > policy.last_max {
            failures.push(format!(
                "{} #{} too long: {:.1} past {} closing-line maximum (rewrite with signal, not filler)",
                spec.name,
                index + 1,
                actual,
                policy.last_max
            ));
        }
    }
    Ok(failures)
}

pub fn measure(
    workspace: &Workspace,
    specs: &[DocumentSpec],
    show_all: bool,
    emit: bool,
) -> Result<Vec<String>> {
    let compiler = Compiler::new(workspace)?;
    let mut failures = Vec::new();
    for spec in specs {
        let metrics = evaluate_with(workspace, &compiler, spec)?;
        let advisories = preference_warnings(workspace, spec, &metrics)?;
        if emit {
            for advisory in &advisories {
                println!("WARN {advisory}");
            }
        }
        let mut document_failures = summary_failures(workspace, spec, &metrics)?;
        for (index, metric) in metrics.iter().enumerate() {
            let state = violation(metric)?;
            let advisory = is_summary(metric);
            if show_all || state.is_some() || advisory {
                let status = if advisory {
                    "NOTE"
                } else if state.is_none() {
                    "PASS"
                } else {
                    "FAIL"
                };
                let unit = metric.get("unit").and_then(Value::as_str).unwrap_or("%");
                println!(
                    "{status} {} #{} {} {:.1}{unit} (target {}{unit}, allowed {}–{}{unit}): {}",
                    spec.name,
                    index + 1,
                    string_field(metric, "kind")?,
                    number_field(metric, "actual_fill")?,
                    number_field(metric, "target_fill")?,
                    number_field(metric, "min_fill")?,
                    number_field(metric, "max_fill")?,
                    compact_text(metric.get("text").unwrap_or(&Value::Null))
                );
            }
            if let Some(failure) = line_failure(spec, index, metric)? {
                document_failures.push(failure);
            }
        }
        failures.extend(document_failures.iter().cloned());
        if emit && !show_all {
            println!(
                "{} {}: {} measured lines{}",
                if document_failures.is_empty() {
                    "PASS"
                } else {
                    "FAIL"
                },
                spec.name,
                metrics.len(),
                if advisories.is_empty() {
                    String::new()
                } else {
                    format!(", {} preference warning(s)", advisories.len())
                }
            );
        }
    }
    if !failures.is_empty() && emit {
        eprintln!(
            "Line measurement failed. Rewrite with relevant, verified signal—not filler—then run `ccvl measure` again."
        );
    }
    Ok(failures)
}

pub fn violation(metric: &Value) -> Result<Option<&'static str>> {
    let actual = number_field(metric, "actual_fill")?;
    if actual < number_field(metric, "min_fill")? {
        Ok(Some("too short"))
    } else if actual > number_field(metric, "max_fill")? {
        Ok(Some("too long"))
    } else {
        Ok(None)
    }
}

fn validate_metric_set(
    workspace: &Workspace,
    spec: &DocumentSpec,
    metrics: &[Value],
) -> Result<()> {
    let mut counts = HashMap::<&str, usize>::new();
    for metric in metrics {
        *counts.entry(string_field(metric, "kind")?).or_default() += 1;
    }
    match spec.kind {
        DocumentKind::Cv => {
            ensure!(
                counts.get("cv-summary") == Some(&5),
                "{}: expected exactly five measured Summary lines",
                spec.name
            );
            for required in ["cv-heading", "cv-subheading", "cv-bullet"] {
                ensure!(
                    counts.get(required).copied().unwrap_or_default() > 0,
                    "{}: no measured {required} lines found",
                    spec.name
                );
            }
        }
        DocumentKind::CoverLetter => {
            let contract = workspace.read_json("ccvl.json")?;
            let contract = contract
                .pointer("/documents/cover_letter")
                .context("missing cover-letter contract")?;
            let paragraph_counts = paragraph_counts(spec, metrics)?;
            let paragraph_contracts = contract
                .get("paragraphs")
                .and_then(Value::as_array)
                .context("missing paragraph contracts")?;
            for (index, (actual, declared)) in
                paragraph_counts.iter().zip(paragraph_contracts).enumerate()
            {
                let min = usize::try_from(
                    declared
                        .pointer("/lines/minimum")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                )?;
                let max = usize::try_from(
                    declared
                        .pointer("/lines/maximum")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                )?;
                ensure!(
                    (min..=max).contains(actual),
                    "{}: paragraph {} must use {min}–{max} lines, found {actual}",
                    spec.name,
                    index + 1
                );
            }
            let body = counts.get("cl-body").copied().unwrap_or_default();
            ensure!(
                (25..=28).contains(&body),
                "{}: expected 25–28 body lines, found {body}",
                spec.name
            );
            ensure!(
                counts.get("cl-highlight") == Some(&5),
                "{}: expected five one-line highlights",
                spec.name
            );
            ensure!(
                counts.get("cl-vertical-gap") == Some(&1)
                    && counts.get("cl-highlight-center") == Some(&1),
                "{}: expected one vertical-gap and one highlight-position metric",
                spec.name
            );
            ensure!(
                metrics.len() == body + 7,
                "{}: unexpected cover-letter metric set",
                spec.name
            );
        }
    }
    Ok(())
}

fn paragraph_pattern() -> &'static regex::Regex {
    static PATTERN: OnceLock<regex::Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        regex::Regex::new(r"^cl\.paragraph\.(\d+)\.(\d+)$").expect("fixed metric id pattern")
    })
}

fn summary_counsels(
    workspace: &Workspace,
    spec: &DocumentSpec,
    metrics: &[Value],
) -> Result<Vec<String>> {
    let policy = summary_policy(workspace, spec)?;
    let mut warnings = Vec::new();
    for (index, metric) in metrics
        .iter()
        .enumerate()
        .filter(|(_, metric)| is_summary(metric))
    {
        let actual = number_field(metric, "actual_fill")?;
        if actual < policy.floor && policy.allow_thin {
            warnings.push(format!(
                "{}: line {} is thin at {:.1}; accepted as explicitly wanted",
                spec.name,
                index + 1,
                actual
            ));
        } else if actual > policy.edge && actual <= policy.last_max {
            warnings.push(format!(
                "{}: line {} spills {:.1} points past the block edge; accepted as invisible",
                spec.name,
                index + 1,
                actual - policy.edge
            ));
        }
    }
    Ok(warnings)
}

fn paragraph_counts(spec: &DocumentSpec, metrics: &[Value]) -> Result<Vec<usize>> {
    let pattern = paragraph_pattern();
    let mut counts = vec![0; 6];
    for metric in metrics
        .iter()
        .filter(|metric| metric.get("kind").and_then(Value::as_str) == Some("cl-body"))
    {
        let id = string_field(metric, "id")?;
        let captures = pattern
            .captures(id)
            .with_context(|| format!("{}: malformed cover-letter body metric id", spec.name))?;
        let paragraph = captures[1].parse::<usize>()?;
        ensure!(
            (1..=6).contains(&paragraph),
            "{}: metric references an unknown paragraph",
            spec.name
        );
        counts[paragraph - 1] += 1;
    }
    Ok(counts)
}

/// Accepted-but-dispreferred line totals per cover-letter region, plus the
/// target-neutral salutation counsel. Spelled out as warnings rather than
/// failures; the fused check gate calls this for its error propagation even
/// when it discards the advisories.
pub fn preference_warnings(
    workspace: &Workspace,
    spec: &DocumentSpec,
    metrics: &[Value],
) -> Result<Vec<String>> {
    if spec.kind == DocumentKind::Cv {
        return summary_counsels(workspace, spec, metrics);
    }
    if spec.kind != DocumentKind::CoverLetter {
        return Ok(Vec::new());
    }
    let counts = paragraph_counts(spec, metrics)?;
    let contract = workspace.read_json("ccvl.json")?;
    let contract = contract
        .pointer("/documents/cover_letter")
        .context("missing cover-letter contract")?;
    let mut warnings = Vec::new();
    for region in contract
        .get("paragraph_regions")
        .and_then(Value::as_array)
        .context("missing paragraph regions")?
    {
        let numbers = region
            .get("paragraphs")
            .and_then(Value::as_array)
            .context("missing region paragraphs")?
            .iter()
            .filter_map(Value::as_u64)
            .map(usize::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let total = numbers
            .iter()
            .map(|number| counts[number - 1])
            .sum::<usize>();
        let preferred = region
            .get("preferred_totals")
            .and_then(Value::as_array)
            .context("missing preferred totals")?
            .iter()
            .filter_map(Value::as_u64)
            .map(usize::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if !preferred.contains(&total) {
            warnings.push(format!(
                "{}: paragraphs {}–{} use {total} lines; accepted, but {} is preferred",
                spec.name,
                numbers[0],
                numbers[numbers.len() - 1],
                preferred
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" or ")
            ));
        }
    }
    if counts[5] != 3 {
        warnings.push(format!(
            "{}: paragraph 6 uses {} lines; accepted, but 3 is preferred to mirror paragraph 1",
            spec.name, counts[5]
        ));
    }
    warnings.extend(recipient_warnings(workspace, spec)?);
    Ok(warnings)
}

/// Visible, non-blocking counsel when a cover letter has no recipient name.
/// The Typst renderer falls back to the generic salutation ("Dear Hiring
/// Manager," / "Sehr geehrte Damen und Herren"), which stays valid for the
/// target-neutral showcase; a tailored opportunity should name a person.
/// German records additionally warn when the name carries no parsable
/// Herr/Frau honorific. Missing application inputs (unit fixtures) yield no
/// warning so metric-set tests stay focused.
pub fn recipient_warnings(workspace: &Workspace, spec: &DocumentSpec) -> Result<Vec<String>> {
    use cgreet::de_honorific_warning;

    if spec.kind != DocumentKind::CoverLetter {
        return Ok(Vec::new());
    }
    let Some(typst_path) = spec.inputs.get("application") else {
        return Ok(Vec::new());
    };
    let inside = workspace.existing_inside(typst_path.trim_start_matches('/'));
    let Ok(resolved) = inside else {
        return Ok(Vec::new());
    };
    let Ok(relative) = workspace.relative(&resolved) else {
        return Ok(Vec::new());
    };
    let Ok(record) = workspace.read_toml_value(relative) else {
        return Ok(Vec::new());
    };
    let name = record
        .pointer("/job/cl_recipient/name")
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut warnings = Vec::new();
    if let Some(warning) = recipient_salutation_warning(&spec.name, name) {
        warnings.push(warning);
        return Ok(warnings);
    }
    let language = record
        .pointer("/options/language")
        .and_then(Value::as_str)
        .unwrap_or("");
    if language.to_ascii_lowercase().starts_with("de")
        && let Some(warning) = de_honorific_warning(&spec.name, name)
    {
        warnings.push(warning);
    }
    Ok(warnings)
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("metric has no {field}"))
}

fn number_field(value: &Value, field: &str) -> Result<f64> {
    value
        .get(field)
        .and_then(Value::as_f64)
        .with_context(|| format!("metric has no numeric {field}"))
}

fn compact_text(value: &Value) -> String {
    let text = value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_owned);
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= 120 {
        compact
    } else {
        format!("{}…", compact.chars().take(119).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use serde_json::json;

    use super::*;

    fn workspace() -> Workspace {
        Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    fn cover_letter_spec() -> DocumentSpec {
        DocumentSpec {
            name: "fixture".to_owned(),
            kind: DocumentKind::CoverLetter,
            source: PathBuf::from("fixture.typ"),
            output: PathBuf::from("fixture.pdf"),
            inputs: std::collections::BTreeMap::new(),
            expected_pages: 1,
        }
    }

    fn cv_spec(application: &str) -> DocumentSpec {
        let mut inputs = std::collections::BTreeMap::new();
        inputs.insert("application".to_owned(), application.to_owned());
        DocumentSpec {
            name: "fixture".to_owned(),
            kind: DocumentKind::Cv,
            source: PathBuf::from("fixture.typ"),
            output: PathBuf::from("fixture.pdf"),
            inputs,
            expected_pages: 4,
        }
    }

    fn summary_metric(actual: f64) -> Value {
        json!({
            "kind": "cv-summary",
            "id": "cv.summary.1",
            "text": "evidence",
            "actual_fill": actual,
            "min_fill": 60,
            "target_fill": 82,
            "max_fill": 100,
        })
    }

    fn repo_cv_spec() -> DocumentSpec {
        cv_spec("cvl/de-ch/application.toml")
    }

    fn metric(kind: &str, identifier: &str) -> Value {
        json!({"kind": kind, "id": identifier})
    }

    fn metric_set(paragraph_lengths: &[usize]) -> Vec<Value> {
        let mut metrics = paragraph_lengths
            .iter()
            .enumerate()
            .flat_map(|(paragraph, length)| {
                (1..=*length).map(move |line| {
                    metric("cl-body", &format!("cl.paragraph.{}.{line}", paragraph + 1))
                })
            })
            .collect::<Vec<_>>();
        metrics
            .extend((1..=5).map(|index| metric("cl-highlight", &format!("cl.highlight.{index}"))));
        metrics.push(metric("cl-vertical-gap", "cl.vertical-gap"));
        metrics.push(metric("cl-highlight-center", "cl.highlight-center"));
        metrics
    }

    #[test]
    fn closing_line_spill_renders_without_wrapping() {
        // A closing line at 100.8% (inside the 102 maximum) must stay one
        // visual line: exact-width boxes spill into the margin instead of
        // re-wrapping, which previously added a sixth summary line and
        // overflowed the page despite green metrics. Five repeated spill
        // lines fit one 60mm page exactly; with auto-width boxes they wrap
        // to ten lines over two pages.
        let engine = ctypst::Engine::builder()
            .root(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .fonts(ctypst::fonts::documents())
            .build()
            .unwrap();
        let spill = "Damit unterstütze ich Leverage Experts pragmatisch in Performance-, Portfolio- und Transformationsmandaten.";
        let source = format!(
            "#import \"/.agent/typst/line-contract.typ\": measured-lines\n\
             #import \"/.agent/typst/styles/document.typ\": document-style\n\
             #show: document-style.with(locale: \"de-ch\")\n\
             #set page(height: 60mm)\n\
             #set text(hyphenate: false)\n\
             #let spill = \"{spill}\"\n\
             #measured-lines(\"t\", \"x\", range(5).map(i => (text: spill, min_fill: 60, target_fill: 82, max_fill: 102)), exact-width: true)"
        );
        let output = engine
            .compile(
                ctypst::CompileRequest::new("spill.typ")
                    .source_file("spill.typ", source)
                    .pages(ctypst::PageConstraint::Exactly(1)),
            )
            .unwrap();
        let metrics = ctypst::query_json(&output.document, "ccvl-line").unwrap();
        assert_eq!(metrics.len(), 5);
        for metric in &metrics {
            let fill = metric["actual_fill"].as_f64().unwrap();
            assert!(
                fill > 100.0 && fill <= 102.0,
                "want a real spill, got {fill}"
            );
        }
    }

    #[test]
    fn paragraph_closing_spill_renders_without_wrapping() {
        // Cover-letter closing lines share the 102 maximum, so a spill
        // with too few spaces to justify away must still stay one visual
        // line: three paragraphs with a short line plus a 102.0% closing
        // line fit one 60mm page; with flowing text the closings re-wrap
        // to nine lines over two pages.
        let engine = ctypst::Engine::builder()
            .root(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .fonts(ctypst::fonts::documents())
            .build()
            .unwrap();
        let short = "Kurz und gut geschrieben steht hier.";
        let spill = "Donaudampfschifffahrtsgesellschaftskapitän Gioacchino Rossini encountered extraordinary circumstances daily.";
        let source = format!(
            "#import \"/.agent/typst/line-contract.typ\": measured-paragraph\n\
             #import \"/.agent/typst/styles/document.typ\": document-style\n\
             #show: document-style.with(locale: \"de-ch\")\n\
             #set page(height: 60mm)\n\
             #set text(hyphenate: false)\n\
             #let short = \"{short}\"\n\
             #let spill = \"{spill}\"\n\
             #for p in range(3) {{\n\
             block(breakable: false)[#measured-paragraph(\"t.\" + str(p), \"x\", ((text: short, min_fill: 1, target_fill: 82, max_fill: 100), (text: spill, min_fill: 60, target_fill: 82, max_fill: 102)), justify: true)]\n\
             }}"
        );
        let output = engine
            .compile(
                ctypst::CompileRequest::new("spill-cl.typ")
                    .source_file("spill-cl.typ", source)
                    .pages(ctypst::PageConstraint::Exactly(1)),
            )
            .unwrap();
        let metrics = ctypst::query_json(&output.document, "ccvl-line").unwrap();
        assert_eq!(metrics.len(), 6);
        for metric in metrics
            .iter()
            .filter(|metric| metric["max_fill"].as_f64() == Some(102.0))
        {
            let fill = metric["actual_fill"].as_f64().unwrap();
            assert!(
                fill > 100.0 && fill <= 102.0,
                "want a real spill, got {fill}"
            );
        }
    }

    #[test]
    fn underfill_and_overflow_are_both_failures_for_any_unit() {
        let base = json!({"min_fill": 60, "target_fill": 80, "max_fill": 95});
        let mut metric = base.clone();
        metric["actual_fill"] = json!(59.9);
        assert_eq!(violation(&metric).unwrap(), Some("too short"));
        metric["actual_fill"] = json!(80.0);
        assert_eq!(violation(&metric).unwrap(), None);
        metric["actual_fill"] = json!(95.1);
        assert_eq!(violation(&metric).unwrap(), Some("too long"));

        metric["unit"] = json!("pt");
        metric["min_fill"] = json!(12);
        metric["target_fill"] = json!(20);
        metric["max_fill"] = json!(30);
        metric["actual_fill"] = json!(11.9);
        assert_eq!(violation(&metric).unwrap(), Some("too short"));
        metric["actual_fill"] = json!(24.1);
        assert_eq!(violation(&metric).unwrap(), None);
        metric["actual_fill"] = json!(30.1);
        assert_eq!(violation(&metric).unwrap(), Some("too long"));
    }

    #[test]
    fn cover_letter_metric_set_requires_structure_and_layout_metrics() {
        let workspace = workspace();
        let spec = cover_letter_spec();
        let complete = metric_set(&[3, 6, 6, 5, 5, 3]);
        validate_metric_set(&workspace, &spec, &complete).unwrap();
        assert!(
            preference_warnings(&workspace, &spec, &complete)
                .unwrap()
                .is_empty()
        );

        let dispreferred = metric_set(&[3, 5, 6, 5, 5, 3]);
        validate_metric_set(&workspace, &spec, &dispreferred).unwrap();
        let warnings = preference_warnings(&workspace, &spec, &dispreferred).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("paragraphs 2–3 use 11 lines"));

        let shorter_close = metric_set(&[3, 5, 5, 5, 5, 2]);
        validate_metric_set(&workspace, &spec, &shorter_close).unwrap();
        assert_eq!(
            preference_warnings(&workspace, &spec, &shorter_close).unwrap(),
            [
                "fixture: paragraph 6 uses 2 lines; accepted, but 3 is preferred to mirror paragraph 1"
            ]
        );

        for missing_kind in ["cl-vertical-gap", "cl-highlight-center"] {
            let incomplete = complete
                .iter()
                .filter(|item| item["kind"].as_str() != Some(missing_kind))
                .cloned()
                .collect::<Vec<_>>();
            let error = validate_metric_set(&workspace, &spec, &incomplete)
                .unwrap_err()
                .to_string();
            assert!(error.contains("vertical-gap and one highlight-position"));
        }
    }

    #[test]
    fn summary_lines_never_fail_line_failure() {
        let spec = repo_cv_spec();
        let thin = summary_metric(9.2);
        assert!(line_failure(&spec, 0, &thin).unwrap().is_none());
        let spill = summary_metric(100.8);
        assert!(line_failure(&spec, 0, &spill).unwrap().is_none());
    }

    #[test]
    fn summary_counsel_fails_thin_and_intolerable_spill() {
        let workspace = workspace();
        let spec = repo_cv_spec();
        let failures = summary_failures(&workspace, &spec, &[summary_metric(9.2)]).unwrap();
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("allow_thin"));
        assert!(
            summary_failures(&workspace, &spec, &[summary_metric(100.8)])
                .unwrap()
                .is_empty()
        );
        let failures = summary_failures(&workspace, &spec, &[summary_metric(102.1)]).unwrap();
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("closing-line maximum"));
    }

    #[test]
    fn summary_counsel_notes_allowed_thin_and_tolerated_spill() {
        let workspace = workspace();
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("ccvl.json"),
            "{\"documents\":{\"cv\":{\"summary_fill\":{\"minimum\":60,\"target\":82,\"maximum\":100}}},\"last_line_maximum\":102}",
        )
        .unwrap();
        std::fs::write(
            directory.path().join("record.toml"),
            "[cv]\nsummary = \"Evidence.\"\nallow_thin = true\n",
        )
        .unwrap();
        let allowed = Workspace::at(directory.path()).unwrap();
        let spec = cv_spec("record.toml");
        let warnings = preference_warnings(&allowed, &spec, &[summary_metric(9.2)]).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("explicitly wanted"));
        let warnings =
            preference_warnings(&workspace, &repo_cv_spec(), &[summary_metric(100.8)]).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("past the block edge"));
        let warnings =
            preference_warnings(&workspace, &repo_cv_spec(), &[summary_metric(82.0)]).unwrap();
        assert!(warnings.is_empty());
    }

    fn cover_letter_spec_with_application(application: &str) -> DocumentSpec {
        let mut inputs = std::collections::BTreeMap::new();
        inputs.insert("application".to_owned(), application.to_owned());
        DocumentSpec {
            name: "fixture".to_owned(),
            kind: DocumentKind::CoverLetter,
            source: PathBuf::from("fixture.typ"),
            output: PathBuf::from("fixture.pdf"),
            inputs,
            expected_pages: 1,
        }
    }

    #[test]
    fn empty_recipient_name_warns_but_stays_valid() {
        let workspace = workspace();
        // Showcase records ship with an empty recipient: generic salutation
        // stays valid, but measurement must surface a visible advisory.
        for locale in ["cvl/de-ch/application.toml", "cvl/en-ch/application.toml"] {
            let spec = cover_letter_spec_with_application(locale);
            let warnings = recipient_warnings(&workspace, &spec).unwrap();
            assert_eq!(warnings.len(), 1, "locale: {locale}");
            assert!(warnings[0].contains("job.cl_recipient.name is empty"));
            assert!(warnings[0].contains("generic salutation"));
            let metrics = metric_set(&[3, 6, 6, 5, 5, 3]);
            validate_metric_set(&workspace, &spec, &metrics).unwrap();
            let warnings = preference_warnings(&workspace, &spec, &metrics).unwrap();
            assert!(
                warnings
                    .iter()
                    .any(|warning| warning.contains("generic salutation")),
                "locale: {locale}"
            );
        }
        // Fixtures without an application input stay silent so metric-set
        // tests keep asserting exact warning counts.
        assert!(
            recipient_warnings(&workspace, &cover_letter_spec())
                .unwrap()
                .is_empty()
        );
        // CV specs never carry the salutation counsel.
        assert!(
            recipient_warnings(&workspace, &repo_cv_spec())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn typst_salutation_helper_keeps_only_the_last_token() {
        // Typst-level coverage for the shared helper: the same edge cases
        // as the Rust mirror must hold inside the renderer.
        let engine = ctypst::Engine::builder()
            .root(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .fonts(ctypst::fonts::documents())
            .build()
            .unwrap();
        let source = "#import \"/.agent/typst/application.typ\": salutation-last-name, de-salutation\n\
            #assert(salutation-last-name(\"Dr. Jane Doe\") == \"Doe\", message: \"title prefix\")\n\
            #assert(salutation-last-name(\"Ms Test Person\") == \"Person\", message: \"multi-token\")\n\
            #assert(salutation-last-name(\"Madonna\") == \"Madonna\", message: \"single token\")\n\
            #assert(salutation-last-name(\"Anne-Marie Müller-Schmidt\") == \"Müller-Schmidt\", message: \"hyphenated\")\n\
            #assert(salutation-last-name(\"  Jane   Doe  \") == \"Doe\", message: \"padded\")\n\
            #assert(salutation-last-name(\"\") == \"\", message: \"empty\")\n\
            #assert(salutation-last-name(\"   \") == \"\", message: \"whitespace\")\n\
            #assert(de-salutation(\"Frau Dr. Müller\", region: \"ch\") == \"Sehr geehrte Frau Dr. Müller\", message: \"ch titled\")\n\
            #assert(de-salutation(\"Herr Müller\", region: \"ch\") == \"Sehr geehrter Herr Müller\", message: \"ch plain\")\n\
            #assert(de-salutation(\"Frau Müller\", region: \"li\") == \"Sehr geehrte Frau Müller\", message: \"li no comma\")\n\
            #assert(de-salutation(\"Frau Müller\", region: \"de\") == \"Sehr geehrte Frau Müller,\", message: \"de comma\")\n\
            #assert(de-salutation(\"Herr Müller\", region: \"at\") == \"Sehr geehrter Herr Müller,\", message: \"at comma\")\n\
            #assert(de-salutation(\"Herr Prof. Dr. Müller\", region: \"ch\") == \"Sehr geehrter Herr Professor Müller\", message: \"professor wins\")\n\
            #assert(de-salutation(\"\", region: \"ch\") == \"Sehr geehrte Damen und Herren\", message: \"ch generic\")\n\
            #assert(de-salutation(\"\", region: \"de\") == \"Sehr geehrte Damen und Herren,\", message: \"de generic\")\n\
            #assert(de-salutation(\"Jane Doe\", region: \"ch\") == \"Sehr geehrte Damen und Herren\", message: \"no honorific\")\n\
            #assert(de-salutation(\"Hr. Müller\", region: \"ch\") == \"Sehr geehrte Damen und Herren\", message: \"abbreviation rejected\")\n\
            Hello";
        engine
            .compile(
                ctypst::CompileRequest::new("salutation.typ")
                    .source_file("salutation.typ", source)
                    .pages(ctypst::PageConstraint::Exactly(1)),
            )
            .unwrap();
    }
}
