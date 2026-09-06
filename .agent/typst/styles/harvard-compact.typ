// harvard-compact: thin whitespace-only variant of the Harvard default.
// Same renderer, different knobs: the defaults below point at
// harvard-compact.toml while every template keeps working unchanged.

#import "harvard.typ": document-style as harvard-style, load-style

#let document-style(locale: "en-ch", style: load-style("harvard-compact"), doc) = {
  harvard-style(locale: locale, style: style, doc)
}
