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

It verifies:

- the workspace manifest, JSON schemas, applications, profile, canonical
  skills, AI cases, and local Markdown links;
- the Rust-native evaluator and workspace contracts, embedded Typst formatting,
  and clean Git whitespace;
- binary asset integrity and all four bundled Archivo variants;
- all 12 CV variants and four cover letters with zero Typst diagnostics;
- exactly five Summary lines, six cover-letter paragraphs with 26 body
  lines, and five one-line highlights per locale;
- 6–8 verified full stations on CV page 1; exactly 10 two-bullet stations on
  page 2; exactly 10 two-bullet projects on page 3; and three groups of three
  three-line competency blocks on page 4, with stable source markers and
  identical assignments across locales;
- measured minimum and maximum fill for CV headings, subtitles, bullets,
  Summary lines, cover-letter body lines, and highlights;
- bounded vertical gaps and highlight position so the cover letter fills A4
  with distributed rhythm rather than large elastic whitespace blocks;
- explicit paragraph-role budgets, justified prose, and zero paragraph splits;
- exact A4 page counts, usable text layers, embedded, subsetted, and
  Unicode-mapped Archivo fonts;
- unencrypted PDFs without forms, JavaScript, attachments, or fallback fonts;
- machine-readable showcase contact details and rendered cover-letter signatures;
- byte-for-byte reproducibility across two independent renders;
- semantic equality between fresh builds and the checked-in PDFs, excluding
  only the PDF rendition identifier;
- pixel identity of the two CV pages shared by every page preset.

Every CI run builds optimized release binaries natively on Linux
x86_64/aarch64, macOS x86_64/arm64, and Windows x86_64/arm64. Each binary passes
`public-check` and tests proving that both the launcher and direct executable
reject a mismatched workspace. Windows also exercises the PowerShell download
installer. Linux performs the locked Rust unit suite, Clippy, and independent
Poppler, QPDF, and pixel comparisons. A minimal Linux container extracts the
actual download bundle and runs setup without Git, Rust, or a compiler.

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
all seven skills supplied to each low-context call. Both the
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
