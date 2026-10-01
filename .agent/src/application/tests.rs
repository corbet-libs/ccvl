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
            "application_date": "2026-09",
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
fn application_date_rejects_preformatted_prose() {
    let mut draft = application(&[3, 5, 5, 5, 5, 3]);
    draft["options"]["application_date"] = json!("October 7, 2026");
    let error = validate_record(&workspace(), &draft, "fixture", true).unwrap_err();
    assert!(format!("{error:#}").contains("use quoted YYYY-MM-DD or YYYY-MM"));
    draft["options"]["application_date"] = json!("2026-10-07");
    validate_record(&workspace(), &draft, "fixture", true).unwrap();
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
fn document_prose_rejects_punctuation_tics() {
    assert!(reject_punctuation_tics("plain words, no dashes", "here").is_ok());
    // Necessary compounds stay valid here; they surface as advisories
    // for author judgment instead of failing validation.
    for text in [
        "hyphen-ated",
        "RAG-Systeme",
        "Cloud-Ökonomie",
        "50-100%",
        "unspaced–en–dashes",
        "GenAI- und RAG-Systeme",
    ] {
        assert!(
            reject_punctuation_tics(text, "here").is_ok(),
            "unexpected rejection: {text}"
        );
    }
    for text in [
        "em—dash",
        "bar―here",
        "spaced – dash",
        "trailing —",
        "— leading em dash",
        "dash - punctuation",
        "broken- Continuation",
        "minus − sign",
        "ellipsis…",
        "dots...",
        "double--hyphen",
        "non\u{2011}breaking hyphen",
    ] {
        let error = reject_punctuation_tics(text, "here")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("must not"),
            "unexpected pass: {text} ({error})"
        );
    }
}

#[test]
fn document_prose_rejects_space_before_comma() {
    assert!(reject_punctuation_spacing("Worte, sauber getrennt.", "here").is_ok());
    assert!(reject_punctuation_spacing("Aufzählung ohne Fehler", "here").is_ok());
    for text in ["Worte , sauber", "Worte  , sauber", "Worte\u{a0}, sauber"] {
        let error = reject_punctuation_spacing(text, "here")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("must not contain a space before a comma"),
            "unexpected pass: {text} ({error})"
        );
    }
}

#[test]
fn closing_line_requires_fixed_opener() {
    // The fixed opener is a prefix contract, never a full verbatim
    // sentence: any contribution wording may follow it.
    for line in [
        "Ich freue mich auf ein persönliches Gespräch.",
        "\"Ich freue mich, meine Erfahrung einzubringen.\"",
    ] {
        assert!(
            reject_closing_opener(
                &[json!("erste Zeile"), json!("zweite Zeile"), json!(line)],
                "here"
            )
            .is_ok(),
            "unexpected rejection: {line}"
        );
    }
    for line in [
        "Vielen Dank für Ihr Interesse.",
        "ich freue mich auf ein Gespräch.",
        "Gerne freue ich mich auf ein Gespräch.",
        "",
    ] {
        let error = reject_closing_opener(
            &[json!("erste Zeile"), json!("zweite Zeile"), json!(line)],
            "here",
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("must open its final line"),
            "unexpected pass: {line} ({error})"
        );
    }
    // Non-text or missing closing lines belong to the shape gates, not
    // this wording gate.
    assert!(reject_closing_opener(&[], "here").is_ok());
    assert!(reject_closing_opener(&[json!(3)], "here").is_ok());
}

#[test]
fn closing_rejects_forbidden_phrases() {
    for text in [
        "Ich passe fit im Team gut.",
        "FIT IM TEAM beizutragen.",
        "„Fit im Team“ wäre schön.",
        "Dank für Ihre Überlegungen im Voraus.",
        "dank für ihre überlegungen.",
    ] {
        let error = reject_forbidden_close(text, "here")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("must not use the forbidden phrase"),
            "unexpected pass: {text} ({error})"
        );
    }
    for text in [
        "Ich freue mich darauf, in Ihrem Team zum Erfolg beizutragen.",
        "Die Zusammenarbeit im Team stärkt die Datenstrecke.",
    ] {
        assert!(
            reject_forbidden_close(text, "here").is_ok(),
            "unexpected rejection: {text}"
        );
    }
}

