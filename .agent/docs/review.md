# Independent application review

ccvl prepares immutable evidence and rendered artifacts, validates cited critic
results and limits corrections. An external coordinator supplies a fresh
reviewer context through an already authorised agent environment. The CLI does
not invoke a model, choose a provider or create an OS sandbox. Read-only critic
behaviour is an orchestration instruction; hashes detect source changes, not
prove that a model read or understood an artifact.

Use `ccvl-apply` or `ccvl-cv` as actor and `ccvl-review` as critic. Review general
CVs/open letters by their stated purpose; targeted packages use an archived
posting and attributable research. No vacancy is invented for a general review.
Private evidence, actual opportunities and derived reports stay downstream and
never enter external CI.

## Prepare the run

Write a JSON specification using the exact fields below. Paths are relative
to the workspace and must refer to real inputs. The schema is implemented in
[review types](../src/review/types.rs); unknown fields are rejected.

| Field | Value |
|---|---|
| `purpose` | `{ "kind": "targeted" or "general", "description": "actual audience and task" }` |
| `actor_id` | Identity of the drafting context |
| `model` | The actual selected reviewer model identifier |
| `max_input_tokens`, `max_output_tokens`, `time_limit_seconds` | Explicit positive budgets for this run |
| `documents` | Selected CV/CL records, one entry per reviewed document/variant |
| `sources` | Relevant original candidate evidence, confirmations, posting and research |
| `rubric` | Paths to the actual applicable editorial/reviewer instructions and explicit preferences |
| `editable` | Canonical content paths the actor may revise in this run |

A document entry contains `id`, `document` (`cv` or `cl`), lowercase `locale`,
`pages`, `application` and `profile`, with optional `style`, `substyle` and
`paper`. Resolve the intended selections from the task and record. A source
entry contains `id`, `path` and `role`: `candidate`, `confirmation`, `posting`
or `research`. Include all relevant evidence, including contradictions; an
actor summary or previously generated CV is not independent support. Targeted
runs need posting and research evidence; their absence is an evidence gap.
Missing files/tool output cannot become a passed review.

Pin the selected document contract, user preferences and applicable rules in
`rubric`. [Editorial guidance](editorial.md) is universal; the AIDA contract
binds paragraph roles to its defined structure and line budgets. Shared locale
conventions come from
cletter/family public interfaces; ccvl does not define duplicate locale rules.
Keep original names, quotations, user choices and qualifiers needed for factual
meaning. Apply persuasive framing and MECE requirements to every candidate.
Select budgets from the authorised environment and actual evaluation evidence;
example numbers are not product defaults. The coordinator must enforce provider
limits and cancellation while a call is running; the CLI validates recorded
usage and current run state when a result arrives. Unavailable usage telemetry
is recorded explicitly as null, not invented; known budget overruns fail.

From the workspace root, using `ccvl` or its platform launcher:

```sh
ccvl review prepare <spec.json> <run-dir>
ccvl review status <run-dir>
```

The run directory must be a new `.agent/cache/review/<id>` or
`opportunities/<organisation-key>/<position-key>/review/<id>` location. Prepare
compiles the selected documents, measures them, verifies PDF requirements,
extracts actual text and rasterises every page. It records failures rather than
manufacturing success. Use the existing authorised compute route for those
operations; the command does not install a reviewer service.

`state.json` identifies the current revision and manifest hash. The initial
manifest is `revision-0/manifest.json`; its `artifacts` entries identify frozen
snapshots by ID, hash, role and path, with document/page locators where relevant.
Read those paths instead of guessing output names. The manifest records source
and runtime identities, pinned specification, dependencies, resolved document
settings, actual check outcomes, changed artifacts and limitations. Objects are
content-addressed snapshots. Neither snapshots nor generated PDFs are a second
authoritative wording source.

Preserve saved review JSON byte-for-byte, including specifications and external
results that may themselves be frozen as evidence. The review writer serializes
JSON without a final newline; manifests, accepted results and revision ancestry
bind those exact bytes. Repository text hygiene accepts valid JSON with or without
that newline under `opportunities/<organisation>/<position>/review/`, while still
checking UTF-8, CR line endings, trailing whitespace and merge markers. Ordinary
source files retain their final-newline requirement. This encoding exception
neither validates a review's readiness nor permits publishing private reviews;
use `review status` for the current packet's integrity and readiness checks.

