//! Storage-independent CCVL style contracts and versioned asset bundles.
//! Hosts supply records and bytes; the core never opens a project directory.

pub mod contract;
pub mod edit;
#[cfg(feature = "render")]
pub mod pdf;
#[cfg(feature = "render")]
pub mod render;
pub mod style;

pub use style::{StyleBundle, StyleDocument};

/// Normalize a locale supported by CCVL's language/country style interface.
pub fn normalize_locale(value: &str) -> anyhow::Result<String> {
    use anyhow::{Context, ensure};
    let value = cletter::normalize_locale_id(value);
    let (language, country) = value
        .split_once('-')
        .context("locale must be language-country")?;
    ensure!(
        (2..=3).contains(&language.len())
            && country.len() == 2
            && language.bytes().all(|c| c.is_ascii_lowercase())
            && country.bytes().all(|c| c.is_ascii_lowercase()),
        "unsupported locale: {value}; expected language-country"
    );
    Ok(value)
}
