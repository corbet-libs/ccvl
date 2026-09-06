#import "generated/tables.typ": countries-table, locales-table

/// Table key for a locale: exact code, base language, fallback.
#let base-language(code) = code.split("-").at(0)

#let resolve-key(locale) = {
  if locale in locales-table.locales {
    locale
  } else if base-language(locale) in locales-table.locales {
    base-language(locale)
  } else {
    locales-table.fallback
  }
}

/// Opening line: the `named` template with `{name}` filled when a name is
/// given, otherwise the formal address. An explicit override always wins.
#let opening(locale, name: none, override: none) = {
  if override != none {
    override
  } else {
    let entry = locales-table.locales.at(resolve-key(locale))
    if name != none and name.trim() != "" {
      entry.named.replace("{name}", name)
    } else {
      entry.formal
    }
  }
}

/// Subject line: prefix plus title, or the unsolicited default.
/// The override replaces the prefix only.
#let subject(locale, title: none, prefix-override: none) = {
  let entry = locales-table.locales.at(resolve-key(locale))
  if title != none and title.trim() != "" {
    let prefix = if prefix-override != none { prefix-override } else { entry.subject_prefix }
    prefix + " " + title
  } else {
    entry.subject_unsolicited
  }
}

/// Swiss orthography: ß→ss where the locale demands it.
#let apply-ortho(locale, text) = {
  if locales-table.locales.at(resolve-key(locale)).use_ss {
    text.replace("ß", "ss")
  } else {
    text
  }
}

/// Country extraction from a free-text location: keywords, then cantons.
#let country-from-location(location) = {
  let lower = location.to-lower().trim()
  if lower == "" {
    none
  } else {
    let found = none
    for (keyword, code) in countries-table.keywords {
      if lower.contains(keyword) {
        found = code
        break
      }
    }
    if found == none {
      for part in lower.split(regex("[,;]")) {
        if part.trim() in countries-table.cantons {
          found = "CH"
          break
        }
      }
    }
    found
  }
}

/// Resolve a language plus an optional location to a BCP 47 locale.
#let resolve-locale(language: none, location: none) = {
  let normalized = if language == none { "en" } else { language.trim() }
  if normalized == "" {
    "en"
  } else {
    let base = base-language(normalized.to-lower())
    let country = if location == none { none } else { country-from-location(location) }
    if country == none {
      normalized
    } else {
      locales-table.variants.at(base, default: (:)).at(country, default: base)
    }
  }
}
