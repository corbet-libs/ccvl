use super::*;
use crate::styles;
use std::fs;

fn repository() -> (tempfile::TempDir, Workspace) {
    crate::test_support::fixture_workspace()
}

fn selected(workspace: &Workspace, style: &str, substyle: &str, locale: &str) -> StyleLeaf {
    let selection = styles::selection(workspace, "cv", Some(style), Some(substyle)).unwrap();
    styles::leaf(workspace, "cv", locale, &selection).unwrap()
}

#[test]
fn explanation_preserves_precedence_and_reports_each_winning_source() {
    let (_fixture, workspace) = repository();
    let compact = resolve(
        &workspace,
        &selected(&workspace, "ledger", "compact", "en-ch"),
    )
    .unwrap();
    assert_eq!(compact["settings"]["paragraph"]["leading_em"], 0.7);
    assert!(compact["settings"]["text"].get("leading_em").is_none());
    assert_eq!(compact["settings"]["header"]["after_pt"], 5.5);
    assert_eq!(
        compact["origins"]["/header/after_pt"],
        "cvl/cv/ledger/compact/substyle.toml"
    );
    assert_eq!(
        compact["origins"]["/paragraph/leading_em"],
        "cvl/shared/ledger/defaults.toml"
    );
    assert_eq!(
        compact["origins"]["/page/paper"],
        "cvl/cv/ledger/style.toml#paper.sizes.a4.settings"
    );
    assert_eq!(compact["sources"].as_array().unwrap().len(), 4);
    let standard = resolve(
        &workspace,
        &selected(&workspace, "ledger", "primary", "en-ch"),
    )
    .unwrap();
    assert_eq!(standard["settings"]["page"]["paper"], "a4");
    assert_eq!(standard["settings"]["text"]["font"], "Archivo");
    assert_eq!(
        standard["origins"]["/page/paper"],
        "cvl/cv/ledger/style.toml#paper.sizes.a4.settings"
    );
    // An independent renderer is never assigned this schema or a guessed merge.
    let mut custom = selected(&workspace, "ledger", "primary", "en-ch");
    custom.settings_adapter = None;
    assert!(
        resolve(&workspace, &custom)
            .unwrap_err()
            .to_string()
            .contains("renderer owns settings resolution")
    );
}

#[test]
fn rust_and_direct_typst_reject_the_same_invalid_adapter_settings() {
    let (_fixture, workspace) = repository();
    let schema = workspace.read_json(SCHEMA).unwrap();
    let settings = resolve(
        &workspace,
        &selected(&workspace, "ledger", "primary", "en-ch"),
    )
    .unwrap()["settings"]
        .clone();
    let engine = ctypst::Engine::builder()
        .root(workspace.root())
        .fonts(ctypst::fonts::documents())
        .build()
        .unwrap();
    for (group, key, value) in [
        ("block", "align", json!("centre")),
        ("page", "binding", json!("lef")),
        ("text", "cjk_latin_spacing", json!("off")),
        ("text", "direction", json!("ltrr")),
        ("text", "hyphenate", json!("false")),
        ("text", "fallback", json!("true")),
        ("text", "size_pt", json!(0)),
        ("text", "weight", json!(1000)),
        ("page", "columns", json!(1.5)),
        ("page", "margin_top_mm", json!(-1)),
        ("paragraph", "linebreaks", json!("fast")),
        ("paragraph", "leading", json!(0.7)),
        ("text", "leading_em", json!(0.7)),
        ("page", "fill", json!("blak")),
        ("text", "features", json!({"badtag": true})),
        ("text", "variations", json!({"wght": "bold"})),
    ] {
        let mut invalid = settings.clone();
        invalid[group][key] = value;
        assert!(validate(&invalid, &schema, true).is_err(), "{group}.{key}");
        let source = format!(
            "#import \"/.agent/typst/document.typ\": apply-document-settings\n#let settings = json(bytes({:?}))\n#show: apply-document-settings.with(settings)\nValidation fixture.",
            invalid.to_string()
        );
        let error = engine
            .compile(
                ctypst::CompileRequest::new("validation.typ").source_file("validation.typ", source),
            )
            .err()
            .expect("invalid settings must fail");
        assert!(
            error.to_string().contains(&format!("{group}.{key}")),
            "{group}.{key}: {error}"
        );
    }
    // New design-specific tables remain unrestricted even with the adapter.
    let mut extended = settings;
    extended["custom_geometry"] = json!({"shape": "hexagon", "nested": {"layers": [1, 2, 3]}});
    extended["block"]["align"] = "center".into();
    extended["page"]["binding"] = "right".into();
    extended["text"]["cjk_latin_spacing"] = "none".into();
    extended["page"]["paper"] = "custom".into();
    extended["page"]["width_mm"] = 200.into();
    extended["page"]["height_mm"] = 160.into();
    validate(&extended, &schema, true).unwrap();
    let source = format!(
        "#import \"/.agent/typst/document.typ\": apply-document-settings\n#let settings = json(bytes({:?}))\n#show: apply-document-settings.with(settings)\nValidation fixture.",
        extended.to_string()
    );
    engine
        .compile(
            ctypst::CompileRequest::new("validation.typ").source_file("validation.typ", source),
        )
        .unwrap();
}

