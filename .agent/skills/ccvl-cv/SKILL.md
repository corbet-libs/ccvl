---
name: ccvl-cv
description: Write, tailor, render, and verify ccvl CV variants when Summary, experience, projects, competencies, keywords, or page presets change.
---

# Create and revise a ccvl CV

Preserve the Harvard-style hierarchy and make the document legible to both
recruiters and specialists.

## Writing contract

- Use the simplest language that a recruiter can understand while retaining
  terms a domain specialist will recognise.
- Lead with outcome or scale, then ownership and method. Remove filler and
  duplicated meaning.
- Use only verified profile claims. Keywords improve retrieval but do not
  create factual permission.
- Keep capability groups mutually exclusive and collectively useful. Prefer a
  few recognisable terms over keyword stuffing.
- Treat the checked-in showcase as visual design evidence, never as facts or
  reusable wording for a new user. Its personal content is not a template.

## Summary

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
a future second family would bring its own contract. Before polishing or
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
actions: always do both.

Every controlled CV line is measured against a minimum, target, and maximum
fill percentage from `cvl/cv/contract.toml` for its actual Typst container. A sparse or
overflowing line is a failed draft. Add relevant, verified signal or tighten
the wording, then run `ccvl measure` again. Never use filler merely to make a
draft pass.

## Verification

Render every affected locale, substyle, and preset. Require the station gate and requested two-, three-, or
four-page count, inspect every rendered page, and extract the PDF text layer.
Reject clipped content, accidental extra pages, missing glyphs, placeholders,
or any line outside its declared bounds. Run the matching platform `measure`
command until it passes, then run `check` before completion.
