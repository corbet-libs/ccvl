# Opportunities

Every concrete job lives directly under a stable two-part key:

```text
opportunities/<organisation-key>/<position-key>/
├── application.toml
├── posting.md                 optional archived source
├── research.md                optional attributable company and role research
├── interview-<stage>.md       optional preparation
├── submission.md              optional observed submission
├── outcome.md                 optional observed outcome
├── typst/
│   ├── cv.typ                 resolved standalone copy (generated, do not edit)
│   └── cl.typ                 only when the cover letter is enabled (generated)
└── pdfs/
    ├── cv.pdf
    └── cl.pdf                 only when the cover letter is enabled
```

The `typst/` copies are the locale templates with their `sys.inputs`
defaults resolved for the record (application and profile paths, page
count, style), so each compiles standalone and reproduces its neighbour
PDF. They are build artifacts: editing `application.toml` regenerates
both `typst/` and `pdfs/`; editing a locale template regenerates both as
well. Do not hand-edit `typst/` — re-run `build-opportunity`. `pdfs/` and
`typst/` are tracked in git (PDFs via LFS); commit whenever you feel like it.

Both keys use lowercase ASCII letters, numbers, hyphens, or underscores. The
single `application.toml` owns the options (language, pages, cover-letter
switch), the flowing Summary paragraph, optional six-paragraph cover letter,
and five highlights. Its
directory is the key; do not add generic `companies/`, `positions/`,
`applications/`, or market-map layers.

Company, role, and fit research belongs here with its sources. General facts
or preferences learned about the user belong in `interview/`.

Create the record deterministically, then ask the `ccvl-apply` skill to
research, draft, review, measure, and render it:

```sh
bash ./ccvl new-opportunity <organisation-key> <position-key>
bash ./ccvl build-opportunity <organisation-key> <position-key>
```

The public ccvl repository intentionally contains only this scaffold. Real
postings, research, recipient details, tailored documents, and outcomes belong
in a private downstream.
