// Synthetic ledger renderer for engine tests. It applies the document-v1
// settings and paper preset, loads resolved wording and emits fixed metrics
// that satisfy the family contracts without measuring text.
#import "/.agent/typst/application.typ": load-application
#import "/.agent/typst/document.typ": apply-document-settings
#import "/.agent/typst/paper.typ": paper-settings, resolve-paper
#import "/.agent/typst/profile.typ": load-profile

#let metric(kind, id, actual, bounds, layout: false, unit: none) = {
  let value = (
    kind: kind,
    id: id,
    text: id,
    actual_fill: actual,
    min_fill: bounds.minimum,
    target_fill: bounds.target,
    max_fill: bounds.maximum,
  )
  if unit != none { value.insert("unit", unit) }
  if layout { [#metadata(value) <ccvl-layout>] } else { [#metadata(value) <ccvl-line>] }
}

// Assign the letter contract's fill bounds to measured lines. Only the
// paragraph's closing line may use the lower body minimum; importing these
// helpers reads no record and emits no content.
#let letter-contract() = toml("/cvl/cl/ledger/contract.toml")
#let with-body-fill(lines) = {
  let body = letter-contract().line_fill.body
  range(lines.len()).map(index => (
    text: lines.at(index),
    min_fill: if index + 1 == lines.len() { body.minimum } else { body.non_final_minimum },
    target_fill: body.target,
    max_fill: body.maximum,
  ))
}
#let with-highlight-fill(value) = {
  let fill = letter-contract().line_fill.highlight
  (text: value, min_fill: fill.minimum, target_fill: fill.target, max_fill: fill.maximum)
}
#let bounds(line) = (minimum: line.min_fill, target: line.target_fill, maximum: line.max_fill)

#let settings(document, locale, record, defaults-path, substyle-path, layout-path, paper-input) = {
  let preset = resolve-paper(
    toml("/cvl/" + document + "/ledger/style.toml"),
    locale,
    requested: paper-input,
    recorded: record.options.at(document + "_paper", default: none),
  )
  paper-settings((toml(defaults-path), toml(substyle-path), toml(layout-path)), preset)
}

#let contact(profile) = [#profile.name #linebreak() #profile.email | #profile.phone-label]

#let render-cv(
  application-path: none,
  profile-path: none,
  strings-path: none,
  substyle-path: none,
  shared-defaults-path: none,
  layout-path: none,
  paper-input: "",
  pages: 4,
) = {
  let record = load-application(application-path)
  let strings = toml(strings-path)
  let contract = toml("/cvl/cv/ledger/contract.toml")
  assert(pages in contract.presets, message: "unsupported ledger CV page count")
  assert(record.options.language == strings.locale, message: "ledger locale mismatch")
  let (profile, localized-profile) = load-profile(profile-path)
  show: apply-document-settings.with(settings(
    "cv",
    strings.locale,
    record,
    shared-defaults-path,
    substyle-path,
    layout-path,
    paper-input,
  ))
  contact(profile)
  parbreak()
  [*#strings.summary*]
  parbreak()
  record.cv.summary
  let fill = contract.summary_fill
  for line in range(contract.summary_lines) {
    metric("cv-summary", "cv.summary." + str(line + 1), fill.target, fill)
  }
  for page in range(1, pages) {
    pagebreak()
    [*#strings.experience* #linebreak() Synthetic ledger page #(page + 1).]
  }
}

#let render-cl(
  application-path: none,
  profile-path: none,
  strings-path: none,
  substyle-path: none,
  shared-defaults-path: none,
  layout-path: none,
  paper-input: "",
) = {
  let record = load-application(application-path)
  let strings = toml(strings-path)
  let contract = letter-contract()
  assert(record.options.language == strings.locale, message: "ledger locale mismatch")
  let letter = record.cl
  assert(letter.paragraphs.len() == contract.paragraphs.len(), message: "ledger paragraph count")
  assert(letter.highlights.len() == contract.highlights.count, message: "ledger highlight count")
  let (profile, localized-profile) = load-profile(profile-path)
  show: apply-document-settings.with(settings(
    "cl",
    strings.locale,
    record,
    shared-defaults-path,
    substyle-path,
    layout-path,
    paper-input,
  ))
  contact(profile)
  parbreak()
  [*#strings.subject*]
  for (paragraph, lines) in letter.paragraphs.enumerate() {
    parbreak()
    for (index, line) in with-body-fill(lines).enumerate() {
      let id = "cl.paragraph." + str(paragraph + 1) + "." + str(index + 1)
      metric("cl-body", id, line.target_fill, bounds(line))
      line.text
      if index + 1 < lines.len() { linebreak() }
    }
    if paragraph == 2 {
      for (index, highlight) in letter.highlights.enumerate() {
        parbreak()
        let line = with-highlight-fill(highlight)
        metric("cl-highlight", "cl.highlight." + str(index + 1), line.target_fill, bounds(line))
        line.text
      }
    }
  }
  let rhythm = contract.vertical_rhythm
  metric("cl-vertical-gap", "cl.vertical-gap", rhythm.gap_pt.target, rhythm.gap_pt, layout: true, unit: "pt")
  metric(
    "cl-highlight-center",
    "cl.highlight-center",
    rhythm.highlight_center_percent.target,
    rhythm.highlight_center_percent,
    layout: true,
  )
}
