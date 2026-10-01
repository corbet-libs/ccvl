use super::*;
use ccvl_core::render::StyleRenderer;

/// Bundles export from the synthetic fixture; the real exportable styles'
/// portable parity is a `check` gate.
fn source() -> (tempfile::TempDir, Workspace) {
    let (directory, workspace) = crate::test_support::fixture_workspace();
    // Exports carry the real license metadata and texts.
    crate::test_support::copy_repository(workspace.root(), "REUSE.toml");
    crate::test_support::copy_repository(workspace.root(), "LICENSES");
    (directory, workspace)
}

#[test]
fn bundles_exclude_showcase_records_and_unsupported_inputs() {
    let (_fixture, workspace) = source();
    let bundle = export(
        &workspace,
        "cv",
        "en-ch",
        Some("grid"),
        Some("wide"),
        None,
        None,
    )
    .unwrap();
    assert!(
        !bundle
            .files
            .keys()
            .any(|path| path.ends_with("content.toml")
                || path.ends_with("wording.toml")
                || path.contains("/preview/"))
    );
    assert!(!bundle.files.contains_key("cvl/profile.toml"));
    assert!(bundle.files.contains_key("cvl/cv/grid/layout.typ"));
    let mut bad = bundle.clone();
    bad.engine_api += 1;
    assert!(bad.validate().is_err());
    // A style embedding candidate source files cannot become a reusable bundle.
    let contract = workspace.path("cvl/cv/ledger/contract.toml");
    let text = fs::read_to_string(&contract).unwrap();
    fs::write(
        &contract,
        format!("source_files = [\"src/entries.typ\"]\n{text}"),
    )
    .unwrap();
    assert!(export(&workspace, "cv", "en-ch", Some("ledger"), None, None, None).is_err());
    fs::write(&contract, text).unwrap();
    export(&workspace, "cv", "en-ch", Some("ledger"), None, None, None).unwrap();
    let mut bad = bundle;
    bad.files.insert("../escape".into(), vec![]);
    assert!(bad.validate().is_err());
}

#[test]
fn portable_bundles_render_resolved_content_with_an_explicit_summary_allowance() {
    let (_fixture, workspace) = source();
    fs::create_dir_all(workspace.path(".agent/cache")).unwrap();
    let scratch = tempfile::tempdir_in(workspace.path(".agent/cache")).unwrap();
    let record_path = scratch.path().join("application.toml");
    let profile_path = workspace.path("cvl/profile.toml");
    let substyle = "wide";
    let language = "en";
    let locale = format!("{language}-ch");
    let mut record = crate::content::read_record(
        &workspace,
        workspace.path(format!("cvl/cv/grid/{substyle}/{language}/ch/content.toml")),
    )
    .unwrap();
    record["cv"]["allow_thin"] = json!(true);
    crate::application::validate_record(&workspace, &record, "fixture", true).unwrap();
    fs::write(&record_path, toml::to_string(&record).unwrap()).unwrap();
    let bundle = export(
        &workspace,
        "cv",
        &locale,
        Some("grid"),
        Some(substyle),
        Some(1),
        None,
    )
    .unwrap();
    let profile = workspace.read_toml_value("cvl/profile.toml").unwrap();
    let project = bundle.with_record(record, profile).unwrap();
    let renderer = StyleRenderer::new(&project.bundle).unwrap();
    let portable_document = renderer.compile(&project).unwrap();
    let selection = styles::selection(&workspace, "cv", Some("grid"), Some(substyle)).unwrap();
    let leaf = styles::leaf(&workspace, "cv", &locale, &selection).unwrap();
    let spec = crate::render::document_spec(
        &workspace,
        &leaf,
        1,
        &record_path,
        &profile_path,
        Some(&scratch.path().join("direct.pdf")),
        None,
    )
    .unwrap();
    let compiler = crate::render::Compiler::new(&workspace).unwrap();
    let direct = compiler.compile(&workspace, &spec).unwrap();
    compiler.export(&spec, &direct).unwrap();
    assert_eq!(
        renderer.pdf(&portable_document).unwrap(),
        fs::read(&spec.output).unwrap(),
        "portable parity: {substyle}/{locale}"
    );
    if let Some(directory) = std::env::var_os("CCVL_STYLE_EVIDENCE") {
        let directory =
            std::path::PathBuf::from(directory).join(format!("grid-{substyle}-{locale}-a4"));
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("bundle.json"),
            serde_json::to_vec(&project.bundle).unwrap(),
        )
        .unwrap();
        fs::copy(&record_path, directory.join("application.toml")).unwrap();
        fs::copy(&profile_path, directory.join("profile.toml")).unwrap();
        fs::copy(&spec.output, directory.join("expected.pdf")).unwrap();
    }
    let key = "/record/cv/items";
    let mut edited = project.clone();
    let before = edited.record.clone();
    assert!(edited.set_field(key, "[invalid JSON").is_err());
    assert_eq!(edited.record, before);
    assert!(
        edited
            .set_field("/record/options/cv_style", "another")
            .is_err()
    );
    edited.set_field(key, "[]").unwrap();
    assert_eq!(edited.record["cv"]["items"].as_array().unwrap().len(), 0);
    assert!(edited.editing_context().get("files").is_none());
    let mut changed = project.clone();
    let entry = changed.bundle.entry.clone();
    std::sync::Arc::make_mut(&mut changed.bundle)
        .files
        .get_mut(&entry)
        .unwrap()
        .push(b' ');
    assert!(changed.validate().is_err());
}
