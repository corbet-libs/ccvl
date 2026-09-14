use std::path::Path;

use anyhow::{Context, Result, ensure};
use regex::Regex;
use serde_json::{Map, Value};

use crate::workspace::{Workspace, read_toml_value};

const RECORD_VERSION: u64 = 4;
const PROFILE_VERSION: u64 = 1;
const STATIONS_VERSION: u64 = 1;

const JOB_FIELDS: &[&str] = &[
    "id",
    "title",
    "organization",
    "location",
    "source",
    "url",
    "description",
    "connections",
    "company_context",
    "notes",
];

/// Opportunity records keep only what code or the document build reads plus
/// the role identity a human needs at a glance. Everything else about the
/// position lives in the sibling `posting.md`, the one human-readable
/// position reference. Showcase leaves keep the full table.
const OPPORTUNITY_JOB_FIELDS: &[&str] = &["id", "title", "organization", "location"];

const RECIPIENT_FIELDS: &[&str] = &[
    "name",
    "title",
    "company",
    "address_line_1",
    "address_line_2",
];

const PROFILE_FIELDS: &[&str] = &[
    "name",
    "email",
    "phone_label",
    "phone_href",
    "location",
    "languages",
    "linkedin",
    "website",
];

const PROFILE_TOP: &[&str] = &[
    "schema_version",
    "name",
    "email",
    "phone_label",
    "phone_href",
    "location",
    "languages",
    "linkedin",
    "website",
    "localized",
];

/// Correspondence rules live in cletter and its subordinate libraries.
/// The renderer uses the same upstream facade's pinned Typst sources.
pub use cletter::{
    Region, de_honorific_warning, de_salutation, recipient_salutation_warning,
    salutation_honorific, salutation_last_name, salutation_surname, salutation_titles,
};

/// Non-mutating correspondence guidance for a selected explicit locale.
/// Reviewers decide whether a match is prose or protected original content.
pub fn locale_conventions(locale: &str) -> Result<Value> {
    let locale = crate::styles::normalize_locale(locale)?;
    Ok(serde_json::json!({
        "locale": locale,
        "provider": "cletter",
        "orthography_replacements": cletter::orthography_replacements(&locale),
        "scope": "Apply to authored prose only. Preserve names, exact quotes, URLs, source material and explicit user choices.",
    }))
}

pub use crate::styles::{Selection, StyleLeaf};

pub fn cv_leaves(workspace: &Workspace) -> Result<Vec<StyleLeaf>> {
    crate::styles::leaves(workspace, "cv")
}

pub fn cl_leaves(workspace: &Workspace) -> Result<Vec<StyleLeaf>> {
    crate::styles::leaves(workspace, "cl")
}

pub fn default_cv_substyle(workspace: &Workspace) -> Result<String> {
    Ok(crate::styles::selection(workspace, "cv", None, None)?.substyle)
}

pub fn default_cl_substyle(workspace: &Workspace) -> Result<String> {
    Ok(crate::styles::selection(workspace, "cl", None, None)?.substyle)
}

/// The default style's contract, used by profile commands without a selection.
pub fn document_contract(workspace: &Workspace, document: &str) -> Result<Value> {
    crate::styles::contract(
        workspace,
        document,
        &crate::styles::default_style(workspace, document)?,
    )
}

pub fn resolve_cv_substyle(
    workspace: &Workspace,
    application: &Value,
    location: &str,
) -> Result<String> {
    Ok(crate::styles::record_selection(workspace, "cv", application, location)?.substyle)
}

pub fn resolve_cl_substyle(
    workspace: &Workspace,
    application: &Value,
    location: &str,
) -> Result<String> {
    Ok(crate::styles::record_selection(workspace, "cl", application, location)?.substyle)
}

