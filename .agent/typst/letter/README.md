# letter/ — correspondence library modules (vendored)

Typst sources vendored from the correspondence family. Nothing here is
imported by the locale templates yet (adoption is incremental); these
files only make the libraries available. Do not edit vendored files by
hand — re-copy from the pinned versions below.

| File | Source | Version |
|------|--------|---------|
| `farewell.typ`, `generated/closing.typ` | `corbet-labs/cfarewell` `typst/` | 0.1.1 |
| `ink.typ` | `corbet-labs/cink` `typst/` | 0.1.1 |
| `letter.typ`, `generated/tables.typ` | `corbet-labs/cletter` `typst/` | 0.1.1 |

The Rust side depends on the same versions from crates.io (`cgreet 0.2`
additionally). Re-vendor procedure: copy the files, update this table,
run the full `check` gate — any render delta fails closed.
