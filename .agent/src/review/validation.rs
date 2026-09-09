use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};

use super::{
    storage,
    types::{
        Artifact, Citation, Finding, FindingKind, Manifest, PurposeKind, Resolution, ReviewResult,
        RunState,
    },
};
use crate::Workspace;

pub(super) fn fresh(
    workspace: &Workspace,
    run: &Path,
    manifest: &Manifest,
    allow_edits: bool,
) -> Result<()> {
    ensure!(
        manifest.runtime == crate::runtime::ID,
        "runtime changed; start a new review run"
    );
    ensure!(
        manifest.source_identity == crate::runtime_source::fingerprint(workspace.root())?,
        "runtime source changed; start a new review run"
    );
    let editable = manifest
        .spec
        .editable
        .iter()
        .map(|path| {
            workspace
                .existing_inside(path)
                .and_then(|path| storage::relative(workspace, &path))
        })
        .collect::<Result<BTreeSet<_>>>()?;
    for (path, expected) in &manifest.dependencies {
        if allow_edits && editable.contains(path) {
            continue;
        }
        let current = if workspace.path(path).is_file() {
            Some(storage::digest(&fs::read(
                workspace.existing_inside(path)?,
            )?))
        } else {
            None
        };
        ensure!(&current == expected, "stale dependency: {path}");
    }
    for artifact in &manifest.artifacts {
        let path = storage::artifact_path(run, artifact)?;
        ensure!(
            storage::digest(&fs::read(path)?) == artifact.sha256,
            "snapshot changed: {}",
            artifact.id
        );
        if let Some(original) = &artifact.original {
            if allow_edits && editable.contains(original) {
                continue;
            }
            ensure!(
                storage::digest(&fs::read(workspace.existing_inside(original)?)?)
                    == artifact.sha256,
                "stale original: {original}"
            );
        }
    }
    Ok(())
}

