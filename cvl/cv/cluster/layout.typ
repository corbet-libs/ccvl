// Measured three-category openings; study records keep station placeholders explicit.
#import "/.agent/typst/application.typ": load-application
#import "/.agent/typst/profile.typ": load-profile
#import "/.agent/typst/paper.typ": paper-settings, resolve-paper
#import "/.agent/typst/document.typ": apply-document-settings
#import "/.agent/typst/line-contract.typ": measured-content-line, measured-paragraph, wrap-exact
#import "/.agent/typst/justified-bullet.typ": measured-bullet

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
  let application = load-application(application-path)
  let content = application.cv
  let strings = toml(strings-path)
  let (profile, localized-profile) = load-profile(profile-path)
  let localized = localized-profile.at(strings.locale)
  let paper = resolve-paper(
    toml("style.toml"),
    strings.locale,
    requested: paper-input,
    recorded: application.options.at("cv_paper", default: none),
  )
  let settings = paper-settings((toml(defaults-path), toml(substyle-path), toml(layout-path)), paper)
  assert(pages == 1, message: "Cluster opening page must request exactly one page")
  assert(application.options.language == strings.locale, message: "Cluster locale mismatch")
  assert(content.groups.len() == 3, message: "Cluster page requires exactly three categories")
  let allocation = settings.cluster.stations_per_group
  assert(allocation in ((2, 2, 2), (2, 3, 2)), message: "Unsupported cluster station allocation")
  assert(settings.cluster.category_extra_before_mm >= 0, message: "Category spacing must be nonnegative")
  assert(settings.cluster.heading_outer_gap_mm > 0, message: "Heading outer gap must be positive")
  assert(settings.cluster.heading_rule_gap_mm > 0, message: "Heading-to-rule gap must be positive")
  let station-ids = ()
  for (group-index, group) in content.groups.enumerate() {
    assert(
      group.stations.len() == allocation.at(group-index),
      message: "Category " + str(group-index + 1) + " must contain " + str(allocation.at(group-index)) + " stations",
    )
    for station in group.stations {
      assert(station.bullets.len() == 3, message: "Each cluster station requires exactly three bullets")
      assert(station.id not in station-ids, message: "A station cannot occur in two clusters")
      station-ids.push(station.id)
    }
  }
  set document(title: "Cluster CV | " + profile.name, author: (profile.name,))
  show: apply-document-settings.with(settings)
  show link: it => text(fill: rgb(settings.accents.link), it)

  let measured-row(id, kind, body, minimum, target) = measured-content-line(
    id,
    kind,
    body,
    minimum,
    target,
    100,
  )
  let bullet-mark = box(width: 10.5pt, height: 7.35pt, align(horizon, align(center, polygon(
    fill: black,
    (0pt, 0pt),
    (4.41pt, 2.75625pt),
    (0pt, 5.5125pt),
  ))))
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
  align(center, text(size: 15.75pt, weight: "bold", strings.demo_heading))
  v(settings.header.after_pt * 1pt)
  align(center, text(size: 9.03pt, contacts.join([ | ])))

  // Bound the title to its glyphs, then compensate neighboring lines for their
  // descenders/ascenders so the outer gaps are measured from visible ink.
  let optical-section(title, previous, following, id: none) = context {
    let gap = settings.cluster.heading_outer_gap_mm * 1mm
    let previous-descent = measure(text(top-edge: 0pt, bottom-edge: "bounds", previous)).height
    let following-ascent = measure(text(top-edge: "bounds", bottom-edge: 0pt, following)).height
    let following-frame = measure(text(top-edge: settings.text.top_edge, bottom-edge: 0pt, following)).height
    v(gap + previous-descent)
    let heading = text(size: 12pt, weight: "bold", top-edge: "bounds", bottom-edge: "bounds", upper(title))
    if id == none { heading } else { measured-row(id, "cluster-category", heading, 15, 60) }
    v(settings.cluster.heading_rule_gap_mm * 1mm)
    block(width: 100%, height: 0.5pt, fill: black, above: 0pt, below: 0pt)
    v(gap + following-ascent - following-frame)
  }

  let section(title) = [
    #v(settings.cv.compact_heading_spacing_pt * 1pt)
    #text(size: 12pt, weight: "bold", upper(title))
    #v(settings.cv.rule_gap_pt * 1pt)
    #line(length: 100%, stroke: 0.5pt + black)
    #v(settings.cv.compact_heading_spacing_pt * 1pt)
  ]
  block(width: 100%, breakable: false)[
    #if settings.cluster.optical_headings {
      optical-section(strings.summary, text(size: 9.03pt, contacts.join([ | ])), text(content.summary))
    } else { section(strings.summary) }
    #set text(hyphenate: false)
    #layout(size => {
      let lines = wrap-exact(content.summary, size.width, 5, "cluster.summary", last-max: 102)
      measured-paragraph(
        "cluster.summary",
        "cv-summary",
        lines
          .enumerate()
          .map(((i, value)) => (
            text: value,
            // Preserve an explicitly accepted summary without padding its prose.
            min_fill: if content.at("allow_thin", default: false) { 1 } else { 95 },
            target_fill: 97,
            max_fill: if i == 4 { 102 } else { 100 },
          )),
        justify: true,
      )
    })
  ]
  for (group-index, group) in content.groups.enumerate() {
    block(width: 100%, breakable: false)[
      #if settings.cluster.optical_headings {
        let previous = if group-index == 0 { content.summary } else {
          content.groups.at(group-index - 1).stations.last().bullets.last()
        }
        optical-section(
          group.title,
          text(previous),
          text(size: 11pt, weight: "bold", group.stations.first().heading),
          id: "cluster.category." + str(group-index + 1),
        )
      } else {
        v(
          settings.cv.compact_heading_spacing_pt * 1pt
            + if group-index > 0 {
              settings.cluster.category_extra_before_mm * 1mm
            } else { 0mm },
        )
        measured-row(
          "cluster.category." + str(group-index + 1),
          "cluster-category",
          text(size: 12pt, weight: "bold", upper(group.title)),
          15,
          60,
        )
        v(settings.cv.rule_gap_pt * 1pt)
        line(length: 100%, stroke: 0.5pt + black)
        v(settings.cv.compact_heading_spacing_pt * 1pt)
      }
      #for (station-index, station) in group.stations.enumerate() {
        if station-index > 0 { v(settings.cv.entry_spacing_pt * 1pt) }
        measured-row(
          station.id + ".heading",
          "cluster-station",
          text(size: 11pt, weight: "bold", station.heading),
          8,
          45,
        )
        v(settings.cv.heading_after_pt * 1pt)
        measured-row(station.id + ".role", "cluster-role", text(size: 10pt, station.role), 35, 65)
        v(settings.cv.subheading_after_pt * 1pt)
        for (bullet-index, bullet) in station.bullets.enumerate() {
          if bullet-index > 0 { v(settings.cv.bullet_after_pt * 1pt) }
          grid(
            columns: (settings.cv.bullet_indent_pt * 1pt, 1fr),
            gutter: 0pt,
            bullet-mark,
            measured-bullet(
              station.id + ".bullet." + str(bullet-index + 1),
              "cluster-bullet",
              text(bullet),
              80,
              90,
              100,
              justify: settings.cv.at("justify_bullets", default: false),
            ),
          )
        }
      }
    ]
  }
  context {
    assert(counter(page).final().first() == 1, message: "Cluster content overflowed its single page")
  }
}
