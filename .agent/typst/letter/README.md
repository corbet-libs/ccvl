# letter/ — correspondence library modules (vendored)

Typst sources vendored from the correspondence family. Both modules are
adopted by the locale templates (`cvl/de-ch/cl.typ`, `cvl/en-ch/cl.typ`).
Do not edit vendored files by hand — re-copy from the pinned versions
below.

| File | Source | Version |
|------|--------|---------|
| `farewell.typ`, `generated/closing.typ` | `corbet-labs/cfarewell` `typst/` | 0.1.1 |
| `ink.typ` | `corbet-labs/cink` `typst/` | 0.1.1 |

Why vendored: the embedded Typst engine compiles files inside this
workspace only. It cannot fetch Typst sources from a registry at render
time (ccvl renders offline and deterministically), so the Typst side of
each library is copied here and pinned. The Rust side needs no vendoring:
`Cargo.toml` depends on the same libraries from crates.io.

Why only these two: `cgreet` (salutations) ships Rust + TypeScript only,
with no Typst module — the matching rules are mirrored for the renderer
in `.agent/typst/application.typ` (`salutation-honorific`,
`salutation-titles`, `salutation-surname`, `de-salutation`) and the Rust
side re-exports `cgreet` from `ccvl::application`. `cletter` (openings,
subjects, locale resolution) is intentionally not vendored: ccvl's
Harvard subjects are bespoke and its records carry an explicit locale,
so there is nothing to adopt. The Rust binary therefore depends only on
`cgreet`; `cfarewell`/`cink` contribute Typst only.

Re-vendor procedure: copy the files, update this table, run the full
`check` gate — any render delta fails closed.
