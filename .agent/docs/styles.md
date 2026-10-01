# Independent document styles

Each `cvl/cv/<style>/` and `cvl/cl/<style>/` directory declares its own
`style.toml`. Style names and substyle names are lowercase letters, digits,
hyphens or underscores. A substyle name is scoped to its parent style.

```toml
id = "example"
api = 1
documents = ["cv"]
supports_locales = ["en-us"]
pages = [1, 2]
default_pages = 1
substyles = ["standard"]
default_substyle = "standard"
# Optional paths, relative to this style directory and inside the workspace:
# defaults = "tokens.toml"
# fonts = ["assets/Example-Regular.ttf"]
# settings_adapter = "document-v1" # optional shared settings protocol

# Optional paper selection; omission keeps renderer-owned fixed geometry.
[paper.defaults]
en-us = "us-letter"

[paper.sizes.us-letter]
size_pt = [612, 792]
label = "US Letter"

[paper.sizes.us-letter.settings.page]
paper = "us-letter"
```

The workspace manifest names the document roots and their `default_style`.
The engine discovers every style directory under each root. For every listed
substyle and locale it expects:

```text
<substyle>/substyle.toml
<substyle>/<language>/<country>/content.toml
<substyle>/<language>/<country>/strings.toml
<substyle>/<language>/<country>/layout.toml   # optional engine input
<substyle>/<language>/<country>/typst/cv.typ   # cl.typ for letters
```

The contents of `substyle.toml` and `strings.toml` belong to the style.
A style importing `.agent/typst/document.typ` declares
`settings_adapter = "document-v1"`. This opts into the family → substyle →
locale → selected paper deep merge and shared validation. The adapter reserves only `page`,
`text`, `paragraph` and `block`; additional tables remain style-owned.
Omit this field for an independent renderer with its own settings schema.

`content.toml` keeps the application envelope (options and job metadata) and
the selected document's style-owned `[cv]` or `[cl]` fields. The record's
document selection must match its directory. A style may share code between
its substyles in any internal arrangement. Neither a shared renderer nor
`src/` is mandatory.

## Wording within a style

Harvard's `aligned` and `d-plus` substyles demonstrate optical title alignment.
`d-plus` additionally uses optional `[cv.spacing_by_page_pt."2"]`,
`[cv.entry_extra_by_page_mm]` and `[cv.bullet_extra_by_page_mm]` tables. The
renderer advances a logical content-page counter at explicit page breaks;
`cv-gap(name)` resolves these settings without depending on automatic physical
pagination. Substyles without overrides retain their original spacing path.
`cv.justify_bullets` is optional and defaults to false. The justified summary
has its own existing paragraph settings and is unaffected by that switch.
These are style-owned settings, not engine-wide requirements. See
[Harvard D+](../../cvl/cv/harvard/d-plus/README.md) and the
[unfinished cluster example](../../cvl/cv/cluster/README.md).

Substyles can share wording for the same document, style, language and country.
Each style owns its source, even when two styles happen to contain equal text.

```text
cvl/cv/harvard/
├── content/en/ch/wording.toml       # shared [cv] wording
├── standard/en/ch/content.toml     # reference and local record metadata
└── compact/en/ch/content.toml      # reference and explicit [cv] exceptions
```

The leaf declares the source visibly:

```toml
[wording]
source = "../../../content/en/ch/wording.toml"
```

The source contains only `[cv]` for a CV style or `[cl]` for a letter style.
It must be this style's `content/<language>/<country>/wording.toml`; references
to another style, document or locale fail. Sources cannot reference more sources.
Private opportunity records remain self-contained and cannot import showcase
wording.

| File | What to edit there |
| --- | --- |
| Style's `wording.toml` | Wording shared by its substyles |
| Leaf's `content.toml` | Record metadata and explicit wording exceptions |

An optional `[cv]` or `[cl]` table in the leaf overrides the shared fields.
Nested tables merge by field; a supplied scalar or array replaces the entire
value. Arrays are never appended or patched by position. No text is rewritten
or shortened automatically. A leaf can keep all its wording inline by omitting
`[wording]`.

