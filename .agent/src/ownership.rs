//! Ownership guard: measurement semantics live in `ctypst`, not here.
//!
//! Fails if anyone reintroduces a local measurement source builder,
//! calibration or line derivation, the retired ruler contract, or a vendored
//! copy of the shared program. Product rules, counsel, `wrap-exact`, and
//! document checks stay local by design and are not flagged.

#![cfg(test)]

use std::path::{Path, PathBuf};

use crate::workspace::Workspace;

// Split literals so this guard never flags its own forbidden list.
const FORBIDDEN: &[&str] = &[
    concat!("build_measure", "_all_source"),
    concat!("escape_for", "_measure"),
    concat!("escape_id", "_for_typst"),
    concat!("derive", "_lines"),
    concat!("cv-ruler", "-v1"),
    concat!("careervector-ruler", "-v1"),
];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut directories = vec![root.join(".agent/src"), root.join("cvl")];
    while let Some(directory) = directories.pop() {
        let entries = std::fs::read_dir(&directory).expect("workspace tree is readable");
        for entry in entries {
            let path = entry.expect("dir entry is readable").path();
            if path.is_dir() {
                if path
                    .file_name()
                    .is_some_and(|name| name != "target" && name != "output")
                {
                    directories.push(path);
                }
            } else if path.extension().is_some_and(|extension| {
                extension == "rs" || extension == "typ" || extension == "toml"
            }) {
                files.push(path);
            }
        }
    }
    files
}

#[test]
fn measurement_implementation_lives_in_ctypst() {
    let workspace = Workspace::at(&workspace_root()).expect("workspace resolves");
    assert!(
        workspace
            .root()
            .join("cvl/cv/harvard/standard/de/ch/content.toml")
            .is_file(),
        "ownership scan runs at the workspace root"
    );
    let mut violations = Vec::new();
    for path in source_files(workspace.root()) {
        let content = std::fs::read_to_string(&path).expect("source is readable");
        for forbidden in FORBIDDEN {
            if content.contains(forbidden) {
                violations.push(format!("{} contains {forbidden}", path.display()));
            }
        }
        if path
            .file_name()
            .is_some_and(|name| name == "measure-v1.typ")
        {
            violations.push(format!("{} vendors the shared program", path.display()));
        }
    }
    assert!(
        violations.is_empty(),
        "reintroduced measurement code:\n{}",
        violations.join("\n")
    );
}
