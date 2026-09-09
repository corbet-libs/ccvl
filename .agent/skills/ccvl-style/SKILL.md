---
name: ccvl-style
description: Create or revise ccvl document styles, substyles, locale layouts and presentation defaults, with rendered examples and contract checks. Use for visual design and style architecture; use ccvl-cv or ccvl-apply for wording in an existing style.
---

# Create a document style

Read `.agent/docs/styles.md` for the interface and
`.agent/docs/typst-defaults.md` before choosing presentation settings.
Inspect the actual selected style and current `list-documents` output.

A style owns composition, not candidate facts or universal writing policy.
Read [editorial guidance](../../docs/editorial.md) when the design exposes
content roles. AIDA is optional and independent of visual style: map it into
this style's valid fields only when selected, without importing Harvard's
six-paragraph counts. Keep national spelling/greeting/closing rules upstream in
cletter/family, while retaining explicit user and style overrides.

## Establish the design

Use the user's stated document types, visual direction, locales, paper sizes,
page presets and substyles. Resolve unspecified choices from the task and
state the assumptions. Ask only when a missing preference materially changes
the result and cannot be inferred. English is a language, not a paper size:
confirm or explicitly choose A4, US Letter or custom dimensions per locale.
Do not assume US Letter from a language-only `en` alias (which maps to en-ch).
Declare the paper sizes the style supports and an explicit default for each
locale. A document can select another supported size without another folder
level or a new substyle. Reject unsupported selections; never shrink text or
change the requested page count automatically to make another paper size fit.

A style owns the visual system: page geometry, fonts, composition and content
fields. A substyle is a variation within that style. Create
`cvl/<cv|cl>/<style>/<substyle>/<language>/<country>/`; never put a substyle
beside its parent style. Independent styles need not import Harvard or use
`src/`. Share code only where it actually belongs to the same design.

## Implement the interface

- Author locale identifiers entirely in lowercase, preserving proper names,
  exact quotations and original evidence when changing metadata.
- Declare the style ID, document, locales, page presets/default and
  substyles/default in `style.toml`. Preserve existing workspace defaults
  unless the user asks to change them.
- Provide each leaf's `content.toml`, `strings.toml`, `layout.toml` and
  `typst/<cv|cl>.typ`, plus each substyle's configuration. Keep chrome in
  strings and language/region/direction in layout. Keep named paper presets and
  their locale defaults in the style definition.
- Share wording only between substyles of the same document/style/locale.
  Put the owning `[cv]` or `[cl]` table in the style's
  `content/<language>/<country>/wording.toml`; declare its relative path under
  `[wording].source` in each leaf. Keep record metadata and explicit exceptions
  local. Nested fields merge; arrays replace whole arrays. Never reference
  another style's wording or import public showcase wording into an opportunity.
  Use `load-application` from `.agent/typst/application.typ` so direct Typst
  compilation resolves the same content as the CLI.
- Put explicit page, text, paragraph and block decisions in style-owned
  defaults. The optional `.agent/typst/document.typ` adapter applies those
  values; it does not choose a design. Declare `settings_adapter = "document-v1"`
  when using it; the engine and direct Typst adapter share one validation schema.
  Use `paragraph.leading_em` as the single line-spacing authority, and
  `explain-style <cv|cl> <locale> --style <id> --substyle <id>` to inspect merged
  inputs and their sources. Independent renderers keep their own schema.
  Retain `auto` only as a deliberate,
  documented algorithm choice. Set spacing/insets/shape strokes explicitly
  for components introduced by the style.
- The supplied `layout` input must reach the renderer; writing metadata
  without consuming it is not implementation. Ensure resolved opportunity
  copies use input defaults so they also compile inside the workspace.
- Declare style-owned content fields and useful PDF/metric rules. Use
  `[pdf.by_locale.<locale>]` for locale-specific dimensions. Match expected
  sizes to orientation and actual output. Never weaken Harvard's contracts
  to accommodate an unrelated style.
- Provide blank document fields in `scaffold.toml`. Reuse approved content
  only at its factual scope. Invented demonstration text is allowed only
  when requested and must be unmistakably labelled as a demonstration.
  Never turn sample claims into a real candidate's record.

## Verify and make review easy

Render every added or affected substyle, locale and page preset. Run the
platform `measure`, `check` and `public-check` commands, plus available
independent PDF checks. Check page dimensions, font names, page count,
extracted text, glyph coverage and all rendered pages. Exercise US Letter
separately from A4; a locale label alone proves nothing about paper size.
Exercise each declared paper size, including a nondefault selection within the
same locale. Keep existing default output names and use distinct names for
alternate-size PDFs in the same output directory.
Compare substyles with the same content so their differences are visible.

Update the style gallery, folder overview, reproducible preview instructions
and license coverage. Link to actual PDFs and explain what varies between
the style and its substyles. Generic changes go upstream to ccvl, then merge
downstream under the established delivery workflow; private records stay
in applications. Follow `.agent/AGENT.md` for release completion.


A style review checks both mechanical rules and every affected rendered page.
If independent evidence/editorial review is requested, use
[the review protocol](../../docs/review.md) and `ccvl-review` with fresh context
and actual artifacts; never describe an author self-check as independence.
General demonstration documents use their declared purpose without a fictional
vacancy. Preserve existing content truth, factual qualifiers and user choices
while changing presentation. A missing image or exhausted correction allowance
is incomplete coverage, not a visual pass.
