use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const BUNDLE_SCHEMA: u32 = 1;
const INPUT_PREFIX: &str = "__ccvl_inputs/";

/// One selected style/substyle/locale/page/paper variant, with exact upstream assets.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StyleBundle {
    pub schema_version: u32,
    pub engine_api: u32,
    pub id: String,
    pub version: String,
    pub document: String,
    pub style: String,
    pub substyle: String,
    pub locale: String,
    pub pages: usize,
    pub entry: String,
    pub inputs: BTreeMap<String, String>,
    pub contract: Value,
    pub scaffold: Value,
    pub fonts: Vec<String>,
    pub files: BTreeMap<String, Vec<u8>>,
}

/// User data is separate from the reusable bundle. Both file and API adapters
/// can supply this value without a filesystem-shaped core API.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StyleDocument {
    pub bundle: Arc<StyleBundle>,
    pub bundle_sha256: String,
    pub record: Value,
    pub profile: Value,
}

impl StyleBundle {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == BUNDLE_SCHEMA && self.engine_api == 1,
            "unsupported CCVL style bundle or engine API"
        );
        ensure!(
            !self.id.is_empty() && !self.version.is_empty(),
            "style identity and version are required"
        );
        ensure!(
            matches!(self.document.as_str(), "cv" | "cl"),
            "unsupported document type"
        );
        ensure!(
            crate::normalize_locale(&self.locale)? == self.locale,
            "bundle locale must be canonical"
        );
        ensure!(self.pages > 0, "bundle must declare its page count");
        ensure!(!self.files.is_empty(), "style bundle has no assets");
        let mut total = 0_usize;
        for (path, bytes) in &self.files {
            validate_path(path)?;
            ensure!(
                !path.starts_with(INPUT_PREFIX),
                "style asset shadows a document input"
            );
            total = total
                .checked_add(bytes.len())
                .context("style asset size overflow")?;
        }
        ensure!(total <= 64 * 1024 * 1024, "style bundle exceeds 64 MiB");
        validate_path(&self.entry)?;
        ensure!(
            self.files.contains_key(&self.entry),
            "style entry point is missing"
        );
        for font in &self.fonts {
            ensure!(
                self.files.contains_key(font),
                "declared style font is missing: {font}"
            );
        }
        ensure!(
            self.inputs.get("locale") == Some(&self.locale),
            "bundle locale input differs"
        );
        ensure!(
            self.inputs.get("pages") == Some(&self.pages.to_string()),
            "bundle page input differs"
        );
        for name in ["application", "profile"] {
            ensure!(
                !self.inputs.contains_key(name),
                "user input {name} must not be bundled"
            );
        }
        for name in [
            "strings",
            "substyle",
            "shared-defaults",
            "layout",
            "contract",
        ] {
            if let Some(path) = self.inputs.get(name) {
                ensure!(
                    self.files.contains_key(path.trim_start_matches('/')),
                    "missing style input {name}: {path}"
                );
            }
        }
        ensure!(
            self.scaffold.is_object(),
            "style scaffold must be an object"
        );
        crate::contract::validate_contract(&self.contract)
    }

    pub fn sha256(&self) -> Result<String> {
        Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(self)?)))
    }

    pub fn with_record(self, record: Value, profile: Value) -> Result<StyleDocument> {
        self.validate()?;
        let document = StyleDocument {
            bundle_sha256: self.sha256()?,
            bundle: Arc::new(self),
            record,
            profile,
        };
        document.validate()?;
        Ok(document)
    }
}

impl StyleDocument {
    pub fn validate(&self) -> Result<()> {
        self.bundle.validate()?;
        ensure!(
            self.bundle.sha256()? == self.bundle_sha256,
            "style bundle changed; select its new version explicitly"
        );
        ensure!(
            self.record.is_object() && self.profile.is_object(),
            "document and profile must be objects"
        );
        ensure!(
            self.record.get("wording").is_none(),
            "consumer records must contain resolved wording"
        );
        let options = self
            .record
            .get("options")
            .context("document options are missing")?;
        ensure!(
            options.get("language").and_then(Value::as_str) == Some(&self.bundle.locale),
            "document locale differs from selected bundle"
        );
        for (suffix, expected) in [
            ("style", &self.bundle.style),
            ("substyle", &self.bundle.substyle),
        ] {
            ensure!(
                options
                    .get(format!("{}_{}", self.bundle.document, suffix))
                    .and_then(Value::as_str)
                    == Some(expected),
                "document {suffix} differs from selected bundle"
            );
        }
        let page_key = if self.bundle.document == "cv" {
            "pages"
        } else {
            "cl_pages"
        };
        if let Some(pages) = options.get(page_key) {
            ensure!(
                pages.as_u64() == Some(self.bundle.pages as u64),
                "document pages differ from selected bundle"
            );
        }
        if let Some(paper) = options.get(format!("{}_paper", self.bundle.document)) {
            ensure!(
                paper.as_str() == self.bundle.inputs.get("paper").map(String::as_str),
                "document paper differs from selected bundle"
            );
        }
        let content = self
            .record
            .get(&self.bundle.document)
            .and_then(Value::as_object)
            .context("selected document content is missing")?;
        if let Some(fields) = self
            .bundle
            .contract
            .get("content_fields")
            .and_then(Value::as_array)
        {
            for key in content.keys() {
                ensure!(
                    fields.iter().any(|field| field.as_str() == Some(key)),
                    "unknown style content field: {key}"
                );
            }
        }
        Ok(())
    }
}

pub fn validate_path(path: &str) -> Result<()> {
    ensure!(
        !path.is_empty()
            && !path.starts_with('/')
            && !path.contains(['\\', ':', '\0'])
            && path.split('/').all(|part| !matches!(part, "" | "." | "..")),
        "invalid virtual asset path: {path:?}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_paths_cannot_escape_or_alias() {
        for path in [
            "../data", "/data", "a/../b", "a//b", "C:/data", "a\\b", "./a",
        ] {
            assert!(validate_path(path).is_err(), "{path}");
        }
        validate_path("cvl/cl/example/layout.typ").unwrap();
    }
}
