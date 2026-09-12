//! Platform-independent identity of everything compiled into the ccvl runtime.
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;

pub fn inputs(root: &Path) -> io::Result<Vec<String>> {
    let mut sources = Vec::new();
    collect(root, &root.join(".agent/src"), &mut sources)?;
    collect(root, &root.join(".agent/core/src"), &mut sources)?;
    sources.sort();
    let mut paths = [
        "Cargo.toml",
        ".agent/core/Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".agent/build.rs",
    ]
    .map(str::to_owned)
    .to_vec();
    paths.extend(sources);
    Ok(paths)
}

fn collect(root: &Path, directory: &Path, paths: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect(root, &path, paths)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(
                path.strip_prefix(root)
                    .expect("input is below root")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    Ok(())
}

pub fn fingerprint(root: &Path) -> io::Result<String> {
    let mut hash = Sha256::new();
    for relative in inputs(root)? {
        let digest = Sha256::digest(fs::read(root.join(&relative))?);
        hash.update(format!("{relative} {digest:x}\n"));
    }
    Ok(format!("{:x}", hash.finalize()))
}
