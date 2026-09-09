# Selecting CI checks

On a provisioned build worker, use `bash .agent/scripts/ci-check.sh <check>...`:

| Check | Coverage |
|---|---|
| `rust` (default) | Exact Rust 1.94.0 formatting, locked unit/document tests and Clippy |
| `lint` | Actionlint, ShellCheck, and REUSE with preinstalled tools |
| `documents` | Locked Linux release build and independent PDF/text/layout verification |

The command uses existing tools and defaults to one Cargo job and two Rust
test threads. Run only checks whose inputs changed or whose results are
missing. Private downstream data must remain on trusted internal workers.
Both rustup-managed and directly provisioned exact Rust versions are supported.
An explicitly selected `RUST_TOOLCHAIN` override permits supplementary checks
with another installed version; the compiler identity is printed and those
results do not replace the pinned 1.94.0 release gate. No implicit fallback or
toolchain installation occurs.

The manual Crow `verify` workflow accepts `CHECKS=rust`, `lint`, or `documents`.
Its required execution inputs are `SOURCE_ARCHIVE` and `SOURCE_SHA256`: stage
a `git archive` of the exact committed revision on the worker, and dispatch
that same revision. The pipeline verifies the archive hash and embedded Git
commit against `CI_COMMIT_SHA` before extracting any source. Declared variables
have empty defaults for Crow configuration compatibility; missing source
inputs fail closed during execution.
Crow uses `ci-targets/ccvl` under `CARGO_HOME` (or `$HOME/.cargo`) for compiled
output; an optional `CARGO_TARGET_DIR` overrides that writable project cache.
Crow serializes access with a one-minute lock wait, and bounds each selected
check to 45 minutes with a 30-second forced-termination grace. An omitted
cache variable uses the worker's Cargo home, which must persist for reuse.

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
