// Harvard style: the default ccvl page setup, moved from document.typ.
// Element styles and the letterhead live visibly in each cvl/<locale>
// document; measurement primitives live in line-contract.typ.
//
// This module also owns the shared style registry. A style is one TOML knob
// file plus one Typst renderer below `.agent/typst/styles/`, selected per
// record through `options.style`. The single source of truth for the default
// and the available names is the `styles` section in `ccvl.json`, so adding
// a style means adding `<name>.typ` plus `<name>.toml` and listing `<name>`
// there; no template fork is needed.

#let style-manifest = json("/ccvl.json").styles
#let default-style = style-manifest.default
#let known-styles = style-manifest.available

// Resolve a raw style name to its knob table. Empty means the record predates
// styles and renders with the default. Unknown names fail here with the
// available list instead of a missing-file error deeper in the render.
#let load-style(raw-name) = {
  let name = if raw-name == "" { default-style } else { raw-name }
  assert(
    name in known-styles,
    message: "unknown style "
      + repr(name)
      + ". Available styles: "
      + known-styles.join(", ")
      + ". Set options.style in application.toml.",
  )
  toml("/.agent/typst/styles/" + name + ".toml")
}

#let document-style(locale: "en-ch", style: load-style(default-style), doc) = {
  let locale-parts = locale.split("-")
  assert(locale-parts.len() == 2, message: "locale must contain language and region subtags")
  set page(
    paper: "a4",
    margin: (
      top: style.page.margin_top_mm * 1mm,
      bottom: style.page.margin_bottom_mm * 1mm,
      left: style.page.margin_left_mm * 1mm,
      right: style.page.margin_right_mm * 1mm,
    ),
  )
  set text(
    font: "Archivo",
    size: style.text.size_pt * 1pt,
    fill: black,
    lang: locale-parts.first(),
    region: locale-parts.last(),
    top-edge: "cap-height",
    bottom-edge: "baseline",
  )
  set par(leading: style.text.leading_em * 1em, justify: false, spacing: 0pt)
  set block(above: 0pt, below: 0pt)
  show link: it => text(fill: rgb(style.accents.link), it)
  doc
}
