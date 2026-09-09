# Selecting CI checks

GitHub Actions is preferred when available for public source checks. Pushes to
`main` and pull requests run Rust and lint checks automatically. Six-platform
release preparation and publication remain an explicit manual dispatch. Crow
provides manually selected fallback checks when Actions is unavailable; do not
run both providers for the same validation without a missing result or changed
input. Private downstream data and credential-bearing jobs stay on trusted
internal workers.

On a provisioned build worker, use `bash .agent/scripts/ci-check.sh <check>...`:

| Check | Coverage |
|---|---|
| `rust` (default) | Stable Rust formatting, locked unit/document tests and Clippy |
| `lint` | Actionlint, ShellCheck, and REUSE with preinstalled tools |
| `documents` | Locked Linux release build and independent PDF/text/layout verification |

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
bootstrap and CI-selector behavior tests run with lint; Windows bootstrap
selection is also tested in the native release matrix.

The manual Crow `ccid` workflow accepts `CHECKS=rust`, `lint`, or `documents`.
Its `.ci/ccid.toml` selectors invoke the same commands. The operator submission
helper stages the exact committed source archive, verifies locally available
LFS and submodule inputs, and supplies the pinned shared tool archive and
binary with their SHA-256 digests. Missing inputs fail closed. The adapter
checks the source commit against `CI_COMMIT_SHA` before execution.

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

This provides selected Linux validation during a hosted-provider outage. It
does not create the six native release bundles, test the Git-free user archive,
prove macOS/Windows behavior, or publish releases. All native publication gates
in `releases.md` remain required. A registry dependency that is not published
still blocks a locked build; a provisional path-patched lock is not release
evidence. Private sync and external model evaluation require their own trusted
workflows and are not part of this validation pipeline.

The manual Crow `downstream-sync` workflow stages a verified source archive and
a Git bundle containing the same commit and its ancestry. It builds the public
gate with existing Rust in a dedicated, locked cache, then merges into the
private downstream and runs its boundary and document checks. Immediately before
pushing it rechecks public `main`; changed or unavailable upstream metadata
prevents publication. Existing private SSH credentials remain on the internal
worker. The operator helper reads `.ci/archives.toml` to stage Git history only
for this workflow. Lint includes isolated tests for stale upstream and failed
document gates; those fixtures do not substitute for a real downstream run.
