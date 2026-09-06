#import "generated/closing.typ": closing-table

/// Valediction for a BCP 47 locale: exact code, base language, English
/// fallback. An explicit override always wins.
#let base-language(locale) = locale.split("-").at(0)

#let resolve-key(locale) = {
  if locale in closing-table.locales {
    locale
  } else if base-language(locale) in closing-table.locales {
    base-language(locale)
  } else {
    closing-table.fallback
  }
}

#let closing(locale, override: none) = {
  if override != none {
    override
  } else {
    closing-table.locales.at(resolve-key(locale), default: "")
  }
}
