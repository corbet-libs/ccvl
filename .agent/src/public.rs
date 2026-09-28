use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use regex::Regex;
use walkdir::WalkDir;

use crate::workspace::Workspace;

const EXCLUDED_ROOTS: &[&str] = &[
    ".cache",
    ".git",
    "applications",
    "evidence",
    "out",
    "outcomes",
    "private",
    "sources",
    "submissions",
    "target",
    "tmp",
];

fn is_excluded_path(path: &Path) -> bool {
    path.starts_with(".agent/cache")
        || path.components().next().is_some_and(|part| {
            EXCLUDED_ROOTS
                .iter()
                .any(|excluded| part.as_os_str() == *excluded)
        })
}

pub fn public_files(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(workspace.root()).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = workspace.relative(entry.path())?;
        if is_excluded_path(&relative) {
            continue;
        }
        files.push(entry.into_path());
    }
    files.sort();
    Ok(files)
}

pub fn validate_repository(workspace: &Workspace) -> Result<()> {
    validate_no_python_artifacts(workspace)?;
    validate_markdown_links(workspace)?;
    validate_text_files(workspace)
}

fn validate_no_python_artifacts(workspace: &Workspace) -> Result<()> {
    let mut offenders = public_files(workspace)?
        .into_iter()
        .filter_map(|path| {
            workspace
                .relative(&path)
                .ok()
                .filter(|relative| is_python_artifact(relative) && !is_ci_python_helper(relative))
        })
        .collect::<BTreeSet<_>>();
    for entry in WalkDir::new(workspace.root()).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_dir() {
            continue;
        }
        let relative = workspace.relative(entry.path())?;
        let excluded = is_excluded_path(&relative);
        if !excluded && is_python_artifact(&relative) {
            offenders.insert(relative);
        }
    }
    ensure!(
        offenders.is_empty(),
        "public repository contains forbidden Python source or toolchain metadata: {}",
        offenders
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

fn is_ci_python_helper(path: &Path) -> bool {
    // CI orchestration and focused rendering use these explicit helpers. This is not a
    // second product runtime or permission for arbitrary Python source.
    [
        ".agent/scripts/release-evidence.py",
        ".agent/scripts/downstream-check.py",
        ".agent/scripts/candidate-date.py",
        ".agent/tests/test_release_evidence.py",
        ".agent/tests/test_downstream_check.py",
        ".agent/tests/test_application_dates.py",
    ]
    .iter()
    .any(|candidate| path == Path::new(candidate))
}

fn is_python_artifact(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            ["py", "pyc", "pyo"]
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
        || path
            .components()
            .any(|part| part.as_os_str() == "__pycache__")
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, "pyproject.toml" | "uv.lock" | ".python-version"))
}

pub fn validate_boundary(workspace: &Workspace) -> Result<()> {
    for root in [
        "applications",
        "evidence",
        "out",
        "outcomes",
        "private",
        "sources",
        "submissions",
    ] {
        ensure!(
            !workspace.path(root).exists(),
            "private downstream path exists: {root}"
        );
    }
    let allowed = BTreeSet::from([
        PathBuf::from("interview/README.md"),
        PathBuf::from("interview/imports/README.md"),
        PathBuf::from("interview/stations.toml"),
        PathBuf::from("opportunities/README.md"),
    ]);
    for path in public_files(workspace)? {
        let relative = workspace.relative(&path)?;
        let in_workspace_data = relative.components().next().is_some_and(|part| {
            part.as_os_str() == "opportunities" || part.as_os_str() == "interview"
        });
        ensure!(
            !in_workspace_data || allowed.contains(&relative),
            "public workspace contains private interview or opportunity data: {}",
            relative.display()
        );
    }
    for entry in WalkDir::new(workspace.root()).follow_links(false) {
        let entry = entry?;
        let relative = workspace.relative(entry.path())?;
        if relative.starts_with(".git")
            || relative.starts_with(".cache")
            || relative.starts_with(".agent/cache")
            || relative.starts_with("target")
        {
            continue;
        }
        ensure!(
            !entry.path_is_symlink(),
            "symlink requires manual publication review: {}",
            relative.display()
        );
    }
    for root in ["cvl", ".agent/typst"] {
        for entry in WalkDir::new(workspace.path(root)) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let bytes = fs::read(entry.path())?;
                ensure!(
                    !bytes.starts_with(b"version https://git-lfs.github.com/spec/v1"),
                    "unresolved Git LFS pointer: {}",
                    workspace.relative(entry.path())?.display()
                );
            }
        }
    }
    let secret = Regex::new(
        r"(?m)-----BEGIN (?:[A-Z ]+ )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|ASIA[0-9A-Z]{16}|github_pat_[A-Za-z0-9_]{20,}|gh[pousr]_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9_-]{20,}|xox[baprs]-[A-Za-z0-9-]{10,}",
    )?;
    let private = Regex::new(concat!(
        r"/home/",
        "richc",
        r"|julian-corbet/",
        "applications",
        r"|BEGIN OPENSSH ",
        "PRIVATE KEY"
    ))?;
    for path in public_files(workspace)? {
        let relative = workspace.relative(&path)?;
        let is_font = relative.starts_with(".agent/typst/fonts");
        let is_output = relative.extension().is_some_and(|ext| ext == "pdf")
            && relative.components().any(|part| {
                part.as_os_str() == "output"
                    || part.as_os_str() == "pdfs"
                    || part.as_os_str() == "pdf"
            });
        if relative != Path::new("cvl/assets/signature.png") && !is_font && !is_output {
            let text = String::from_utf8_lossy(&fs::read(&path)?).into_owned();
            ensure!(
                !secret.is_match(&text),
                "potential secret found: {}",
                relative.display()
            );
            let declares_public_identifiers =
                relative == Path::new(".agent/docs/public-identifiers.md");
            if !declares_public_identifiers {
                ensure!(
                    !private.is_match(&text),
                    "private workspace identifier found: {}",
                    relative.display()
                );
            }
        }
    }
    Ok(())
}

