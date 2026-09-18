// Application records store calendar values, never preformatted locale text.
#import "letter/letter.typ": long-date, month-year

#let format-application-date(locale, value) = {
  assert(type(value) == str, message: "options.application_date must be a quoted date string")
  if value == "" { return "" }
  assert(locale != "", message: "options.language is required for a nonempty application date")
  assert(
    value.match(regex("^[0-9]{4}-[0-9]{2}(-[0-9]{2})?$")) != none,
    message: "options.application_date: use quoted YYYY-MM-DD or YYYY-MM, or an empty string; localized date text is not accepted",
  )
  let parts = value.split("-").map(int)
  let formatted = if parts.len() == 3 {
    long-date(locale, ..parts)
  } else {
    month-year(locale, ..parts)
  }
  assert(formatted != none, message: "options.application_date: invalid calendar date")
  formatted
}
