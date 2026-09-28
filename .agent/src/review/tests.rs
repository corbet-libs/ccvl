use std::collections::BTreeMap;
use std::path::Path;

use tempfile::{TempDir, tempdir};

use super::*;

#[path = "tests_recovery.rs"]
mod recovery;

#[path = "tests_render.rs"]
mod rendered_packet;

struct Fixture {
    directory: TempDir,
    workspace: Workspace,
    run: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempdir().unwrap();
        let root = directory.path();
        fs::create_dir_all(root.join(".agent/src")).unwrap();
        fs::create_dir_all(root.join(".agent/core/src")).unwrap();
        for path in [
            "ccvl.json",
            "Cargo.toml",
            ".agent/core/Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            ".agent/build.rs",
        ] {
            fs::write(root.join(path), "{}\n").unwrap();
        }
        fs::create_dir_all(root.join("cvl/cv/demo/en/ch")).unwrap();
        fs::write(
            root.join("cvl/cv/demo/en/ch/content.toml"),
            "claim = 'Participated in a project'\n",
        )
        .unwrap();
        fs::write(root.join("evidence.txt"), "Participated in a project\n").unwrap();
        fs::write(
            root.join("rubric.txt"),
            "Preserve the scope of candidate evidence.\n",
        )
        .unwrap();
        let workspace = Workspace::at(root).unwrap();
        let run = workspace.path(".agent/cache/review/test");
        fs::create_dir_all(run.join("revision-0/objects")).unwrap();
        let mut fixture = Self {
            directory,
            workspace,
            run,
        };
        fixture.install(0, None);
        fixture
    }

    fn spec() -> Spec {
        Spec {
            purpose: Purpose {
                kind: PurposeKind::General,
                description: "General CV".into(),
            },
            actor_id: "actor".into(),
            model: "test-critic".into(),
            max_input_tokens: 10000,
            max_output_tokens: 10000,
            time_limit_seconds: 3600,
            documents: vec![Document {
                id: "cv".into(),
                document: "cv".into(),
                locale: "en-ch".into(),
                pages: 1,
                application: "cvl/cv/demo/en/ch/content.toml".into(),
                profile: "cvl/profile.toml".into(),
                style: None,
                substyle: None,
                paper: None,
            }],
            sources: vec![Source {
                id: "candidate".into(),
                path: "evidence.txt".into(),
                role: SourceRole::Candidate,
            }],
            rubric: vec!["rubric.txt".into()],
            editable: vec!["cvl/cv/demo/en/ch/content.toml".into()],
        }
    }

    fn install(&mut self, revision: u32, parent: Option<(String, String)>) {
        fs::create_dir_all(revision_path(&self.run, revision).join("objects")).unwrap();
        let artifacts = [
            (
                "rule:0",
                "rule",
                b"Preserve the scope of candidate evidence.\n".as_slice(),
                None,
            ),
            (
                "source:candidate",
                "evidence",
                b"Participated in a project\n".as_slice(),
                Some("evidence.txt".into()),
            ),
            (
                "cv:content",
                "content",
                b"{\"claim\":\"Participated in a project\"}".as_slice(),
                None,
            ),
            (
                "cv:page:1",
                "page",
                b"synthetic image bytes".as_slice(),
                None,
            ),
        ]
        .into_iter()
        .map(|(id, role, bytes, original)| {
            storage::freeze(
                &self.run,
                revision,
                id,
                role,
                bytes,
                original,
                None,
                if role == "page" { Some(1) } else { None },
            )
            .unwrap()
        })
        .collect();
        let manifest = Manifest {
            schema_version: SCHEMA,
            revision,
            parent_manifest: parent.as_ref().map(|pair| pair.0.clone()),
            parent_result: parent.map(|pair| pair.1),
            runtime: crate::runtime::ID.into(),
            source_identity: crate::runtime_source::fingerprint(self.workspace.root()).unwrap(),
            created_at: storage::now().unwrap(),
            spec: Self::spec(),
            artifacts,
            dependencies: BTreeMap::from([(
                "evidence.txt".into(),
                Some(storage::digest(
                    &fs::read(self.workspace.path("evidence.txt")).unwrap(),
                )),
            )]),
            checks: vec![Check {
                document: "cv".into(),
                operation: "compile".into(),
                passed: true,
                diagnostics: vec![],
            }],
            documents: vec![],
            changed_artifacts: if revision > 0 {
                vec!["cv:content".into()]
            } else {
                vec![]
            },
            limitations: vec![],
        };
        let hash = storage::write_new(
            &revision_path(&self.run, revision).join("manifest.json"),
            &manifest,
        )
        .unwrap();
        save_state(
            &self.run,
            &RunState {
                schema_version: SCHEMA,
                revision,
                corrections_used: revision,
                pending_revision: false,
                cancelled: false,
                manifest_sha256: hash,
                result_sha256: None,
            },
        )
        .unwrap();
    }

    fn result(&self) -> ReviewResult {
        let (state, manifest) = load(&self.run).unwrap();
        ReviewResult {
            schema_version: SCHEMA,
            revision: state.revision,
            manifest_sha256: state.manifest_sha256,
            reviewer: Reviewer {
                agent_id: format!("critic-{}", state.revision),
                model: "test-critic".into(),
                fresh_context: true,
            },
            coverage: Coverage {
                artifacts_read: manifest
                    .artifacts
                    .iter()
                    .map(|artifact| artifact.id.clone())
                    .collect(),
                unavailable: vec![],
                not_applicable: BTreeMap::from([(
                    "posting-specific".into(),
                    "No vacancy for a general CV".into(),
                )]),
                evidence_pass: true,
                presentation_pass: true,
            },
            findings: vec![],
            regression_checked: true,
            changed_artifacts_reviewed: manifest.changed_artifacts,
            input_tokens: Some(100),
            output_tokens: Some(100),
        }
    }

    fn finding(&self, kind: FindingKind) -> Finding {
        let (_, manifest) = load(&self.run).unwrap();
        let cite = |id: &str, locator: &str, excerpt: &str| Citation {
            artifact: id.into(),
            sha256: manifest
                .artifacts
                .iter()
                .find(|artifact| artifact.id == id)
                .unwrap()
                .sha256
                .clone(),
            locator: locator.into(),
            excerpt: excerpt.into(),
        };
        Finding {
            id: "scope".into(),
            kind,
            category: Category::AttributionScope,
            material: true,
            explicit_requirement: false,
            materiality_reason: "Scope changes the substantive claim".into(),
            confidence: 0.9,
            rule: cite(
                "rule:0",
                "line:1",
                "Preserve the scope of candidate evidence.",
            ),
            location: cite("cv:content", "json:/claim", "Participated in a project"),
            evidence: vec![cite(
                "source:candidate",
                "line:1",
                "Participated in a project",
            )],
            correction_constraint: "Retain supported scope".into(),
            resolution: Resolution::Open,
            verification: None,
        }
    }

    fn submit(&self, result: &ReviewResult) -> Result<Status> {
        let path = self.workspace.path("result.json");
        fs::write(&path, serde_json::to_vec(result).unwrap()).unwrap();
        run(
            &self.workspace,
            Command::Submit {
                run_dir: self.run.clone(),
                result: path,
            },
        )
    }
}

