use super::*;
use serde_json::json;
use std::fs;

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn fixture() -> (tempfile::TempDir, Workspace) {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    write(
        root,
        "ccvl.json",
        &json!({"documents": {"cv": {"root": "cvl/cv"}, "cover_letter": {"root": "cvl/cl"}}})
            .to_string(),
    );
    for (document, style) in [("cv", "first"), ("cv", "second"), ("cl", "first")] {
        let prefix = format!("cvl/{document}/{style}");
        write(
            root,
            &format!("{prefix}/style.toml"),
            &format!(
                "id = {style:?}\napi = 1\ndocuments = [{document:?}]\nsupports_locales = [\"en-ch\", \"en-us\"]\npages = [1]\ndefault_pages = 1\nsubstyles = [\"one\", \"two\"]\n"
            ),
        );
        for locale in ["ch", "us"] {
            write(
                root,
                &format!("{prefix}/content/en/{locale}/wording.toml"),
                &format!(
                    "[{document}]\ntitle = {style:?}\nitems = [\"first\", \"second\"]\n[{document}.details]\nleft = \"inherited\"\nright = \"shared\"\n"
                ),
            );
            for substyle in ["one", "two"] {
                write(
                    root,
                    &format!("{prefix}/{substyle}/en/{locale}/content.toml"),
                    &format!(
                        "revision = 7\n[wording]\nsource = \"../../../content/en/{locale}/wording.toml\"\n"
                    ),
                );
            }
        }
    }
    let repository = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    write(
        root,
        ".agent/typst/application.typ",
        &fs::read_to_string(repository.path(".agent/typst/application.typ")).unwrap(),
    );
    let workspace = Workspace::at(root).unwrap();
    (temporary, workspace)
}

fn query_record(workspace: &Workspace, path: &str) -> Result<Value> {
    let engine = ctypst::Engine::builder()
        .root(workspace.root())
        .fonts(ctypst::fonts::documents())
        .build()?;
    let source = format!(
        "#import \"/.agent/typst/application.typ\": load-application\n#set text(font: \"Archivo\")\n#metadata(load-application(\"/{path}\")) <resolved-record>\nWording fixture.\n"
    );
    let output = engine
        .compile(ctypst::CompileRequest::new("fixture.typ").source_file("fixture.typ", source))?;
    ctypst::query_json(&output.document, "resolved-record")?
        .into_iter()
        .next()
        .context("missing resolved record")
}

#[test]
fn shared_edits_reach_sibling_substyles_but_preserve_explicit_exceptions() {
    let (_temporary, workspace) = fixture();
    let one = "cvl/cv/first/one/en/ch/content.toml";
    let two = "cvl/cv/first/two/en/ch/content.toml";
    write(
        workspace.root(),
        two,
        "revision = 7\n[wording]\nsource = \"../../../content/en/ch/wording.toml\"\n[cv]\nitems = [\"replacement\"]\n[cv.details]\nright = \"exception\"\n",
    );
    let before = read_record(&workspace, two).unwrap();
    assert_eq!(before["cv"]["items"], json!(["replacement"]));
    assert_eq!(
        before["cv"]["details"],
        json!({"left": "inherited", "right": "exception"})
    );
    assert_eq!(before["revision"], 7);
    assert!(before.get("wording").is_none());
    write(
        workspace.root(),
        "cvl/cv/first/content/en/ch/wording.toml",
        "[cv]\ntitle = \"Updated shared wording\"\nitems = [\"changed\"]\n[cv.details]\nleft = \"updated\"\nright = \"updated\"\n",
    );
    for path in [one, two] {
        let resolved = read_record(&workspace, path).unwrap();
        assert_eq!(resolved["cv"]["title"], "Updated shared wording");
        assert_eq!(resolved, query_record(&workspace, path).unwrap());
    }
    let resolved = read_record(&workspace, two).unwrap();
    assert_eq!(resolved["cv"]["items"], json!(["replacement"]));
    assert_eq!(
        resolved["cv"]["details"],
        json!({"left": "updated", "right": "exception"})
    );
    assert_eq!(
        read_record(&workspace, "cvl/cv/second/one/en/ch/content.toml").unwrap()["cv"]["title"],
        "second"
    );
    assert_eq!(
        read_record(&workspace, "cvl/cl/first/one/en/ch/content.toml").unwrap()["cl"]["title"],
        "first"
    );
    assert_eq!(
        read_record(&workspace, "cvl/cv/first/one/en/us/content.toml").unwrap()["cv"]["title"],
        "first"
    );
}

#[test]
fn source_scope_and_nested_references_fail_in_rust_and_direct_typst() {
    let (_temporary, workspace) = fixture();
    let path = "cvl/cv/first/one/en/ch/content.toml";
    for source in [
        "../../../../second/content/en/ch/wording.toml",
        "../../../../../cl/first/content/en/ch/wording.toml",
        "../../../content/en/us/wording.toml",
        "/cvl/cv/first/content/en/ch/wording.toml",
        "../../../content/en/ch/other.toml",
    ] {
        write(
            workspace.root(),
            path,
            &format!("[wording]\nsource = {source:?}\n"),
        );
        assert!(read_record(&workspace, path).is_err(), "{source}");
        assert!(query_record(&workspace, path).is_err(), "{source}");
    }
    write(
        workspace.root(),
        path,
        "[wording]\nsource = \"../../../content/en/ch/wording.toml\"\n",
    );
    for shared in [
        "[cl]\nbody = \"wrong document\"\n",
        "[wording]\nsource = \"elsewhere.toml\"\n[cv]\ntitle = \"nested\"\n",
        "[cv]\ntitle = \"mixed\"\n[cl]\nbody = \"wrong document\"\n",
    ] {
        write(
            workspace.root(),
            "cvl/cv/first/content/en/ch/wording.toml",
            shared,
        );
        assert!(read_record(&workspace, path).is_err());
        assert!(query_record(&workspace, path).is_err());
    }
}

