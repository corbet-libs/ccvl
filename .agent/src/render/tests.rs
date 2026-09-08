use tempfile::tempdir;

use super::*;

#[test]
fn cvl_outputs_use_numeric_page_names() {
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    for (locale, lang) in [("de-ch", "de"), ("en-ch", "en")] {
        for substyle in ["standard", "compact"] {
            for pages in [2, 3, 4] {
                let spec =
                    cvl_cv_spec(&workspace, locale, pages, Some(&selection(substyle))).unwrap();
                assert_eq!(
                    spec.output,
                    workspace.path(format!(
                        "cvl/cv/harvard/{substyle}/{lang}/ch/pdf/cv-{pages}.pdf"
                    ))
                );
                assert_eq!(
                    spec.source,
                    workspace.path(format!("cvl/cv/harvard/{substyle}/{lang}/ch/typst/cv.typ"))
                );
            }
        }
        for substyle in ["left-rule", "frame"] {
            let spec = cvl_cl_spec(&workspace, locale, None, Some(&selection(substyle))).unwrap();
            assert_eq!(
                spec.output,
                workspace.path(format!("cvl/cl/harvard/{substyle}/{lang}/ch/pdf/cl.pdf"))
            );
        }
    }
    assert!(cvl_cv_spec(&workspace, "en-ch", 1, None).is_err());
    assert!(cvl_cv_spec(&workspace, "en-ch", 5, None).is_err());
    assert!(cvl_cv_spec(&workspace, "en-ch", 4, Some(&selection("nope"))).is_err());
    assert!(cvl_cl_spec(&workspace, "en-ch", None, Some(&selection("nope"))).is_err());
}

#[test]
fn opportunity_record_selects_its_locale_pages_and_documents() {
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let mut document = workspace
        .read_toml_value("cvl/cv/harvard/standard/en/ch/content.toml")
        .unwrap();
    document["options"]["language"] = "en-ch".into();
    document["options"]["pages"] = 3.into();
    document["options"]["generate_cl"] = false.into();
    document.as_object_mut().unwrap().remove("cl");
    assert_eq!(
        opportunity_options(&document).unwrap(),
        OpportunityOptions {
            locale: "en-ch".to_owned(),
            pages: 3,
            cover_letter: false,
        }
    );
}

#[test]
fn disabled_cover_letter_removes_stale_output() {
    let directory = tempdir().unwrap();
    let pdfs = directory.path().join("pdfs");
    let typst = directory.path().join("typst");
    fs::create_dir_all(&pdfs).unwrap();
    fs::create_dir_all(&typst).unwrap();
    let stale_pdf = pdfs.join("cl.pdf");
    let stale_typ = typst.join("cl.typ");
    fs::write(&stale_pdf, b"stale").unwrap();
    fs::write(&stale_typ, b"stale").unwrap();
    remove_stale_cover_letter(&pdfs, &typst, false).unwrap();
    assert!(!stale_pdf.exists());
    assert!(!stale_typ.exists());

    fs::write(&stale_pdf, b"current").unwrap();
    fs::write(&stale_typ, b"current").unwrap();
    remove_stale_cover_letter(&pdfs, &typst, true).unwrap();
    assert!(stale_pdf.exists());
    assert!(stale_typ.exists());
}

#[test]
fn input_default_rewrite_points_copy_at_record() {
    let cv = "#let pages = int(sys.inputs.at(\"pages\", default: \"4\"))\n\
                  #let application-path = sys.inputs.at(\"application\", default: \"/cvl/cv/harvard/standard/en/ch/content.toml\")\n";
    let resolved = rewrite_input_default(
        cv,
        "application",
        "/opportunities/acme/lead/application.toml",
    );
    let resolved = rewrite_input_default(&resolved, "pages", "3");
    assert!(resolved.contains(
        "sys.inputs.at(\"application\", default: \"/opportunities/acme/lead/application.toml\")"
    ));
    assert!(resolved.contains("sys.inputs.at(\"pages\", default: \"3\")"));
    assert!(!resolved.contains("/cvl/cv/harvard/standard/en/ch/content.toml"));

    let cl = "#let substyle-path = sys.inputs.at(\"substyle\", default: \"/cvl/cl/harvard/left-rule/de/ch/substyle.toml\")\n";
    let resolved =
        rewrite_input_default(cl, "substyle", "/cvl/cl/harvard/frame/de/ch/substyle.toml");
    assert!(resolved.contains(
        "sys.inputs.at(\"substyle\", default: \"/cvl/cl/harvard/frame/de/ch/substyle.toml\")"
    ));
    let shared = "#let shared-defaults-path = sys.inputs.at(\"shared-defaults\", default: \"/cvl/shared/harvard/defaults.toml\")\n";
    assert_eq!(
        rewrite_input_default(
            shared,
            "shared-defaults",
            "/cvl/shared/harvard/defaults.toml"
        ),
        shared
    );
    // A template that no longer carries the input survives unchanged.
    assert_eq!(
        rewrite_input_default("#let x = 1\n", "application", "/elsewhere.toml"),
        "#let x = 1\n"
    );
}