fn validate_markdown_links(workspace: &Workspace) -> Result<()> {
    let link = Regex::new(r"!?\[[^]]*\]\(([^)]+)\)")?;
    let mut errors = Vec::new();
    for path in public_files(workspace)?
        .into_iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
    {
        let text = fs::read_to_string(&path)?;
        for captures in link.captures_iter(&text) {
            let mut destination = captures[1].trim().to_owned();
            if destination.starts_with('<') && destination.contains('>') {
                destination = destination[1..destination.find('>').expect("checked")].to_owned();
            } else {
                destination = destination
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_owned();
            }
            if destination.is_empty()
                || ["#", "http://", "https://", "mailto:"]
                    .iter()
                    .any(|prefix| destination.starts_with(prefix))
            {
                continue;
            }
            let clean = destination
                .split(['#', '?'])
                .next()
                .unwrap_or_default()
                .replace("%20", " ");
            let candidate = if clean.starts_with('/') {
                workspace.path(clean.trim_start_matches('/'))
            } else {
                path.parent()
                    .context("Markdown file has no parent")?
                    .join(clean)
            };
            if !candidate.exists() {
                errors.push(format!(
                    "{} -> {destination}",
                    workspace.relative(&path)?.display()
                ));
            }
        }
    }
    if !errors.is_empty() {
        bail!("broken local Markdown links: {}", errors.join(", "));
    }
    Ok(())
}

fn is_review_json(path: &Path) -> bool {
    let parts = path.components().collect::<Vec<_>>();
    parts.len() >= 5
        && parts[0].as_os_str() == "opportunities"
        && parts[3].as_os_str() == "review"
        && path
            .extension()
            .is_some_and(|extension| extension == "json")
}

