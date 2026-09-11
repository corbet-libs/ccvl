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
unresolved conflicts, preferences, and the station allocation plan. The shipped Harvard CV
layout gate requires 6–8 experience entries on page 1, exactly
10 two-bullet supporting entries on page 2, exactly 10 two-bullet projects on
page 3, and three groups of three three-line competency blocks on page 4.

This domain may contain unverified or conflicted material. Only verified,
uniquely assigned facts may cross into `cvl/` or an opportunity.

## `cvl/`: the general document

`cvl/` contains approved sources and outputs. `cvl/profile.toml` supplies
approved profile fields. The hierarchy is document → style → substyle →
language → country:

```text
cvl/cv/harvard/{standard,compact}/{de,en}/ch/{content.toml,strings.toml,typst/,pdf/}
cvl/cl/harvard/{left-rule,frame}/{de,en}/ch/{content.toml,strings.toml,typst/,pdf/}
cvl/cv/harvard/content/{de,en}/ch/wording.toml
cvl/cl/harvard/content/{de,en}/ch/wording.toml
cvl/shared/harvard/{style.typ,defaults.toml,application.typ}
```

Each style owns its `style.toml`, optional `contract.toml` and content scaffold,
and all of its layout code. Harvard uses a `src/` folder within each document
style. Other styles are siblings of Harvard and may organise their internals
differently. Shared Harvard tokens and helpers are explicitly Harvard-owned.
Wording sources belong to one document/style/locale and may be referenced only
by its substyles. Leaf records keep visible references, metadata and explicit
exceptions; private opportunity records remain self-contained.

The engine discovers style metadata, resolves record/profile inputs, compiles
the selected entry point, and verifies its declared page and measurement rules.
Fonts, shapes, page geometry and content structure belong to the style.
Individual builds validate the selected style and its inputs. Full workspace
checks still validate every registered style, so isolated builds do not hide
broken experiments from release validation.
Records select `cv_style`/`cl_style` and `cv_substyle`/`cl_substyle` under
`options`; workspace and style metadata declare the defaults. See
[styles.md](styles.md) for the interface and [../../cvl/README.md](../../cvl/README.md)
for the shipped tree.

## `opportunities/`: keyed job packages

One concrete role owns one directory and one canonical tailored record:

```text
opportunities/<organisation-key>/<position-key>/application.toml
```

The job directory owns the posting, attributable organisation and role
research, fit analysis, tailored Summary, optional cover letter, interview
preparation, submission record, and outcome. The application record selects
its locale, page presets and optional document papers from the selected styles.
Rendered `<Name>_<Org>_<Pos>_CV.pdf` and optional
`<Name>_<Org>_<Pos>_CL.pdf` go in its local `pdfs/` directory; resolved standalone
Typst copies use the same file stems in its local `typst/` directory.
The [opportunity filename convention](applications.md) uses the final display-name
token and normalized opportunity keys, without changing the source name.

There is no standalone market map. General preferences or durable facts
learned about the user belong in `interview/`; research about a company or
role belongs inside its concrete opportunity. There are also no top-level
`applications/`, `submissions/`, `outcomes/`, or `out/` layers.

## Product mechanism

`.agent/` owns the canonical skills, neutral scaffolds, Rust source,
tests, bootstrap scripts, internal documentation, Typst engine, and
workspace manifest validation. Each style owns its document contracts. Skills own editorial judgment;
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
