//! Hash-bound, local review packets for an externally orchestrated independent critic.
//!
//! This interface validates artifacts and results; it does not invoke a model or
//! create an OS sandbox. Provider limits and fresh context remain attestations.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use clap::Subcommand;

use crate::Workspace;
use storage::{CORRECTIONS, Lock, SCHEMA, load, read, revision_path, save_state};
pub use types::*;

mod package;
mod render;
mod storage;
mod types;
mod validation;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Compile selected documents and freeze an independent review packet.
    Prepare { spec: PathBuf, run_dir: PathBuf },
    /// Validate a fresh external critic's structured review against the packet.
    Submit { run_dir: PathBuf, result: PathBuf },
    /// Revalidate hashes and report readiness without changing the run.
    Status { run_dir: PathBuf },
    /// Reserve one of two corrections before the actor edits canonical wording.
    BeginRevision { run_dir: PathBuf },
    /// Compile and freeze the reserved correction using the pinned specification.
    PrepareRevision { run_dir: PathBuf },
    /// Cancel the run; cancellation never produces readiness.
    Cancel { run_dir: PathBuf },
}

pub fn run(workspace: &Workspace, command: Command) -> Result<Status> {
    match command {
        Command::Prepare { spec, run_dir } => {
            let spec: Spec = read(&workspace.existing_inside(spec)?)?;
            package::validate_spec(workspace, &spec)?;
            let run = storage::run_path(workspace, &run_dir)?;
            fs::create_dir_all(run.parent().context("run has no parent")?)?;
            if !run.exists() {
                fs::create_dir(&run)?;
            }
            let _lock = Lock::acquire(&run)?;
            ensure!(
                !run.join("state.json").exists(),
                "review run already exists; choose a new run ID"
            );
            let request = run.join("request.json");
            if request.is_file() {
                let pending: Spec = read(&request)?;
                ensure!(
                    pending == spec,
                    "interrupted initial prepare has a different pinned request"
                );
            } else {
                for entry in fs::read_dir(&run)? {
                    let entry = entry?;
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    ensure!(
                        entry.file_type()?.is_file()
                            && (name == ".lock" || name.starts_with(".review-tmp-")),
                        "existing run lacks a pinned request and contains unrecognized files"
                    );
                }
                storage::write_new(&request, &spec)?;
            }
            let completed = revision_path(&run, 0).join("manifest.json");
            let hash = if completed.is_file() {
                let manifest: Manifest = read(&completed)?;
                ensure!(
                    manifest.revision == 0
                        && manifest.spec == spec
                        && manifest.parent_manifest.is_none(),
                    "interrupted initial manifest has the wrong identity"
                );
                validation::fresh(workspace, &run, &manifest, false)?;
                storage::digest(&fs::read(&completed)?)
            } else {
                let manifest = package::build(workspace, &run, 0, &spec, None)?;
                storage::write_new(&completed, &manifest)?
            };
            save_state(
                &run,
                &RunState {
                    schema_version: SCHEMA,
                    revision: 0,
                    corrections_used: 0,
                    pending_revision: false,
                    cancelled: false,
                    manifest_sha256: hash,
                    result_sha256: None,
                },
            )?;
            status(workspace, &run_dir)
        }
        Command::Submit { run_dir, result } => {
            let run = storage::run_path(workspace, &run_dir)?;
            let _lock = Lock::acquire(&run)?;
            let (mut state, manifest) = load(&run)?;
            ensure!(
                !state.cancelled && !state.pending_revision,
                "run is cancelled or an actor revision is pending"
            );
            ensure!(
                state.result_sha256.is_none(),
                "this revision already has an immutable review result"
            );
            ensure!(
                storage::now()?.saturating_sub(manifest.created_at)
                    <= manifest.spec.time_limit_seconds,
                "critic deadline exceeded; run remains incomplete"
            );
            validation::fresh(workspace, &run, &manifest, false)?;
            let result: ReviewResult = read(&workspace.existing_inside(result)?)?;
            validation::validate(&run, &state, &manifest, &result)?;
            let destination = revision_path(&run, state.revision).join("result.json");
            state.result_sha256 = Some(if destination.is_file() {
                let expected = storage::digest(&serde_json::to_vec_pretty(&result)?);
                ensure!(
                    storage::digest(&fs::read(&destination)?) == expected,
                    "interrupted submission has a different immutable result"
                );
                expected
            } else {
                storage::write_new(&destination, &result)?
            });
            save_state(&run, &state)?;
            status(workspace, &run_dir)
        }
        Command::Status { run_dir } => status(workspace, &run_dir),
        Command::BeginRevision { run_dir } => {
            let run = storage::run_path(workspace, &run_dir)?;
            let _lock = Lock::acquire(&run)?;
            let (mut state, manifest) = load(&run)?;
            ensure!(
                !state.cancelled && !state.pending_revision,
                "run is cancelled or already has a pending correction"
            );
            ensure!(
                state.corrections_used < CORRECTIONS,
                "two-correction allowance exhausted; no further edits are authorized by this run"
            );
            validation::fresh(workspace, &run, &manifest, false)?;
            if manifest
                .artifacts
                .iter()
                .any(|artifact| artifact.role == "page")
            {
                let result = accepted_result(&run, &state)?
                    .context("review the rendered candidate before reserving another correction")?;
                ensure!(
                    validation::complete(&manifest, &result),
                    "finish source, page and regression coverage before another correction"
                );
            }
            // Reserve and consume before editing, including revisions that fail compilation.
            state.corrections_used += 1;
            state.pending_revision = true;
            save_state(&run, &state)?;
            status(workspace, &run_dir)
        }
        Command::PrepareRevision { run_dir } => {
            let run = storage::run_path(workspace, &run_dir)?;
            let _lock = Lock::acquire(&run)?;
            let (mut state, previous) = load(&run)?;
            ensure!(
                !state.cancelled && state.pending_revision,
                "reserve a correction with begin-revision before editing"
            );
            validation::fresh(workspace, &run, &previous, true)?;
            accepted_result(&run, &state)?;
            let revision = state.revision + 1;
            let completed = revision_path(&run, revision).join("manifest.json");
            let (manifest, hash) = if completed.is_file() {
                // A completed manifest may precede an interrupted atomic state
                // update. Reuse it only if the whole candidate still matches.
                let manifest: Manifest = read(&completed)?;
                ensure!(
                    manifest.revision == revision
                        && manifest.spec == previous.spec
                        && manifest.parent_manifest.as_ref() == Some(&state.manifest_sha256),
                    "interrupted candidate has the wrong identity"
                );
                validation::fresh(workspace, &run, &manifest, false)?;
                let hash = storage::digest(&fs::read(&completed)?);
                (manifest, hash)
            } else {
                let manifest = package::build(
                    workspace,
                    &run,
                    revision,
                    &previous.spec,
                    Some((&previous, &state.manifest_sha256)),
                )?;
                let hash = storage::write_new(
                    &revision_path(&run, revision).join("manifest.json"),
                    &manifest,
                )?;
                (manifest, hash)
            };
            ensure!(manifest.revision == revision, "candidate revision mismatch");
            state.revision = revision;
            state.pending_revision = false;
            state.manifest_sha256 = hash;
            state.result_sha256 = None;
            save_state(&run, &state)?;
            status(workspace, &run_dir)
        }
        Command::Cancel { run_dir } => {
            let run = storage::run_path(workspace, &run_dir)?;
            let _lock = Lock::acquire(&run)?;
            let (mut state, _) = load(&run)?;
            state.cancelled = true;
            save_state(&run, &state)?;
            status(workspace, &run_dir)
        }
    }
}

