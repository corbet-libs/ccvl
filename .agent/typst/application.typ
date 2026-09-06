// Shape validation for TOML application records shared by the CV and
// cover-letter renderers. Deep bounds live in Rust and in the measured
// render; here only what the renderers index is asserted, so a hand-edited
// record fails with a location instead of a cryptic field access.
#let workspace = json("/ccvl.json")
#let cover-letter-contract = workspace.documents.cover_letter
#let cv-contract = workspace.documents.cv
#let known-styles = workspace.styles.available
#let default-style-name = workspace.styles.default
// Uniform closing-line grace for every measured paragraph (summary and
// cover letter alike): a closing line may spill invisibly past the block.
#let last-line-maximum = workspace.at("last_line_maximum", default: 102)

// Last whitespace-separated token of a recipient name for the salutation.
// "Dr. Jane Doe" -> "Doe"; single-token and hyphenated names survive;
// empty/whitespace yields "" so callers fall back to the generic greeting.
#let salutation-last-name(name) = {
  let trimmed = name.trim()
  if trimmed == "" {
    ""
  } else {
    let tokens = trimmed.split(regex("\\s+")).filter(token => token != "")
    tokens.last()
  }
}

// Locale-correct German salutations (SN 010130 for ch/li, DIN 5008 for
// de/at; at follows DIN since ÖNORM A 1080 was withdrawn in 2018).
//
// The recipient `name` field holds the full address form, e.g.
// "Frau Dr. Müller" or "Herr Müller". Only the honorific, academic titles,
// and surname render; first names never appear in a formal salutation.
//
// - Honorific: "Frau" -> Frau, "Herr"/"Herrn" -> Herr. Abbreviations such as
//   "Hr."/"Fr." are rejected (unhöflich); the Anrede always uses "Herr",
//   never the accusative "Herrn" (which belongs only in the postal address)
//   and never abbreviates "Frau".
// - Titles: "Dr." stays abbreviated, "Prof." normalises to the spelled-out
//   "Professor"; "Dipl.-Ing." and "Mag." survive. Protocol keeps only the
//   highest title, so Professor suppresses Dr.
// - Punctuation: ch and li use no comma (the next sentence starts uppercase);
//   de and at use a comma (the sentence continues lowercase).
// - No parsable honorific or surname falls back to the generic
//   "Sehr geehrte Damen und Herren" so the letter stays formally safe;
//   the Rust gate warns so a human supplies Herr/Frau.
#let salutation-honorific(name) = {
  let tokens = name.trim().split(regex("\\s+")).filter(token => token != "")
  if tokens.len() == 0 { "" } else {
    let first = tokens.first()
    if first.match(regex("^(?i)frau\\.?$")) != none { "frau" } else if first.match(regex("^(?i)herrn?\\.?$")) != none {
      "herr"
    } else { "" }
  }
}

#let salutation-title-kind(token) = {
  if token.match(regex("^(?i)dr\\.?$")) != none { "Dr." } else if token.match(regex("^(?i)prof(\\.|essor)?$")) != none {
    "Professor"
  } else if token.match(regex("^(?i)dipl\\.?-?ing\\.?$")) != none { "Dipl.-Ing." } else if (
    token.match(regex("^(?i)dipling$")) != none
  ) { "Dipl.-Ing." } else if token.match(regex("^(?i)mag(\\.|ister)?$")) != none { "Mag." } else { "" }
}

#let salutation-titles(name) = {
  let tokens = name.trim().split(regex("\\s+")).filter(token => token != "")
  let kept = ()
  for token in tokens {
    let kind = salutation-title-kind(token)
    if kind != "" and kind not in kept { kept.push(kind) }
  }
  // Protocol: Professor outranks Dr.; never stack both.
  if "Professor" in kept { ("Professor",) } else { kept }
}

#let salutation-surname(name) = {
  let tokens = name.trim().split(regex("\\s+")).filter(token => token != "")
  let significant = tokens.filter(token => (
    salutation-title-kind(token) == ""
      and token.match(regex("^(?i)(herrn?|frau)\\.?$")) == none
      and token.match(regex("^(?i)(mr|mrs|ms|miss|phd|ma|ba|bsc|msc)\\.?$")) == none
  ))
  if significant.len() == 0 { "" } else { significant.last() }
}

#let de-salutation(name, region: "ch") = {
  assert(
    region in ("ch", "li", "de", "at"),
    message: "de-salutation region must be ch, li, de, or at",
  )
  let comma = if region == "ch" or region == "li" { "" } else { "," }
  let honorific = salutation-honorific(name)
  let surname = salutation-surname(name)
  if honorific == "" or surname == "" {
    "Sehr geehrte Damen und Herren" + comma
  } else {
    let titles = salutation-titles(name)
    let title-part = if titles.len() == 0 { "" } else { " " + titles.join(" ") }
    if honorific == "frau" {
      "Sehr geehrte Frau" + title-part + " " + surname + comma
    } else {
      "Sehr geehrter Herr" + title-part + " " + surname + comma
    }
  }
}