#[test]
fn document_prose_rejects_grouped_thousands() {
    for text in [
        "1'000+ Gespräche analysiert.",
        "CHF 10 Mio verhandelt.",
        "GPA 4.0 mit 91 Adaptern.",
        "20+ Jahre Wartungsdaten.",
    ] {
        assert!(
            reject_number_grouping(text, "here").is_ok(),
            "unexpected rejection: {text}"
        );
    }
    for text in [
        "1.000 Gespräche",
        "2.000 Mitarbeitende",
        "1,000 calls",
        "CHF 100,000 Umsatz",
    ] {
        let error = reject_number_grouping(text, "here")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("must not use German/US thousands grouping"),
            "unexpected pass: {text} ({error})"
        );
    }
}

#[test]
fn document_prose_rejects_comma_ch_grades() {
    for text in [
        "Bestnote 6.0 (CH).",
        "Note 6.0 (CH) / 1,0 (DE).",
        "Note 1,0 (DE) in Frankfurt.",
    ] {
        assert!(
            reject_comma_ch_grade(text, "here").is_ok(),
            "unexpected rejection: {text}"
        );
    }
    for text in ["Bestnote 6,0 (CH)", "Note 5,6 (CH)"] {
        let error = reject_comma_ch_grade(text, "here").unwrap_err().to_string();
        assert!(
            error.contains("Swiss grades use dot display"),
            "unexpected pass: {text} ({error})"
        );
    }
}

#[test]
fn document_prose_rejects_german_bestnote_without_de_tag() {
    for text in [
        "Bestnote 6.0 (CH).",
        "Bestnote 6.0 (CH) / 1,0 (DE).",
        "Abschluss mit Note 1,0 in Frankfurt.",
    ] {
        assert!(
            reject_german_bestnote(text, "here").is_ok(),
            "unexpected rejection: {text}"
        );
    }
    for text in ["Bestnote 1,0", "Bestnote: 1.0 für Zürich"] {
        let error = reject_german_bestnote(text, "here")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("must not stand as the best grade"),
            "unexpected pass: {text} ({error})"
        );
    }
}

#[test]
fn closing_rejects_task_invitations_in_both_languages() {
    for text in [
        "Ich freue mich auf ein persönliches Gespräch.",
        "I would welcome the chance to discuss your priorities.",
        "Kunden schicken mir ihre Daten zur Auswertung.",
    ] {
        assert!(
            reject_task_invite(text, "here").is_ok(),
            "unexpected rejection: {text}"
        );
    }
    for text in [
        "Schicken Sie mir zwei Fragen zu Ihrer Plattform.",
        "Send me two questions about your platform.",
        "Send me a problem statement and I will solve it.",
    ] {
        let error = reject_task_invite(text, "here").unwrap_err().to_string();
        assert!(
            error.contains("must not invite test questions"),
            "unexpected pass: {text} ({error})"
        );
    }
}

#[test]
fn summary_rejects_formula_openings() {
    // Competence-first openings from real fixed records stay valid.
    for text in [
        "Als Cloud & Platform Engineer baue und betreibe ich Cloud Infrastrukturen.",
        "Analytiker für Gremien, die entscheiden müssen.",
        "Mit Physikstudium (Bestnote: 6.0) und CDI Executive Education bringe ich Kalkulation.",
        "As a physicist and consultant turned AI builder, I ship production GenAI.",
    ] {
        assert!(
            reject_formula_opening(text, "here").is_ok(),
            "unexpected rejection: {text}"
        );
    }
    for text in [
        "Bewerbung als Project Controller bei Pilatus in Stans.",
        "bewerbung als Strategy & Portfolio Manager.",
        "Applying as Machine Learning Engineer at Destinus in Zurich, CH.",
        "I am applying as Intern in the Lufthansa Group Digital Hangar.",
        "\"Bewerbung als Training System Engineer bei Rheinmetall.\"",
    ] {
        let error = reject_formula_opening(text, "here")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("must not open with a formula application phrase"),
            "unexpected pass: {text} ({error})"
        );
    }
}

#[test]
fn hyphen_marks_are_highlighted_for_author_judgment() {
    assert!(hyphen_advisories("plain words", "here").is_empty());
    let advisories = hyphen_advisories("RAG-Systeme und Cloud-Ökonomie", "here");
    assert_eq!(advisories.len(), 2, "unexpected advisories: {advisories:?}");
    assert!(advisories[0].contains("here"));
    assert!(advisories[0].contains("RAG-Systeme"));
    assert!(advisories[1].contains("Cloud-Ökonomie"));
    assert!(advisories[1].contains("author judgment"));
}

