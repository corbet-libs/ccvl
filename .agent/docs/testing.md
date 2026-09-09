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
whitespace, and bundled font integrity. It discovers **36 PDF variants / 68
pages** from the style definitions, including both A4 and US Letter demos.

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
one further compile in a fresh process per variant proves cross-process
byte reproducibility. Independent tools inspect the generated and tracked
PDFs, compare every page's pixels and text, and check declared shared pages.
The artifact directory must not exist, and no PDFs are exported until all
document checks pass. It is temporary evidence for that invocation, not a
persistent cache that could bypass current source checks.

Regression fixtures verify a complete workspace without Harvard; selected
style/content/page defaults; actual paper and font changes through layout
inputs; locale-specific geometry; incorrect PDF policy rejection; and wrapped
contact names. All newly added or affected pages also need visual review.
`render-previews.sh` regenerates the [gallery](../../cvl/README.md) from the
registered PDFs; thumbnails complement full-resolution page inspection.
It reuses previews only when PDF bytes, page count, the renderer executable
and reported version, rendering script, and every output image still match.
Missing or changed images are rebuilt. Cache records live in the ignored
`.agent/cache/previews/` directory; a failed render does not publish partial
pages or update its record. A successful refresh removes surplus numbered
previews for that PDF when its page count shrinks. `test_previews.sh` exercises reuse, each source
of invalidation, and recovery from a failure partway through a document.

An explicit release dispatch builds optimized release binaries natively on Linux
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

The credential-bearing workflow is currently disabled and restricted to a
trusted self-hosted runner. It never runs with secrets on hosted runners, pull
requests, or forks. A rate limit or provider outage is reported distinctly and
fails evaluation; it is never presented as a semantic pass.

To run the same evaluation outside Actions, set `GROQ_API_KEY` without writing
it to the repository, then run:

```sh
source .agent/scripts/rust-toolchain.sh
ccvl_select_rust_toolchain
"${CCVL_CARGO_COMMAND[@]}" run --quiet --locked -- skill-eval
```

The report is written to the ignored
`.agent/cache/ai-skill-eval/report.json` path.
