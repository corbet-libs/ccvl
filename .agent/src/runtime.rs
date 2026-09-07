use std::path::Path;

use anyhow::{Context, Result, ensure};

pub const ID: &str = env!("CCVL_RUNTIME_ID");

pub fn verify(root: &Path) -> Result<()> {
    let expected = crate::runtime_source::fingerprint(root)
        .context("cannot identify this workspace's runtime source")?;
    ensure!(
        expected == ID,
        "this ccvl binary does not match the workspace runtime; run the workspace setup command to install its matching precompiled binary"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn built_identity_matches_source_and_rejects_runtime_edits() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        verify(root).unwrap();
        let directory = tempfile::tempdir().unwrap();
        for relative in crate::runtime_source::inputs(root).unwrap() {
            let destination = directory.path().join(&relative);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(root.join(relative), destination).unwrap();
        }
        verify(directory.path()).unwrap();
        fs::create_dir_all(directory.path().join("cvl")).unwrap();
        fs::write(directory.path().join("cvl/profile.toml"), "personal edit").unwrap();
        verify(directory.path()).unwrap();
        fs::write(directory.path().join(".agent/src/new.rs"), "// new source\n").unwrap();
        assert!(verify(directory.path()).is_err());
    }
}