#[test]
fn custom_dimensions_unused_fields_missing_values_and_reversed_bounds_fail() {
    let (_fixture, workspace) = repository();
    let schema = workspace.read_json(SCHEMA).unwrap();
    let settings = resolve(
        &workspace,
        &selected(&workspace, "ledger", "primary", "en-ch"),
    )
    .unwrap()["settings"]
        .clone();
    let mut custom = settings.clone();
    custom["page"]["paper"] = "custom".into();
    assert!(validate(&custom, &schema, true).is_err());
    custom["page"]["width_mm"] = 200.into();
    custom["page"]["height_mm"] = 100.into();
    validate(&custom, &schema, true).unwrap();
    custom["page"]["paper"] = "us-letter".into();
    assert!(validate(&custom, &schema, true).is_err());
    let mut missing = settings.clone();
    missing["paragraph"]
        .as_object_mut()
        .unwrap()
        .remove("leading_em");
    assert!(validate(&missing, &schema, true).is_err());
    let mut reversed = settings;
    reversed["paragraph"]["tracking_min_pt"] = 1.into();
    assert!(validate(&reversed, &schema, true).is_err());
}

#[test]
fn invalid_overridden_input_still_names_its_source_file() {
    let (_fixture, original) = repository();
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    fs::write(root.join("ccvl.json"), "{}").unwrap();
    let mut leaf = selected(&original, "ledger", "primary", "en-ch");
    let relative = original.relative(&leaf.dir).unwrap();
    leaf.dir = root.join(relative);
    fs::create_dir_all(&leaf.dir).unwrap();
    leaf.defaults = Some(root.join("defaults.toml"));
    fs::copy(
        original.path("cvl/shared/ledger/defaults.toml"),
        root.join("defaults.toml"),
    )
    .unwrap();
    fs::create_dir_all(root.join(".agent/typst")).unwrap();
    fs::copy(original.path(SCHEMA), root.join(SCHEMA)).unwrap();
    fs::write(leaf.substyle_file(), "[block]\nalign = \"typo\"\n").unwrap();
    fs::write(
        leaf.dir.join("layout.toml"),
        "[block]\nalign = \"left\"\n[text]\nlang = \"en\"\nregion = \"CH\"\n",
    )
    .unwrap();
    let workspace = Workspace::at(root).unwrap();
    let error = format!("{:#}", resolve(&workspace, &leaf).unwrap_err());
    assert!(
        error.contains("substyle.toml") && error.contains("block.align"),
        "{error}"
    );
}
