use super::*;
use crate::{application, measure, pdf, render};
use serde_json::json;

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// This workspace deliberately contains no Harvard files or shared renderer.
fn independent_workspace() -> (tempfile::TempDir, Workspace) {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    let original = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let manifest = json!({
        "format": "ccvl-workspace", "schema_version": 8,
        "documents": {
            "cv": {"root": "cvl/cv", "default_style": "orbit"},
            "cover_letter": {"root": "cvl/cl", "default_style": "postcard"}
        }
    });
    write(root, "ccvl.json", &format!("{manifest}\n"));
    write(
        root,
        "cvl/profile.toml",
        &fs::read_to_string(original.path("cvl/profile.toml")).unwrap(),
    );
    let mut record: toml::Value = toml::from_str(
        &fs::read_to_string(original.path("cvl/cv/harvard/standard/en/ch/content.toml")).unwrap(),
    )
    .unwrap();
    record["options"]
        .as_table_mut()
        .unwrap()
        .insert("language".into(), "en-us".into());
    record["options"]
        .as_table_mut()
        .unwrap()
        .insert("pages".into(), 1.into());
    record["options"]
        .as_table_mut()
        .unwrap()
        .insert("cl_pages".into(), 2.into());
    record["options"]
        .as_table_mut()
        .unwrap()
        .insert("cv_style".into(), "orbit".into());
    record["options"]
        .as_table_mut()
        .unwrap()
        .insert("cv_substyle".into(), "standard".into());
    record["options"]
        .as_table_mut()
        .unwrap()
        .insert("cl_style".into(), "postcard".into());
    record["options"]
        .as_table_mut()
        .unwrap()
        .insert("cl_substyle".into(), "standard".into());
    record["cv"] = toml::toml! {
        heading = "Independent layout fixture"
        body = "This fixture exercises a landscape document with columns and a serif face. It contains no career claims."
    }.into();
    record["cl"] = toml::toml! {
        message = "Independent letter fixture"
        note = "This letter exercises two custom-sized pages without Harvard paragraph counts, highlights, or a signature."
    }.into();
    write(
        root,
        ".agent/scaffolds/opportunity/application.toml",
        &fs::read_to_string(original.path(".agent/scaffolds/opportunity/application.toml"))
            .unwrap(),
    );
    let record = toml::to_string(&record).unwrap();
    for (document, style, pages) in [("cv", "orbit", 1), ("cl", "postcard", 2)] {
        let base = format!("cvl/{document}/{style}");
        write(
            root,
            &format!("{base}/style.toml"),
            &format!(
                "id = {style:?}\napi = 1\ndocuments = [{document:?}]\nsupports_locales = [\"en-us\"]\npages = [{pages}]\ndefault_pages = {pages}\nsubstyles = [\"standard\"]\ndefault_substyle = \"standard\"\n"
            ),
        );
        write(root, &format!("{base}/standard/substyle.toml"), "");
        write(
            root,
            &format!("{base}/standard/en/us/content.toml"),
            &record,
        );
        write(
            root,
            &format!("{base}/standard/en/us/strings.toml"),
            "locale = \"en-us\"\n",
        );
    }
    write(
        root,
        "cvl/cv/orbit/contract.toml",
        r#"
[pdf]
size_pt = [680.315, 368.504]
[[metric_rules]]
kind = "orbit-title"
minimum = 1
maximum = 1
"#,
    );
    write(
        root,
        "cvl/cv/orbit/standard/en/us/typst/cv.typ",
        r##"
#set page(width: 240mm, height: 130mm, margin: 12mm)
#set text(font: "EB Garamond", size: 14pt)
#let record = toml(sys.inputs.at("application", default: "/cvl/cv/orbit/standard/en/us/content.toml"))
#grid(columns: (1fr, 2fr), gutter: 12mm,
  rect(fill: rgb("#153347"), inset: 12pt, text(fill: white, record.cv.heading)),
  [#record.cv.body],
)
#metadata((kind: "orbit-title", id: "orbit.title", actual_fill: 80, min_fill: 70, target_fill: 80, max_fill: 100, text: record.cv.heading)) <ccvl-line>
"##,
    );
    write(
        root,
        "cvl/cl/postcard/standard/en/us/typst/cl.typ",
        r##"
#set page(width: 120mm, height: 180mm, margin: 12mm)
#set text(font: "IBM Plex Serif", size: 15pt)
#let record = toml(sys.inputs.at("application", default: "/cvl/cl/postcard/standard/en/us/content.toml"))
#for index in range(int(sys.inputs.at("pages", default: "2"))) {
  if index > 0 { pagebreak() }
  rect(fill: rgb("#e7eff6"), inset: 12pt, radius: 8pt)[#record.cl.message]
  parbreak()
  record.cl.note
}
"##,
    );
    let workspace = Workspace::at(root).unwrap();
    (temporary, workspace)
}

