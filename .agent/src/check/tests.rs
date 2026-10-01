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
fn checked_in_manifest_styles_and_contracts_are_fixed() {
    let workspace = Workspace::discover(None).unwrap();
    validate_manifest(&workspace).unwrap();
    validate_correspondence(&workspace).unwrap();
    validate_styles(&workspace).unwrap();
    validate_contracts(&workspace).unwrap();
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

fn harvard_manifest_fixture() -> (TempDir, Workspace) {
    let repository = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let temporary = TempDir::new().unwrap();
    // Style validation only needs these references to exist, with real
    // registry and manifest copies. Keep the fixture isolated from
    // private records and local agent directories.
    for relative in [
        "cvl/profile.toml",
        "interview/stations.toml",
        "cvl/README.md",
        "interview/README.md",
        "opportunities/README.md",
        ".agent/schemas/review-result.schema.json",
        "cvl/shared/harvard/style.toml",
        "cvl/shared/harvard/defaults.toml",
        "cvl/shared/harvard/style.typ",
        "cvl/cv/harvard/src/cv.typ",
        "cvl/cv/harvard/src/entries-de.typ",
        "cvl/cv/harvard/src/entries-en.typ",
        "cvl/cl/harvard/src/cl.typ",
    ] {
        let path = temporary.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
    }
    for relative in [
        "ccvl.json",
        "cvl/cv/harvard/style.toml",
        "cvl/cl/harvard/style.toml",
        "cvl/cv/harvard/contract.toml",
        "cvl/cl/harvard/contract.toml",
    ] {
        let path = temporary.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, fs::read(repository.path(relative)).unwrap()).unwrap();
    }
    // Only Harvard is copied, so each document offers a single style slot.
    let mut manifest = repository.read_json("ccvl.json").unwrap();
    for key in ["cv", "cover_letter"] {
        manifest["documents"][key]["slots"]["styles"] = 1.into();
    }
    fs::write(temporary.path().join("ccvl.json"), format!("{manifest}\n")).unwrap();
    for leaf in cv_leaves(&repository)
        .unwrap()
        .into_iter()
        .chain(cl_leaves(&repository).unwrap())
        .filter(|leaf| leaf.style == "harvard")
    {
        for relative in [
            repository.relative(&leaf.content()).unwrap(),
            repository.relative(&leaf.strings()).unwrap(),
            repository.relative(&leaf.adapter()).unwrap(),
            repository.relative(&leaf.substyle_file()).unwrap(),
        ] {
            let path = temporary.path().join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "").unwrap();
        }
    }
    let workspace = Workspace::at(temporary.path()).unwrap();
    (temporary, workspace)
}

