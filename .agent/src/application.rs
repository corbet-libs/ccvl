use anyhow::{Context, Result, bail, ensure};
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

/// Greeting rules live in the `cgreet` library
/// (`https://github.com/corbet-labs/cgreet`, mirrored for the renderer in
/// `.agent/typst/application.typ`) and are re-exported here so existing paths
/// keep working.
pub use cgreet::{
    Region, de_honorific_warning, de_salutation, recipient_salutation_warning,
    salutation_honorific, salutation_last_name, salutation_surname, salutation_titles,
};

/// Resolve the render style for an application record.
///
/// `options.style` names one entry of the `styles` section in `ccvl.json`
/// (one Typst renderer plus one TOML knob file below
/// `.agent/typst/styles/`). Records written before styles existed omit the
/// field — as does an empty string — and render with the manifest default
/// (`harvard`). Unknown names fail with the available list.
pub fn resolve_style(workspace: &Workspace, application: &Value, location: &str) -> Result<String> {
    let manifest = workspace.read_json("ccvl.json")?;
    let styles = manifest
        .pointer("/styles")
        .context("ccvl.json has no styles section")?;
    let default = styles
        .get("default")
        .and_then(Value::as_str)
        .context("ccvl.json styles.default is missing")?;
    let available = styles
        .get("available")
        .and_then(Value::as_array)
        .context("ccvl.json styles.available is missing")?;
    let available = available
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .context("ccvl.json styles.available must be style names")?;
    ensure!(
        !available.is_empty() && available.contains(&default),
        "ccvl.json: default style {default:?} is not available"
    );
    if let Some(value) = application.pointer("/options/style") {
        ensure!(
            value.is_string(),
            "{location}.options.style must be a style name"
        );
    }
    let raw = application
        .pointer("/options/style")
        .and_then(Value::as_str)
        .unwrap_or("");
    let resolved = if raw.is_empty() { default } else { raw };
    ensure!(
        available.contains(&resolved),
        "{location}: unknown style {raw:?}; expected one of {} (set options.style in {location})",
        available.join(", ")
    );
    Ok(resolved.to_owned())
}

