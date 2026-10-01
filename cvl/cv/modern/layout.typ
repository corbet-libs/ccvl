// Modern CV renderer. Scaffold only: renders the approved profile header until the design is defined.
#import "/.agent/typst/application.typ": load-application
#import "/.agent/typst/profile.typ": load-profile
#import "/.agent/typst/paper.typ": paper-settings, resolve-paper
#import "/.agent/typst/document.typ": apply-document-settings

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
  assert(pages == 1, message: "Modern requests exactly one page")
  assert(application.options.language == strings.locale, message: "Modern locale mismatch")
  set document(title: "Modern CV | " + profile.name, author: (profile.name,))
  show: apply-document-settings.with(settings)

  let contacts = (
    profile.email,
    profile.phone-label,
    profile.location,
    profile.languages,
    localized.nationality-and-permit,
    localized.availability,
  ).filter(item => item != none)
  text(weight: "bold", profile.name)
  linebreak()
  contacts.join([ | ])
}
