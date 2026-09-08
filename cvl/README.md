# General CV and cover letter

This directory contains the approved, renderable general document. It is a
projection of verified user evidence, not the evidence store itself.

```text
cvl/
├── profile.toml
├── assets/
├── shared/{style.toml,defaults.toml}
├── cv/{standard,compact}/{de,en}/ch/{content.toml,strings.toml,typst/,pdf/}
└── cl/{left-rule,frame}/{de,en}/ch/{content.toml,strings.toml,typst/,pdf/}
```

`profile.toml` contains only approved fields used in the rendered header. Each
locale's `content.toml` provides the general Summary paragraph and cover
letter; `strings.toml` holds section/subject chrome only. The `typst/`
directories hold the visible CV/cover-letter logic; shared measurement
primitives and neutral scaffolds belong in `.agent/`; source
documents, the rich profile, journal, and station allocation belong in
`interview/`.

Each record selects one CV substyle (`standard`|`compact`) and one
cover-letter substyle (`left-rule`|`frame`) per document, defaulting to
`standard` / `left-rule` when the fields are absent. The record fields are `options.cv_substyle` and `options.cl_substyle`. All substyles satisfy
the same line and vertical-rhythm contracts, so switching substyles never
changes what the `measure`/`check` gates require of the content.

Family identity lives in `cv/style.toml` and `cl/style.toml` (`id =
"harvard"`); each substyle top holds a `substyle.toml` delta — `compact`
thins whitespace only and `frame` swaps the highlight panel to a full border,
both keeping Harvard's contracts. A second family would arrive as a new
top-level sibling, never nested inside Harvard.

A keyed opportunity supplies its own `application.toml` from
`../opportunities/<organisation>/<position>/` while reusing this general CV
body and render profile.

The Harvard CV has a fixed layout: 6–8 full experience entries on page 1, exactly 10
two-bullet supporting entries on page 2, exactly 10 two-bullet projects on page
3, and three groups of three three-line competency blocks on page 4. Run
`bash ./ccvl profile-status --verify-sources` before rendering, and verify the
requested locale, exact page counts, and a usable PDF text layer afterwards.
