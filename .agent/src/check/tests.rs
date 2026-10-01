use super::*;

#[test]
fn checked_artifacts_reject_existing_directories_and_failed_checks_publish_nothing() {
    let temporary = TempDir::new().unwrap();
    fs::write(temporary.path().join("ccvl.json"), "{}").unwrap();
    let workspace = Workspace::at(temporary.path()).unwrap();
    let artifacts = temporary.path().join("artifacts");
    let error = run_with_artifacts(&workspace, Some(&artifacts)).unwrap_err();
    assert!(error.to_string().contains("unsupported workspace"));
    assert!(!artifacts.exists());

    fs::create_dir(&artifacts).unwrap();
    fs::write(artifacts.join("existing.pdf"), "previous output").unwrap();
    let error = run_with_artifacts(&workspace, Some(&artifacts)).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("artifact directory already exists")
    );
    assert_eq!(
        fs::read_to_string(artifacts.join("existing.pdf")).unwrap(),
        "previous output"
    );
}

#[test]
fn correspondence_copies_reject_local_edits_and_unrecorded_sources() {
    let temporary = TempDir::new().unwrap();
    fs::write(temporary.path().join("ccvl.json"), "{}").unwrap();
    let root = temporary.path().join(".agent/typst/letter");
    fs::create_dir_all(&root).unwrap();
    let source = b"#let greeting = \"Hello\"\n";
    fs::write(root.join("letter.typ"), source).unwrap();
    fs::write(
        root.join("source.json"),
        serde_json::to_vec(&json!({
            "files": {"letter.typ": format!("{:x}", Sha256::digest(source))}
        }))
        .unwrap(),
    )
    .unwrap();
    let workspace = Workspace::at(temporary.path()).unwrap();
    validate_correspondence(&workspace).unwrap();
    fs::write(root.join("letter.typ"), "#let greeting = \"Changed\"").unwrap();
    assert!(
        validate_correspondence(&workspace)
            .unwrap_err()
            .to_string()
            .contains("update it upstream")
    );
    fs::write(root.join("letter.typ"), source).unwrap();
    fs::write(root.join("private-rule.typ"), "#let rule = true").unwrap();
    assert!(
        validate_correspondence(&workspace)
            .unwrap_err()
            .to_string()
            .contains("unrecorded correspondence source")
    );
}

#[test]
fn manifest_accepts_ci_metadata_and_rejects_legacy_roots() {
    let (temporary, workspace) = crate::test_support::fixture_workspace();
    validate_manifest(&workspace).unwrap();
    for relative in [".ci/ccid.toml", ".crow/ccid.yaml"] {
        let path = temporary.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "CI metadata fixture").unwrap();
    }
    validate_manifest(&workspace).unwrap();
    let legacy = temporary.path().join(".claude");
    fs::create_dir(&legacy).unwrap();
    fs::write(legacy.join("settings.json"), "{}").unwrap();
    let error = validate_manifest(&workspace).unwrap_err().to_string();
    assert!(error.contains("legacy workspace path must be removed: .claude"));
}

/// Replace one TOML value at a JSON-pointer-like path, returning the original text.
fn weaken(path: &Path, pointer: &str, weakened: toml::Value) -> String {
    let text = fs::read_to_string(path).unwrap();
    let mut contract: toml::Value = toml::from_str(&text).unwrap();
    let target =
        pointer
            .split('/')
            .filter(|part| !part.is_empty())
            .fold(&mut contract, |value, part| {
                value
                    .as_table_mut()
                    .expect("contract section")
                    .get_mut(part)
                    .expect("contract key")
            });
    *target = weakened;
    fs::write(path, toml::to_string(&contract).unwrap()).unwrap();
    text
}

#[test]
fn cover_letter_contract_rejects_weakened_density_and_closing_spill() {
    let (temporary, workspace) = crate::test_support::fixture_workspace();
    validate_manifest(&workspace).unwrap();
    validate_styles(&workspace).unwrap();
    validate_frozen_contracts(&workspace).unwrap();
    let contract_path = temporary.path().join("cvl/cl/ledger/contract.toml");
    for (pointer, weakened, message) in [
        ("/line_fill/body/non_final_minimum", 75, "body fill"),
        ("/line_fill/body/minimum", 60, "body fill"),
        ("/line_fill/body/maximum", 102, "body fill"),
        ("/line_fill/highlight/minimum", 60, "highlight fill"),
        ("/highlights/count", 4, "five highlights"),
        ("/body_lines/maximum", 27, "body contract"),
        ("/widow_or_orphan_lines", 1, "widow/orphan"),
    ] {
        let text = weaken(&contract_path, pointer, weakened.into());
        let error = format!("{:#}", validate_frozen_contracts(&workspace).unwrap_err());
        assert!(error.contains(message), "{pointer}: {error}");
        assert!(error.contains("cl/ledger contract"), "{pointer}: {error}");
        fs::write(&contract_path, text).unwrap();
        validate_frozen_contracts(&workspace).unwrap();
    }
}

