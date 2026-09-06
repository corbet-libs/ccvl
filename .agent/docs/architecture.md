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
and cover letter. `cvl/profile.toml` supplies the public header fields. Each
locale has one `cv.typ`, one `cl.typ`, one general `application.toml`, and an
output directory:

```text
cvl/<locale>/cv.typ
cvl/<locale>/cl.typ
cvl/<locale>/application.toml
cvl/<locale>/output/cv-{2,3,4}.pdf
cvl/<locale>/output/cl.pdf
```

Both `cv.typ` files hold the complete visible CV logic and both `cl.typ`
files the complete cover-letter logic; they are the editing surface. The
Typst engine, shared measurement primitives, styles, fonts, and layout
contracts live under `.agent/typst/`; they are mechanism, not a second CV
data model.

Each record selects a render style through `options.style`, defaulting to
`harvard` when the field is absent. A style is one Typst renderer plus one
TOML knob file below `.agent/typst/styles/` (see `.agent/typst/README.md`);
the available names live in the `styles` section of `ccvl.json`. All styles
share the same line and vertical-rhythm contracts, and horizontal measure
stays identical across styles, so every style passes the same
`measure`/`check` gates with the same content.

## `opportunities/`: keyed job packages

One concrete role owns one directory and one canonical tailored record:

```text
opportunities/<organisation-key>/<position-key>/application.toml
```

The job directory owns the posting, attributable organisation and role
research, fit analysis, tailored Summary, optional cover letter, interview
preparation, submission record, and outcome. The application record selects
its locale and CV page count. Generated `cv.pdf` and optional `cl.pdf` go in
its local `output/` directory.

There is no standalone market map. General preferences or durable facts
learned about the user belong in `interview/`; research about a company or
role belongs inside its concrete opportunity. There are also no top-level
`applications/`, `submissions/`, `outcomes/`, or `out/` layers.

## Product mechanism

`.agent/` owns the canonical skills, neutral scaffolds, Rust source,
tests, bootstrap scripts, internal documentation, Typst engine, and
machine-readable `ccvl.json` contract. Skills own editorial judgment;
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
