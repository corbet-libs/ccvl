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

/// One render leaf: a substyle in one locale, i.e. the directory
/// `<root>/<substyle>/<lang>/ch` holding `content.toml`, `strings.toml`,
/// `typst/`, and `pdf/`. `locale` is the record language (`de-ch`/`en-ch`);
/// the directory language is its first subtag (`de`/`en`).
pub struct StyleLeaf {
    pub document: &'static str,
    pub substyle: String,
    pub locale: &'static str,
    pub dir: std::path::PathBuf,
}

impl StyleLeaf {
    #[must_use]
    pub fn content(&self) -> std::path::PathBuf {
        self.dir.join("content.toml")
    }

    #[must_use]
    pub fn strings(&self) -> std::path::PathBuf {
        self.dir.join("strings.toml")
    }

    #[must_use]
    pub fn adapter(&self) -> std::path::PathBuf {
        self.dir.join("typst").join(if self.document == "cv" {
            "cv.typ"
        } else {
            "cl.typ"
        })
    }

    #[must_use]
    pub fn substyle_file(&self) -> std::path::PathBuf {
        self.dir
            .parent()
            .and_then(|path| path.parent())
            .expect("leaf dir has a substyle parent")
            .join("substyle.toml")
    }
}

struct StyleRegistry {
    root: String,
    substyles: Vec<String>,
    default: String,
}

/// Read one document family's registry: the discovery root from
/// `ccvl.json documents` plus its `style.toml` (`substyles`, `default_substyle`).
fn style_registry(workspace: &Workspace, document: &str) -> Result<StyleRegistry> {
    let manifest_key = if document == "cv" {
        "cv"
    } else {
        "cover_letter"
    };
    let manifest = workspace.read_json("ccvl.json")?;
    let root = manifest
        .pointer(&format!("/documents/{manifest_key}/root"))
        .and_then(Value::as_str)
        .with_context(|| format!("ccvl.json documents.{manifest_key}.root is missing"))?
        .to_owned();
    let style = workspace.read_toml_value(format!("{root}/style.toml"))?;
    let substyles = style
        .get("substyles")
        .and_then(Value::as_array)
        .with_context(|| format!("{root}/style.toml has no substyles list"))?
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .with_context(|| format!("{root}/style.toml substyles must be substyle names"))?
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let default = style
        .get("default_substyle")
        .and_then(Value::as_str)
        .with_context(|| format!("{root}/style.toml has no default_substyle"))?
        .to_owned();
    ensure!(
        !substyles.is_empty() && substyles.contains(&default),
        "{root}/style.toml: default substyle {default:?} is not listed"
    );
    Ok(StyleRegistry {
        root,
        substyles,
        default,
    })
}

/// Every render leaf of one document family: each `style.toml` substyle in
/// each supported locale. Fails when a leaf directory, its `substyle.toml`,
/// or one of its `content.toml`/`strings.toml`/adapter files is missing, so
/// a half-added substyle cannot render silently.
pub fn style_leaves(workspace: &Workspace, document: &'static str) -> Result<Vec<StyleLeaf>> {
    ensure!(
        document == "cv" || document == "cl",
        "unknown document family: {document}"
    );
    let registry = style_registry(workspace, document)?;
    let style = workspace.read_toml_value(format!("{}/style.toml", registry.root))?;
    let locales = style
        .get("supports_locales")
        .and_then(Value::as_array)
        .with_context(|| format!("{}/style.toml has no supports_locales", registry.root))?;
    let mut leaves = Vec::new();
    for substyle in &registry.substyles {
        let substyle_file = workspace.path(format!("{}/{substyle}/substyle.toml", registry.root));
        ensure!(
            substyle_file.is_file(),
            "substyle {substyle} is missing {}",
            substyle_file.display()
        );
        for locale in locales {
            let locale = locale.as_str().with_context(|| {
                format!("{}/style.toml locales must be locale names", registry.root)
            })?;
            let (language, dir) = match locale {
                "de-ch" => ("de-ch", format!("{}/{substyle}/de/ch", registry.root)),
                "en-ch" => ("en-ch", format!("{}/{substyle}/en/ch", registry.root)),
                _ => bail!(
                    "{}/style.toml supports unknown locale: {locale}",
                    registry.root
                ),
            };
            let leaf = StyleLeaf {
                document,
                substyle: substyle.clone(),
                locale: language,
                dir: workspace.path(&dir),
            };
            for path in [leaf.content(), leaf.strings(), leaf.adapter()] {
                ensure!(
                    path.is_file(),
                    "leaf {} is missing {}",
                    leaf.dir.display(),
                    path.display()
                );
            }
            leaves.push(leaf);
        }
    }
    Ok(leaves)
}

pub fn cv_leaves(workspace: &Workspace) -> Result<Vec<StyleLeaf>> {
    style_leaves(workspace, "cv")
}

