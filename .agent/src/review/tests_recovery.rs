use super::*;

#[test]
fn saved_review_passes_repository_hygiene_without_changing_hashes() {
    let mut fixture = Fixture::new();
    let run_dir = fixture
        .workspace
        .path("opportunities/example/role/review/run");
    fs::create_dir_all(run_dir.parent().unwrap()).unwrap();
    fs::rename(&fixture.run, &run_dir).unwrap();
    fixture.run = run_dir;
    storage::write_new(&fixture.run.join("request.json"), &Fixture::spec()).unwrap();
    let result_path = fixture.run.parent().unwrap().join("result-0.json");
    storage::write_new(&result_path, &fixture.result()).unwrap();
    let submitted = run(
        &fixture.workspace,
        Command::Submit {
            run_dir: fixture.run.clone(),
            result: result_path,
        },
    )
    .unwrap();
    assert_eq!(submitted.state, "ready");
    let paths = crate::public::public_files(&fixture.workspace).unwrap();
    let before = paths
        .iter()
        .map(|path| fs::read(path).unwrap())
        .collect::<Vec<_>>();
    crate::public::validate_repository(&fixture.workspace).unwrap();
    for (path, bytes) in paths.iter().zip(before) {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    let verified = status(&fixture.workspace, &fixture.run).unwrap();
    assert_eq!(verified.state, "ready");
    assert_eq!(verified.manifest_sha256, submitted.manifest_sha256);

    // Even an otherwise harmless newline must still invalidate a hashed record.
    let manifest = revision_path(&fixture.run, 0).join("manifest.json");
    let mut bytes = fs::read(&manifest).unwrap();
    bytes.push(b'\n');
    fs::write(manifest, bytes).unwrap();
    assert!(
        status(&fixture.workspace, &fixture.run)
            .unwrap_err()
            .to_string()
            .contains("review manifest changed")
    );
}

#[test]
fn json_citations_use_decoded_quotes_backslashes_and_newlines() {
    let fixture = Fixture::new();
    let (mut state, mut manifest) = load(&fixture.run).unwrap();
    let passage = "Built a \"quoted\" C:\\tool\nwith measured evidence";
    let artifact = storage::freeze(
        &fixture.run,
        0,
        "cv:content",
        "content",
        &serde_json::to_vec(&serde_json::json!({"claim": passage})).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    *manifest
        .artifacts
        .iter_mut()
        .find(|artifact| artifact.id == "cv:content")
        .unwrap() = artifact;
    let bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    fs::write(revision_path(&fixture.run, 0).join("manifest.json"), &bytes).unwrap();
    state.manifest_sha256 = storage::digest(&bytes);
    save_state(&fixture.run, &state).unwrap();
    let mut result = fixture.result();
    let mut finding = fixture.finding(FindingKind::Uncertainty);
    finding.location.excerpt = passage.into();
    result.findings.push(finding);
    assert_eq!(fixture.submit(&result).unwrap().state, "needs-evidence");
}

#[test]
fn interrupted_pending_revision_retries_without_another_correction() {
    let fixture = Fixture::new();
    fixture.submit(&fixture.result()).unwrap();
    run(
        &fixture.workspace,
        Command::BeginRevision {
            run_dir: fixture.run.clone(),
        },
    )
    .unwrap();
    let (pending, _) = load(&fixture.run).unwrap();
    fs::create_dir_all(revision_path(&fixture.run, 1).join("objects")).unwrap();
    fs::write(
        revision_path(&fixture.run, 1).join("partial.pdf"),
        "interrupted bytes",
    )
    .unwrap();
    let first = run(
        &fixture.workspace,
        Command::PrepareRevision {
            run_dir: fixture.run.clone(),
        },
    )
    .unwrap();
    assert_eq!(first.revision, 1);
    assert_eq!(first.corrections_used, 1);
    assert_eq!(
        fs::read(fixture.run.join("interrupted-revision-1-0/partial.pdf")).unwrap(),
        b"interrupted bytes"
    );
    // Simulate interruption after manifest publication but before state update.
    save_state(&fixture.run, &pending).unwrap();
    let resumed = run(
        &fixture.workspace,
        Command::PrepareRevision {
            run_dir: fixture.run.clone(),
        },
    )
    .unwrap();
    assert_eq!(resumed.manifest_sha256, first.manifest_sha256);
    assert_eq!(resumed.corrections_used, 1);
}

#[test]
fn interrupted_initial_prepare_keeps_its_pinned_specification() {
    let fixture = Fixture::new();
    let run_dir = fixture.workspace.path(".agent/cache/review/interrupted");
    fs::create_dir_all(run_dir.join("revision-0/objects")).unwrap();
    let spec = Fixture::spec();
    storage::write_new(&run_dir.join("request.json"), &spec).unwrap();
    let source = fixture.workspace.path("request.json");
    fs::write(&source, serde_json::to_vec(&spec).unwrap()).unwrap();
    let outcome = run(
        &fixture.workspace,
        Command::Prepare {
            spec: source,
            run_dir: run_dir.clone(),
        },
    )
    .unwrap();
    assert_eq!(outcome.revision, 0);
    assert_eq!(outcome.corrections_used, 0);
    assert!(run_dir.join("interrupted-revision-0-0").is_dir());
}

#[test]
fn interrupted_submission_resumes_only_the_identical_result() {
    let fixture = Fixture::new();
    let result = fixture.result();
    fixture.submit(&result).unwrap();
    let (mut state, _) = load(&fixture.run).unwrap();
    state.result_sha256 = None;
    save_state(&fixture.run, &state).unwrap();
    let mut changed = fixture.result();
    changed.output_tokens = None;
    assert!(fixture.submit(&changed).is_err());
    assert_eq!(fixture.submit(&result).unwrap().state, "ready");
}

#[test]
fn initial_directory_and_owned_temporary_files_are_resumable() {
    let fixture = Fixture::new();
    let run_dir = fixture.workspace.path(".agent/cache/review/empty");
    fs::create_dir_all(&run_dir).unwrap();
    fs::write(run_dir.join(".review-tmp-interrupted"), "partial request").unwrap();
    let source = fixture.workspace.path("request.json");
    fs::write(&source, serde_json::to_vec(&Fixture::spec()).unwrap()).unwrap();
    let outcome = run(
        &fixture.workspace,
        Command::Prepare {
            spec: source,
            run_dir: run_dir.clone(),
        },
    )
    .unwrap();
    assert_eq!(outcome.revision, 0);
    assert!(run_dir.join(".review-tmp-interrupted").is_file());
    assert!(
        storage::write_new(
            &run_dir.join("request.json"),
            &serde_json::json!({"replacement":true})
        )
        .is_err()
    );
    assert_eq!(
        read::<Spec>(&run_dir.join("request.json")).unwrap(),
        Fixture::spec()
    );
}
