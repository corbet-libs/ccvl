#import "/cvl/cv/modern/layout.typ": render
#render(
  application-path: sys.inputs.at("application", default: "/cvl/cv/modern/standard/de/ch/content.toml"),
  profile-path: sys.inputs.at("profile", default: "/cvl/profile.toml"),
  strings-path: sys.inputs.at("strings", default: "/cvl/cv/modern/standard/de/ch/strings.toml"),
  substyle-path: sys.inputs.at("substyle", default: "/cvl/cv/modern/standard/substyle.toml"),
  defaults-path: sys.inputs.at("shared-defaults", default: "/cvl/shared/modern/defaults.toml"),
  layout-path: sys.inputs.at("layout", default: "/cvl/cv/modern/standard/de/ch/layout.toml"),
  paper-input: sys.inputs.at("paper", default: ""),
  pages: int(sys.inputs.at("pages", default: "1")),
)