pub(super) fn validate(
    run: &Path,
    state: &RunState,
    manifest: &Manifest,
    result: &ReviewResult,
) -> Result<()> {
    ensure!(
        result.schema_version == storage::SCHEMA,
        "unsupported review result schema"
    );
    ensure!(
        result.revision == state.revision && result.manifest_sha256 == state.manifest_sha256,
        "review targets stale artifacts"
    );
    ensure!(
        result.reviewer.fresh_context
            && !result.reviewer.agent_id.trim().is_empty()
            && result.reviewer.agent_id != manifest.spec.actor_id,
        "critic must attest fresh context and an identity different from the actor"
    );
    ensure!(
        result.reviewer.model == manifest.spec.model,
        "critic model differs from pinned model"
    );
    ensure!(
        result
            .input_tokens
            .is_none_or(|tokens| tokens <= manifest.spec.max_input_tokens)
            && result
                .output_tokens
                .is_none_or(|tokens| tokens <= manifest.spec.max_output_tokens),
        "reported critic token usage exceeds the declared limits"
    );
    let known = manifest
        .artifacts
        .iter()
        .map(|artifact| artifact.id.as_str())
        .collect::<BTreeSet<_>>();
    let read = result
        .coverage
        .artifacts_read
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    ensure!(
        read.len() == result.coverage.artifacts_read.len() && read.is_subset(&known),
        "coverage has duplicates or unknown artifact IDs"
    );
    ensure!(
        result
            .coverage
            .not_applicable
            .values()
            .all(|reason| !reason.trim().is_empty()),
        "not-applicable criteria need reasons"
    );
    if manifest.spec.purpose.kind == PurposeKind::General {
        ensure!(
            result
                .coverage
                .not_applicable
                .contains_key("posting-specific"),
            "general reviews must mark posting-specific criteria inapplicable"
        );
    } else {
        ensure!(
            !result
                .coverage
                .not_applicable
                .contains_key("posting-specific"),
            "targeted reviews cannot skip posting-specific criteria"
        );
    }
    let expected = manifest.changed_artifacts.iter().collect::<BTreeSet<_>>();
    let checked = result
        .changed_artifacts_reviewed
        .iter()
        .collect::<BTreeSet<_>>();
    ensure!(
        checked.is_subset(&expected) && checked.len() == result.changed_artifacts_reviewed.len(),
        "regression coverage names unknown or duplicate changed artifacts"
    );
    let previous = previous_result(run, manifest)?;
    if let Some(previous) = &previous {
        ensure!(
            previous.reviewer.agent_id != result.reviewer.agent_id,
            "each revision requires a fresh critic agent identity"
        );
    }
    let mut ids = BTreeSet::new();
    for finding in &result.findings {
        ensure!(
            !finding.id.trim().is_empty() && ids.insert(&finding.id),
            "finding IDs must be nonempty and unique"
        );
        ensure!(
            finding.confidence.is_finite() && (0.0..=1.0).contains(&finding.confidence),
            "confidence must be in 0–1"
        );
        ensure!(
            finding.kind != FindingKind::Preference
                || !finding.material
                || finding.explicit_requirement,
            "preferences can block readiness only when they cite an explicit user requirement"
        );
        ensure!(
            !finding.materiality_reason.trim().is_empty()
                && !finding.correction_constraint.trim().is_empty(),
            "findings require materiality reasons and correction constraints"
        );
        let rule = citation(run, manifest, &finding.rule)?;
        ensure!(
            rule.role == "rule",
            "finding rule must cite a pinned rubric or selected contract"
        );
        let location = citation(run, manifest, &finding.location)?;
        ensure!(
            ["content", "text", "page"].contains(&location.role.as_str()),
            "finding location must identify actual content or rendered text/page"
        );
        ensure!(
            !finding.evidence.is_empty(),
            "findings require cited evidence or an exact visual observation"
        );
        for evidence in &finding.evidence {
            citation(run, manifest, evidence)?;
        }
        for reference in std::iter::once(&finding.rule)
            .chain(std::iter::once(&finding.location))
            .chain(finding.evidence.iter())
        {
            ensure!(
                read.contains(reference.artifact.as_str()),
                "finding cites an artifact not recorded as read"
            );
        }
        if finding.resolution == Resolution::OptionalSuggestionDeclined {
            ensure!(
                finding.kind == FindingKind::Preference && !finding.material,
                "only nonmaterial preferences may be optionally declined"
            );
        }
        if finding.resolution != Resolution::Open {
            ensure!(
                finding
                    .verification
                    .as_ref()
                    .is_some_and(|text| !text.trim().is_empty()),
                "resolution requires independent verification or counterevidence explanation"
            );
        }
        if finding.resolution == Resolution::FixedAndVerified {
            ensure!(
                previous
                    .as_ref()
                    .is_some_and(|prior| prior.findings.iter().any(|old| old.id == finding.id)),
                "a verified closure must refer to a finding in the previous revision"
            );
        }
    }
    if let Some(previous) = &previous {
        for old in &previous.findings {
            if unresolved(old) && old.material {
                let current = result.findings.iter().find(|finding| finding.id == old.id).with_context(|| format!("previous material finding {} requires explicit verification; omission is not closure", old.id))?;
                if current.kind != old.kind
                    || !current.material
                    || current.rule.sha256 != old.rule.sha256
                {
                    ensure!(
                        current
                            .verification
                            .as_ref()
                            .is_some_and(|text| !text.trim().is_empty())
                            && !current.evidence.is_empty(),
                        "reassessing a prior material finding requires independently cited counterevidence and an explanation"
                    );
                }
            }
        }
    }
    Ok(())
}

