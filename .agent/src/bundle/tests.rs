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
    let mut bad = bundle;
    bad.files.insert("../escape".into(), vec![]);
    assert!(bad.validate().is_err());
}

#[test]
fn upstream_and_portable_renderers_produce_the_same_document() {
    let workspace = source();
    let scratch = tempfile::tempdir_in(workspace.root()).unwrap();
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
                Some(1),
                Some(paper),
            )
            .unwrap();
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
                    "cl_style":style,"cl_substyle":substyle,"cl_pages":1,"cl_paper":paper},
                "job":job,"cl":{"subject":"Style integration fixture", "opening":"Synthetic document for rendering verification.",
                    "body":["This is a test fixture. It contains no candidate claims.", "The same upstream assets must render in both consumers."],
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
                1,
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
            let mut changed = project.clone();
            std::sync::Arc::make_mut(&mut changed.bundle)
                .files
                .get_mut(&changed.bundle.entry.clone())
                .unwrap()
                .push(b' ');
            assert!(changed.validate().is_err());
        }
    }
}
