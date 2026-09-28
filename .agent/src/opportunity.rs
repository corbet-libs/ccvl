use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::workspace::Workspace;

pub fn record_path(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
    require_exists: bool,
) -> Result<PathBuf> {
    let pattern = Regex::new(r"^[a-z0-9]+(?:[-_][a-z0-9]+)*$")?;
    for (label, value) in [("organisation", organisation), ("position", position)] {
        if !pattern.is_match(value) {
            bail!("invalid {label} key: {value:?}");
        }
    }
    let record = workspace.path(format!(
        "opportunities/{organisation}/{position}/application.toml"
    ));
    if require_exists && !workspace.input_is_file(&record) {
        bail!(
            "opportunity record does not exist: opportunities/{organisation}/{position}/application.toml"
        );
    }
    Ok(record)
}

/// Instantiate neutral metadata and the configured styles' empty content fields.
/// Style scaffolds contain only document fields; never copy showcase wording.
pub fn blank_record(workspace: &Workspace) -> Result<toml::Value> {
    let mut record: toml::Value = toml::from_str(&fs::read_to_string(
        workspace.path(".agent/scaffolds/opportunity/application.toml"),
    )?)
    .context("invalid scaffold application.toml")?;
    for document in ["cv", "cl"] {
        let selected = crate::styles::selection(workspace, document, None, None)?;
        let definition = crate::styles::definition(workspace, document, &selected.style)?;
        record["options"]
            .as_table_mut()
            .context("missing options table")?
            .insert(format!("{document}_style"), selected.style.clone().into());
        record["options"]
            .as_table_mut()
            .context("missing options table")?
            .insert(format!("{document}_substyle"), selected.substyle.into());
        let page_key = if document == "cv" {
            "pages"
        } else {
            "cl_pages"
        };
        record["options"]
            .as_table_mut()
            .context("missing options table")?
            .insert(
                page_key.into(),
                i64::try_from(definition.default_pages)?.into(),
            );
        let path = crate::styles::root(workspace, document)?
            .join(&selected.style)
            .join("scaffold.toml");
        let content = if path.exists() {
            toml::from_str(&fs::read_to_string(workspace.existing_inside(&path)?)?)?
        } else {
            toml::Value::Table(toml::Table::new())
        };
        record
            .as_table_mut()
            .context("invalid scaffold table")?
            .insert(document.into(), content);
    }
    Ok(record)
}

/// MECE split: position prose lives in posting.md, the one human-readable
/// position reference. The record keeps only the role identity plus the
/// document build.
pub fn strip_position_prose(document: &mut toml::Value) {
    if let Some(job) = document.get_mut("job").and_then(toml::Value::as_table_mut) {
        for field in [
            "source",
            "url",
            "description",
            "connections",
            "company_context",
            "notes",
        ] {
            job.remove(field);
        }
    }
}

pub fn create_record(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
    cover_letter: bool,
) -> Result<PathBuf> {
    let destination = record_path(workspace, organisation, position, false)?;
    if destination.exists() {
        bail!(
            "refusing to overwrite existing opportunity: {}",
            workspace.relative(&destination)?.display()
        );
    }
    let mut document = blank_record(workspace)?;
    document["job"]["id"] = toml::Value::String(format!("{organisation}--{position}"));
    strip_position_prose(&mut document);
    if !cover_letter {
        // No letter needed: leave it out in the first place instead of
        // writing a [cl] table the user must delete (validation rejects a
        // disabled letter that retains hidden content).
        document["options"]["generate_cl"] = toml::Value::Boolean(false);
        if let Some(table) = document.as_table_mut() {
            table.remove("cl");
        }
    }
    let parent = destination
        .parent()
        .context("opportunity path has no parent")?;
    fs::create_dir_all(parent)?;
    fs::write(&destination, format!("{}\n", toml::to_string(&document)?))?;
    let posting = parent.join("posting.md");
    if !posting.exists() {
        fs::write(
            &posting,
            format!(
                "# Posting reference — {organisation}/{position}\n\
                 \n\
                 - Source:\n\
                 - Retrieved:\n\
                 - Organisation:\n\
                 - Contact:\n\
                 - Workplace:\n\
                 - Tasks:\n\
                 - Requirements:\n\
                 - Process:\n\
                 - Company context:\n\
                 - Notes:\n"
            ),
        )?;
    }
    Ok(destination)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn temporary_workspace() -> (tempfile::TempDir, Workspace) {
        let directory = tempdir().unwrap();
        let original = Workspace::at(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        for relative in [
            "ccvl.json",
            ".agent/scaffolds/opportunity/application.toml",
            "cvl/cv/harvard/style.toml",
            "cvl/cl/harvard/style.toml",
            "cvl/cv/harvard/scaffold.toml",
            "cvl/cl/harvard/scaffold.toml",
            "cvl/shared/harvard/defaults.toml",
        ] {
            let target = directory.path().join(relative);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::copy(original.path(relative), target).unwrap();
        }
        let workspace = Workspace::at(directory.path()).unwrap();
        (directory, workspace)
    }

    #[test]
    fn path_keys_cannot_escape() {
        let workspace = Workspace::at(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert_eq!(
            record_path(&workspace, "acme", "strategy-lead", false).unwrap(),
            workspace.path("opportunities/acme/strategy-lead/application.toml")
        );
        assert!(record_path(&workspace, "../acme", "lead", false).is_err());
        assert!(record_path(&workspace, "ACME", "lead", false).is_err());
        assert!(record_path(&workspace, "acme", "lead/../other", false).is_err());
    }

    #[test]
    fn new_opportunity_is_keyed_and_never_overwritten() {
        let (_directory, workspace) = temporary_workspace();
        let record = create_record(&workspace, "example_org", "strategy-lead", true).unwrap();
        let text = fs::read_to_string(&record).unwrap();
        let document: toml::Value = toml::from_str(&text).unwrap();
        assert_eq!(
            document["job"]["id"].as_str(),
            Some("example_org--strategy-lead")
        );
        assert!(document["job"].get("description").is_none());
        assert!(document["job"].get("notes").is_none());
        assert_eq!(document["options"]["generate_cl"].as_bool(), Some(true));
        assert!(document.get("cl").is_some());
        let skeleton = fs::read_to_string(record.parent().unwrap().join("posting.md")).unwrap();
        assert!(
            skeleton.starts_with("# Posting reference — example_org/strategy-lead"),
            "unexpected skeleton: {skeleton}"
        );
        let error = create_record(&workspace, "example_org", "strategy-lead", true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("refusing to overwrite"));
    }

    #[test]
    fn new_opportunity_without_cover_letter_omits_the_cl_table() {
        let (_directory, workspace) = temporary_workspace();
        let record = create_record(&workspace, "example_org", "cv-only-role", false).unwrap();
        let text = fs::read_to_string(&record).unwrap();
        let document: toml::Value = toml::from_str(&text).unwrap();
        assert_eq!(document["options"]["generate_cl"].as_bool(), Some(false));
        assert!(
            document.get("cl").is_none(),
            "a CV-only record must not contain a [cl] table to delete"
        );
    }
}