pub fn validate_all(workspace: &Workspace) -> Result<()> {
    let mut candidates = vec![
        workspace.path(".agent/scaffolds/opportunity/application.toml"),
        workspace.path("cvl/de-ch/application.toml"),
        workspace.path("cvl/en-ch/application.toml"),
    ];
    let opportunities = workspace.path("opportunities");
    if opportunities.is_dir() {
        for organisation in std::fs::read_dir(&opportunities)? {
            let organisation = organisation?;
            if !organisation.file_type()?.is_dir() {
                continue;
            }
            for position in std::fs::read_dir(organisation.path())? {
                let position = position?;
                let record = position.path().join("application.toml");
                if record.is_file() {
                    candidates.push(record);
                }
            }
        }
    }
    candidates.sort();
    for path in candidates {
        let application = read_toml_value(&path)?;
        let template = path == workspace.path(".agent/scaffolds/opportunity/application.toml");
        validate_record(
            workspace,
            &application,
            &workspace.relative(&path)?.display().to_string(),
            !template,
        )?;
        let relative = workspace.relative(&path)?;
        if relative == std::path::Path::new("cvl/de-ch/application.toml")
            && application
                .pointer("/options/language")
                .and_then(Value::as_str)
                != Some("de-CH")
        {
            bail!("{}: expected de-CH language", relative.display());
        }
        if relative == std::path::Path::new("cvl/en-ch/application.toml")
            && application
                .pointer("/options/language")
                .and_then(Value::as_str)
                != Some("en-CH")
        {
            bail!("{}: expected en-CH language", relative.display());
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
    ensure_no_unknown(localized, &["de-CH", "en-CH"], location)?;
    for locale in ["de-CH", "en-CH"] {
        let table = localized
            .get(locale)
            .and_then(Value::as_object)
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
            "style",
        ],
        location,
    )?;
    let language = options
        .get("language")
        .and_then(Value::as_str)
        .context("options.language is missing")?;
    ensure!(
        ["", "de-CH", "en-CH"].contains(&language),
        "{location}.options.language: expected de-CH or en-CH"
    );
    let pages = options
        .get("pages")
        .and_then(Value::as_u64)
        .context("options.pages is missing")?;
    ensure!(
        [2, 3, 4].contains(&pages),
        "{location}.options.pages: expected 2, 3, or 4"
    );
    let generate_cl = options
        .get("generate_cl")
        .and_then(Value::as_bool)
        .context("options.generate_cl is not a boolean")?;
    options
        .get("application_date")
        .and_then(Value::as_str)
        .context("options.application_date is missing")?;
    resolve_style(workspace, application, location)?;

    let job = object_at(application, "/job")?;
    let mut allowed = JOB_FIELDS.to_vec();
    allowed.push("cl_recipient");
    ensure_no_unknown(job, &allowed, location)?;
    for field in JOB_FIELDS {
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

    let cv = object_at(application, "/cv")?;
    ensure_no_unknown(cv, &["summary", "allow_thin"], location)?;
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

    if !generate_cl {
        ensure!(
            object.get("cl").is_none(),
            "{location}.cl: a disabled cover letter may not retain hidden content"
        );
        return Ok(());
    }
    let cl = object_at(application, "/cl")?;
    ensure_no_unknown(cl, &["paragraphs", "highlights"], location)?;

    let contract = workspace.read_json("ccvl.json")?;
    let cl_contract = contract
        .pointer("/documents/cover_letter")
        .context("ccvl.json has no cover-letter contract")?;
    let paragraph_contracts = array_at(cl_contract, "/paragraphs")?;
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
    validate_count(
        total,
        cl_contract
            .pointer("/body_lines")
            .context("missing body line contract")?,
        &format!("{location}.cl.paragraphs"),
        "body lines",
    )?;
    for region in array_at(cl_contract, "/paragraph_regions")? {
        let numbers = array_at(region, "/paragraphs")?
            .iter()
            .map(|value| value.as_u64().context("invalid paragraph number"))
            .collect::<Result<Vec<_>>>()?;
        let start =
            usize::try_from(numbers.first().copied().context("empty paragraph region")?)? - 1;
        let end = usize::try_from(numbers.last().copied().context("empty paragraph region")?)?;
        validate_count(
            counts[start..end].iter().sum(),
            region,
            &format!("{location}.cl.paragraphs[{}:{}]", start + 1, end),
            "shared lines",
        )?;
    }

    let highlights = cl
        .get("highlights")
        .and_then(Value::as_array)
        .context("cl.highlights is not an array")?;
    let expected = usize::try_from(u64_at(cl_contract, "/highlights/count")?)?;
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
mod tests {
    use serde_json::json;

    use super::*;

    fn workspace() -> Workspace {
        Workspace::at(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    fn lines(count: usize) -> Vec<Value> {
        (0..count).map(|_| json!("evidence")).collect()
    }

    fn application(paragraph_lengths: &[usize]) -> Value {
        json!({
            "schema_version": 4,
            "revision": 0,
            "options": {
                "language": "de-CH",
                "pages": 4,
                "generate_cl": true,
                "application_date": "September 2026",
            },
            "job": {
                "id": "fixture",
                "title": "Fixture",
                "organization": "Fixture",
                "location": "Fixture",
                "source": "Fixture",
                "url": "Fixture",
                "description": "Fixture",
                "connections": "",
                "company_context": "",
                "notes": "",
                "cl_recipient": {
                    "name": "",
                    "title": "",
                    "company": "",
                    "address_line_1": "",
                    "address_line_2": "",
                },
            },
            "cv": {"summary": "Flowing evidence paragraph."},
            "cl": {
                "paragraphs": paragraph_lengths.iter().map(|length| lines(*length)).collect::<Vec<_>>(),
                "highlights": lines(5),
            },
        })
    }

    #[test]
    fn cv_only_application_is_valid_without_hidden_cover_letter_content() {
        let mut draft = application(&[3, 6, 6, 5, 5, 3]);
        draft.as_object_mut().unwrap().remove("cl");
        draft["options"]["generate_cl"] = json!(false);
        validate_record(&workspace(), &draft, "fixture", true).unwrap();

        draft["cl"] = json!({"paragraphs": [], "highlights": []});
        let error = validate_record(&workspace(), &draft, "fixture", true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("disabled cover letter"));
    }

    #[test]
    fn accepted_cover_letter_distributions_match_the_declared_regions() {
        let workspace = workspace();
        for lengths in [
            [3, 6, 6, 5, 5, 3],
            [3, 5, 7, 5, 5, 3],
            [3, 7, 5, 5, 5, 2],
            [3, 5, 5, 5, 5, 2],
            [3, 5, 6, 5, 5, 3],
        ] {
            validate_record(&workspace, &application(&lengths), "fixture", true).unwrap();
        }
    }

    #[test]
    fn fixed_individual_and_shared_line_budgets_are_enforced() {
        let workspace = workspace();
        for (lengths, expected) in [
            ([2, 6, 6, 5, 5, 3], "paragraphs[1]: expected 3–3 lines"),
            ([3, 4, 6, 5, 5, 3], "paragraphs[2]: expected 5–7 lines"),
            ([3, 6, 6, 5, 5, 4], "paragraphs[6]: expected 2–3 lines"),
            (
                [3, 5, 5, 6, 7, 2],
                "paragraphs[4:5]: expected 10–12 shared lines",
            ),
        ] {
            let error = validate_record(&workspace, &application(&lengths), "fixture", true)
                .unwrap_err()
                .to_string();
            assert!(error.contains(expected), "unexpected error: {error}");
        }
    }

    #[test]
    fn every_middle_paragraph_distribution_obeys_all_declared_bounds() {
        let workspace = workspace();
        for second in 4..=8 {
            for third in 4..=8 {
                for fourth in 4..=8 {
                    for fifth in 4..=8 {
                        let middle = [second, third, fourth, fifth];
                        let valid = middle.iter().all(|count| (5..=7).contains(count))
                            && (10..=12).contains(&(second + third))
                            && (10..=12).contains(&(fourth + fifth))
                            && (20..=22).contains(&middle.iter().sum::<usize>());
                        let lengths = [3, second, third, fourth, fifth, 3];
                        assert_eq!(
                            validate_record(&workspace, &application(&lengths), "fixture", true,)
                                .is_ok(),
                            valid,
                            "unexpected result for {middle:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn german_flowing_summary_with_special_characters_validates() {
        let workspace = workspace();
        let mut draft = application(&[3, 6, 6, 5, 5, 3]);
        draft["cv"]["summary"] = json!(
            "Mittelstandsmandate verbinden Finanzen, Betrieb und Technologie. \
             Ich vereine Portfolioanalyse, Corporate Finance und Transformation mit \
             praktischer Cloud-/KI-Umsetzung. Damit unterstütze ich Leverage Experts \
             pragmatisch in Performance-, Portfolio- und Transformationsmandaten. \
             GenAI bei CENVION | RAG-Suche, CHF 10 Mio., 20+ Jahre, für & mit."
        );
        validate_record(&workspace, &draft, "fixture", true).unwrap();
        draft["options"]["generate_cl"] = json!(false);
        draft.as_object_mut().unwrap().remove("cl");
        validate_record(&workspace, &draft, "fixture", true).unwrap();
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let mut draft = application(&[3, 6, 6, 5, 5, 3]);
        draft["job"]["smuggled"] = json!("nope");
        let error = validate_record(&workspace(), &draft, "fixture", true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown fields"));
    }

    #[test]
    fn empty_rendered_text_is_rejected() {
        let mut draft = application(&[3, 6, 6, 5, 5, 3]);
        draft["cv"]["summary"] = json!("  ");
        let error = validate_record(&workspace(), &draft, "fixture", true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("cannot be empty"));
        draft["cv"]["summary"] = json!("Flowing evidence paragraph.");
        draft["cl"]["highlights"][0] = json!("  ");
        let error = validate_record(&workspace(), &draft, "fixture", true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("cannot be empty"));
    }

    #[test]
    fn missing_recipient_name_warns_without_failing_validation() {
        // Empty/whitespace names stay valid (showcase target-neutral letters)
        // but produce a visible, non-blocking advisory.
        let draft = application(&[3, 6, 6, 5, 5, 3]);
        validate_record(&workspace(), &draft, "fixture", true).unwrap();
        let warning = recipient_salutation_warning(
            "fixture",
            draft["job"]["cl_recipient"]["name"].as_str().unwrap(),
        )
        .expect("empty showcase recipient must warn");
        assert!(warning.contains("job.cl_recipient.name is empty"));
        assert!(warning.contains("generic salutation"));
        assert!(recipient_salutation_warning("fixture", "Dr. Jane Doe").is_none());
        assert!(recipient_salutation_warning("fixture", "   ").is_some());
    }

    #[test]
    fn style_defaults_to_harvard_for_legacy_records() {
        // The fixture carries no options.style, like records written before
        // styles existed: validation accepts it and resolution yields the
        // manifest default.
        let workspace = workspace();
        let draft = application(&[3, 6, 6, 5, 5, 3]);
        validate_record(&workspace, &draft, "fixture", true).unwrap();
        assert_eq!(
            resolve_style(&workspace, &draft, "fixture").unwrap(),
            "harvard"
        );

        let mut empty = draft.clone();
        empty["options"]["style"] = json!("");
        validate_record(&workspace, &empty, "fixture", true).unwrap();
        assert_eq!(
            resolve_style(&workspace, &empty, "fixture").unwrap(),
            "harvard"
        );

        let mut compact = draft.clone();
        compact["options"]["style"] = json!("harvard-compact");
        validate_record(&workspace, &compact, "fixture", true).unwrap();
        assert_eq!(
            resolve_style(&workspace, &compact, "fixture").unwrap(),
            "harvard-compact"
        );
    }

    #[test]
    fn unknown_style_fails_with_available_list() {
        let workspace = workspace();
        let mut draft = application(&[3, 6, 6, 5, 5, 3]);
        draft["options"]["style"] = json!("nope");
        let error = validate_record(&workspace, &draft, "fixture", true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown style"), "unexpected error: {error}");
        assert!(error.contains("harvard"), "unexpected error: {error}");
        let error = resolve_style(&workspace, &draft, "fixture")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("harvard-compact"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn non_string_style_is_rejected() {
        let mut draft = application(&[3, 6, 6, 5, 5, 3]);
        draft["options"]["style"] = json!(3);
        let error = resolve_style(&workspace(), &draft, "fixture")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("options.style must be a style name"),
            "unexpected error: {error}"
        );
    }
}
