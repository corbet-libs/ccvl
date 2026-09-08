// The shared schema validates only adapter-owned tables; other tables belong to the style.
#let schema = json("document-settings.json")
#let nonempty-text(value) = type(value) == str and value.trim() != ""
#let number(value) = type(value) in (int, float) and value > -float.inf and value < float.inf
#let color(value) = (
  type(value) == str and value.match(regex("^#?([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$")) != none
)
#let tag(value) = type(value) == str and value.match(regex("^[!-~]{4}$")) != none
#let tags(value, features) = (
  type(value) == dictionary
    and value
      .pairs()
      .all(pair => {
        tag(pair.first()) and if features { type(pair.last()) in (int, bool) } else { number(pair.last()) }
      })
)
#let field-valid(value, rule) = {
  let valid = if rule.type == "boolean" { type(value) == bool } else if rule.type == "boolean-auto" {
    type(value) == bool or value == "auto"
  } else if rule.type == "number" { number(value) } else if rule.type == "integer" { type(value) == int } else if (
    rule.type == "string"
  ) { nonempty-text(value) } else if rule.type == "color" { color(value) } else if rule.type == "color-auto" {
    color(value) or value == "auto"
  } else if rule.type == "font" {
    nonempty-text(value) or (type(value) == array and value.len() > 0 and value.all(nonempty-text))
  } else if rule.type == "features" { tags(value, true) or (type(value) == array and value.all(tag)) } else if (
    rule.type == "variations"
  ) { tags(value, false) } else { panic("unknown adapter schema type " + rule.type) }
  (
    valid
      and (not ("enum" in rule) or value in rule.enum)
      and (
        not ("minimum" in rule) or value >= rule.minimum
      )
      and (not ("exclusive_minimum" in rule) or value > rule.exclusive_minimum)
      and (
        not ("maximum" in rule) or value <= rule.maximum
      )
  )
}
#let validate-settings(settings) = {
  for (group, rules) in schema {
    assert(group in settings, message: "missing adapter settings " + group)
    let values = settings.at(group)
    assert(type(values) == dictionary, message: group + " must be a table")
    for (key, value) in values {
      let field = group + "." + key
      assert(key in rules, message: "unknown adapter setting " + field)
      assert(field-valid(value, rules.at(key)), message: "invalid " + field + " = " + repr(value))
    }
    for (key, rule) in rules {
      assert(rule.at("optional", default: false) or key in values, message: "missing " + group + "." + key)
    }
  }
  if settings.page.paper == "custom" {
    assert(
      "width_mm" in settings.page and "height_mm" in settings.page,
      message: "custom paper requires page.width_mm and page.height_mm",
    )
  } else {
    assert(
      not ("width_mm" in settings.page) and not ("height_mm" in settings.page),
      message: "page.width_mm/height_mm are unused unless page.paper = custom",
    )
  }
  let q = settings.paragraph
  assert(
    q.word_spacing_min_percent <= q.word_spacing_max_percent,
    message: "paragraph word spacing bounds are reversed",
  )
  assert(q.tracking_min_pt <= q.tracking_max_pt, message: "paragraph tracking bounds are reversed")
}
