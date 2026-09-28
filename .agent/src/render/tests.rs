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
    let mut document =
        crate::content::read_record(&workspace, "cvl/cv/harvard/standard/en/ch/content.toml")
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
fn filename_tokens_are_explicit_sanitized_display_name_tokens() {
    for (name, expected) in [
        ("Sample Taylor", "Taylor"),
        ("Sample de Silva", "Silva"),
        ("\tSample  O'Neil_--  ", "O_Neil"),
        ("Sample /../Ng\\", "Ng"),
        ("Sample 王", "王"),
    ] {
        assert_eq!(applicant_filename_token(name).unwrap(), expected);
    }
    assert!(applicant_filename_token(" \t").is_err());
    assert!(applicant_filename_token("Sample _/../_").is_err());
    assert_eq!(filename_key("acme_labs"), "Acme_Labs");
    assert_eq!(filename_key("platform-lead"), "Platform_Lead");
}

#[test]
fn cleanup_removes_only_exact_generated_opportunity_outputs() {
    let directory = tempdir().unwrap();
    let pdfs = directory.path().join("pdfs");
    let typst = directory.path().join("typst");
    fs::create_dir_all(&pdfs).unwrap();
    fs::create_dir_all(&typst).unwrap();
    let stem = "Taylor_Acme_Lead";
    let legacy = [
        pdfs.join("cv.pdf"),
        pdfs.join("cl.pdf"),
        typst.join("cv.typ"),
        typst.join("cl.typ"),
        // Previous kind-last scheme, migrated on rebuild.
        pdfs.join(format!("{stem}_CV.pdf")),
        pdfs.join(format!("{stem}_CL.pdf")),
        typst.join(format!("{stem}_CV.typ")),
        typst.join(format!("{stem}_CL.typ")),
    ];
    // Previous interim scheme: kind-first with hyphens inside segments.
    let interim = [
        pdfs.join("CV_Taylor_Acme_Platform-Lead.pdf"),
        pdfs.join("CL_Taylor_Acme_Platform-Lead.pdf"),
        typst.join("CV_Taylor_Acme_Platform-Lead.typ"),
        typst.join("CL_Taylor_Acme_Platform-Lead.typ"),
    ];
    let interim_names = [
        "CV_Taylor_Acme_Platform-Lead".to_owned(),
        "CL_Taylor_Acme_Platform-Lead".to_owned(),
    ];
    let letter = [
        pdfs.join(format!("CL_{stem}.pdf")),
        typst.join(format!("CL_{stem}.typ")),
    ];
    let keep = [
        pdfs.join(format!("CV_{stem}.pdf")),
        typst.join(format!("CV_{stem}.typ")),
        pdfs.join("CL_Other_Acme_Lead.pdf"),
        typst.join("CL_notes.typ"),
    ];
    for path in legacy.iter().chain(&interim).chain(&letter).chain(&keep) {
        fs::write(path, b"preserved bytes").unwrap();
    }
    remove_stale_opportunity_outputs(&pdfs, &typst, stem, true, &interim_names).unwrap();
    assert!(legacy.iter().chain(&interim).all(|path| !path.exists()));
    for path in letter.iter().chain(&keep) {
        assert_eq!(fs::read(path).unwrap(), b"preserved bytes");
    }
    remove_stale_opportunity_outputs(&pdfs, &typst, stem, false, &interim_names).unwrap();
    assert!(letter.iter().all(|path| !path.exists()));
    for path in keep {
        assert_eq!(fs::read(path).unwrap(), b"preserved bytes");
    }
}

