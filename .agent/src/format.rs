use std::fs;

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

use crate::workspace::Workspace;

pub fn format_typst(workspace: &Workspace, check: bool) -> Result<()> {
    let mut changed = Vec::new();
    for root in ["cvl", ".agent/typst"] {
        for entry in WalkDir::new(workspace.path(root)) {
            let entry = entry?;
            if !entry.file_type().is_file()
                || entry
                    .path()
                    .starts_with(workspace.path(".agent/typst/letter"))
                || entry
                    .path()
                    .extension()
                    .is_none_or(|extension| extension != "typ")
            {
                continue;
            }
            let original = fs::read_to_string(entry.path())?;
            let relative = workspace.relative(entry.path())?;
            let rendered = ctypst::format_source(&original, 120)
                .with_context(|| format!("cannot format {}", relative.display()))?;
            if rendered != original {
                changed.push(workspace.relative(entry.path())?);
                if !check {
                    fs::write(entry.path(), rendered)?;
                }
            }
        }
    }
    if check && !changed.is_empty() {
        bail!(
            "Typst sources require formatting: {}",
            changed
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !check {
        println!("Formatted {} Typst source(s).", changed.len());
    }
    Ok(())
}
