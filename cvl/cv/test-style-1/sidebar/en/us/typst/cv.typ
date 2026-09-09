// test-style-1/sidebar/en-us: all inputs have workspace-relative defaults.
#import "/cvl/cv/test-style-1/layout.typ": render
#render(
  application-path: sys.inputs.at("application", default: "/cvl/cv/test-style-1/sidebar/en/us/content.toml"),
  profile-path: sys.inputs.at("profile", default: "/cvl/profile.toml"),
  strings-path: sys.inputs.at("strings", default: "/cvl/cv/test-style-1/sidebar/en/us/strings.toml"),
  substyle-path: sys.inputs.at("substyle", default: "/cvl/cv/test-style-1/sidebar/substyle.toml"),
  defaults-path: sys.inputs.at("shared-defaults", default: "/cvl/shared/test-style-1/defaults.toml"),
  layout-path: sys.inputs.at("layout", default: "/cvl/cv/test-style-1/sidebar/en/us/layout.toml"),
  paper-input: sys.inputs.at("paper", default: ""),
  pages: int(sys.inputs.at("pages", default: "1")),
)