Renderers using shared wording import `load-application` from
`.agent/typst/application.typ` and call it with the supplied application path.
This applies the same source/override rules when compiling a Typst entry point
directly. Reading the raw leaf with `toml(application-path)` would bypass its
shared wording. Independent renderers using inline content can keep their
own loader.

The CLI also canonicalizes source paths and rejects symlinks that cross the
owning style or locale. Typst's loader can check the declared path and owner,
but cannot resolve filesystem symlinks; use the CLI checks before publishing.

## Paper selection

The optional `paper` registry in `style.toml` lists the supported named presets
under `paper.sizes` and chooses one for every supported locale under
`paper.defaults`. Its IDs are style-owned; A4, US Letter and custom shapes can
all be declared. A style without this registry retains its fixed geometry and
rejects document paper selections.

Each preset owns its `settings`, display `label` and expected PDF `size_pt`.
Dimensions are the actual output width and height in points: a landscape style
declares `[792, 612]` for US Letter. Preset settings are the last adapter layer;
independent renderers can consume their own fields. Avoid duplicate paper
defaults in locale layouts. The PDF dimensions verify the renderer's output
rather than resizing it.

Shipped renderers import `resolve-paper` and `paper-settings` from
`.agent/typst/paper.typ`. Resolve the style's own metadata and locale with
`requested: paper-input` and the record's owning `cv_paper` or `cl_paper` as
`recorded`. The result exposes `id`, `label`, `size_pt` and `settings`.
`paper-settings((family, substyle, layout), preset)` validates and merges the
layers; render any paper label from the resolved preset. Each entry point
forwards `sys.inputs.at("paper", default: "")` to its renderer. Generated
opportunity copies retain the resolved selection in that default.

| Selection source | Precedence |
| --- | --- |
| `--paper <name>` on a document build or watcher | First |
| `options.cv_paper` / `options.cl_paper` in its record | Second |
| Selected style's `paper.defaults.<locale>` | Default |

Unsupported selections fail explicitly. ccvl never shrinks text or changes the
requested page count to fit another size. The style's locale-default paper
keeps existing output names; alternatives use a paper suffix such as
`cv-1-us-letter.pdf` or `cl-a4.pdf` in the same output folder. Document discovery
follows each leaf record's effective selection, preserving its output name
while the record retains its default paper. Full checks compare
those selected outputs with tracked PDFs and render the other declared papers
into temporary validation outputs.

```sh
bash ./ccvl build-cv en-ch 4 --style harvard --substyle d-plus --paper a4
bash ./ccvl build-cv en-ch 1 --style cluster --substyle d-plus --paper a4
bash ./ccvl explain-style cv en-ch --style cluster --paper a4
```

## Rendering interface

The Typst entry point receives these `sys.inputs`: `application`, `profile`,
`locale`, `pages`, `strings`, `substyle`, and the resolved `paper` ID when the
style declares paper presets; optionally `shared-defaults` and
`contract` when the style supplies them, plus `layout` when the leaf has a
`layout.toml`. File values are absolute workspace
paths. Use `sys.inputs.at("application", default: "/path/to/content.toml")`
(and equivalent literal defaults for other inputs) to support generated
opportunity copies that compile within the workspace without CLI inputs.

An optional `scaffold.toml` contains blank document fields for new opportunities,
without a `[cv]` or `[cl]` wrapper. `new-opportunity` combines the neutral job
scaffold with each configured style's content scaffold and page defaults.
Absent content scaffolds produce empty document tables. Never copy personal
showcase claims into a scaffold.

An optional `contract.toml` can declare:

- `content_fields`: an allowed list of document fields; omission permits arbitrary fields.
- `metric_rules`: tables with `kind`, `minimum` and `maximum` occurrence counts
  for emitted `ccvl-line` metrics. Metrics carry their own fill bounds.
- `shared_pages`: page numbers that must remain identical across page presets.
- `[pdf]`: optional `size_pt = [width, height]`, `font_pattern`,
  `minimum_text_chars`, `required_profile_fields`, `require_image`, `version`
  and `tagged`. Optional `[pdf.by_locale.<locale>]` overrides these per locale;
  if supplied, it must cover every supported locale.

