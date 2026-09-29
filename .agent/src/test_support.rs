//! Isolated style fixtures that never enter the shipped document registry.

use std::{fs, path::Path};

use tempfile::TempDir;

use crate::Workspace;

pub(crate) fn independent_styles() -> (TempDir, Workspace) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::copy(source.join("ccvl.json"), root.join("ccvl.json")).unwrap();
    for relative in [".agent/typst", "cvl"] {
        copy_sources(&source.join(relative), &root.join(relative));
    }
    copy_sources(
        &source.join(".agent/tests/fixtures/independent-styles/cvl"),
        &root.join("cvl"),
    );
    let workspace = Workspace::at(root).unwrap();
    (directory, workspace)
}

fn copy_sources(source: &Path, destination: &Path) {
    for entry in walkdir::WalkDir::new(source)
        .into_iter()
        .filter_entry(|entry| {
            !["pdf", "preview", "studies"]
                .iter()
                .any(|name| entry.file_name() == *name)
        })
    {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let target = destination.join(entry.path().strip_prefix(source).unwrap());
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::copy(entry.path(), target).unwrap();
        }
    }
}
