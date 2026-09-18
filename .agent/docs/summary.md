# Summary contract

Every Summary is one flowing paragraph that must typeset to exactly five
lines — not four, not six. The author writes natural prose; the renderer
wraps it to five explicit lines for measurement. The five lines render as
one justified paragraph with a left-bound closing line, exactly like a
cover-letter paragraph: non-final lines are flush on both sides while the
final line stays ragged.

## The three layers

- **Soll** (contract, `cvl/cv/harvard/contract.toml` + record): five lines; density target 97,
  thin floor 95, closing-line maximum 102.
- **Ist** (one measurement): a single compilation emits per-line metrics.
- **Diagnose** (counsel, never silent): the count rule is hard; density only
  advises or fails narrow cases:
  - exactly five lines, else the build fails;
  - a thin line fails, unless the record sets `cv.allow_thin` explicitly —
    wanted thinness stays visible instead of sneaking past;
  - a Summary closing line may extend past the block edge up to its 102%
    closing-line maximum; past it fails. Cover-letter closing lines have
    their own 100% maximum in `.agent/docs/cover-letter.md`.

The public Summary is both a working example and an invitation to contact
its author. Its closing exposes the adaptation formula:

```text
target profile | differentiation | two evidenced results | value offered
```

## Opening

Never open with a formula application phrase: no „Bewerbung als …",
no „Applying as …", no „I am applying as …" plus job title and place.
The role, company and location already live in the CV header, the
record's `job` block and the cover letter; restating them wastes the
first of five lines on a self-evident fact, and the same sentence frame
across opportunities reads as template slop. Lead with target profile,
differentiation, or the strongest evidence instead and move the target
context organically to the end („… bringe ich in … ein"). Opportunity
records fail validation on formula openings; the general showcase keeps
author judgment.

For a real application, the formula remains but the prose must be rewritten
for the specific opportunity. Keywords may improve retrieval, but they never
turn an unsupported capability into a fact. Use plain language that a
recruiter can understand and a specialist can recognise.

## Grades (Swiss scale)

Swiss grades run 1–6 with 6.0 as the best note (5.5 very good, 5 good,
4.5 satisfactory, 4 pass, below 4 fail) — the inverse of the German scale,
where 1.0 is best. Numeric equivalents follow `cgrade` (modified Bavarian
formula); `ccvl` defines only display, never its own conversion table.
For records aimed at a Swiss audience the CH grade leads, in dot display
with an explicit scale tag: the `6.0 (CH)` pattern (`<grade> (CH)`),
optionally paired with the evidenced German equivalent where the record
already carries it (`<CH> (CH) / <DE> (DE)` pattern). Never label a
German-scale best grade as the best grade for a CH audience, and never
present a top CH grade as a weak or adjacent result in `posting.md`
notes. Scope is display only: convert and label evidenced grades, never
invent new facts.

## Punctuation and enumerations

Summaries are grammatical sentences, never keyword lists. Use correct
German enumeration commas — comma between coordinate items, no comma
before “und”/“oder” in a flat enumeration, subordinate and relative
clauses set off correctly — and embed posting keywords in the sentence
flow with priority instead of appending them as a comma-separated
stuffing list.

Mechanical precision limits: opportunity validation rejects only what is
unambiguous — a space before a comma, German/US thousands grouping
(`1.000`, `1,000`; write `1'000`), comma-decimal CH grades (`6,0 (CH)`;
write `6.0 (CH)`), and a German-scale best label without a `(DE)` tag.
A comma before “und”/“oder” is
correct at a clause boundary, so that distinction stays author and
reviewer judgment, as do enumeration structure, clause commas, and
keyword prioritisation. Review checks punctuation on the rendered text.

Underfill and overflow past the closing-line maximum fail: add relevant,
verified signal or tighten the wording, then run `bash ./ccvl measure` or
`.\ccvl.cmd measure` again. Never pass measurement by adding filler.