fn named_opportunity_workspace() -> (tempfile::TempDir, Workspace) {
    let directory = tempdir().unwrap();
    let root = directory.path();
    let write = |relative: &str, content: &str| {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    };
    write(
        "ccvl.json",
        &serde_json::json!({
            "format": "ccvl-workspace", "schema_version": 8,
            "documents": {
                "cv": {"root": "cvl/cv", "default_style": "plain"},
                "cover_letter": {"root": "cvl/cl", "default_style": "plain"}
            }
        })
        .to_string(),
    );
    write("cvl/profile.toml", "name = \"Sample Taylor\"\n");
    let original = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut record: toml::Value = toml::from_str(
        &fs::read_to_string(original.join(".agent/scaffolds/opportunity/application.toml"))
            .unwrap(),
    )
    .unwrap();
    crate::opportunity::strip_position_prose(&mut record);
    let options = record["options"].as_table_mut().unwrap();
    options.insert("language".into(), "en-us".into());
    options.insert("pages".into(), 1.into());
    options.insert("cl_pages".into(), 1.into());
    record["job"]["id"] = "fixture".into();
    for document in ["cv", "cl"] {
        record
            .as_table_mut()
            .unwrap()
            .insert(document.into(), toml::Value::Table(toml::map::Map::new()));
        let base = format!("cvl/{document}/plain");
        write(
            &format!("{base}/style.toml"),
            &format!(
                "id = \"plain\"\napi = 1\ndocuments = [{document:?}]\nsupports_locales = [\"en-us\"]\npages = [1]\ndefault_pages = 1\nsubstyles = [\"standard\"]\ndefault_substyle = \"standard\"\n"
            ),
        );
        write(&format!("{base}/standard/substyle.toml"), "");
        write(
            &format!("{base}/standard/en/us/strings.toml"),
            "locale = \"en-us\"\n",
        );
        write(
            &format!("{base}/standard/en/us/typst/{document}.typ"),
            "#set text(font: \"Archivo\")\nSynthetic naming fixture.\n",
        );
    }
    for document in ["cv", "cl"] {
        write(
            &format!("cvl/{document}/plain/standard/en/us/content.toml"),
            &toml::to_string(&record).unwrap(),
        );
    }
    write(
        "opportunities/acme/platform-lead/application.toml",
        &toml::to_string(&record).unwrap(),
    );
    write(
        "opportunities/acme/platform-lead/posting.md",
        "# Posting reference — acme/platform-lead\n",
    );
    let workspace = Workspace::at(root).unwrap();
    (directory, workspace)
}

#[test]
fn failed_opportunity_replacement_preserves_legacy_bytes_until_every_copy_succeeds() {
    let (_directory, workspace) = named_opportunity_workspace();
    let parent = workspace.path("opportunities/acme/platform-lead");
    let specs = opportunity_specs(&workspace, "acme", "platform-lead").unwrap();
    assert_eq!(
        specs[0].output,
        parent.join("pdfs/CV_Taylor_Acme_Platform_Lead.pdf")
    );
    assert_eq!(
        specs[1].output,
        parent.join("pdfs/CL_Taylor_Acme_Platform_Lead.pdf")
    );
    let legacy = ["pdfs/cv.pdf", "pdfs/cl.pdf", "typst/cv.typ", "typst/cl.typ"];
    for path in legacy {
        let target = parent.join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, path.as_bytes()).unwrap();
    }
    let unrelated = parent.join("typst/CL_Personal_Notes.typ");
    fs::write(&unrelated, "user-owned note").unwrap();
    let original = fs::read_to_string(&specs[1].source).unwrap();
    fs::write(&specs[1].source, "#panic(\"synthetic render failure\")").unwrap();
    assert!(render_opportunity(&workspace, "acme", "platform-lead").is_err());
    for path in legacy {
        assert_eq!(fs::read(parent.join(path)).unwrap(), path.as_bytes());
    }
    fs::write(&specs[1].source, original).unwrap();
    let last_copy = parent.join("typst/CL_Taylor_Acme_Platform_Lead.typ");
    fs::create_dir(&last_copy).unwrap();
    let error = render_opportunity(&workspace, "acme", "platform-lead").unwrap_err();
    assert!(error.to_string().contains("cannot write"), "{error:#}");
    assert!(
        error
            .to_string()
            .contains("CL_Taylor_Acme_Platform_Lead.typ")
    );
    assert!(specs.iter().all(|spec| spec.output.is_file()));
    assert!(
        parent
            .join("typst/CV_Taylor_Acme_Platform_Lead.typ")
            .is_file()
    );
    for path in legacy {
        assert_eq!(fs::read(parent.join(path)).unwrap(), path.as_bytes());
    }
    assert_eq!(fs::read_to_string(&unrelated).unwrap(), "user-owned note");
    fs::remove_dir(&last_copy).unwrap();
    let outputs = render_opportunity(&workspace, "acme", "platform-lead").unwrap();
    assert_eq!(outputs.len(), 4);
    assert!(outputs.iter().all(|path| path.is_file()));
    assert!(legacy.iter().all(|path| !parent.join(path).exists()));
    assert_eq!(fs::read_to_string(unrelated).unwrap(), "user-owned note");
}

