// Harvard cover letter: left-rule, en-ch. Defaults also support direct compilation.
#import "/cvl/cl/harvard/src/cl.typ": render-cl
#let application-path = sys.inputs.at("application", default: "/cvl/cl/harvard/left-rule/en/ch/content.toml")
#let profile-path = sys.inputs.at("profile", default: "/cvl/profile.toml")
#let strings-path = sys.inputs.at("strings", default: "/cvl/cl/harvard/left-rule/en/ch/strings.toml")
#let substyle-path = sys.inputs.at("substyle", default: "/cvl/cl/harvard/left-rule/substyle.toml")
#let paper-input = sys.inputs.at("paper", default: "")
#let layout-path = sys.inputs.at("layout", default: "/cvl/cl/harvard/left-rule/en/ch/layout.toml")
#let shared-defaults-path = sys.inputs.at("shared-defaults", default: "/cvl/shared/harvard/defaults.toml")
#render-cl(
  application-path: application-path,
  profile-path: profile-path,
  strings-path: strings-path,
  substyle-path: substyle-path,
  shared-defaults-path: shared-defaults-path,
  layout-path: layout-path,
  paper-input: paper-input,
)
