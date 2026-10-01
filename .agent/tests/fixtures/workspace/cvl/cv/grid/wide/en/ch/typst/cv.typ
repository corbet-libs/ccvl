// Synthetic grid leaf: wide, en-ch. Defaults support direct compilation.
#import "/cvl/cv/grid/layout.typ": render
#render(
  application-path: sys.inputs.at("application", default: "/cvl/cv/grid/wide/en/ch/content.toml"),
  profile-path: sys.inputs.at("profile", default: "/cvl/profile.toml"),
  strings-path: sys.inputs.at("strings", default: "/cvl/cv/grid/wide/en/ch/strings.toml"),
  substyle-path: sys.inputs.at("substyle", default: "/cvl/cv/grid/wide/substyle.toml"),
  defaults-path: sys.inputs.at("shared-defaults", default: "/cvl/shared/ledger/defaults.toml"),
  layout-path: sys.inputs.at("layout", default: "/cvl/cv/grid/wide/en/ch/layout.toml"),
  paper-input: sys.inputs.at("paper", default: ""),
  pages: int(sys.inputs.at("pages", default: "1")),
)