pub fn validate_all(workspace: &Workspace) -> Result<()> {
    validate_record(
        workspace,
        &serde_json::to_value(crate::opportunity::blank_record(workspace)?)?,
        ".agent/scaffolds/opportunity/application.toml",
        false,
    )?;
    for leaf in cv_leaves(workspace)?
        .into_iter()
        .chain(cl_leaves(workspace)?)
    {
        let record = crate::content::read_record(workspace, leaf.content())?;
        let relative = workspace.relative(&leaf.content())?.display().to_string();
        validate_document_record(workspace, &record, &relative, true, leaf.document)?;
        ensure!(
            record.pointer("/options/language").and_then(Value::as_str)
                == Some(leaf.locale.as_str()),
            "{relative}: expected {} language",
            leaf.locale
        );
        let selected =
            crate::styles::record_selection(workspace, leaf.document, &record, &relative)?;
        ensure!(
            selected == leaf.selection(),
            "{relative}: style/substyle selection does not match this leaf"
        );
    }
    let opportunities = workspace.path("opportunities");
    if opportunities.is_dir() {
        for organisation in std::fs::read_dir(opportunities)? {
            let organisation = organisation?;
            if !organisation.file_type()?.is_dir() {
                continue;
            }
            for position in std::fs::read_dir(organisation.path())? {
                let record = position?.path().join("application.toml");
                if record.is_file() {
                    let relative = workspace.relative(&record)?.display().to_string();
                    validate_record(
                        workspace,
                        &crate::content::read_record(workspace, &record)?,
                        &relative,
                        true,
                    )?;
                }
            }
        }
    }
    Ok(())
}

pub fn validate_profiles(workspace: &Workspace) -> Result<()> {
    for relative in [
        ".agent/scaffolds/interview/profile.toml",
        "cvl/profile.toml",
    ] {
        let profile = read_toml_value(&workspace.path(relative))?;
        validate_profile(&profile, relative)?;
    }
    Ok(())
}

fn validate_profile(profile: &Value, location: &str) -> Result<()> {
    let object = object_at(profile, "")?;
    ensure!(
        u64_at(profile, "/schema_version")? == PROFILE_VERSION,
        "{location}: unsupported profile schema version"
    );
    ensure_no_unknown(object, PROFILE_TOP, location)?;
    for field in PROFILE_FIELDS {
        string_at(profile, &format!("/{field}"), location)?;
    }
    let localized = object_at(profile, "/localized")?;
    for (locale, value) in localized {
        ensure!(
            crate::styles::normalize_locale(locale)? == *locale,
            "{location}.localized.{locale}: expected canonical language-country"
        );
        let table = value
            .as_object()
            .with_context(|| format!("{location}.localized.{locale} is missing"))?;
        ensure_no_unknown(table, &["nationality_and_permit", "availability"], location)?;
        for field in ["nationality_and_permit", "availability"] {
            table
                .get(field)
                .and_then(Value::as_str)
                .with_context(|| format!("{location}.localized.{locale}.{field} is missing"))?;
        }
    }
    Ok(())
}

pub fn validate_station_files(workspace: &Workspace) -> Result<()> {
    for relative in [
        ".agent/scaffolds/interview/stations.toml",
        "interview/stations.toml",
    ] {
        let stations = read_toml_value(&workspace.path(relative))?;
        ensure!(
            u64_at(&stations, "/schema_version")? == STATIONS_VERSION,
            "{relative}: unsupported stations schema version"
        );
        array_at(&stations, "/stations")
            .with_context(|| format!("{relative}: missing stations array"))?;
    }
    Ok(())
}

pub fn validate_record(
    workspace: &Workspace,
    application: &Value,
    location: &str,
    require_text: bool,
) -> Result<()> {
    validate_record_scope(workspace, application, location, require_text, None)
}

/// Validate shared record metadata and only the requested document's rules.
/// Full records and workspace checks still validate every enabled document.
pub fn validate_document_record(
    workspace: &Workspace,
    application: &Value,
    location: &str,
    require_text: bool,
    document: &str,
) -> Result<()> {
    ensure!(
        matches!(document, "cv" | "cl"),
        "unknown document: {document}"
    );
    validate_record_scope(
        workspace,
        application,
        location,
        require_text,
        Some(document),
    )
}

