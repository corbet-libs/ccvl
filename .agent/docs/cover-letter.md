# Cover-letter contract

Every ccvl cover letter contains exactly six body paragraphs and five one-line
highlights. The highlights sit between paragraphs 3 and 4. Paragraph 1 opens in
exactly three lines; paragraph 6 mirrors it with a warm three-line close. The
four central paragraphs carry the evidence and target case in exactly 20 lines:
five lines each.

`ccvl.json` is the machine-readable source of truth. Each paragraph definition
contains its number, semantic role, purpose, and line bounds, making the
contract self-describing for both people and agents.

## Paragraph map

| Block | Role | Purpose | Line contract |
|---|---|---|---:|
| Paragraph 1 | Positioning | Name the target and establish immediate fit. | exactly 3 |
| Paragraph 2 | Primary evidence | Prove the strongest relevant experience and results. | exactly 5 |
| Paragraph 3 | Complementary evidence | Add a second capability domain and the career-wide pattern. | exactly 5 |
| Highlights | Evidence index | Surface five selection dimensions without repeating the letter. | exactly 5 × 1 |
| Paragraph 4 | Differentiation | Explain the value created by the combined evidence. | exactly 5 |
| Paragraph 5 | Target fit | Connect that value to the specific organisation and opportunity. | exactly 5 |
| Paragraph 6 | Warm close | Invite a conversation with warmth and low friction. | exactly 3 |

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

Use the same triangular bullet markers as the CV. Keep the five rows inside
the existing shaded panel with its accent border and colour palette. Align
each triangle and its text with the corresponding CV bullet positions. The
panel extends into both side margins and keeps vertical padding around the
rows. Separate the rows with about half a line of whitespace so each highlight
is easy to scan; follow the CV's spacing rhythm throughout the letter.

Each highlight is measured against its actual container before any visual
spacing: minimum 70%, target 82%, maximum 100%. Add useful evidence to a thin
highlight or tighten an overflowing one; never pad it merely to meet the
minimum.

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

## Salutation

The `name` field of `job.cl_recipient` holds the full address form, e.g.
`"Frau Dr. Müller"` or `"Herr Müller"`. Only the honorific, academic titles,
and surname render; first names never appear in a formal salutation. The
rules live in the `cgreet` library
(`https://github.com/corbet-labs/cgreet`, re-exported
for compatibility via `ccvl::application`) and are mirrored for the renderer
in `.agent/typst/application.typ` (`salutation-honorific`,
`salutation-titles`, `salutation-surname`, `de-salutation`); `en-ch`
additionally uses `salutation-last-name` (`"Dr. Jane Doe"` renders
`Dear Doe,`).

German salutations are locale-correct per country norm:

| Locale | Norm | Named | Generic | Comma |
|---|---|---|---|---|
| de-ch | SN 010130 | `Sehr geehrte Frau Dr. Müller` | `Sehr geehrte Damen und Herren` | none; next sentence starts uppercase |
| de-li | SN 010130 (assumed) | same as de-ch | same as de-ch | none |
| de-de | DIN 5008 | `Sehr geehrte Frau Dr. Müller,` | `Sehr geehrte Damen und Herren,` | comma; sentence continues lowercase |
| de-at | DIN 5008 (ÖNORM A 1080 was withdrawn in 2018) | same as de-de | same as de-de | comma |

Notes:

- `Guten Tag` is informal and never used in a formal application; the
  fallback is always `Sehr geehrte Damen und Herren`.
- Abbreviations: the Anrede uses `Herr`, never the accusative `Herrn`
  (which belongs only in the postal address) and never `Hr.`/`Fr.`
  (unhöflich); `Frau` is never abbreviated. `Dr.` stays abbreviated,
  `Prof.` normalises to the spelled-out `Professor`; `Dipl.-Ing.` and
  `Mag.` survive. Protocol keeps only the highest title, so Professor
  suppresses Dr.
- Liechtenstein has no own correspondence norm on record; it renders
  Swiss-style (no comma, `ss` spelling) given the customs and currency
  union and Alemannic usage. Say so explicitly if a FL recipient asks.
- A name without a parsable Herr/Frau honorific (or without a surname)
  falls back to the generic salutation so the letter stays formally safe.
- English (`en-ch`): the named form stays title-less by design (`Dear Doe,`)
  to avoid misgendering from a surname alone; the generic form is
  `Dear Hiring Manager,` (the hiring-process owner, stronger than a team
  address and more current than `Dear Sir or Madam` / `To Whom It May
  Concern`).

The generic fallback stays valid for the target-neutral showcase, but
`ccvl measure` (and `measure-opportunity`) reports an empty recipient — and,
for German records, a name without Herr/Frau — as a non-blocking `WARN`,
and `ccvl check` repeats it without failing. Provide a real address form
such as `"Frau Dr. Müller"` for every tailored opportunity.

## Iteration contract

Run `bash ./ccvl measure` or `.\ccvl.cmd measure`. A hard line, structure, or
layout violation fails. Rewrite with relevant, verified signal and rerun
measurement; never respond by adding filler, inventing a claim, condensing the
type, or weakening the bounds.

The public showcase is target-neutral and describes the named author. A real
application replaces the role, company fit, evidence selection, and invitation
while keeping every claim traceable to the private evidence base.
