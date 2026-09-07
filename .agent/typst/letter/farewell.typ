#import "generated/closing.typ": closing-table

/// Valediction for a BCP 47 locale: exact code, lowercased exact,
/// lowercased base language, English fallback. Tables keep canonical BCP 47
/// keys; inputs resolve case-insensitively so `de-ch` == `de-CH`.
/// An explicit override always wins.
#let base-language(locale) = locale.split("-").at(0)

#let resolve-key(locale) = {
  if locale in closing-table.locales {
    locale
  } else {
    let lowered = lower(locale)
    let exact = closing-table.locales.keys().filter(k => lower(k) == lowered).at(0, default: none)
    if exact != none {
      exact
    } else {
      let base-lowered = lower(base-language(locale))
      let base = closing-table.locales.keys().filter(k => lower(k) == base-lowered).at(0, default: none)
      if base != none {
        base
      } else {
        closing-table.fallback
      }
    }
  }
}

#let closing(locale, override: none) = {
  if override != none {
    override
  } else {
    closing-table.locales.at(resolve-key(locale), default: "")
  }
}
