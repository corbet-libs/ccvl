use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, ensure};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

use super::types::{Artifact, Manifest, RunState};
use crate::Workspace;

pub(super) const SCHEMA: u32 = 1;
pub(super) const CORRECTIONS: u32 = 2;
pub(super) const MAX_JSON: u64 = 8 * 1024 * 1024;

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

pub(super) fn read<T: DeserializeOwned>(path: &Path) -> Result<T> {
    ensure!(
        fs::metadata(path)?.len() <= MAX_JSON,
        "JSON exceeds 8 MiB: {}",
        path.display()
    );
    serde_json::from_slice(&fs::read(path)?).with_context(|| format!("invalid {}", path.display()))
}

pub(super) fn write_new(path: &Path, value: &impl Serialize) -> Result<String> {
    let bytes = serde_json::to_vec_pretty(value)?;
    let mut file = tempfile::Builder::new()
        .prefix(".review-tmp-")
        .tempfile_in(path.parent().context("review record has no parent")?)?;
    file.write_all(&bytes)?;
    file.as_file().sync_all()?;
    file.persist_noclobber(path)?;
    Ok(digest(&bytes))
}

pub(super) fn save_state(run: &Path, state: &RunState) -> Result<()> {
    let mut temporary = tempfile::NamedTempFile::new_in(run)?;
    temporary.write_all(&serde_json::to_vec_pretty(state)?)?;
    temporary.as_file().sync_all()?;
    temporary.persist(run.join("state.json"))?;
    Ok(())
}

pub(super) fn relative(workspace: &Workspace, path: &Path) -> Result<String> {
    Ok(workspace
        .relative(path)?
        .to_string_lossy()
        .replace('\\', "/"))
}

pub(super) fn run_path(workspace: &Workspace, path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        workspace.relative(path)?
    } else {
        path.to_path_buf()
    };
    ensure!(
        path.components()
            .all(|part| matches!(part, Component::Normal(_))),
        "review path must be a normal relative path"
    );
    let parts = path
        .components()
        .map(|p| p.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let general = parts.len() == 4 && parts[..3] == [".agent", "cache", "review"];
    let targeted = parts.len() == 5 && parts[0] == "opportunities" && parts[3] == "review";
    ensure!(
        general || targeted,
        "review run must be .agent/cache/review/<id> or opportunities/<organisation>/<position>/review/<id>"
    );
    let absolute = workspace.path(path);
    let mut current = workspace.root().to_path_buf();
    for part in parts {
        current.push(part);
        if current.exists() {
            ensure!(
                !fs::symlink_metadata(&current)?.file_type().is_symlink(),
                "review directories cannot be symlinks"
            );
            workspace.existing_inside(&current)?;
        }
    }
    Ok(absolute)
}

pub(super) fn revision_path(run: &Path, revision: u32) -> PathBuf {
    run.join(format!("revision-{revision}"))
}

pub(super) fn load(run: &Path) -> Result<(RunState, Manifest)> {
    let state: RunState = read(&run.join("state.json"))?;
    ensure!(
        state.schema_version == SCHEMA && state.corrections_used <= CORRECTIONS,
        "unsupported or corrupt review state"
    );
    let path = revision_path(run, state.revision).join("manifest.json");
    let bytes = fs::read(&path)?;
    ensure!(
        digest(&bytes) == state.manifest_sha256,
        "review manifest changed"
    );
    let manifest: Manifest = read(&path)?;
    ensure!(
        manifest.schema_version == SCHEMA && manifest.revision == state.revision,
        "review revision mismatch"
    );
    Ok((state, manifest))
}

pub(super) fn artifact_path(run: &Path, artifact: &Artifact) -> Result<PathBuf> {
    let path = Path::new(&artifact.snapshot);
    ensure!(
        path.components()
            .all(|part| matches!(part, Component::Normal(_))),
        "invalid snapshot path"
    );
    let path = run.join(path).canonicalize()?;
    ensure!(
        path.starts_with(run.canonicalize()?),
        "snapshot escapes run directory"
    );
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn freeze(
    run: &Path,
    revision: u32,
    id: &str,
    role: &str,
    bytes: &[u8],
    original: Option<String>,
    document: Option<String>,
    page: Option<usize>,
) -> Result<Artifact> {
    let sha256 = digest(bytes);
    let extension = match role {
        "page" => "png",
        "pdf" => "pdf",
        _ => "txt",
    };
    let snapshot = format!("revision-{revision}/objects/{sha256}.{extension}");
    let path = run.join(&snapshot);
    if path.exists() {
        ensure!(
            digest(&fs::read(&path)?) == sha256,
            "snapshot hash collision"
        );
    } else {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        let mut permissions = file.metadata()?.permissions();
        permissions.set_readonly(true);
        file.set_permissions(permissions)?;
    }
    Ok(Artifact {
        id: id.to_owned(),
        role: role.to_owned(),
        sha256,
        snapshot,
        original,
        document,
        page,
    })
}

pub(super) struct Lock(PathBuf);
impl Lock {
    pub(super) fn acquire(run: &Path) -> Result<Self> {
        let path = run.join(".lock");
        let mut file = OpenOptions::new().write(true).create_new(true).open(&path)
            .context("review run is locked; if its process was interrupted, inspect .lock and remove it only after confirming that process stopped")?;
        writeln!(file, "{}", std::process::id())?;
        Ok(Self(path))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
