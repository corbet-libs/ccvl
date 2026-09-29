use super::*;
use crate::{Workspace, content, render, settings, styles};
use std::{fs, path::Path};

fn repository() -> (tempfile::TempDir, Workspace) {
    crate::test_support::paper_workspace()
}
fn leaf(workspace: &Workspace, document: &'static str, locale: &str) -> styles::StyleLeaf {
    let selected = styles::selection(workspace, document, Some("probe"), Some("plain")).unwrap();
    styles::leaf(workspace, document, locale, &selected).unwrap()
}

#[test]
fn registry_requires_explicit_complete_defaults_and_safe_preset_names() {
    let valid = json!({"defaults": {"en-us": "card"}, "sizes": {"card": {"size_pt": [240, 160], "label": "My card", "settings": {"canvas": {"width": 240}}}}});
    let registry: Registry = serde_json::from_value(valid.clone()).unwrap();
    registry.validate(&["en-us".into()]).unwrap();
    assert!(registry.validate(&["en-ch".into()]).is_err());
    assert!(select(Some(&registry), "en-us", Some("../card")).is_err());
    assert!(select(None, "en-us", Some("card")).is_err());
    assert!(select(None, "en-us", None).unwrap().is_none());
    for malformed in [
        json!({"defaults": {"en-us": "absent"}, "sizes": valid["sizes"]}),
        json!({"defaults": {"en-us": "../card"}, "sizes": {"../card": valid["sizes"]["card"]}}),
        json!({"defaults": {"en-us": "card"}, "sizes": {"card": {"size_pt": [0, 160], "label": "Card", "settings": {}}}}),
    ] {
        assert!(
            serde_json::from_value::<Registry>(malformed)
                .unwrap()
                .validate(&["en-us".into()])
                .is_err()
        );
    }
    let mut unknown = valid;
    unknown["sizes"]["card"]["typo"] = true.into();
    assert!(serde_json::from_value::<Registry>(unknown).is_err());
}

#[test]
fn paper_precedence_names_and_selected_document_scope_are_explicit() {
    let (_fixtures, workspace) = repository();
    let leaf = leaf(&workspace, "cv", "en-ch");
    assert_eq!(leaf.locale, "en-ch");
    let original = render::cvl_spec(&workspace, &leaf, 1).unwrap();
    assert_eq!(original.inputs["paper"], "a4");
    assert_eq!(original.output, leaf.output(1));
    let explicit = render::cvl_spec_with_paper(&workspace, &leaf, 1, Some("a4")).unwrap();
    assert_eq!(explicit.output, original.output);
    let alternate = render::cvl_spec_with_paper(&workspace, &leaf, 1, Some("us-letter")).unwrap();
    assert!(alternate.output.ends_with("pdf/cv-1-us-letter.pdf"));
    assert_eq!(alternate.contract["pdf"]["size_pt"], json!([612.0, 792.0]));
    assert!(render::cvl_spec_with_paper(&workspace, &leaf, 1, Some("../a4")).is_err());
    let temporary = tempfile::tempdir_in(workspace.root()).unwrap();
    let application = temporary.path().join("application.toml");
    let profile = workspace.path("cvl/profile.toml");
    let output = temporary.path().join("my-exact-name.pdf");
    let mut record = content::read_record(&workspace, leaf.content()).unwrap();
    record["options"]["cv_paper"] = "us-letter".into();
    // Malformed opposite-document metadata is irrelevant to a selected CV.
    record["options"]["cl_paper"] = json!({"invalid": true});
    fs::write(&application, toml::to_string(&record).unwrap()).unwrap();
    let selected =
        render::document_spec(&workspace, &leaf, 1, &application, &profile, None, None).unwrap();
    assert_eq!(selected.inputs["paper"], "us-letter");
    assert_eq!(selected.output, alternate.output);
    let override_spec = render::document_spec(
        &workspace,
        &leaf,
        1,
        &application,
        &profile,
        Some(&output),
        Some("a4"),
    )
    .unwrap();
    assert_eq!(override_spec.inputs["paper"], "a4");
    assert_eq!(override_spec.output, output);
    record["options"]["cv_paper"] = "unsupported".into();
    fs::write(&application, toml::to_string(&record).unwrap()).unwrap();
    assert!(
        render::document_spec(&workspace, &leaf, 1, &application, &profile, None, None).is_err()
    );
    assert!(
        render::document_spec(
            &workspace,
            &leaf,
            1,
            &application,
            &profile,
            None,
            Some("a4")
        )
        .is_ok()
    );
    record["options"]["cv_paper"] = 17.into();
    fs::write(&application, toml::to_string(&record).unwrap()).unwrap();
    assert!(
        render::document_spec(
            &workspace,
            &leaf,
            1,
            &application,
            &profile,
            None,
            Some("a4")
        )
        .is_err()
    );
    let selected = styles::selection(&workspace, "cl", Some("probe"), Some("plain")).unwrap();
    let american = styles::leaf(&workspace, "cl", "en-us", &selected).unwrap();
    assert!(
        render::cvl_spec_with_paper(&workspace, &american, 1, Some("a4"))
            .unwrap()
            .output
            .ends_with("pdf/cl-a4.pdf")
    );
    assert_eq!(render::cvl_specs(&workspace).unwrap().len(), 4);
}