pub fn status(workspace: &Workspace, run_dir: &std::path::Path) -> Result<Status> {
    let run = storage::run_path(workspace, run_dir)?;
    let (state, manifest) = load(&run)?;
    let mut outcome = Status {
        state: "incomplete",
        revision: state.revision,
        corrections_used: state.corrections_used,
        corrections_remaining: CORRECTIONS - state.corrections_used,
        reasons: Vec::new(),
        manifest_sha256: state.manifest_sha256.clone(),
    };
    if state.cancelled {
        outcome.reasons.push("run cancelled".into());
        return Ok(outcome);
    }
    if state.pending_revision {
        outcome
            .reasons
            .push("correction reserved; prepare and review it before proceeding".into());
        return Ok(outcome);
    }
    if let Err(error) = validation::fresh(workspace, &run, &manifest, false) {
        outcome
            .reasons
            .push(format!("stale or unavailable artifacts: {error:#}"));
        return Ok(outcome);
    }
    let result = match accepted_result(&run, &state) {
        Ok(result) => result,
        Err(error) => {
            outcome.reasons.push(format!("invalid review: {error:#}"));
            return Ok(outcome);
        }
    };
    let mut evidence_gap = false;
    for check in manifest.checks.iter().filter(|check| !check.passed) {
        evidence_gap |= check.operation == "evidence";
        outcome.reasons.extend(check.diagnostics.clone());
    }
    if manifest
        .checks
        .iter()
        .any(|check| !check.passed && check.operation != "evidence")
    {
        return Ok(outcome);
    }
    if evidence_gap {
        outcome.state = "needs-evidence";
        return Ok(outcome);
    }
    let Some(result) = result else {
        outcome.reasons.push(
            if storage::now()?.saturating_sub(manifest.created_at)
                > manifest.spec.time_limit_seconds
            {
                "critic deadline exceeded"
            } else {
                "awaiting independent critic result"
            }
            .into(),
        );
        return Ok(outcome);
    };
    if let Err(error) = validation::validate(&run, &state, &manifest, &result) {
        outcome.reasons.push(format!("invalid review: {error:#}"));
        return Ok(outcome);
    }
    if !validation::complete(&manifest, &result) {
        outcome
            .reasons
            .push("source, page, two-pass or changed-artifact coverage is incomplete".into());
        return Ok(outcome);
    }
    for finding in result
        .findings
        .iter()
        .filter(|finding| finding.material && validation::unresolved(finding))
    {
        outcome
            .reasons
            .push(format!("{}: {}", finding.id, finding.materiality_reason));
    }
    let blockers = result
        .findings
        .iter()
        .filter(|finding| finding.material && validation::unresolved(finding))
        .collect::<Vec<_>>();
    outcome.state = if blockers.iter().any(|finding| {
        finding.resolution == Resolution::DisputedWithEvidence
            || finding.kind == FindingKind::Preference
    }) {
        "needs-decision"
    } else if blockers
        .iter()
        .any(|finding| finding.kind == FindingKind::Uncertainty)
    {
        "needs-evidence"
    } else if blockers.is_empty() {
        "ready"
    } else {
        "incomplete"
    };
    if !blockers.is_empty() && state.corrections_used == CORRECTIONS {
        outcome
            .reasons
            .push("correction allowance exhausted; this does not approve the candidate".into());
    }
    Ok(outcome)
}

fn accepted_result(run: &std::path::Path, state: &RunState) -> Result<Option<ReviewResult>> {
    let Some(hash) = &state.result_sha256 else {
        return Ok(None);
    };
    let path = revision_path(run, state.revision).join("result.json");
    ensure!(
        &storage::digest(&fs::read(&path)?) == hash,
        "accepted review result changed"
    );
    Ok(Some(read(&path)?))
}

#[cfg(test)]
mod tests;