#[test]
fn resolved_copy_carries_provenance_and_record_defaults() {
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let leaf = cv_leaf(&workspace, "en-ch", &selection("standard")).unwrap();
    let output = tempdir().unwrap().path().join("pdfs").join("cv.pdf");
    let mut spec = cv_spec(
        &workspace,
        "en-ch",
        3,
        &leaf.content(),
        &workspace.path("cvl/profile.toml"),
        &output,
        &selection("standard"),
    )
    .unwrap();
    spec.inputs.insert(
        "application".to_owned(),
        "/opportunities/acme/lead/application.toml".to_owned(),
    );
    let template = fs::read_to_string(leaf.adapter()).unwrap();
    let text = resolved_typ_text(
        &template,
        &spec,
        "cvl/cv/harvard/standard/en/ch/typst/cv.typ",
        "acme",
        "lead",
    );
    assert!(text.starts_with(
        "// Resolved customization copy emitted by `ccvl build-opportunity acme lead`."
    ));
    assert!(text.contains("Template: cvl/cv/harvard/standard/en/ch/typst/cv.typ"));
    assert!(text.contains("application: /opportunities/acme/lead/application.toml"));
    assert!(text.contains("pages: 3"));
    assert!(text.contains("// strings: "));
    assert!(text.contains("// substyle: "));
    assert!(text.contains("// shared-defaults: "));
    assert!(!text.contains("| style: "));
    assert!(text.ends_with('\n'));
    assert!(!text.contains("default: \"/cvl/cv/harvard/standard/en/ch/content.toml\""));
    assert!(text.contains("sys.inputs.at(\"pages\", default: \"3\")"));
}

#[test]
fn emitted_copies_compile_without_cli_inputs() {
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let compiler = Compiler::new(&workspace).unwrap();
    for locale in ["de-ch", "en-ch"] {
        let mut specs = Vec::new();
        for substyle in ["standard", "compact"] {
            specs.push(cvl_cv_spec(&workspace, locale, 3, Some(&selection(substyle))).unwrap());
        }
        for substyle in ["left-rule", "frame"] {
            specs.push(cvl_cl_spec(&workspace, locale, None, Some(&selection(substyle))).unwrap());
        }
        for spec in specs {
            let original = compiler.compile(&workspace, &spec).unwrap();
            let template = fs::read_to_string(&spec.source).unwrap();
            let text = resolved_typ_text(&template, &spec, "fixture", "acme", "lead");
            let standalone = compiler
                .engine
                .compile(
                    CompileRequest::new("standalone.typ")
                        .source_file("standalone.typ", text)
                        .pages(PageConstraint::Exactly(spec.expected_pages)),
                )
                .unwrap();
            assert_eq!(
                compiler.engine.pdf(&original, 0).unwrap(),
                compiler.engine.pdf(&standalone.document, 0).unwrap(),
                "standalone copy differs: {}",
                spec.name,
            );
        }
    }
}

#[test]
fn cvl_specs_carry_their_leaf_paths() {
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    for pages in [2, 3, 4] {
        let spec = cvl_cv_spec(&workspace, "en-ch", pages, Some(&selection("compact"))).unwrap();
        assert_eq!(
            spec.inputs.get("strings").map(String::as_str),
            Some("/cvl/cv/harvard/compact/en/ch/strings.toml")
        );
        assert_eq!(
            spec.inputs.get("substyle").map(String::as_str),
            Some("/cvl/cv/harvard/compact/substyle.toml")
        );
        assert_eq!(
            spec.inputs.get("shared-defaults").map(String::as_str),
            Some("/cvl/shared/harvard/defaults.toml")
        );
        assert!(!spec.inputs.contains_key("style"));
    }
    // No explicit substyle renders the family default.
    let spec = cvl_cv_spec(&workspace, "en-ch", 4, None).unwrap();
    assert_eq!(
        spec.inputs.get("substyle").map(String::as_str),
        Some("/cvl/cv/harvard/standard/substyle.toml")
    );
    let spec = cvl_cl_spec(&workspace, "de-ch", None, None).unwrap();
    assert_eq!(
        spec.inputs.get("substyle").map(String::as_str),
        Some("/cvl/cl/harvard/left-rule/substyle.toml")
    );
    let spec = cvl_cl_spec(&workspace, "de-ch", None, Some(&selection("frame"))).unwrap();
    assert_eq!(
        spec.inputs.get("strings").map(String::as_str),
        Some("/cvl/cl/harvard/frame/de/ch/strings.toml")
    );
}