pub fn cl_leaves(workspace: &Workspace) -> Result<Vec<StyleLeaf>> {
    style_leaves(workspace, "cl")
}

/// Family default substyle when a record selects nothing.
pub fn default_cv_substyle(workspace: &Workspace) -> Result<String> {
    Ok(style_registry(workspace, "cv")?.default)
}

/// Family default substyle when a record selects nothing.
pub fn default_cl_substyle(workspace: &Workspace) -> Result<String> {
    Ok(style_registry(workspace, "cl")?.default)
}

/// Read one document family's measurement contract from the style tree
/// (`<root>/contract.toml`) instead of the manifest.
pub fn document_contract(workspace: &Workspace, document: &str) -> Result<Value> {
    let manifest_key = if document == "cv" {
        "cv"
    } else {
        "cover_letter"
    };
    let manifest = workspace.read_json("ccvl.json")?;
    let root = manifest
        .pointer(&format!("/documents/{manifest_key}/root"))
        .and_then(Value::as_str)
        .with_context(|| format!("ccvl.json documents.{manifest_key}.root is missing"))?;
    workspace.read_toml_value(format!("{root}/contract.toml"))
}

/// Resolve the CV substyle for an application record.
///
/// `options.cv_substyle` names one entry of `cvl/cv/style.toml`.
/// An absent or empty selection uses the family default. Unknown names fail.
pub fn resolve_cv_substyle(
    workspace: &Workspace,
    application: &Value,
    location: &str,
) -> Result<String> {
    resolve_substyle(workspace, application, location, "cv")
}

/// Resolve the cover-letter substyle for an application record. See
/// [`resolve_cv_substyle`]; the default is `left-rule`.
pub fn resolve_cl_substyle(
    workspace: &Workspace,
    application: &Value,
    location: &str,
) -> Result<String> {
    resolve_substyle(workspace, application, location, "cl")
}

fn resolve_substyle(
    workspace: &Workspace,
    application: &Value,
    location: &str,
    document: &str,
) -> Result<String> {
    let registry = style_registry(workspace, document)?;
    let key = if document == "cv" {
        "cv_substyle"
    } else {
        "cl_substyle"
    };
    if let Some(value) = application.pointer(&format!("/options/{key}")) {
        ensure!(
            value.is_string(),
            "{location}.options.{key} must be a substyle name"
        );
    }
    let raw = application
        .pointer(&format!("/options/{key}"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if !raw.is_empty() {
        ensure!(
            registry.substyles.iter().any(|name| name == raw),
            "{location}: unknown {document} substyle {raw:?}; expected one of {} (set options.{key} in {location})",
            registry.substyles.join(", ")
        );
        return Ok(raw.to_owned());
    }
    Ok(registry.default)
}

pub fn validate_all(workspace: &Workspace) -> Result<()> {
    let mut candidates = vec![workspace.path(".agent/scaffolds/opportunity/application.toml")];
    // Every style leaf carries a full application record as its showcase
    // content; discovery (not a hardcoded locale list) keeps new leaves
    // covered.
    for leaf in cv_leaves(workspace)?
        .into_iter()
        .chain(cl_leaves(workspace)?)
    {
        candidates.push(leaf.content());
    }
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
        if let Some(leaf_locale) = leaf_locale(&relative) {
            ensure!(
                application
                    .pointer("/options/language")
                    .and_then(Value::as_str)
                    == Some(leaf_locale),
                "{}: expected {leaf_locale} language",
                relative.display()
            );
            // The style-major tree selects per document: the retired single
            // `options.style` key must not linger in showcase leaves, and a
            // leaf's own selection must name its own substyle (or stay empty
            // for the default).
            ensure!(
                application.pointer("/options/style").is_none(),
                "{}: retired options.style must be replaced by options.cv_substyle/options.cl_substyle",
                relative.display()
            );
            let (key, expected) = leaf_substyle(&relative).with_context(|| {
                format!(
                    "{}: cannot locate the enclosing substyle",
                    relative.display()
                )
            })?;
            if let Some(selected) = application
                .pointer(&format!("/options/{key}"))
                .and_then(Value::as_str)
            {
                ensure!(
                    selected.is_empty() || selected == expected,
                    "{}: options.{key} {selected:?} does not match this {expected} leaf",
                    relative.display()
                );
            }
        }
    }
    Ok(())
}

/// Record language expected by a showcase leaf path
/// (`cvl/cv|cl/<substyle>/<lang>/ch/content.toml`), if any.
fn leaf_locale(relative: &std::path::Path) -> Option<&'static str> {
    if !is_leaf_content(relative) {
        return None;
    }
    let mut parts = relative.components().rev();
    parts.next()?;
    parts.next()?;
    match parts.next()?.as_os_str().to_str()? {
        "de" => Some("de-ch"),
        "en" => Some("en-ch"),
        _ => None,
    }
}