#[test]
fn cv_contract_and_compact_delta_reject_weakened_budgets() {
    let (temporary, workspace) = crate::test_support::fixture_workspace();
    let contract_path = temporary.path().join("cvl/cv/ledger/contract.toml");
    for (pointer, weakened, message) in [
        ("/summary_lines", toml::Value::from(4), "exactly five lines"),
        ("/summary_fill/minimum", 90.into(), "95/97/100"),
        ("/last_line_maximum", 105.into(), "closing-line maximum"),
        (
            "/layout_contract/page_1/entries/maximum",
            9.into(),
            "page 1 contract",
        ),
        (
            "/layout_contract/verified_only",
            false.into(),
            "evidence or MECE",
        ),
    ] {
        let text = weaken(&contract_path, pointer, weakened);
        let error = format!("{:#}", validate_frozen_contracts(&workspace).unwrap_err());
        assert!(error.contains(message), "{pointer}: {error}");
        assert!(error.contains("cv/ledger contract"), "{pointer}: {error}");
        fs::write(&contract_path, text).unwrap();
    }
    validate_frozen_contracts(&workspace).unwrap();
    let compact = temporary.path().join("cvl/cv/ledger/compact/substyle.toml");
    let original = fs::read_to_string(&compact).unwrap();
    for (delta, message) in [
        (
            "[page]\nmargin_left_mm = 10\n",
            "horizontal section changed by the delta: page",
        ),
        (
            "[text]\nsize_pt = 10\n",
            "horizontal section changed by the delta: text",
        ),
        (
            "[accents]\nlink = \"#000000\"\n",
            "horizontal section changed by the delta: accents",
        ),
        (
            "[cv]\nbullet_indent_pt = 8\nentry_spacing_pt = 10.0\n",
            "bullet_indent_pt",
        ),
        (
            "[cv]\nentry_spacing_pt = 14.0\n",
            "compact must tighten vertical whitespace",
        ),
        (
            "[header]\nafter_pt = 5.5\n",
            "compact must tighten vertical whitespace",
        ),
    ] {
        fs::write(&compact, delta).unwrap();
        let error = format!("{:#}", validate_frozen_contracts(&workspace).unwrap_err());
        assert!(error.contains(message), "{delta}: {error}");
        assert!(
            error.contains("cv/ledger compact substyle"),
            "{delta}: {error}"
        );
    }
    fs::write(&compact, original).unwrap();
    validate_frozen_contracts(&workspace).unwrap();
}

#[test]
fn frozen_contracts_bind_harvard_even_without_its_opt_in_markers() {
    let (temporary, workspace) = crate::test_support::fixture_workspace();
    // Without the markers a style owns its geometry: a three-paragraph letter
    // and a summary-free CV are valid independent designs.
    let letter = temporary.path().join("cvl/cl/ledger/contract.toml");
    let text = fs::read_to_string(&letter).unwrap();
    let independent = text.replace("[editorial]\nstructure = \"aida\"\n", "");
    fs::write(&letter, &independent).unwrap();
    weaken(&letter, "/highlights/count", 3.into());
    let cv = temporary.path().join("cvl/cv/ledger/contract.toml");
    let cv_text = fs::read_to_string(&cv).unwrap();
    let mut table: toml::Table = toml::from_str(&cv_text).unwrap();
    table.remove("layout_contract");
    table.insert("summary_lines".into(), 3.into());
    fs::write(&cv, toml::to_string(&table).unwrap()).unwrap();
    validate_frozen_contracts(&workspace).unwrap();
    // The shipped Harvard style keeps the frozen budgets regardless.
    for (document, contract) in [("cl", &letter), ("cv", &cv)] {
        let harvard = temporary.path().join(format!("cvl/{document}/harvard"));
        fs::create_dir_all(&harvard).unwrap();
        let style = fs::read_to_string(
            temporary
                .path()
                .join(format!("cvl/{document}/ledger/style.toml")),
        )
        .unwrap()
        .replace("id = \"ledger\"", "id = \"harvard\"");
        fs::write(harvard.join("style.toml"), style).unwrap();
        fs::copy(contract, harvard.join("contract.toml")).unwrap();
        let error = format!("{:#}", validate_frozen_contracts(&workspace).unwrap_err());
        assert!(
            error.contains(&format!("{document}/harvard contract")),
            "{error}"
        );
        fs::remove_dir_all(&harvard).unwrap();
    }
}

#[test]
fn style_sources_cannot_vendor_measurement_code() {
    let (temporary, workspace) = crate::test_support::fixture_workspace();
    validate_measurement_ownership(&workspace).unwrap();
    fs::write(temporary.path().join("cvl/cv/grid/measure-v1.typ"), "").unwrap();
    let error = validate_measurement_ownership(&workspace)
        .unwrap_err()
        .to_string();
    assert!(error.contains("vendors the shared program"), "{error}");
}

