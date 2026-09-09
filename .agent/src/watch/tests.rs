use super::*;
use crate::render::{Compiler, DocumentKind, DocumentSpec};
use crate::styles::Selection;

fn fixture() -> (tempfile::TempDir, Workspace, DocumentSpec) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::write(root.join("ccvl.json"), "{}\n").unwrap();
    fs::write(root.join("style.json"), "{}\n").unwrap();
    fs::write(
        root.join("main.typ"),
        "#set text(font: \"Archivo\")\n#read(toml(\"settings.toml\").wording)",
    )
    .unwrap();
    fs::write(root.join("settings.toml"), "wording = \"wording.md\"\n").unwrap();
    fs::write(root.join("wording.md"), "First wording").unwrap();
    let workspace = Workspace::at(root).unwrap();
    let spec = DocumentSpec {
        name: "watched document".to_owned(),
        kind: DocumentKind::Cv,
        source: workspace.path("main.typ"),
        output: workspace.path("output.pdf"),
        inputs: std::collections::BTreeMap::default(),
        expected_pages: 1,
        selection: Selection {
            style: "independent".to_owned(),
            substyle: "standard".to_owned(),
        },
        contract: serde_json::json!({}),
        fonts: Vec::new(),
    };
    (directory, workspace, spec)
}

fn build(workspace: &Workspace, spec: &DocumentSpec) -> Result<Vec<PathBuf>> {
    workspace.read_json("style.json")?;
    Ok(vec![Compiler::new(workspace)?.render(workspace, spec)?])
}

fn settle(state: &mut State, workspace: &Workspace, spec: &DocumentSpec) {
    for _ in 0..4 {
        match state.poll(workspace, |workspace| build(workspace, spec)) {
            Some(result) => {
                result.unwrap();
            }
            None => return,
        }
    }
    panic!("watch inputs did not settle");
}

#[test]
fn actual_reads_trigger_while_unrelated_styles_and_generated_outputs_do_not() {
    let (_directory, workspace, spec) = fixture();
    let mut state = State::new(&workspace);
    settle(&mut state, &workspace, &spec);
    assert!(state.inputs.contains(&workspace.path("wording.md")));
    for path in ["unrelated-style.toml", "preview.png", "output.pdf"] {
        fs::write(workspace.path(path), "unrelated change").unwrap();
        assert!(
            state
                .poll(&workspace, |workspace| build(workspace, &spec))
                .is_none()
        );
    }
    fs::write(workspace.path("wording.md"), "Changed actual input").unwrap();
    state
        .poll(&workspace, |workspace| build(workspace, &spec))
        .unwrap()
        .unwrap();
    assert!(fs::read(&spec.output).unwrap().starts_with(b"%PDF-"));
    assert!(
        state
            .poll(&workspace, |workspace| build(workspace, &spec))
            .is_none()
    );
    fs::write(workspace.path("style.json"), "{\"changed\": true}").unwrap();
    state
        .poll(&workspace, |workspace| build(workspace, &spec))
        .unwrap()
        .unwrap();
}

#[test]
fn dependency_changes_drop_old_reads_after_success_and_recover_missing_inputs() {
    let (_directory, workspace, spec) = fixture();
    let mut state = State::new(&workspace);
    settle(&mut state, &workspace, &spec);
    fs::write(
        workspace.path("settings.toml"),
        "wording = \"new/wording.yaml\"\n",
    )
    .unwrap();
    assert!(
        state
            .poll(&workspace, |workspace| build(workspace, &spec))
            .unwrap()
            .is_err()
    );
    assert!(state.inputs.contains(&workspace.path("new/wording.yaml")));
    assert!(state.inputs.contains(&workspace.path("wording.md")));
    assert!(
        state
            .poll(&workspace, |workspace| build(workspace, &spec))
            .unwrap()
            .is_err()
    );
    assert!(
        state
            .poll(&workspace, |workspace| build(workspace, &spec))
            .is_none()
    );
    fs::create_dir(workspace.path("new")).unwrap();
    fs::write(workspace.path("new/wording.yaml"), "Recovered").unwrap();
    settle(&mut state, &workspace, &spec);
    assert!(!state.inputs.contains(&workspace.path("wording.md")));
    fs::write(workspace.path("wording.md"), "No longer used").unwrap();
    assert!(
        state
            .poll(&workspace, |workspace| build(workspace, &spec))
            .is_none()
    );
    fs::write(workspace.path("style.json"), "invalid JSON").unwrap();
    assert!(
        state
            .poll(&workspace, |workspace| build(workspace, &spec))
            .unwrap()
            .is_err()
    );
    fs::write(workspace.path("style.json"), "{}").unwrap();
    settle(&mut state, &workspace, &spec);
}

