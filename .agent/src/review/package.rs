use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

use anyhow::{Result, ensure};

use super::{
    render, storage,
    types::{Check, Manifest, PurposeKind, SourceRole, Spec},
};
use crate::Workspace;

pub(super) fn validate_spec(workspace: &Workspace, spec: &Spec) -> Result<()> {
    ensure!(
        !spec.purpose.description.trim().is_empty(),
        "declare a real document purpose"
    );
    ensure!(
        !spec.actor_id.trim().is_empty() && !spec.model.trim().is_empty(),
        "actor_id and critic model are required"
    );
    ensure!(
        spec.max_input_tokens > 0 && spec.max_output_tokens > 0 && spec.time_limit_seconds > 0,
        "declare positive token and time limits for the external critic"
    );
    ensure!(
        !spec.documents.is_empty() && spec.documents.len() <= 16,
        "review requires 1–16 selected documents"
    );
    ensure!(!spec.rubric.is_empty(), "pin at least one review rubric");
    let mut ids = BTreeSet::new();
    for document in &spec.documents {
        ensure!(
            valid_id(&document.id) && ids.insert(&document.id),
            "document IDs must be unique lowercase ASCII keys"
        );
        ensure!(
            (1..=32).contains(&document.pages),
            "document page count outside 1–32"
        );
    }
    ids.clear();
    for source in &spec.sources {
        ensure!(
            valid_id(&source.id) && ids.insert(&source.id),
            "source IDs must be unique lowercase ASCII keys"
        );
        normal_path(&source.path)?;
        if workspace.path(&source.path).exists() {
            workspace.existing_inside(&source.path)?;
        }
    }
    for path in &spec.rubric {
        workspace.existing_inside(path)?;
    }
    let evidence = spec
        .sources
        .iter()
        .map(|source| workspace.path(&source.path))
        .collect::<BTreeSet<_>>();
    for path in &spec.editable {
        normal_path(path)?;
        let path = workspace.existing_inside(path)?;
        let relative = storage::relative(workspace, &path)?;
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        ensure!(
            (relative.starts_with("cvl/") && ["content.toml", "wording.toml"].contains(&name))
                || (relative.starts_with("opportunities/") && name == "application.toml"),
            "editable paths must be canonical document content or wording: {relative}"
        );
        ensure!(
            !evidence.contains(&path),
            "evidence is never an actor-editable path"
        );
        ensure!(
            !spec.rubric.iter().any(|rule| workspace.path(rule) == path),
            "review criteria are never actor-editable"
        );
    }
    Ok(())
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
}

fn normal_path(path: &Path) -> Result<()> {
    ensure!(
        !path.as_os_str().is_empty()
            && path
                .components()
                .all(|part| matches!(part, Component::Normal(_))),
        "input paths must be workspace-relative without parent traversal"
    );
    Ok(())
}

pub(super) fn dependencies(workspace: &Workspace) -> Result<BTreeMap<String, Option<String>>> {
    workspace
        .observed_inputs()
        .into_iter()
        .filter(|path| !path.is_dir())
        .map(|path| {
            let name = storage::relative(workspace, &path)?;
            let hash = if path.is_file() {
                workspace.existing_inside(&path)?;
                Some(storage::digest(&fs::read(path)?))
            } else {
                None
            };
            Ok((name, hash))
        })
        .collect()
}

