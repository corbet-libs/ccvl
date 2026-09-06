# Shared document library

This directory owns the public profile, application validation, document
styles, reusable components, and bundled font set. Opportunity-specific content
is data and must not be embedded in layout components.

## Styles

`styles/` owns the render-style axis. A style is one Typst renderer plus one
TOML knob file:

```text
.agent/typst/styles/harvard.typ         default page setup + style registry
.agent/typst/styles/harvard.toml        default whitespace and accent knobs
.agent/typst/styles/harvard-compact.typ thin whitespace-only variant
.agent/typst/styles/harvard-compact.toml
.agent/typst/styles/document.typ        back-compat re-export of harvard.typ
```

`harvard` is the default and preserves the long-standing Harvard-style
hierarchy. `harvard-compact` proves the plumbing: same templates, same
contracts, only vertical whitespace tightened. Each record selects its style
through `options.style` (records without the field render as `harvard`);
`render.rs` resolves the name and injects it as the `style` Typst input, and
the templates load that style's knobs through `load-style`.

The single source of truth for the default and the available names is the
`styles` section in `ccvl.json`. Adding a style means adding
`<name>.typ` plus `<name>.toml` here and listing `<name>` there — never
forking a locale template. Unknown names fail in Rust validation and in
Typst with the available list.

All styles satisfy the same line and vertical-rhythm contracts in
`ccvl.json`, so every style passes the same `measure`/`check` gates.
Horizontal measure is style-invariant: page margins, base text size, bullet
indent, and highlight geometry keep the Harvard values in every style,
because changing them reflows every measured line. Styles vary vertical
whitespace and accents only.
