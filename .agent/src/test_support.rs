//! Synthetic test workspaces. Tests never read the checked-in showcase under
//! `cvl/`: they copy `.agent/tests/fixtures/workspace` (see its README) or
//! generate a probe workspace, plus the engine-owned inputs every workspace
//! needs (`.agent/typst`, scaffolds and schemas). Real-data invariants of the
//! showcase are enforced by `ccvl check`; see `.agent/docs/ci.md`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::Workspace;
use serde_json::json;

/// Engine-owned repository inputs. Never use this for `cvl/` content.
pub(crate) fn repository() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// The checked-in synthetic workspace.
pub(crate) fn fixture_root() -> PathBuf {
    repository().join(".agent/tests/fixtures/workspace")
}

fn copy_tree(source: &Path, target: &Path, skip: &dyn Fn(&Path) -> bool) {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.unwrap();
        let relative = entry.path().strip_prefix(source).unwrap();
        if entry.file_type().is_file() && !skip(relative) {
            let destination = target.join(relative);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(entry.path(), destination).unwrap();
        }
    }
}

/// A read-only view of the checked-in fixture for tests that neither write
/// nor render; it lacks the engine inputs that `fixture_workspace` copies.
pub(crate) fn fixture_view() -> Workspace {
    Workspace::at(&fixture_root()).unwrap()
}

/// Copy one engine-owned repository file or directory into a test workspace.
pub(crate) fn copy_repository(root: &Path, relative: &str) {
    let source = repository().join(relative);
    if source.is_file() {
        let destination = root.join(relative);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(source, destination).unwrap();
    } else {
        copy_tree(&source, &root.join(relative), &|_| false);
    }
}

/// Copy the engine-owned inputs a workspace needs to render and validate:
/// the Typst library (without the embedded fonts), scaffolds and schemas.
pub(crate) fn copy_engine_inputs(root: &Path) {
    copy_tree(
        &repository().join(".agent/typst"),
        &root.join(".agent/typst"),
        &|relative| relative.starts_with("fonts"),
    );
    copy_repository(root, ".agent/scaffolds");
    copy_repository(root, ".agent/schemas");
}

/// A mutable copy of the synthetic workspace with the engine inputs.
pub(crate) fn fixture_workspace() -> (tempfile::TempDir, Workspace) {
    let directory = tempfile::tempdir().unwrap();
    copy_tree(&fixture_root(), directory.path(), &|relative| {
        relative == Path::new("README.md")
    });
    copy_engine_inputs(directory.path());
    let workspace = Workspace::at(directory.path()).unwrap();
    (directory, workspace)
}

/// The fixture workspace plus the platform files a full `check` validates:
/// skills, skill cases and bundled fonts. Guides that skills link to are
/// stubs, so the fixture never imports showcase documentation.
pub(crate) fn checkable_fixture_workspace() -> (tempfile::TempDir, Workspace) {
    let (directory, workspace) = fixture_workspace();
    let root = directory.path();
    let mut manifest = workspace.read_json("ccvl.json").unwrap();
    manifest["skills"] =
        crate::workspace::read_json(&repository().join("ccvl.json")).unwrap()["skills"].clone();
    fs::write(root.join("ccvl.json"), format!("{manifest:#}\n")).unwrap();
    for relative in [
        ".agent/skills",
        ".agent/typst/fonts",
        ".agent/tests/skill-cases.json",
    ] {
        copy_repository(root, relative);
    }
    for guide in [
        "applications",
        "editorial",
        "review",
        "cover-letter",
        "styles",
    ] {
        let path = root.join(format!(".agent/docs/{guide}.md"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "# Synthetic fixture guide\n").unwrap();
    }
    (directory, workspace)
}

/// A text-only paper probe, created in a temporary workspace for focused tests.
pub(crate) fn paper_workspace() -> (tempfile::TempDir, Workspace) {
    let source = repository();
    let fixture = fixture_root();
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
    ] {
        write(
            relative,
            &fs::read_to_string(source.join(relative)).unwrap(),
        );
    }
    for relative in ["cvl/shared/ledger/defaults.toml", "cvl/profile.toml"] {
        write(
            relative,
            &fs::read_to_string(fixture.join(relative)).unwrap(),
        );
    }
    for document in ["cv", "cl"] {
        let definition = json!({
            "id": "probe", "api": 1, "settings_adapter": "document-v1",
            "documents": [document], "supports_locales": ["en-ch", "en-us"],
            "pages": [1], "default_pages": 1,
            "substyles": ["plain"],
            "defaults": "../../shared/ledger/defaults.toml",
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
#let settings = paper-settings((toml("/cvl/shared/ledger/defaults.toml"), toml(sys.inputs.at("layout", default: "/{base}/layout.toml"))), preset)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{measure, render, styles};

    #[test]
    fn fixture_workspace_resolves_renders_and_measures_every_leaf() {
        let (_directory, workspace) = fixture_workspace();
        let compiler = render::Compiler::new(&workspace).unwrap();
        let specs = render::cvl_specs(&workspace).unwrap();
        // ledger CV: 2 substyles × 2 locales × 3 presets; grid: 1; ledger letter: 2 × 2.
        assert_eq!(specs.len(), 12 + 1 + 4);
        for spec in specs {
            let document = compiler.compile(&workspace, &spec).unwrap();
            let metrics = measure::document_metrics(&workspace, &spec, &document).unwrap();
            for (index, metric) in metrics.iter().enumerate() {
                assert_eq!(measure::line_failure(&spec, index, metric).unwrap(), None);
            }
            assert_eq!(
                measure::summary_failures(&workspace, &spec, &metrics).unwrap(),
                Vec::<String>::new(),
                "{}",
                spec.name
            );
        }
        for document in ["cv", "cl"] {
            styles::validate_slots(&workspace, document).unwrap();
        }
    }
}
