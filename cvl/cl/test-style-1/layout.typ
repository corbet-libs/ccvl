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
  let contact() = [#text(size: 10pt)[#profile.email#if variant == "sidebar" { linebreak() } else {
      h(10pt)
    }#profile.location]]
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
  let identity() = [
    #label(chrome.label)
    #v(18pt)
    #title(profile.name)
    #v(10pt)
    #contact()
  ]
  let content() = [
    #text(size: 21pt, weight: "bold", record.cl.subject)
    #v(20pt)
    #record.cl.opening
    #v(16pt)
    #for paragraph in record.cl.body {
      paragraph
      v(16pt)
    }
    #text(weight: "bold", record.cl.closing)
  ]
  if variant == "sidebar" {
    grid(
      columns: (1.1fr, 2fr),
      gutter: 22pt,
      align: top,
      block(width: 100%, inset: 18pt, fill: accent, radius: 4pt)[
        #set text(fill: white)
        #identity()
        #v(26pt)
        #label(chrome.paper_label)
        #v(10pt)
        #text(size: 10pt)[Two documents, one style family. This panel is a choice of this substyle.]
      ],
      content(),
    )
  } else {
    block(width: 100%, inset: 20pt, fill: accent, radius: 4pt)[
      #set text(fill: white)
      #identity()
    ]
    v(22pt)
    content()
  }
  footer()
}
