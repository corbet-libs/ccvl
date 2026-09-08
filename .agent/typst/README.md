# Shared document library

This directory owns the public profile, application validation, reusable
components, and bundled font set. Opportunity-specific content is data and
must not be embedded in layout components.

## Styles

The style-major tree below `cvl/` owns the render-style axis. Each document
family has a registry, a measurement contract, shared base knobs, and one
knob delta per substyle:

```text
cvl/cv/style.toml                 CV registry (substyles, default, locales, pages)
cvl/cv/contract.toml              CV measurement and layout contract
cvl/cv/standard/substyle.toml     default CV look (empty delta over shared base)
cvl/cv/compact/substyle.toml      thin whitespace-only CV variant
cvl/cl/style.toml                 cover-letter registry
cvl/cl/contract.toml              cover-letter measurement and rhythm contract
cvl/cl/left-rule/substyle.toml    default panel (single left rule)
cvl/cl/frame/substyle.toml        full-border panel variant
cvl/shared/defaults.toml          locale-free base knobs (page, text, accents, …)
cvl/shared/style.typ              knob merge plus page setup for every leaf
cvl/cv/src/cv.typ                 shared CV renderer (element styles, letterhead)
cvl/cv/src/entries-de.typ         German CV entries (personal showcase content)
cvl/cv/src/entries-en.typ         English CV entries (personal showcase content)
cvl/cl/src/cl.typ                 shared cover-letter renderer
cvl/<doc>/<substyle>/<lang>/ch/   one leaf: content.toml, strings.toml, typst/, pdf/
```

`standard`/`left-rule` are the defaults and preserve the long-standing
Harvard-style hierarchy. `compact`/`frame` prove the plumbing: same
renderers, same contracts, only whitespace or panel geometry changed. Each
record selects its look per document through `options.cv_substyle` and
`options.cl_substyle` (records without the fields render the defaults);
`render.rs` resolves the names and injects the leaf's `strings`, `substyle`,
and `shared-defaults` paths as Typst inputs, and the shared renderers merge
that leaf's knobs through `merge-style`.

The single source of truth for the defaults and the available names is
`cvl/cv/style.toml` and `cvl/cl/style.toml`. Adding a substyle means adding
one `substyle.toml` delta plus one leaf directory per locale — never forking
a renderer. Unknown names fail in Rust validation before the Typst compile.

All substyles satisfy the same line and vertical-rhythm contracts in
`cvl/cv/contract.toml` and `cvl/cl/contract.toml`, so every leaf passes the
same `measure`/`check` gates. Horizontal measure is style-invariant: page
margins, base text size, bullet indent, and highlight geometry keep the
Harvard values in every delta, because changing them reflows every measured
line. Substyles vary vertical whitespace and panel geometry only.
