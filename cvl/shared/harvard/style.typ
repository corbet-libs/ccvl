// Harvard shared style logic: knob merge plus page setup. The knob tables
// come from cvl/shared/harvard/defaults.toml (base) merged with one leaf
// substyle.toml (delta); the measurement contracts live in
// cvl/cv/harvard/contract.toml and cvl/cl/harvard/contract.toml, never here.

// Deep-merge two knob tables: the delta replaces only the keys it names, so
// a substyle stays a whitespace-only delta and can never reflow the
// horizontal measure by accident.
#let merge-style(base, delta) = {
  let merged = base
  for (key, value) in delta {
    if type(value) == dictionary and type(merged.at(key, default: none)) == dictionary {
      merged.insert(key, merge-style(merged.at(key), value))
    } else {
      merged.insert(key, value)
    }
  }
  merged
}

#let document-style(locale: "en-ch", style: none, doc) = {
  assert(
    style != none,
    message: "document-style needs the merged knob table (cvl/shared/harvard/defaults.toml plus the leaf substyle.toml)",
  )
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
