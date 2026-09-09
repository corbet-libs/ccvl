# Selecting CI checks

On a provisioned build worker, use `bash .agent/scripts/ci-check.sh <check>...`:

| Check | Coverage |
|---|---|
| `rust` (default) | Exact Rust 1.94.0 formatting, locked unit/document tests and Clippy |
| `lint` | Actionlint, ShellCheck, and REUSE with preinstalled tools |
| `documents` | Locked Linux release build and independent PDF/text/layout verification |

The command uses existing tools. Crow supplies a memory-bounded parallel job
and test-thread budget through the shared `ccid` adapter. A direct invocation
without those environment settings retains conservative script defaults.
Run only checks whose inputs changed or whose results are
missing. Private downstream data must remain on trusted internal workers.
Both rustup-managed and directly provisioned exact Rust versions are supported.
An explicitly selected `RUST_TOOLCHAIN` override permits supplementary checks
with another installed version; the compiler identity is printed and those
results do not replace the pinned 1.94.0 release gate. No implicit fallback or
toolchain installation occurs.

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
