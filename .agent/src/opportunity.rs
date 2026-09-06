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
    if require_exists && !record.is_file() {
        bail!(
            "opportunity record does not exist: opportunities/{organisation}/{position}/application.toml"
        );
    }
    Ok(record)
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
    let mut document: toml::Value = toml::from_str(&fs::read_to_string(
        workspace.path(".agent/scaffolds/opportunity/application.toml"),
    )?)
    .context("invalid scaffold application.toml")?;
    document["job"]["id"] = toml::Value::String(format!("{organisation}--{position}"));
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
    Ok(destination)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn temporary_workspace() -> (tempfile::TempDir, Workspace) {
        let directory = tempdir().unwrap();
        fs::write(directory.path().join("ccvl.json"), "{}\n").unwrap();
        fs::create_dir_all(directory.path().join(".agent/scaffolds/opportunity")).unwrap();
        fs::write(
            directory
                .path()
                .join(".agent/scaffolds/opportunity/application.toml"),
            "[job]\nid = \"\"\n\n[options]\ngenerate_cl = true\n\n[cl]\nhighlights = []\n",
        )
        .unwrap();
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
        assert_eq!(document["options"]["generate_cl"].as_bool(), Some(true));
        assert!(document.get("cl").is_some());
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
