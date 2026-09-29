# Explicit document defaults

This audit targets the pinned **Typst 0.15.1**, through **ctypst 0.3.2** in
`Cargo.lock`. Revisit it when either dependency changes. It covers settings
used by the shipped text documents, their components, and PDF export; it is
not a universal schema for every possible Typst element.

## Where choices live

1. `cvl/shared/<style>/defaults.toml`: that family's base design, where its
   CV and CL deliberately share settings.
2. `cvl/<document>/<style>/<substyle>/substyle.toml`: substyle overrides.
3. `<substyle>/<language>/<country>/layout.toml`: locale overrides.
4. The selected named paper preset in `style.toml`: explicit paper settings
   and expected PDF dimensions, with a declared default per locale.
5. The renderer: explicit component styling, such as heading size, grid
   columns, borders, internal spacing and intentional per-region overrides.

The shipped renderers deep-merge these files in that order. `layout` is an
optional engine input; the entry point must pass it through. Locale strings
remain in `strings.toml`, separate from geometry. The optional
`.agent/typst/document.typ` adapter maps data to Typst settings without choosing
a font, paper or language. A new style can implement a different settings
schema, renderer arrangement or composition without using this adapter.

Styles using this adapter declare `settings_adapter = "document-v1"` in
`style.toml`. A single rule file, `.agent/typst/document-settings.json`, is read
by both the Rust preflight and Typst adapter. Unknown keys in its four reserved
tables, invalid enums or types, missing required values and reversed bounds
fail explicitly. Rust checks each source layer before merging, so an invalid
value cannot hide behind a later override. Style-owned extension tables stay
unrestricted. Native Typst still checks its own paper names, scripts, font
metrics and rendering constraints.

`bash ./ccvl explain-style cv en-ch --style cluster --substyle d-plus`
reports the merged inputs and source path for each effective value, without
rendering. It covers the declared adapter merge, not component-level Typst
show/set rules. For example, `page.paper` comes from the chosen paper preset, while
`text.font` comes from the family defaults.

The adapter accepts `left`, `center`, `right`, `start` and `end` block alignment;
page binding accepts `left` or `right`; CJK–Latin spacing accepts `auto` or
`none`. Misspellings fail rather than selecting another value. Numeric font
weights are restricted to 100–900, avoiding Typst's silent clamping. Custom
paper requires both dimensions; named paper rejects unused custom dimensions.

**Language does not determine paper.** Harvard and Cluster explicitly choose A4
for their supported locales. A text-only Rust probe exercises US Letter and A4
in either `en-ch` or `en-us`; the engine has no country-to-paper rule.
A command-line paper overrides the record's optional
`cv_paper` or `cl_paper`, which overrides the style's locale default.
The short `en` CLI alias means `en-ch` and
never implies US Letter. Orientation is a separate `flipped` setting.

| Paper | Portrait millimetres | Portrait PDF points | Landscape PDF points |
| --- | --- | --- | --- |
| A4 | 210 × 297 | 595.2756 × 841.8898 | 841.8898 × 595.2756 |
| US Letter | 215.9 × 279.4 | 612 × 792 | 792 × 612 |

`paper = "custom"` in the adapter takes explicit `width_mm` and `height_mm`.
The selected paper preset declares expected PDF dimensions independently of
its renderer settings. These checks do not set page geometry. Styles without
paper presets can retain fixed geometry in their PDF contract; they cannot
accept a paper override. Do not hide overflow by changing a requested page count.

## Page and text decisions

