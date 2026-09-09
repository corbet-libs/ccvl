use anyhow::{Context, Result, ensure};
use serde_json::Value;

use super::normalize_locale;

pub(super) fn validate_contract(value: &Value) -> Result<()> {
    for key in [
        "content_fields",
        "metric_rules",
        "shared_pages",
        "paragraphs",
        "paragraph_regions",
        "source_files",
    ] {
        if let Some(value) = value.get(key) {
            ensure!(value.is_array(), "{key} must be an array");
        }
    }
    for key in ["content_fields", "source_files"] {
        if let Some(values) = value.get(key).and_then(Value::as_array) {
            ensure!(
                values.iter().all(Value::is_string),
                "{key} must contain strings"
            );
        }
    }
    if let Some(pages) = value.get("shared_pages").and_then(Value::as_array) {
        ensure!(
            pages.iter().all(|p| p.as_u64().is_some_and(|p| p > 0)),
            "shared_pages must contain positive page numbers"
        );
    }
    if let Some(rules) = value.get("metric_rules").and_then(Value::as_array) {
        for rule in rules {
            ensure!(
                rule.get("kind")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| !kind.is_empty()),
                "metric rule must have a kind"
            );
            for key in ["minimum", "maximum"] {
                if let Some(count) = rule.get(key) {
                    ensure!(
                        count.as_u64().is_some(),
                        "metric {key} must be a non-negative count"
                    );
                }
            }
            let minimum = rule.get("minimum").and_then(Value::as_u64).unwrap_or(0);
            ensure!(
                rule.get("maximum")
                    .and_then(Value::as_u64)
                    .is_none_or(|maximum| maximum >= minimum),
                "metric count bounds are reversed"
            );
        }
    }
    if let Some(policy) = value.get("pdf") {
        ensure!(policy.is_object(), "pdf must be a table");
        if let Some(version) = policy.get("version") {
            ensure!(
                version.as_str().is_some_and(|v| !v.is_empty()),
                "PDF version must be non-empty text"
            );
        }
        if let Some(tagged) = policy.get("tagged") {
            ensure!(tagged.is_boolean(), "PDF tagged must be a boolean");
        }
        if let Some(overrides) = policy.get("by_locale") {
            for (locale, policy) in overrides.as_object().context("by_locale must be a table")? {
                ensure!(
                    normalize_locale(locale)? == *locale,
                    "PDF locale must be canonical"
                );
                ensure!(
                    policy.get("by_locale").is_none(),
                    "nested PDF locale overrides are not supported"
                );
                validate_contract(&serde_json::json!({"pdf": policy}))?;
            }
        }
        if let Some(size) = policy.get("size_pt") {
            let size = size.as_array().context("size_pt must be an array")?;
            ensure!(
                size.len() == 2
                    && size
                        .iter()
                        .all(|n| n.as_f64().is_some_and(|n| n.is_finite() && n > 0.0)),
                "size_pt must contain positive width and height"
            );
        }
        if let Some(pattern) = policy.get("font_pattern") {
            regex::Regex::new(pattern.as_str().context("font_pattern must be text")?)?;
        }
        if let Some(required) = policy.get("required_profile_fields") {
            ensure!(
                required
                    .as_array()
                    .is_some_and(|fields| fields.iter().all(Value::is_string)),
                "required_profile_fields must contain field names"
            );
        }
        if let Some(required) = policy.get("require_image") {
            ensure!(required.is_boolean(), "require_image must be boolean");
        }
        if let Some(minimum) = policy.get("minimum_text_chars") {
            ensure!(
                minimum.as_u64().is_some_and(|n| n > 0),
                "minimum_text_chars must be positive"
            );
        }
    }
    Ok(())
}