fn fixture_leaf(workspace: &Workspace, document: &'static str, style: &str) -> styles::StyleLeaf {
    let selected = styles::selection(workspace, document, Some(style), None).unwrap();
    styles::leaf(workspace, document, "en-ch", &selected).unwrap()
}

#[test]
fn documents_must_not_read_the_opposite_document_tree() {
    let (temporary, workspace) = crate::test_support::fixture_workspace();
    let compiler = Compiler::new(&workspace).unwrap();
    let leaf = fixture_leaf(&workspace, "cv", "ledger");
    let build = || {
        let observed = workspace.tracked();
        let spec = crate::render::cvl_spec(&observed, &leaf, 2).unwrap();
        compiler.compile(&observed, &spec).unwrap();
        require_document_isolation(&workspace, &observed, &leaf)
    };
    build().unwrap();
    let renderer = temporary.path().join("cvl/shared/ledger/render.typ");
    let text = fs::read_to_string(&renderer).unwrap();
    fs::write(
        &renderer,
        text.replace(
            "  let contract = toml(\"/cvl/cv/ledger/contract.toml\")\n",
            "  let contract = toml(\"/cvl/cv/ledger/contract.toml\")\n  let letter = letter-contract()\n",
        ),
    )
    .unwrap();
    let error = build().unwrap_err().to_string();
    assert!(
        error.contains(
            "cv ledger/primary/en-ch reads the cl document tree: cvl/cl/ledger/contract.toml"
        ),
        "{error}"
    );
}

#[test]
fn resolved_copies_and_portable_bundles_must_reproduce_the_build() {
    let (temporary, workspace) = crate::test_support::fixture_workspace();
    // Exports carry the real license metadata and texts.
    crate::test_support::copy_repository(temporary.path(), "REUSE.toml");
    crate::test_support::copy_repository(temporary.path(), "LICENSES");
    let compiler = Compiler::new(&workspace).unwrap();
    let verify = |document: &'static str, style: &str| {
        let leaf = fixture_leaf(&workspace, document, style);
        let mut spec = crate::render::cvl_spec(&workspace, &leaf, leaf.default_pages).unwrap();
        spec.output = temporary.path().join(format!("{document}-{style}.pdf"));
        let pdf = compiler.render(&workspace, &spec).unwrap();
        require_portable_copies(&workspace, &compiler, &leaf, &spec, &pdf)
    };
    for (document, style) in [("cv", "ledger"), ("cl", "ledger"), ("cv", "grid")] {
        verify(document, style).unwrap();
    }
    // An input without a literal default cannot compile as an emitted copy.
    let entry = temporary.path().join("cvl/cv/grid/wide/en/ch/typst/cv.typ");
    let original = fs::read_to_string(&entry).unwrap();
    fs::write(
        &entry,
        original.replace(
            "sys.inputs.at(\"strings\", default: \"/cvl/cv/grid/wide/en/ch/strings.toml\")",
            "sys.inputs.at(\"strings\")",
        ),
    )
    .unwrap();
    let error = format!("{:#}", verify("cv", "grid").unwrap_err());
    assert!(error.contains("resolved copy"), "{error}");
    fs::write(&entry, original).unwrap();
    // A renderer input outside the exported style breaks the portable bundle.
    let layout = temporary.path().join("cvl/cv/grid/layout.typ");
    let text = fs::read_to_string(&layout).unwrap();
    fs::write(
        &layout,
        format!("#import \"/cvl/shared/ledger/render.typ\": contact\n{text}"),
    )
    .unwrap();
    let error = format!("{:#}", verify("cv", "grid").unwrap_err());
    assert!(error.contains("portable bundle"), "{error}");
}

#[test]
fn portable_bundle_is_an_explicit_boolean_without_source_files() {
    assert!(!portable_bundle(&json!({})).unwrap());
    assert!(portable_bundle(&json!({"portable_bundle": true})).unwrap());
    assert!(portable_bundle(&json!({"portable_bundle": "yes"})).is_err());
    assert!(
        portable_bundle(&json!({"portable_bundle": true, "source_files": ["src/a.typ"]}))
            .unwrap_err()
            .to_string()
            .contains("source_files")
    );
    let (temporary, workspace) = crate::test_support::fixture_workspace();
    let contract = temporary.path().join("cvl/cv/grid/contract.toml");
    let text = fs::read_to_string(&contract).unwrap();
    fs::write(
        &contract,
        text.replace("portable_bundle = true", "portable_bundle = 1"),
    )
    .unwrap();
    let error = format!("{:#}", validate_styles(&workspace).unwrap_err());
    assert!(
        error.contains("cv/grid contract: portable_bundle must be a boolean"),
        "{error}"
    );
}

#[test]
fn fixture_workspace_passes_the_full_check() {
    let (_temporary, workspace) = crate::test_support::checkable_fixture_workspace();
    crate::render::render_cvl(&workspace).unwrap();
    run(&workspace).unwrap();
}
