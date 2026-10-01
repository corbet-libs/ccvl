// Synthetic independent grid layout: two columns and fixed item metrics.
#import "/.agent/typst/application.typ": load-application
#import "/.agent/typst/document.typ": apply-document-settings
#import "/.agent/typst/paper.typ": paper-settings, resolve-paper
#import "/.agent/typst/profile.typ": load-profile

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
  let strings = toml(strings-path)
  let (profile, localized-profile) = load-profile(profile-path)
  let paper = resolve-paper(
    toml("style.toml"),
    strings.locale,
    requested: paper-input,
    recorded: record.options.at("cv_paper", default: none),
  )
  show: apply-document-settings.with(paper-settings(
    (toml(defaults-path), toml(substyle-path), toml(layout-path)),
    paper,
  ))
  grid(
    columns: (1fr, 2fr),
    gutter: 12pt,
    [#profile.email #linebreak() #strings.heading],
    [
      #record.cv.summary
      #for (index, item) in record.cv.items.enumerate() {
        [#metadata((
            kind: "grid-item",
            id: "grid.item." + str(index + 1),
            text: item,
            actual_fill: 80,
            min_fill: 60,
            target_fill: 80,
            max_fill: 100,
          )) <ccvl-line>]
        parbreak()
        item
      }
    ],
  )
}