#[test]
fn ready_requires_current_independent_full_coverage() {
    let fixture = Fixture::new();
    let mut result = fixture.result();
    result.reviewer.agent_id = "actor".into();
    assert!(fixture.submit(&result).is_err());
    result.reviewer.agent_id = "critic".into();
    result
        .coverage
        .artifacts_read
        .retain(|id| id != "cv:page:1");
    assert_eq!(fixture.submit(&result).unwrap().state, "incomplete");
    assert!(
        run(
            &fixture.workspace,
            Command::BeginRevision {
                run_dir: fixture.run.clone()
            }
        )
        .is_err()
    );
    let clean = Fixture::new();
    assert_eq!(clean.submit(&clean.result()).unwrap().state, "ready");
    fs::write(clean.workspace.path("evidence.txt"), "Different evidence").unwrap();
    assert_eq!(
        status(&clean.workspace, &clean.run).unwrap().state,
        "incomplete"
    );
}

#[test]
fn contradictory_or_unknown_unavailable_coverage_is_not_accepted() {
    let fixture = Fixture::new();
    for unavailable in [
        vec!["unknown-artifact"],
        vec!["cv:page:1", "cv:page:1"],
        vec!["cv:content"],
    ] {
        let mut result = fixture.result();
        result
            .coverage
            .artifacts_read
            .retain(|id| id != "cv:page:1");
        result.coverage.unavailable = unavailable.into_iter().map(str::to_owned).collect();
        assert!(fixture.submit(&result).is_err());
        assert!(load(&fixture.run).unwrap().0.result_sha256.is_none());
    }
}

#[test]
fn partial_review_retains_findings_without_claiming_visual_completion() {
    let fixture = Fixture::new();
    let mut result = fixture.result();
    result
        .coverage
        .artifacts_read
        .retain(|id| id != "cv:page:1");
    result.coverage.unavailable.push("cv:page:1".into());
    result.coverage.presentation_pass = false;
    result.findings.push(fixture.finding(FindingKind::Error));
    assert_eq!(fixture.submit(&result).unwrap().state, "incomplete");
    let state = load(&fixture.run).unwrap().0;
    let accepted = accepted_result(&fixture.run, &state).unwrap().unwrap();
    assert_eq!(accepted.findings.len(), 1);
    assert!(accepted.coverage.evidence_pass);
    assert_eq!(accepted.coverage.unavailable, ["cv:page:1"]);
    assert!(!accepted.coverage.presentation_pass);
}