#[test]
fn opportunity_walk_collects_hyphen_advisories() {
    let mut draft = application(&[3, 5, 5, 5, 5, 3]);
    draft["cv"]["summary"] = json!("RAG-Systeme in der Praxis.");
    draft["cl"]["highlights"][0] = json!("AI-Plattformen");
    let advisories = opportunity_hyphen_advisories(&draft, "opportunities/fixture/lead");
    assert!(
        advisories
            .iter()
            .any(|entry| entry.contains("cv.summary") && entry.contains("RAG-Systeme")),
        "unexpected advisories: {advisories:?}"
    );
    assert!(
        advisories
            .iter()
            .any(|entry| entry.contains("cl.highlights[1]") && entry.contains("AI-Plattformen")),
        "unexpected advisories: {advisories:?}"
    );
}

#[test]
fn unknown_fields_are_rejected() {
    let mut draft = application(&[3, 5, 5, 5, 5, 3]);
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
    assert!(warning.contains("formal salutation"));
    assert!(recipient_salutation_warning("fixture", "Dr. Jane Doe").is_none());
    assert!(recipient_salutation_warning("fixture", "   ").is_some());
}

#[test]
fn salutation_override_is_optional_but_must_be_a_single_line() {
    // Absent or empty falls back to the name-derived salutation.
    let draft = application(&[3, 5, 5, 5, 5, 3]);
    validate_record(&workspace(), &draft, "fixture", true).unwrap();

    let mut explicit = draft.clone();
    explicit["job"]["cl_recipient"]["salutation_override"] =
        json!("Sehr geehrte Frau Schmidt Meyer");
    validate_record(&workspace(), &explicit, "fixture", true).unwrap();

    let mut empty = draft.clone();
    empty["job"]["cl_recipient"]["salutation_override"] = json!("");
    validate_record(&workspace(), &empty, "fixture", true).unwrap();

    let mut typed = draft.clone();
    typed["job"]["cl_recipient"]["salutation_override"] = json!(42);
    let error = validate_record(&workspace(), &typed, "fixture", true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("salutation_override must be a string"));

    let mut multiline = draft.clone();
    multiline["job"]["cl_recipient"]["salutation_override"] =
        json!("Sehr geehrte Frau Meyer\nSehr geehrte Frau Schmidt");
    let error = validate_record(&workspace(), &multiline, "fixture", true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("must be a single line"));
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
    assert_eq!(cv_leaves(&workspace).unwrap().len(), 16);
    assert_eq!(cl_leaves(&workspace).unwrap().len(), 4);
    let documents = crate::render::cvl_specs(&workspace).unwrap();
    assert_eq!(documents.len(), 36);
    assert_eq!(
        documents
            .iter()
            .map(|spec| spec.expected_pages)
            .sum::<usize>(),
        84
    );
    let cv = cv_leaves(&workspace)
        .unwrap()
        .into_iter()
        .filter(|leaf| leaf.style == "harvard")
        .collect::<Vec<_>>();
    assert_eq!(cv.len(), 8);
    for (substyle, locale) in [
        ("standard", "de-ch"),
        ("standard", "en-ch"),
        ("compact", "de-ch"),
        ("compact", "en-ch"),
        ("aligned", "de-ch"),
        ("aligned", "en-ch"),
        ("d-plus", "de-ch"),
        ("d-plus", "en-ch"),
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
    let cluster = cv_leaves(&workspace)
        .unwrap()
        .into_iter()
        .filter(|leaf| leaf.style == "cluster")
        .collect::<Vec<_>>();
    assert_eq!(cluster.len(), 8);
    for substyle in ["standard", "middle-three", "middle-three-spaced", "d-plus"] {
        for locale in ["de-ch", "en-ch"] {
            let leaf = cluster
                .iter()
                .find(|leaf| leaf.substyle == substyle && leaf.locale == locale)
                .unwrap_or_else(|| panic!("missing cluster leaf {substyle} {locale}"));
            assert!(leaf.content().is_file());
            assert!(leaf.strings().is_file());
            assert!(leaf.adapter().is_file());
            assert!(leaf.substyle_file().is_file());
        }
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
