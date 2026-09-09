// A source reference shares wording only between substyles of one document/style/locale.
// Application envelopes and private opportunity records remain self-contained.
#let merge-wording(base, overrides) = {
  let result = base
  for (key, value) in overrides {
    result.insert(key, if type(value) == dictionary and type(result.at(key, default: none)) == dictionary {
      merge-wording(result.at(key), value)
    } else { value })
  }
  result
}
#let load-application(path) = {
  let record = toml(path)
  if not ("wording" in record) { return record }
  assert(
    type(record.wording) == dictionary and record.wording.keys() == ("source",),
    message: "wording must contain only source",
  )
  let parts = path.split("/")
  assert(
    parts.len() >= 6 and parts.last() == "content.toml" and not path.starts-with("/opportunities/"),
    message: "wording references are allowed only in registered showcase leaves; private opportunities must be self-contained",
  )
  let language = parts.at(-3)
  let country = parts.at(-2)
  let style-root = parts.slice(0, parts.len() - 4).join("/")
  let expected = "../../../content/" + language + "/" + country + "/wording.toml"
  assert(
    record.wording.source == expected,
    message: "wording.source must be " + repr(expected) + "; sharing is scoped to the same document, style and locale",
  )
  let shared = toml(style-root + "/content/" + language + "/" + country + "/wording.toml")
  assert(
    shared.keys() == ("cv",) or shared.keys() == ("cl",),
    message: "shared wording must contain only [cv] or [cl]; nested sources and cross-document content are forbidden",
  )
  let document = shared.keys().first()
  let manifest = json("/ccvl.json")
  let key = if document == "cv" { "cv" } else { "cover_letter" }
  let root = "/" + manifest.documents.at(key).root.trim("/") + "/"
  assert(path.starts-with(root), message: "shared wording table must match its owning document root")
  let relative = path.slice(root.len()).split("/")
  assert(relative.len() == 5, message: "wording references require a registered document/style/substyle/locale leaf")
  let (style, substyle, _, _, _) = relative
  let definition = toml(root + style + "/style.toml")
  assert(
    definition.id == style and definition.documents == (document,),
    message: "wording style definition does not match its owner",
  )
  assert(substyle in definition.substyles, message: "wording references require a registered substyle leaf")
  assert(
    language + "-" + country in definition.supports_locales,
    message: "wording references require a registered locale leaf",
  )
  let base = shared.at(document)
  let overrides = record.at(document, default: (:))
  assert(
    type(base) == dictionary and type(overrides) == dictionary,
    message: "shared wording and leaf overrides must be document tables",
  )
  record.insert(document, merge-wording(base, overrides))
  let _ = record.remove("wording")
  record
}
