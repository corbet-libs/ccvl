use std::path::PathBuf;

use serde_json::json;

use super::*;

/// Contracts and records come from the synthetic fixture's ledger styles.
fn workspace() -> Workspace {
    crate::test_support::fixture_view()
}

fn cover_letter_spec() -> DocumentSpec {
    DocumentSpec {
        name: "fixture".to_owned(),
        kind: DocumentKind::CoverLetter,
        source: PathBuf::from("fixture.typ"),
        output: PathBuf::from("fixture.pdf"),
        inputs: std::collections::BTreeMap::new(),
        expected_pages: 1,
        selection: crate::styles::Selection {
            style: "ledger".into(),
            substyle: "rule".into(),
        },
        fonts: Vec::new(),
        contract: crate::styles::contract(&workspace(), "cl", "ledger").unwrap(),
    }
}

fn cv_spec(application: &str) -> DocumentSpec {
    let mut inputs = std::collections::BTreeMap::new();
    inputs.insert("application".to_owned(), application.to_owned());
    DocumentSpec {
        name: "fixture".to_owned(),
        kind: DocumentKind::Cv,
        source: PathBuf::from("fixture.typ"),
        output: PathBuf::from("fixture.pdf"),
        inputs,
        expected_pages: 4,
        selection: crate::styles::Selection {
            style: "ledger".into(),
            substyle: "primary".into(),
        },
        fonts: Vec::new(),
        contract: crate::styles::contract(&workspace(), "cv", "ledger").unwrap(),
    }
}

fn summary_metric(actual: f64) -> Value {
    json!({
        "kind": "cv-summary",
        "id": "cv.summary.1",
        "text": "evidence",
        "actual_fill": actual,
        "min_fill": 60,
        "target_fill": 82,
        "max_fill": 100,
    })
}

fn repo_cv_spec() -> DocumentSpec {
    cv_spec("cvl/cv/ledger/primary/de/ch/content.toml")
}

fn metric(kind: &str, identifier: &str) -> Value {
    json!({"kind": kind, "id": identifier})
}

fn metric_set(paragraph_lengths: &[usize]) -> Vec<Value> {
    let mut metrics = paragraph_lengths
        .iter()
        .enumerate()
        .flat_map(|(paragraph, length)| {
            (1..=*length).map(move |line| {
                metric("cl-body", &format!("cl.paragraph.{}.{line}", paragraph + 1))
            })
        })
        .collect::<Vec<_>>();
    metrics.extend((1..=5).map(|index| metric("cl-highlight", &format!("cl.highlight.{index}"))));
    metrics.push(metric("cl-vertical-gap", "cl.vertical-gap"));
    metrics.push(metric("cl-highlight-center", "cl.highlight-center"));
    metrics
}

#[test]
fn cover_letter_templates_enforce_body_closing_and_highlight_bounds() {
    let (_fixture, workspace) = crate::test_support::fixture_workspace();
    let engine = ctypst::Engine::builder()
        .root(workspace.root())
        .fonts(ctypst::fonts::documents())
        .build()
        .unwrap();
    let cases: [(&str, f64, &[Option<&str>]); 8] = [
        ("cl-body", 85.0, &[Some("too short"), None]),
        ("cl-body", 85.0, &[None]),
        ("cl-body", 74.9, &[Some("too short")]),
        ("cl-body", 95.0, &[None, None]),
        ("cl-body", 100.0, &[None]),
        ("cl-body", 101.0, &[Some("too long")]),
        ("cl-highlight", 69.9, &[Some("too short")]),
        ("cl-highlight", 70.0, &[None]),
    ];
    for locale in ["de-ch", "en-ch"] {
        for (kind, fill, expected) in cases {
            // Use the fixture renderer's contract assignment and a measured
            // width, independent of any wording. The same
            // 85% line must fail before a break and pass as the paragraph
            // close; the CV's 102% grace must not leak here.
            // Fill helpers are independent of application inputs; importing
            // them must not read a leaf record or emit a document.
            let content = if kind == "cl-body" {
                format!(
                    "measured-paragraph(\"fixture\", \"cl-body\", \
                         with-body-fill(range({}).map(_ => evidence)))",
                    expected.len()
                )
            } else {
                "measured-line(\"fixture\", \"cl-highlight\", \
                     with-highlight-fill(evidence))"
                    .to_owned()
            };
            let source = format!(
                "#import \"/cvl/shared/ledger/render.typ\": with-body-fill, with-highlight-fill\n\
                     #import \"/.agent/typst/line-contract.typ\": measured-paragraph, measured-line\n\
                     #set page(width: 200pt, height: 100pt, margin: 10pt)\n\
                     #set text(font: \"Archivo\", size: 10pt, hyphenate: false)\n\
                     #let evidence = \"Verified engineering work\"\n\
                     #context {{\n\
                       let width = measure(text(evidence)).width * 100 / {fill}\n\
                       block(width: width)[#{content}]\n\
                     }}"
            );
            let mut report_inputs = std::collections::BTreeMap::new();
            report_inputs.insert("line-contracts".to_owned(), "report".to_owned());
            let report = engine
                .compile(
                    ctypst::CompileRequest::new("density.typ")
                        .source_file("density.typ", source.clone())
                        .inputs(report_inputs)
                        .pages(ctypst::PageConstraint::Exactly(1)),
                )
                .unwrap();
            let metrics = ctypst::query_json(&report.document, "ccvl-line").unwrap();
            assert_eq!(metrics.len(), expected.len());
            for (metric, expected_failure) in metrics.iter().zip(expected) {
                assert_eq!(
                    violation(metric).unwrap(),
                    *expected_failure,
                    "{locale} {kind} at {fill}%: {metric}"
                );
            }
            let enforced = engine.compile(
                ctypst::CompileRequest::new("density.typ")
                    .source_file("density.typ", source)
                    .pages(ctypst::PageConstraint::Exactly(1)),
            );
            let failure = enforced.err().map(|error| error.to_string());
            assert_eq!(
                failure.is_none(),
                expected.iter().all(Option::is_none),
                "{locale} {kind} at {fill}%: {failure:?}"
            );
        }
    }
}

