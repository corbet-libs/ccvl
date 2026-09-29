#import "/.agent/typst/paper.typ": paper-settings, resolve-paper
#import "/.agent/typst/application.typ": load-application
#import "/.agent/typst/document.typ": apply-document-settings
#let render(
  application-path: none,
  profile-path: none,
  strings-path: none,
  substyle-path: none,
  defaults-path: none,
  layout-path: none,
  paper-input: "",
  pages: 1,
) = {
  let record = load-application(application-path)
  let profile = toml(profile-path)
  let chrome = toml(strings-path)
  let paper = resolve-paper(toml("../style.toml"), chrome.locale, requested: paper-input, recorded: record.options.at(
    "cv_paper",
    default: none,
  ))
  let config = paper-settings((toml(defaults-path), toml(substyle-path), toml(layout-path)), paper)
  let accent = rgb(config.demo.accent)
  let soft = rgb(config.demo.soft)
  let variant = config.demo.variant
  assert(record.options.language == chrome.locale, message: "record and locale differ")
  assert(config.text.lang + "-" + lower(config.text.region) == chrome.locale, message: "layout locale differs")
  set document(title: chrome.label + " | " + chrome.cv_label, author: (profile.name,))
  show: apply-document-settings.with(config)
  let label(body) = text(size: 9pt, weight: "bold", tracking: 1pt, body)
  let title(body) = text(size: 25pt, weight: "bold", body)
  let contact() = [#text(size: 10pt)[#profile.email #h(10pt) #profile.location]]
  let footer() = [#v(12pt)#line(length: 100%, stroke: 0.6pt + accent)#v(6pt)#text(
      size: 9pt,
      fill: accent,
    )[#chrome.notice #h(1fr) #paper.label]]
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
      align(right)[#label(paper.label)#v(9pt)#text(size: 10pt, profile.email)],
    )
    v(16pt)
    line(length: 100%, stroke: 1.2pt + accent)
    v(18pt)
    if index == 0 {
      text(size: 21pt, weight: "bold", record.cv.title)
      v(10pt)
      record.cv.intro
      v(18pt)
      if variant == "cards" {
        grid(
          columns: (1fr, 1fr), gutter: 14pt,
          ..record.cv.sections.map(entry => section(entry.title, entry.body, boxed: true))
        )
      } else {
        for (number, entry) in record.cv.sections.enumerate() {
          grid(
            columns: (30pt, 1fr, 2fr),
            gutter: 14pt,
            align: top,
            circle(radius: 12pt, fill: accent, stroke: none, text(fill: white, str(number + 1))),
            text(size: 15pt, weight: "bold", fill: accent, entry.title),
            entry.body,
          )
          v(14pt)
        }
      }
    } else {
      text(size: 25pt, weight: "bold", "A second page is this style’s decision.")
      v(18pt)
      grid(
        columns: (1fr, 1fr, 1fr),
        gutter: 18pt,
        section(
          "01 / Select",
          "The record names a CV style and substyle. Language-country selects the locale directory. Page count selects one of the style’s declared presets.",
          boxed: variant == "cards",
        ),
        section(
          "02 / Compose",
          "Style defaults, the substyle delta and locale layout settings form the effective presentation. This renderer owns the visual composition.",
          boxed: variant == "cards",
        ),
        section(
          "03 / Verify",
          "The same record produces each substyle. The checker compares the declared dimensions, selected fonts and page count with the actual PDF.",
          boxed: variant == "cards",
        ),
      )
      v(28pt)
      text(
        size: 16pt,
        fill: accent,
        "The one-page preset omits this page; the content is not squeezed into a Harvard contract.",
      )
    }
    footer()
  }
}
