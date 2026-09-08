// Harvard CV: compact, de-ch. Defaults also make this leaf directly compilable.
#import "/cvl/cv/src/cv.typ": cv-renderer
#import "/cvl/cv/src/entries-de.typ": render-entries
#let cv-pages = int(sys.inputs.at("cv-pages", default: "4"))
#let application-path = sys.inputs.at("application", default: "/cvl/cv/compact/de/ch/content.toml")
#let profile-path = sys.inputs.at("profile", default: "/cvl/profile.toml")
#let strings-path = sys.inputs.at("strings", default: "/cvl/cv/compact/de/ch/strings.toml")
#let substyle-path = sys.inputs.at("substyle", default: "/cvl/cv/compact/substyle.toml")
#let shared-defaults-path = sys.inputs.at("shared-defaults", default: "/cvl/shared/defaults.toml")
#let cv = cv-renderer(
  application-path: application-path,
  profile-path: profile-path,
  cv-pages: cv-pages,
  strings-path: strings-path,
  substyle-path: substyle-path,
  shared-defaults-path: shared-defaults-path,
)
#let (apply-style, render-cv-start, assert-page-count) = cv
#show: apply-style
#render-cv-start()
#render-entries(cv)
#assert-page-count(cv-pages)
