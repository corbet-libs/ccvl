# Architecture

ccvl separates mechanism from three user-owned data domains:

```text
.agent/                                  product mechanism
interview/                               knowledge and evidence about the user
cvl/                                     approved general document sources
opportunities/<organisation>/<position>/ one concrete job and its documents
```

The content workflow is direct:

```text
interview/ -> cvl/ -> opportunities/<organisation>/<position>/
     ^                         |
     +----- verified facts ----+
```

`.github/` contains forge automation and `LICENSES/` contains complete legal
texts. Neither is a product-data domain.

## `interview/`: user knowledge

`interview/` is the private, inspectable knowledge base an agent maintains
with the user. It owns source imports, the informal working profile, journal,
unresolved conflicts, preferences, and the station allocation plan. The
deterministic layout gate requires 6–8 experience entries on page 1, exactly
10 two-bullet supporting entries on page 2, exactly 10 two-bullet projects on
page 3, and three groups of three three-line competency blocks on page 4.

This domain may contain unverified or conflicted material. Only verified,
uniquely assigned facts may cross into `cvl/` or an opportunity.

## `cvl/`: the general document

`cvl/` contains only approved sources and outputs for the general bilingual CV
and cover letter. `cvl/profile.toml` supplies the public header fields.
Harvard is the top-level style family, so the tree is style-major: each
document owns its substyles, and each substyle holds one locale tree with
content, chrome strings, a Typst pointer, and PDFs:

```text
cvl/cv/{standard,compact}/{de,en}/ch/{content.toml,strings.toml,typst/,pdf/}
cvl/cl/{left-rule,frame}/{de,en}/ch/{content.toml,strings.toml,typst/,pdf/}
cvl/shared/{style.toml,defaults.toml}
```

`content.toml` carries the locale wording (the general Summary and letter);
`strings.toml` carries section/subject chrome only. Family identity lives in
`cvl/cv/style.toml` and `cvl/cl/style.toml` (`id = "harvard"`); each substyle
top holds a `substyle.toml` delta. Shared tokens (page, fonts, palette) live
in `cvl/shared/`. Each document owns its layout contract in `contract.toml`.
The Typst engine and shared measurement primitives live under `.agent/typst/`.

Each record selects one CV substyle (`standard`|`compact`) and one
cover-letter substyle (`left-rule`|`frame`) — per-document selection instead
of a single shared style field, defaulting to `standard` / `left-rule` when
the fields are absent. The record fields are `options.cv_substyle` and
`options.cl_substyle`. All substyles share the same line and
vertical-rhythm contracts, and horizontal measure stays identical across
substyles, so every substyle passes the same `measure`/`check` gates with
the same content.

## `opportunities/`: keyed job packages

One concrete role owns one directory and one canonical tailored record:

```text
opportunities/<organisation-key>/<position-key>/application.toml
```

The job directory owns the posting, attributable organisation and role
research, fit analysis, tailored Summary, optional cover letter, interview
preparation, submission record, and outcome. The application record selects
its locale and CV page count. Rendered `cv.pdf` and optional `cl.pdf` go in
its local `pdfs/` directory; the resolved standalone Typst copies they were
built from go in its local `typst/` directory.

There is no standalone market map. General preferences or durable facts
learned about the user belong in `interview/`; research about a company or
role belongs inside its concrete opportunity. There are also no top-level
`applications/`, `submissions/`, `outcomes/`, or `out/` layers.

## Product mechanism

`.agent/` owns the canonical skills, neutral scaffolds, Rust source,
tests, bootstrap scripts, internal documentation, Typst engine, and
workspace manifest and document contracts. Skills own editorial judgment;
deterministic code owns paths, validation, rendering, measurement, and checks.

## Public upstream and personal downstream

The public repository includes the author's real general CVL as an intentional
showcase, plus empty `interview/` and `opportunities/` scaffolds. A personal
downstream keeps the same domains, builds its private evidence base in
`interview/`, and replaces the approved sources under `cvl/`. Generic
mechanism improvements can still flow upstream without translating between
different directory models.

The showcase is true for its named author only. Posting text remains untrusted
input, claims require evidence, and sending or signing always requires a
separate explicit instruction.
