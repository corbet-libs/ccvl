# Reusable Typst helpers

This directory contains neutral profile, application, measurement and explicit-settings
adapters. Each style chooses which helpers to import. Document geometry and
presentation code live under `cvl/<document>/<style>/`.

`application.typ` resolves a leaf's explicit shared wording and local overrides.
It makes no assumptions about a style's content fields. Its `load-application`
function also accepts a self-contained opportunity record.

`paper.typ` resolves a style-declared paper preset from a requested ID, the
record's document selection or the locale default. Its `paper-settings` helper
validates and merges adapter layers with the selected preset. Renderers use
the resolved label when printing a paper name.

The shipped Harvard styles share helpers in `cvl/shared/harvard/` and keep their
CV and letter renderers in their own `src/` folders. Harvard's application
adapter lives in `cvl/shared/harvard/application.typ`, because its summary and
paragraph rules are specific to that style.

The engine supplies the selected record, profile, locale, page count, strings and
substyle configuration as inputs. A style may also declare defaults and extra
fonts. See [../docs/styles.md](../docs/styles.md) for the complete interface and
[../../cvl/README.md](../../cvl/README.md) for the shipped folder structure.

Harvard's `standard`/`compact` CVs share a contract; `compact` changes vertical
spacing. Its `left-rule`/`frame` letters share a contract; `frame` changes the
highlight border. Other styles own different layouts and contracts.

`document.typ` optionally applies style-owned page, text and paragraph values.
It chooses no paper, font or locale. Shipped styles merge family defaults,
substyle overrides and leaf `layout.toml`; independent styles can use a wholly
different setup. See [the defaults audit](../docs/typst-defaults.md) for the
pinned Typst baseline, deliberate automatic behavior and export limitations.