#[test]
fn independent_styles_render_without_harvard_geometry_content_or_sources() {
    let (_temporary, workspace) = independent_workspace();
    assert!(!workspace.path("cvl/cv/harvard").exists());
    assert!(!workspace.path("cvl/shared").exists());
    assert!(!workspace.path("interview/stations.toml").exists());
    let specs = render::cvl_specs(&workspace).unwrap();
    assert_eq!(specs.len(), 2);
    assert_eq!(specs[0].expected_pages, 1);
    assert_eq!(specs[1].expected_pages, 2);
    assert!(!specs[0].inputs.contains_key("shared-defaults"));
    let compiler = render::Compiler::new(&workspace).unwrap();
    for spec in specs {
        let document = compiler.compile(&workspace, &spec).unwrap();
        let metrics = measure::document_metrics(&workspace, &spec, &document).unwrap();
        assert!(
            measure::summary_failures(&workspace, &spec, &metrics)
                .unwrap()
                .is_empty()
        );
        assert!(
            measure::preference_warnings(&workspace, &spec, &metrics)
                .unwrap()
                .is_empty()
        );
        let output = compiler.export(&spec, &document).unwrap();
        pdf::verify(
            &output,
            spec.expected_pages,
            &[],
            spec.contract.get("pdf").unwrap_or(&json!({})),
        )
        .unwrap();
        let pdf = lopdf::Document::load(output).unwrap();
        for id in pdf.get_pages().values() {
            for font in pdf.get_page_fonts(*id).unwrap().values() {
                let name = font.get(b"BaseFont").unwrap().as_name().unwrap();
                assert!(!String::from_utf8_lossy(name).contains("Archivo"));
            }
        }
    }
    let entries = render::list_documents(&workspace).unwrap();
    assert_eq!(entries[0]["style"], "orbit");
    assert_eq!(entries[1]["style"], "postcard");
}

#[test]
fn selections_are_scoped_by_style_and_reject_missing_or_escaping_names() {
    let (_temporary, workspace) = independent_workspace();
    let selected = selection(&workspace, "cv", None, None).unwrap();
    assert_eq!(
        selected,
        Selection {
            style: "orbit".into(),
            substyle: "standard".into()
        }
    );
    assert!(selection(&workspace, "cv", Some("harvard"), Some("standard")).is_err());
    assert!(selection(&workspace, "cv", Some("orbit"), Some("frame")).is_err());
    assert!(selection(&workspace, "cv", Some("../cl/postcard"), None).is_err());
    assert!(
        definition(&workspace, "cv", "orbit")
            .unwrap()
            .pages
            .contains(&1)
    );
    let leaf = leaf(&workspace, "cv", "en-us", &selected).unwrap();
    let record = workspace
        .read_toml_value(workspace.relative(&leaf.content()).unwrap())
        .unwrap();
    application::validate_record(&workspace, &record, "fixture", true).unwrap();
    let mut invalid = record;
    invalid["options"]["cv_style"] = 123.into();
    assert!(record_selection(&workspace, "cv", &invalid, "fixture").is_err());
}

