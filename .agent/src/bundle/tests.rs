use super::*;
use ccvl_core::render::StyleRenderer;

fn source() -> Workspace {
    Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn bundles_exclude_showcase_records_and_unsupported_inputs() {
    let bundle = export(
        &source(),
        "cl",
        "en-ch",
        Some("test-style-1"),
        Some("sidebar"),
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
    assert!(bundle.files.contains_key("cvl/cl/test-style-1/layout.typ"));
    let mut bad = bundle.clone();
    bad.engine_api += 1;
    assert!(bad.validate().is_err());
    assert!(export(&source(), "cv", "en-ch", Some("harvard"), None, None, None).is_err());
    let mut bad = bundle;
    bad.files.insert("../escape".into(), vec![]);
    assert!(bad.validate().is_err());
}

#[test]
fn cluster_bundle_renders_resolved_content_with_an_explicit_summary_allowance() {
    let workspace = source();
    fs::create_dir_all(workspace.path(".agent/cache")).unwrap();
    let scratch = tempfile::tempdir_in(workspace.path(".agent/cache")).unwrap();
    let record_path = scratch.path().join("application.toml");
    let profile_path = workspace.path("cvl/profile.toml");
    for language in ["de", "en"] {
        let locale = format!("{language}-ch");
        let mut record = crate::content::read_record(
            &workspace,
            workspace.path(format!("cvl/cv/cluster/d-plus/{language}/ch/content.toml")),
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
            Some("d-plus"),
            Some(1),
            None,
        )
        .unwrap();
        let profile = workspace.read_toml_value("cvl/profile.toml").unwrap();
        let project = bundle.with_record(record, profile).unwrap();
        let renderer = StyleRenderer::new(&project.bundle).unwrap();
        let portable_document = renderer.compile(&project).unwrap();
        let selection =
            styles::selection(&workspace, "cv", Some("cluster"), Some("d-plus")).unwrap();
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
            "portable cluster parity: {locale}"
        );
    }
}

#[test]
fn upstream_and_portable_renderers_produce_the_same_document() {
    let workspace = source();
    fs::create_dir_all(workspace.path(".agent/cache")).unwrap();
    let scratch = tempfile::tempdir_in(workspace.path(".agent/cache")).unwrap();
    let record_path = scratch.path().join("application.toml");
    let profile_path = scratch.path().join("profile.toml");
    let profile = json!({"schema_version":1,"name":"Style integration fixture", "email":"fixture@example.invalid", "location":"Synthetic test input"});
    fs::write(&profile_path, toml::to_string(&profile).unwrap()).unwrap();
    for (style, substyle) in [
        ("test-style-1", "sidebar"),
        ("test-style-1", "topbar"),
        ("test-style-2", "cards"),
        ("test-style-2", "timeline"),
    ] {
        for (locale, paper) in [
            ("en-ch", "a4"),
            ("en-ch", "us-letter"),
            ("en-us", "us-letter"),
        ] {
            let bundle = export(
                &workspace,
                "cl",
                locale,
                Some(style),
                Some(substyle),
                None,
                Some(paper),
            )
            .unwrap();
            let pages = bundle.pages;
            let mut job = json!({});
            for key in [
                "id",
                "title",
                "organization",
                "location",
                "source",
                "url",
                "description",
                "connections",
                "company_context",
                "notes",
            ] {
                job[key] = "".into();
            }
            job["id"] = "fixture".into();
            job["cl_recipient"] =
                json!({"name":"","title":"","company":"","address_line_1":"","address_line_2":""});
            let record = json!({"schema_version":4,"revision":0,
                "options":{"language":locale,"pages":1,"generate_cl":true,"application_date":"2026-01-01",
                    "cl_style":style,"cl_substyle":substyle,"cl_pages":pages,"cl_paper":paper},
                "job":job,"cl":{"subject":"Style integration fixture", "opening":"Synthetic document for rendering verification.",
                    "body":["This is a test fixture. It contains no candidate claims.", "The same upstream assets must render in both consumers.", "This third section verifies the two-page composition."],
                    "closing":"End of fixture"}});
            fs::write(&record_path, toml::to_string(&record).unwrap()).unwrap();
            let project = bundle.with_record(record, profile.clone()).unwrap();
            let portable = StyleRenderer::new(&project.bundle).unwrap();
            let bundled = portable.compile(&project).unwrap();
            let selection =
                styles::selection(&workspace, "cl", Some(style), Some(substyle)).unwrap();
            let leaf = styles::leaf(&workspace, "cl", locale, &selection).unwrap();
            let spec = crate::render::document_spec(
                &workspace,
                &leaf,
                pages,
                &record_path,
                &profile_path,
                Some(&scratch.path().join("direct.pdf")),
                Some(paper),
            )
            .unwrap();
            let compiler = crate::render::Compiler::new(&workspace).unwrap();
            let direct = compiler.compile(&workspace, &spec).unwrap();
            compiler.export(&spec, &direct).unwrap();
            assert_eq!(
                portable.pdf(&bundled).unwrap(),
                fs::read(&spec.output).unwrap(),
                "{style}/{substyle}/{locale}/{paper}"
            );
            if let Some(directory) = std::env::var_os("CCVL_STYLE_EVIDENCE") {
                let directory = std::path::PathBuf::from(directory)
                    .join(format!("{style}-{substyle}-{locale}-{paper}"));
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
            let key = "/record/cl/body";
            let mut edited = project.clone();
            let before = edited.record.clone();
            assert!(edited.set_field(key, "[invalid JSON").is_err());
            assert_eq!(edited.record, before);
            assert!(
                edited
                    .set_field("/record/options/cl_style", "another")
                    .is_err()
            );
            edited
                .set_field(key, r#"["A", "B", "C", "D", "E", "F"]"#)
                .unwrap();
            assert_eq!(edited.record["cl"]["body"].as_array().unwrap().len(), 6);
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