#[cfg(unix)]
#[test]
fn opportunity_output_symlinks_cannot_redirect_writes_or_cleanup() {
    use std::os::unix::fs::symlink;
    for sibling in [false, true] {
        for directory_name in ["pdfs", "typst"] {
            let (_directory, workspace) = named_opportunity_workspace();
            let external = tempdir().unwrap();
            let destination = if sibling {
                let path = workspace.path("opportunities/other/role/outputs");
                fs::create_dir_all(&path).unwrap();
                path
            } else {
                external.path().to_path_buf()
            };
            let sentinel = destination.join(if directory_name == "pdfs" {
                "cl.pdf"
            } else {
                "cl.typ"
            });
            fs::write(&sentinel, b"outside bytes").unwrap();
            let parent = workspace.path("opportunities/acme/platform-lead");
            symlink(&destination, parent.join(directory_name)).unwrap();
            let error = render_opportunity(&workspace, "acme", "platform-lead").unwrap_err();
            assert!(
                error.to_string().contains("must not be a symlink"),
                "{error:#}"
            );
            assert_eq!(fs::read(&sentinel).unwrap(), b"outside bytes");
            assert_eq!(fs::read_dir(&destination).unwrap().count(), 1);
        }
    }
    for output in [
        "pdfs/CV_Taylor_Acme_Platform_Lead.pdf",
        "typst/CL_Taylor_Acme_Platform_Lead.typ",
        "pdfs/cl.pdf",
    ] {
        let (_directory, workspace) = named_opportunity_workspace();
        let external = tempdir().unwrap();
        let sentinel = external.path().join("original");
        fs::write(&sentinel, b"outside bytes").unwrap();
        let path = workspace
            .path("opportunities/acme/platform-lead")
            .join(output);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        symlink(&sentinel, &path).unwrap();
        let error = render_opportunity(&workspace, "acme", "platform-lead").unwrap_err();
        assert!(
            error.to_string().contains("must not be a symlink"),
            "{error:#}"
        );
        assert_eq!(fs::read(sentinel).unwrap(), b"outside bytes");
        assert!(fs::symlink_metadata(path).unwrap().file_type().is_symlink());
    }
    let (_directory, workspace) = named_opportunity_workspace();
    let parent = workspace.path("opportunities/acme/platform-lead");
    let sibling = workspace.path("opportunities/acme/other");
    fs::rename(&parent, &sibling).unwrap();
    fs::create_dir(sibling.join("pdfs")).unwrap();
    fs::write(sibling.join("pdfs/cl.pdf"), b"sibling bytes").unwrap();
    symlink(&sibling, &parent).unwrap();
    let error = render_opportunity(&workspace, "acme", "platform-lead").unwrap_err();
    assert!(
        error.to_string().contains("must not be a symlink"),
        "{error:#}"
    );
    assert_eq!(
        fs::read(sibling.join("pdfs/cl.pdf")).unwrap(),
        b"sibling bytes"
    );
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
    let mut resolved = crate::content::read_record(&workspace, leaf.content()).unwrap();
    resolved["options"]["cv_substyle"] = "compact".into();
    let text = toml::to_string(&resolved).unwrap();
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

#[test]
fn alternate_papers_have_real_dimensions_labels_and_standalone_parity() {
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let compiler = Compiler::new(&workspace).unwrap();
    let temporary = tempfile::tempdir().unwrap();
    for (document, style, substyle, locale, paper, label) in [
        (
            "cv",
            "test-style-1",
            "sidebar",
            "en-ch",
            "us-letter",
            "US-LETTER",
        ),
        ("cl", "test-style-1", "topbar", "en-us", "a4", "A4"),
        ("cv", "test-style-2", "cards", "en-us", "a4", "A4"),
        (
            "cl",
            "test-style-2",
            "timeline",
            "en-ch",
            "us-letter",
            "US-LETTER",
        ),
    ] {
        let selected =
            styles::selection(&workspace, document, Some(style), Some(substyle)).unwrap();
        let leaf = styles::leaf(&workspace, document, locale, &selected).unwrap();
        let mut spec =
            cvl_spec_with_paper(&workspace, &leaf, leaf.default_pages, Some(paper)).unwrap();
        let original = compiler.compile(&workspace, &spec).unwrap();
        spec.output = temporary.path().join(format!("{document}-{style}.pdf"));
        compiler.export(&spec, &original).unwrap();
        crate::pdf::verify(
            &spec.output,
            spec.expected_pages,
            &[],
            &spec.contract["pdf"],
        )
        .unwrap();
        let mut wrong = spec.contract["pdf"].clone();
        wrong["size_pt"] = serde_json::json!([100, 100]);
        assert!(crate::pdf::verify(&spec.output, spec.expected_pages, &[], &wrong).is_err());
        let pdf = lopdf::Document::load(&spec.output).unwrap();
        let text = pdf.extract_text(&[1]).unwrap();
        assert!(
            text.contains(label),
            "selected paper label is absent: {text}"
        );
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
        assert!(
            compiler.engine.pdf(&original, 0).unwrap()
                == compiler.engine.pdf(&standalone.document, 0).unwrap(),
            "alternate standalone copy differs: {}",
            spec.name
        );
    }
}
