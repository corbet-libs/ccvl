// Harvard cover letter: frame, en-ch. Defaults also support direct compilation.
#import "/cvl/cl/src/cl.typ": render-cl
#let application-path = sys.inputs.at("application", default: "/cvl/cl/frame/en/ch/content.toml")
#let profile-path = sys.inputs.at("profile", default: "/cvl/profile.toml")
#let strings-path = sys.inputs.at("strings", default: "/cvl/cl/frame/en/ch/strings.toml")
#let substyle-path = sys.inputs.at("substyle", default: "/cvl/cl/frame/substyle.toml")
#let shared-defaults-path = sys.inputs.at("shared-defaults", default: "/cvl/shared/defaults.toml")
#render-cl(
  application-path: application-path,
  profile-path: profile-path,
  strings-path: strings-path,
  substyle-path: substyle-path,
  shared-defaults-path: shared-defaults-path,
)