#[test]
fn new_opportunities_take_page_and_content_defaults_from_the_selected_styles() {
    let (_temporary, workspace) = independent_workspace();
    write(
        workspace.root(),
        "cvl/cv/orbit/scaffold.toml",
        "heading = \"\"\nbody = \"\"\n",
    );
    write(
        workspace.root(),
        "cvl/cl/postcard/scaffold.toml",
        "message = \"\"\nnote = \"\"\n",
    );
    let record = crate::opportunity::blank_record(&workspace).unwrap();
    assert_eq!(record["options"]["pages"].as_integer(), Some(1));
    assert_eq!(record["options"]["cl_pages"].as_integer(), Some(2));
    assert_eq!(record["options"]["cv_style"].as_str(), Some("orbit"));
    assert!(record["cv"].get("summary").is_none());
    assert!(record["cl"].get("paragraphs").is_none());
    application::validate_record(
        &workspace,
        &serde_json::to_value(record).unwrap(),
        "fixture",
        false,
    )
    .unwrap();
}

#[test]
fn full_workspace_check_accepts_independent_styles() {
    let (_temporary, workspace) = independent_workspace();
    let original = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let mut manifest = original.read_json("ccvl.json").unwrap();
    manifest["documents"] = workspace.read_json("ccvl.json").unwrap()["documents"].clone();
    write(workspace.root(), "ccvl.json", &format!("{manifest}\n"));
    for relative in [
        ".agent/scaffolds/interview/profile.toml",
        ".agent/scaffolds/interview/stations.toml",
        ".agent/tests/skill-cases.json",
    ] {
        write(
            workspace.root(),
            relative,
            &fs::read_to_string(original.path(relative)).unwrap(),
        );
    }
    write(
        workspace.root(),
        "interview/stations.toml",
        "schema_version = 1\nstations = []\n",
    );
    for relative in [
        "cvl/README.md",
        "interview/README.md",
        "opportunities/README.md",
    ] {
        write(workspace.root(), relative, "# Independent fixture\n");
    }
    for directory in [".agent/skills", ".agent/typst/fonts"] {
        for entry in walkdir::WalkDir::new(original.path(directory)) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                let relative = original.relative(entry.path()).unwrap();
                let target = workspace.path(relative);
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    // The profile is locale data, not a built-in de-ch/en-ch list.
    let mut profile: toml::Value =
        toml::from_str(&fs::read_to_string(workspace.path("cvl/profile.toml")).unwrap()).unwrap();
    let fields = profile["localized"]["en-ch"].clone();
    let mut locales = toml::Table::new();
    locales.insert("en-us".into(), fields);
    profile["localized"] = locales.into();
    write(
        workspace.root(),
        "cvl/profile.toml",
        &toml::to_string(&profile).unwrap(),
    );
    crate::format::format_typst(&workspace, false).unwrap();
    render::render_cvl(&workspace).unwrap();
    crate::check::run(&workspace).unwrap();
    assert!(!workspace.path("cvl/cv/harvard").exists());
    assert!(!workspace.path("cvl/cl/harvard").exists());
}

#[test]
fn identical_substyle_names_in_two_cv_styles_have_distinct_paths() {
    let (_temporary, workspace) = independent_workspace();
    let from = workspace.path("cvl/cv/orbit");
    for entry in walkdir::WalkDir::new(&from) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(&from).unwrap();
            let target = workspace.path("cvl/cv/second").join(relative);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(
                target,
                fs::read_to_string(entry.path())
                    .unwrap()
                    .replace("orbit", "second"),
            )
            .unwrap();
        }
    }
    let first = leaf(
        &workspace,
        "cv",
        "en-us",
        &selection(&workspace, "cv", Some("orbit"), Some("standard")).unwrap(),
    )
    .unwrap();
    let second = leaf(
        &workspace,
        "cv",
        "en-us",
        &selection(&workspace, "cv", Some("second"), Some("standard")).unwrap(),
    )
    .unwrap();
    assert_ne!(first.adapter(), second.adapter());
    assert_ne!(first.output(1), second.output(1));
    assert_eq!(leaves(&workspace, "cv").unwrap().len(), 2);
}