fn validate_record_scope(
    workspace: &Workspace,
    application: &Value,
    location: &str,
    require_text: bool,
    document: Option<&str>,
) -> Result<()> {
    let object = object_at(application, "")?;
    ensure_no_unknown(
        object,
        &["schema_version", "revision", "options", "job", "cv", "cl"],
        location,
    )?;
    ensure!(
        u64_at(application, "/schema_version")? == RECORD_VERSION,
        "{location}: unsupported application schema version"
    );
    u64_at(application, "/revision")?;

    let options = object_at(application, "/options")?;
    ensure_no_unknown(
        options,
        &[
            "language",
            "pages",
            "generate_cl",
            "application_date",
            "cv_style",
            "cl_style",
            "cv_substyle",
            "cl_substyle",
            "cl_pages",
            "cv_paper",
            "cl_paper",
        ],
        location,
    )?;
    let language = options
        .get("language")
        .and_then(Value::as_str)
        .context("options.language is missing")?;
    if !language.is_empty() {
        ensure!(
            crate::styles::normalize_locale(language)? == language,
            "{location}.options.language: expected canonical language-country"
        );
    }
    let pages = options
        .get("pages")
        .and_then(Value::as_u64)
        .context("options.pages is missing")?;
    let generate_cl = options
        .get("generate_cl")
        .and_then(Value::as_bool)
        .context("options.generate_cl is not a boolean")?;
    options
        .get("application_date")
        .and_then(Value::as_str)
        .context("options.application_date is missing")?;

    let job = object_at(application, "/job")?;
    // Location strings use the workspace-relative display form; normalize
    // separators so the opportunities/ gate holds on every platform.
    let normalized = location.replace('\\', "/");
    let opportunity = normalized.starts_with("opportunities/");
    let required: &[&str] = if opportunity {
        OPPORTUNITY_JOB_FIELDS
    } else {
        JOB_FIELDS
    };
    let mut allowed = required.to_vec();
    allowed.push("cl_recipient");
    ensure_no_unknown(job, &allowed, location)?;
    for field in required {
        job.get(*field)
            .and_then(Value::as_str)
            .with_context(|| format!("{location}.job.{field} is missing"))?;
    }
    let id = job
        .get("id")
        .and_then(Value::as_str)
        .context("job.id is missing")?;
    ensure!(
        Regex::new(r"^[A-Za-z0-9_-]*$")?.is_match(id),
        "{location}.job.id: expected letters, numbers, hyphens, or underscores"
    );
    ensure!(
        !require_text || !id.trim().is_empty(),
        "{location}.job.id is required"
    );
    let recipient = job
        .get("cl_recipient")
        .and_then(Value::as_object)
        .context("job.cl_recipient is missing")?;
    ensure_no_unknown(recipient, RECIPIENT_FIELDS, location)?;
    for field in RECIPIENT_FIELDS {
        recipient
            .get(*field)
            .and_then(Value::as_str)
            .with_context(|| format!("{location}.job.cl_recipient.{field} is missing"))?;
    }
    if opportunity {
        let posting = Path::new(&normalized)
            .parent()
            .with_context(|| format!("{location}: opportunity path has no parent"))?;
        ensure!(
            workspace.path(posting.join("posting.md")).is_file(),
            "{location}: posting.md is missing; every opportunity keeps its human-readable position reference beside the record"
        );
    }

    if document != Some("cl") {
        let cv_selection = crate::styles::record_selection(workspace, "cv", application, location)?;
        let cv_style = crate::styles::definition(workspace, "cv", &cv_selection.style)?;
        if !language.is_empty() {
            crate::paper::select(
                cv_style.paper.as_ref(),
                language,
                crate::paper::record_choice(application, "cv")?,
            )?;
        }
        ensure!(
            cv_style.pages.contains(&usize::try_from(pages)?),
            "{location}.options.pages: unsupported by CV style {}",
            cv_selection.style
        );
        let cv_contract = crate::styles::contract(workspace, "cv", &cv_selection.style)?;
        let cv = object_at(application, "/cv")?;
        validate_content_fields(cv, &cv_contract, location)?;
        if cv_contract.get("summary_lines").is_some() {
            let summary = cv
                .get("summary")
                .and_then(Value::as_str)
                .context("cv.summary is missing")?;
            ensure!(
                !require_text || !summary.trim().is_empty(),
                "{location}.cv.summary: a rendered summary cannot be empty"
            );
            if let Some(allow_thin) = cv.get("allow_thin") {
                ensure!(
                    allow_thin.is_boolean(),
                    "{location}.cv.allow_thin must be a boolean"
                );
            }
        }
    }

    if document == Some("cv") {
        return Ok(());
    }

    if !generate_cl {
        ensure!(
            document != Some("cl"),
            "{location}: cover-letter generation is disabled"
        );
        ensure!(
            object.get("cl").is_none(),
            "{location}.cl: a disabled cover letter may not retain hidden content"
        );
        return Ok(());
    }
    let letter_selection = crate::styles::record_selection(workspace, "cl", application, location)?;
    let letter_style = crate::styles::definition(workspace, "cl", &letter_selection.style)?;
    if !language.is_empty() {
        crate::paper::select(
            letter_style.paper.as_ref(),
            language,
            crate::paper::record_choice(application, "cl")?,
        )?;
    }
    if let Some(pages) = options.get("cl_pages") {
        let pages = usize::try_from(
            pages
                .as_u64()
                .context("options.cl_pages must be a positive page count")?,
        )?;
        ensure!(
            letter_style.pages.contains(&pages),
            "{location}.options.cl_pages: unsupported by letter style {}",
            letter_selection.style
        );
    }

    let cl = object_at(application, "/cl")?;
    let letter_contract = crate::styles::contract(workspace, "cl", &letter_selection.style)?;
    validate_content_fields(cl, &letter_contract, location)?;
    if letter_contract.get("paragraphs").is_none() {
        return Ok(());
    }
    let paragraph_contracts = array_at(&letter_contract, "/paragraphs")?;
    let paragraphs = cl
        .get("paragraphs")
        .and_then(Value::as_array)
        .context("cl.paragraphs is not an array")?;
    ensure!(
        paragraphs.len() == paragraph_contracts.len(),
        "{location}.cl.paragraphs: expected {} paragraphs, found {}",
        paragraph_contracts.len(),
        paragraphs.len()
    );

    let mut counts = Vec::with_capacity(paragraphs.len());
    for (index, (paragraph, paragraph_contract)) in
        paragraphs.iter().zip(paragraph_contracts).enumerate()
    {
        let lines = paragraph
            .as_array()
            .with_context(|| format!("{location}.cl.paragraphs[{}] is not an array", index + 1))?;
        let bounds = paragraph_contract
            .get("lines")
            .context("missing paragraph bounds")?;
        let minimum = usize::try_from(u64_at(bounds, "/minimum")?)?;
        let maximum = usize::try_from(u64_at(bounds, "/maximum")?)?;
        ensure!(
            (minimum..=maximum).contains(&lines.len()),
            "{location}.cl.paragraphs[{}]: expected {minimum}–{maximum} lines, found {}",
            index + 1,
            lines.len()
        );
        counts.push(lines.len());
        for (line_index, line) in lines.iter().enumerate() {
            let text = line.as_str().with_context(|| {
                format!(
                    "{location}.cl.paragraphs[{}].lines[{}] is not text",
                    index + 1,
                    line_index + 1
                )
            })?;
            ensure!(
                !require_text || !text.trim().is_empty(),
                "{location}.cl.paragraphs[{}].lines[{}]: a rendered line cannot be empty",
                index + 1,
                line_index + 1
            );
        }
    }
    let total = counts.iter().sum::<usize>();
    if let Some(bounds) = letter_contract.get("body_lines") {
        validate_count(
            total,
            bounds,
            &format!("{location}.cl.paragraphs"),
            "body lines",
        )?;
    }
    for region in letter_contract
        .get("paragraph_regions")
        .map(|_| array_at(&letter_contract, "/paragraph_regions"))
        .transpose()?
        .into_iter()
        .flatten()
    {
        let numbers = array_at(region, "/paragraphs")?
            .iter()
            .map(|value| value.as_u64().context("invalid paragraph number"))
            .collect::<Result<Vec<_>>>()?;
        let start = usize::try_from(numbers.first().copied().context("empty paragraph region")?)?
            .checked_sub(1)
            .context("paragraph numbers start at 1")?;
        let end = usize::try_from(numbers.last().copied().context("empty paragraph region")?)?;
        ensure!(
            start < end && end <= counts.len(),
            "paragraph region is out of bounds"
        );
        validate_count(
            counts[start..end].iter().sum(),
            region,
            &format!("{location}.cl.paragraphs[{}:{}]", start + 1, end),
            "shared lines",
        )?;
    }

    if letter_contract.get("highlights").is_none() {
        return Ok(());
    }
    let highlights = cl
        .get("highlights")
        .and_then(Value::as_array)
        .context("cl.highlights is not an array")?;
    let expected = usize::try_from(u64_at(&letter_contract, "/highlights/count")?)?;
    ensure!(
        highlights.len() == expected,
        "{location}.cl.highlights: expected {expected} items, found {}",
        highlights.len()
    );
    for (index, highlight) in highlights.iter().enumerate() {
        let text = highlight
            .as_str()
            .with_context(|| format!("{location}.cl.highlights[{}] is not text", index + 1))?;
        ensure!(
            !require_text || !text.trim().is_empty(),
            "{location}.cl.highlights[{}]: a rendered highlight cannot be empty",
            index + 1
        );
    }
    Ok(())
}

