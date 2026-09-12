//! Filesystem adapter for publishing portable style variants without user records.
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use ccvl_core::{StyleBundle, StyleDocument};
use serde_json::json;

use crate::{Workspace, styles};

#[allow(clippy::too_many_arguments)]
pub fn export(
    workspace: &Workspace,
    document: &str,
    locale: &str,
    style: Option<&str>,
    substyle: Option<&str>,
    pages: Option<usize>,
    paper: Option<&str>,
) -> Result<StyleBundle> {
    let document = match document {
        "cv" => "cv",
        "cl" => "cl",
        _ => anyhow::bail!("unsupported document type: {document}"),
    };
    let selection = styles::selection(workspace, document, style, substyle)?;
    let leaf = styles::leaf(workspace, document, locale, &selection)?;
    ensure!(
        leaf.contract
            .get("source_files")
            .and_then(serde_json::Value::as_array)
            .is_none_or(Vec::is_empty),
        "this style embeds candidate source files; move them to document inputs before exporting a reusable bundle"
    );
    let pages = pages.unwrap_or(leaf.default_pages);
    ensure!(
        leaf.pages.contains(&pages),
        "unsupported style page count: {pages}"
    );
    let selected_paper = crate::paper::select(leaf.paper.as_ref(), &leaf.locale, paper)?;
    if leaf.settings_adapter.is_some() {
        crate::settings::resolve_with_paper(workspace, &leaf, paper)?;
    }
    let mut inputs = crate::render::style_inputs(workspace, &leaf, pages, "", "", paper)?;
    inputs.remove("application");
    inputs.remove("profile");
    let mut files = BTreeMap::new();
    for directory in [
        workspace.path(".agent/typst"),
        leaf.style_dir().to_path_buf(),
        workspace.path(format!("cvl/shared/{}", leaf.style)),
        workspace.path("LICENSES"),
    ] {
        if directory.is_dir() {
            collect(workspace, &directory, &mut files)?;
        }
    }
    if let Some(defaults) = &leaf.defaults {
        add(workspace, defaults, &mut files)?;
    }
    let fonts = leaf
        .fonts
        .iter()
        .map(|path| add(workspace, path, &mut files))
        .collect::<Result<Vec<_>>>()?;
    let scaffold_path = leaf.style_dir().join("scaffold.toml");
    let scaffold = if scaffold_path.is_file() {
        workspace.read_toml_value(workspace.relative(&scaffold_path)?)?
    } else {
        json!({})
    };
    let entry = workspace
        .relative(&leaf.adapter())?
        .to_string_lossy()
        .replace('\\', "/");
    let mut bundle = StyleBundle {
        schema_version: ccvl_core::style::BUNDLE_SCHEMA,
        engine_api: 1,
        id: format!(
            "{document}/{}/{}/{}/{pages}/{}",
            leaf.style,
            leaf.substyle,
            leaf.locale,
            selected_paper.map_or("fixed", |(id, _)| id)
        ),
        version: "pending-content-hash".into(),
        document: document.into(),
        style: leaf.style.clone(),
        substyle: leaf.substyle.clone(),
        locale: leaf.locale.clone(),
        pages,
        entry,
        inputs,
        contract: crate::paper::contract(&leaf, selected_paper),
        scaffold,
        fonts,
        files,
    };
    bundle.version = format!("sha256:{}", bundle.sha256()?);
    bundle.validate()?;
    Ok(bundle)
}

fn collect(
    workspace: &Workspace,
    directory: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().context("style path must be UTF-8")?;
        // These directories/files contain user wording or generated showcase output.
        if matches!(
            name,
            "content" | "pdf" | "pdfs" | "preview" | "content.toml" | "wording.toml" | "README.md"
        ) {
            continue;
        }
        ensure!(
            !entry.file_type()?.is_symlink(),
            "style bundles cannot contain symlinks"
        );
        if entry.file_type()?.is_dir() {
            collect(workspace, &entry.path(), files)?;
        } else {
            add(workspace, &entry.path(), files)?;
        }
    }
    Ok(())
}

fn add(
    workspace: &Workspace,
    path: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<String> {
    let path = workspace.existing_inside(path)?;
    let relative = workspace
        .relative(&path)?
        .to_string_lossy()
        .replace('\\', "/");
    ccvl_core::style::validate_path(&relative)?;
    files.insert(relative.clone(), fs::read(path)?);
    Ok(relative)
}

pub fn render(bundle: &Path, record: &Path, profile: &Path, output: &Path) -> Result<()> {
    let bundle: StyleBundle = serde_json::from_slice(&fs::read(bundle)?)?;
    let project: StyleDocument = bundle.with_record(
        crate::workspace::read_toml_value(record)?,
        crate::workspace::read_toml_value(profile)?,
    )?;
    let renderer = ccvl_core::render::StyleRenderer::new(&project.bundle)?;
    let document = renderer.compile(&project)?;
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, renderer.pdf(&document)?)?;
    Ok(())
}

#[cfg(test)]
mod tests;
