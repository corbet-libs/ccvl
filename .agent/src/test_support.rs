//! A text-only paper probe, created in a temporary workspace for focused tests.

use std::{fs, path::Path};

use crate::Workspace;
use serde_json::json;

pub(crate) fn paper_workspace() -> (tempfile::TempDir, Workspace) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let write = |relative: &str, text: &str| {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    };
    write(
        "ccvl.json",
        &json!({
            "format": "ccvl-workspace", "schema_version": 8,
            "documents": {
                "cv": {"root": "cvl/cv", "default_style": "probe"},
                "cover_letter": {"root": "cvl/cl", "default_style": "probe"}
            }
        })
        .to_string(),
    );
    for relative in [
        ".agent/typst/paper.typ",
        ".agent/typst/document.typ",
        ".agent/typst/document-settings.typ",
        ".agent/typst/document-settings.json",
        "cvl/shared/harvard/defaults.toml",
        "cvl/profile.toml",
    ] {
        write(
            relative,
            &fs::read_to_string(source.join(relative)).unwrap(),
        );
    }
    for document in ["cv", "cl"] {
        let definition = json!({
            "id": "probe", "api": 1, "settings_adapter": "document-v1",
            "documents": [document], "supports_locales": ["en-ch", "en-us"],
            "pages": [1], "default_pages": 1,
            "substyles": ["plain"], "default_substyle": "plain",
            "defaults": "../../shared/harvard/defaults.toml",
            "paper": {
                "defaults": {"en-ch": "a4", "en-us": "us-letter"},
                "sizes": {
                    "a4": {"size_pt": [595.2756, 841.8898], "label": "A4", "settings": {"page": {"paper": "a4"}}},
                    "us-letter": {"size_pt": [612, 792], "label": "US-LETTER", "settings": {"page": {"paper": "us-letter"}}},
                    "landscape": {"size_pt": [841.8898, 595.2756], "label": "Landscape A4", "settings": {"page": {"paper": "a4", "flipped": true}}}
                }
            }
        });
        write(
            &format!("cvl/{document}/probe/style.toml"),
            &toml::to_string(&definition).unwrap(),
        );
        write(&format!("cvl/{document}/probe/plain/substyle.toml"), "");
        for region in ["ch", "us"] {
            let base = format!("cvl/{document}/probe/plain/en/{region}");
            let mut record: serde_json::Value = toml::from_str(
                &fs::read_to_string(source.join(".agent/scaffolds/opportunity/application.toml"))
                    .unwrap(),
            )
            .unwrap();
            record["options"] = json!({
                "language": format!("en-{region}"), "pages": 1, "cl_pages": 1, "generate_cl": true,
                "application_date": "", "cv_style": "probe", "cv_substyle": "plain", "cl_style": "probe", "cl_substyle": "plain"
            });
            record["job"]["id"] = "paper-probe".into();
            record["cl"] = json!({"message": "Paper geometry test."});
            record["cv"] = json!({"message": "Paper geometry test."});
            write(
                &format!("{base}/content.toml"),
                &toml::to_string(&record).unwrap(),
            );
            write(&format!("{base}/strings.toml"), "");
            write(
                &format!("{base}/layout.toml"),
                &format!("[page]\n[text]\nlang = \"en\"\nregion = {region:?}\n"),
            );
            write(
                &format!("{base}/typst/{document}.typ"),
                &format!(
                    r#"
#import "/.agent/typst/document.typ": apply-document-settings
#import "/.agent/typst/paper.typ": resolve-paper, paper-settings
#let record = toml(sys.inputs.at("application", default: "/{base}/content.toml"))
#let preset = resolve-paper(toml("/cvl/{document}/probe/style.toml"), "en-{region}", requested: sys.inputs.at("paper", default: ""), recorded: record.options.at("{document}_paper", default: none))
#let settings = paper-settings((toml("/cvl/shared/harvard/defaults.toml"), toml(sys.inputs.at("layout", default: "/{base}/layout.toml"))), preset)
#show: apply-document-settings.with(settings)
#preset.label
#record.{document}.message
"#
                ),
            );
        }
    }
    let workspace = Workspace::at(root).unwrap();
    (directory, workspace)
}
