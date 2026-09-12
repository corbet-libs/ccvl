---
name: ccvl-install
description: Prepare or repair the native ccvl binary and stable Rust toolchain required to build and verify documents.
---

# Install ccvl tooling

Establish the smallest working toolchain for the current host and prove it by
rendering the checked-in general CVL.

## Workflow

1. Detect the host. Released binaries are available for Linux x86_64, verified
   on Ubuntu 24.04 with glibc 2.39. Run `bash ./ccvl bootstrap` there. For an
   explicitly requested developer source build, the native dispatchers remain
   `bash ./ccvl` on Linux/macOS and `.\ccvl.cmd` on Windows. Other platforms
   have no verified prebuilt release. Do not improvise a parallel installer,
   require Git knowledge, or route Windows users through WSL.
2. If it reports that the prebuilt binary or a suitable stable Rust toolchain plus
   repository-local binary are ready, run the matching platform `check`
   command and stop changing the environment.
3. If the user explicitly requested setup or installation, run the matching
   platform `setup` command: it fetches the checksum-verified prebuilt
   binary matching the workspace runtime identity. A missing or mismatched
   download must fail without a source-build fallback. Use `setup --from-source`
   only for an explicitly requested developer build. Otherwise show the plan
   before its changes.
4. If the harness cannot support the platform, report its exact boundary and
   use `.agent/docs/tooling.md`; do not guess package names.
5. Do not replace an existing package strategy or working global toolchain. The
   repository-local `ccvl` binary is built from `Cargo.lock` with stable Rust
   meeting the `Cargo.toml` minimum. Reuse a suitable installed compiler; when
   none is available, the stable toolchain belongs in the repository-local cache.
6. Never widen filesystem permissions, enable package lifecycle scripts, add a
   hidden hook, or weaken `.gitignore` to make setup pass.
7. Confirm that the bundled Archivo files are real fonts and that the embedded
   document engine discovers all four variants.
8. Require the full setup check before starting profile or document edits.

Use the repository commands for rendering. The native binary embeds the Typst
0.15.1 compiler, Typstyle 0.15.1 formatter, and font pack; it neither needs an
external document runtime nor discovers system fonts.

The required tools and their roles are listed in `.agent/docs/tooling.md`.
The stable Rust channel, `Cargo.toml` minimum, `Cargo.lock`, and checksum-pinned bootstrap
govern the retained native developer build paths. Those paths do not establish
verified binary availability; the released platform set and native gates are in
`.agent/docs/releases.md`. Prefer the
non-privileged, repository-local bootstrap; use a native package manager only
for a missing bootstrap command or compiler prerequisite it cannot provide. Do
not introduce containers or an application database for this file-native
product.