#[test]
fn closing_line_spill_renders_without_wrapping() {
    // A closing line at 100.8% (inside the 102 maximum) must stay one
    // visual line: exact-width boxes spill into the margin instead of
    // re-wrapping, which previously added a sixth summary line and
    // overflowed the page despite green metrics. Five repeated spill
    // lines fit one 60mm page exactly; with auto-width boxes they wrap
    // to ten lines over two pages. The block width makes the spill
    // explicit, independent of any style's page geometry.
    let (_fixture, workspace) = crate::test_support::fixture_workspace();
    let engine = ctypst::Engine::builder()
        .root(workspace.root())
        .fonts(ctypst::fonts::documents())
        .build()
        .unwrap();
    let spill = "Synthetic closing line that spills slightly past its measured block.";
    let source = |exact: bool| {
        format!(
            "#import \"/.agent/typst/line-contract.typ\": measured-lines\n\
             {FIXTURE_SETTINGS}\
             #set page(height: 60mm)\n\
             #set text(hyphenate: false)\n\
             #let spill = \"{spill}\"\n\
             #context block(width: measure(text(spill)).width / 1.008)[#measured-lines(\"t\", \"x\", range(5).map(i => (text: spill, min_fill: 60, target_fill: 82, max_fill: 102)), exact-width: {exact})]"
        )
    };
    let compile = |exact: bool| {
        engine.compile(
            ctypst::CompileRequest::new("spill.typ")
                .source_file("spill.typ", source(exact))
                .pages(ctypst::PageConstraint::Exactly(1)),
        )
    };
    let output = compile(true).unwrap();
    let metrics = ctypst::query_json(&output.document, "ccvl-line").unwrap();
    assert_eq!(metrics.len(), 5);
    for metric in &metrics {
        let fill = metric["actual_fill"].as_f64().unwrap();
        assert!(
            fill > 100.0 && fill <= 102.0,
            "want a real spill, got {fill}"
        );
    }
    assert!(
        compile(false).is_err(),
        "auto-width boxes must re-wrap the spill onto a second page"
    );
}

/// The fixture family's document-v1 settings: the shared adapter baseline
/// (A4, 12/15mm margins, Archivo 10.5pt, no block spacing).
const FIXTURE_SETTINGS: &str = "#import \"/.agent/typst/document.typ\": apply-document-settings\n\
    #import \"/.agent/typst/paper.typ\": resolve-paper, paper-settings\n\
    #let preset = resolve-paper(toml(\"/cvl/cv/ledger/style.toml\"), \"de-ch\")\n\
    #show: apply-document-settings.with(paper-settings((toml(\"/cvl/shared/ledger/defaults.toml\"), toml(\"/cvl/cv/ledger/primary/substyle.toml\"), toml(\"/cvl/cv/ledger/primary/de/ch/layout.toml\")), preset))\n";