#[test]
fn private_records_are_self_contained_and_whole_array_overrides_can_be_empty() {
    let (_temporary, workspace) = fixture();
    let path = "cvl/cv/first/one/en/ch/content.toml";
    write(
        workspace.root(),
        path,
        "[wording]\nsource = \"../../../content/en/ch/wording.toml\"\n[cv]\nitems = []\ncustom_shape = [{arbitrary = {nested = [1, 2, 3]}}]\n",
    );
    let resolved = read_record(&workspace, path).unwrap();
    assert_eq!(resolved["cv"]["items"], json!([]));
    assert_eq!(resolved, query_record(&workspace, path).unwrap());
    let private = "opportunities/example/role/application.toml";
    write(
        workspace.root(),
        private,
        &toml::to_string(&resolved).unwrap(),
    );
    assert_eq!(resolved, read_record(&workspace, private).unwrap());
    assert_eq!(resolved, query_record(&workspace, private).unwrap());
    write(
        workspace.root(),
        private,
        "[wording]\nsource = \"../../../content/en/ch/wording.toml\"\n",
    );
    assert!(
        read_record(&workspace, private)
            .unwrap_err()
            .to_string()
            .contains("self-contained")
    );
    assert!(query_record(&workspace, private).is_err());
}

#[cfg(unix)]
#[test]
fn a_wording_symlink_cannot_cross_style_ownership() {
    let (_temporary, workspace) = fixture();
    let expected = workspace.path("cvl/cv/first/content/en/ch/wording.toml");
    fs::remove_file(&expected).unwrap();
    std::os::unix::fs::symlink(
        workspace.path("cvl/cv/second/content/en/ch/wording.toml"),
        expected,
    )
    .unwrap();
    let error = read_record(&workspace, "cvl/cv/first/one/en/ch/content.toml")
        .unwrap_err()
        .to_string();
    assert!(error.contains("symlink"), "{error}");
}

#[test]
fn shared_letter_does_not_read_or_observe_an_absent_or_malformed_cv_root() {
    let (_temporary, workspace) = fixture();
    fs::remove_dir_all(workspace.path("cvl/cv")).unwrap();
    let path = "cvl/cl/first/one/en/ch/content.toml";
    for malformed in [false, true] {
        if malformed {
            let mut manifest = workspace.read_json("ccvl.json").unwrap();
            manifest["documents"]["cv"]["root"] = 123.into();
            write(workspace.root(), "ccvl.json", &manifest.to_string());
        }
        let observed = workspace.tracked();
        let resolved = read_record(&observed, path).unwrap();
        assert_eq!(resolved["cl"]["title"], "first");
        assert_eq!(resolved, query_record(&workspace, path).unwrap());
        assert!(
            !observed
                .observed_inputs()
                .iter()
                .any(|path| path.starts_with(workspace.path("cvl/cv")))
        );
    }
}

fn copy_sources(original: &Workspace, root: &Path, relative: &str) {
    let source = original.path(relative);
    if source.is_file() {
        let target = root.join(relative);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(source, target).unwrap();
        return;
    }
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.unwrap();
        if !entry.file_type().is_file()
            || entry
                .path()
                .components()
                .any(|part| part.as_os_str() == "pdf" || part.as_os_str() == "preview")
        {
            continue;
        }
        let relative = original.relative(entry.path()).unwrap();
        let target = root.join(relative);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(entry.path(), target).unwrap();
    }
}

#[test]
fn real_harvard_renderers_ignore_missing_and_malformed_opposite_contracts() {
    let original = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    for (document, substyle, opposite) in [("cv", "standard", "cl"), ("cl", "left-rule", "cv")] {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        for relative in [
            "ccvl.json",
            ".agent/typst",
            "cvl/shared/harvard",
            "cvl/assets",
            "cvl/profile.toml",
            "interview/stations.toml",
            &format!("cvl/{document}/harvard"),
        ] {
            copy_sources(&original, root, relative);
        }
        let workspace = Workspace::at(root).unwrap();
        let selected = styles::Selection {
            style: "harvard".into(),
            substyle: substyle.into(),
        };
        let original_leaf = styles::leaf(&original, document, "en-ch", &selected).unwrap();
        let baseline = fs::read(original_leaf.output(original_leaf.default_pages)).unwrap();
        for malformed in [false, true] {
            if malformed {
                write(
                    root,
                    &format!("cvl/{opposite}/harvard/contract.toml"),
                    "invalid = [",
                );
            }
            let observed = workspace.tracked();
            let leaf = styles::leaf(&observed, document, "en-ch", &selected).unwrap();
            let spec = crate::render::cvl_spec(&observed, &leaf, leaf.default_pages).unwrap();
            let output = crate::render::Compiler::new(&observed)
                .unwrap()
                .render(&observed, &spec)
                .unwrap();
            assert!(
                fs::read(output).unwrap() == baseline,
                "{document}, opposite malformed={malformed}"
            );
            assert!(
                !observed
                    .observed_inputs()
                    .iter()
                    .any(|path| path.starts_with(workspace.path(format!("cvl/{opposite}"))))
            );
        }
    }
}
