# AIDA / Harvard cover-letter contract

In ccvl, AIDA and the 26-body-line structure with its defined paragraphs mean
the same contract. Harvard implements this contract; its AIDA argument roles
and measured paragraph structure belong together. Independent document
structures keep their own contracts.
Persuasive embellishment within truthful meaning and MECE argument coverage
apply to every user and style, independently of this layout.

A Harvard cover letter contains exactly six body paragraphs and five one-line
highlights. The highlights sit between paragraphs 3 and 4. Paragraph 1 opens in
exactly three lines; paragraph 6 mirrors it with a warm three-line close. The
four central paragraphs carry the evidence and target case in exactly 20 lines:
five lines each.

`cvl/cl/harvard/contract.toml` is the machine-readable source of truth. Each paragraph definition
contains its number, semantic role, purpose, and line bounds, making the
contract self-describing for both people and agents.

## Paragraph map

The contract's `editorial` and `aida_stage` fields associate AIDA with each
paragraph's role and line budget. The compiler enforces measurable structure;
the author and independent reviewer also enforce the argument roles.

| Block | Role | AIDA stage | Purpose | Lines |
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
and reader benefit before remaining CV material; strengthen their framing while
preserving qualifications needed for factual meaning.

Give each block a distinct contribution: paragraph 3 complements paragraph 2,
paragraph 4 draws out their value, and paragraph 5 makes the audience-specific
connection. Rephrasing the same achievement in all four does not satisfy MECE.
Across the letter and supporting CV, cover the material supported priorities
and account for evidence gaps in preparation without inventing qualifications.

The valediction and signature follow paragraph 6 and do not count as a seventh
paragraph.

## Paragraph 6: warm close (Action)

Paragraph 6 has exactly three lines — part of the `3 | 5 | 5 | 5 | 5 | 3`
budgets above, never two or four:

| Line | Content | Function |
|---|---|---|
| 1 | Core tasks of the posting, named in its terms | Shows role understanding |
| 2 | Continuation framing the contribution at the employer and place | Locates the contribution |
| 3 | Closing sentence opening with “Ich freue mich” | Fixed opener, no variation |

Three hard conditions for German opportunity letters (the general
showcase keeps author judgment):

1. Always exactly 3 lines; the contract already enforces this.
2. Line 1 names core tasks from `posting.md` in concrete posting terms —
   actual task nouns, never generic placeholders for challenges.
3. Line 3 opens with “Ich freue mich” and points at the contribution to
   the team, with slots for the supported role and context:
   `… als XXX … ZZZ …` (XXX = the contribution role, ZZZ = the project
   or company context). Only the opener prefix is enforced, never a full
   verbatim sentence. Opportunity records fail validation when the final
   line does not open with the fixed opener.

Forbidden in the close: team-fit formulas of the “fit im Team” pattern
and thanks-for-consideration formulas of the “Dank für Ihre
Überlegungen” pattern (matched case-insensitively; opportunity records
fail validation). Never invite the reader to send test questions or a
problem statement in exchange for a solution — in German (“Schicken Sie
mir …”) or in English (“send me … questions”, “problem statement”).
Opportunity records fail validation on such invitations in every language.

## English opportunity letters

The same six-paragraph contract applies: paragraph 6 has exactly three
lines naming the posting's core tasks first and closing on the
contribution to the team, in the house voice (“I would welcome …”, as in
the general showcase). The fixed-opener prefix is enforced for German
records only; English closings stay reviewer judgment until the first
targeted English letter pins the house opener with the user. The
task-invitation ban and the line count bind English records mechanically.

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
concrete evidence. Give the five highlights distinct selection dimensions
supported by the candidate's record. They may concisely signpost fuller prose;
five paraphrases of one selling point do not supply distinct coverage. Account
for material unsupported dimensions in preparation, without invented claims or
compulsory gap apologies in the panel.

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

Reusable locale resolution/canonicalization, spelling, salutations, closings
and dates belong to cletter and its family. ccvl consumes their public helpers
and passes explicit user/style choices. Missing shared behavior belongs upstream,
without duplicate locale tables or algorithms in ccvl. The Harvard style retains
its explicit English override:
`en-ch` uses a title-less named form (`Dear Doe,`) and the generic
`Dear Hiring Manager,`. Such explicit style/user choices are not replaced by
locale inference. See [the correspondence integration](../typst/letter/README.md)
for the consumed helpers; do not copy a competing national-norm table here.

Use lowercase locale identifiers. Apply library orthographic transformations to
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