#[test]
fn declared_font_inputs_are_watched_even_when_engine_initialization_fails() {
    let (_directory, workspace, mut spec) = fixture();
    let font = workspace.path("extra.ttf");
    fs::write(&font, ctypst::fonts::archivo()[0]).unwrap();
    spec.fonts = vec![font.clone()];
    let mut state = State::new(&workspace);
    settle(&mut state, &workspace, &spec);
    assert!(state.inputs.contains(&font));
    fs::write(&font, "not a font").unwrap();
    assert!(
        state
            .poll(&workspace, |workspace| build(workspace, &spec))
            .unwrap()
            .is_err()
    );
    fs::write(&font, ctypst::fonts::archivo()[0]).unwrap();
    settle(&mut state, &workspace, &spec);
}

#[test]
fn optional_inputs_and_edits_during_a_build_trigger_a_settling_build() {
    let (_directory, workspace, _spec) = fixture();
    let mut state = State::new(&workspace);
    let build = |workspace: &Workspace| -> Result<Vec<PathBuf>> {
        workspace.read_text("wording.md")?;
        workspace.input_is_file("optional.toml");
        Ok(Vec::new())
    };
    state.poll(&workspace, build).unwrap().unwrap();
    state.poll(&workspace, build).unwrap().unwrap();
    assert!(state.poll(&workspace, build).is_none());
    fs::write(workspace.path("optional.toml"), "created = true").unwrap();
    state
        .poll(&workspace, |workspace| {
            build(workspace)?;
            fs::write(workspace.path("wording.md"), "Changed while building")?;
            Ok(Vec::new())
        })
        .unwrap()
        .unwrap();
    assert!(state.previous.is_none());
    state.poll(&workspace, build).unwrap().unwrap();
    assert!(state.poll(&workspace, build).is_none());
}

#[cfg(unix)]
#[test]
fn replacing_a_symlink_is_detected_even_with_identical_contents() {
    use std::os::unix::fs::symlink;
    let (_directory, workspace, spec) = fixture();
    fs::rename(workspace.path("wording.md"), workspace.path("original.md")).unwrap();
    fs::copy(
        workspace.path("original.md"),
        workspace.path("replacement.md"),
    )
    .unwrap();
    symlink("original.md", workspace.path("wording.md")).unwrap();
    let mut state = State::new(&workspace);
    settle(&mut state, &workspace, &spec);
    fs::remove_file(workspace.path("wording.md")).unwrap();
    symlink("replacement.md", workspace.path("wording.md")).unwrap();
    state
        .poll(&workspace, |workspace| build(workspace, &spec))
        .unwrap()
        .unwrap();
    settle(&mut state, &workspace, &spec);
    assert!(!state.inputs.contains(&workspace.path("original.md")));
    fs::write(workspace.path("original.md"), "Unused target").unwrap();
    assert!(
        state
            .poll(&workspace, |workspace| build(workspace, &spec))
            .is_none()
    );
}

#[test]
fn malformed_opportunity_keys_are_rejected_before_watching() {
    let (_directory, workspace, _spec) = fixture();
    assert!(crate::opportunity::record_path(&workspace, "../acme", "lead", false).is_err());
    assert!(crate::opportunity::record_path(&workspace, "acme", "lead", false).is_ok());
}
