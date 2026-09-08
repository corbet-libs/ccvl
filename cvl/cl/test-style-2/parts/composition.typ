#import "/.agent/typst/document.typ": apply-document-settings, merge-settings
#let render(
  application-path: none,
  profile-path: none,
  strings-path: none,
  substyle-path: none,
  defaults-path: none,
  layout-path: none,
  pages: 1,
) = {
  let record = toml(application-path)
  let profile = toml(profile-path)
  let chrome = toml(strings-path)
  let config = merge-settings(merge-settings(toml(defaults-path), toml(substyle-path)), toml(layout-path))
  let accent = rgb(config.demo.accent)
  let soft = rgb(config.demo.soft)
  let variant = config.demo.variant
  assert(record.options.language == chrome.locale, message: "record and locale differ")
  assert(config.text.lang + "-" + lower(config.text.region) == chrome.locale, message: "layout locale differs")
  set document(title: chrome.label + " | " + chrome.cl_label, author: (profile.name,))
  show: apply-document-settings.with(config)
  let label(body) = text(size: 9pt, weight: "bold", tracking: 1pt, body)
  let title(body) = text(size: 25pt, weight: "bold", body)
  let contact() = [#text(size: 10pt)[#profile.email #h(10pt) #profile.location]]
  let footer() = [#v(12pt)#line(length: 100%, stroke: 0.6pt + accent)#v(6pt)#text(
      size: 9pt,
      fill: accent,
    )[#chrome.notice #h(1fr) #chrome.paper_label]]
  let section(heading, body, boxed: false) = block(
    width: 100%,
    breakable: false,
    inset: if boxed { 14pt } else { 0pt },
    fill: if boxed { soft } else { none },
    radius: if boxed { 8pt } else { 0pt },
  )[
    #text(size: 15pt, weight: "bold", fill: accent, heading)
    #v(8pt)
    #body
  ]
  for index in range(pages) {
    if index > 0 { pagebreak(weak: false) }
    grid(
      columns: (2fr, 1fr),
      gutter: 18pt,
      align: bottom,
      [#label(chrome.label)#v(9pt)#title(profile.name)],
      align(right)[#label(chrome.paper_label)#v(9pt)#text(size: 10pt, profile.email)],
    )
    v(16pt)
    line(length: 100%, stroke: 1.2pt + accent)
    v(18pt)
    text(size: 24pt, weight: "bold", record.cl.subject)
    v(18pt)
    if index == 0 {
      grid(
        columns: if variant == "cards" { (1fr, 1fr) } else { (1fr, 2fr) }, gutter: 24pt,
        section("Opening / 01", record.cl.opening, boxed: variant == "cards"),
        section("The composition", record.cl.body.first(), boxed: variant == "cards"),
      )
      v(24pt)
      text(size: 18pt, fill: accent, "Continued on the second page declared by this letter style.")
    } else {
      grid(
        columns: (1fr, 1fr),
        gutter: 24pt,
        section("Explicit choices / 02", record.cl.body.at(1), boxed: variant == "cards"),
        section("What changes / 03", record.cl.body.at(2), boxed: variant == "cards"),
      )
      v(28pt)
      text(size: 18pt, weight: "bold", record.cl.closing)
    }
    footer()
  }
}
