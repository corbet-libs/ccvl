# Summary contract

Every Summary is one flowing paragraph that must typeset to exactly five
lines — not four, not six. The author writes natural prose; the renderer
wraps it to five explicit lines for measurement. The five lines render as
one justified paragraph with a left-bound closing line, exactly like a
cover-letter paragraph: non-final lines are flush on both sides while the
final line stays ragged.

## The three layers

- **Soll** (contract, `ccvl.json` + record): five lines; density target 97,
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

For a real application, the formula remains but the prose must be rewritten
for the specific opportunity. Keywords may improve retrieval, but they never
turn an unsupported capability into a fact. Use plain language that a
recruiter can understand and a specialist can recognise.

Underfill and overflow past the closing-line maximum fail: add relevant,
verified signal or tighten the wording, then run `bash ./ccvl measure` or
`.\ccvl.cmd measure` again. Never pass measurement by adding filler.
