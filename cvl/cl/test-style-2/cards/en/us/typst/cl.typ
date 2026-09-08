// test-style-2/cards/en-us: all inputs have workspace-relative defaults.
#import "/cvl/cl/test-style-2/parts/composition.typ": render
#render(
  application-path: sys.inputs.at("application", default: "/cvl/cl/test-style-2/cards/en/us/content.toml"),
  profile-path: sys.inputs.at("profile", default: "/cvl/profile.toml"),
  strings-path: sys.inputs.at("strings", default: "/cvl/cl/test-style-2/cards/en/us/strings.toml"),
  substyle-path: sys.inputs.at("substyle", default: "/cvl/cl/test-style-2/cards/substyle.toml"),
  defaults-path: sys.inputs.at("shared-defaults", default: "/cvl/shared/test-style-2/defaults.toml"),
  layout-path: sys.inputs.at("layout", default: "/cvl/cl/test-style-2/cards/en/us/layout.toml"),
  pages: int(sys.inputs.at("pages", default: "2")),
)
