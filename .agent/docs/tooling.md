# Tooling

ccvl uses one native runtime and a small verification toolchain:

| Tool | Required for |
|---|---|
| ccvl Rust binary | schemas, compilation, formatting, PDF checks, ATS text, fonts, and reproducibility |
| Rust 1.94.0 + Cargo | explicit developer builds with `setup --from-source` |
| Poppler + QPDF | secondary Linux CI validation |

Run `bash ./ccvl bootstrap` on Linux/macOS or `.\ccvl.cmd bootstrap` on Windows
for a read-only setup plan. The matching `setup` command installs only the precompiled runtime whose embedded
source identity matches the workspace, then runs the complete check. Platform
bundles already include this executable below `.agent/cache/ccvl/`. It supports Linux and macOS on x86_64/aarch64 plus native Windows on
x86_64/ARM64.

For development, `setup --from-source` builds using the exact Rust toolchain.
An experienced user may provide that toolchain independently and run
the platform `doctor` plus `check` commands. A matching global toolchain is
reused; otherwise the repository-local path takes precedence only while ccvl
runs.

## Line measurement

Run `bash ./ccvl measure` or `.\ccvl.cmd measure` after changing CV or
cover-letter text. Add `--all` to print every actual, target, and allowed fill
percentage. The command measures the real Archivo glyph width inside each
Typst container. Underfill and overflow return a non-zero exit status and an
instruction to rewrite and repeat the measurement.

## Watch mode

`bash ./ccvl watch-cv <locale> [pages]`, `bash ./ccvl watch-cl <locale>`,
and `bash ./ccvl watch-opportunity <organisation-key> <position-key>`
rebuild on every change instead of exiting. CV and cover-letter watchers accept
`--substyle <name>`; omission selects the family's configured default.
The watcher hashes the style-major
locale templates (`cvl/cv/` and `cvl/cl/` substyle trees with their `typst/`
pointers), the shared `.agent/typst` machinery, `cvl/profile.toml`,
`ccvl.json`, the relevant record, and the generated opportunity
`typst/*.typ` copies; any change re-renders the PDFs (plus the resolved
`.typ` copies for opportunities). Built PDFs are excluded from the hash so a
render never retriggers itself. The loop uses the embedded engine and a
standard-library polling interval, so no extra runtime or file-watching
dependency is needed. `just watch <organisation-key> <position-key>`
delegates to `watch-opportunity`; the `justfile` lists the remaining
shortcuts.

## Bundled fonts

Archivo is included directly in the repository and embedded in the native
binary together with the Typst 0.15.1 compiler and Typstyle 0.15.1 formatter.
Use `bash ./ccvl build` or `.\ccvl.cmd build`; no external document runtime or
system font is used. The platform `check` command treats any Typst diagnostic or
non-Archivo PDF font as a failure.

Do not widen permissions, enable package lifecycle scripts, add hidden hooks,
or weaken `.gitignore` as part of setup.
