use std::collections::HashMap;
use std::sync::OnceLock;

use crate::render::{Compiler, DocumentKind, DocumentSpec, opportunity_specs};
use crate::workspace::Workspace;
use anyhow::{Context, Result, ensure};
use ctypst::Document;
use serde_json::Value;

pub fn cvl_specs(workspace: &Workspace) -> Result<Vec<DocumentSpec>> {
    let mut specs = Vec::new();
    for leaf in crate::styles::leaves(workspace, "cv")?
        .into_iter()
        .chain(crate::styles::leaves(workspace, "cl")?)
    {
        let pages = *leaf
            .pages
            .iter()
            .max()
            .context("style has no page presets")?;
        let mut spec = crate::render::cvl_spec(workspace, &leaf, pages)?;
        spec.inputs
            .insert("line-contracts".to_owned(), "report".to_owned());
        specs.push(spec);
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
    if is_summary(metric) && spec.contract.get("summary_fill").is_some() {
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
    let contract = &spec.contract;
    let fill = contract
        .pointer("/summary_fill")
        .context("CV contract has no summary fill")?;
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
        .context("CV contract has no closing-line maximum")?;
    let typst_path = spec
        .inputs
        .get("application")
        .context("summary spec has no application input")?;
    let record = crate::content::read_record(workspace, typst_path.trim_start_matches('/'))?;
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
    if spec.kind != DocumentKind::Cv || spec.contract.get("summary_fill").is_none() {
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
    _workspace: &Workspace,
    spec: &DocumentSpec,
    metrics: &[Value],
) -> Result<()> {
    let mut counts = HashMap::<&str, usize>::new();
    for metric in metrics {
        *counts.entry(string_field(metric, "kind")?).or_default() += 1;
    }
    if let Some(rules) = spec.contract.get("metric_rules").and_then(Value::as_array) {
        for rule in rules {
            let kind = rule
                .get("kind")
                .and_then(Value::as_str)
                .context("metric rule has no kind")?;
            let actual = counts.get(kind).copied().unwrap_or_default();
            let minimum =
                usize::try_from(rule.get("minimum").and_then(Value::as_u64).unwrap_or(0))?;
            let maximum = rule
                .get("maximum")
                .and_then(Value::as_u64)
                .map(usize::try_from)
                .transpose()?;
            ensure!(
                actual >= minimum && maximum.is_none_or(|maximum| actual <= maximum),
                "{}: measured {kind} count {actual} violates its style contract",
                spec.name
            );
        }
    }
    if let Some(paragraphs) = spec.contract.get("paragraphs").and_then(Value::as_array) {
        let counts = paragraph_counts(spec, metrics)?;
        for (index, (actual, declared)) in counts.iter().zip(paragraphs).enumerate() {
            let minimum = usize::try_from(
                declared
                    .pointer("/lines/minimum")
                    .and_then(Value::as_u64)
                    .context("paragraph minimum is missing")?,
            )?;
            let maximum = usize::try_from(
                declared
                    .pointer("/lines/maximum")
                    .and_then(Value::as_u64)
                    .context("paragraph maximum is missing")?,
            )?;
            ensure!(
                (minimum..=maximum).contains(actual),
                "{}: paragraph {} must use {minimum}–{maximum} lines, found {actual}",
                spec.name,
                index + 1
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
    if spec.contract.get("summary_fill").is_none() {
        return Ok(Vec::new());
    }
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
    let paragraph_total = spec
        .contract
        .get("paragraphs")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let mut counts = vec![0; paragraph_total];
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
            (1..=paragraph_total).contains(&paragraph),
            "{}: metric references an unknown paragraph",
            spec.name
        );
        counts[paragraph - 1] += 1;
    }
    Ok(counts)
}

/// Target-neutral salutation counsel for cover letters. The strict 3|5|5|5|5|3
/// framework has no accepted-but-dispreferred line totals: every deviation
/// from the exact paragraph, region, and body budgets fails validation, so
/// there is nothing to warn about beyond the recipient. The fused check gate
/// calls this for its error propagation even when it discards the advisories.
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
    // Validate the metric ids still reference known paragraphs so malformed
    // inputs fail here with a location instead of silently yielding no counsel.
    if spec.contract.get("paragraphs").is_none() {
        return Ok(Vec::new());
    }
    let _ = paragraph_counts(spec, metrics)?;
    recipient_warnings(workspace, spec)
}

/// Visible, non-blocking counsel when a cover letter has no recipient name.
/// The Typst renderer falls back to the formal salutation ("Dear Hiring
/// Manager," / "Sehr geehrte Damen und Herren"), which stays valid for the
/// target-neutral showcase; a tailored opportunity should name a person.
/// German records additionally warn when the name carries no parsable
/// Herr/Frau honorific. Missing application inputs (unit fixtures) yield no
/// warning so metric-set tests stay focused.
pub fn recipient_warnings(workspace: &Workspace, spec: &DocumentSpec) -> Result<Vec<String>> {
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
    let language = record
        .pointer("/options/language")
        .and_then(Value::as_str)
        .unwrap_or("");
    Ok(cletter::warnings(&spec.name, language, name))
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
mod tests;
