# Harvard cover-letter contract

The selected style owns a letter's content shape and geometry. This document
describes Harvard; independent styles keep their own contracts. Universal
[evidence and writing guidance](editorial.md) and the optional AIDA recipe are
separate from presentation.

A Harvard cover letter contains exactly six body paragraphs and five one-line
highlights. The highlights sit between paragraphs 3 and 4. Paragraph 1 opens in
exactly three lines; paragraph 6 mirrors it with a warm three-line close. The
four central paragraphs carry the evidence and target case in exactly 20 lines:
five lines each.

`cvl/cl/harvard/contract.toml` is the machine-readable source of truth. Each paragraph definition
contains its number, semantic role, purpose, and line bounds, making the
contract self-describing for both people and agents.

## Paragraph map

When AIDA is selected, Harvard maps it as follows. The contract's
`editorial` and `aida_stage` fields document this optional mapping for agents;
the compiler enforces geometry, not rhetorical quality or recipe selection.

| Block | Role | Optional AIDA stage | Purpose | Lines |
|---|---|---|---|---:|
| Paragraph 1 | Positioning | Attention | Establish the target or general purpose and immediate relevance. | 3 |
| Paragraph 2 | Primary evidence | Interest | Present the strongest relevant experience and results. | 5 |
| Paragraph 3 | Complementary evidence | Interest | Add useful complementary evidence. | 5 |
| Highlights | Evidence index | — | Surface five supported selection dimensions without repeating prose. | 5 × 1 |
| Paragraph 4 | Differentiation | Desire | Explain the useful contribution supported by the combined evidence. | 5 |
| Paragraph 5 | Target fit | Desire | Connect that contribution to this employer or the declared general audience. | 5 |
| Paragraph 6 | Warm close | Action | Invite an appropriate conversation or next step. | 3 |

Do not print the stage names as drafting labels or explain what a paragraph
would contain in a later application. Ordinary uses of those words remain
valid. General/open letters do not need a fictional vacancy. Select evidence
and reader benefit before remaining CV material; preserve real qualifications.


The valediction and signature follow paragraph 6 and do not count as a seventh
paragraph.

## Shared line budgets

The framework is strict: `3 | 5 | 5 | 5 | 5 | 3`, with 20 central lines and
26 body lines overall. There are no flexible lines and no dispreferred-but-valid
totals:

- paragraphs 2–3: exactly 10 lines;
- paragraphs 4–5: exactly 10 lines;
- paragraphs 2–5 together: exactly 20 lines.

## Justification and paragraph integrity

Every body line is explicit. Typst measures its natural width with the actual
font and container before justification adds word spacing. The minimum depends
on where the line sits in its paragraph:

| Line | Minimum natural fill | Target | Maximum natural fill |
|---|---:|---:|---:|
| Non-final body line | 95% | 97% | 100% |
| Paragraph closing line | 75% | 97% | 100% |

The target guides drafting; the minimum and maximum determine whether a line
passes. A non-final line is justified to the full measure. Its 95% minimum
limits the added word spacing and requires substantial content before that
stretch. The paragraph's closing line stays ragged and may end naturally at
75% or more. Every cover-letter body line is capped at 100%; the CV Summary
has its own closing-line policy in `.agent/docs/summary.md`.

For an underfilled line, add relevant, verified evidence or restructure the
paragraph to distribute its argument more evenly. For overflow, tighten the
wording. Repetition, generic praise, and invented claims are never acceptable
ways to reach a width target.

Justification is explicit per break — Typst's plain `linebreak()` always
creates an unjustified break, so `measured-paragraph` passes
`justify: true` to every inter-line break while the closing line, which
carries no break, stays left-bound by construction.

Each paragraph is an unbreakable Typst block. Manual line breaks, the one-page
contract, and the two natural-width floors together permit zero widows,
orphans, wrapped lines, or sparse paragraph endings. Underfill and overflow
both fail the draft and prompt another evidence-backed rewrite.

## Five highlights

The highlights form the visual and argumentative hinge between evidence and
application. Each is exactly one measured line with a recognisable heading and
concrete evidence. Together they cover the target's main selection dimensions
without duplicating the prose verbatim.