fn citation<'a>(run: &Path, manifest: &'a Manifest, cite: &Citation) -> Result<&'a Artifact> {
    let artifact = manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.id == cite.artifact)
        .context("citation names an unknown artifact")?;
    ensure!(
        artifact.sha256 == cite.sha256,
        "citation uses a stale artifact hash"
    );
    ensure!(
        !cite.locator.trim().is_empty() && !cite.excerpt.trim().is_empty(),
        "citation needs an exact locator and excerpt/visual observation"
    );
    if let Some(page) = artifact.page {
        ensure!(
            cite.locator.split('#').next() == Some(format!("page:{page}").as_str()),
            "citation page locator differs from the artifact page"
        );
    }
    if artifact.role != "page" && artifact.role != "pdf" {
        let bytes = fs::read(storage::artifact_path(run, artifact)?)?;
        let text = std::str::from_utf8(&bytes).context(
            "binary source needs an extracted-text citation or rendered-page observation",
        )?;
        if artifact.page.is_some() {
            ensure!(
                text.contains(&cite.excerpt),
                "citation excerpt does not occur on the cited page"
            );
        }
        if artifact.page.is_none() {
            if let Some(pointer) = cite.locator.strip_prefix("json:") {
                let value: serde_json::Value = serde_json::from_str(text)?;
                let target = value
                    .pointer(pointer)
                    .context("citation JSON pointer does not exist")?;
                let located = target
                    .as_str()
                    .map_or_else(|| target.to_string(), str::to_owned);
                ensure!(
                    located.contains(&cite.excerpt),
                    "citation excerpt is absent at its JSON pointer"
                );
            } else {
                let line = cite
                    .locator
                    .strip_prefix("line:")
                    .context("text citations require line:N or json:/pointer locators")?
                    .parse::<usize>()?;
                ensure!(
                    line > 0
                        && text
                            .lines()
                            .nth(line - 1)
                            .is_some_and(|text| text.contains(&cite.excerpt)),
                    "citation excerpt is absent from its line locator"
                );
            }
        }
    }
    Ok(artifact)
}

pub(super) fn previous_result(run: &Path, manifest: &Manifest) -> Result<Option<ReviewResult>> {
    let mut child = manifest.clone();
    let mut latest = None;
    // A mechanically failed intermediate candidate has no critic result. Walk
    // through it without dropping prior obligations and verify all ancestors.
    while child.revision > 0 {
        let directory = storage::revision_path(run, child.revision - 1);
        let path = directory.join("manifest.json");
        ensure!(
            child.parent_manifest.as_ref() == Some(&storage::digest(&fs::read(&path)?)),
            "parent manifest changed"
        );
        let parent: Manifest = storage::read(&path)?;
        ensure!(
            parent.revision + 1 == child.revision && parent.schema_version == storage::SCHEMA,
            "invalid revision ancestry"
        );
        for artifact in &parent.artifacts {
            ensure!(
                storage::digest(&fs::read(storage::artifact_path(run, artifact)?)?)
                    == artifact.sha256,
                "ancestor snapshot changed: {}",
                artifact.id
            );
        }
        if let Some(expected) = &child.parent_result {
            let path = directory.join("result.json");
            ensure!(
                &storage::digest(&fs::read(&path)?) == expected,
                "parent review result changed"
            );
            let result: ReviewResult = storage::read(&path)?;
            if latest.is_none() {
                latest = Some(result);
            }
        }
        child = parent;
    }
    Ok(latest)
}

pub(super) fn complete(manifest: &Manifest, result: &ReviewResult) -> bool {
    result.coverage.evidence_pass
        && result.coverage.presentation_pass
        && result.coverage.unavailable.is_empty()
        && manifest
            .artifacts
            .iter()
            .filter(|artifact| artifact.role != "input")
            .all(|artifact| result.coverage.artifacts_read.contains(&artifact.id))
        && result.regression_checked
        && manifest
            .changed_artifacts
            .iter()
            .all(|id| result.changed_artifacts_reviewed.contains(id))
}

pub(super) fn unresolved(finding: &Finding) -> bool {
    matches!(
        finding.resolution,
        Resolution::Open | Resolution::DisputedWithEvidence
    )
}
