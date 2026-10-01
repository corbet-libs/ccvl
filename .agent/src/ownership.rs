//! Ownership guard: measurement semantics live in `ctypst`, not here.
//!
//! Flags a reintroduced local measurement source builder, calibration or line
//! derivation, the retired ruler contract, or a vendored copy of the shared
//! program. Product rules, counsel, `wrap-exact`, and document checks stay
//! local by design and are not flagged. The engine sources are scanned by a
//! unit test; `check` scans the document styles under `cvl/`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

// Split literals so this guard never flags its own forbidden list.
const FORBIDDEN: &[&str] = &[
    concat!("build_measure", "_all_source"),
    concat!("escape_for", "_measure"),
    concat!("escape_id", "_for_typst"),
    concat!("derive", "_lines"),
    concat!("cv-ruler", "-v1"),
    concat!("careervector-ruler", "-v1"),
];

fn source_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        let entries = std::fs::read_dir(&directory)
            .with_context(|| format!("cannot read {}", directory.display()))?;
        for entry in entries {
            let path = entry?.path();
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
    files.sort();
    Ok(files)
}

/// Every reintroduced measurement implementation below `root`.
pub fn violations(root: &Path) -> Result<Vec<String>> {
    let mut violations = Vec::new();
    for path in source_files(root)? {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;
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
    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measurement_implementation_lives_in_ctypst() {
        let violations = violations(&crate::test_support::repository().join(".agent/src")).unwrap();
        assert!(
            violations.is_empty(),
            "reintroduced measurement code:\n{}",
            violations.join("\n")
        );
    }

    #[test]
    fn scan_flags_forbidden_builders_and_a_vendored_program() {
        let directory = tempfile::tempdir().unwrap();
        let style = directory.path().join("cv/example");
        std::fs::create_dir_all(style.join("output")).unwrap();
        std::fs::write(style.join("clean.typ"), "#let ok = 1\n").unwrap();
        std::fs::write(style.join("output/ignored.typ"), concat!("cv-ruler", "-v1")).unwrap();
        assert_eq!(violations(directory.path()).unwrap(), Vec::<String>::new());
        std::fs::write(
            style.join("layout.typ"),
            concat!("#let ", "derive", "_lines = none\n"),
        )
        .unwrap();
        std::fs::write(style.join("measure-v1.typ"), "").unwrap();
        let found = violations(directory.path()).unwrap();
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(found[0].ends_with(concat!("layout.typ contains ", "derive", "_lines")));
        assert!(found[1].ends_with("measure-v1.typ vendors the shared program"));
    }
}
