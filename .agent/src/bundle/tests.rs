use super::*;
use ccvl_core::render::StyleRenderer;

fn source() -> Workspace {
    Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn bundles_exclude_showcase_records_and_unsupported_inputs() {
    let workspace = source();
    let bundle = export(
        &workspace,
        "cv",
        "en-ch",
        Some("cluster"),
        Some("d-plus"),
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
    assert!(bundle.files.contains_key("cvl/cv/cluster/layout.typ"));
    let mut bad = bundle.clone();
    bad.engine_api += 1;
    assert!(bad.validate().is_err());
    assert!(export(&source(), "cv", "en-ch", Some("harvard"), None, None, None).is_err());
    let mut bad = bundle;
    bad.files.insert("../escape".into(), vec![]);
    assert!(bad.validate().is_err());
}

#[test]
fn cluster_bundles_render_resolved_content_with_an_explicit_summary_allowance() {
    let workspace = source();
    fs::create_dir_all(workspace.path(".agent/cache")).unwrap();
    let scratch = tempfile::tempdir_in(workspace.path(".agent/cache")).unwrap();
    let record_path = scratch.path().join("application.toml");
    let profile_path = workspace.path("cvl/profile.toml");
    for substyle in ["standard", "middle-three", "middle-three-spaced", "d-plus"] {
        for language in ["de", "en"] {
            let locale = format!("{language}-ch");
            let mut record = crate::content::read_record(
                &workspace,
                workspace.path(format!(
                    "cvl/cv/cluster/{substyle}/{language}/ch/content.toml"
                )),
            )
            .unwrap();
            record["cv"]["allow_thin"] = json!(true);
            crate::application::validate_record(&workspace, &record, "fixture", true).unwrap();
            fs::write(&record_path, toml::to_string(&record).unwrap()).unwrap();
            let bundle = export(
                &workspace,
                "cv",
                &locale,
                Some("cluster"),
                Some(substyle),
                Some(1),
                None,
            )
            .unwrap();
            let profile = workspace.read_toml_value("cvl/profile.toml").unwrap();
            let project = bundle.with_record(record, profile).unwrap();
            let renderer = StyleRenderer::new(&project.bundle).unwrap();
            let portable_document = renderer.compile(&project).unwrap();
            let selection =
                styles::selection(&workspace, "cv", Some("cluster"), Some(substyle)).unwrap();
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
                "portable cluster parity: {substyle}/{locale}"
            );
            if let Some(directory) = std::env::var_os("CCVL_STYLE_EVIDENCE") {
                let directory = std::path::PathBuf::from(directory)
                    .join(format!("cluster-{substyle}-{locale}-a4"));
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
            let key = "/record/cv/groups";
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
            assert_eq!(edited.record["cv"]["groups"].as_array().unwrap().len(), 0);
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
    }
}
