use anyhow::{Context, Result, ensure};
use regex::Regex;

/// Store dates independently of language; cdate owns the displayed convention.
/// Empty dates stay undated, and month-only values never acquire an inferred day.
pub fn format_application_date(locale: &str, value: &str) -> Result<String> {
    if value.is_empty() {
        return Ok(String::new());
    }
    ensure!(
        !locale.is_empty(),
        "options.language is required for a nonempty application date"
    );
    let parts = Regex::new(r"^([0-9]{4})-([0-9]{2})(?:-([0-9]{2}))?$")?
        .captures(value)
        .context("options.application_date: use quoted YYYY-MM-DD or YYYY-MM, or an empty string; localized date text is not accepted")?;
    let year = parts[1].parse::<i32>()?;
    let month = parts[2].parse::<u32>()?;
    let formatted = if let Some(day) = parts.get(3) {
        cletter::long_date(locale, year, month, day.as_str().parse::<u32>()?)
    } else {
        cletter::month_year(locale, year, month)
    };
    formatted.context("options.application_date: invalid calendar date")
}

#[cfg(test)]
mod tests {
    use super::format_application_date;

    #[test]
    fn application_date_vectors() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/application-dates.json"
        ))
        .unwrap();
        for case in cases.as_array().unwrap() {
            let locale = case["locale"].as_str().unwrap();
            let input = case["input"].as_str().unwrap();
            let actual = format_application_date(locale, input);
            if let Some(expected) = case["expected"].as_str() {
                assert_eq!(actual.unwrap(), expected, "{locale} {input}");
            } else {
                assert!(actual.is_err(), "unexpected valid date: {locale} {input}");
            }
        }
    }
}
