---
name: ccvl-cv
description: Write, tailor, render, and verify ccvl CV variants when Summary, experience, projects, competencies, keywords, or page presets change.
---

# Create and revise a ccvl CV

Resolve `options.cv_style` and `options.cv_substyle`, then read that style’s
metadata and contract under `cvl/cv/<style>/`. Make the document legible to
recruiters and specialists. Harvard is the shipped default; another style may
use entirely different content fields, geometry, fonts and page presets.

Read [editorial guidance](../../docs/editorial.md) for evidence, audience and
register decisions. A general CV follows the stated audience; a targeted CV
answers its archived posting. Do not invent a vacancy for a general document.

## Writing contract

- Use the simplest language that a recruiter can understand while retaining
  terms a domain specialist will recognise.
- Lead with outcome or scale, then ownership and method. Remove filler and
  duplicated meaning.
- Use only verified profile claims. Keywords improve retrieval but do not
  create factual permission. Read original supporting sources rather than
  treating a derived CV as its own evidence. Preserve modeled versus realised
  outcomes, estimates, dates, ownership and independent-work scope.
- Preserve names, exact quotations and explicit user choices. Author locale IDs
  in lowercase; use cletter/family conventions only for appropriate generated
  prose, never a global replacement over evidence or protected text.
- Keep capability groups mutually exclusive and collectively useful. Prefer a
  few recognisable terms over keyword stuffing.
- Treat the checked-in showcase as visual design evidence, never as facts or
  reusable wording for a new user. Its personal content is not a template.

## Summary (Harvard CV)

Maintain the opportunity-independent master below `cvl/`. For every
concrete role, read `cv.summary` from
`opportunities/<organisation-key>/<position-key>/application.toml`. It is
one flowing paragraph that must typeset to exactly five lines — not four,
not six (see `.agent/docs/summary.md`). Density only counsels, except thin
lines, which fail unless the record explicitly sets `cv.allow_thin`. The
paragraph must express:

```text
target profile | differentiation | two evidenced results | value offered
```

The public showcase may combine this formula with an invitation to contact the
author. A real application must be target-specific.

## Fixed layout gate (Harvard CV contract)

These counts are the Harvard family's contract — shared by the `standard`
and `compact` CV substyles, which differ only in whitespace — not engine law:
other styles bring their own contracts. Before polishing or
tailoring, load `interview/stations.toml` and run
`ccvl profile-status --verify-sources`. Page 1 must contain 6–8 full experience
stations. Page 2 contains exactly 10 supporting stations with two bullets each.
Page 3 contains exactly 10 projects or initiatives with two bullets each. Page
4 contains three competency groups, each with three blocks of three keyword
lines. A full entry has its own heading and supporting content. Bullets and
compact standalone lines do not count as entries.

If the gate reports underfill, return to `ccvl-profile` and collect more
material. Prefer converting substantial verified independent work, projects,
research, teaching, leadership, or engagement into truthfully labelled
experience stations. Move facts; never duplicate them across sections.

If any fixed section is overfilled or malformed, the rendered PDF is still a
failed draft. Restore the declared slots exactly: page 2 has 10 two-bullet
stations, page 3 has 10 two-bullet projects, and page 4 has three groups of
three competency blocks with three keyword lines each. Rank, merge coherent
material, move, or leave lower-value stations unassigned, then rerun the
deterministic source gate and continue the collection/allocation loop until
every fixed count passes. Never accept a successful render as proof that extra
content is valid, and never relax the manifest to fit the draft. Restoring the
fixed slots and iterating on the failed gate are two separately required
actions: always do both within the active review run's correction allowance.
An exhausted allowance leaves the draft incomplete, never accepted.

Every controlled CV line is measured against a minimum, target, and maximum
fill percentage from `cvl/cv/harvard/contract.toml` for its actual Typst container. A sparse or
overflowing line is a failed draft. Add relevant, verified signal or tighten
the wording, then run `ccvl measure` again. Never use filler merely to make a
draft pass.

## Verification

Render every affected style, substyle, locale and preset. Enforce its declared
contract (including the station gate for Harvard), requested paper dimensions
and exact page count. Use only papers declared by the style; preserve an
explicit selection without shrinking text to make it fit. Inspect every
rendered page and extract the PDF text layer.
Reject clipped content, accidental extra pages, missing glyphs, placeholders,
or any line outside its declared bounds. Run the matching platform `measure`
command until it passes, then run `check` before completion.


For independent review, follow [the review protocol](../../docs/review.md) and
route the frozen sources, selected rules, extracted text and every page to
`ccvl-review` in fresh context. The critic checks evidence and actual output;
an actor summary and a successful compile cannot replace that review. Keep
errors, missing support and optional preferences distinct and cite findings.

An actor–critic run allows two corrections after the initial candidate. Call
`ccvl review begin-revision <run-dir>` before editing and
`ccvl review prepare-revision <run-dir>` afterwards. Check every allowed
candidate, including a mechanically failed one, before considering another.
Re-review current artifacts for both fixes and new errors. Stale hashes,
unavailable tools or unread pages cannot become a pass; exhausted corrections
remain incomplete, material evidence gaps need evidence, and a material user
choice conflict needs a decision. An optional tone preference cannot erase a
truthful qualification or override the user's chosen wording.
