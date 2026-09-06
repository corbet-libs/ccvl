# General CV and cover letter

This directory contains the approved, renderable general document. It is a
projection of verified user evidence, not the evidence store itself.

```text
cvl/
├── profile.toml
├── assets/
├── de-ch/
│   ├── application.toml
│   ├── cv.typ
│   ├── cl.typ
│   └── output/{cv-2.pdf,cv-3.pdf,cv-4.pdf,cl.pdf}
└── en-ch/
    └── ...
```

`profile.toml` contains only approved fields used in the rendered header. Each
locale's `application.toml` provides the general Summary paragraph and cover
letter; both `cv.typ` files hold the complete visible CV logic and both
`cl.typ` files the complete cover-letter logic. Shared measurement
primitives, styles, and neutral scaffolds belong in `.agent/`; source
documents, the rich profile, journal, and station allocation belong in
`interview/`.

Each `application.toml` selects a render style through `options.style`,
defaulting to `harvard` when the field is absent. The available styles are
listed in the `styles` section of `ccvl.json`, with whitespace and accent
knobs below `.agent/typst/styles/`. All styles satisfy the same line and
vertical-rhythm contracts, so switching styles never changes what the
`measure`/`check` gates require of the content.

A keyed opportunity supplies its own `application.toml` from
`../opportunities/<organisation>/<position>/` while reusing this general CV
body and render profile.

The CV has a fixed layout: 6–8 full experience entries on page 1, exactly 10
two-bullet supporting entries on page 2, exactly 10 two-bullet projects on page
3, and three groups of three three-line competency blocks on page 4. Run
`bash ./ccvl profile-status --verify-sources` before rendering.
