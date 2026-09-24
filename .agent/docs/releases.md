# Compiled releases

Normal users download the Linux x86_64 workspace bundle from the
[latest release](https://github.com/corbet-libs/ccvl/releases/latest). Each
archive contains the exact checked-in workspace plus its optimized native
binary in `.agent/cache/ccvl/bin/`. Extract it and run the included dispatcher;
neither Git nor Rust is required.

## Required delivery path

Eligible public GitHub Actions remains supported and preferred when available.
Crow independently provides release preparation and publication; delivery does
not wait for Actions. Routine GitHub pushes and pull requests run applicable
source checks when Actions is available; Crow provides explicit equivalent
checks. Releases remain manual actions. Both providers call the same native
build, package and publication scripts.
Current stable Rust is selected without an exact compiler pin; Crow uses an
already provisioned compatible compiler and never installs one implicitly.

The explicit released platform set is **Linux x86_64**, declared in
`.agent/release-platforms.txt` and listed by
`python3 .agent/scripts/release-evidence.py platforms`. Missing artifacts never
shrink that set. Linux ARM64, both macOS architectures and both Windows
architectures have no verified prebuilt release. Their native developer paths
and Actions runner mappings remain available. Any future advertised target
needs its own native proof; either provider can supply eligible evidence.

For every released target, the native command verifies the running
OS/architecture and compiler host, builds the locked native target, then runs
public document and native runtime checks before packaging. Cross-compilation
is not native evidence. Linux independent PDF verification reuses that exact
executable. Rust tests, Clippy, workflow/shell/license checks and the real
Git/Rust-free archive executor are separate required gates.

Each package includes a receipt binding its CI provider/run, exact source
commit, canonical workspace contents, source archive, Cargo.lock, compiler,
native host, runtime identity and asset checksums. The publisher requires every
released platform's receipt and four gate receipts from the same source and
dependency identity;
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
lock. Prefer an eligible available GHA execution where it can complete the
selected work; otherwise use Crow without waiting for GHA. Do not dispatch
duplicate work or treat a real failed check as provider unavailability.

| Workflow | Action | Existing executor requirement |
|---|---|---|
| `release-linux` | Rust/lint, native Linux x86_64 package, independent PDF gate | Provisioned Linux x86_64 worker and existing tools |
| `release-archive` | Run the exact bundle with Git and Rust absent | A real minimal Linux x86_64 executor |
| `release-publish` | Verify every released package and all four gates, then publish immutable releases | Linux worker, existing `gh`/`jq`, publication-only secret |

`release-linux` prints its retained directory under the locked Cargo target,
keyed by source commit and Crow run. Supply that explicit `CCVL_RELEASE_DIR` to
later stages. Native jobs use
`bash .agent/scripts/native-release.sh PLATFORM OUTPUT` and contribute their
packages/receipts to that directory. A future platform must be explicitly added
to release policy before it is advertised. Transfer exact files without
replacing a different existing artifact; run `release-ci.sh check` before publication.
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

**Executor availability remains a separate delivery prerequisite.** Verify the
actual native OS/architecture, existing tools and minimal filesystem for each
run. Keep host inventory and provisioning details outside ccvl. These routes do
not provision workers or turn Linux evidence into another platform's evidence.

A binary linked to Nix store paths needs a separately verified ordinary-Linux
runtime path before distribution. When adapting a retained native executable,
preserve its original compile provenance, record the exact transformation and
run public/runtime and Linux deep checks on the final bytes before publication.
The unchanged archive check must then run that exact bundle in a real minimal
filesystem with its ordinary runtime libraries and Git/Rust absent. Record the
tested OS/libc scope: the released Linux x86_64 path is verified on Ubuntu 24.04
with glibc 2.39. This does not establish compatibility with every Linux
distribution or older libc. A sandbox exposing Nix dependencies proves only that
environment. Neither a masked PATH nor a Linux cross-build supplies missing
native or ordinary-system compatibility evidence.

## Immutable release tags

Two immutable release tags serve different purposes:

- `runtime-<source-fingerprint>` contains the released platforms' executables,
  checksums, identities, and a runtime manifest. Setup uses this exact tag.
- `build-<git-commit>` contains the released platforms' workspace bundles and
  checksums. This is the user-facing latest release, including current templates, documents and
  exact native/gate receipts.

Workspace-only changes reuse a matching runtime identity while still producing
a new set of complete user bundles. A failed release leaves the last verified
download intact; a checkout with newer runtime source refuses that old binary.
The runtime manifest also binds the released platform set. Widening that set
cannot overwrite an existing immutable runtime release; its tag and asset
identity must be resolved explicitly before publication.

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
GHA release CI installs current stable; Crow reuses a provisioned compatible
compiler. Both record the actual compiler version.
The channel file remains part of the runtime source identity; download hashes,
source fingerprints, every released target's native checks and all four shared
gates remain mandatory.

## Reusing work

Crow retains dependencies in dedicated, locked Cargo storage keyed by repository
identity. On GHA, the pinned Rust cache action retains dependency downloads and
compiled dependencies, keyed by OS, architecture, compiler, dependency
lockfile, and build settings; only `main` saves these separate test and release
caches. Workspace executables receive current source-bound verification on
each run; a cache hit alone is not a passing check. Debug information is
disabled for the CI test build to reduce cache size. Successful dependency work
remains available when a later test fails, so fixes do not repeat a cold build.

Release binaries are built once per released platform per run and retained for
later gates and publication. GHA artifacts have a three-day retention period;
Crow retains packages under its explicit release directory. Published downloads
remain release assets. Eligible public source can use GitHub-hosted runners.
Private downstream content stays on trusted internal workers, with its own
ownership and document gates.

Linux independent verification runs one public check and retains its freshly
verified PDFs for Poppler, QPDF and pixel comparisons. Each variant still
compiles twice inside Rust and once in a separate process. Download identity
checking uses `doctor`; complete setup remains tested in the clean archive
environment. No document-validation result is reused across CI runs.
