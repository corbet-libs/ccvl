#import "/.agent/typst/paper.typ": paper-settings, resolve-paper
#import "/.agent/typst/application.typ": load-application
#import "/.agent/typst/application-date.typ": format-application-date
// Shared Harvard cover letter; inputs belong to the calling leaf.
#import "/cvl/shared/harvard/style.typ": document-style
#import "/cvl/shared/harvard/application.typ": load-cover-letter-contract, validate-application
#import "/.agent/typst/line-contract.typ": line-contract-mode, measured-content-line, measured-paragraph
#import "/.agent/typst/profile.typ": load-profile
#import "/.agent/typst/letter/letter.typ": (
  closing as farewell-closing, salutation, signature-image,
)

#let cover-letter-contract = load-cover-letter-contract()


#let highlight-text(value) = {
  let parts = value.split(" | ")
  if parts.len() > 1 {
    text(weight: "bold", parts.first()) + text(" | " + parts.slice(1).join(" | "))
  } else {
    text(value)
  }
}
#let body-fill = cover-letter-contract.line_fill.body
#let highlight-fill = cover-letter-contract.line_fill.highlight
#let with-body-fill(lines) = range(lines.len()).map(index => (
  text: lines.at(index),
  // A justified non-final line is stretched to the full measure, so a thin
  // one buys ugly word spacing rather than safety; only the ragged closing
  // line may legitimately fall back to the lower floor.
  min_fill: if index + 1 == lines.len() { body-fill.minimum } else { body-fill.non_final_minimum },
  target_fill: body-fill.target,
  // Unlike the CV Summary, no cover-letter line may spill past the measure:
  // the closing line shares the same 100% ceiling as every other line.
  max_fill: body-fill.maximum,
))
#let with-highlight-fill(value) = (
  text: value,
  min_fill: highlight-fill.minimum,
  target_fill: highlight-fill.target,
  max_fill: highlight-fill.maximum,
)


