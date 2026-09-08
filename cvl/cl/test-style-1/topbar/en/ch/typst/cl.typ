// test-style-1/topbar/en-ch: all inputs have workspace-relative defaults.
#import "/cvl/cl/test-style-1/layout.typ": render
#render(
  application-path: sys.inputs.at("application", default: "/cvl/cl/test-style-1/topbar/en/ch/content.toml"),
  profile-path: sys.inputs.at("profile", default: "/cvl/profile.toml"),
  strings-path: sys.inputs.at("strings", default: "/cvl/cl/test-style-1/topbar/en/ch/strings.toml"),
  substyle-path: sys.inputs.at("substyle", default: "/cvl/cl/test-style-1/topbar/substyle.toml"),
  defaults-path: sys.inputs.at("shared-defaults", default: "/cvl/shared/test-style-1/defaults.toml"),
  layout-path: sys.inputs.at("layout", default: "/cvl/cl/test-style-1/topbar/en/ch/layout.toml"),
  pages: int(sys.inputs.at("pages", default: "1")),
)
