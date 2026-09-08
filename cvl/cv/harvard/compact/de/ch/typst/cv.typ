// Harvard CV: compact, de-ch. Defaults also make this leaf directly compilable.
#import "/cvl/cv/harvard/src/cv.typ": cv-renderer
#import "/cvl/cv/harvard/src/entries-de.typ": render-entries
#let cv-pages = int(sys.inputs.at("pages", default: "4"))
#let application-path = sys.inputs.at("application", default: "/cvl/cv/harvard/compact/de/ch/content.toml")
#let profile-path = sys.inputs.at("profile", default: "/cvl/profile.toml")
#let strings-path = sys.inputs.at("strings", default: "/cvl/cv/harvard/compact/de/ch/strings.toml")
#let substyle-path = sys.inputs.at("substyle", default: "/cvl/cv/harvard/compact/substyle.toml")
#let layout-path = sys.inputs.at("layout", default: "/cvl/cv/harvard/compact/de/ch/layout.toml")
#let shared-defaults-path = sys.inputs.at("shared-defaults", default: "/cvl/shared/harvard/defaults.toml")
#let cv = cv-renderer(
  application-path: application-path,
  profile-path: profile-path,
  cv-pages: cv-pages,
  strings-path: strings-path,
  substyle-path: substyle-path,
  shared-defaults-path: shared-defaults-path,
  layout-path: layout-path,
)
#let (apply-style, render-cv-start, assert-page-count) = cv
#show: apply-style
#render-cv-start()
#render-entries(cv)
#assert-page-count(cv-pages)
