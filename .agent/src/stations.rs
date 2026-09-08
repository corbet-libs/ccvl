use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail, ensure};
use regex::Regex;
use serde_json::Value;

use crate::workspace::Workspace;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assessment {
    pub page_counts: BTreeMap<u8, usize>,
    pub unassigned: usize,
    pub unresolved_assigned: Vec<String>,
    pub experience_candidates: Vec<String>,
    pub problems: Vec<String>,
}

impl Assessment {
    #[must_use]
    pub fn ready(&self) -> bool {
        self.problems.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceLayout {
    pub station_ids: BTreeMap<u8, Vec<String>>,
    pub project_ids: Vec<String>,
    pub competency_groups: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
struct SourceEntry {
    id: String,
    bullets: usize,
    offset: usize,
}

pub fn load_plan(workspace: &Workspace, path: &Path) -> Result<Value> {
    workspace.read_toml_value(workspace.relative(path)?)
}

pub fn assess(workspace: &Workspace, document: &Value, location: &str) -> Result<Assessment> {
    let rules = crate::application::document_contract(workspace, "cv")?;
    assess_with_contract(document, location, &rules)
}

fn assess_with_contract(document: &Value, location: &str, contract: &Value) -> Result<Assessment> {
    validate_semantics(document, location)?;
    let rules = contract.get("layout_contract");
    let stations = document
        .get("stations")
        .and_then(Value::as_array)
        .context("station plan has no stations")?;
    let mut page_counts = BTreeMap::from([(1, 0), (2, 0)]);
    let mut unresolved_assigned = Vec::new();
    let mut experience_candidates = Vec::new();
    let mut unassigned = 0;
    for station in stations {
        let id = string(station, "id")?;
        let page = station
            .get("page")
            .and_then(Value::as_u64)
            .map(u8::try_from)
            .transpose()?;
        let verified = station.get("status").and_then(Value::as_str) == Some("verified");
        if let Some(page) = page {
            if verified {
                *page_counts.entry(page).or_default() += 1;
            } else {
                unresolved_assigned.push(id.to_owned());
            }
        } else {
            unassigned += 1;
        }
        if station.get("experience_eligible").and_then(Value::as_bool) == Some(true)
            && page != Some(1)
        {
            experience_candidates.push(id.to_owned());
        }
    }
    let mut problems = Vec::new();
    if let Some(rules) = rules {
        for page in [1_u8, 2] {
            let bounds = rules
                .pointer(&format!("/page_{page}/entries"))
                .context("missing station page contract")?;
            let (minimum, maximum) = if let Some(exact) = bounds.as_u64() {
                (exact, exact)
            } else {
                (
                    bounds
                        .get("minimum")
                        .and_then(Value::as_u64)
                        .context("missing minimum")?,
                    bounds
                        .get("maximum")
                        .and_then(Value::as_u64)
                        .context("missing maximum")?,
                )
            };
            let count = page_counts[&page];
            if count < usize::try_from(minimum)? {
                problems.push(format!(
                    "page {page} is underfilled: {count} stations; minimum {minimum}"
                ));
            } else if count > usize::try_from(maximum)? {
                problems.push(format!(
                    "page {page} is overcrowded: {count} stations; maximum {maximum}"
                ));
            }
        }
    }
    if !unresolved_assigned.is_empty() {
        problems.push(format!(
            "assigned stations are not verified: {}",
            unresolved_assigned.join(", ")
        ));
    }
    Ok(Assessment {
        page_counts,
        unassigned,
        unresolved_assigned,
        experience_candidates,
        problems,
    })
}

pub fn validate_interview(workspace: &Workspace, require_ready: bool) -> Result<Assessment> {
    validate_style(
        workspace,
        &crate::styles::default_style(workspace, "cv")?,
        require_ready,
    )
}

/// A style opts into station allocation and source-marker checks through its
/// own contract. Other renderers need no station page structure or source files.
pub fn validate_style(
    workspace: &Workspace,
    style: &str,
    require_ready: bool,
) -> Result<Assessment> {
    let document = load_plan(workspace, &workspace.path("interview/stations.toml"))?;
    let contract = crate::styles::contract(workspace, "cv", style)?;
    let assessment = assess_with_contract(&document, "interview/stations.toml", &contract)?;
    if require_ready && !assessment.ready() {
        bail!(
            "{}. Run ccvl profile-status and continue the profile interview",
            assessment.problems.join("; ")
        );
    }
    if let Some(files) = contract.get("source_files").and_then(Value::as_array) {
        let root = crate::styles::root(workspace, "cv")?.join(style);
        let mut baseline: Option<SourceLayout> = None;
        for file in files {
            let path = workspace.existing_inside(
                root.join(file.as_str().context("source_files must contain paths")?),
            )?;
            let layout = verify_source_counts(workspace, &document, &path)?;
            if let Some(baseline) = &baseline {
                ensure!(
                    layout.project_ids == baseline.project_ids,
                    "CV locales use different page-3 project IDs or ordering"
                );
                ensure!(
                    layout.competency_groups == baseline.competency_groups,
                    "CV locales use different page-4 competency IDs, grouping, or ordering"
                );
            } else {
                baseline = Some(layout);
            }
        }
    }
    Ok(assessment)
}

/// Validate the opt-in four-page station-marker source format used by Harvard.
pub fn validate_typst_layout(workspace: &Workspace, path: &Path) -> Result<SourceLayout> {
    let source = fs::read_to_string(path)?;
    let pages = source.split("#cv-pagebreak()").collect::<Vec<_>>();
    ensure!(
        pages.len() >= 4,
        "{}: expected at least 4 CV page segments, found {}",
        workspace.relative(path)?.display(),
        pages.len()
    );
    let station_one = marked_entries(pages[0], "station")?;
    let station_two = marked_entries(pages[1], "station")?;
    ensure!(
        (6..=8).contains(&station_one.len()),
        "{}: page 1 has {} full entries; allowed 6–8",
        workspace.relative(path)?.display(),
        station_one.len()
    );
    require_entries(workspace, path, 2, &station_two, 10, 2, "stations")?;
    let projects = marked_entries(pages[2], "project")?;
    require_entries(workspace, path, 3, &projects, 10, 2, "projects")?;
    let competencies = marked_entries(pages[3], "competency")?;
    require_entries(workspace, path, 4, &competencies, 9, 3, "competency blocks")?;

    let mut owners = HashMap::new();
    for (page, entries) in [
        (1, &station_one),
        (2, &station_two),
        (3, &projects),
        (4, &competencies),
    ] {
        for entry in entries {
            if let Some(previous) = owners.insert(entry.id.clone(), page) {
                bail!(
                    "{}: entry marker {:?} appears on pages {previous} and {page}; every visible entry needs one unique owner",
                    workspace.relative(path)?.display(),
                    entry.id
                );
            }
        }
    }
    let group_re = Regex::new(r"(?m)^\s*#cv-spacious-heading\[")?;
    let groups = group_re.find_iter(pages[3]).collect::<Vec<_>>();
    ensure!(
        groups.len() == 3,
        "{}: page 4 has {} competency groups; exactly 3 required",
        workspace.relative(path)?.display(),
        groups.len()
    );
    let mut competency_groups = Vec::new();
    for (index, group) in groups.iter().enumerate() {
        let end = groups
            .get(index + 1)
            .map_or(pages[3].len(), regex::Match::start);
        let ids = competencies
            .iter()
            .filter(|entry| entry.offset > group.start() && entry.offset < end)
            .map(|entry| entry.id.clone())
            .collect::<Vec<_>>();
        ensure!(
            ids.len() == 3,
            "{}: page 4 competency group {} has {} blocks; exactly 3 required",
            workspace.relative(path)?.display(),
            index + 1,
            ids.len()
        );
        competency_groups.push(ids);
    }
    Ok(SourceLayout {
        station_ids: BTreeMap::from([
            (1, station_one.into_iter().map(|entry| entry.id).collect()),
            (2, station_two.into_iter().map(|entry| entry.id).collect()),
        ]),
        project_ids: projects.into_iter().map(|entry| entry.id).collect(),
        competency_groups,
    })
}

pub fn format_report(workspace: &Workspace, assessment: &Assessment) -> Result<String> {
    let rules = crate::application::document_contract(workspace, "cv")?;
    if rules.get("layout_contract").is_none() {
        return Ok(format!(
            "CV station plan: {}\nNo station layout required by the selected style.\nUnassigned candidates: {}",
            if assessment.ready() {
                "READY"
            } else {
                "NOT READY"
            },
            assessment.unassigned
        ));
    }
    let rules = rules
        .pointer("/layout_contract")
        .context("missing CV layout contract")?;
    let state = if assessment.ready() {
        "READY"
    } else {
        "NOT READY"
    };
    let mut lines = vec![
        format!("CV station plan: {state}"),
        format!(
            "Page 1: {} stations (allowed 6–8; target 7)",
            assessment.page_counts[&1]
        ),
        format!(
            "Page 2: {} stations (fixed 10; 2 bullets each)",
            assessment.page_counts[&2]
        ),
        "Page 3: fixed 10 projects; 2 bullets each".to_owned(),
        "Page 4: fixed 3×3 competency blocks; 3 keyword lines each".to_owned(),
        format!("Unassigned candidates: {}", assessment.unassigned),
    ];
    if !assessment.problems.is_empty() {
        lines.push("Problems:".to_owned());
        lines.extend(
            assessment
                .problems
                .iter()
                .map(|problem| format!("- {problem}")),
        );
    }
    if assessment.page_counts[&1] < 6 {
        lines.push("Next: ask for more experience and revisit paid, unpaid, independent, project, research, leadership, repair, teaching, and community work. Convert substantial work into a truthful experience station; never invent employment.".to_owned());
        if !assessment.experience_candidates.is_empty() {
            lines.push(format!(
                "Existing page-1 candidates to assess or move, never duplicate: {}",
                assessment.experience_candidates.join(", ")
            ));
        }
    }
    if assessment.page_counts[&2] < 10 {
        lines.push("Next: ask about education, continuing development, credentials, research, publications, awards, communities, volunteering, and personal responsibility until page 2 has exactly ten stations.".to_owned());
    }
    let _ = rules;
    Ok(lines.join("\n"))
}

fn validate_semantics(document: &Value, location: &str) -> Result<()> {
    let stations = document
        .get("stations")
        .and_then(Value::as_array)
        .context("station plan has no stations")?;
    if !stations.is_empty() {
        ensure!(
            !document
                .get("updated")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .is_empty(),
            "{location}.updated: a non-empty station plan needs a review date"
        );
    }
    let mut station_ids = BTreeSet::new();
    let mut fact_owners = HashMap::new();
    for (index, station) in stations.iter().enumerate() {
        let item = format!("{location}.stations[{}]", index + 1);
        let id = string(station, "id")?;
        ensure!(
            station_ids.insert(id),
            "{item}.id: duplicate station id {id:?}"
        );
        let page = station.get("page").and_then(Value::as_u64);
        let section = string(station, "section")?;
        ensure!(
            page.is_some() != section.trim().is_empty(),
            "{item}: page and section must be assigned together"
        );
        if page == Some(1) {
            ensure!(
                section == "experience"
                    && station.get("experience_eligible").and_then(Value::as_bool) == Some(true),
                "{item}: page 1 accepts only truthfully experience-eligible stations in section 'experience'"
            );
        }
        if station.get("status").and_then(Value::as_str) == Some("verified") {
            ensure!(
                !string(station, "label")?.trim().is_empty(),
                "{item}.label: a verified station needs a label"
            );
            ensure!(
                !string(station, "anchor")?.trim().is_empty(),
                "{item}.anchor: a verified station needs context or a period"
            );
            ensure!(
                !station
                    .get("facts")
                    .and_then(Value::as_array)
                    .is_none_or(Vec::is_empty),
                "{item}.facts: a verified station needs at least one fact"
            );
            ensure!(
                !station
                    .get("source_refs")
                    .and_then(Value::as_array)
                    .is_none_or(Vec::is_empty),
                "{item}.source_refs: a verified station needs provenance"
            );
        }
        for (fact_index, fact) in station
            .get("facts")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let fact_id = string(fact, "id")?;
            ensure!(
                !string(fact, "text")?.trim().is_empty(),
                "{item}.facts[{}].text: fact text cannot be empty",
                fact_index + 1
            );
            if let Some(owner) = fact_owners.insert(fact_id, id) {
                bail!(
                    "{item}.facts[{}].id: fact {fact_id:?} already belongs to station {owner:?}; MECE assignment requires one owner",
                    fact_index + 1
                );
            }
        }
    }
    Ok(())
}

fn verify_source_counts(
    workspace: &Workspace,
    document: &Value,
    path: &Path,
) -> Result<SourceLayout> {
    let layout = validate_typst_layout(workspace, path)?;
    let stations = document
        .get("stations")
        .and_then(Value::as_array)
        .context("station plan has no stations")?;
    let expected = [1_u8, 2]
        .into_iter()
        .map(|page| {
            let ids = stations
                .iter()
                .filter(|station| {
                    station.get("page").and_then(Value::as_u64) == Some(u64::from(page))
                        && station.get("status").and_then(Value::as_str) == Some("verified")
                })
                .filter_map(|station| station.get("id").and_then(Value::as_str).map(str::to_owned))
                .collect::<Vec<_>>();
            (page, ids)
        })
        .collect::<BTreeMap<_, _>>();
    ensure!(
        layout.station_ids == expected,
        "{}: station source IDs differ from the plan",
        workspace.relative(path)?.display()
    );
    Ok(layout)
}

fn marked_entries(source: &str, marker: &str) -> Result<Vec<SourceEntry>> {
    let marker_re = Regex::new(&format!(
        r"(?m)^\s*// ccvl-{marker}: ([a-z0-9]+(?:[-_][a-z0-9]+)*)\s*\n\s*#cv-h\["
    ))?;
    let heading_re = Regex::new(r"(?m)^\s*#cv-h\[")?;
    let bullet_re = Regex::new(r"(?m)^\s*#cv-b\[")?;
    let matches = marker_re.captures_iter(source).collect::<Vec<_>>();
    ensure!(
        heading_re.find_iter(source).count() == matches.len(),
        "full entries and ccvl markers differ"
    );
    Ok(matches
        .iter()
        .enumerate()
        .map(|(index, capture)| {
            let whole = capture.get(0).expect("whole capture");
            let end = matches
                .get(index + 1)
                .and_then(|next| next.get(0))
                .map_or(source.len(), |item| item.start());
            SourceEntry {
                id: capture[1].to_owned(),
                bullets: bullet_re.find_iter(&source[whole.end()..end]).count(),
                offset: whole.start(),
            }
        })
        .collect())
}

fn require_entries(
    workspace: &Workspace,
    path: &Path,
    page: u8,
    entries: &[SourceEntry],
    count: usize,
    bullets: usize,
    label: &str,
) -> Result<()> {
    ensure!(
        entries.len() == count,
        "{}: page {page} has {} full entries; exactly {count} required",
        workspace.relative(path)?.display(),
        entries.len()
    );
    let invalid = entries
        .iter()
        .filter(|entry| entry.bullets != bullets)
        .map(|entry| format!("{}={}", entry.id, entry.bullets))
        .collect::<Vec<_>>();
    ensure!(
        invalid.is_empty(),
        "{}: page {page} {label} must each have exactly {bullets} bullets; {}",
        workspace.relative(path)?.display(),
        invalid.join(", ")
    );
    Ok(())
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("missing string field {key}"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::tempdir_in;

    use super::*;

    fn workspace() -> Workspace {
        Workspace::at(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    fn station(index: usize, page: Option<u8>, status: &str, experience: bool) -> Value {
        json!({
            "id": format!("station-{index}"),
            "label": format!("Station {index}"),
            "anchor": "Example context | 2020–2021",
            "kind": if experience { "employment" } else { "education" },
            "status": status,
            "page": page,
            "section": match page {
                Some(1) => "experience",
                Some(2) => "education",
                _ => "",
            },
            "experience_eligible": experience,
            "facts": [{"id": format!("fact-{index}"), "text": format!("Fact {index}")}],
            "source_refs": ["user-confirmed:2026-09-03"]
        })
    }

    fn plan(page_one: usize, page_two: usize) -> Value {
        let mut stations = (1..=page_one)
            .map(|index| station(index, Some(1), "verified", true))
            .collect::<Vec<_>>();
        stations
            .extend((1..=page_two).map(|index| station(100 + index, Some(2), "verified", false)));
        json!({"schema_version": 1, "updated": "2026-09-03", "stations": stations})
    }

    fn source_entry(marker: &str, identifier: &str, bullets: usize) -> String {
        let mut content = vec![
            format!("// ccvl-{marker}: {identifier}"),
            format!("#cv-h[{identifier}]"),
        ];
        content.extend((0..bullets).map(|index| format!("#cv-b[Bullet {index}]")));
        content.join("\n")
    }

    fn source_document(
        page_two_entries: usize,
        page_two_bullets: usize,
        projects: usize,
        project_bullets: usize,
        competency_groups: &[usize],
        competency_bullets: usize,
    ) -> String {
        let page_one = (0..6)
            .map(|index| source_entry("station", &format!("experience-{index}"), 1))
            .collect::<Vec<_>>()
            .join("\n");
        let page_two = (0..page_two_entries)
            .map(|index| source_entry("station", &format!("support-{index}"), page_two_bullets))
            .collect::<Vec<_>>()
            .join("\n");
        let page_three = (0..projects)
            .map(|index| source_entry("project", &format!("project-{index}"), project_bullets))
            .collect::<Vec<_>>()
            .join("\n");
        let mut competency_index = 0;
        let groups = competency_groups
            .iter()
            .enumerate()
            .map(|(group_index, size)| {
                let mut entries = vec![format!("#cv-spacious-heading[Group {group_index}]")];
                for _ in 0..*size {
                    entries.push(source_entry(
                        "competency",
                        &format!("competency-{competency_index}"),
                        competency_bullets,
                    ));
                    competency_index += 1;
                }
                entries.join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "{page_one}\n#cv-pagebreak()\n{page_two}\n#cv-pagebreak()\n{page_three}\n#cv-pagebreak()\n{groups}"
        )
    }

    fn validate_source(source: &str) -> Result<SourceLayout> {
        let workspace = workspace();
        let directory = tempdir_in(workspace.root()).unwrap();
        let path = directory.path().join("cv.typ");
        fs::write(&path, source).unwrap();
        validate_typst_layout(&workspace, &path)
    }

    #[test]
    fn six_to_eight_and_exactly_ten_are_ready() {
        let workspace = workspace();
        for page_one in [6, 7, 8] {
            assert!(
                assess(&workspace, &plan(page_one, 10), "fixture")
                    .unwrap()
                    .ready()
            );
        }
    }

    #[test]
    fn underfilled_and_overcrowded_pages_require_iteration() {
        let workspace = workspace();
        for (page_one, page_two, phrase) in [
            (5, 10, "page 1 is underfilled"),
            (9, 10, "page 1 is overcrowded"),
            (7, 9, "page 2 is underfilled"),
            (7, 11, "page 2 is overcrowded"),
        ] {
            let result = assess(&workspace, &plan(page_one, page_two), "fixture").unwrap();
            assert!(!result.ready());
            assert!(
                result
                    .problems
                    .iter()
                    .any(|problem| problem.contains(phrase))
            );
        }
    }

    #[test]
    fn only_verified_assigned_stations_count() {
        let workspace = workspace();
        let mut document = plan(6, 10);
        document["stations"][0]["status"] = json!("unverified");
        let result = assess(&workspace, &document, "fixture").unwrap();
        assert_eq!(result.page_counts[&1], 5);
        assert_eq!(result.unresolved_assigned, ["station-1"]);
        assert!(!result.ready());
    }

    #[test]
    fn unassigned_experience_candidates_are_reported() {
        let workspace = workspace();
        let mut document = plan(5, 10);
        document["stations"]
            .as_array_mut()
            .unwrap()
            .push(station(999, None, "verified", true));
        let result = assess(&workspace, &document, "fixture").unwrap();
        assert_eq!(result.experience_candidates, ["station-999"]);
        assert!(
            format_report(&workspace, &result)
                .unwrap()
                .contains("station-999")
        );
    }

    #[test]
    fn facts_have_one_owner_and_page_section_move_together() {
        let workspace = workspace();
        let mut duplicate = plan(6, 10);
        duplicate["stations"][1]["facts"][0]["id"] = json!("fact-1");
        let error = assess(&workspace, &duplicate, "fixture")
            .unwrap_err()
            .to_string();
        assert!(error.contains("MECE assignment requires one owner"));

        let mut missing_section = plan(6, 10);
        missing_section["stations"][0]["section"] = json!("");
        let error = assess(&workspace, &missing_section, "fixture")
            .unwrap_err()
            .to_string();
        assert!(error.contains("page and section must be assigned together"));
    }

    #[test]
    fn compact_headings_do_not_count_and_full_entries_need_markers() {
        let source = "// ccvl-station: one\n#cv-h[One]\n#cv-hu[Compact]\n";
        assert_eq!(marked_entries(source, "station").unwrap().len(), 1);
        assert!(marked_entries("#cv-h[Unmarked]\n", "station").is_err());
    }

    #[test]
    fn fixed_four_page_layout_is_accepted() {
        let layout = validate_source(&source_document(10, 2, 10, 2, &[3, 3, 3], 3)).unwrap();
        assert_eq!(layout.station_ids[&2].len(), 10);
        assert_eq!(layout.project_ids.len(), 10);
        assert_eq!(
            layout
                .competency_groups
                .iter()
                .map(Vec::len)
                .collect::<Vec<_>>(),
            [3, 3, 3]
        );
    }

    #[test]
    fn page_two_and_three_counts_and_bullets_are_fixed() {
        for (source, expected) in [
            (
                source_document(9, 2, 10, 2, &[3, 3, 3], 3),
                "page 2 has 9 full entries; exactly 10 required",
            ),
            (
                source_document(11, 2, 10, 2, &[3, 3, 3], 3),
                "page 2 has 11 full entries; exactly 10 required",
            ),
            (
                source_document(10, 1, 10, 2, &[3, 3, 3], 3),
                "page 2 stations must each have exactly 2 bullets",
            ),
            (
                source_document(10, 2, 9, 2, &[3, 3, 3], 3),
                "page 3 has 9 full entries; exactly 10 required",
            ),
            (
                source_document(10, 2, 10, 3, &[3, 3, 3], 3),
                "page 3 projects must each have exactly 2 bullets",
            ),
        ] {
            let error = validate_source(&source).unwrap_err().to_string();
            assert!(error.contains(expected), "unexpected error: {error}");
        }
    }

    #[test]
    fn page_four_is_three_groups_of_three_three_bullet_blocks() {
        for (source, expected) in [
            (
                source_document(10, 2, 10, 2, &[3, 3], 3),
                "page 4 has 6 full entries; exactly 9 required",
            ),
            (
                source_document(10, 2, 10, 2, &[2, 4, 3], 3),
                "competency group 1 has 2 blocks; exactly 3 required",
            ),
            (
                source_document(10, 2, 10, 2, &[3, 3, 3], 2),
                "page 4 competency blocks must each have exactly 3 bullets",
            ),
        ] {
            let error = validate_source(&source).unwrap_err().to_string();
            assert!(error.contains(expected), "unexpected error: {error}");
        }
    }

    #[test]
    fn entry_markers_cannot_be_reused_across_pages() {
        let source = source_document(10, 2, 10, 2, &[3, 3, 3], 3).replace(
            "// ccvl-project: project-0",
            "// ccvl-project: experience-0",
        );
        let error = validate_source(&source).unwrap_err().to_string();
        assert!(error.contains("every visible entry needs one unique owner"));
    }

    #[test]
    fn checked_in_interview_plan_matches_both_locales() {
        let result = validate_interview(&workspace(), true).unwrap();
        assert_eq!(result.page_counts, BTreeMap::from([(1, 8), (2, 10)]));
    }
}