#[test]
fn manifest_accepts_ci_metadata_and_rejects_legacy_roots() {
    let (temporary, workspace) = harvard_manifest_fixture();
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

#[test]
fn cover_letter_contract_rejects_weakened_density_and_closing_spill() {
    let (temporary, workspace) = harvard_manifest_fixture();
    validate_manifest(&workspace).unwrap();
    validate_styles(&workspace).unwrap();
    validate_contracts(&workspace).unwrap();

    for (pointer, weakened, message) in [
        ("/line_fill/body/non_final_minimum", 75, "body fill"),
        ("/line_fill/body/minimum", 60, "body fill"),
        ("/line_fill/body/maximum", 102, "body fill"),
        ("/line_fill/highlight/minimum", 60, "highlight fill"),
    ] {
        let contract_path = temporary.path().join("cvl/cl/harvard/contract.toml");
        let text = fs::read_to_string(&contract_path).unwrap();
        let mut contract: toml::Value = toml::from_str(&text).unwrap();
        let target = pointer.split('/').filter(|part| !part.is_empty()).fold(
            &mut contract,
            |value, part| {
                value
                    .as_table_mut()
                    .expect("contract section")
                    .get_mut(part)
                    .expect("contract key")
            },
        );
        *target = toml::Value::Integer(weakened);
        fs::write(&contract_path, toml::to_string(&contract).unwrap()).unwrap();
        let error = validate_contracts(&workspace).unwrap_err().to_string();
        assert!(error.contains(message), "{pointer}: {error}");
        fs::write(&contract_path, text).unwrap();
        validate_contracts(&workspace).unwrap();
    }
}
/// Pin the frozen measurement contracts in the style tree. Every value here
/// is a product guarantee: weakening one silently reflows measured lines.
fn validate_contracts(workspace: &Workspace) -> Result<()> {
    let cv = styles::contract(workspace, "cv", "harvard")?;
    ensure!(
        cv.pointer("/presets") == Some(&json!([2, 3, 4])),
        "CV contract: presets must be [2, 3, 4]"
    );
    ensure!(
        cv.pointer("/summary_lines") == Some(&Value::from(5)),
        "CV contract: every CV Summary must render to exactly five lines"
    );
    ensure!(
        cv.pointer("/summary_fill") == Some(&json!({"minimum": 95, "target": 97, "maximum": 100})),
        "CV contract: Summary fill defaults must be 95/97/100"
    );
    ensure!(
        cv.pointer("/last_line_maximum") == Some(&Value::from(102)),
        "CV contract: closing-line maximum must be 102"
    );
    let layout = cv
        .pointer("/layout_contract")
        .context("CV contract has no layout contract")?;
    ensure!(
        layout.pointer("/page_1/entries")
            == Some(&json!({"minimum": 6, "target": 7, "maximum": 8})),
        "CV contract: page 1 contract changed"
    );
    ensure!(
        layout.pointer("/page_2") == Some(&json!({"entries": 10, "bullets_per_entry": 2})),
        "CV contract: page 2 contract changed"
    );
    ensure!(
        layout.pointer("/page_3") == Some(&json!({"entries": 10, "bullets_per_entry": 2})),
        "CV contract: page 3 contract changed"
    );
    ensure!(
        layout.pointer("/page_4")
            == Some(&json!({"groups": 3, "entries_per_group": 3, "bullets_per_entry": 3})),
        "CV contract: page 4 contract changed"
    );
    ensure!(
        layout.get("verified_only") == Some(&Value::Bool(true))
            && layout.get("unique_fact_assignment") == Some(&Value::Bool(true)),
        "CV contract: evidence or MECE guarantees were weakened"
    );
    let cl = styles::contract(workspace, "cl", "harvard")?;
    let paragraphs = cl
        .get("paragraphs")
        .and_then(Value::as_array)
        .context("cover-letter contract has no paragraphs")?;
    // The strict 3|5|5|5|5|3 framework: exactly six paragraphs with exact
    // line budgets.
    ensure!(
        paragraphs.len() == 6,
        "cover-letter contract: paragraph framework changed"
    );
    for (paragraph, (minimum, maximum)) in
        paragraphs
            .iter()
            .zip([(3, 3), (5, 5), (5, 5), (5, 5), (5, 5), (3, 3)])
    {
        ensure!(
            paragraph.pointer("/lines/minimum") == Some(&Value::from(minimum))
                && paragraph.pointer("/lines/maximum") == Some(&Value::from(maximum)),
            "cover-letter contract: paragraph line framework changed"
        );
    }
    ensure!(
        cl.pointer("/body_lines") == Some(&json!({"minimum": 26, "target": 26, "maximum": 26})),
        "cover-letter contract: body contract changed"
    );
    ensure!(
        cl.pointer("/highlights/count") == Some(&Value::from(5)),
        "cover letter needs exactly five highlights"
    );
    ensure!(
        cl.pointer("/line_fill/body")
            == Some(&json!({
                "minimum": 75,
                "non_final_minimum": 95,
                "target": 97,
                "maximum": 100
            })),
        "cover-letter contract: body fill must stay 75/95/97/100; the higher \
         non-final floor is the density gate and may not be weakened"
    );
    ensure!(
        cl.pointer("/line_fill/highlight")
            == Some(&json!({"minimum": 70, "target": 82, "maximum": 100})),
        "cover-letter contract: highlight fill must stay 70/82/100"
    );
    ensure!(
        cl.pointer("/vertical_rhythm/gap_pt")
            == Some(&json!({"minimum": 12, "target": 20, "maximum": 30})),
        "cover-letter contract: vertical rhythm changed"
    );
    ensure!(
        cl.pointer("/vertical_rhythm/highlight_center_percent")
            == Some(&json!({"minimum": 50, "target": 56, "maximum": 60})),
        "cover-letter contract: highlight position changed"
    );
    ensure!(
        cl.pointer("/widow_or_orphan_lines") == Some(&Value::from(0)),
        "cover-letter contract: widow/orphan rule changed"
    );
    Ok(())
}

#[test]
fn shipped_harvard_contracts_remain_strict() {
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    validate_contracts(&workspace).unwrap();
}
