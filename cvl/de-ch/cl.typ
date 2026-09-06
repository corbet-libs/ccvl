// The real German cover letter: every block is visible and editable here.
// Data comes from the TOML record; shared machinery (measurement, styles,
// header chrome) stays below .agent/typst and carries no content.
#import "/.agent/typst/styles/harvard.typ": document-style, load-style
#import "/.agent/typst/application.typ": cover-letter-contract, de-salutation, last-line-maximum, validate-application
#import "/.agent/typst/line-contract.typ": line-contract-mode, measured-line, measured-paragraph
#import "/.agent/typst/profile.typ": localized-profile, profile

#let application-path = sys.inputs.at("application", default: "/cvl/de-ch/application.toml")
#let application = toml(application-path)
#validate-application(application, expected-language: "de-CH", require-cl: true)

// Style axis: the explicit `style` input injected by render.rs (resolved from
// options.style) wins; a manual render without it falls back to the record,
// then to the harvard default. Whitespace and accents below come from that
// style's TOML knobs, never from forked literals.
#let style-input = sys.inputs.at("style", default: "")
#let style-name = if style-input != "" { style-input } else { application.options.at("style", default: "harvard") }
#let style = load-style(style-name)
#let cover = style.cover
#let header-after = style.header.after_pt * 1pt
#let subject-after = cover.subject_after_pt * 1pt
#let highlight-gap = cover.highlight_gap_pt * 1pt
#let closing-before = cover.closing_before_pt * 1pt
#let closing-after = cover.closing_after_pt * 1pt
#show: document-style.with(locale: "de-ch", style: style)

#set document(title: "Anschreiben | " + profile.name, author: (profile.name,))

#let job = application.job
#let letter = application.cl
#let recipient = job.cl_recipient
// Recipient address is recorded in application.toml (job.cl_recipient) for
// provenance but intentionally not printed on the cover letter.

#let body-fill = cover-letter-contract.line_fill.body
#let highlight-fill = cover-letter-contract.line_fill.highlight
#let with-body-fill(lines) = range(lines.len()).map(index => (
  text: lines.at(index),
  min_fill: body-fill.minimum,
  target_fill: body-fill.target,
  max_fill: if index + 1 == lines.len() { last-line-maximum } else { body-fill.maximum },
))
#let with-highlight-fill(text) = (
  text: text,
  min_fill: highlight-fill.minimum,
  target_fill: highlight-fill.target,
  max_fill: highlight-fill.maximum,
)

#let subject = if job.organization.trim() == "" {
  [Offene Bewerbung | #job.title]
} else {
  [Bewerbung als #job.title]
}
#let salutation = [#de-salutation(recipient.name, region: "ch")]
#let closing = [Freundliche Grüsse]

#let paragraph(index) = block(breakable: false)[
  #measured-paragraph(
    "cl.paragraph." + str(index + 1),
    "cl-body",
    with-body-fill(letter.paragraphs.at(index)),
    justify: cover-letter-contract.justify_body,
  )
]
#let highlights = block(
  fill: rgb(style.accents.highlight_background),
  stroke: (left: 2.5pt + rgb(style.accents.link)),
  inset: cover.highlight_inset_pt * 1pt,
  radius: 2pt,
  width: 100%,
)[
  #for index in range(cover-letter-contract.highlights.count) {
    grid(
      columns: (cover.highlight_number_width_mm * 1mm, 1fr),
      [#text(weight: "bold", fill: rgb(style.accents.link))[#(index + 1)]],
      [#measured-line(
        "cl.highlight." + str(index + 1),
        "cl-highlight",
        with-highlight-fill(letter.highlights.at(index)),
      )],
    )
    if index < cover-letter-contract.highlights.count - 1 { v(highlight-gap) }
  }
]

