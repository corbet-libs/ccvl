# One keyed package per opportunity

Every concrete opportunity has one canonical tailored-data file:

```text
opportunities/<organisation-key>/<position-key>/application.toml
```

The general CV foundation must first pass `ccvl profile-status
--verify-sources`. Tailoring cannot repair a core page that lacks enough
verified stations.

Create it without choosing or copying paths manually:

```sh
bash ./ccvl new-opportunity <organisation-key> <position-key>
```

The command validates both keys, copies
`.agent/scaffolds/opportunity/application.toml`, assigns a stable ID, and
refuses to overwrite an existing record. When no cover letter is needed,
pass `--no-cover-letter`: the record is written with `generate_cl = false`
and no `[cl]` table, so there is nothing to delete afterwards (validation
rejects a disabled letter that retains hidden content). Schema version 4
contains:

- `options`: language, CV page count, cover-letter switch, application date,
  and render style (`style`, defaulting to `harvard`; see the `styles`
  section in `ccvl.json` and `.agent/typst/README.md`);
- `job`: vacancy, organisation, source, description, context, notes, and
  recipient (`job.cl_recipient.name` holds the full address form such as
  `"Frau Dr. Müller"` for the locale-correct salutation; empty falls back
  to the generic greeting with a warning, see
  `.agent/docs/cover-letter.md`);
- `cv.summary`: one flowing paragraph that must typeset to exactly five
  lines;
- when the cover letter is enabled, exactly six paragraphs following
  `.agent/docs/cover-letter.md` and exactly five one-line highlights.

Line lengths are authored as plain text; fill defaults come from `ccvl.json`.
Typst measures actual glyph width with the bundled font. The Summary must
render to exactly five lines; thin Summary lines fail unless explicitly
allowed, and a closing line may spill invisibly up to the uniform
closing-line maximum (cover-letter paragraphs share the rule). Explicit
cover-letter lines fail outside their bounds. Underfill and overflow past
the maximum fail and prompt another evidence-backed rewrite.

The opportunity directory is the lifecycle unit. An archived posting,
research, working rules, interview preparation, or outcome may sit beside the
TOML file. Summary and cover-letter fields remain solely in `application.toml`.

Measure and render one opportunity from the repository root:

```sh
bash ./ccvl measure-opportunity <organisation-key> <position-key>
bash ./ccvl build-opportunity <organisation-key> <position-key>
```

On Windows:

```powershell
.\ccvl.cmd measure-opportunity <organisation-key> <position-key>
.\ccvl.cmd build-opportunity <organisation-key> <position-key>
```

No locale or page argument is needed: the record owns both. The build writes
`output/cv.pdf` and, when enabled, `output/cl.pdf` directly below the keyed
opportunity. Alongside each PDF it emits a resolved customization copy of
the template it rendered: `output/cv.typ` and, when enabled,
`output/cl.typ`. Each copy is the locale template with its `sys.inputs`
defaults resolved for the opportunity (application and profile paths, the
record's page count for the CV, and the resolved style), so it compiles standalone and reproduces
the neighbouring PDF. The copies are build artifacts: do not edit them by
hand; re-run `build-opportunity` to refresh. Disabling the letter yields a
CV-only package and removes any stale generated `cl.pdf` and `cl.typ` from
an earlier build.

Keep a live package fresh while tailoring:

```sh
bash ./ccvl watch-opportunity <organisation-key> <position-key>
# or:
just watch <organisation-key> <position-key>
```

The watcher hashes the record's locale templates (`cvl/<locale>/*.typ`),
the shared Typst machinery (`.agent/typst/**/*.typ`), `cvl/profile.toml`,
`ccvl.json`, the opportunity record, and the generated `output/*.typ`
copies, then rebuilds the PDFs plus the resolved copies on change.
`watch-cv` and `watch-cl` provide the same loop for one general locale
document.
