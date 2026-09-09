// Shape validation for TOML application records shared by the CV and
// cover-letter renderers. Deep bounds live in Rust and in the measured
// render; here only what the renderers index is asserted, so a hand-edited
// record fails with a location instead of a cryptic field access.
//
// Contracts live with the style, not the manifest: cvl/cv/harvard/contract.toml and
// cvl/cl/harvard/contract.toml. Substyle membership (which cv/cl look a record
// selects) is enforced in Rust before the compile; here the selection keys
// only need to be strings when present.
// Loading a contract is explicit and local to the requested document.
#let load-cover-letter-contract() = toml("/cvl/cl/harvard/contract.toml")
#let load-cv-contract() = toml("/cvl/cv/harvard/contract.toml")

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
    ("schema_version", "revision", "options", "job"),
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
  if require-cv {
    assert(
      options.pages in (2, 3, 4),
      message: "application.options.pages must be 2, 3, or 4",
    )
  }
  // Rust validates per-document substyle membership before compilation.
  let selection-keys = (
    (if require-cv { ("cv_substyle",) } else { () }) + (if require-cl { ("cl_substyle",) } else { () })
  )
  for key in selection-keys {
    let value = options.at(key, default: "")
    assert(
      type(value) == str,
      message: "application.options." + key + " must be a substyle name",
    )
  }
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

  if require-cv {
    require-fields(application, ("cv",), "application")
    require-fields(application.cv, ("summary",), "application.cv")
    assert(
      type(application.cv.summary) == str,
      message: "application.cv.summary must be one flowing paragraph",
    )
    assert(
      application.cv.summary.trim() != "",
      message: "application.cv.summary is required",
    )
  }

  assert(
    type(options.generate_cl) == bool,
    message: "application.options.generate_cl must be a boolean",
  )
  if require-cl {
    let cover-letter-contract = load-cover-letter-contract()
    assert(options.generate_cl, message: "the cover-letter renderer requires options.generate_cl to be true")
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
  }
}
