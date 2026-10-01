// Synthetic ledger cl leaf: frame, de-ch. Defaults support direct compilation.
#import "/cvl/shared/ledger/render.typ": render-cl
#render-cl(
  application-path: sys.inputs.at("application", default: "/cvl/cl/ledger/frame/de/ch/content.toml"),
  profile-path: sys.inputs.at("profile", default: "/cvl/profile.toml"),
  strings-path: sys.inputs.at("strings", default: "/cvl/cl/ledger/frame/de/ch/strings.toml"),
  substyle-path: sys.inputs.at("substyle", default: "/cvl/cl/ledger/frame/substyle.toml"),
  shared-defaults-path: sys.inputs.at("shared-defaults", default: "/cvl/shared/ledger/defaults.toml"),
  layout-path: sys.inputs.at("layout", default: "/cvl/cl/ledger/frame/de/ch/layout.toml"),
  paper-input: sys.inputs.at("paper", default: ""),
)