#[test]
fn paragraph_closing_spill_renders_without_wrapping() {
    // The generic paragraph helper supports callers that explicitly
    // allow 102%; cover-letter templates cap every line at 100%.
    // A permitted spill must still stay one visual line: three paragraphs
    // with a short line plus a 101% closing line fit one 60mm page; with
    // flowing text the closings re-wrap to nine lines over two pages.
    let (_fixture, workspace) = crate::test_support::fixture_workspace();
    let engine = ctypst::Engine::builder()
        .root(workspace.root())
        .fonts(ctypst::fonts::documents())
        .build()
        .unwrap();
    let short = "Short synthetic line.";
    let spill = "Synthetic paragraph closing line that may spill a little.";
    let source = format!(
        "#import \"/.agent/typst/line-contract.typ\": measured-paragraph\n\
             {FIXTURE_SETTINGS}\
             #set page(height: 60mm)\n\
             #set text(hyphenate: false)\n\
             #let short = \"{short}\"\n\
             #let spill = \"{spill}\"\n\
             #for p in range(3) {{\n\
             context block(breakable: false, width: measure(text(spill)).width / 1.01)[#measured-paragraph(\"t.\" + str(p), \"x\", ((text: short, min_fill: 1, target_fill: 82, max_fill: 100), (text: spill, min_fill: 60, target_fill: 82, max_fill: 102)), justify: true)]\n\
             }}"
    );
    let output = engine
        .compile(
            ctypst::CompileRequest::new("spill-cl.typ")
                .source_file("spill-cl.typ", source)
                .pages(ctypst::PageConstraint::Exactly(1)),
        )
        .unwrap();
    let metrics = ctypst::query_json(&output.document, "ccvl-line").unwrap();
    assert_eq!(metrics.len(), 6);
    for metric in metrics
        .iter()
        .filter(|metric| metric["max_fill"].as_f64() == Some(102.0))
    {
        let fill = metric["actual_fill"].as_f64().unwrap();
        assert!(
            fill > 100.0 && fill <= 102.0,
            "want a real spill, got {fill}"
        );
    }
}

#[test]
fn underfill_and_overflow_are_both_failures_for_any_unit() {
    let base = json!({"min_fill": 60, "target_fill": 80, "max_fill": 95});
    let mut metric = base.clone();
    metric["actual_fill"] = json!(59.9);
    assert_eq!(violation(&metric).unwrap(), Some("too short"));
    metric["actual_fill"] = json!(80.0);
    assert_eq!(violation(&metric).unwrap(), None);
    metric["actual_fill"] = json!(95.1);
    assert_eq!(violation(&metric).unwrap(), Some("too long"));

    metric["unit"] = json!("pt");
    metric["min_fill"] = json!(12);
    metric["target_fill"] = json!(20);
    metric["max_fill"] = json!(30);
    metric["actual_fill"] = json!(11.9);
    assert_eq!(violation(&metric).unwrap(), Some("too short"));
    metric["actual_fill"] = json!(24.1);
    assert_eq!(violation(&metric).unwrap(), None);
    metric["actual_fill"] = json!(30.1);
    assert_eq!(violation(&metric).unwrap(), Some("too long"));
}

#[test]
fn cover_letter_metric_set_requires_structure_and_layout_metrics() {
    let workspace = workspace();
    let spec = cover_letter_spec();
    let complete = metric_set(&[3, 5, 5, 5, 5, 3]);
    validate_metric_set(&workspace, &spec, &complete).unwrap();
    assert_eq!(
        preference_warnings(&workspace, &spec, &complete).unwrap(),
        Vec::<String>::new()
    );

    // The strict 3|5|5|5|5|3 framework has no dispreferred-but-valid
    // totals: any deviation from exactly 5 central lines or a 3-line
    // close fails validation instead of warning.
    for invalid in [
        metric_set(&[3, 6, 6, 5, 5, 3]),
        metric_set(&[3, 5, 6, 5, 5, 3]),
        metric_set(&[3, 5, 5, 5, 5, 2]),
        metric_set(&[3, 4, 6, 5, 5, 3]),
    ] {
        validate_metric_set(&workspace, &spec, &invalid).unwrap_err();
    }

    for missing_kind in ["cl-vertical-gap", "cl-highlight-center"] {
        let incomplete = complete
            .iter()
            .filter(|item| item["kind"].as_str() != Some(missing_kind))
            .cloned()
            .collect::<Vec<_>>();
        let error = validate_metric_set(&workspace, &spec, &incomplete)
            .unwrap_err()
            .to_string();
        assert!(error.contains(missing_kind), "{error}");
    }
}