/// Substyle selection key and enclosing substyle name for a showcase leaf
/// path, if the path is a leaf content record.
fn leaf_substyle(relative: &std::path::Path) -> Option<(&'static str, String)> {
    if !is_leaf_content(relative) {
        return None;
    }
    let mut parts = relative.components().rev();
    parts.next()?;
    parts.next()?;
    parts.next()?;
    let substyle = parts.next()?.as_os_str().to_str()?.to_owned();
    let document = parts.next()?.as_os_str().to_str()?;
    match document {
        "cv" => Some(("cv_substyle", substyle)),
        "cl" => Some(("cl_substyle", substyle)),
        _ => None,
    }
}

fn is_leaf_content(relative: &std::path::Path) -> bool {
    let mut parts = relative.components();
    let root = parts.next().and_then(|part| part.as_os_str().to_str());
    let document = parts.next().and_then(|part| part.as_os_str().to_str());
    matches!(root, Some("cvl")) && matches!(document, Some("cv" | "cl"))
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
    ensure_no_unknown(localized, &["de-ch", "en-ch"], location)?;
    for locale in ["de-ch", "en-ch"] {
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
            "cv_substyle",
            "cl_substyle",
        ],
        location,
    )?;
    let language = options
        .get("language")
        .and_then(Value::as_str)
        .context("options.language is missing")?;
    ensure!(
        ["", "de-ch", "en-ch"].contains(&language),
        "{location}.options.language: expected de-ch or en-ch"
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
    resolve_cv_substyle(workspace, application, location)?;
    resolve_cl_substyle(workspace, application, location)?;

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

    let cl_contract = document_contract(workspace, "cl")?;
    let paragraph_contracts = array_at(&cl_contract, "/paragraphs")?;
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
    for region in array_at(&cl_contract, "/paragraph_regions")? {
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
    let expected = usize::try_from(u64_at(&cl_contract, "/highlights/count")?)?;
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
                "language": "de-ch",
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
        let mut draft = application(&[3, 5, 5, 5, 5, 3]);
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
    fn strict_cover_letter_line_budgets_are_enforced() {
        // The strict 3|5|5|5|5|3 framework admits exactly one distribution:
        // the valid letter passes while every off-by-one in any paragraph
        // fails, which also implies the pair (10), central (20), and body
        // (26) totals without a separate region test.
        let workspace = workspace();
        validate_record(
            &workspace,
            &application(&[3, 5, 5, 5, 5, 3]),
            "fixture",
            true,
        )
        .unwrap();
        for (lengths, expected) in [
            ([2, 5, 5, 5, 5, 3], "paragraphs[1]: expected 3–3 lines"),
            ([4, 5, 5, 5, 5, 3], "paragraphs[1]: expected 3–3 lines"),
            ([3, 4, 5, 5, 5, 3], "paragraphs[2]: expected 5–5 lines"),
            ([3, 6, 5, 5, 5, 3], "paragraphs[2]: expected 5–5 lines"),
            ([3, 5, 4, 5, 5, 3], "paragraphs[3]: expected 5–5 lines"),
            ([3, 5, 6, 5, 5, 3], "paragraphs[3]: expected 5–5 lines"),
            ([3, 5, 5, 4, 5, 3], "paragraphs[4]: expected 5–5 lines"),
            ([3, 5, 5, 6, 5, 3], "paragraphs[4]: expected 5–5 lines"),
            ([3, 5, 5, 5, 4, 3], "paragraphs[5]: expected 5–5 lines"),
            ([3, 5, 5, 5, 6, 3], "paragraphs[5]: expected 5–5 lines"),
            ([3, 5, 5, 5, 5, 2], "paragraphs[6]: expected 3–3 lines"),
            ([3, 5, 5, 5, 5, 4], "paragraphs[6]: expected 3–3 lines"),
        ] {
            let error = validate_record(&workspace, &application(&lengths), "fixture", true)
                .unwrap_err()
                .to_string();
            assert!(error.contains(expected), "unexpected error: {error}");
        }
    }

    #[test]
    fn german_flowing_summary_with_special_characters_validates() {
        let workspace = workspace();
        let mut draft = application(&[3, 5, 5, 5, 5, 3]);
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
        let mut draft = application(&[3, 5, 5, 5, 5, 3]);
        draft["job"]["smuggled"] = json!("nope");
        let error = validate_record(&workspace(), &draft, "fixture", true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown fields"));
    }

    #[test]
    fn empty_rendered_text_is_rejected() {
        let mut draft = application(&[3, 5, 5, 5, 5, 3]);
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
        let draft = application(&[3, 5, 5, 5, 5, 3]);
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
    fn substyles_default_to_standard_and_left_rule() {
        // The fixture carries no selection keys, like records written before
        // per-document selection existed: validation accepts it and
        // resolution yields the family defaults.
        let workspace = workspace();
        let draft = application(&[3, 5, 5, 5, 5, 3]);
        validate_record(&workspace, &draft, "fixture", true).unwrap();
        assert_eq!(
            resolve_cv_substyle(&workspace, &draft, "fixture").unwrap(),
            "standard"
        );
        assert_eq!(
            resolve_cl_substyle(&workspace, &draft, "fixture").unwrap(),
            "left-rule"
        );

        let mut empty = draft.clone();
        empty["options"]["cv_substyle"] = json!("");
        empty["options"]["cl_substyle"] = json!("");
        validate_record(&workspace, &empty, "fixture", true).unwrap();
        assert_eq!(
            resolve_cv_substyle(&workspace, &empty, "fixture").unwrap(),
            "standard"
        );
        assert_eq!(
            resolve_cl_substyle(&workspace, &empty, "fixture").unwrap(),
            "left-rule"
        );

        let mut selected = draft.clone();
        selected["options"]["cv_substyle"] = json!("compact");
        selected["options"]["cl_substyle"] = json!("frame");
        validate_record(&workspace, &selected, "fixture", true).unwrap();
        assert_eq!(
            resolve_cv_substyle(&workspace, &selected, "fixture").unwrap(),
            "compact"
        );
        assert_eq!(
            resolve_cl_substyle(&workspace, &selected, "fixture").unwrap(),
            "frame"
        );
    }

    #[test]
    fn retired_style_selection_is_rejected() {
        let workspace = workspace();
        let mut draft = application(&[3, 5, 5, 5, 5, 3]);
        draft["options"]["style"] = json!("harvard");
        let error = validate_record(&workspace, &draft, "fixture", true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("style"), "unexpected error: {error}");
    }

    #[test]
    fn unknown_substyle_fails_with_available_list() {
        let workspace = workspace();
        let mut draft = application(&[3, 5, 5, 5, 5, 3]);
        draft["options"]["cv_substyle"] = json!("nope");
        let error = validate_record(&workspace, &draft, "fixture", true)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("unknown cv substyle"),
            "unexpected error: {error}"
        );
        assert!(error.contains("compact"), "unexpected error: {error}");
        let error = resolve_cv_substyle(&workspace, &draft, "fixture")
            .unwrap_err()
            .to_string();
        assert!(error.contains("standard"), "unexpected error: {error}");

        draft["options"]
            .as_object_mut()
            .unwrap()
            .remove("cv_substyle");
        draft["options"]["cl_substyle"] = json!("nope");
        let error = resolve_cl_substyle(&workspace, &draft, "fixture")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("unknown cl substyle"),
            "unexpected error: {error}"
        );
        assert!(error.contains("frame"), "unexpected error: {error}");
    }

    #[test]
    fn style_leaves_cover_every_substyle_and_locale() {
        let workspace = workspace();
        let cv = cv_leaves(&workspace).unwrap();
        assert_eq!(cv.len(), 4);
        for (substyle, locale) in [
            ("standard", "de-ch"),
            ("standard", "en-ch"),
            ("compact", "de-ch"),
            ("compact", "en-ch"),
        ] {
            let leaf = cv
                .iter()
                .find(|leaf| leaf.substyle == substyle && leaf.locale == locale)
                .unwrap_or_else(|| panic!("missing CV leaf {substyle} {locale}"));
            assert!(leaf.content().is_file());
            assert!(leaf.strings().is_file());
            assert!(leaf.adapter().is_file());
            assert!(leaf.substyle_file().is_file());
        }
        let cl = cl_leaves(&workspace).unwrap();
        assert_eq!(cl.len(), 4);
        for (substyle, locale) in [
            ("left-rule", "de-ch"),
            ("left-rule", "en-ch"),
            ("frame", "de-ch"),
            ("frame", "en-ch"),
        ] {
            assert!(
                cl.iter()
                    .any(|leaf| leaf.substyle == substyle && leaf.locale == locale),
                "missing cover-letter leaf {substyle} {locale}"
            );
        }
    }

    #[test]
    fn non_string_substyle_is_rejected() {
        let mut draft = application(&[3, 5, 5, 5, 5, 3]);
        draft["options"]["cv_substyle"] = json!(3);
        let error = resolve_cv_substyle(&workspace(), &draft, "fixture")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("options.cv_substyle must be a substyle name"),
            "unexpected error: {error}"
        );
        draft["options"]
            .as_object_mut()
            .unwrap()
            .remove("cv_substyle");
        draft["options"]["cl_substyle"] = json!(3);
        let error = resolve_cl_substyle(&workspace(), &draft, "fixture")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("options.cl_substyle must be a substyle name"),
            "unexpected error: {error}"
        );
    }
}