#let render-cl(
  application-path: none,
  profile-path: none,
  strings-path: none,
  substyle-path: none,
  shared-defaults-path: none,
  layout-path: none,
  paper-input: "",
) = {
  let (profile, localized-profile) = load-profile(profile-path)
  // Merge the shared base knobs with this leaf's substyle delta (panel
  // geometry only). Whitespace and accents below come from cl-style, never
  // from forked literals.
  let cl-strings = toml(strings-path)
  // Locale comes from the leaf's strings file and is cross-checked against
  // the record below: a de-ch record through en-ch strings fails here.
  let doc-locale = cl-strings.locale
  let application = load-application(application-path)
  let paper = resolve-paper(toml("../style.toml"), doc-locale, requested: paper-input, recorded: application.options.at(
    "cl_paper",
    default: none,
  ))
  let cl-style = paper-settings((toml(shared-defaults-path), toml(substyle-path), toml(layout-path)), paper)
  validate-application(application, expected-language: doc-locale, require-cl: true)
  let cover = cl-style.cover
  let header-after = cl-style.header.after_pt * 1pt
  let subject-after = cover.subject_after_pt * 1pt
  let highlight-gap = cover.highlight_gap_pt * 1pt
  let closing-before = cover.closing_before_pt * 1pt
  let closing-after = cover.closing_after_pt * 1pt

  set document(
    title: (if doc-locale == "de-ch" { "Anschreiben | " } else { "Cover Letter | " }) + profile.name,
    author: (profile.name,),
  )

  show: document-style.with(locale: doc-locale, style: cl-style)

  let job = application.job
  let letter = application.cl
  let recipient = job.cl_recipient
  // Recipient address is recorded in application.toml (job.cl_recipient) for
  // provenance but intentionally not printed on the cover letter.

  let subject = if job.organization.trim() == "" {
    [#cl-strings.subject_open | #job.title]
  } else {
    [#cl-strings.subject_application_for #job.title]
  }
  let salutation = [#salutation(doc-locale, recipient.name)]
  let closing = [#farewell-closing(doc-locale)]

  let paragraph(index) = block(width: 100%, breakable: false)[
    #measured-paragraph(
      "cl.paragraph." + str(index + 1),
      "cl-body",
      with-body-fill(letter.paragraphs.at(index)),
      justify: cover-letter-contract.justify_body,
    )
  ]
  // Keep the CV triangle and gutter inside a panel aligned with the body column.
  let highlight-indent = cl-style.cv.bullet_indent_pt * 1pt
  let highlight-border = cover.highlight_border_left_pt * 1pt
  let highlight-bullet() = box(width: 10.5pt, height: 7.35pt, align(horizon, align(center, polygon(
    fill: rgb(cl-style.accents.link),
    (0pt, 0pt),
    (4.41pt, 2.75625pt),
    (0pt, 5.5125pt),
  ))))
  let highlights = block(
    fill: rgb(cl-style.accents.highlight_background),
    stroke: (
      top: cover.highlight_border_top_pt * 1pt + rgb(cl-style.accents.link),
      right: cover.highlight_border_right_pt * 1pt + rgb(cl-style.accents.link),
      bottom: cover.highlight_border_bottom_pt * 1pt + rgb(cl-style.accents.link),
      left: highlight-border + rgb(cl-style.accents.link),
    ),
    inset: cover.highlight_inset_pt * 1pt,
    // Strokes are centred on their path. Inset the left path by half its width
    // so the outer blue edge starts exactly at the paragraph boundary.
    outset: (left: -highlight-border / 2),
    radius: 2pt,
    width: 100%,
  )[
    #for index in range(cover-letter-contract.highlights.count) {
      let value = letter.highlights.at(index)
      let filled = with-highlight-fill(value)
      grid(
        columns: (highlight-indent, 1fr),
        align: horizon,
        highlight-bullet(),
        [#measured-content-line(
          "cl.highlight." + str(index + 1),
          "cl-highlight",
          highlight-text(filled.text),
          filled.min_fill,
          filled.target_fill,
          filled.max_fill,
          source-text: value,
        )],
      )
      if index < cover-letter-contract.highlights.count - 1 { v(highlight-gap) }
    }
  ]

  let header-content = {
    let localized = localized-profile.at(doc-locale)
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
  let subject-content = [
    #grid(
      columns: (1fr, auto),
      align: (left, right),
      text(size: 12pt, weight: "bold", subject), text(size: 10.5pt, format-application-date(doc-locale, application.options.application_date)),
    )
    #v(subject-after)
    #line(length: 100%, stroke: 0.5pt + black)
  ]
  let salutation-content = [#salutation]
  let closing-content = block(breakable: false)[
    #closing
    #v(closing-before)
    #signature-image("/cvl/assets/signature.png", 31.5)
    #v(closing-after)
    #profile.name
  ]

  // Full-page vertical rhythm, sequenced by the leaf adapter after importing
  // this module. Kept in a function (not top-level content) so test fixtures
  // can import this module's fill helpers without emitting output.
  layout(size => {
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
    let weights = cover.gap_weights
    assert(weights.len() == gap-count and weights.all(weight => weight > 0))
    let remaining-height = size.height - fixed-height
    let total-weight = weights.fold(0, (total, weight) => total + weight)
    let gaps = weights.map(weight => remaining-height * weight / total-weight)
    let smallest-gap-pt = calc.round(10 * calc.min(..gaps) / 1pt) / 10
    let largest-gap-pt = calc.round(10 * calc.max(..gaps) / 1pt) / 10
    let highlight-top = (
      heights.slice(0, 6).fold(0pt, (total, height) => total + height)
        + gaps.slice(0, 6).fold(0pt, (total, gap) => total + gap)
    )
    let highlight-center = (
      100 * (highlight-top + heights.at(6) / 2) / size.height
    )
    let rhythm = cover-letter-contract.vertical_rhythm
    let metrics = (
      (
        id: "cl.vertical-gap",
        kind: "cl-vertical-gap",
        text: "role-weighted gaps: " + str(smallest-gap-pt) + "–" + str(largest-gap-pt) + " pt",
        // One range metric proves every gap is in bounds: report the smallest
        // when it breaches the floor, otherwise the largest checks the ceiling.
        actual_fill: if smallest-gap-pt < rhythm.gap_pt.minimum { smallest-gap-pt } else { largest-gap-pt },
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
        rows: gaps.fold((), (rows, gap) => rows + (auto, gap)) + (auto,),
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
}
