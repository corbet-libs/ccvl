#import "/.agent/typst/paper.typ": paper-settings, resolve-paper
#import "/.agent/typst/application.typ": load-application
// Shared Harvard CV renderer. Each leaf supplies its inputs explicitly.
#import "/cvl/shared/harvard/style.typ": document-style
#import "/cvl/shared/harvard/application.typ": load-cv-contract, validate-application
#import "/.agent/typst/line-contract.typ": measured-content-line, measured-paragraph, wrap-exact
#import "/.agent/typst/justified-bullet.typ": measured-bullet
#import "/.agent/typst/profile.typ": load-profile

#let cv-contract = load-cv-contract()
// The CV closing line has its own allowance; CL paragraphs use their own bounds.
#let last-line-maximum = cv-contract.at("last_line_maximum", default: 102)


#let cv-renderer(
  application-path: none,
  profile-path: none,
  cv-pages: none,
  strings-path: none,
  substyle-path: none,
  shared-defaults-path: none,
  layout-path: none,
  paper-input: "",
) = {
  let (profile, localized-profile) = load-profile(profile-path)
  // Merge the shared base knobs with this leaf's substyle delta (a standard
  // leaf merges an empty delta). Whitespace below comes from cv-style, never
  // from forked literals.
  let cv-strings = toml(strings-path)
  // Locale comes from the leaf's strings file and is cross-checked against
  // the record below: a de-ch record through en-ch strings fails here.
  let doc-locale = cv-strings.locale
  let application = load-application(application-path)
  let paper = resolve-paper(toml("../style.toml"), doc-locale, requested: paper-input, recorded: application.options.at(
    "cv_paper",
    default: none,
  ))
  let cv-style = paper-settings((toml(shared-defaults-path), toml(substyle-path), toml(layout-path)), paper)
  validate-application(application, expected-language: doc-locale, require-cv: true)

  // Style whitespace knobs from the active style; the element styles below
  // consume these instead of forked literals.
  let cv-superheading-outer-spacing = cv-style.cv.superheading_outer_spacing_pt * 1pt
  let cv-compact-heading-spacing = cv-style.cv.compact_heading_spacing_pt * 1pt
  let cv-spacious-heading-spacing = cv-style.cv.spacious_heading_spacing_pt * 1pt
  let cv-entry-spacing = cv-style.cv.entry_spacing_pt * 1pt
  let cv-heading-after = cv-style.cv.heading_after_pt * 1pt
  let cv-subheading-after = cv-style.cv.subheading_after_pt * 1pt
  let cv-bullet-after = cv-style.cv.bullet_after_pt * 1pt
  let cv-competency-heading-after = cv-style.cv.competency_heading_after_pt * 1pt
  let cv-rule-gap = cv-style.cv.rule_gap_pt * 1pt
  let cv-superheading-inner = cv-style.cv.superheading_inner_pt * 1pt
  let cv-header-after = cv-style.header.after_pt * 1pt
  let cv-bullet-indent = cv-style.cv.bullet_indent_pt * 1pt
  let balance-headings = cv-style.cv.at("balance_headings", default: false)
  // Advance only at explicit content-page boundaries. Physical page numbers
  // would make spacing depend on the pagination it changes.
  let content-page = counter("harvard-content-page")
  let spacing-overrides = cv-style.cv.at("spacing_by_page_pt", default: (:))

  let cv-gap(name) = {
    let extras = if name == "entry_spacing_pt" {
      cv-style.cv.at("entry_extra_by_page_mm", default: (:))
    } else { (:) }
    if spacing-overrides.len() == 0 and extras.len() == 0 {
      v(cv-style.cv.at(name) * 1pt)
    } else {
      context {
        let page = str(content-page.get().first())
        let values = spacing-overrides.at(page, default: (:))
        v(values.at(name, default: cv-style.cv.at(name)) * 1pt + extras.at(page, default: 0) * 1mm)
      }
    }
  }

  // Visible CV presentation: every element style and the letterhead live here
  // so editors never hunt below .agent/typst. Only page setup, measurement,
  // validation, and profile data stay imported.
  let cv-bullet() = box(width: 10.5pt, height: 7.35pt, align(horizon, align(center, polygon(
    fill: rgb("#000000"),
    (0pt, 0pt),
    (4.41pt, 2.75625pt),
    (0pt, 5.5125pt),
  ))))

  // Entry heading (bold). Override size: #cv-h(size: 14pt)[...]
  let cv-h(size: 11pt, min-fill: 15, target-fill: 45, max-fill: 100, t) = measured-content-line(
    "cv.heading",
    "cv-heading",
    text(size: size, weight: "bold", t),
    min-fill,
    target-fill,
    max-fill,
  )
  let cv-hu(size: 11pt, min-fill: 60, target-fill: 85, max-fill: 100, t) = {
    set strong(delta: -300)
    measured-content-line(
      "cv.emphasized-heading",
      "cv-emphasized-heading",
      text(size: size, weight: "bold", t),
      min-fill,
      target-fill,
      max-fill,
    )
  }
  // Entry subheading. Override size: #cv-s(size: 9pt)[...]
  let cv-s(size: 10pt, min-fill: 35, target-fill: 65, max-fill: 100, t) = measured-content-line(
    "cv.subheading",
    "cv-subheading",
    text(size: size, t),
    min-fill,
    target-fill,
    max-fill,
  )
  // Bullet row. Override indent/gutter: #cv-b(indent: 12pt)[...]
  let cv-b(indent: cv-bullet-indent, gutter: 0pt, min-fill: 80, target-fill: 90, max-fill: 100, t) = {
    let body = grid(
      columns: (indent, 1fr),
      gutter: gutter,
      cv-bullet(),
      measured-bullet(
        "cv.bullet",
        "cv-bullet",
        t,
        min-fill,
        target-fill,
        max-fill,
        justify: cv-style.cv.at("justify_bullets", default: false),
      ),
    )
    let extras = cv-style.cv.at("bullet_extra_by_page_mm", default: (:))
    if extras.len() == 0 { body } else {
      context {
        let extra = extras.at(str(content-page.get().first()), default: 0) * 1mm
        if extra != 0mm { v(extra) }
        body
      }
    }
  }

  let cv-entry-gap() = cv-gap("entry_spacing_pt")

  // Page-level heading for dedicated CV pages such as projects or competencies.
  let cv-superheading(t) = {
    let body = block(width: 100%, breakable: false, inset: (top: cv-superheading-outer-spacing))[
      #set par(spacing: 0pt)
      #line(length: 100%, stroke: 0.5pt + black)
      #v(cv-superheading-inner)
      #align(center, text(size: 17pt, weight: "bold", upper(t)))
      #v(cv-superheading-inner)
      #line(length: 100%, stroke: 0.5pt + black)
    ]
    if balance-headings { move(dy: cv-style.cv.superheading_shift_pt * 1pt, body) } else { body }
  }

  // Preserve the allocated slot; move only its heading and rule to centre the
  // visible whitespace. This translation does not shift the following text.
  let cv-section-heading(spacing, t, shift: 0pt) = {
    let body = block(breakable: false)[
      #v(spacing)
      #set par(spacing: 0pt)
      #text(size: 12pt, weight: "bold", upper(t))
      #v(cv-rule-gap)
      #line(length: 100%, stroke: 0.5pt + black)
      #v(spacing)
    ]
    if balance-headings {
      context {
        let title = text(size: 12pt, weight: "bold", upper(t))
        let ink-ascent = measure(text(top-edge: "bounds", bottom-edge: 0pt, title)).height
        let cap-ascent = measure(text(top-edge: "cap-height", bottom-edge: 0pt, title)).height
        move(dy: shift + calc.max(0pt, ink-ascent - cap-ascent) / 2, body)
      }
    } else { body }
  }

  // Compact section heading for dense CV pages. The aligned continuation page
  // shares a small extra gap equally above/below its four headings, extending
  // to the project page's bottom edge without changing any entry's own spacing.
  let cv-compact-heading(t) = {
    let continuation-titles = (
      cv-strings.education,
      cv-strings.professional_development,
      cv-strings.engagement,
      cv-strings.personal,
    )
    let extra = if continuation-titles.any(title => t == [#title]) {
      cv-style.cv.at("continuation_heading_extra_pt", default: 0) * 1pt
    } else { 0pt }
    let shift = cv-style.cv.at("compact_heading_shift_pt", default: 0)
    if t == [#cv-strings.education] {
      shift = cv-style.cv.at("first_continuation_heading_shift_pt", default: shift)
    }
    cv-section-heading(
      cv-compact-heading-spacing + extra,
      t,
      shift: shift * 1pt,
    )
  }

  // Spacious section heading for dedicated project and competency pages.
  let cv-spacious-heading(t) = {
    let first = t == [#cv-strings.projects_ongoing] or t == [#cv-strings.pillar_ai]
    cv-section-heading(
      cv-spacious-heading-spacing,
      t,
      shift: cv-style.cv.at(
        if first { "first_spacious_heading_shift_pt" } else { "spacious_heading_shift_pt" },
        default: 0,
      )
        * 1pt,
    )
  }

  // Keep named CV variants honest: a fourpager must render exactly four pages.
  let assert-page-count(expected) = context {
    let actual = counter(page).final().first()
    assert(actual == expected, message: "CV rendered " + str(actual) + " pages; expected " + str(expected))
  }

  let brand(body) = box(body)
  let cv-header() = {
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
    v(cv-header-after)
    align(center)[
      #text(size: 9.03pt)[
        #contacts.join([ | ])
      ]
    ]
  }
  let cv-pagebreak() = [
    #content-page.step()
    #pagebreak()
    #cv-header()
  ]

  // Header plus Summary, sequenced by the leaf adapter after importing this
  // module. Kept in a function (not top-level content) so entries files and
  // test fixtures can import this module's bindings without emitting output.
  let render-cv-start() = [
    #content-page.update(1)
    #cv-header()

    #block(width: 100%, breakable: false)[
      #cv-compact-heading[Summary]
      // The record holds one flowing paragraph; wrapping to exactly five lines
      // happens here so authors never count breaks by hand.
      #set text(hyphenate: false)
      #layout(size => {
        let fill = cv-contract.summary_fill
        let thin-ok = "allow_thin" in application.cv and application.cv.allow_thin
        let lines = wrap-exact(
          application.cv.summary,
          size.width,
          5,
          "application.cv.summary",
          last-max: last-line-maximum,
        )
        let mapped = range(lines.len()).map(index => {
          let line = lines.at(index)
          (
            text: line,
            min_fill: if thin-ok { 1 } else { fill.minimum },
            target_fill: fill.target,
            max_fill: if index + 1 == lines.len() { last-line-maximum } else { fill.maximum },
          )
        })
        // The five wrapped lines render as one justified paragraph with a
        // left-bound closing line, exactly like a cover-letter paragraph. An
        // approved closing-line spill keeps its fixed-width box so it extends
        // invisibly into the margin instead of wrapping.
        measured-paragraph("cv.summary", "cv-summary", mapped, justify: true)
      })
    ]
  ]

  let apply-style(body) = {
    set document(title: "Curriculum Vitae | " + profile.name, author: (profile.name,))
    document-style(locale: doc-locale, style: cv-style, body)
  }
  (
    brand: brand,
    cv-b: cv-b,
    cv-bullet-after: cv-bullet-after,
    cv-compact-heading: cv-compact-heading,
    cv-competency-heading-after: cv-competency-heading-after,
    cv-entry-gap: cv-entry-gap,
    cv-gap: cv-gap,
    cv-h: cv-h,
    cv-heading-after: cv-heading-after,
    cv-hu: cv-hu,
    cv-pagebreak: cv-pagebreak,
    cv-pages: cv-pages,
    cv-s: cv-s,
    cv-spacious-heading: cv-spacious-heading,
    cv-strings: cv-strings,
    cv-subheading-after: cv-subheading-after,
    cv-superheading: cv-superheading,
    render-cv-start: render-cv-start,
    assert-page-count: assert-page-count,
    apply-style: apply-style,
  )
}
