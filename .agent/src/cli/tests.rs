use super::*;

fn watched_workspace() -> (tempfile::TempDir, Workspace) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir_all(root.join("cvl/cv/harvard/standard/de/ch/typst")).unwrap();
    fs::create_dir_all(root.join(".agent/typst")).unwrap();
    fs::create_dir_all(root.join("opportunities/acme/lead/pdfs")).unwrap();
    fs::create_dir_all(root.join("opportunities/acme/lead/typst")).unwrap();
    fs::write(root.join("ccvl.json"), "{}\n").unwrap();
    fs::write(
        root.join("cvl/cv/harvard/standard/de/ch/typst/cv.typ"),
        "#let x = 1\n",
    )
    .unwrap();
    fs::write(
        root.join("cvl/cv/harvard/standard/de/ch/content.toml"),
        "language = \"en-ch\"\n",
    )
    .unwrap();
    fs::write(root.join(".agent/typst/shared.typ"), "#let y = 2\n").unwrap();
    fs::write(
        root.join("opportunities/acme/lead/application.toml"),
        "language = \"en-ch\"\n",
    )
    .unwrap();
    fs::write(
        root.join("opportunities/acme/lead/typst/cv.typ"),
        "#let x = 1\n",
    )
    .unwrap();
    fs::write(root.join("opportunities/acme/lead/pdfs/cv.pdf"), b"%PDF-").unwrap();
    let workspace = Workspace::at(root).unwrap();
    (directory, workspace)
}

fn fixture_digest(workspace: &Workspace) -> Vec<u8> {
    digest_roots(&[
        workspace.path("cvl"),
        workspace.path(".agent/typst"),
        workspace.path("ccvl.json"),
        workspace.path("opportunities/acme/lead"),
    ])
    .unwrap()
}

#[test]
fn watched_extensions_cover_templates_records_and_generated_typs() {
    for watched in [
        "cv.typ",
        "application.toml",
        "contract.json",
        "asset.png",
        "font.ttf",
        "drawing.svg",
    ] {
        assert!(is_watched(Path::new(watched)), "{watched} was ignored");
    }
    for ignored in ["cv.pdf", "notes.md", "no-extension"] {
        assert!(!is_watched(Path::new(ignored)), "{ignored} was watched");
    }
}

#[test]
fn digest_reacts_to_templates_records_contracts_and_generated_typs() {
    let (_directory, workspace) = watched_workspace();
    let baseline = fixture_digest(&workspace);
    // A rebuilt PDF alone must not retrigger the watcher.
    fs::write(
        workspace.path("opportunities/acme/lead/pdfs/cv.pdf"),
        b"%PDF-changed",
    )
    .unwrap();
    assert_eq!(fixture_digest(&workspace), baseline);
    // Untouched content hashes stably.
    assert_eq!(fixture_digest(&workspace), baseline);
    for relative in [
        "cvl/cv/harvard/standard/de/ch/typst/cv.typ",
        "cvl/cv/harvard/standard/de/ch/content.toml",
        ".agent/typst/shared.typ",
        "ccvl.json",
        "opportunities/acme/lead/application.toml",
        "opportunities/acme/lead/typst/cv.typ",
    ] {
        let path = workspace.path(relative);
        let before = fs::read(&path).unwrap();
        fs::write(&path, [before.clone(), b"changed\n".to_vec()].concat()).unwrap();
        assert_ne!(
            fixture_digest(&workspace),
            baseline,
            "{relative} was ignored"
        );
        fs::write(&path, before).unwrap();
        assert_eq!(fixture_digest(&workspace), baseline);
    }
}

#[test]
fn invalid_record_keys_are_rejected_before_watching() {
    let (_directory, workspace) = watched_workspace();
    assert!(opportunity::record_path(&workspace, "../acme", "lead", true).is_err());
    assert!(opportunity::record_path(&workspace, "acme", "lead", true).is_ok());
}

#[test]
fn style_substyle_and_page_arguments_remain_independent() {
    let cv = Args::try_parse_from([
        "ccvl",
        "build-cv",
        "en-us",
        "1",
        "--style",
        "orbit",
        "--substyle",
        "standard",
    ])
    .unwrap();
    assert!(
        matches!(cv.command, Command::BuildCv { pages: Some(1), style: Some(style), substyle: Some(substyle), .. } if style == "orbit" && substyle == "standard")
    );
    let cl = Args::try_parse_from([
        "ccvl",
        "build-cl",
        "en-us",
        "--pages",
        "2",
        "--style",
        "postcard",
        "--substyle",
        "standard",
    ])
    .unwrap();
    assert!(
        matches!(cl.command, Command::BuildCl { pages: Some(2), style: Some(style), substyle: Some(substyle), .. } if style == "postcard" && substyle == "standard")
    );
}

#[test]
fn explain_style_accepts_document_locale_and_scoped_selection() {
    let args = Args::try_parse_from([
        "ccvl",
        "explain-style",
        "cl",
        "en-us",
        "--style",
        "test-style-2",
        "--substyle",
        "cards",
    ])
    .unwrap();
    assert!(
        matches!(args.command, Command::ExplainStyle { document, locale, style: Some(style), substyle: Some(substyle) } if document == "cl" && locale == "en-us" && style == "test-style-2" && substyle == "cards")
    );
    assert!(Args::try_parse_from(["ccvl", "explain-style", "resume", "en-us"]).is_err());
}
