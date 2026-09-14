use serde_json::json;

use super::*;

fn workspace() -> Workspace {
    Workspace::at(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn lines(count: usize) -> Vec<Value> {
    (0..count).map(|_| json!("evidence")).collect()
}

fn application(paragraph_lengths: &[usize]) -> Value {
    json!({
        "schema_version": 4,
        "revision": 0,
        "options": {
            "language": "de-ch",
            "pages": 4,
            "generate_cl": true,
            "application_date": "September 2026",
        },
        "job": {
            "id": "fixture",
            "title": "Fixture",
            "organization": "Fixture",
            "location": "Fixture",
            "source": "Fixture",
            "url": "Fixture",
            "description": "Fixture",
            "connections": "",
            "company_context": "",
            "notes": "",
            "cl_recipient": {
                "name": "",
                "title": "",
                "company": "",
                "address_line_1": "",
                "address_line_2": "",
            },
        },
        "cv": {"summary": "Flowing evidence paragraph."},
        "cl": {
            "paragraphs": paragraph_lengths.iter().map(|length| lines(*length)).collect::<Vec<_>>(),
            "highlights": lines(5),
        },
    })
}

#[test]
fn cv_only_application_is_valid_without_hidden_cover_letter_content() {
    let mut draft = application(&[3, 5, 5, 5, 5, 3]);
    draft.as_object_mut().unwrap().remove("cl");
    draft["options"]["generate_cl"] = json!(false);
    validate_record(&workspace(), &draft, "fixture", true).unwrap();

    draft["cl"] = json!({"paragraphs": [], "highlights": []});
    let error = validate_record(&workspace(), &draft, "fixture", true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("disabled cover letter"));
}

#[test]
fn strict_cover_letter_line_budgets_are_enforced() {
    // The strict 3|5|5|5|5|3 framework admits exactly one distribution:
    // the valid letter passes while every off-by-one in any paragraph
    // fails, which also implies the pair (10), central (20), and body
    // (26) totals without a separate region test.
    let workspace = workspace();
    validate_record(
        &workspace,
        &application(&[3, 5, 5, 5, 5, 3]),
        "fixture",
        true,
    )
    .unwrap();
    for (lengths, expected) in [
        ([2, 5, 5, 5, 5, 3], "paragraphs[1]: expected 3–3 lines"),
        ([4, 5, 5, 5, 5, 3], "paragraphs[1]: expected 3–3 lines"),
        ([3, 4, 5, 5, 5, 3], "paragraphs[2]: expected 5–5 lines"),
        ([3, 6, 5, 5, 5, 3], "paragraphs[2]: expected 5–5 lines"),
        ([3, 5, 4, 5, 5, 3], "paragraphs[3]: expected 5–5 lines"),
        ([3, 5, 6, 5, 5, 3], "paragraphs[3]: expected 5–5 lines"),
        ([3, 5, 5, 4, 5, 3], "paragraphs[4]: expected 5–5 lines"),
        ([3, 5, 5, 6, 5, 3], "paragraphs[4]: expected 5–5 lines"),
        ([3, 5, 5, 5, 4, 3], "paragraphs[5]: expected 5–5 lines"),
        ([3, 5, 5, 5, 6, 3], "paragraphs[5]: expected 5–5 lines"),
        ([3, 5, 5, 5, 5, 2], "paragraphs[6]: expected 3–3 lines"),
        ([3, 5, 5, 5, 5, 4], "paragraphs[6]: expected 3–3 lines"),
    ] {
        let error = validate_record(&workspace, &application(&lengths), "fixture", true)
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "unexpected error: {error}");
    }
}

#[test]
fn german_flowing_summary_with_special_characters_validates() {
    let workspace = workspace();
    let mut draft = application(&[3, 5, 5, 5, 5, 3]);
    draft["cv"]["summary"] = json!(
        "Mittelstandsmandate verbinden Finanzen, Betrieb und Technologie. \
             Ich vereine Portfolioanalyse, Corporate Finance und Transformation mit \
             praktischer Cloud-/KI-Umsetzung. Damit unterstütze ich Leverage Experts \
             pragmatisch in Performance-, Portfolio- und Transformationsmandaten. \
             GenAI bei CENVION | RAG-Suche, CHF 10 Mio., 20+ Jahre, für & mit."
    );
    validate_record(&workspace, &draft, "fixture", true).unwrap();
    draft["options"]["generate_cl"] = json!(false);
    draft.as_object_mut().unwrap().remove("cl");
    validate_record(&workspace, &draft, "fixture", true).unwrap();
}

#[test]
fn document_prose_rejects_every_dash() {
    assert!(reject_dashes("plain words, no dashes", "here").is_ok());
    for text in [
        "hyphen-ated",
        "en–dash",
        "em—dash",
        "non‐breaking hyphen",
        "figure‒dash",
        "horizontal―bar",
        "minus − sign",
    ] {
        let error = reject_dashes(text, "here").unwrap_err().to_string();
        assert!(
            error.contains("must not contain dashes"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn unknown_fields_are_rejected() {    let mut draft = application(&[3, 5, 5, 5, 5, 3]);
    draft["job"]["smuggled"] = json!("nope");
    let error = validate_record(&workspace(), &draft, "fixture", true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("unknown fields"));
}

#[test]
fn empty_rendered_text_is_rejected() {
    let mut draft = application(&[3, 5, 5, 5, 5, 3]);
    draft["cv"]["summary"] = json!("  ");
    let error = validate_record(&workspace(), &draft, "fixture", true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("cannot be empty"));
    draft["cv"]["summary"] = json!("Flowing evidence paragraph.");
    draft["cl"]["highlights"][0] = json!("  ");
    let error = validate_record(&workspace(), &draft, "fixture", true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("cannot be empty"));
}

#[test]
fn missing_recipient_name_warns_without_failing_validation() {
    // Empty/whitespace names stay valid (showcase target-neutral letters)
    // but produce a visible, non-blocking advisory.
    let draft = application(&[3, 5, 5, 5, 5, 3]);
    validate_record(&workspace(), &draft, "fixture", true).unwrap();
    let warning = recipient_salutation_warning(
        "fixture",
        draft["job"]["cl_recipient"]["name"].as_str().unwrap(),
    )
    .expect("empty showcase recipient must warn");
    assert!(warning.contains("job.cl_recipient.name is empty"));
    assert!(warning.contains("generic salutation"));
    assert!(recipient_salutation_warning("fixture", "Dr. Jane Doe").is_none());
    assert!(recipient_salutation_warning("fixture", "   ").is_some());
}

#[test]
fn substyles_default_to_standard_and_left_rule() {
    // The fixture carries no selection keys, like records written before
    // per-document selection existed: validation accepts it and
    // resolution yields the family defaults.
    let workspace = workspace();
    let draft = application(&[3, 5, 5, 5, 5, 3]);
    validate_record(&workspace, &draft, "fixture", true).unwrap();
    assert_eq!(
        resolve_cv_substyle(&workspace, &draft, "fixture").unwrap(),
        "standard"
    );
    assert_eq!(
        resolve_cl_substyle(&workspace, &draft, "fixture").unwrap(),
        "left-rule"
    );

    let mut empty = draft.clone();
    empty["options"]["cv_substyle"] = json!("");
    empty["options"]["cl_substyle"] = json!("");
    validate_record(&workspace, &empty, "fixture", true).unwrap();
    assert_eq!(
        resolve_cv_substyle(&workspace, &empty, "fixture").unwrap(),
        "standard"
    );
    assert_eq!(
        resolve_cl_substyle(&workspace, &empty, "fixture").unwrap(),
        "left-rule"
    );

    let mut selected = draft.clone();
    selected["options"]["cv_substyle"] = json!("compact");
    selected["options"]["cl_substyle"] = json!("frame");
    validate_record(&workspace, &selected, "fixture", true).unwrap();
    assert_eq!(
        resolve_cv_substyle(&workspace, &selected, "fixture").unwrap(),
        "compact"
    );
    assert_eq!(
        resolve_cl_substyle(&workspace, &selected, "fixture").unwrap(),
        "frame"
    );
}

#[test]
fn retired_style_selection_is_rejected() {
    let workspace = workspace();
    let mut draft = application(&[3, 5, 5, 5, 5, 3]);
    draft["options"]["style"] = json!("harvard");
    let error = validate_record(&workspace, &draft, "fixture", true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("style"), "unexpected error: {error}");
}

#[test]
fn unknown_substyle_fails_with_available_list() {
    let workspace = workspace();
    let mut draft = application(&[3, 5, 5, 5, 5, 3]);
    draft["options"]["cv_substyle"] = json!("nope");
    let error = validate_record(&workspace, &draft, "fixture", true)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("unknown cv substyle"),
        "unexpected error: {error}"
    );
    assert!(error.contains("compact"), "unexpected error: {error}");
    let error = resolve_cv_substyle(&workspace, &draft, "fixture")
        .unwrap_err()
        .to_string();
    assert!(error.contains("standard"), "unexpected error: {error}");

    draft["options"]
        .as_object_mut()
        .unwrap()
        .remove("cv_substyle");
    draft["options"]["cl_substyle"] = json!("nope");
    let error = resolve_cl_substyle(&workspace, &draft, "fixture")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("unknown cl substyle"),
        "unexpected error: {error}"
    );
    assert!(error.contains("frame"), "unexpected error: {error}");
}

#[test]
fn style_leaves_cover_every_substyle_and_locale() {
    let workspace = workspace();
    let cv = cv_leaves(&workspace)
        .unwrap()
        .into_iter()
        .filter(|leaf| leaf.style == "harvard")
        .collect::<Vec<_>>();
    assert_eq!(cv.len(), 4);
    for (substyle, locale) in [
        ("standard", "de-ch"),
        ("standard", "en-ch"),
        ("compact", "de-ch"),
        ("compact", "en-ch"),
    ] {
        let leaf = cv
            .iter()
            .find(|leaf| leaf.substyle == substyle && leaf.locale == locale)
            .unwrap_or_else(|| panic!("missing CV leaf {substyle} {locale}"));
        assert!(leaf.content().is_file());
        assert!(leaf.strings().is_file());
        assert!(leaf.adapter().is_file());
        assert!(leaf.substyle_file().is_file());
    }
    let cl = cl_leaves(&workspace)
        .unwrap()
        .into_iter()
        .filter(|leaf| leaf.style == "harvard")
        .collect::<Vec<_>>();
    assert_eq!(cl.len(), 4);
    for (substyle, locale) in [
        ("left-rule", "de-ch"),
        ("left-rule", "en-ch"),
        ("frame", "de-ch"),
        ("frame", "en-ch"),
    ] {
        assert!(
            cl.iter()
                .any(|leaf| leaf.substyle == substyle && leaf.locale == locale),
            "missing cover-letter leaf {substyle} {locale}"
        );
    }
}

#[test]
fn non_string_substyle_is_rejected() {
    let mut draft = application(&[3, 5, 5, 5, 5, 3]);
    draft["options"]["cv_substyle"] = json!(3);
    let error = resolve_cv_substyle(&workspace(), &draft, "fixture")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("options/cv_substyle must be a substyle name"),
        "unexpected error: {error}"
    );
    draft["options"]
        .as_object_mut()
        .unwrap()
        .remove("cv_substyle");
    draft["options"]["cl_substyle"] = json!(3);
    let error = resolve_cl_substyle(&workspace(), &draft, "fixture")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("options/cl_substyle must be a substyle name"),
        "unexpected error: {error}"
    );
}
