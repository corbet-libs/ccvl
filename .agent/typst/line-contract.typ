// Measured single-line presentation contract shared by every document renderer.
#let line-contract-mode = sys.inputs.at("line-contracts", default: "enforce")
#assert(
  line-contract-mode in ("enforce", "report"),
  message: "line-contracts must be enforce or report",
)

#let line-contract-marker(
  id,
  kind,
  body,
  min-fill,
  target-fill,
  max-fill,
  available-width,
  source-text: none,
) = {
  let measured = measure(body)
  let actual-fill = calc.round(1000 * measured.width / available-width) / 10
  let metric = (
    id: id,
    kind: kind,
    text: if source-text == none { repr(body) } else { source-text },
    actual_fill: actual-fill,
    min_fill: min-fill,
    target_fill: target-fill,
    max_fill: max-fill,
  )
  if line-contract-mode == "enforce" {
    let guidance = " Rewrite with relevant, verified signal; then run the line measurement again."
    assert(
      actual-fill >= min-fill,
      message: id
        + " is too short: measured "
        + str(actual-fill)
        + "%, target "
        + str(target-fill)
        + "%, allowed "
        + str(min-fill)
        + "–"
        + str(max-fill)
        + "%."
        + guidance,
    )
    assert(
      actual-fill <= max-fill,
      message: id
        + " is too long: measured "
        + str(actual-fill)
        + "%, target "
        + str(target-fill)
        + "%, allowed "
        + str(min-fill)
        + "–"
        + str(max-fill)
        + "%."
        + guidance,
    )
  }
  [#metadata(metric) <ccvl-line>]
}

#let measured-content-line(
  id,
  kind,
  body,
  min-fill,
  target-fill,
  max-fill,
  source-text: none,
  // Pre-measured lines (wrap-exact output) render in a box fixed to their
  // natural width. An auto-width box re-wraps any spill past 100% onto a new
  // visual line, which silently adds lines the metric gate approved. A fixed
  // box spills at most `max-fill` invisibly into the margin instead.
  exact-width: false,
) = layout(size => {
  [
    #line-contract-marker(
      id,
      kind,
      body,
      min-fill,
      target-fill,
      max-fill,
      size.width,
      source-text: source-text,
    )
    #if exact-width {
      box(width: measure(body).width, body)
    } else {
      box(body)
    }
  ]
})

#let measured-line(id, kind, contract, exact-width: false) = measured-content-line(
  id,
  kind,
  text(contract.text),
  contract.min_fill,
  contract.target_fill,
  contract.max_fill,
  source-text: contract.text,
  exact-width: exact-width,
)

#let measured-lines(id, kind, lines, exact-width: false) = {
  for (index, line) in lines.enumerate() {
    measured-line(id + "." + str(index + 1), kind, line, exact-width: exact-width)
    if index < lines.len() - 1 {
      linebreak()
    }
  }
}

// Wrap flowing text into exactly `count` lines at the current style.
// Breaks happen only at spaces, so the measured widths below are exactly
// what the renderer lays out. Unlike greedy filling (which strands runt
// last lines), this packs with dynamic programming: among all exact-`count`
// packings it keeps the one with the fullest thinnest line.
// Non-final lines stay within 100%; the closing line may use `last-max`
// (uniform closing-line grace for every measured paragraph). Anything
// beyond is gross overflow and fails here.
#let wrap-exact(
  text,
  width,
  count,
  scope,
  last-max: 102,
) = {
  // Split on single spaces and drop the gaps from runs of whitespace, so no
  // regex engine behavior can smuggle fragments into the word list.
  let words = text.trim().split(" ").filter(word => word != "")
  assert(words.len() > 0, message: scope + ": nothing to lay out")
  let total = words.len()
  // Candidate lines from each start word, with exact measured fills.
  // Plain loops (not mapped closures) so every measure call runs in the
  // caller's layout context.
  let candidates = ()
  for start in range(total) {
    let options = ()
    let line = ""
    for end in range(start + 1, total + 1) {
      line = if end == start + 1 { words.at(start) } else { line + " " + words.at(end - 1) }
      let fill = calc.round(1000 * measure(box(line)).width / width) / 10
      let cap = if end == total { last-max } else { 100 }
      if fill > cap { break }
      options.push((end: end, fill: fill))
    }
    candidates.push(options)
  }
  // No per-word pre-check here: it misfired on valid input, while the
  // packing assert below plus the measured line gate cover truly overfull
  // words with precise locations.
  // best["k:i"]: fullest thinnest line and predecessor start for the first
  // i words packed into exactly k lines; absent when unreachable. A flat
  // dictionary keeps every access to definitely supported primitives.
  let best = (:)
  best.insert("0:0", (fill: last-max + 1, prev: -1))
  for k in range(1, count + 1) {
    for start in range(total) {
      let prev = best.at(str(k - 1) + ":" + str(start), default: none)
      if prev == none { continue }
      for option in candidates.at(start) {
        let fill = calc.min(prev.fill, option.fill)
        let key = str(k) + ":" + str(option.end)
        let current = best.at(key, default: none)
        if current == none or fill > current.fill {
          best.insert(key, (fill: fill, prev: start))
        }
      }
    }
  }
  let final = best.at(str(count) + ":" + str(total), default: none)
  assert(
    final != none,
    message: scope + " cannot pack into " + str(count) + " lines. Add or cut signal until the count fits.",
  )
  let lines = ()
  let cursor = total
  for k in range(count, 0, step: -1) {
    let entry = best.at(str(k) + ":" + str(cursor))
    lines.push(words.slice(entry.prev, cursor).join(" "))
    cursor = entry.prev
  }
  lines.rev()
}

// Render explicit lines as one justified paragraph while measuring their natural widths.
// Manual line breaks and the caller's unbreakable block prevent widows and orphans.
// Non-final lines are justified; the final line stays left-bound: Typst's plain
// `linebreak()` always creates an unjustified break, so each inter-line break
// opts into justification explicitly. The paragraph's closing line carries no
// break and remains ragged by construction.
//
// Callers must place the paragraph in a full-width container (for example a
// `block(width: 100%, breakable: false)`): an auto-width block does not
// expand its children, so the paragraph would shrink-wrap to its widest line
// and justify to that width instead of the column.
#let measured-paragraph(id, kind, lines, justify: true) = layout(size => {
  [
    #for (index, line) in lines.enumerate() {
      line-contract-marker(
        id + "." + str(index + 1),
        kind,
        text(line.text),
        line.min_fill,
        line.target_fill,
        line.max_fill,
        size.width,
        source-text: line.text,
      )
    }
    #set par(justify: justify)
    #for (index, line) in lines.enumerate() {
      // A closing line past 100% carries an approved spill (up to its
      // maximum): fix its box so the spill extends invisibly into the
      // margin. Flowing text would re-wrap it when justification cannot
      // shrink the spaces enough, silently adding a line the metric gate
      // approved. Lines at or below 100% keep flowing text, so green
      // documents render exactly as before.
      let spill = calc.round(1000 * measure(text(line.text)).width / size.width) / 10 > 100
      if index + 1 == lines.len() and spill {
        box(width: measure(text(line.text)).width, text(line.text))
      } else {
        text(line.text)
      }
      if index < lines.len() - 1 {
        linebreak(justify: justify)
      }
    }
  ]
})
