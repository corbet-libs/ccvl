# Testing ccvl

ccvl separates deterministic release checks from a deliberately small-model
behavioural test. A green AI test is supporting evidence, not proof that every
agent will behave correctly.

## Mechanical checks

Run the complete local suite with:

```sh
bash ./ccvl check
```

On Windows, run `.\ccvl.cmd check` instead.

It verifies the workspace manifest, schemas, application/profile data, declared
skills and evaluation cases, local Markdown links, Typst formatting, Git
whitespace, and bundled font integrity. It discovers **36 tracked PDF variants /
68 pages** from the style definitions, including both A4 and US Letter demos.
Each leaf record selects its showcase paper; checks also render its other
supported paper selections for validation, without requiring another set of
tracked showcase PDFs.

For every style it checks the requested page count, valid PDF geometry,
usable text, embedded Unicode-mapped fonts, repeat-render reproducibility and
semantic equality with checked-in outputs. Contracts optionally require exact
dimensions (including per-locale overrides), font families, contact fields,
images, PDF version and tagged structure. Contact matching accepts whitespace
and line breaks; incorrect or missing names still fail. PDFs must be
unencrypted and free of forms, JavaScript and attachments.

Harvard opts into its five-line Summary, measured headings/bullets, six letter
paragraphs, five highlights, fixed station allocation and paragraph-role
budgets. Its 2/3/4-page CV presets require identical shared pages. These are
Harvard rules, not assumptions about every document style. The independent
styles own different fields and page geometry and do not need Harvard metrics.
`public-check` adds the public/private boundary and secret checks. Linux's
`check-linux-deep.sh` independently uses Poppler, QPDF and page images.
It runs `public-check --artifacts <new-directory>` once and reuses those
freshly checked PDFs. The Rust check still compiles report/enforce pairs;
one further compile in a fresh process per tracked variant proves cross-process
byte reproducibility. Independent tools inspect the generated and tracked
PDFs, compare every page's pixels and text, and check declared shared pages.
The artifact directory must not exist, and no PDFs are exported until all
document checks, including nondefault papers, pass. It is temporary evidence for that invocation, not a
persistent cache that could bypass current source checks.

Regression fixtures verify a complete workspace without Harvard; selected
style/content/page defaults; actual paper and font changes through layout
inputs; locale-specific geometry; incorrect PDF policy rejection; and wrapped
contact names. All newly added or affected pages also need visual review.
Additional fixtures compare resolved shared wording with direct Typst loading,
check source ownership and leaf overrides, and build actual Harvard documents
with an unrelated document's contract missing or malformed. A selected build
must succeed independently; full workspace checks still reject broken styles.
Selected font fixtures compile different styles in one compiler without
loading each other's extra fonts. Watcher fixtures exercise actual transitive
reads, missing and changing dependencies, invalid metadata, font recovery,
optional inputs, symlinks and edits during compilation.

`render-previews.sh` regenerates the [gallery](../../cvl/README.md) from the
registered PDFs; thumbnails complement full-resolution page inspection.
It reuses previews only when PDF bytes, page count, the renderer executable
and reported version, rendering script, and every output image still match.
Missing or changed images are rebuilt. Cache records live in the ignored
`.agent/cache/previews/` directory; a failed render does not publish partial
pages or update its record. A successful refresh removes surplus numbered
previews for that PDF when its page count shrinks. `test_previews.sh` exercises reuse, each source
of invalidation, and recovery from a failure partway through a document.

Every CI run builds optimized release binaries natively on Linux
x86_64/aarch64, macOS x86_64/arm64, and Windows x86_64/arm64. Each binary passes
`public-check` and tests proving that both the launcher and direct executable
reject a mismatched workspace. Windows also exercises the PowerShell download
installer. Linux performs the locked Rust unit suite, Clippy, and independent
Poppler, QPDF, and pixel comparisons. A minimal Linux container extracts the
actual download bundle and runs setup without Git, Rust, or a compiler.
The Linux independent-check job verifies the downloaded binary's checksum
and workspace identity with `doctor`; it avoids rerunning setup's full
document suite before its own public check. The archive job continues to
exercise the complete user setup path.

