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
locale deep merge and shared validation. The adapter reserves only `page`,
`text`, `paragraph` and `block`; additional tables remain style-owned.
Omit this field for an independent renderer with its own settings schema.

`content.toml` uses the common application envelope (options and job metadata)
with style-owned `[cv]` and `[cl]` tables. The record's document selection must
match its directory. A style may share code between its substyles in any
internal arrangement. Neither a shared renderer nor `src/` is mandatory.

The Typst entry point receives these `sys.inputs`: `application`, `profile`,
`locale`, `pages`, `strings`, `substyle`; optionally `shared-defaults` and
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

Inspect the merged adapter inputs without compiling or creating outputs:

```sh
bash ./ccvl explain-style cv en-us --style test-style-1 --substyle sidebar
bash ./ccvl explain-style cl en-ch --style harvard --substyle frame
```

The JSON lists sources in precedence order, merged `settings`, and `origins`
keyed by JSON Pointer (for example `/page/paper`). A value retains the last
source that actually supplies it, including equal-value overrides. The result
explains adapter inputs; component-specific renderer overrides such as header
font size are outside its scope. A renderer that has not opted in receives a
clear unsupported-adapter error instead of a guessed explanation.

Keep reusable style changes upstream in ccvl, then merge them into personal
applications repositories. Private content stays downstream.

## Shipped demonstrations

`test-style-1` uses a root `layout.typ`, with `sidebar` and `topbar` portrait
compositions. `test-style-2` uses `parts/composition.typ`, with `cards` and
`timeline` landscape compositions. Both ship CV and CL entry points for
`en-ch` A4 and `en-us` US Letter. Neither imports Harvard. They share only the
optional neutral settings adapter, the profile, and the engine interface.

See [the gallery](../../cvl/README.md) for PDFs and visible comparisons,
[the defaults audit](typst-defaults.md) for settings and deliberate `auto`
choices, and [ccvl-style](../skills/ccvl-style/SKILL.md) for style creation.

```sh
bash ./ccvl build-cv en-us --style test-style-1 --substyle sidebar
bash ./ccvl build-cv en-us 2 --style test-style-2 --substyle cards
bash ./ccvl build-cl en-ch --style test-style-2 --substyle timeline
```
