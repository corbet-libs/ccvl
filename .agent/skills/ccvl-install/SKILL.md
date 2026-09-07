---
name: ccvl-install
description: Prepare or repair the native ccvl binary and locked Rust toolchain required to build and verify documents.
---

# Install ccvl tooling

Establish the smallest working toolchain for the current host and prove it by
rendering the checked-in general CVL.

## Workflow

1. Detect the host. Run `bash ./ccvl bootstrap` on Linux/macOS or
   `.\ccvl.cmd bootstrap` on native Windows. Do not improvise a parallel
   installer, require Git knowledge, or route Windows users through WSL.
2. If it reports that the prebuilt binary or the exact Rust toolchain plus
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
   repository-local `ccvl` binary is built from `Cargo.lock` with Rust
   1.94.0; when that exact toolchain is not already available, the managed
   toolchain belongs in the repository-local cache.
6. Never widen filesystem permissions, enable package lifecycle scripts, add a
   hidden hook, or weaken `.gitignore` to make setup pass.
7. Confirm that the bundled Archivo files are real fonts and that the embedded
   document engine discovers all four variants.
8. Require the full setup check before starting profile or document edits.

Use the repository commands for rendering. The native binary embeds the Typst
0.15.1 compiler, Typstyle 0.15.1 formatter, and font pack; it neither needs an
external document runtime nor discovers system fonts.

The required tools and their roles are listed in `.agent/docs/tooling.md`.
The pinned Rust toolchain, `Cargo.lock`, and checksum-pinned bootstrap are
authoritative for all six supported OS/architecture pairs. Prefer the
non-privileged, repository-local bootstrap; use a native package manager only
for a missing bootstrap command or compiler prerequisite it cannot provide. Do
not introduce containers or an application database for this file-native
product.
