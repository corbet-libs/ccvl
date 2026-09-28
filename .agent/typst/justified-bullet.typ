// Optional CV study treatment. Measure natural text before expanding spaces;
// justification must never make underfilled wording pass a content contract.
#import "/.agent/typst/line-contract.typ": line-contract-marker, measured-content-line

#let measured-bullet(id, kind, body, min-fill, target-fill, max-fill, justify: false) = {
  if not justify {
    measured-content-line(id, kind, body, min-fill, target-fill, max-fill)
  } else {
    layout(size => {
      line-contract-marker(id, kind, body, min-fill, target-fill, max-fill, size.width)
      box(width: size.width)[
        #set text(hyphenate: false)
        #set par(justify: true)
        #body#linebreak(justify: true)
      ]
    })
  }
}