#let header-content = {
  let localized = localized-profile.at("de-ch")
  let contacts = (
    link("mailto:" + profile.email)[#profile.email],
    if profile.phone-label != none and profile.phone-href != none {
      link(profile.phone-href)[#profile.phone-label]
    },
    profile.location,
    profile.languages,
    localized.nationality-and-permit,
    link(profile.linkedin)[LinkedIn],
    link(profile.website)[Web],
    localized.availability,
  ).filter(item => item != none)

  align(center)[#text(size: 15.75pt, weight: "bold")[#profile.name]]
  v(header-after)
  align(center)[
    #text(size: 9.03pt)[
      #contacts.join([ | ])
    ]
  ]
}
#let subject-content = [
  #grid(
    columns: (1fr, auto),
    align: (left, right),
    text(size: 12pt, weight: "bold", subject), text(size: 10.5pt, application.options.application_date),
  )
  #v(subject-after)
  #line(length: 100%, stroke: 0.5pt + black)
]
#let salutation-content = [#salutation]
#let closing-content = block(breakable: false)[
  #closing
  #v(closing-before)
  #image("/cvl/assets/signature.png", height: 31.5pt)
  #v(closing-after)
  #profile.name
]

#layout(size => {
  let content-blocks = (
    header-content,
    subject-content,
    salutation-content,
    paragraph(0),
    paragraph(1),
    paragraph(2),
    highlights,
    paragraph(3),
    paragraph(4),
    paragraph(5),
    closing-content,
  )
  let heights = content-blocks.map(item => {
    measure(item, width: size.width).height
  })
  let fixed-height = heights.fold(0pt, (total, height) => total + height)
  let gap-count = content-blocks.len() - 1
  let gap-height = (size.height - fixed-height) / gap-count
  let highlight-top = (
    heights.slice(0, 6).fold(0pt, (total, height) => total + height) + 6 * gap-height
  )
  let highlight-center = (
    100 * (highlight-top + heights.at(6) / 2) / size.height
  )
  let rhythm = cover-letter-contract.vertical_rhythm
  let metrics = (
    (
      id: "cl.vertical-gap",
      kind: "cl-vertical-gap",
      text: "equal distributed gap between cover-letter content blocks",
      actual_fill: calc.round(10 * gap-height / 1pt) / 10,
      min_fill: rhythm.gap_pt.minimum,
      target_fill: rhythm.gap_pt.target,
      max_fill: rhythm.gap_pt.maximum,
      unit: "pt",
    ),
    (
      id: "cl.highlight-center",
      kind: "cl-highlight-center",
      text: "vertical centre of the highlight block",
      actual_fill: calc.round(10 * highlight-center) / 10,
      min_fill: rhythm.highlight_center_percent.minimum,
      target_fill: rhythm.highlight_center_percent.target,
      max_fill: rhythm.highlight_center_percent.maximum,
      unit: "%",
    ),
  )
  if line-contract-mode == "enforce" {
    for metric in metrics {
      assert(
        metric.actual_fill >= metric.min_fill and metric.actual_fill <= metric.max_fill,
        message: metric.id
          + " measured "
          + str(metric.actual_fill)
          + metric.unit
          + ", target "
          + str(metric.target_fill)
          + metric.unit
          + ", allowed "
          + str(metric.min_fill)
          + "–"
          + str(metric.max_fill)
          + metric.unit
          + ". Adjust evidenced content or line allocation, then measure again.",
      )
    }
  }
  [
    #for metric in metrics [
      #metadata(metric) <ccvl-layout>
    ]
  ]
  block(width: 100%, height: size.height)[
    #grid(
      columns: (1fr,),
      rows: (
        auto,
        1fr,
        auto,
        1fr,
        auto,
        1fr,
        auto,
        1fr,
        auto,
        1fr,
        auto,
        1fr,
        auto,
        1fr,
        auto,
        1fr,
        auto,
        1fr,
        auto,
        1fr,
        auto,
      ),
      align: (left, top),
      header-content,
      [],
      subject-content,
      [],
      salutation-content,
      [],
      paragraph(0),
      [],
      paragraph(1),
      [],
      paragraph(2),
      [],
      highlights,
      [],
      paragraph(3),
      [],
      paragraph(4),
      [],
      paragraph(5),
      [],
      closing-content,
    )
  ]
})
