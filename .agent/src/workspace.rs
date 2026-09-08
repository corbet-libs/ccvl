use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    pub fn discover(explicit: Option<&Path>) -> Result<Self> {
        if let Some(root) = explicit {
            return Self::at(root);
        }
        if let Ok(cwd) = std::env::current_dir()
            && let Some(root) = find_root(&cwd)
        {
            return Self::at(&root);
        }
        if let Ok(executable) = std::env::current_exe()
            && let Some(parent) = executable.parent()
            && let Some(root) = find_root(parent)
        {
            return Self::at(&root);
        }
        bail!("no ccvl.json found in the current directory or its parents")
    }

    pub fn at(root: &Path) -> Result<Self> {
        let root = root
            .canonicalize()
            .with_context(|| format!("cannot resolve workspace {}", root.display()))?;
        if !root.join("ccvl.json").is_file() {
            bail!("{} is not a ccvl workspace", root.display());
        }
        Ok(Self { root })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }

    pub fn existing_inside(&self, value: impl AsRef<Path>) -> Result<PathBuf> {
        let value = value.as_ref();
        let candidate = if value.is_absolute() {
            value.to_path_buf()
        } else {
            self.root.join(value)
        };
        let resolved = candidate
            .canonicalize()
            .with_context(|| format!("cannot resolve {}", value.display()))?;
        if !resolved.starts_with(&self.root) {
            bail!(
                "input must be inside the ccvl workspace: {}",
                value.display()
            );
        }
        Ok(resolved)
    }

    pub fn relative(&self, path: &Path) -> Result<PathBuf> {
        path.strip_prefix(&self.root)
            .map(Path::to_path_buf)
            .with_context(|| format!("{} is outside the ccvl workspace", path.display()))
    }

    pub fn typst_path(&self, path: &Path) -> Result<String> {
        let relative = self.relative(&self.existing_inside(path)?)?;
        Ok(format!(
            "/{}",
            relative.to_string_lossy().replace('\\', "/")
        ))
    }

    pub fn read_json(&self, relative: impl AsRef<Path>) -> Result<Value> {
        read_json(&self.path(relative))
    }

    /// Read a TOML record and convert it to a JSON value so the existing
    /// structural and semantic checks keep working on one value model.
    /// Only plain TOML data (strings, integers, booleans, arrays, tables)
    /// is supported; datetimes have no meaning in ccvl records.
    pub fn read_toml_value(&self, relative: impl AsRef<Path>) -> Result<Value> {
        read_toml_value(&self.path(relative))
    }
}

pub fn read_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("cannot read JSON file {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("invalid JSON in {}", path.display()))
}

pub fn read_toml_value(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("cannot read TOML file {}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("invalid TOML in {}", path.display()))?;
    serde_json::to_value(&value)
        .with_context(|| format!("unsupported TOML data in {}", path.display()))
}

fn find_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|candidate| candidate.join("ccvl.json").is_file())
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> Workspace {
        Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    #[test]
    fn repository_root_is_discoverable() {
        let workspace = Workspace::discover(None).unwrap();
        assert!(workspace.root().join("ccvl.json").is_file());
    }

    #[test]
    fn parent_escape_is_rejected() {
        let workspace = repository();
        // "../.." always exists and always lies outside the workspace, unlike
        // a named sibling which may resolve back inside when the checkout
        // itself carries that name.
        assert!(workspace.existing_inside("../..").is_err());
    }

    #[test]
    fn three_user_data_groups_are_top_level_and_documented() {
        let workspace = repository();
        for name in ["interview", "cvl", "opportunities"] {
            assert!(workspace.path(name).is_dir());
            assert!(workspace.path(name).join("README.md").is_file());
        }
    }

    #[test]
    fn manifest_names_the_interview_cvl_and_keyed_opportunity_contracts() {
        let repository = repository();
        let manifest_text = fs::read_to_string(repository.path("ccvl.json")).unwrap();
        let workspace_groups = manifest_text.split_once("\"workspace_groups\"").unwrap().1;
        let interview = workspace_groups.find("\"interview\"").unwrap();
        let cvl = workspace_groups.find("\"cvl\"").unwrap();
        let opportunities = workspace_groups.find("\"opportunities\"").unwrap();
        assert!(interview < cvl && cvl < opportunities);

        let manifest: Value = serde_json::from_str(&manifest_text).unwrap();
        let groups = manifest["workspace_groups"].as_object().unwrap();
        assert_eq!(groups["interview"]["stations"], "interview/stations.toml");
        assert_eq!(groups["cvl"]["profile"], "cvl/profile.toml");
        assert_eq!(
            groups["opportunities"]["path"],
            "opportunities/<organisation-key>/<position-key>"
        );
        assert_eq!(groups["opportunities"]["record"], "application.toml");
        assert_eq!(groups["opportunities"]["pdfs"], "pdfs");
        assert_eq!(groups["opportunities"]["typst"], "typst");
    }

    #[test]
    fn cv_outputs_use_numeric_page_names() {
        let workspace = repository();
        for (document, substyle, lang, name) in [
            ("cv", "standard", "de", "cv-2.pdf"),
            ("cv", "standard", "de", "cv-3.pdf"),
            ("cv", "standard", "de", "cv-4.pdf"),
            ("cv", "compact", "en", "cv-2.pdf"),
            ("cv", "compact", "en", "cv-3.pdf"),
            ("cv", "compact", "en", "cv-4.pdf"),
            ("cl", "left-rule", "de", "cl.pdf"),
            ("cl", "frame", "en", "cl.pdf"),
        ] {
            assert!(
                workspace
                    .path(format!("cvl/{document}/{substyle}/{lang}/ch/pdf/{name}"))
                    .is_file(),
                "missing cvl/{document}/{substyle}/{lang}/ch/pdf/{name}"
            );
        }
    }

    #[test]
    fn private_downstream_sync_is_push_driven_and_gated() {
        let workflow = fs::read_to_string(repository().path(".crow.yaml")).unwrap();
        assert!(workflow.contains("event: push"));
        assert!(workflow.contains("branch: main"));
        assert!(!workflow.contains("event: cron"));
        assert!(!workflow.contains("event: manual"));
        assert!(workflow.contains("CARGO_TARGET_DIR: /caches/cargo/targets/ccvl"));
        assert!(workflow.contains("cargo build --locked"));
        assert!(!workflow.contains("cargo build --locked --release"));
        assert!(workflow.contains("downstream-check"));
        let push = workflow.find("git push --porcelain").unwrap();
        assert!(workflow.find("downstream-check").unwrap() < push);
        assert!(workflow.find("\"$gate\" check").unwrap() < push);
    }
}