#[test]
fn explanation_reports_selected_preset_and_validates_before_merging() {
    let (_fixtures, workspace) = repository();
    let mut leaf = leaf(&workspace, "cv", "en-ch");
    let explained = settings::resolve_with_paper(&workspace, &leaf, Some("us-letter")).unwrap();
    assert_eq!(explained["paper"]["id"], "us-letter");
    assert_eq!(explained["settings"]["page"]["paper"], "us-letter");
    assert_eq!(
        explained["origins"]["/page/paper"],
        "cvl/cv/probe/style.toml#paper.sizes.us-letter.settings"
    );
    leaf.paper
        .as_mut()
        .unwrap()
        .sizes
        .get_mut("us-letter")
        .unwrap()
        .settings["paragraph"] = json!({"alignment": "typo"});
    assert!(settings::resolve_with_paper(&workspace, &leaf, Some("us-letter")).is_err());
    // A malformed unselected preset does not change selected renderer inputs.
    settings::resolve_with_paper(&workspace, &leaf, Some("a4")).unwrap();
}

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn independent_renderer_owns_custom_paper_geometry_and_settings() {
    let (_fixtures, original) = repository();
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    write(
        root,
        "ccvl.json",
        r#"{"format":"ccvl-workspace","schema_version":8,"documents":{"cv":{"root":"cvl/cv","default_style":"orbit"}}}"#,
    );
    for source in [
        ".agent/typst/paper.typ",
        ".agent/typst/document.typ",
        ".agent/typst/document-settings.typ",
        ".agent/typst/document-settings.json",
        "cvl/profile.toml",
    ] {
        write(
            root,
            source,
            &fs::read_to_string(original.path(source)).unwrap(),
        );
    }
    write(
        root,
        "cvl/cv/orbit/style.toml",
        r#"
id = "orbit"
api = 1
documents = ["cv"]
supports_locales = ["en-us"]
pages = [1]
default_pages = 1
substyles = ["plain"]
default_substyle = "plain"
[paper.defaults]
en-us = "card"
[paper.sizes.card]
size_pt = [240, 160]
label = "Card"
[paper.sizes.card.settings.canvas]
width = 240
height = 160
[paper.sizes.banner]
size_pt = [320, 140]
label = "Banner"
[paper.sizes.banner.settings.canvas]
width = 320
height = 140
"#,
    );
    let mut record =
        content::read_record(&original, "cvl/cv/probe/plain/en/us/content.toml").unwrap();
    record["options"]["cv_style"] = "orbit".into();
    record["options"]["cv_substyle"] = "plain".into();
    record["options"]["generate_cl"] = false.into();
    record.as_object_mut().unwrap().remove("cl");
    record["cv"] = json!({"message": "Independent custom paper"});
    write(
        root,
        "cvl/cv/orbit/plain/en/us/content.toml",
        &toml::to_string(&record).unwrap(),
    );
    write(root, "cvl/cv/orbit/plain/substyle.toml", "");
    write(root, "cvl/cv/orbit/plain/en/us/strings.toml", "");
    write(
        root,
        "cvl/cv/orbit/plain/en/us/typst/cv.typ",
        r#"
#import "/.agent/typst/paper.typ": resolve-paper
#let record = toml(sys.inputs.at("application", default: "/cvl/cv/orbit/plain/en/us/content.toml"))
#let preset = resolve-paper(toml("/cvl/cv/orbit/style.toml"), "en-us", requested: sys.inputs.at("paper", default: ""), recorded: record.options.at("cv_paper", default: none))
#set page(width: preset.settings.canvas.width * 1pt, height: preset.settings.canvas.height * 1pt, margin: 10pt)
#set text(font: "IBM Plex Serif", size: 10pt)
#record.cv.message
"#,
    );
    let workspace = Workspace::at(root).unwrap();
    let selected = styles::selection(&workspace, "cv", Some("orbit"), Some("plain")).unwrap();
    let leaf = styles::leaf(&workspace, "cv", "en-us", &selected).unwrap();
    assert!(leaf.settings_adapter.is_none());
    let compiler = render::Compiler::new(&workspace).unwrap();
    for choice in ["card", "banner"] {
        let spec = render::cvl_spec_with_paper(&workspace, &leaf, 1, Some(choice)).unwrap();
        let output = compiler.render(&workspace, &spec).unwrap();
        crate::pdf::verify(&output, 1, &[], &spec.contract["pdf"]).unwrap();
    }
}

