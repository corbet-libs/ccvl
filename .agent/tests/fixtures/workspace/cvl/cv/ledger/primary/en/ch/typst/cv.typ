// Synthetic ledger cv leaf: primary, en-ch. Defaults support direct compilation.
#import "/cvl/shared/ledger/render.typ": render-cv
#render-cv(
  application-path: sys.inputs.at("application", default: "/cvl/cv/ledger/primary/en/ch/content.toml"),
  profile-path: sys.inputs.at("profile", default: "/cvl/profile.toml"),
  strings-path: sys.inputs.at("strings", default: "/cvl/cv/ledger/primary/en/ch/strings.toml"),
  substyle-path: sys.inputs.at("substyle", default: "/cvl/cv/ledger/primary/substyle.toml"),
  shared-defaults-path: sys.inputs.at("shared-defaults", default: "/cvl/shared/ledger/defaults.toml"),
  layout-path: sys.inputs.at("layout", default: "/cvl/cv/ledger/primary/en/ch/layout.toml"),
  paper-input: sys.inputs.at("paper", default: ""),
  pages: int(sys.inputs.at("pages", default: "4")),
)