Harvard additionally uses its summary and paragraph contracts and the
`layout_contract` / `source_files` four-page station-marker protocol. Those
checks apply only to a style opting into those contracts; they are not a
requirement for independent layouts. Styles without metrics need not emit them.
The engine always checks the requested page count, valid positive page geometry,
PDF integrity, embedded fonts with Unicode maps and a usable text layer.

All 16 bundled ctypst font faces are available. A style may declare additional
workspace font files; provide their redistribution licenses when publishing.
`bash ./ccvl list-documents` lists every discovered output as JSON. `check`,
`measure` and Linux independent PDF checks enumerate these same definitions.

```sh
bash ./ccvl build-cv en-ch 4 --style harvard --substyle compact
bash ./ccvl build-cl en-ch --style harvard --substyle frame
```

An individual build resolves its selected style and extra fonts. A broken
unrelated style does not prevent that build. `check`, `public-check` and full
document enumeration continue to inspect the complete workspace.

Inspect the merged adapter inputs without compiling or creating outputs:

```sh
bash ./ccvl explain-style cv en-ch --style cluster --substyle d-plus
bash ./ccvl explain-style cl en-ch --style harvard --substyle frame
```

The JSON lists sources in precedence order, merged `settings`, and `origins`
keyed by JSON Pointer (for example `/page/paper`). A value retains the last
source that actually supplies it, including equal-value overrides. It also
reports the selected `paper` and whether the CLI, record or locale default
selected it. Preset values still cite their style-definition source.
The explanation covers adapter inputs; component-specific renderer overrides such as header
font size are outside its scope. A renderer that has not opted in receives a
clear unsupported-adapter error instead of a guessed explanation.

Keep reusable style changes upstream in ccvl, then merge them into personal
applications repositories. Private content stays downstream.

## Shipped styles

Harvard supplies chronological CVs and cover letters. Cluster supplies a grouped
CV opening with the D+ comparison workflow. Modern is a scaffold with `standard`
and `timeline` substyles whose design is not yet defined. All support German and
English Swiss locales on portrait A4. See [the gallery](../../cvl/README.md) for PDFs,
[the defaults audit](typst-defaults.md) for settings and deliberate `auto`
choices, and [ccvl-style](../skills/ccvl-style/SKILL.md) for style creation.

## AIDA structure and shared conventions

Visual styles own content fields, paragraph/line geometry, fonts, page counts
and supported papers. In ccvl, AIDA means the complete six-paragraph,
26-body-line letter contract with five additional one-line highlights between
paragraphs 3 and 4. Harvard implements this structure and its AIDA argument
roles together; a three-paragraph structure is a different contract. The
`editorial` and paragraph `aida_stage` metadata declare the association;
mechanical checks enforce the measurable rules and independent review checks
the argument roles. [Editorial guidance](editorial.md) owns
universal evidence and writing decisions.

Author locale identifiers in lowercase. Shared spelling, greeting and closing
conventions belong upstream in cletter/family; consume them while retaining
explicit user/style overrides and preserving protected names, quotes and
original evidence. A locale does not automatically override the selected paper.
[Independent review](review.md) checks the actual selected contracts and pages,
including general documents without a vacancy.

## Portable consumer bundles

`ccvl export-style cv en-ch --style cluster --substyle d-plus --paper a4
--output style.json` exports an exact variant, including template sources,
contracts, fonts and license notices. A bundle contains no profile, application,
wording records or rendered showcase output. Styles with embedded candidate
`source_files` must move those facts into explicit inputs before export.

`ccvl render-style --bundle style.json --application application.toml --profile
profile.toml --output document.pdf` renders outside a CCVL filesystem workspace.
The application must contain resolved wording and select the same variant.

The `ccvl-core` crate lives in `.agent/core`. Its `render` feature provides the
same portable renderer to native consumers. The bundle owns field structure;
`StyleDocument::editable_fields`, `field_value` and `set_field` are shared by
manual editors and chat tools. Arrays and objects use complete JSON replacements.
`editing_context` excludes template sources and binary assets from chat context.
A content hash binds each document to an explicitly selected bundle version.

Storage and chat hosts remain consumer responsibilities. CCVL stays the rapid
style contribution upstream: edit a style here, verify its affected variants,
then publish a new bundle. Consumers do not maintain template copies. A style
bundle and a rendered PDF do not establish editorial readiness or authorize
signatures and application submission.
