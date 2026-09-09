use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub purpose: Purpose,
    pub actor_id: String,
    pub model: String,
    pub max_input_tokens: u64,
    pub max_output_tokens: u64,
    pub time_limit_seconds: u64,
    pub documents: Vec<Document>,
    pub sources: Vec<Source>,
    pub rubric: Vec<PathBuf>,
    pub editable: Vec<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Purpose {
    pub kind: PurposeKind,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PurposeKind {
    General,
    Targeted,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub id: String,
    pub document: String,
    pub locale: String,
    pub pages: usize,
    pub application: PathBuf,
    pub profile: PathBuf,
    pub style: Option<String>,
    pub substyle: Option<String>,
    pub paper: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub path: PathBuf,
    pub role: SourceRole,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceRole {
    Candidate,
    Confirmation,
    Posting,
    Research,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub revision: u32,
    pub parent_manifest: Option<String>,
    pub parent_result: Option<String>,
    pub runtime: String,
    pub source_identity: String,
    pub created_at: u64,
    pub spec: Spec,
    pub artifacts: Vec<Artifact>,
    pub dependencies: BTreeMap<String, Option<String>>,
    pub checks: Vec<Check>,
    pub documents: Vec<Value>,
    pub changed_artifacts: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub id: String,
    pub role: String,
    pub sha256: String,
    pub snapshot: String,
    pub original: Option<String>,
    pub document: Option<String>,
    pub page: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub document: String,
    pub operation: String,
    pub passed: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunState {
    pub schema_version: u32,
    pub revision: u32,
    pub corrections_used: u32,
    pub pending_revision: bool,
    pub cancelled: bool,
    pub manifest_sha256: String,
    pub result_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewResult {
    pub schema_version: u32,
    pub revision: u32,
    pub manifest_sha256: String,
    pub reviewer: Reviewer,
    pub coverage: Coverage,
    pub findings: Vec<Finding>,
    pub regression_checked: bool,
    pub changed_artifacts_reviewed: Vec<String>,
    #[serde(deserialize_with = "required_option")]
    pub input_tokens: Option<u64>,
    #[serde(deserialize_with = "required_option")]
    pub output_tokens: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Reviewer {
    pub agent_id: String,
    pub model: String,
    pub fresh_context: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub artifacts_read: Vec<String>,
    pub unavailable: Vec<String>,
    pub not_applicable: BTreeMap<String, String>,
    pub evidence_pass: bool,
    pub presentation_pass: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub id: String,
    pub kind: FindingKind,
    pub category: Category,
    pub material: bool,
    #[serde(default)]
    pub explicit_requirement: bool,
    pub materiality_reason: String,
    pub confidence: f64,
    pub rule: Citation,
    pub location: Citation,
    pub evidence: Vec<Citation>,
    pub correction_constraint: String,
    pub resolution: Resolution,
    #[serde(deserialize_with = "required_option")]
    pub verification: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FindingKind {
    Error,
    Uncertainty,
    Preference,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Support,
    AttributionScope,
    Consistency,
    Relevance,
    Language,
    Presentation,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Resolution {
    Open,
    FixedAndVerified,
    DisputedWithEvidence,
    OptionalSuggestionDeclined,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Citation {
    pub artifact: String,
    pub sha256: String,
    pub locator: String,
    pub excerpt: String,
}

#[derive(Debug, Serialize)]
pub struct Status {
    pub state: &'static str,
    pub revision: u32,
    pub corrections_used: u32,
    pub corrections_remaining: u32,
    pub reasons: Vec<String>,
    pub manifest_sha256: String,
}

fn required_option<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(deserializer)
}