#let require-fields(value, fields, scope) = {
  for field in fields {
    assert(field in value, message: scope + "." + field + " is required")
  }
}

#let validate-application(
  application,
  expected-language: none,
  require-cv: false,
  require-cl: false,
) = {
  require-fields(
    application,
    ("schema_version", "revision", "options", "job", "cv"),
    "application",
  )
  assert(
    application.schema_version == 4,
    message: "unsupported application schema version",
  )

  let options = application.options
  require-fields(
    options,
    ("language", "pages", "generate_cl", "application_date"),
    "application.options",
  )
  assert(
    options.pages in (2, 3, 4),
    message: "application.options.pages must be 2, 3, or 4",
  )
  // Style axis: options.style is optional and defaults to the manifest
  // default ("harvard") for records written before styles existed. An
  // unknown name fails here with the available list.
  let style-name = options.at("style", default: default-style-name)
  assert(
    type(style-name) == str,
    message: "application.options.style must be a style name",
  )
  assert(
    style-name == "" or style-name in known-styles,
    message: "unknown style " + repr(style-name) + ". Available styles: " + known-styles.join(", ") + ".",
  )
  if expected-language != none {
    assert(
      options.language == expected-language,
      message: "application language must be " + expected-language,
    )
  }

  let job = application.job
  require-fields(
    job,
    (
      "id",
      "title",
      "organization",
      "location",
      "source",
      "url",
      "description",
      "connections",
      "company_context",
      "notes",
      "cl_recipient",
    ),
    "application.job",
  )
  assert(
    job.id.trim() != "",
    message: "application.job.id is required",
  )
  require-fields(
    job.cl_recipient,
    ("name", "title", "company", "address_line_1", "address_line_2"),
    "application.job.cl_recipient",
  )

  require-fields(application.cv, ("summary",), "application.cv")
  assert(
    type(application.cv.summary) == str,
    message: "application.cv.summary must be one flowing paragraph",
  )
  if require-cv {
    assert(
      application.cv.summary.trim() != "",
      message: "application.cv.summary is required",
    )
  }

  assert(
    type(options.generate_cl) == bool,
    message: "application.options.generate_cl must be a boolean",
  )
  if options.generate_cl {
    require-fields(application, ("cl",), "application")
    let letter = application.cl
    require-fields(letter, ("paragraphs", "highlights"), "application.cl")
    let paragraph-contracts = cover-letter-contract.paragraphs
    assert(
      letter.paragraphs.len() == paragraph-contracts.len(),
      message: "application.cl.paragraphs must contain exactly " + str(paragraph-contracts.len()) + " items",
    )
    for (paragraph-index, paragraph) in letter.paragraphs.enumerate() {
      let scope = "application.cl.paragraphs." + str(paragraph-index + 1)
      let bounds = paragraph-contracts.at(paragraph-index).lines
      assert(
        paragraph.len() >= bounds.minimum and paragraph.len() <= bounds.maximum,
        message: scope + " must contain " + str(bounds.minimum) + "–" + str(bounds.maximum) + " lines",
      )
      if require-cl {
        for (line-index, line) in paragraph.enumerate() {
          assert(
            line.trim() != "",
            message: scope + ".lines." + str(line-index + 1) + " is required",
          )
        }
      }
    }
    let body-lines = letter.paragraphs.fold(0, (total, paragraph) => total + paragraph.len())
    let body-contract = cover-letter-contract.body_lines
    assert(
      body-lines >= body-contract.minimum and body-lines <= body-contract.maximum,
      message: "cover-letter body must contain "
        + str(body-contract.minimum)
        + "–"
        + str(body-contract.maximum)
        + " lines",
    )
    for region in cover-letter-contract.paragraph_regions {
      let start = region.paragraphs.first() - 1
      let end = region.paragraphs.last()
      let region-lines = letter.paragraphs.slice(start, end).fold(0, (total, paragraph) => total + paragraph.len())
      assert(
        region-lines >= region.minimum and region-lines <= region.maximum,
        message: "cover-letter paragraphs "
          + str(region.paragraphs.first())
          + "–"
          + str(region.paragraphs.last())
          + " must contain "
          + str(region.minimum)
          + "–"
          + str(region.maximum)
          + " lines",
      )
    }
    assert(
      letter.highlights.len() == cover-letter-contract.highlights.count,
      message: "application.cl.highlights must contain exactly "
        + str(cover-letter-contract.highlights.count)
        + " one-line items",
    )
    if require-cl {
      for (index, highlight) in letter.highlights.enumerate() {
        assert(
          highlight.trim() != "",
          message: "application.cl.highlights." + str(index + 1) + " is required",
        )
      }
    }
  } else {
    assert(
      "cl" not in application,
      message: "a disabled cover letter may not retain hidden content",
    )
    assert(
      not require-cl,
      message: "the cover-letter renderer requires options.generate_cl to be true",
    )
  }
}
