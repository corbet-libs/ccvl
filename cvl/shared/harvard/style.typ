// Harvard selects values; the neutral adapter applies the declared Typst settings.
#import "/.agent/typst/document.typ": apply-document-settings, merge-settings
#let merge-style = merge-settings
#let document-style(locale: "en-ch", style: none, doc) = {
  assert(style != none, message: "Harvard needs explicit settings")
  assert(style.text.lang + "-" + lower(style.text.region) == locale, message: "layout locale mismatch")
  show: apply-document-settings.with(style)
  show link: it => text(fill: rgb(style.accents.link), it)
  doc
}
