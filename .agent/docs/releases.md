# Compiled releases

Normal users download a platform workspace bundle from the
[latest release](https://github.com/corbet-labs/ccvl/releases/latest). Each
archive contains the exact checked-in workspace plus its optimized native
binary in `.agent/cache/ccvl/bin/`. Extract it and run the included dispatcher;
neither Git nor Rust is required.

## Required delivery path

Every push to `main` runs the CI workflow. Its reusable Binaries job builds all
six standard native runner targets: Linux x86_64/aarch64, macOS x86_64/arm64,
and Windows x86_64/arm64. Every executable runs the public document checks and
runtime identity tests. The same Linux executable feeds the independent PDF
checks and the minimal-container archive test. Rust unit tests, Clippy, shell
and workflow lint, and license checks also gate publication.

Only the final CI job may publish. It downloads the artifacts from that exact
run, requires all six identities and checksums, uploads a complete draft, then
makes it public. A failed platform prevents publication. A superseded source
revision cannot replace the latest release. Published assets are never
overwritten or deleted by the workflow.

Two immutable release tags serve different purposes:

- `runtime-<source-fingerprint>` contains the six executable downloads,
  checksums, identities, and a manifest. Setup uses this exact tag.
- `build-<git-commit>` contains six workspace bundles and checksums. This is
  the user-facing latest release, including current templates and documents.

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
to request a local build with the pinned toolchain.

## Reusing work

The pinned Rust cache action retains dependency downloads and compiled
dependencies, keyed by OS, architecture, compiler, dependency lockfile, and
build settings. Workspace executables are rebuilt and verified on each run;
they are not trusted merely because a cache hit occurred. Test and release
caches are separate, and debug information is disabled for the CI test build
to reduce cache size. Only `main` saves caches.

Release binaries are built once per platform per run and passed between jobs
as artifacts. Packages have a three-day Actions retention period; published
downloads are retained as release assets. Public source uses standard
GitHub-hosted runners. Private downstream content stays on the maintainer's
self-hosted workflow and is never uploaded to those runners.
