use std::io::Write;

use super::*;

#[test]
fn actual_prepare_exports_readable_pdf_text_and_every_page_image() {
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    // Own the whole temporary opportunity so all created parents disappear,
    // including after a panic. Never leave cache directories in tested source.
    let opportunity = tempfile::Builder::new()
        .prefix("review-smoke-")
        .tempdir_in(workspace.path("opportunities"))
        .unwrap();
    let cache = opportunity.path().join("fixture/review");
    fs::create_dir_all(&cache).unwrap();
    let run_dir = tempfile::Builder::new()
        .prefix("smoke-")
        .tempdir_in(&cache)
        .unwrap();
    let mut evidence = tempfile::NamedTempFile::new_in(&cache).unwrap();
    writeln!(
        evidence,
        "Renderer smoke fixture; evidence support has not been semantically reviewed."
    )
    .unwrap();
    let mut request = tempfile::NamedTempFile::new_in(&cache).unwrap();
    let spec = Spec {
        purpose: Purpose {
            kind: PurposeKind::General,
            description: "Exercise actual packet rendering; do not claim semantic readiness".into(),
        },
        actor_id: "smoke-actor".into(),
        model: "external-smoke-critic".into(),
        max_input_tokens: 1000,
        max_output_tokens: 1000,
        time_limit_seconds: 300,
        documents: vec![Document {
            id: "cv".into(),
            document: "cv".into(),
            locale: "en-ch".into(),
            pages: 1,
            application: "cvl/cv/cluster/d-plus/en/ch/content.toml".into(),
            profile: "cvl/profile.toml".into(),
            style: Some("cluster".into()),
            substyle: Some("d-plus".into()),
            paper: None,
        }],
        sources: vec![Source {
            id: "smoke".into(),
            path: workspace.relative(evidence.path()).unwrap(),
            role: SourceRole::Confirmation,
        }],
        rubric: vec![".agent/docs/editorial.md".into()],
        editable: vec![],
    };
    request
        .write_all(&serde_json::to_vec(&spec).unwrap())
        .unwrap();
    let outcome = run(
        &workspace,
        Command::Prepare {
            spec: request.path().to_path_buf(),
            run_dir: run_dir.path().to_path_buf(),
        },
    )
    .unwrap();
    assert_eq!(outcome.state, "incomplete");
    assert!(
        outcome
            .reasons
            .iter()
            .any(|reason| reason.contains("awaiting independent")),
        "{:?}",
        outcome.reasons
    );
    let (_, manifest) = load(run_dir.path()).unwrap();
    assert!(
        manifest.checks.iter().all(|check| check.passed),
        "{:?}",
        manifest.checks
    );
    let pages = manifest
        .artifacts
        .iter()
        .filter(|artifact| artifact.role == "page")
        .collect::<Vec<_>>();
    assert_eq!(pages.len(), 1);
    let page = fs::File::open(storage::artifact_path(run_dir.path(), pages[0]).unwrap()).unwrap();
    let image = png::Decoder::new(std::io::BufReader::new(page))
        .read_info()
        .unwrap();
    assert!(image.info().width > 500 && image.info().height > 500);
    let text = manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.id == "cv:text:1")
        .unwrap();
    assert!(
        !fs::read_to_string(storage::artifact_path(run_dir.path(), text).unwrap())
            .unwrap()
            .trim()
            .is_empty()
    );
    assert!(
        manifest
            .artifacts
            .iter()
            .any(|artifact| artifact.id == "cv:pdf")
    );
    assert!(
        manifest
            .artifacts
            .iter()
            .any(|artifact| artifact.id == "cv:correspondence")
    );
}