## Run a fresh critic

Give the critic the run directory, the `ccvl-review` skill and the requested
result destination. Do not supply the actor's desired verdict, evaluation answer
key or purportedly fixed findings as established truth. The critic reads the
original evidence and selected rules, extracted final text and every rendered
page itself. Claim markers prove bookkeeping, not that the wording follows
from the source. A model without image access cannot claim page coverage.
If a required input is unavailable, continue the accessible evidence,
extracted-text and mechanical checks, recording the missing coverage and an
incomplete review.

Review evidence/relevance first and writing/presentation second. Check what a
reasonable reader would infer against the original evidence or explicit user
confirmation. Persuasive embellishment is expected: retain strong framing,
value and narrative when their factual implications are supported. A factual
objection must name the unsupported implication; forceful wording or departure
from a source's phrasing alone is not a defect. Preserve necessary scope and
estimates while allowing unnecessary hedges to be removed. Judge locale
exhibits against the matching per-locale file (BCP 47 primary subtag,
English inline examples otherwise); quoted exhibits illustrate the rules,
they never override them.

Check MECE in both directions: each block contributes a distinct argument, and
the package addresses the material priorities of its audience and purpose.
Identify repeated arguments and relevant supported dimensions they crowd out.
Evidence may support multiple requirements; concise highlights and summaries
can signpost fuller evidence. Missing support belongs in preparation/review as
a gap, never an invented capability or compulsory apology in the letter.
Classify content coverage/overlap under `relevance` (or `language` for redundant
phrasing); cite the actual passages and priority. Explain materiality rather
than treating all repetition or every unmentioned job keyword as blocking.

Check the strongest relevant supported facts before residual CV material,
while retaining useful transferable evidence. For an AIDA letter, check the
defined six paragraph roles, exact 26 body lines and five additional highlights
between paragraphs 3 and 4 together. For German opportunity letters also check
the paragraph-6 close against its contract (three lines, core tasks named,
final line opening with „Ich freue mich"; forbidden close phrases are language
findings). Other document structures keep their declared contracts. Check
punctuation on the rendered summary and letter text (space before a comma and
„, und"/„, oder" in flat enumerations are defects; enumeration structure and
keyword prioritisation are language findings). A short conventional
close, ordinary “interest” or a supported skill label is not inherently a defect.

## Result interface

Every review delivers supported findings, an account of its coverage and the
CLI's computed outcome. An empty finding list still needs complete coverage;
an unavailable input still permits the accessible checks. Write JSON with the
fields below, then submit it and read the returned state and reasons. Populate
IDs, hashes, citations, coverage and usage from the actual run; there is no
ready flag for the critic to assert.

| Field | Contents |
|---|---|
| `schema_version` | `1` |
| `revision`, `manifest_sha256` | Current revision and exact manifest hash from run state |
| `reviewer` | `agent_id`, actual `model`, truthful `fresh_context` boolean |
| `coverage` | `artifacts_read`, `unavailable`, `not_applicable`, `evidence_pass`, `presentation_pass` |
| `findings` | Zero or more supported finding records |
| `regression_checked` | Whether the revised whole candidate was checked for new errors |
| `changed_artifacts_reviewed` | Actual reviewed artifact IDs from the current change set |
| `input_tokens`, `output_tokens` | Actual reported usage, or null when telemetry is unavailable; never guessed counts |

`coverage.artifacts_read` lists every artifact actually read. Complete coverage
requires all manifest artifacts except compiler dependencies with role `input`.
`coverage.unavailable` lists known manifest artifact IDs that the critic could
not inspect. Both lists require unique IDs and cannot overlap. Preparation
records sources that could not be frozen in the manifest's failed checks;
do not invent artifact IDs for them. Record incomplete passes honestly.
In general mode, add
`coverage.not_applicable["posting-specific"]` with a reason explaining why
vacancy criteria do not apply. A green compile or actor explanation cannot
substitute for a source, text or visual pass.

Each finding contains:

- `id`, `kind` (`error`, `uncertainty`, `preference`) and `category` (`support`,
  `attribution-scope`, `consistency`, `relevance`, `language`, `presentation`);
- `material`, `materiality_reason`, and a separate `confidence` from 0 to 1;
  optional `explicit_requirement` defaults to false and must be true for a
  material preference that cites the exact pinned user requirement;
