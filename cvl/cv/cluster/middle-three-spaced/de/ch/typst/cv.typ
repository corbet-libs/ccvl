// Standalone one-page cluster opening; the comparison script appends Harvard.
#import "/cvl/cv/cluster/layout.typ": render
#render(
  application-path: sys.inputs.at("application", default: "/cvl/cv/cluster/middle-three-spaced/de/ch/content.toml"),
  profile-path: sys.inputs.at("profile", default: "/cvl/profile.toml"),
  strings-path: sys.inputs.at("strings", default: "/cvl/cv/cluster/middle-three-spaced/de/ch/strings.toml"),
  substyle-path: sys.inputs.at("substyle", default: "/cvl/cv/cluster/middle-three-spaced/substyle.toml"),
  defaults-path: sys.inputs.at("shared-defaults", default: "/cvl/shared/cluster/defaults.toml"),
  layout-path: sys.inputs.at("layout", default: "/cvl/cv/cluster/middle-three-spaced/de/ch/layout.toml"),
  paper-input: sys.inputs.at("paper", default: ""),
  pages: int(sys.inputs.at("pages", default: "1")),
)