#[test]
fn findings_distinguish_uncertainty_error_and_explicit_preference() {
    for (kind, expected) in [
        (FindingKind::Error, "incomplete"),
        (FindingKind::Uncertainty, "needs-evidence"),
        (FindingKind::Preference, "needs-decision"),
    ] {
        let fixture = Fixture::new();
        let mut result = fixture.result();
        let mut finding = fixture.finding(kind);
        finding.explicit_requirement = true;
        result.findings.push(finding);
        assert_eq!(fixture.submit(&result).unwrap().state, expected);
    }
    let fixture = Fixture::new();
    let mut result = fixture.result();
    result
        .findings
        .push(fixture.finding(FindingKind::Preference));
    assert!(fixture.submit(&result).is_err());
    result.findings[0].material = false;
    assert_eq!(fixture.submit(&result).unwrap().state, "ready");
}

#[test]
fn rejects_fabricated_excerpt_stale_hash_and_wrong_locator() {
    let fixture = Fixture::new();
    let (state, manifest) = load(&fixture.run).unwrap();
    let mut result = fixture.result();
    result.findings.push(fixture.finding(FindingKind::Error));
    result.findings[0].evidence[0].excerpt = "Led the organization".into();
    assert!(validation::validate(&fixture.run, &state, &manifest, &result).is_err());
    result.findings[0] = fixture.finding(FindingKind::Error);
    result.findings[0].location.locator = "json:/absent".into();
    assert!(validation::validate(&fixture.run, &state, &manifest, &result).is_err());
    result.findings[0] = fixture.finding(FindingKind::Error);
    result.findings[0].rule.sha256 = "0".repeat(64);
    assert!(validation::validate(&fixture.run, &state, &manifest, &result).is_err());
}

#[test]
fn two_corrections_are_reserved_before_editing_and_final_revision_is_reviewed() {
    let mut fixture = Fixture::new();
    assert!(
        run(
            &fixture.workspace,
            Command::PrepareRevision {
                run_dir: fixture.run.clone()
            }
        )
        .is_err()
    );
    for revision in 0..=2 {
        let mut result = fixture.result();
        result.findings.push(fixture.finding(FindingKind::Error));
        assert_eq!(fixture.submit(&result).unwrap().state, "incomplete");
        let (state, _) = load(&fixture.run).unwrap();
        let reserve = run(
            &fixture.workspace,
            Command::BeginRevision {
                run_dir: fixture.run.clone(),
            },
        );
        if revision < 2 {
            assert_eq!(reserve.unwrap().corrections_used, revision + 1);
            assert!(
                run(
                    &fixture.workspace,
                    Command::BeginRevision {
                        run_dir: fixture.run.clone()
                    }
                )
                .is_err()
            );
            fixture.install(
                revision + 1,
                Some((state.manifest_sha256, state.result_sha256.unwrap())),
            );
        } else {
            assert!(reserve.is_err());
        }
    }
}

#[test]
fn prior_material_findings_require_verified_closure_and_regression_coverage() {
    let mut fixture = Fixture::new();
    let mut result = fixture.result();
    result.findings.push(fixture.finding(FindingKind::Error));
    fixture.submit(&result).unwrap();
    let (state, _) = load(&fixture.run).unwrap();
    fixture.install(
        1,
        Some((state.manifest_sha256, state.result_sha256.unwrap())),
    );
    let mut revised = fixture.result();
    assert!(fixture.submit(&revised).is_err());
    revised.findings.push(fixture.finding(FindingKind::Error));
    revised.findings[0].resolution = Resolution::FixedAndVerified;
    assert!(fixture.submit(&revised).is_err());
    revised.findings[0].verification = Some(
        "Independently read the current supported wording and checked every changed artifact"
            .into(),
    );
    revised.changed_artifacts_reviewed.clear();
    assert_eq!(fixture.submit(&revised).unwrap().state, "incomplete");
}

#[test]
fn cancel_deadline_token_budget_and_strict_result_schema_fail_closed() {
    let fixture = Fixture::new();
    let mut result = fixture.result();
    result.input_tokens = Some(10001);
    assert!(fixture.submit(&result).is_err());
    let mut value = serde_json::to_value(fixture.result()).unwrap();
    value["invented_pass"] = true.into();
    assert!(serde_json::from_value::<ReviewResult>(value).is_err());
    assert_eq!(
        run(
            &fixture.workspace,
            Command::Cancel {
                run_dir: fixture.run.clone()
            }
        )
        .unwrap()
        .state,
        "incomplete"
    );
    assert!(fixture.submit(&fixture.result()).is_err());
}