#[test]
fn summary_lines_never_fail_line_failure() {
    let spec = repo_cv_spec();
    let thin = summary_metric(9.2);
    assert!(line_failure(&spec, 0, &thin).unwrap().is_none());
    let spill = summary_metric(100.8);
    assert!(line_failure(&spec, 0, &spill).unwrap().is_none());
}

#[test]
fn summary_counsel_fails_thin_and_intolerable_spill() {
    let workspace = workspace();
    let spec = repo_cv_spec();
    let failures = summary_failures(&workspace, &spec, &[summary_metric(9.2)]).unwrap();
    assert_eq!(failures.len(), 1);
    assert!(failures[0].contains("allow_thin"));
    assert_eq!(
        summary_failures(&workspace, &spec, &[summary_metric(100.8)]).unwrap(),
        Vec::<String>::new()
    );
    let failures = summary_failures(&workspace, &spec, &[summary_metric(102.1)]).unwrap();
    assert_eq!(failures.len(), 1);
    assert!(failures[0].contains("closing-line maximum"));
}

#[test]
fn summary_counsel_notes_allowed_thin_and_tolerated_spill() {
    let workspace = workspace();
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("ccvl.json"),
        "{\"documents\":{\"cv\":{\"root\":\"cvl/cv\"},\"cover_letter\":{\"root\":\"cvl/cl\"}}}",
    )
    .unwrap();
    std::fs::create_dir_all(directory.path().join("cvl/cv/ledger")).unwrap();
    std::fs::write(
        directory.path().join("cvl/cv/ledger/contract.toml"),
        "last_line_maximum = 102\n[summary_fill]\nminimum = 60\ntarget = 82\nmaximum = 100\n",
    )
    .unwrap();
    std::fs::write(
        directory.path().join("record.toml"),
        "[cv]\nsummary = \"Evidence.\"\nallow_thin = true\n",
    )
    .unwrap();
    let allowed = Workspace::at(directory.path()).unwrap();
    let spec = cv_spec("record.toml");
    let warnings = preference_warnings(&allowed, &spec, &[summary_metric(9.2)]).unwrap();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("explicitly wanted"));
    let warnings =
        preference_warnings(&workspace, &repo_cv_spec(), &[summary_metric(100.8)]).unwrap();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("past the block edge"));
    let warnings =
        preference_warnings(&workspace, &repo_cv_spec(), &[summary_metric(82.0)]).unwrap();
    assert_eq!(warnings, Vec::<String>::new());
}

fn cover_letter_spec_with_application(application: &str) -> DocumentSpec {
    let mut inputs = std::collections::BTreeMap::new();
    inputs.insert("application".to_owned(), application.to_owned());
    DocumentSpec {
        name: "fixture".to_owned(),
        kind: DocumentKind::CoverLetter,
        source: PathBuf::from("fixture.typ"),
        output: PathBuf::from("fixture.pdf"),
        inputs,
        expected_pages: 1,
        selection: crate::styles::Selection {
            style: "ledger".into(),
            substyle: "rule".into(),
        },
        fonts: Vec::new(),
        contract: crate::styles::contract(&workspace(), "cl", "ledger").unwrap(),
    }
}

#[test]
fn recipient_counsel_uses_correspondence_locale_rules_without_duplicate_warnings() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("ccvl.json"), "{}").unwrap();
    let workspace = Workspace::at(directory.path()).unwrap();
    let spec = cover_letter_spec_with_application("record.toml");
    for (locale, name, expected) in [
        ("DE-ch", "Alex Example", Some("no parsable honorific")),
        ("en-us", "Alex Example", Some("no parsable honorific")),
        // An unrelated three-letter language beginning with "de" is not German.
        ("den-ca", "Alex Example", None),
        ("de-ch", "Frau Dr. Müller", None),
        ("de-ch", "", Some("is empty")),
        ("en-us", "", Some("is empty")),
    ] {
        let record = json!({
            "options": {"language": locale},
            "job": {"cl_recipient": {"name": name}},
        });
        std::fs::write(
            directory.path().join("record.toml"),
            toml::to_string(&record).unwrap(),
        )
        .unwrap();
        let warnings = recipient_warnings(&workspace, &spec).unwrap();
        assert_eq!(
            warnings.len(),
            usize::from(expected.is_some()),
            "{locale} {name}"
        );
        if let Some(fragment) = expected {
            assert!(warnings[0].starts_with("fixture:"));
            assert!(warnings[0].contains(fragment), "{}", warnings[0]);
        }
    }
}

