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
whitespace, and bundled font integrity. It discovers **36 registered PDF variants /
84 pages** from the Harvard and Cluster style definitions.
Each leaf record selects its showcase paper; checks also render its other
supported paper selections for validation, without requiring another set of
tracked showcase PDFs. A text-only Rust probe verifies A4/US Letter and
portrait/landscape selection in a temporary workspace. Settings tests use
Harvard; portable bundle tests render all four Cluster substyles in both locales.
The retired demonstration designs are deleted, including their fixture sources.

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

An explicit release dispatch builds optimized release binaries natively for
every platform in `.agent/release-platforms.txt`, currently Linux x86_64.
Each released binary passes `public-check` and tests proving that both the
launcher and direct executable
reject a mismatched workspace. A future Windows release must also exercise its
PowerShell download installer on a real native worker. Linux performs the
locked Rust unit suite, Clippy, and independent Poppler, QPDF, and pixel
comparisons. A real minimal Linux executor extracts the
actual download bundle and runs setup without Git, Rust, or a compiler.
The Linux independent-check job verifies the downloaded binary's checksum
and workspace identity with `doctor`; it avoids rerunning setup's full
document suite before its own public check. The archive job continues to
exercise the complete user setup path.

The same tested files become release assets; publication never rebuilds or
fetches an older rolling binary. Both GHA and Crow publication require every
explicitly released native target and all four shared gates. Unavailable GHA
does not prevent Crow delivery, and missing artifacts never silently reduce
the released platform set. Validation also runs ShellCheck, Actionlint,
and REUSE. See [Releases](releases.md) for cache and identity details.

The same line contract is available directly with `bash ./ccvl measure` or
`.\ccvl.cmd measure`. It reports all violations in one pass so underfill or
overflow causes an editorial iteration instead of a one-error-at-a-time loop.

## Small-model skill evaluation

The `Skill evaluation` workflow sends its generic decision cases to
the configured Groq model, defaulting to `openai/gpt-oss-20b`, when account usage
is authorized. Cases are deterministically batched by canonical skill, with the
complete matching skill and the descriptions of all declared skills supplied
to each low-context call. Both the
expected routing and answer key are withheld. A deterministic evaluator then
requires the correct skill, every expected action, no forbidden action, and a
valid response structure. Every option needs an explicit assessment, including
options the model excludes. It publishes all assessments, concise reasons, provider
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

The report is written to the ignored `.agent/cache/skill-eval/report.json` path.

For each case, return `case_id`, the chosen `skill`, a short case `reason`, and
`assessments` containing every supplied option exactly once:

```json
{
  "id": "exact-option-id",
  "applicable": false,
  "reason": "Brief justification grounded in the supplied skill."
}
```

Set `applicable` to true for every appropriate action under the supplied skill,
including required simultaneous obligations, and false otherwise.
Each assessment must contain exactly these three fields, use a
boolean, and give a nonempty reason of at most 12 words. The case reason also
remains limited to 12 words. Assess concurrent obligations independently;
completing one action does not discharge another.

The scorer derives `selected` from true assessments and applies the unchanged
hidden required/forbidden keys. Unknown, duplicate, missing or malformed
assessments fail. An explicitly false assessment of a required action still
fails; complete structure alone is not semantic success. Report schema 2 retains
both the assessments and derived selections. Legacy selected-only decisions are
rejected; keep historical evidence with its original scorer and do not invent
assessments to convert it.

`skill-eval --response-file <path>` scores this same contract without inference.
Its reports identify `source: response-file` and leave the inference provider
unknown (`null`). The model is also unknown unless explicitly supplied with
`--model`; that is a caller-provided label, not authenticated provenance.
Embedded response labels, the Groq environment and the default hosted model
never establish saved-response provenance. Retain original requests, responses
and provider telemetry in the producing adapter's independently verified receipt;
importing decisions does not make an offline fixture a provider result.

