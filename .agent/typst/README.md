# Reusable Typst helpers

This directory contains neutral profile and measurement adapters and the bundled
font set. Each style chooses which helpers to import. Document geometry and
presentation code live under `cvl/<document>/<style>/`.

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
