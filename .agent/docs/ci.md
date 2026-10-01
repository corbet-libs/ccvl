# Selecting CI checks

GitHub Actions is preferred when available for public source checks. Pushes to
`main` and pull requests run the checks their changes require when Actions is
available; see [Change-scoped GitHub checks](#change-scoped-github-checks).
Release preparation and publication remain an explicit manual
dispatch, using the platform set in `.agent/release-platforms.txt`, currently
Linux x86_64. Crow independently provides selected checks and the complete
release path when Actions is unavailable; do not
run both providers for the same validation without a missing result or changed
input. Private downstream data and credential-bearing jobs stay on trusted
internal workers.

On a provisioned build worker, use `bash .agent/scripts/ci-check.sh <check>...`:

| Check | Coverage |
|---|---|
| `rust` (default) | Stable Rust formatting, Clippy and locked unit/document tests |
| `lint` | Actionlint, ShellCheck, REUSE and the CI script behavior tests with preinstalled tools |
| `documents` | Locked Linux release build and independent PDF/text/layout verification |
| `styles` | Optimized non-LTO build, then `public-check` limited to the space-separated `<doc>/<style>` list in `CCVL_CHECK_STYLES` (every style when unset) |
| `skill-eval-build`, then `skill-eval` | Explicit trusted small-model evaluation; separate credential-free build and credential-bearing evaluation |

The command uses existing tools. Crow supplies a memory-bounded parallel job
and test-thread budget through the shared `ccid` adapter. A direct invocation
without those environment settings retains conservative script defaults.
Run only checks whose inputs changed or whose results are
missing. Private downstream data must remain on trusted internal workers.
GitHub-hosted jobs install the current stable channel and report the actual
compiler version. Crow reuses suitable preinstalled Rust, preferring an installed
stable channel and otherwise the provisioned rustup default or standalone
compiler. `Cargo.toml` declares the minimum supported version, not an exact
compiler requirement. `RUST_TOOLCHAIN` can explicitly select another installed
stable compiler; a missing selection fails visibly. No worker toolchain
installation occurs. Lint checks likewise reuse existing tools, including
installed Nix store packages omitted from the worker's PATH. The lightweight
bootstrap and CI-selector behavior tests run with lint. A future advertised
Windows binary must also pass bootstrap checks on its actual native platform.

## Change-scoped GitHub checks

The first `ci.yml` job, `changes`, runs `.agent/scripts/ci-changes.sh github`.
It lists changed paths with `git diff --no-renames --name-only`: a pull request
compares its head with the merge base of base and head (`base...head`); a push
to `main` compares `before..after`. A missing, all-zero or unresolvable commit,
and every other event (manual release dispatch, the daily schedule), selects
everything. It outputs `rust`, `documents` and `styles`.

| Changed path | Rust job | Document styles |
|---|---|---|
| `.github/workflows/ci.yml`, `.agent/scripts/ci-changes.sh` | yes | all |
| `.agent/src/**`, `.agent/core/**`, `.agent/build.rs`, `.agent/typst/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `ccvl.json` | yes | all |
| `cvl/cv/<style>/**`, `cvl/cl/<style>/**` | no | `<doc>/<style>`; when that style no longer exists, the Rust job and all |
| `cvl/shared/<family>/**` | no | same-named styles and styles whose files reference `shared/<family>/`; when none, the Rust job and all |
| `cvl/profile.toml`, `cvl/assets/**` | no | all |
| other `cvl/**` | yes | all |
| `cvl/README.md` | yes | none |
| `interview/stations.toml` | yes | CV styles whose `contract.toml` declares `[layout_contract…]` |
| `.agent/skills/**`, `.agent/scaffolds/**`, `.agent/schemas/**`, `.agent/tests/fixtures/**`, `.agent/tests/skill-cases.json`, `.agent/docs/editorial.md`, `interview/**`, `opportunities/**`, `REUSE.toml`, `LICENSES/**`, `.crow/downstream-sync.yaml`, `.agent/release-platforms.txt`, and the Rust job's `ci-check.sh`, `release-ci.sh`, `rust-toolchain.sh`, `release-evidence.py`, `downstream-sync.sh` | yes | none |
| Other `.agent/docs/**`, root and `.github` Markdown, `.agent/AGENT.md`, the release/skill/sync workflows, shell and PowerShell tests, bootstrap/release/lint-only scripts, `.ci/**`, other `.crow/**`, dispatchers, `justfile` | no | none |
| Anything else | yes | all |

The Rust rows follow what Rust tests read. Engine tests run on the synthetic
workspace in `.agent/tests/fixtures/workspace` and never read the showcase
styles, shared families, profile or assets under `cvl/`; see
[Rust test fixtures](testing.md#rust-test-fixtures). With every directory of
`cvl/` except its README deleted, the suite still passes. They still read
skills, scaffolds, schemas, the fixtures, skill cases, data-root READMEs
(including `cvl/README.md`), licenses bundled by style exports, the editorial
rubric of the review smoke test and the downstream-sync guard, so those paths
keep the Rust job.

The real-data invariants these tests used to pin are enforced by `check`,
which the document job runs (`public-check` for the selected styles keeps every
workspace-wide check):

| Invariant | Enforced by |
|---|---|
| Harvard and opted-in contracts keep the frozen station, Summary, compact-delta and AIDA budgets | workspace-wide frozen-contract check |
| No style vendors measurement code | workspace-wide ownership scan of `cvl/` |
| Manifest, slots, leaf inventory and referenced files | workspace-wide manifest and style checks |
| Every leaf compiles at every preset and passes its measurement gates; tracked outputs exist and match | document checks of the selected styles |
| Letter metrics carry the contract's fill bounds | measurement of the selected styles |
| A build never reads the other document's tree | document checks of the selected styles |
| Resolved customization copies and portable bundles (`portable_bundle`, Cluster) reproduce the build | document checks of the selected styles |
| The interview station plan fits the station protocol and the entry sources | Harvard CV document checks; `interview/stations.toml` selects them |

So a pull request that touches only one style's files, or one shared family,
runs the scoped document job and lint, not the Rust job. A removed or renamed
style, an unreferenced family and any other `cvl/` path still select
everything. A false skip is worse than a slow run, so an unclassified path
selects everything; extend the explicit no-op rows only after checking that no
Rust test or document reads the path, and add a Rust-test read of `cvl/` only
with its selector row. Update `.agent/tests/test_ci_changes.sh` with the rule.

Jobs gated on the selection fail open: when `changes` fails, the Rust and
document jobs still run, the latter for every style. A skipped job reports
"skipped", which satisfies required checks. Lint always runs: it takes under a
minute in parallel, and REUSE must see every new file. Manual dispatch always
selects the Rust job, so its `gate-rust` and `gate-lint` receipts and the
release gates are unchanged.

`Scoped document checks` runs `ci-check.sh styles` with the selected styles.
It builds an optimized binary without thin LTO and with 16 codegen units in a
separate `ci-documents` Cargo profile: dependencies keep release optimization
and its own cache, while the link avoids the slow release settings. A debug
build compiles faster but checks every document about 15 times slower. Then it
runs `public-check --style ...`, which keeps every workspace-wide check. A pull
request that touches only prose runs lint alone; one that touches only a style
runs that style's document checks and lint, each under a minute with warm
caches, in parallel. The Rust job lists Clippy
before tests so lint findings surface before the longer test build.

Both Rust jobs restore caches saved by `main`. Pushes to `main` that change
engine inputs refresh them, and a daily scheduled run on `main` rebuilds them
after a new stable toolchain, so pull requests rarely start cold. The schedule
has its own concurrency group and never runs release jobs. Rust is not pinned.

The manual Crow `ccid` workflow accepts `CHECKS=rust`, `lint`, or `documents`.
Its `.ci/ccid.toml` selectors invoke the same commands. The operator submission
helper stages the exact committed source archive, verifies locally available
LFS and submodule inputs, and supplies the pinned shared tool archive and
binary with their SHA-256 digests. Missing inputs fail closed. The adapter
checks the source commit against `CI_COMMIT_SHA` before execution.

The manual `candidate-date` Crow workflow checks an exact unpublished date-fix
candidate without building or publishing the full application. Its separate
`.ci/archives.toml` revision is staged with a verified digest; the ordinary
published workflow source retains its own identity. The candidate's focused
Rust module resolves its small registry dependency graph once and retains the
lockfile before locked checks. Typst helper/letter regressions reuse the offline
`typst==0.15.0` cache; a missing cached Typst dependency fails visibly.
Receipts and rendered artifacts remain under the persistent Cargo target's
`ccvl-candidate-date/<candidate>/<harness>/run-*` directory. This proof does
not replace the locked full application or native release gates. Dispatch only
this workflow for the pinned candidate; routine selectors do not stage it.

Compiled targets live in persistent dedicated Cargo storage, namespaced by
canonical repository identity. An explicit `CARGO_TARGET_DIR` is honored.
The shared tool locks the actual target directory, retains unchanged source
freshness, and cleans its owned source scratch. Worker package-cache settings
remain authoritative. `CI_JOBS`, `CI_TEST_THREADS`, `CI_MEMORY_MB` and
`CI_TIMEOUT` permit explicit bounded overrides; memory admission still applies.
The superseded single-core `verify` workflow has been removed.

The staged archive avoids a source clone from GitHub. The Crow forge integration
may still need GitHub to retrieve workflow configuration; a submission failure
is not a test result. The operator owns registration and staging configuration.

These selectors provide source validation. The separate Crow `release-linux`,
`release-archive` and `release-publish` workflows provide the complete released
Linux x86_64 path; see [Releases](releases.md). They require real native evidence
for every advertised platform and all four shared gates. Linux evidence does
not prove macOS, Windows or ARM behavior. A registry dependency that is not published
still blocks a locked build; a provisional path-patched lock is not release
evidence. Private sync and external model evaluation have separate manually
selected trusted workflows; neither runs with routine Rust or lint checks.

The manual Crow `downstream-sync` workflow stages a verified source archive and
a Git bundle containing the same commit and its ancestry. It builds the public
gate with existing Rust in a dedicated, locked cache, then merges into the
private downstream and runs its boundary and document checks. Immediately before
pushing it rechecks public `main`; changed or unavailable upstream metadata
prevents publication. Existing private SSH credentials remain on the internal
worker. The operator helper reads `.ci/archives.toml` to stage Git history only
for this workflow. Lint includes isolated tests for stale upstream and failed
document gates; those fixtures do not substitute for a real downstream run.

The manual Crow `skill-eval` workflow runs only for public ccvl `main`. Its first
step compiles the evaluator with locked dependencies and existing Rust, without
provider credentials. Its second step binds the repository secret
`ccvl_groq_api_key` to `GROQ_API_KEY`. That secret must be configured by an
authorized operator before dispatch; adding the workflow does not provision a
credential or prove the provider is available. A missing key is a configuration
failure, never a passing evaluation. The legacy GHA skill job remains disabled.
Once the exact public candidate and credential are ready, the operator submission
is `crow-ci run --repo /path/to/ccvl --workflow skill-eval` using the configured
shared helper; use `plan` in place of `run` to inspect the staged identities first.

Both steps verify the source archive and shared ccid binary, use the same
dedicated target storage and acquire its lock. The evaluator's recorded SHA-256
and runtime identity must still match when the second step starts; its CLI also
checks the runtime against the extracted source. A concurrent different build
fails closed. Memory admission and the declared `CI_TIMEOUT` apply through ccid;
provider execution has an additional 30-minute limit. This workflow does not
install compilers or run a native release matrix.

Only checked-in public synthetic `.agent/tests/skill-cases.json` cases and
`.agent/skills` instructions are sent to Groq, with the existing
`openai/gpt-oss-20b` model. Private profiles, opportunities and document artifacts
are not evaluator inputs. Verify that the configured account permits this
evaluation without paid usage before dispatch. The evaluator bounds requests,
retries and completion tokens; dispatch only when the exact source lacks valid
provider results. Do not substitute response-file fixtures for provider evidence.

The script retains JSON, Markdown and source/binary/lock identities under
`$CARGO_TARGET_DIR/ccvl-skill-eval/<commit>/run.<unique>/`, outside disposable
source scratch. It prints the evidence path and public JSON report to the Crow
log, including failed or unavailable results. CLI exit codes remain `0` (pass),
`1` (semantic failure), `2` (configuration failure) and `75` (provider unavailable);
the ccid adapter reports any nonzero result as a failed check. A timeout or absent
report also fails and supplies no semantic proof. Lint runs isolated workflow
guard fixtures only; it neither requires a credential nor calls the model.