- `rule`, `location`, and an `evidence` array of citations;
- `correction_constraint`, describing supported meaning and persuasive value
  to preserve or restore, plus any distinct argument/coverage to improve;
- `resolution`: `open`, `fixed-and-verified`, `disputed-with-evidence` or
  `optional-suggestion-declined`; `verification` records actual verification
  when applicable, otherwise null.

Every citation has `artifact`, `sha256`, `locator` and `excerpt`. Use the exact
manifest ID and hash. Text excerpts must occur in the cited text or decoded JSON field;
page citations describe an actual visible observation. Use `json:/pointer`
for JSON content/rules, `line:N` for an excerpt on that 1-based text line, and
`page:N` or `page:N#region` for page/text artifacts. Extracted page text requires
the page prefix too: `page:1#line:15`, not `line:15`. A finding's `location`
must reference content, extracted text or a rendered page. Selection and
measurement metadata can support it through `evidence`. Locate the affected
passage/page precisely, cite the applicable rule and original support or
contradiction. Missing evidence is uncertainty, not automatically falsehood.
Confidence does not determine materiality. Optional preferences cannot erase
facts or override explicit choices. Do not require literal source phrasing or
extra caveats when the reader's factual understanding is already defensible.
No finding quota is required.

```sh
ccvl review submit <run-dir> <result.json>
ccvl review status <run-dir>
```

Protocol command success is not readiness: `submit` and `status` can exit zero
while returning `incomplete`, `needs-evidence` or `needs-decision`. Always read
the JSON state and reasons. Invalid submissions fail and remain unaccepted.
An accepted result is immutable; an incomplete accepted coverage result needs
a new independent run, not an overwritten result.

Interrupted preparation retries the same reserved correction and retains partial
artifacts. A completed manifest is reused only while its hashes remain current.
Finding ancestry remains bound across failed intervening revisions. A stale
lock needs confirmation that its owning process stopped; time alone does not
authorise removing it.

The accepted result is stored with its revision. The CLI validates identities,
citations, coverage, budgets and freshness and computes status. A structurally
valid review remains fallible: human/independent adjudication establishes
semantic quality, not JSON validation alone.

## Correct, verify, stop

The initial candidate has **two corrections**. Before editing an allowed
canonical content path, consume the next allowance:

```sh
ccvl review begin-revision <run-dir>
# The actor makes this correction in canonical content.
ccvl review prepare-revision <run-dir>
# A fresh critic reviews the current artifacts and writes the next result.
ccvl review submit <run-dir> <result.json>
ccvl review status <run-dir>
```

A revised candidate consumes its allowance even if it fails mechanically.
Check every allowed revision, and review it when renderable, before deciding
whether another is permitted. Do not make a third final correction after
exhaustion. Fixing one finding can introduce another; inspect the current whole
candidate and changed artifacts. The actor may dispute a finding with evidence,
but the critic must verify resolution against actual artifacts. A previous
finding may be reclassified when independent counterevidence and verification
establish that the critic was mistaken. If a passage was deleted, cite its
current containing field and explain the verified removal; parent hashes
retain the previous finding. Changing
sources, rubric, selections or purpose starts a new run with explicit context;
never restart just to conceal an exhausted correction limit.

| Outcome | Meaning |
|---|---|
| `ready` | Current artifacts checked and fully reviewed without unresolved material factual defects |
| `needs-evidence` | Material support or confirmation is unavailable |
| `needs-decision` | A material conflict or explicit user choice requires a decision |
| `incomplete` | Coverage/tool/provider failure, stale artifacts or exhausted corrections |

Preparation and correction can also report an intermediate state. Use the
returned status and reasons, not a favourable interpretation of the filename.
Continue independent work while obtaining needed evidence or decisions; qualify
or remove unsupported wording only when it preserves the requested meaning.
Cancellation uses `ccvl review cancel <run-dir>`; the external coordinator must
also stop any in-flight provider call. No review outcome authorises submission,
messaging, signing or accepting declarations.

## Evaluation

[Testing](testing.md) distinguishes deterministic gates, action-selection
skill cases and independent artifact evaluations. Evaluate a synthetic clean
control alongside each seeded defect, hide the answer key from both agents and
report incomplete runs honestly. No implementation or passing action labels
alone establish that independent review improves real application quality.
