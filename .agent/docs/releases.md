# Compiled releases

Normal users download a platform workspace bundle from the
[latest release](https://github.com/corbet-labs/ccvl/releases/latest). Each
archive contains the exact checked-in workspace plus its optimized native
binary in `.agent/cache/ccvl/bin/`. Extract it and run the included dispatcher;
neither Git nor Rust is required.

## Required delivery path

Eligible public GitHub Actions is preferred. Crow provides manual fallback
commands for release preparation and publication. Routine pushes and pull
requests run applicable source checks; releases remain explicit manual actions.
Both providers call the same native build, package and publication scripts.
Current stable Rust is selected without an exact compiler pin; Crow uses an
already provisioned compatible compiler and never installs one implicitly.

All six real native targets are required: Linux x86_64/aarch64, macOS
x86_64/arm64 and Windows x86_64/arm64. The native command verifies the running
OS/architecture and compiler host, builds the locked native target, then runs
public document and native runtime checks before packaging. Cross-compilation
is not native evidence. Linux independent PDF verification reuses that exact
executable. Rust tests, Clippy, workflow/shell/license checks and the real
Git/Rust-free archive executor are separate required gates.

Each package includes a receipt binding its CI provider/run, exact source
commit, canonical workspace contents, source archive, Cargo.lock, compiler,
native host, runtime identity and asset checksums. The publisher requires all
six receipts and four gate receipts from the same source/dependency identity;
the Linux deep/archive gates additionally identify their exact native receipt.
GHA and Crow archives may encode tar metadata differently, so canonical source
file/mode hashes establish equal source contents across providers. Reports are
retained beside the artifacts. They are trusted CI evidence, not cryptographic
attestations against a compromised executor.

Only publication receives a release credential. It creates a complete draft,
verifies every remote asset's bytes and then makes the release public. Existing
assets are never overwritten or deleted, including draft assets. Repeated
publication verifies matching bytes and uploads only missing draft assets.
Incomplete published releases, different bytes, missing native results and
superseded main revisions fail closed. Workspace release assets include full
native/gate receipts; the runtime release has a stable runtime manifest so a
matching immutable runtime can be reused by later workspace-only revisions.

## Crow release routes

Use the existing shared `crow-ci` dispatcher for committed source staging,
verified ccid execution, memory admission and the actual persistent Cargo target
lock. Prefer an eligible available GHA execution first; do not dispatch duplicate
work or treat a real failed check as provider unavailability.

| Workflow | Action | Existing executor requirement |
|---|---|---|
| `release-linux` | Rust/lint, native Linux x86_64 package, independent PDF gate | Provisioned Linux x86_64 worker and existing tools |
| `release-archive` | Run the exact bundle with Git and Rust absent | A real minimal Linux x86_64 executor |
| `release-publish` | Verify all six packages/four gates and publish immutable releases | Linux worker, existing `gh`/`jq`, publication-only secret |

`release-linux` prints its retained directory under the locked Cargo target,
keyed by source commit and Crow run. Supply that explicit `CCVL_RELEASE_DIR` to
later stages. Other genuine native jobs use
`bash .agent/scripts/native-release.sh PLATFORM OUTPUT` and contribute their
packages/receipts to that directory. Transfer exact files without replacing a
different existing artifact; run `release-ci.sh check` before publication.
`release-publish` requires the manual-event repository secret
`ccvl_release_github_token`; it never compiles with that credential. Registry or
GitHub authorization settings are not changed by these commands.

An already provisioned sandbox can be selected for `release-archive` with the
paired `ARCHIVE_EXECUTOR` and `ARCHIVE_EXECUTOR_SHA256` inputs. The workflow
copies and verifies this standalone executable before passing it the verified
archive-check script and explicit package directory. The executor must run that
unchanged check in a real minimal filesystem and preserve its exact package and
CI identities. It must provide existing runtime tools without installing them or
exposing Git/Rust through another path. Keep host-specific sandbox configuration
outside ccvl. With neither input, the check runs directly and still requires an
already minimal worker; an incomplete or mismatched pair fails before execution.

**Executor availability remains a separate delivery prerequisite.** The current
Crow inventory has only a Linux x86_64 local executor. ARM, macOS and Windows
native workers remain unavailable. The existing Nix daemon may provide a real
minimal filesystem using only retained runtime dependencies, but its archive
gate is not yet verified. A sandbox supplied with Nix library paths proves only
that environment; it does not establish that a binary linked to those paths
works on ordinary Linux. The shared verified ccid binary also targets Linux
x86_64 only. These routes do not provision workers or turn Linux evidence into
six-platform evidence. The ordinary worker still fails the archive check because
Git/Rust are present. All-six Crow fallback remains incomplete until actual
native and minimal-executor evidence exists. A fake receipt, a masked PATH or a
Linux cross-build cannot fill that gap.

## Immutable release tags

Two immutable release tags serve different purposes:

- `runtime-<source-fingerprint>` contains the six executable downloads,
  checksums, identities, and a runtime manifest. Setup uses this exact tag.
- `build-<git-commit>` contains six workspace bundles and checksums. This is
  the user-facing latest release, including current templates, documents and
  exact native/gate receipts.

Workspace-only changes reuse a matching runtime identity while still producing
a new set of complete user bundles. A failed release leaves the last verified
download intact; a checkout with newer runtime source refuses that old binary.

## Source identity and setup

The build embeds a SHA-256 identity over `Cargo.toml`, `Cargo.lock`,
`rust-toolchain.toml`, `.agent/build.rs`, and the sorted Rust source files in
`.agent/src/`. Each input contributes its relative path and SHA-256 digest.
The Rust, Bash, and PowerShell implementations must produce the same value;
native CI tests enforce this agreement.

The launcher checks the executable's `runtime-id` before every command. The
executable independently checks its workspace, including when called directly.
CV text, Typst templates, profile data, interview evidence, and opportunities
are runtime inputs rather than compiled code and do not force recompilation.

Setup verifies both the download checksum and embedded source identity before
installing it. It never labels an arbitrary downloaded binary as current and
never silently falls back to compilation. Developers use `setup --from-source`
to request a local build with a suitable installed stable compiler, or a
repository-local stable toolchain when none meets the `Cargo.toml` minimum.
Release CI installs current stable and records its resolved compiler version.
The channel file remains part of the runtime source identity; download hashes,
source fingerprints, and all six native gates remain mandatory.

## Reusing work

The pinned Rust cache action retains dependency downloads and compiled
dependencies, keyed by OS, architecture, compiler, dependency lockfile, and
build settings. Workspace executables are rebuilt and verified on each run;
they are not trusted merely because a cache hit occurred. Test and release
caches are separate, and debug information is disabled for the CI test build
to reduce cache size. Only `main` saves caches. Successful dependency work is
also retained when a later test fails, so fixes do not repeat a cold build.

Release binaries are built once per platform per run and passed between jobs
as artifacts. Packages have a three-day Actions retention period; published
downloads are retained as release assets. Public source uses standard
GitHub-hosted runners. Private downstream content stays on the maintainer's
self-hosted workflow and is never uploaded to those runners.

Linux independent verification runs one public check and retains its freshly
verified PDFs for Poppler, QPDF and pixel comparisons. Each variant still
compiles twice inside Rust and once in a separate process. Download identity
checking uses `doctor`; complete setup remains tested in the clean archive
environment. No document-validation result is reused across CI runs.
