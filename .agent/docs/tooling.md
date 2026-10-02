# Tooling

ccvl uses one native runtime and a small verification toolchain:

| Tool | Required for |
|---|---|
| ccvl Rust binary | schemas, compilation, formatting, PDF checks, ATS text, fonts, and reproducibility |
| Stable Rust + Cargo (minimum in `Cargo.toml`) | explicit developer builds with `setup --from-source` |
| Poppler + QPDF | secondary Linux CI validation |

Run `bash ./ccvl bootstrap` on Linux x86_64 for a read-only setup plan. The
matching `setup` command installs only the precompiled runtime whose embedded
source identity matches the workspace, then runs the complete check. The Linux
x86_64 bundle includes this executable below `.agent/cache/ccvl/`; its ordinary
Linux runtime is verified on Ubuntu 24.04 with glibc 2.39. Linux ARM64, macOS and
Windows have no verified prebuilt release.

For development, `setup --from-source` reuses an installed stable compiler
meeting the `Cargo.toml` minimum, including a newer system version. The plan
reports the actual compiler version. An experienced user may provide Rust
independently and run the platform `doctor` plus `check` commands. If no suitable
compiler exists, explicit developer setup installs the stable channel in the
repository-local cache; it does not replace the global toolchain. Normal users
continue to use the matching precompiled runtime without Rust. Native source
build paths remain available for Linux x86_64/ARM64, macOS Intel/Apple Silicon
and Windows x86_64/ARM64; their presence is not native release evidence. Use
`bash ./ccvl setup --from-source` on Linux/macOS or
`.\ccvl.cmd setup --from-source` on Windows only for an explicitly requested
developer build.

Claude Code on the web runs `.claude/hooks/session-start.sh` at session start
(registered in `.claude/settings.json`; it does nothing outside the web). It
installs missing CI packages (`file`, Poppler, QPDF, `jq`, ShellCheck), CI's
pinned actionlint into `.agent/cache/tools/bin`, and the precompiled runtime
matching the checkout through `bash ./ccvl setup`. When the engine source has
no matching release yet, it says so and leaves the explicit developer build,
`bash ./ccvl setup --from-source`, to the session.

## Line measurement

Run `bash ./ccvl measure` or `.\ccvl.cmd measure` after changing CV or
cover-letter text. Add `--all` to print every actual, target, and allowed fill
percentage. The command measures the selected font’s real glyph width inside each
Typst container. Underfill and overflow return a non-zero exit status and an
instruction to rewrite and repeat the measurement.

## Watch mode

`bash ./ccvl watch-cv <locale> [pages]`, `bash ./ccvl watch-cl <locale>`,
and `bash ./ccvl watch-opportunity <organisation-key> <position-key>`
rebuild when an input changes and stay running after compile errors. CV and cover-letter watchers accept
`--style <name>` and `--substyle <name>`; omissions use the record or configured
defaults. Letters also accept `--pages <count>`.
Both document watchers accept `--paper <name>`; precedence is the command-line
selection, the record's document paper, then the style's locale default.
Unsupported papers fail explicitly and the watcher waits for corrected inputs.
The watcher observes the files read while resolving and compiling the selected
document: workspace and style metadata, wording, settings, profile, declared
fonts, and transitive Typst imports and data/image reads. It hashes these inputs
every 500 ms without an extension filter or prescribed renderer folder layout.
Unrelated styles and generated outputs do not trigger a rebuild merely by
changing. A PDF or image explicitly read by a renderer is still an input.
Rebuilding an opportunity refreshes its PDFs and resolved `.typ` copies.

Missing files and symlink replacement are detected. After an error, the watcher
retains the last working dependencies plus newly attempted inputs so fixing or
creating an input recovers automatically. A successful build replaces that set.
Dependency discovery or an edit during compilation causes one settling build
before the output is reported, preventing a concurrent edit from being missed.
The loop uses the embedded engine and standard-library polling, so no extra
runtime or file-watching dependency is needed. `just watch <organisation-key> <position-key>`
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
