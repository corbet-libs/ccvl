// test-style-2/cards/en-ch: all inputs have workspace-relative defaults.
#import "/cvl/cv/test-style-2/parts/composition.typ": render
#render(
  application-path: sys.inputs.at("application", default: "/cvl/cv/test-style-2/cards/en/ch/content.toml"),
  profile-path: sys.inputs.at("profile", default: "/cvl/profile.toml"),
  strings-path: sys.inputs.at("strings", default: "/cvl/cv/test-style-2/cards/en/ch/strings.toml"),
  substyle-path: sys.inputs.at("substyle", default: "/cvl/cv/test-style-2/cards/substyle.toml"),
  defaults-path: sys.inputs.at("shared-defaults", default: "/cvl/shared/test-style-2/defaults.toml"),
  layout-path: sys.inputs.at("layout", default: "/cvl/cv/test-style-2/cards/en/ch/layout.toml"),
  pages: int(sys.inputs.at("pages", default: "2")),
)
