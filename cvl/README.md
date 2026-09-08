# General CV and cover letter

`cvl/` contains approved document sources and their rendered outputs. Verified
user evidence lives in `interview/`; concrete job records live in `opportunities/`.

The hierarchy is **document → style → substyle → language → country**:

```text
cvl/
├── profile.toml
├── assets/
├── shared/harvard/              # sharing chosen by Harvard only
├── cv/
│   └── harvard/
│       ├── style.toml           # locales, substyles and page presets
│       ├── contract.toml        # Harvard's content/layout/PDF rules
│       ├── scaffold.toml        # blank Harvard CV fields
│       ├── src/                 # Harvard renderer and career entries
│       ├── standard/{de,en}/ch/{content.toml,strings.toml,typst/,pdf/}
│       └── compact/{de,en}/ch/{content.toml,strings.toml,typst/,pdf/}
└── cl/
    └── harvard/
        ├── style.toml
        ├── contract.toml
        ├── scaffold.toml
        ├── src/                 # Harvard letter renderer
        ├── left-rule/{de,en}/ch/{content.toml,strings.toml,typst/,pdf/}
        └── frame/{de,en}/ch/{content.toml,strings.toml,typst/,pdf/}
```

Another style is a sibling of `harvard` below its document. It owns its fonts,
geometry, layout code, content fields, page presets and optional contract.
It may use columns, landscape pages or a different letter structure. It need
not import Harvard, have a `src/` folder, or use `cvl/shared/harvard/`.

The shared protocol is metadata and inputs: discover the style, select a
substyle/locale/page preset, provide the record and profile, compile its Typst
entry point, and verify the output. `.agent/typst/` contains reusable neutral
measurement and profile helpers. See `.agent/docs/styles.md` for the interface.

Records select `options.cv_style` / `options.cl_style` and
`options.cv_substyle` / `options.cl_substyle`. Missing style fields use the
explicit defaults in `ccvl.json`; missing substyle fields use that style's
`default_substyle`. The shipped defaults are Harvard standard and Harvard
left-rule. `options.pages` selects CV pages; `options.cl_pages` optionally
selects letter pages. The style declares the supported counts.

Harvard currently shares one contract across each document's substyles:
`compact` adjusts CV spacing and `frame` changes the letter highlight border.
Its CV uses 2/3/4 pages, A4 and Archivo; its four-page station layout and its
five-line summary remain Harvard requirements. Use
`bash ./ccvl profile-status --verify-sources` for that station protocol.