fn validate_content_fields(
    object: &Map<String, Value>,
    contract: &Value,
    location: &str,
) -> Result<()> {
    if let Some(fields) = contract.get("content_fields").and_then(Value::as_array) {
        let fields = fields
            .iter()
            .map(|value| value.as_str().context("content_fields must contain names"))
            .collect::<Result<Vec<_>>>()?;
        ensure_no_unknown(object, &fields, location)?;
    }
    Ok(())
}

fn ensure_no_unknown(object: &Map<String, Value>, allowed: &[&str], location: &str) -> Result<()> {
    let unknown = object
        .keys()
        .filter(|key| !allowed.contains(&key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    ensure!(
        unknown.is_empty(),
        "{location}: unknown fields {}",
        unknown.join(", ")
    );
    Ok(())
}

fn validate_count(actual: usize, bounds: &Value, location: &str, label: &str) -> Result<()> {
    let minimum = usize::try_from(u64_at(bounds, "/minimum")?)?;
    let maximum = usize::try_from(u64_at(bounds, "/maximum")?)?;
    ensure!(
        (minimum..=maximum).contains(&actual),
        "{location}: expected {minimum}–{maximum} {label}, found {actual}"
    );
    Ok(())
}

pub(crate) fn array_at<'a>(value: &'a Value, pointer: &str) -> Result<&'a Vec<Value>> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .with_context(|| format!("missing array at {pointer}"))
}

pub(crate) fn object_at<'a>(value: &'a Value, pointer: &str) -> Result<&'a Map<String, Value>> {
    value
        .pointer(pointer)
        .and_then(Value::as_object)
        .with_context(|| format!("missing object at {pointer}"))
}

pub(crate) fn u64_at(value: &Value, pointer: &str) -> Result<u64> {
    value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .with_context(|| format!("missing integer at {pointer}"))
}

fn string_at<'a>(value: &'a Value, pointer: &str, location: &str) -> Result<&'a str> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .with_context(|| format!("{location}: missing text at {pointer}"))
}

#[cfg(test)]
mod tests;
