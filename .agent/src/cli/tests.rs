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
        "en-us",
        "--style",
        "test-style-2",
        "--substyle",
        "cards",
    ])
    .unwrap();
    assert!(
        matches!(args.command, Command::ExplainStyle { document, locale, style: Some(style), substyle: Some(substyle), .. } if document == "cl" && locale == "en-us" && style == "test-style-2" && substyle == "cards")
    );
    assert!(Args::try_parse_from(["ccvl", "explain-style", "resume", "en-us"]).is_err());
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
