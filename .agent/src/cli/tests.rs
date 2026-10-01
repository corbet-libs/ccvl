use super::*;

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
        "en-ch",
        "--style",
        "harvard",
        "--substyle",
        "frame",
    ])
    .unwrap();
    assert!(
        matches!(args.command, Command::ExplainStyle { document, locale, style: Some(style), substyle: Some(substyle), .. } if document == "cl" && locale == "en-ch" && style == "harvard" && substyle == "frame")
    );
    assert!(Args::try_parse_from(["ccvl", "explain-style", "resume", "en-ch"]).is_err());
}

#[test]
fn paper_arguments_do_not_add_a_locale_or_hierarchy_level() {
    for command in ["build-cv", "build-cl", "watch-cv", "watch-cl"] {
        let args =
            Args::try_parse_from(["ccvl", command, "en-ch", "--paper", "us-letter"]).unwrap();
        match args.command {
            Command::BuildCv { paper, .. }
            | Command::BuildCl { paper, .. }
            | Command::WatchCv { paper, .. }
            | Command::WatchCl { paper, .. } => assert_eq!(paper.as_deref(), Some("us-letter")),
            _ => panic!("unexpected command"),
        }
    }
    let args = Args::try_parse_from([
        "ccvl",
        "explain-style",
        "cv",
        "en-ch",
        "--paper",
        "us-letter",
    ])
    .unwrap();
    assert!(
        matches!(args.command, Command::ExplainStyle { paper: Some(paper), .. } if paper == "us-letter")
    );
}

#[test]
fn document_checks_accept_repeatable_style_filters() {
    for command in ["check", "public-check", "measure"] {
        let args = Args::try_parse_from([
            "ccvl",
            command,
            "--style",
            "cv/modern",
            "--style",
            "cl/harvard",
        ])
        .unwrap();
        let styles = match args.command {
            Command::Check { styles }
            | Command::PublicCheck { styles, .. }
            | Command::Measure { styles, .. } => styles,
            other => panic!("unexpected command {other:?}"),
        };
        assert_eq!(styles, ["cv/modern", "cl/harvard"]);
        let args = Args::try_parse_from(["ccvl", command]).unwrap();
        assert!(matches!(
            args.command,
            Command::Check { styles } | Command::PublicCheck { styles, .. } | Command::Measure { styles, .. }
                if styles.is_empty()
        ));
    }
}

#[test]
fn reserved_default_resolves_command_line_and_record_selections() {
    let workspace = Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let selected = cli_selection(&workspace, Some("default"), Some("default"), None, "cv").unwrap();
    assert_eq!(
        (selected.style.as_str(), selected.substyle.as_str()),
        ("harvard", "d-plus")
    );
    let selected = cli_selection(&workspace, Some("default"), Some("default"), None, "cl").unwrap();
    assert_eq!(
        (selected.style.as_str(), selected.substyle.as_str()),
        ("harvard", "left-rule")
    );
    // A record selecting `default` resolves the same way; explicit flags win.
    let directory = tempfile::tempdir_in(workspace.root()).unwrap();
    let record = directory.path().join("application.toml");
    let mut content = crate::content::read_record(
        &workspace,
        workspace.path("cvl/cv/harvard/compact/en/ch/content.toml"),
    )
    .unwrap();
    content["options"]["cv_style"] = "default".into();
    content["options"]["cv_substyle"] = "default".into();
    std::fs::write(&record, toml::to_string(&content).unwrap()).unwrap();
    let selected = cli_selection(&workspace, None, None, Some(&record), "cv").unwrap();
    assert_eq!(selected.substyle, "d-plus");
    let selected = cli_selection(&workspace, None, Some("compact"), Some(&record), "cv").unwrap();
    assert_eq!(selected.substyle, "compact");
    for (style, substyle, message) in [
        (
            Some("harvard"),
            Some("slot-5"),
            "cv harvard/slot-5 is an empty slot; it has no design yet",
        ),
        (
            Some("slot-4"),
            None,
            "cv slot-4 is an empty style slot; it has no design yet",
        ),
    ] {
        let error = cli_selection(&workspace, style, substyle, None, "cv")
            .unwrap_err()
            .to_string();
        assert_eq!(error, message);
    }
}

#[test]
fn list_styles_is_a_workspace_command() {
    assert!(matches!(
        Args::try_parse_from(["ccvl", "list-styles"])
            .unwrap()
            .command,
        Command::ListStyles
    ));
}
