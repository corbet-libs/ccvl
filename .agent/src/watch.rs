//! Rebuild from observed inputs, without prescribing a style's source layout.
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::workspace::Workspace;

pub fn run(
    workspace: &Workspace,
    label: &str,
    build: impl Fn(&Workspace) -> Result<Vec<PathBuf>>,
) -> Result<()> {
    println!("Watching inputs for {label}. Press Ctrl-C to stop.");
    let mut state = State::new(workspace);
    loop {
        if let Some(result) = state.poll(workspace, &build) {
            match result {
                Ok(outputs) if state.previous.is_some() => {
                    for output in outputs {
                        println!("Rendered {}", output.display());
                    }
                }
                Ok(_) => {}
                Err(error) => eprintln!("{error:#}\nWaiting for an input change."),
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}

struct State {
    inputs: BTreeSet<PathBuf>,
    previous: Option<Vec<u8>>,
}

impl State {
    fn new(workspace: &Workspace) -> Self {
        Self {
            inputs: BTreeSet::from([workspace.path("ccvl.json")]),
            previous: None,
        }
    }

    fn poll(
        &mut self,
        workspace: &Workspace,
        build: impl FnOnce(&Workspace) -> Result<Vec<PathBuf>>,
    ) -> Option<Result<Vec<PathBuf>>> {
        let before = digest(workspace, &self.inputs);
        if self.previous.as_ref() == Some(&before) {
            return None;
        }
        let observed = workspace.tracked();
        observed.observe_input("ccvl.json");
        let result = build(&observed);
        let mut inputs = observed.observed_inputs();
        if let Ok(outputs) = &result {
            for output in outputs {
                inputs.remove(output);
                if let Ok(path) = output.canonicalize() {
                    inputs.remove(&path);
                }
            }
        } else {
            // A partial failed compile must retain enough of the last working
            // dependency graph to recover, plus any newly attempted inputs.
            inputs.extend(self.inputs.iter().cloned());
        }
        // Recheck after dependency discovery or a concurrent edit. Once the set
        // is known, comparing before/after prevents edits during compilation
        // from being mistaken for already-rendered content.
        let unsettled =
            !inputs.is_subset(&self.inputs) || before != digest(workspace, &self.inputs);
        self.previous = if unsettled {
            None
        } else {
            Some(digest(workspace, &inputs))
        };
        self.inputs = inputs;
        Some(result)
    }
}

fn digest(workspace: &Workspace, inputs: &BTreeSet<PathBuf>) -> Vec<u8> {
    let mut hash = Sha256::new();
    for path in inputs {
        hash.update(path.to_string_lossy().as_bytes());
        hash.update([0]);
        match path.canonicalize() {
            Ok(resolved) => {
                hash.update(resolved.to_string_lossy().as_bytes());
                hash.update([0]);
                if !resolved.starts_with(workspace.root()) {
                    // A changed symlink can become unsafe after a valid build.
                    // Detect it, but let the normal renderer report the error.
                    hash.update(b"outside workspace");
                } else if resolved.is_dir() {
                    // Directory identity matters; unrelated entries and their
                    // modification times do not affect this document.
                    hash.update(b"directory");
                } else if resolved.is_file() {
                    match fs::read(&resolved) {
                        Ok(bytes) => {
                            hash.update(b"file");
                            hash.update(bytes);
                        }
                        Err(error) => {
                            hash.update(format!("read error: {:?}", error.kind()));
                        }
                    }
                } else {
                    hash.update(b"not a regular file");
                }
            }
            Err(error) => hash.update(format!("resolve error: {:?}", error.kind())),
        }
        hash.update([0]);
    }
    hash.finalize().to_vec()
}

#[cfg(test)]
mod tests;