fn validate_text_files(workspace: &Workspace) -> Result<()> {
    let suffixes = [
        "cmd", "csv", "json", "lock", "md", "ps1", "rs", "sh", "toml", "typ", "yaml", "yml",
    ];
    let names = [".gitattributes", ".gitignore", "ccvl"];
    let trailing = Regex::new(r"(?m)[ \t]+$")?;
    let merge = Regex::new(r"(?m)^(<{7}|={7}|>{7})(?: |$)")?;
    let mut errors = Vec::new();
    for path in public_files(workspace)? {
        let relative = workspace.relative(&path)?;
        let textual = relative
            .extension()
            .and_then(|item| item.to_str())
            .is_some_and(|ext| suffixes.contains(&ext))
            || relative
                .file_name()
                .and_then(|item| item.to_str())
                .is_some_and(|name| names.contains(&name));
        if !textual {
            continue;
        }
        let bytes = fs::read(&path)?;
        let Ok(text) = String::from_utf8(bytes) else {
            errors.push(format!("{}: not valid UTF-8", relative.display()));
            continue;
        };
        // Review packets use exact serialized bytes for hashes and recovery.
        // Saved specifications/results can also be frozen as source evidence.
        // Their JSON encoding need not end in LF; never normalize these bytes.
        let review_json = is_review_json(&relative);
        if review_json && let Err(error) = serde_json::from_str::<serde_json::Value>(&text) {
            errors.push(format!(
                "{}: invalid review JSON: {error}",
                relative.display()
            ));
        }
        if !review_json && !text.is_empty() && !text.ends_with('\n') {
            errors.push(format!("{}: missing final newline", relative.display()));
        }
        if text.contains('\r') {
            errors.push(format!("{}: contains CR line endings", relative.display()));
        }
        if trailing.is_match(&text) {
            errors.push(format!("{}: trailing whitespace", relative.display()));
        }
        if merge.is_match(&text) {
            errors.push(format!("{}: unresolved merge marker", relative.display()));
        }
    }
    if !errors.is_empty() {
        bail!("text hygiene failures: {}", errors.join(", "));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn review_json_newline_policy_keeps_other_hygiene_checks() {
        let directory = tempdir().unwrap();
        fs::write(directory.path().join("ccvl.json"), "{}\n").unwrap();
        let workspace = Workspace::at(directory.path()).unwrap();
        let relative = "opportunities/example/role/review/run/revision-0/manifest.json";
        let path = workspace.path(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        for text in ["{}", "{}\n"] {
            fs::write(&path, text).unwrap();
            validate_text_files(&workspace).unwrap();
            assert_eq!(fs::read_to_string(&path).unwrap(), text);
        }
        for (text, message) in [
            ("{", "invalid review JSON"),
            ("{\n", "invalid review JSON"),
            ("", "invalid review JSON"),
            ("{}\r\n", "contains CR line endings"),
            ("{} \n", "trailing whitespace"),
            ("<<<<<<< conflict\n{}", "unresolved merge marker"),
        ] {
            fs::write(&path, text).unwrap();
            let error = validate_text_files(&workspace).unwrap_err().to_string();
            assert!(error.contains(message), "{error}");
        }
        fs::write(&path, b"\xff").unwrap();
        assert!(
            validate_text_files(&workspace)
                .unwrap_err()
                .to_string()
                .contains("not valid UTF-8")
        );
        fs::remove_file(&path).unwrap();

        for relative in [
            "config.json",
            "cvl/review/result.json",
            "opportunities/example/role/content.json",
            "opportunities/example/role/reviews/result.json",
            "opportunities/example/review/result.json",
            "opportunities/example/role/review/notes.md",
        ] {
            let path = workspace.path(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "{}").unwrap();
            let error = validate_text_files(&workspace).unwrap_err().to_string();
            assert!(error.contains("missing final newline"), "{error}");
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn review_json_still_cannot_be_published() {
        let directory = tempdir().unwrap();
        fs::write(directory.path().join("ccvl.json"), "{}\n").unwrap();
        let workspace = Workspace::at(directory.path()).unwrap();
        let path = workspace.path("opportunities/example/role/review/result.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "{}").unwrap();
        validate_repository(&workspace).unwrap();
        let error = validate_boundary(&workspace).unwrap_err().to_string();
        assert!(
            error.contains("private interview or opportunity data"),
            "{error}"
        );
    }

    #[test]
    fn python_source_and_toolchain_metadata_are_forbidden() {
        for path in [
            "script.py",
            ".agent/scripts/check.PY",
            ".agent/scripts/check.pyc",
            ".agent/scripts/check.PYO",
            ".agent/scripts/__pycache__",
            ".agent/scripts/__pycache__/check.cpython-314.pyc",
            "pyproject.toml",
            "uv.lock",
            ".python-version",
        ] {
            assert!(is_python_artifact(Path::new(path)), "{path} was accepted");
        }
        for path in [
            "Cargo.toml",
            "Cargo.lock",
            ".agent/src/main.rs",
            ".agent/scripts/check.sh",
        ] {
            assert!(!is_python_artifact(Path::new(path)), "{path} was rejected");
        }
    }

    #[test]
    fn ci_python_helpers_do_not_allow_unapproved_python_source() {
        let directory = tempdir().unwrap();
        fs::write(directory.path().join("ccvl.json"), "{}\n").unwrap();
        for relative in [
            ".agent/scripts/release-evidence.py",
            ".agent/scripts/downstream-check.py",
            ".agent/scripts/candidate-date.py",
            ".agent/tests/test_release_evidence.py",
            ".agent/tests/test_downstream_check.py",
            ".agent/tests/test_application_dates.py",
        ] {
            let path = directory.path().join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "# CI helper fixture\n").unwrap();
        }
        let workspace = Workspace::at(directory.path()).unwrap();
        validate_no_python_artifacts(&workspace).unwrap();
        for relative in [".agent/scripts/unapproved.py", ".agent/src/product.py"] {
            let path = directory.path().join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "# forbidden product runtime\n").unwrap();
            assert!(validate_no_python_artifacts(&workspace).is_err());
            fs::remove_file(path).unwrap();
        }
        validate_no_python_artifacts(&workspace).unwrap();
    }

    #[test]
    fn empty_python_cache_directory_is_rejected_on_the_public_surface() {
        let directory = tempdir().unwrap();
        fs::write(directory.path().join("ccvl.json"), "{}\n").unwrap();
        fs::create_dir_all(directory.path().join(".agent/scripts/__pycache__")).unwrap();
        let workspace = Workspace::at(directory.path()).unwrap();
        let error = validate_no_python_artifacts(&workspace)
            .unwrap_err()
            .to_string();
        assert!(error.contains(".agent/scripts/__pycache__"));
    }
}
