# Todo — ccvl justification + output restructure

## 1. Summary/paragraph justification — fixed and verified

**Root cause (two layers):** Verified against the vendored Typst 0.15.1 sources, not just docs.
- Typst's plain `linebreak()` *always* creates an unjustified break — `par(justify: true)` only affects automatic breaking, which never happens here since all breaks are manual. So every Summary line *and* every cover-paragraph line rendered ragged-left (confirmed in the PDFs: uniform ~2.2 word gaps, ragged right edges).
- Additionally, `block(breakable: false)` with auto width does **not** expand its children (confirmed in `block.rs`: expansion only when size is explicit), so paragraphs shrink-wrapped to their own widest line instead of the column. The Summary only looked right by accident — its over-wide spill box forced full width.

**Changes:** `linebreak(justify: true)` for inter-line breaks in `measured-paragraph` (closing line has no break → stays left-bound); Summary now renders via `measured-paragraph` in both locales; `block(width: 100%, breakable: false)` on cover paragraph wrappers and Summary blocks. Docs (`summary.md`, `cover-letter.md`) updated.

**Verified from the PDFs:** non-final lines flush to the column with stretched gaps, closings left-bound at natural width; `measure`, `build`, `check` all pass. One note: bbox extraction shows ±2pt noise on some justified line edges (unmapped separator advance) — rendering follows Typst's justification to one width per paragraph; not a defect.

## 2. Output restructure — understanding (implement after confirming)

Mapped every touchpoint with two subagents (code + docs/tests/CI).

**Target layout** (`opportunities/<org>/<pos>/`, org/pos structure unchanged):

```text
config.toml                    (renamed from application.toml)
typst/cv.typ, typst/cl.typ     (hand-editable sources, watched)
pdf/cv.pdf, pdf/cl.pdf         (built outputs)
```

**The core inversion:** today `output/*.typ` are *generated, do-not-edit* resolved copies (`emit_resolved_typ`, header literally says "Do not edit: regenerated on every build"). The change makes `typst/` a *source*. Consequences:
- `new-opportunity` must seed `typst/` (locale templates with inputs resolved to `../config.toml`, as starting points, not artifacts).
- `build-opportunity` needs two modes: toml changed → regenerate `typst/` + `pdf/`; only `typst/` changed → rebuild `pdf/` only. **Conflict policy needed:** if both were hand-edited, does a toml build overwrite `typst/` (destroying hand edits) or refuse/warn?
- `watch-opportunity` watches `typst/` (plus toml, shared machinery) and ignores `pdf/`; `format` must now cover `typst/`; `ownership`/`public-check` must learn `pdf/` instead of `output/`; `.gitignore` switches to `/opportunities/*/*/pdf/` with `typst/` tracked.
- Hand-edited `typst/` must still pass the measurement gates on build, or hand edits could silently break the contract (assume yes, enforce mode stays).
- Blast radius: `render.rs`, `cli.rs`, `opportunity.rs`, `application.rs`, `check.rs`, `workspace.rs`, `ownership.rs`, `public.rs`, `format.rs`, `ccvl.json`, `.gitignore`, `justfile`, the scaffold rename, ~15 docs/skills files (including contradictions to rewrite: "do not edit them by hand" in `applications.md`), plus a REUSE licensing gap (`opportunities/**` currently falls through to FSL default). CI itself has no path literals — it flows through `check` — but `check-linux-deep.sh` is `cvl/`-only today.

**Open questions before implementing:**
1. **Scope:** opportunities only, `cvl/<locale>/` keeps `application.toml` + sources + `output/`? (That leaves two naming schemes side by side.)
2. **Conflict policy:** toml edit overwrites hand-edited `typst/` (simplest, typst always derived unless only typst changed), or typst-wins with a warning, or build refuses when both diverge?
3. **Seeding:** `new-opportunity` creates `typst/` immediately (editable from minute one), or lazily on first build?
4. **Gates:** hand-edited typst must pass `measure` gates on build (fail the build if not), correct?