The “Typst default” column records the library's baseline, not our design.
The explicit values below are checked-in choices. See the official
[page reference](https://typst.app/docs/reference/layout/page/) and
[text reference](https://typst.app/docs/reference/text/text/).

| Setting | Typst 0.15.1 default | Shipped choice / location |
| --- | --- | --- |
| Paper | A4 | Named style presets, explicit locale defaults and optional document selection |
| Orientation / columns | Portrait / 1 | Harvard and Cluster portrait; 1 page column, renderer grids compose content |
| Margins | `auto`, scaled from the shorter edge (25 mm on A4) | Explicit 12 mm top/bottom, 15 mm left/right |
| Bleed / binding | 0 / auto from direction | 0 / left for these left-to-right documents |
| Page fill | `auto` | Harvard and Cluster deliberately retain auto |
| Page furniture | No numbering; automatic header/footer | Adapter explicitly clears numbering, header/footer and background/foreground |
| Header/footer positioning | 30% ascent/descent | Explicit 30%; inactive while those fields are empty |
| Font / size | Libertinus Serif / 11 pt | Harvard and Cluster use Archivo 10.5 pt as their base |
| Fallback fonts | Enabled | Enabled; PDF font contracts reject unexpected families |
| Weight / style / stretch | 400 / normal / 100% | Explicit same baseline; components opt into bold and different sizes |
| Text paint | Black fill, no stroke | Explicit black / none; components specify accent or white |
| Language / region / direction | English / none / auto | Leaf declares language, CH region, and `ltr` |
| Script | Auto from characters | Deliberate `auto`; language/direction are separately explicit |
| Tracking / word spacing / baseline | 0 / 100% / 0 | Explicit same baseline; labels deliberately add tracking |
| CJK–Latin spacing | Auto | Deliberate auto; not a claim of CJK support or suitable fonts |
| Top/bottom edges | Cap height / baseline | Explicit same values; heading and box geometry depends on them |
| Punctuation overhang | Enabled | Harvard and Cluster retain true |
| Hyphenation | Auto, follows justification | Auto with explicit local overrides |
| Kerning / ordinary ligatures | Enabled | Explicit true |
| Alternates / discretionary and historical ligatures | Disabled | Explicit false; no stylistic set |
| Numeral form / width | Auto from font | Deliberate auto, so each selected font's own numerals apply |
| Slashed zero / fractions | Disabled | Explicit false |
| OpenType features / variable axes | Empty overrides | Explicit empty dictionaries; font defaults remain intentional |
| Smart quotes | Enabled, locale-derived forms | Explicit enabled, alternative false, quote forms auto from the explicit locale |
| Line-breaking costs | 100% hyphenation, runt, widow, orphan | Explicit 100% each; these are penalties, not guarantees of widow/orphan elimination |

A defect found during the audit: Harvard's TOML declared paper and font, but
its renderer still used literal A4 and Archivo. Its renderer now consumes the
merged settings. That change preserved the 16 Harvard PDFs available at the time.
Typography defaults must reach the renderer; a declarative
file that nothing reads cannot configure a document.

## Paragraphs and components

Typst's baseline paragraph leading is 0.65 em, spacing 1.2 em, justification
false and line breaking auto. The latter chooses the optimized algorithm for
justified paragraphs. Harvard and Cluster declare base leading of 0.7 em,
zero paragraph spacing and
no baseline justification. Intentional Harvard regions still opt into their
established justification/spacing. All retain `linebreaks = "auto"` as an
explicit algorithm choice. [Paragraph reference](https://typst.app/docs/reference/model/par/).

`paragraph.leading_em` is the single authority for line spacing, including
Harvard; `text.leading_em` is invalid. Substyles may override the paragraph
value without a renderer silently replacing it.

First-line and hanging indents are explicitly zero; indent-all is false.
Justification limits explicitly retain word spacing between two-thirds and
1.5 times normal, with no additional tracking. Paragraph line numbering is
explicitly absent. Base block alignment is left, above/below spacing zero,
breakability true, and inset/outset/radius zero, with no paint/stroke/clip or
sticky behavior. Grids declare their columns, gutters and
alignment. These component dimensions belong to each renderer, not to the
engine's interface.

Do not generalize this text-document adapter into requirements for shapes,
images, tables, math, lists, footnotes or a new graphical design. A style using
those elements must choose the relevant settings and add suitable output
checks. For example, images need explicit sizing/fit and alternative text;
numbered lists need explicit marker and indentation decisions.

## Export decisions and limits

The ctypst PDF API accepts the document and an epoch. It supplies a UTC
export timestamp, then uses pinned `typst_pdf::PdfOptions` defaults. It does
not expose arbitrary PDF options. We record that boundary instead of
pretending a TOML field configures an unsupported export feature.

| Export setting | Current explicit policy |
| --- | --- |
| Timestamp | ccvl `SOURCE_DATE_EPOCH`, default **0**; reject negative/invalid values. Document date deliberately auto so the supplied epoch applies |
| Title / author | Each renderer sets them from its document and approved profile |
| Description / keywords | Adapter explicitly clears them; add only deliberately |
| PDF version | Pinned exporter produces **1.7**; shipped contracts assert actual version |
| Tagged structure | Pinned exporter enables tagging; shipped contracts assert a structure tree exists |
| Pages | All rendered pages; contract checks requested count |
| PDF/A or PDF/UA conformance | No additional standard requested; tagging alone does not certify accessibility |
| Document identifier | Exporter auto, derived from document metadata; retained deliberately |
| Creator | Exporter auto includes Typst version; retained deliberately |
| Pretty printing | Disabled by pinned exporter |
| Fonts / text | Check embedded fonts, Unicode maps, required identity fields, minimum usable text and each style's permitted families |

Implementation sources:
[Typst PDF options at v0.15.1](https://github.com/typst/typst/blob/v0.15.1/crates/typst-pdf/src/lib.rs),
[ctypst 0.3.2 PDF adapter](https://docs.rs/crate/ctypst/0.3.2/source/src/pdf.rs).
The native check validates PDF contracts, and Linux checks independently use
qpdf/Poppler plus rendering and repeat-build comparisons. These are useful
technical checks; visual review and accessible reading-order review still
matter for each new design.