#[test]
fn empty_recipient_name_warns_but_stays_valid() {
    let workspace = workspace();
    // Target-neutral records leave the recipient empty: formal salutation
    // stays valid, but measurement must surface a visible advisory.
    for locale in [
        "cvl/cl/ledger/rule/de/ch/content.toml",
        "cvl/cl/ledger/rule/en/ch/content.toml",
    ] {
        let spec = cover_letter_spec_with_application(locale);
        let warnings = recipient_warnings(&workspace, &spec).unwrap();
        assert_eq!(warnings.len(), 1, "locale: {locale}");
        assert!(warnings[0].contains("job.cl_recipient.name is empty"));
        assert!(warnings[0].contains("formal salutation"));
        let metrics = metric_set(&[3, 5, 5, 5, 5, 3]);
        validate_metric_set(&workspace, &spec, &metrics).unwrap();
        let warnings = preference_warnings(&workspace, &spec, &metrics).unwrap();
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("formal salutation")),
            "locale: {locale}"
        );
    }
    // Fixtures without an application input stay silent so metric-set
    // tests keep asserting exact warning counts.
    assert_eq!(
        recipient_warnings(&workspace, &cover_letter_spec()).unwrap(),
        Vec::<String>::new()
    );
    // CV specs never carry the salutation counsel.
    assert_eq!(
        recipient_warnings(&workspace, &repo_cv_spec()).unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn typst_salutation_helper_keeps_only_the_last_token() {
    // Typst-level coverage for the shared helper: the same edge cases
    // as the Rust mirror must hold inside the renderer.
    let engine = ctypst::Engine::builder()
        .root(crate::test_support::repository())
        .fonts(ctypst::fonts::documents())
        .build()
        .unwrap();
    let source = "#import \"/.agent/typst/letter/letter.typ\": salutation-last-name, salutation, opening\n\
            #assert(salutation-last-name(\"Dr. Jane Doe\") == \"Doe\", message: \"title prefix\")\n\
            #assert(salutation-last-name(\"Ms Test Person\") == \"Person\", message: \"multi-token\")\n\
            #assert(salutation-last-name(\"Madonna\") == \"Madonna\", message: \"single token\")\n\
            #assert(salutation-last-name(\"Anne-Marie Müller-Schmidt\") == \"Müller-Schmidt\", message: \"hyphenated\")\n\
            #assert(salutation-last-name(\"  Jane   Doe  \") == \"Doe\", message: \"padded\")\n\
            #assert(salutation-last-name(\"\") == \"\", message: \"empty\")\n\
            #assert(salutation-last-name(\"   \") == \"\", message: \"whitespace\")\n\
            #assert(salutation(\"de-ch\", \"Frau Dr. Müller\") == \"Sehr geehrte Frau Dr. Müller\", message: \"ch titled\")\n\
            #assert(salutation(\"de-ch\", \"Herr Müller\") == \"Sehr geehrter Herr Müller\", message: \"ch plain\")\n\
            #assert(salutation(\"de-li\", \"Frau Müller\") == \"Sehr geehrte Frau Müller\", message: \"li no comma\")\n\
            #assert(salutation(\"de\", \"Frau Müller\") == \"Sehr geehrte Frau Müller,\", message: \"de comma\")\n\
            #assert(salutation(\"de-at\", \"Herr Müller\") == \"Sehr geehrter Herr Müller,\", message: \"at comma\")\n\
            #assert(salutation(\"de-ch\", \"Herr Prof. Dr. Müller\") == \"Sehr geehrter Herr Professor Müller\", message: \"professor wins\")\n\
            #assert(salutation(\"de-ch\", \"\") == \"Sehr geehrte Damen und Herren\", message: \"ch generic\")\n\
            #assert(salutation(\"de\", \"\") == \"Sehr geehrte Damen und Herren,\", message: \"de generic\")\n\
            #assert(salutation(\"de-ch\", \"Jane Doe\") == \"Sehr geehrte Damen und Herren\", message: \"no honorific\")\n\
            #assert(salutation(\"de-ch\", \"Hr. Müller\") == \"Sehr geehrte Damen und Herren\", message: \"abbreviation rejected\")\n\
            #assert(salutation(\"de-ch\", \"Frau Anna Schmidt Meyer\") == \"Sehr geehrte Frau Meyer\", message: \"spaced double surname shortens\")\n\
            #assert(opening(\"de-ch\", override: \"Sehr geehrte Frau Schmidt Meyer\") == \"Sehr geehrte Frau Schmidt Meyer\", message: \"explicit override wins verbatim\")\n\
            #assert(opening(\"de-ch\", override: \"\") == \"\", message: \"empty override stays empty\")\n\
            #assert(opening(\"de-ch\", name: \"Frau Müller\") == \"Sehr geehrte Frau Müller\", message: \"opening falls back to name\")\n\
            #assert(opening(\"de-ch\") == \"Sehr geehrte Damen und Herren\", message: \"opening falls back to formal\")\n\
            #assert(salutation(\"fr\", \"Madame Dupont\") == \"Madame Dupont,\", message: \"french\")\n\
            #assert(salutation(\"it\", \"Sig. Rossi\") == \"Gentile Sig. Rossi,\", message: \"italian\")\n\
            #assert(salutation(\"rm\", \"signur Schmid\") == \"Stimà signur Schmid,\", message: \"romansh\")\n\
            #assert(salutation(\"en\", \"Ms Smith\") == \"Dear Ms Smith,\", message: \"english\")\n\
            Hello";
    engine
        .compile(
            ctypst::CompileRequest::new("salutation.typ")
                .source_file("salutation.typ", source)
                .pages(ctypst::PageConstraint::Exactly(1)),
        )
        .unwrap();
}

#[test]
fn summary_counsel_uses_shared_wording_and_leaf_exceptions_like_typst() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let leaf = "cvl/cv/ledger/primary/en/ch/content.toml";
    let shared = "cvl/cv/ledger/content/en/ch/wording.toml";
    for relative in [leaf, shared, ".agent/typst/application.typ"] {
        std::fs::create_dir_all(root.join(relative).parent().unwrap()).unwrap();
    }
    std::fs::write(
        root.join("ccvl.json"),
        json!({"documents": {
            "cv": {"root": "cvl/cv"}, "cover_letter": {"root": "cvl/cl"}
        }})
        .to_string(),
    )
    .unwrap();
    std::fs::write(root.join("cvl/cv/ledger/style.toml"),
        "id = \"ledger\"\napi = 1\ndocuments = [\"cv\"]\nsupports_locales = [\"en-ch\"]\npages = [4]\ndefault_pages = 4\nsubstyles = [\"primary\"]\n"
    ).unwrap();
    std::fs::copy(
        crate::test_support::repository().join(".agent/typst/application.typ"),
        root.join(".agent/typst/application.typ"),
    )
    .unwrap();
    let workspace = Workspace::at(root).unwrap();
    let spec = cv_spec(leaf);
    let source = format!(
        "#import \"/.agent/typst/application.typ\": load-application\n#set text(font: \"Archivo\")\n#metadata(load-application(\"/{leaf}\").cv.allow_thin) <thin-policy>\nPolicy fixture.\n"
    );
    for (shared_allowance, leaf_override, expected) in [
        (true, None, true),
        (true, Some(false), false),
        (false, Some(true), true),
        (false, None, false),
    ] {
        std::fs::write(
            root.join(shared),
            format!("[cv]\nsummary = \"Evidence.\"\nallow_thin = {shared_allowance}\n"),
        )
        .unwrap();
        let exception = leaf_override
            .map(|value| format!("[cv]\nallow_thin = {value}\n"))
            .unwrap_or_default();
        std::fs::write(
            root.join(leaf),
            format!("[wording]\nsource = \"../../../content/en/ch/wording.toml\"\n{exception}"),
        )
        .unwrap();
        let failures = summary_failures(&workspace, &spec, &[summary_metric(9.2)]).unwrap();
        assert_eq!(failures.is_empty(), expected);
        let warnings = preference_warnings(&workspace, &spec, &[summary_metric(9.2)]).unwrap();
        assert_eq!(
            warnings
                .iter()
                .any(|warning| warning.contains("explicitly wanted")),
            expected
        );
        let engine = ctypst::Engine::builder()
            .root(root)
            .fonts(ctypst::fonts::documents())
            .build()
            .unwrap();
        let output = engine
            .compile(
                ctypst::CompileRequest::new("fixture.typ")
                    .source_file("fixture.typ", source.clone()),
            )
            .unwrap();
        assert_eq!(
            ctypst::query_json(&output.document, "thin-policy").unwrap(),
            vec![json!(expected)]
        );
    }
}