The entire highlight panel, including the outer edge of its blue border,
aligns with the body paragraphs' left and right edges. Keep its accent border
and colour palette. (`left-rule` renders the accent as a left border; the
`frame` substyle renders the same panel with a full border and wider padding
— panel chrome only, same contracts.) Place the unchanged CV triangles and highlight text inside
the panel, with 8 pt inner padding and the CV's 10.5 pt marker column. Account
for the stroke width when aligning the visible panel edge. About half a line
of whitespace between rows keeps the five highlights distinct.

Highlight text inherits the CV and body font size of 10.5 pt. Set the category
before the existing `|` separator in bold and the evidence in regular weight.
Use label weight and spacing for hierarchy while keeping the shared font size.
Keep the rows left aligned with natural, ragged endings.

Measure the actual styled text, including its font size and bold category,
against its available inner width after panel padding and the marker column:
minimum 70%, target 82%, maximum 100%.
Ragged endings within these bounds are intentional. Add useful evidence to a
thin highlight or tighten an overflowing one; never pad it or stretch its word
spacing merely to reach the target.

## Vertical rhythm

The renderer places the header, subject, salutation, six paragraphs,
highlights, and valediction/signature in one full-height A4 grid. Ten flexible
gaps distribute the remaining height using style weights for each transition:
tighter spacing joins the header, subject and salutation; paragraph gaps give
the argument room; the largest gaps frame the highlights. The closing stays
close to the final paragraph, and the signature block remains anchored at the
foot of the page. The fixed line counts remain unchanged.

Every rendered gap must remain within 12–30 pt; the 20 pt target guides the
overall rhythm. Validation covers the actual gap range: the gap metric checks
the minimum if it falls below the floor, and otherwise the maximum against the
ceiling. The highlight centre targets 56% of usable page height and must remain
within 50–60%. A sparse or over-compressed page therefore fails even when every
individual line fits.

Gap lengths and the highlight centre come from the actual rendered Typst
boxes. The selected style's spacing and measured block heights may move the
highlights slightly away from the geometric centre within those bounds. The
recipient record in `application.toml` is data-only provenance and is not
printed; only the salutation uses the recipient name.

## Correspondence and protected text

The `name` field of `job.cl_recipient` stores the actual supplied address form,
for example `"Frau Dr. Müller"`. Do not infer a person's honorific or alter an
exact name to satisfy a locale convention. Missing recipient information uses
the supported generic fallback and produces a non-blocking diagnostic; it is
valid for a general/open showcase. Obtain a real address form when available
for a targeted letter, without inventing one.

Shared spelling, salutation and closing conventions belong to cletter and its
family. ccvl's Harvard renderer retains its explicit English override:
`en-ch` uses a title-less named form (`Dear Doe,`) and the generic
`Dear Hiring Manager,`. Such explicit style/user choices are not replaced by
locale inference. See [the correspondence integration](../typst/letter/README.md)
for the consumed helpers; do not copy a competing national-norm table here.

Use lowercase locale identifiers. Apply shared orthographic transformations to
appropriate generated prose only, preserving exact names, quotations, URLs
and source evidence. If a safe boundary is unavailable, diagnose the specific
passage. Register, evidence and relevance remain [ccvl editorial choices](editorial.md),
not a claim that every country requires one tone or argument recipe.

## Iteration contract

Run `bash ./ccvl measure` or `.\ccvl.cmd measure`. A hard line, structure, or
layout violation fails. Rewrite with relevant, verified signal and rerun
measurement; never respond by adding filler, inventing a claim, condensing the
type, or weakening the bounds.

The public showcase is target-neutral and describes the named author. A real
application replaces the role, company fit, evidence selection, and invitation
while keeping every claim traceable to the private evidence base.


Independent review follows [the review protocol](review.md). The initial
candidate has at most two corrections in that run; check the allowance before
editing and check/review every allowed revision before considering another.
Underfill after the allowance, missing evidence or unread pages is an honest
unfinished result, never permission to weaken the contract or claim readiness.
