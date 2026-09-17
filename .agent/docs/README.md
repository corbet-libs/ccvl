# Agent docs topology

Maintainer map for `.agent/docs/`. Canonical homes: each rule lives in
exactly one file; everything else points at it. Load this map when
editing docs or skills, never during application runs (it is not
referenced from any runbook).

## Canonical homes

| Rule domain | Home | Must not duplicate into |
|---|---|---|
| Evidence, register, MECE, targeting, hyphen criteria | `editorial.md` | skills, contracts |
| Letter geometry (paragraphs, lines, highlights) | `cvl/<style>/contract.toml` + `cover-letter.md` | skills, scaffolds |
| Review protocol, finding taxonomy, correction allowance | `review.md` | skills (route, do not restate) |
| Skill run order, self-check, readiness | `skills/ccvl-apply/SKILL.md` | docs |
| Locale mechanics (spelling, greetings, closings, dates) | cletter and family | ccvl docs, skills, code |
| German drafting exhibits | `examples-de.md` | instruction prose anywhere |

## Load order per run

Skills declare their own read lists; the shape is always frontmatter,
then `SKILL.md` body, then at most one level of linked files. Reference
files longer than a hundred lines open with their scope in the first
screen. Locale exhibits resolve by BCP 47 primary subtag
(`examples-<subtag>.md`, English inline as fallback); never load an
unmatched locale.

## Supported locales registry

| Language tag | Exhibits | Mechanics |
|---|---|---|
| `de-ch`, `de-li`, `de-de`, `de-at` | `examples-de.md` | cletter |
| `en-*` | inline English examples (fallback) | cletter |
| `fr-*`, `it-*` | none yet (follow the exhibit skeleton below) | cletter |

## Exhibit skeleton

Every `examples-<subtag>.md` uses the same sections so gaps stay
visible: Evidence binding, Register, Targeting. Instruction prose stays
English; the locale appears only inside quoted exhibits.