#[test]
fn malformed_style_contracts_fail_instead_of_disabling_checks() {
    let (_temporary, workspace) = independent_workspace();
    for invalid in [
        "metric_rules = false",
        "shared_pages = [0]",
        "[pdf]\nsize_pt = \"A4\"",
        "[pdf]\nrequire_image = 1",
        "[pdf]\nversion = 17",
        "[pdf]\ntagged = \"yes\"",
        "[[metric_rules]]\nkind = \"title\"\nminimum = -1",
    ] {
        write(workspace.root(), "cvl/cv/orbit/contract.toml", invalid);
        assert!(contract(&workspace, "cv", "orbit").is_err(), "{invalid}");
    }
}

#[test]
fn pdf_geometry_is_selected_per_locale_and_missing_locale_fails() {
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let selected = selection(&workspace, "cv", Some("test-style-1"), Some("sidebar")).unwrap();
    let swiss = leaf(&workspace, "cv", "en-ch", &selected).unwrap();
    let american = leaf(&workspace, "cv", "en-us", &selected).unwrap();
    assert_eq!(
        swiss.contract.pointer("/pdf/size_pt"),
        Some(&json!([595.2756, 841.8898]))
    );
    assert_eq!(
        american.contract.pointer("/pdf/size_pt"),
        Some(&json!([612, 792]))
    );
    assert!(
        render::cvl_spec(&workspace, &american, 1).unwrap().inputs["layout"]
            .ends_with("/en/us/layout.toml")
    );
    let (_temporary, isolated) = independent_workspace();
    write(
        isolated.root(),
        "cvl/cv/orbit/contract.toml",
        "[pdf.by_locale.en-ch]\nsize_pt = [612, 792]\n",
    );
    assert!(
        leaves(&isolated, "cv")
            .unwrap_err()
            .to_string()
            .contains("missing PDF policy for en-us")
    );
}

#[test]
fn layout_input_changes_exported_paper_and_font_and_pdf_policy_is_enforced() {
    let (_temporary, workspace) = independent_workspace();
    let original = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    for (source, destination) in [
        (".agent/typst/document.typ", ".agent/typst/document.typ"),
        ("cvl/shared/test-style-1/defaults.toml", "defaults.toml"),
    ] {
        write(
            workspace.root(),
            destination,
            &fs::read_to_string(original.path(source)).unwrap(),
        );
    }
    write(
        workspace.root(),
        "cvl/cv/orbit/standard/en/us/typst/cv.typ",
        r#"
#import "/.agent/typst/document.typ": apply-document-settings, merge-settings
#let config = merge-settings(toml("/defaults.toml"), toml(sys.inputs.at("layout")))
#show: apply-document-settings.with(config)
Example#linebreak()Applicant

Explicit settings must change the actual paper and embedded typeface.
"#,
    );
    let selected = selection(&workspace, "cv", Some("orbit"), None).unwrap();
    for (paper, font, size, pattern) in [
        (
            "a4",
            "IBM Plex Serif",
            json!([595.2756, 841.8898]),
            "IBMPlexSerif",
        ),
        ("us-letter", "EB Garamond", json!([612, 792]), "EBGaramond"),
    ] {
        write(
            workspace.root(),
            "cvl/cv/orbit/standard/en/us/layout.toml",
            &format!(
                "[page]\npaper = {paper:?}\n[text]\nfont = {font:?}\nlang = \"en\"\nregion = \"US\"\n"
            ),
        );
        let spec = render::cvl_cv_spec(&workspace, "en-us", 1, Some(&selected)).unwrap();
        let compiler = render::Compiler::new(&workspace).unwrap();
        let output = compiler.render(&workspace, &spec).unwrap();
        let policy =
            json!({"size_pt": size, "font_pattern": pattern, "version": "1.7", "tagged": true});
        pdf::verify(&output, 1, &["Example Applicant".into()], &policy).unwrap();
        assert!(pdf::verify(&output, 1, &["Different Applicant".into()], &policy).is_err());
        assert!(pdf::verify(&output, 1, &[], &json!({"version": "2.0"})).is_err());
        assert!(pdf::verify(&output, 1, &[], &json!({"tagged": false})).is_err());
    }
}