Each skill group is split into at most two cases per request, preserving skill
and case order. This reduces output pressure on the existing 1,800 completion-token
cap without raising that cap or changing the model and retry limits.
Repeating the skill/catalog prompt increases total prompt work;
batch provenance records the index and exact case IDs. Provider truncation or
incomplete option coverage still fails and supplies no passing evaluation.


## Actor–critic behavioral evaluation

`.agent/tests/skill-cases.json` includes decision cases for `ccvl-review`,
the AIDA paragraph/line contract, independent non-AIDA styles, protected locale
text, general letters,
source entailment, scope, clean controls, unavailable images and exhausted
corrections. Additional decision cases distinguish persuasive reframing from
invented authority, allocate distinct relevant CV contributions, preserve
purposeful summaries and handle known optional qualification gaps without
weakening supported work. These apply to any candidate, independently of the
showcase author. They run through the existing `skill-eval` interface. They test
routing and choices; they do not prove an agent wrote or inspected real output.

The separate synthetic review corpus is
[review-evaluation/cases.json](../tests/review-evaluation/cases.json), with
[evaluator-only expectations](../tests/review-evaluation/answer-key.json).
Its content-only contracts are independent of the shipped style catalogue;
no demonstration renderer is bundled. Artifact-specific evaluations must supply
a renderer and actual outputs satisfying the declared contract before assessing
rendering. It supplies eleven paired synthetic clean/seeded-defect cases plus general,
missing-image, provider-failure and actual-paper-mismatch cases. All career,
employer and source details are deliberately fictional evaluation data; never
use them as a real profile or copy private inputs into these fixtures.

The three editorial pairs exercise factual meaning and content allocation:

- An integrated workbook and usable handover support a confident reporting
  solution claim; colleague users do not establish management authority or
  department-wide deployment.
- Triage, roster planning and onboarding need distinct evidence. A short
  overview can signpost those arguments, while paraphrasing the triage example
  cannot replace the other two contributions.
- Confirmed volunteer coordination and reporting can make a strong practical
  case. A known missing desirable certification belongs in preparation; it
  neither licenses an invented credential nor requires an apology in the letter.

These pair descriptions and their evaluator-only expectations must stay out
of actor and critic contexts. A vocabulary match is not a pass: the evaluator
must check the supported implication, the preserved strength of a correction,
and which relevant contribution each passage supplies.

For independent forward testing, give a fresh agent one case's request and raw
sources, the named skill and its real references, and the minimum selected
style files. Keep this evaluation guide, the answer key, pair identity, expected verdict and actor
rationale out of its context. Use an isolated temporary workspace and the
existing authorised compute route. Create valid profile/application records
from the synthetic facts, render actual documents with the selected style,
then prepare a real review package. The fixture's three body entries are
independent-style content, not Harvard line arrays. Confirm the style/substyle
exists in current discovery before rendering; don't silently change a user
choice to mask a fixture/interface mismatch.

Use separate drafting and seeded-review runs. For drafting, provide the request
and sources without the fixture's `cl` draft; assess what the actor actually
writes. For seeded review, render the supplied `cl` unchanged so preparation
cannot silently repair the defect before the critic sees it. Freeze the actual
text, sources, selections, contracts and rendered pages in the review package.
Give each critic only that one package, the canonical review skill and its
editorial/protocol references, and its result destination. Replace fixture IDs
with opaque run IDs and retain their mapping only for the evaluator. Do not
give one critic both sides of a pair or the corpus file containing its twin.
A text-only run can establish source/editorial findings, but remains incomplete
for visual coverage and must be reported separately from full artifact review.

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

For the editorial pairs, also record whether a clean confident claim was
unnecessarily weakened, whether an invented implication was precisely located,
whether supported distinct contributions cover the required duties, and whether
a useful overview or real evidence reuse was wrongly rejected as duplication.
Known absent desirable qualifications are coverage notes, not unresolved support
for a claim the letter never makes. Judge optional advice separately from
material defects; do not reward a critic for manufacturing objections.
