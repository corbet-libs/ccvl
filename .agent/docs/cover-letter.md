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

Every body line is explicit and measured before justification. Its natural
glyph width must cover at least 75% of the available measure, targets 90%,
and may never exceed 100% — except a paragraph's closing line, which shares
the uniform closing-line maximum of 102% with the CV Summary. This prevents
a short stranded line from being hidden by extreme word spacing.
Non-final lines are then justified; the final line stays
ragged but remains subject to the same natural-width floor.
Justification is explicit per break — Typst's plain `linebreak()` always
creates an unjustified break, so `measured-paragraph` passes
`justify: true` to every inter-line break while the closing line, which
carries no break, stays left-bound by construction.

Each paragraph is an unbreakable Typst block. Manual line breaks, the one-page
contract, and the 75% floor together permit zero widows, orphans, wrapped lines,
or sparse paragraph endings. Underfill and overflow both fail the draft and
prompt another evidence-backed rewrite.

## Five highlights

The highlights form the visual and argumentative hinge between evidence and
application. Each is exactly one measured line with a recognisable heading and
concrete evidence. Together they cover the target's main selection dimensions
without duplicating the prose verbatim.

## Vertical rhythm

The renderer places the header, subject, salutation, six paragraphs,
highlights, and valediction/signature in one full-height A4 grid. Ten equal
flexible gaps distribute the remaining height instead of collecting it in empty
slabs. A gap targets 20 pt and must remain within 12–30 pt. The highlight centre
targets 56% of usable page height and must remain within 50–60%. A sparse or
over-compressed page therefore fails deterministically even when every
individual line fits.

Both values come from the actual rendered Typst boxes. Accepted line-budget
variation may move the highlights slightly away from the geometric centre
without abandoning the composition. The recipient record in
`application.toml` is data-only provenance and is not printed; only the
salutation uses the recipient name.

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