#[test]
fn run_paths_cannot_escape_private_review_locations() {
    let fixture = Fixture::new();
    for path in [
        "cvl/review/run",
        ".agent/cache/review/../../x",
        "opportunities/org/job/review/run/extra",
    ] {
        assert!(storage::run_path(&fixture.workspace, Path::new(path)).is_err());
    }
    assert!(
        storage::run_path(
            &fixture.workspace,
            Path::new("opportunities/org/job/review/run")
        )
        .is_ok()
    );
    assert!(fixture.directory.path().is_dir());
}

#[test]
fn unmeasured_usage_is_explicit_and_does_not_invent_telemetry() {
    let fixture = Fixture::new();
    let mut result = fixture.result();
    result.input_tokens = None;
    result.output_tokens = None;
    assert_eq!(fixture.submit(&result).unwrap().state, "ready");
    let mut value = serde_json::to_value(&result).unwrap();
    value.as_object_mut().unwrap().remove("input_tokens");
    assert!(serde_json::from_value::<ReviewResult>(value).is_err());
}

#[test]
fn every_reserved_mechanically_failed_revision_is_checked_and_consumed() {
    let fixture = Fixture::new();
    fixture.submit(&fixture.result()).unwrap();
    for revision in 1..=2 {
        run(
            &fixture.workspace,
            Command::BeginRevision {
                run_dir: fixture.run.clone(),
            },
        )
        .unwrap();
        // This fixture has no compilable style. Record the failed candidate
        // without losing or refunding its reserved correction.
        let outcome = run(
            &fixture.workspace,
            Command::PrepareRevision {
                run_dir: fixture.run.clone(),
            },
        )
        .unwrap();
        assert_eq!(outcome.state, "incomplete");
        assert_eq!(outcome.revision, revision);
        assert_eq!(outcome.corrections_used, revision);
        let (_, manifest) = load(&fixture.run).unwrap();
        assert!(
            manifest
                .checks
                .iter()
                .any(|check| !check.passed && check.operation == "render")
        );
    }
    assert!(
        run(
            &fixture.workspace,
            Command::BeginRevision {
                run_dir: fixture.run.clone()
            }
        )
        .is_err()
    );
}

#[test]
fn expired_external_review_remains_incomplete() {
    let fixture = Fixture::new();
    let (mut state, mut manifest) = load(&fixture.run).unwrap();
    manifest.created_at = storage::now().unwrap() - manifest.spec.time_limit_seconds - 1;
    let bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    fs::write(revision_path(&fixture.run, 0).join("manifest.json"), &bytes).unwrap();
    state.manifest_sha256 = storage::digest(&bytes);
    save_state(&fixture.run, &state).unwrap();
    assert!(fixture.submit(&fixture.result()).is_err());
    let outcome = status(&fixture.workspace, &fixture.run).unwrap();
    assert_eq!(outcome.state, "incomplete");
    assert!(
        outcome
            .reasons
            .iter()
            .any(|reason| reason.contains("deadline"))
    );
}

#[test]
fn failed_middle_revision_cannot_erase_prior_material_findings() {
    let mut fixture = Fixture::new();
    let mut result = fixture.result();
    result.findings.push(fixture.finding(FindingKind::Error));
    fixture.submit(&result).unwrap();
    let (first, _) = load(&fixture.run).unwrap();
    fixture.install(
        1,
        Some((first.manifest_sha256, first.result_sha256.unwrap())),
    );
    let (middle, _) = load(&fixture.run).unwrap();
    // No critic result for revision 1, as when it cannot render.
    fixture.install(2, Some((middle.manifest_sha256, String::new())));
    let (mut state, mut manifest) = load(&fixture.run).unwrap();
    manifest.parent_result = None;
    let bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    fs::write(revision_path(&fixture.run, 2).join("manifest.json"), &bytes).unwrap();
    state.manifest_sha256 = storage::digest(&bytes);
    save_state(&fixture.run, &state).unwrap();
    let mut current = fixture.result();
    assert!(fixture.submit(&current).is_err());
    current.findings.push(fixture.finding(FindingKind::Error));
    current.findings[0].resolution = Resolution::FixedAndVerified;
    current.findings[0].verification = Some(
        "Rechecked original finding against current wording, original source and all changed pages"
            .into(),
    );
    assert_eq!(fixture.submit(&current).unwrap().state, "ready");
    fs::write(revision_path(&fixture.run, 0).join("result.json"), "{}").unwrap();
    assert_eq!(
        status(&fixture.workspace, &fixture.run).unwrap().state,
        "incomplete"
    );
}
