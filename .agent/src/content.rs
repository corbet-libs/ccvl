//! Explicit, one-level wording sharing within a document/style/locale.
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::{styles, workspace::Workspace};

/// Read a self-contained application, or resolve a registered showcase leaf's
/// shared wording. Returned records contain no source directive.
pub fn read_record(workspace: &Workspace, path: impl AsRef<Path>) -> Result<Value> {
    let path = workspace.existing_inside(path)?;
    let mut record = workspace.read_toml_value(workspace.relative(&path)?)?;
    let Some(reference) = record.get("wording") else {
        return Ok(record);
    };
    let reference = reference
        .as_object()
        .context("wording must be a table with one source")?;
    ensure!(reference.len() == 1, "wording must contain only source");
    let source = reference
        .get("source")
        .and_then(Value::as_str)
        .context("wording.source must be a path string")?;
    ensure!(
        path.file_name().is_some_and(|name| name == "content.toml")
            && !path.starts_with(workspace.path("opportunities")),
        "wording references are allowed only in registered showcase content.toml leaves; private opportunities must be self-contained"
    );
    let leaf = path.parent().context("wording record has no parent")?;
    let language = leaf
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .context("wording leaf has no language directory")?;
    let country = leaf
        .file_name()
        .and_then(|name| name.to_str())
        .context("wording leaf has no country directory")?;
    let style_dir = leaf
        .ancestors()
        .nth(3)
        .context("wording leaf has no style directory")?;
    let expected_source = format!("../../../content/{language}/{country}/wording.toml");
    ensure!(
        source == expected_source,
        "{}: wording.source must be {expected_source:?}; sharing is scoped to the same document, style and locale",
        path.display()
    );
    let expected_path = style_dir
        .join("content")
        .join(language)
        .join(country)
        .join("wording.toml");
    let source_path = workspace.existing_inside(path.parent().unwrap().join(source))?;
    ensure!(
        source_path == expected_path,
        "wording.source must not escape its document/style/locale through a symlink"
    );
    let shared = workspace.read_toml_value(workspace.relative(&source_path)?)?;
    let shared = shared
        .as_object()
        .context("shared wording must be a table")?;
    ensure!(
        shared.len() == 1,
        "shared wording must contain only [cv] or [cl]; nested sources and cross-document content are forbidden"
    );
    let document = match shared.keys().next().map(String::as_str) {
        Some("cv") => "cv",
        Some("cl") => "cl",
        _ => anyhow::bail!(
            "shared wording must contain only [cv] or [cl]; nested sources are forbidden"
        ),
    };
    // The shared table identifies its document; only that root is resolved or observed.
    // The registered leaf path then proves this is the table's actual owner.
    validate_owner(workspace, &path, document)?;
    let mut wording = shared[document].clone();
    ensure!(
        wording.is_object(),
        "shared [{document}] wording must be a table"
    );
    if let Some(overrides) = record.get(document) {
        ensure!(
            overrides.is_object(),
            "leaf [{document}] wording overrides must be a table"
        );
        merge(&mut wording, overrides);
    }
    let fields = record
        .as_object_mut()
        .context("application must be a table")?;
    fields.remove("wording");
    fields.insert(document.to_owned(), wording);
    Ok(record)
}

fn validate_owner(workspace: &Workspace, path: &Path, document: &str) -> Result<()> {
    let root = styles::root(workspace, document)?;
    let relative = path.strip_prefix(&root).context(
        "shared wording table must match its owning document root; private opportunities must be self-contained"
    )?;
    let parts = relative
        .iter()
        .map(|part| part.to_str().context("wording path must be UTF-8"))
        .collect::<Result<Vec<_>>>()?;
    let [style, substyle, language, country, "content.toml"] = parts.as_slice() else {
        anyhow::bail!(
            "wording references require a registered document/style/substyle/locale leaf"
        );
    };
    let definition = styles::definition(workspace, document, style)?;
    ensure!(
        definition.designed_substyles().any(|name| name == substyle),
        "wording references require a registered substyle leaf"
    );
    let locale = format!("{language}-{country}");
    ensure!(
        definition.supports_locales.contains(&locale),
        "wording references require a registered locale leaf"
    );
    Ok(())
}

/// Tables merge recursively; every scalar or array replaces the complete value.
/// Empty arrays intentionally clear arrays. There is no deletion or index patch syntax.
fn merge(base: &mut Value, overrides: &Value) {
    if let (Some(base), Some(overrides)) = (base.as_object_mut(), overrides.as_object()) {
        for (key, value) in overrides {
            merge(base.entry(key).or_insert(Value::Null), value);
        }
    } else {
        *base = overrides.clone();
    }
}

#[cfg(test)]
mod tests;