The same tested files become release assets; publication never rebuilds or
fetches an older rolling binary. The CI workflow requires all six builds and
all validation jobs before publication. It also runs ShellCheck, Actionlint,
and REUSE. See [Releases](releases.md) for cache and identity details.

The same line contract is available directly with `bash ./ccvl measure` or
`.\ccvl.cmd measure`. It reports all violations in one pass so underfill or
overflow causes an editorial iteration instead of a one-error-at-a-time loop.

## Small-model skill evaluation

The `Skill evaluation` workflow sends its generic decision cases to
Groq's free-tier `openai/gpt-oss-20b` model. Cases are deterministically batched
by canonical skill, with the complete matching skill and the descriptions of
all declared skills supplied to each low-context call. Both the
expected routing and answer key are withheld. A deterministic evaluator then
requires the correct skill, every expected action, no forbidden action, and a
valid response structure. It publishes all decisions, concise reasons, provider
finish status, and token usage as a workflow artifact.

The workflow runs only in `corbet-labs/ccvl`, on relevant pushes to `main` or a
manual dispatch. It never runs with secrets on pull requests or in forks. A
rate limit or provider outage is reported distinctly and fails the workflow;
it is never presented as a semantic pass.

To run the same evaluation outside Actions, set `GROQ_API_KEY` without writing
it to the repository, then run:

```sh
cargo "+1.94.0" run --quiet --locked -- skill-eval
```

The report is written to the ignored
`.agent/cache/ai-skill-eval/report.json` path.


## Actor–critic behavioral evaluation

`.agent/tests/skill-cases.json` includes decision cases for `ccvl-review`,
optional AIDA in independent styles, protected locale text, general letters,
source entailment, scope, clean controls, unavailable images and exhausted
corrections. These run through the existing `skill-eval` interface. They test
routing and choices; they do not prove an agent wrote or inspected real output.

The separate artifact corpus is
[review-evaluation/cases.json](../tests/review-evaluation/cases.json), with
[evaluator-only expectations](../tests/review-evaluation/answer-key.json).
It supplies eight paired synthetic clean/seeded-defect cases plus general,
missing-image, provider-failure and actual-paper-mismatch cases. All career,
employer and source details are deliberately fictional evaluation data; never
use them as a real profile or copy private inputs into these fixtures.

For independent forward testing, give a fresh agent one case's request and raw
sources, the named skill and its real references, and the minimum selected
style files. Keep the answer key, pair identity, expected verdict and actor
rationale out of its context. Use an isolated temporary workspace and the
existing authorised compute route. Create valid profile/application records
from the synthetic facts, render actual documents with the selected style,
then prepare a real review package. The fixture's three body entries are
independent-style content, not Harvard line arrays. Confirm the style/substyle
exists in current discovery before rendering; don't silently change a user
choice to mask a fixture/interface mismatch.

Evaluate each paired control and seeded draft with identical source/context
coverage. Judge the generated/retained content and actual findings with cited
sources and pages, not only action labels. Missing-image/provider cases
exercise their stated limitation rather than inventing a successful call.
For the paper-mismatch case, the fixture deliberately renders the override;
compare those actual dimensions with the unchanged explicit request. Revision
cases require review of the current whole candidate, including new errors.

An independent evaluator reads the withheld expectations after completion.
Record material-error recall, misses, false alarms on clean controls, valid
corrections, introduced errors, unresolved disputes, final quality, completion,
rounds, actual token usage, latency and actual cost. Include tool/provider
failures as incomplete outcomes in the denominator; retain source, model and
rubric identities. Compare against an actor-only baseline and repeat paired
runs to expose variability. Until these actual artifact runs are executed,
report them as pending; checked-in fixtures or a passing decision suite are
not a behavioral pass.