#[test]
fn mismatched_record_selection_fails_before_compiling() {
    // A record selecting compact must not render through the standard
    // leaf: the mismatch fails in Rust with both names.
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let leaf = cv_leaf(&workspace, "en-ch", &selection("standard")).unwrap();
    let directory = tempfile::tempdir_in(workspace.root()).unwrap();
    let record = directory.path().join("application.toml");
    let text = fs::read_to_string(leaf.content())
        .unwrap()
        .replace("cv_substyle = \"standard\"", "cv_substyle = \"compact\"");
    fs::write(&record, text).unwrap();
    let output = directory.path().join("cv.pdf");
    let error = cv_spec(
        &workspace,
        "en-ch",
        2,
        &record,
        &workspace.path("cvl/profile.toml"),
        &output,
        &selection("standard"),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("compact"), "unexpected error: {error}");
    assert!(error.contains("standard"), "unexpected error: {error}");
}

#[test]
fn style_major_leaves_compile() {
    // The default leaves render end to end through the shared renderers
    // under exact page constraints.
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let compiler = Compiler::new(&workspace).unwrap();
    let cv = cvl_cv_spec(&workspace, "de-ch", 2, None).unwrap();
    compiler.compile(&workspace, &cv).unwrap();
    let cl = cvl_cl_spec(&workspace, "de-ch", None, None).unwrap();
    compiler.compile(&workspace, &cl).unwrap();
}

#[test]
fn compact_substyles_pass_the_same_measurement_gates() {
    use crate::measure::{document_metrics, line_failure, summary_failures};

    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let compiler = Compiler::new(&workspace).unwrap();
    // Two-page CV exercises pages 1-2; the cover letter exercises the
    // vertical-rhythm gate. Both compile under an exact page constraint.
    let mut cv = cvl_cv_spec(&workspace, "en-ch", 2, Some(&selection("compact"))).unwrap();
    cv.inputs
        .insert("line-contracts".to_owned(), "report".to_owned());
    let document = compiler.compile(&workspace, &cv).unwrap();
    let metrics = document_metrics(&workspace, &cv, &document).unwrap();
    assert!(
        summary_failures(&workspace, &cv, &metrics)
            .unwrap()
            .is_empty()
    );
    for (index, metric) in metrics.iter().enumerate() {
        assert!(
            line_failure(&cv, index, metric).unwrap().is_none(),
            "compact CV failure at #{index}: {metric}"
        );
    }

    for substyle in ["left-rule", "frame"] {
        let mut cl = cvl_cl_spec(&workspace, "en-ch", None, Some(&selection(substyle))).unwrap();
        cl.inputs
            .insert("line-contracts".to_owned(), "report".to_owned());
        let document = compiler.compile(&workspace, &cl).unwrap();
        let metrics = document_metrics(&workspace, &cl, &document).unwrap();
        for (index, metric) in metrics.iter().enumerate() {
            assert!(
                line_failure(&cl, index, metric).unwrap().is_none(),
                "{substyle} cover-letter failure at #{index}: {metric}"
            );
        }
    }
}

#[test]
fn compact_delta_keeps_horizontal_measure_and_accents() {
    // Horizontal measure feeds every fill percentage, so the compact
    // delta may only tighten vertical whitespace. Pin the invariant
    // between the shared base knobs and the delta; the gates above prove
    // the result still passes.
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let base = workspace
        .read_toml_value("cvl/shared/harvard/defaults.toml")
        .unwrap();
    let compact = workspace
        .read_toml_value("cvl/cv/harvard/compact/substyle.toml")
        .unwrap();
    let delta = compact.as_object().unwrap();
    for forbidden in ["page", "text", "accents"] {
        assert!(
            !delta.contains_key(forbidden),
            "horizontal section changed by the delta: {forbidden}"
        );
    }
    let cv = delta.get("cv").and_then(|value| value.as_object());
    assert!(
        cv.is_none_or(|table| !table.contains_key("bullet_indent_pt")),
        "horizontal knob changed by the delta: bullet_indent_pt"
    );
    let fill = |style: &serde_json::Value, pointer: &str| {
        style.pointer(pointer).and_then(serde_json::Value::as_f64)
    };
    assert!(
        fill(&compact, "/cv/entry_spacing_pt") < fill(&base, "/cv/entry_spacing_pt"),
        "compact must tighten vertical whitespace"
    );
    assert!(!delta.is_empty());
}

fn selection(substyle: &str) -> Selection {
    Selection {
        style: "harvard".to_owned(),
        substyle: substyle.to_owned(),
    }
}
