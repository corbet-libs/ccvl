// Named paper presets are opt-in and owned entirely by the selected style.
#import "document.typ": merge-settings
#import "document-settings.typ": number, validate-settings

#let resolve-paper(metadata, locale, requested: "", recorded: none) = {
  let registry = metadata.at("paper", default: none)
  assert(type(registry) == dictionary, message: "style does not declare paper presets")
  assert(registry.keys().all(key => key in ("defaults", "sizes")), message: "unknown paper registry field")
  let defaults = registry.at("defaults", default: none)
  let sizes = registry.at("sizes", default: none)
  assert(type(defaults) == dictionary, message: "paper.defaults must be a table")
  assert(type(sizes) == dictionary and sizes.len() > 0, message: "paper.sizes must be a nonempty table")
  assert(
    defaults.keys().sorted() == metadata.supports_locales.sorted(),
    message: "paper defaults must cover exactly the supported locales",
  )
  for (language, id) in defaults {
    assert(type(id) == str and id in sizes, message: "unsupported default paper for " + language)
  }
  for (id, preset) in sizes {
    assert(
      id.match(regex("^[a-z0-9_-]+$")) != none,
      message: "paper IDs must contain only lowercase letters, digits, hyphens or underscores",
    )
    assert(type(preset) == dictionary, message: "paper preset must be a table: " + id)
    assert(
      preset.keys().all(key => key in ("label", "size_pt", "settings")),
      message: "unknown paper preset field: " + id,
    )
    let label = preset.at("label", default: none)
    let size = preset.at("size_pt", default: none)
    assert(type(label) == str and label.trim() != "", message: "paper label must be nonempty: " + id)
    assert(
      type(size) == array and size.len() == 2 and size.all(value => number(value) and value > 0),
      message: "paper size_pt must contain two finite positive dimensions: " + id,
    )
    assert(type(preset.at("settings", default: none)) == dictionary, message: "paper settings must be a table: " + id)
  }
  assert(type(requested) == str, message: "paper input must be text")
  assert(recorded == none or type(recorded) == str, message: "record paper selection must be text")
  let id = if requested != "" { requested } else if recorded != none { recorded } else {
    defaults.at(locale, default: none)
  }
  assert(
    type(id) == str and id in sizes,
    message: "unsupported paper " + repr(id) + "; expected one of " + sizes.keys().join(", "),
  )
  (id: id, ..sizes.at(id))
}

// Validate each supplied layer before merging so later overrides cannot hide
// invalid settings. Complete validation runs after the selected preset overlay.
#let paper-settings(layers, preset) = {
  let settings = (:)
  for layer in layers {
    validate-settings(layer, partial: true)
    for key in ("paper", "width_mm", "height_mm") {
      assert(
        not (key in layer.at("page", default: (:))),
        message: "page." + key + " belongs in the selected paper preset",
      )
    }
    settings = merge-settings(settings, layer)
  }
  validate-settings(preset.settings, partial: true)
  settings = merge-settings(settings, preset.settings)
  validate-settings(settings)
  settings
}