#[test]
fn showcase_record_selection_matches_explanation_listing_and_build() {
    let (_fixtures, workspace) = repository();
    let leaf = leaf(&workspace, "cv", "en-ch");
    let mut record: toml::Value =
        toml::from_str(&fs::read_to_string(leaf.content()).unwrap()).unwrap();
    record["options"]
        .as_table_mut()
        .unwrap()
        .insert("cv_paper".into(), "us-letter".into());
    fs::write(leaf.content(), toml::to_string(&record).unwrap()).unwrap();
    let spec = render::cvl_spec(&workspace, &leaf, 1).unwrap();
    assert!(spec.output.ends_with("pdf/cv-1-us-letter.pdf"));
    let explained = settings::explain(&workspace, &leaf, None).unwrap();
    assert_eq!(explained["paper"]["id"], spec.inputs["paper"]);
    assert_eq!(
        explained["paper"]["selection_source"],
        "cvl/cv/probe/plain/en/ch/content.toml#options.cv_paper"
    );
    let listed = render::list_documents(&workspace).unwrap();
    let entry = listed
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            entry["document"] == "cv" && entry["substyle"] == "plain" && entry["locale"] == "en-ch"
        })
        .unwrap();
    assert_eq!(entry["paper"], spec.inputs["paper"]);
    assert_eq!(
        entry["output"],
        workspace.relative(&spec.output).unwrap().to_str().unwrap()
    );
    assert_eq!(
        settings::explain(&workspace, &leaf, Some("a4")).unwrap()["paper"]["selection_source"],
        "CLI --paper"
    );
    // Invalid hidden lower geometry cannot be rescued by a valid preset.
    let layout = leaf.dir.join("layout.toml");
    let text = fs::read_to_string(&layout).unwrap();
    fs::write(
        layout,
        text.replace("[page]", "[page]\npaper = \"nonsense\""),
    )
    .unwrap();
    assert!(settings::resolve_with_paper(&workspace, &leaf, Some("a4")).is_err());
}
