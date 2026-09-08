// Explicit Typst 0.15.1 settings. No paper, font or locale is chosen here.
#import "document-settings.typ": validate-settings
#let merge-settings(base, override) = {
  let result = base
  for (key, value) in override {
    result.insert(key, if type(value) == dictionary and type(result.at(key, default: none)) == dictionary {
      merge-settings(result.at(key), value)
    } else { value })
  }
  result
}
#let automatic(value) = if value == "auto" { auto } else { value }
#let direction(value) = if value == "ltr" { ltr } else if value == "rtl" { rtl } else {
  panic("direction must be ltr or rtl")
}
#let apply-document-settings(settings, body) = {
  validate-settings(settings)
  let p = settings.page
  let t = settings.text
  let q = settings.paragraph
  let geometry = if p.paper == "custom" { (width: p.width_mm * 1mm, height: p.height_mm * 1mm) } else {
    (paper: p.paper)
  }
  set page(
    ..geometry,
    flipped: p.flipped,
    columns: p.columns,
    margin: (
      top: p.margin_top_mm * 1mm,
      bottom: p.margin_bottom_mm * 1mm,
      left: p.margin_left_mm * 1mm,
      right: p.margin_right_mm * 1mm,
    ),
    bleed: p.bleed_mm * 1mm,
    binding: (left: left, right: right).at(p.binding),
    fill: if p.fill == "auto" { auto } else { rgb(p.fill) },
    numbering: none,
    supplement: none,
    number-align: center + bottom,
    header: none,
    footer: none,
    background: none,
    foreground: none,
    header-ascent: p.header_ascent_percent * 1%,
    footer-descent: p.footer_descent_percent * 1%,
  )
  set text(
    font: t.font,
    fallback: t.fallback,
    style: t.style,
    weight: t.weight,
    stretch: t.stretch_percent * 1%,
    size: t.size_pt * 1pt,
    fill: if t.fill == "#000000" { black } else { rgb(t.fill) },
    stroke: none,
    lang: t.lang,
    region: t.region,
    dir: direction(t.direction),
    script: automatic(t.script),
    tracking: t.tracking_pt * 1pt,
    spacing: t.spacing_percent * 1%,
    baseline: t.baseline_pt * 1pt,
    cjk-latin-spacing: ("auto": auto, "none": none).at(t.cjk_latin_spacing),
    top-edge: t.top_edge,
    bottom-edge: t.bottom_edge,
    overhang: t.overhang,
    hyphenate: automatic(t.hyphenate),
    kerning: t.kerning,
    alternates: t.alternates,
    stylistic-set: none,
    ligatures: t.ligatures,
    discretionary-ligatures: t.discretionary_ligatures,
    historical-ligatures: t.historical_ligatures,
    number-type: automatic(t.number_type),
    number-width: automatic(t.number_width),
    slashed-zero: t.slashed_zero,
    fractions: t.fractions,
    features: t.features,
    variations: t.variations,
    costs: (
      hyphenation: t.cost_hyphenation_percent * 1%,
      runt: t.cost_runt_percent * 1%,
      widow: t.cost_widow_percent * 1%,
      orphan: t.cost_orphan_percent * 1%,
    ),
  )
  set par(
    leading: q.leading_em * 1em,
    spacing: q.spacing_pt * 1pt,
    justify: q.justify,
    linebreaks: automatic(q.linebreaks),
    first-line-indent: (amount: q.first_line_indent_pt * 1pt, all: q.indent_all),
    hanging-indent: q.hanging_indent_pt * 1pt,
    justification-limits: (
      spacing: (min: q.word_spacing_min_percent * 1%, max: q.word_spacing_max_percent * 1%),
      tracking: (min: q.tracking_min_pt * 1pt, max: q.tracking_max_pt * 1pt),
    ),
  )
  set par.line(numbering: none)
  set align((left: left, center: center, right: right, start: start, end: end).at(settings.block.align))
  set block(
    above: settings.block.above_pt * 1pt,
    below: settings.block.below_pt * 1pt,
    breakable: settings.block.breakable,
    inset: 0pt,
    outset: 0pt,
    radius: 0pt,
    fill: none,
    stroke: none,
    clip: false,
    sticky: false,
  )
  set smartquote(enabled: settings.text.smart_quotes, alternative: false, quotes: auto)
  // The exporter supplies SOURCE_DATE_EPOCH; this is an explicit, deterministic choice.
  set document(date: auto, description: none, keywords: ())
  body
}