pub(super) fn build(
    workspace: &Workspace,
    run: &Path,
    revision: u32,
    spec: &Spec,
    parent: Option<(&Manifest, &str)>,
) -> Result<Manifest> {
    validate_spec(workspace, spec)?;
    let workspace = workspace.tracked();
    for path in crate::runtime_source::inputs(workspace.root())? {
        workspace.observe_input(path);
    }
    let directory = storage::revision_path(run, revision);
    preserve_interrupted(&directory)?;
    fs::create_dir(&directory)?;
    fs::create_dir(directory.join("objects"))?;
    let mut manifest = Manifest {
        schema_version: storage::SCHEMA, revision, parent_manifest: parent.map(|(_, hash)| hash.to_owned()),
        parent_result: if revision > 0 { fs::read(storage::revision_path(run, revision - 1).join("result.json")).ok().map(|bytes| storage::digest(&bytes)) } else { None },
        runtime: crate::runtime::ID.to_owned(), source_identity: crate::runtime_source::fingerprint(workspace.root())?,
        created_at: storage::now()?, spec: spec.clone(), artifacts: Vec::new(), dependencies: BTreeMap::new(),
        checks: Vec::new(), documents: Vec::new(), changed_artifacts: Vec::new(),
        limitations: vec!["The external coordinator must launch a fresh critic. Agent identity, fresh context, model and token usage are reviewer attestations; ccvl does not invoke or monitor a provider.".into(), "Snapshots are hash-verified and marked read-only; same-user filesystem access is not OS isolation. Compiler time cannot be preempted by this protocol; the declared deadline rejects late critic submissions.".into()],
    };
    for source in &spec.sources {
        workspace.observe_input(&source.path);
        let available = workspace
            .existing_inside(&source.path)
            .and_then(|path| Ok(fs::read(path)?));
        match available {
            Ok(bytes) => {
                manifest.artifacts.push(storage::freeze(
                    run,
                    revision,
                    &format!("source:{}", source.id),
                    if bytes.starts_with(b"%PDF-") {
                        "pdf"
                    } else {
                        "evidence"
                    },
                    &bytes,
                    Some(source.path.to_string_lossy().replace('\\', "/")),
                    None,
                    None,
                )?);
                if bytes.starts_with(b"%PDF-") {
                    let extraction = lopdf::Document::load_mem(&bytes).and_then(|document| {
                        document
                            .extract_text(&document.get_pages().keys().copied().collect::<Vec<_>>())
                    });
                    match extraction {
                        Ok(text) if !text.trim().is_empty() => manifest.artifacts.push(storage::freeze(run, revision, &format!("source:{}:text", source.id), "evidence", text.as_bytes(), None, None, None)?),
                        _ => manifest.checks.push(Check { document: source.id.clone(), operation: "evidence".into(), passed: false, diagnostics: vec!["source PDF needs an attributable transcription or visual evidence access".into()] }),
                    }
                }
            }
            Err(error) => manifest.checks.push(Check {
                document: source.id.clone(),
                operation: "evidence".into(),
                passed: false,
                diagnostics: vec![format!("source unavailable: {error:#}")],
            }),
        }
    }
    let candidate = spec.sources.iter().any(|source| {
        matches!(
            source.role,
            SourceRole::Candidate | SourceRole::Confirmation
        )
    });
    let target = spec.purpose.kind == PurposeKind::General
        || [SourceRole::Posting, SourceRole::Research]
            .iter()
            .all(|role| spec.sources.iter().any(|source| &source.role == role));
    if !candidate || !target {
        manifest.checks.push(Check { document: "sources".into(), operation: "evidence".into(), passed: false, diagnostics: vec!["candidate evidence is required; targeted purposes also require an archived posting and attributable research".into()] });
    }
    for (index, rule) in spec.rubric.iter().enumerate() {
        let path = workspace.existing_inside(rule)?;
        manifest.artifacts.push(storage::freeze(
            run,
            revision,
            &format!("rule:{index}"),
            "rule",
            &fs::read(&path)?,
            Some(storage::relative(&workspace, &path)?),
            None,
            None,
        )?);
    }
    // Discover actual compiler reads, then pin them before the final compilation.
    // A changing/new dependency in the second pass invalidates this candidate.
    for request in &spec.documents {
        if let Ok((mut resolved, _)) =
            render::resolve(&workspace, request, &directory.join("discovery.pdf"))
        {
            resolved
                .inputs
                .insert("line-contracts".into(), "report".into());
            if let Ok(compiler) = crate::render::Compiler::new(&workspace) {
                let _ = compiler.compile(&workspace, &resolved);
            }
        }
    }
    let before = dependencies(&workspace)?;
    for request in &spec.documents {
        match render::capture(
            &workspace,
            run,
            revision,
            request,
            &mut manifest.artifacts,
            &mut manifest.checks,
        ) {
            Ok(value) => manifest.documents.push(value),
            Err(error) => manifest.checks.push(Check {
                document: request.id.clone(),
                operation: "render".into(),
                passed: false,
                diagnostics: vec![format!("{error:#}")],
            }),
        }
    }
    manifest.dependencies = dependencies(&workspace)?;
    if before != manifest.dependencies {
        manifest.checks.push(Check { document: "inputs".into(), operation: "input-stability".into(), passed: false, diagnostics: vec!["inputs changed or new dependencies appeared during rendering; prepare a new candidate".into()] });
    }
    for (path, hash) in &manifest.dependencies {
        if let Some(hash) = hash {
            let bytes = fs::read(workspace.existing_inside(path)?)?;
            ensure!(
                &storage::digest(&bytes) == hash,
                "input changed while freezing: {path}"
            );
            manifest.artifacts.push(storage::freeze(
                run,
                revision,
                &format!("input:{path}"),
                "input",
                &bytes,
                Some(path.clone()),
                None,
                None,
            )?);
        }
    }
    if let Some((previous, _)) = parent {
        manifest.changed_artifacts = manifest
            .artifacts
            .iter()
            .filter(|artifact| {
                artifact.role != "input"
                    && previous
                        .artifacts
                        .iter()
                        .find(|old| old.id == artifact.id)
                        .is_none_or(|old| old.sha256 != artifact.sha256)
            })
            .map(|artifact| artifact.id.clone())
            .collect();
    }
    Ok(manifest)
}

fn preserve_interrupted(directory: &Path) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }
    ensure!(
        !fs::symlink_metadata(directory)?.file_type().is_symlink(),
        "revision directory cannot be a symlink"
    );
    ensure!(
        !directory.join("manifest.json").exists(),
        "revision already has a completed manifest; do not overwrite it"
    );
    let name = directory.file_name().unwrap().to_string_lossy();
    let parent = directory.parent().unwrap();
    let mut sequence = 0_u64;
    loop {
        let preserved = parent.join(format!("interrupted-{name}-{sequence}"));
        if !preserved.exists() {
            fs::rename(directory, &preserved)?;
            break;
        }
        sequence += 1;
    }
    Ok(())
}
