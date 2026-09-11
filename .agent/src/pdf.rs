use std::collections::BTreeMap;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{Context, Result, ensure};
use lopdf::{Dictionary, Document, Object, ObjectId};
use regex::{Regex, bytes::Regex as BytesRegex};
use sha2::{Digest, Sha256};

static INSTANCE_ID: LazyLock<BytesRegex> = LazyLock::new(|| {
    BytesRegex::new(r"(<xmpMM:InstanceID>)[^<]*(</xmpMM:InstanceID>)")
        .expect("the rendition identifier pattern is valid")
});

static LANGUAGE_BAG: LazyLock<BytesRegex> = LazyLock::new(|| {
    BytesRegex::new(r"(?s)(<dc:language>)(.*?)(</dc:language>)")
        .expect("the language metadata pattern is valid")
});
static LANGUAGE_ITEM: LazyLock<BytesRegex> = LazyLock::new(|| {
    BytesRegex::new(r"(<rdf:li>)([A-Za-z0-9-]+)(</rdf:li>)")
        .expect("the language item pattern is valid")
});

/// Keep exported locale identifiers consistent with workspace identifiers.
/// Typst emits region subtags in uppercase. Change only PDF language entries
/// and the XMP language list; names, prose, links and other metadata survive.
pub fn lowercase_locales(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut document = Document::load_mem(bytes).context("cannot parse exported PDF")?;
    // A language string may also be referenced by protected metadata. Resolve
    // strings from this snapshot and replace only the language reference.
    let indirect_strings = document
        .objects
        .iter()
        .filter(|(_, object)| matches!(object, Object::String(..)))
        .map(|(id, object)| (*id, object.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut changed = false;
    for object in document.objects.values_mut() {
        changed |= lowercase_language_entries(object, &indirect_strings);
        if let Object::Stream(stream) = object
            && stream.dict.get(b"Type").and_then(Object::as_name).ok() == Some(b"Metadata")
            && stream.dict.get(b"Subtype").and_then(Object::as_name).ok() == Some(b"XML")
        {
            let xml = stream.get_plain_content()?;
            let normalized = LANGUAGE_BAG.replace_all(&xml, |bag: &regex::bytes::Captures<'_>| {
                let items =
                    LANGUAGE_ITEM.replace_all(&bag[2], |item: &regex::bytes::Captures<'_>| {
                        let locale = std::str::from_utf8(&item[2])
                            .expect("the language item pattern matches only ASCII");
                        [
                            item[1].to_vec(),
                            cletter::normalize_locale_id(locale).into_bytes(),
                            item[3].to_vec(),
                        ]
                        .concat()
                    });
                [&bag[1], items.as_ref(), &bag[3]].concat()
            });
            if normalized.as_ref() != xml {
                changed = true;
                stream.dict.remove(b"Filter");
                stream.dict.remove(b"DecodeParms");
                stream.set_content(normalized.into_owned());
            }
        }
    }
    if !changed {
        return Ok(bytes.to_vec());
    }
    let mut output = Vec::new();
    document.save_to(&mut output)?;
    Ok(output)
}

fn lowercase_language_entries(
    object: &mut Object,
    indirect_strings: &BTreeMap<ObjectId, Object>,
) -> bool {
    let mut changed = false;
    match object {
        Object::Array(items) => {
            for item in items {
                changed |= lowercase_language_entries(item, indirect_strings);
            }
        }
        Object::Dictionary(dictionary)
        | Object::Stream(lopdf::Stream {
            dict: dictionary, ..
        }) => {
            for (key, value) in dictionary.iter_mut() {
                if key == b"Lang" {
                    if let Object::Reference(reference) = value {
                        if let Some(target) = indirect_strings.get(reference) {
                            let mut normalized = target.clone();
                            if lowercase_language_string(&mut normalized) {
                                *value = normalized;
                                changed = true;
                            }
                        }
                    } else {
                        changed |= lowercase_language_string(value);
                    }
                } else {
                    changed |= lowercase_language_entries(value, indirect_strings);
                }
            }
        }
        _ => {}
    }
    changed
}

fn lowercase_language_string(object: &mut Object) -> bool {
    let Object::String(value, _) = object else {
        return false;
    };
    let normalized = if value.starts_with(&[0xfe, 0xff]) || value.starts_with(&[0xff, 0xfe]) {
        let big_endian = value[0] == 0xfe;
        let payload = &value[2..];
        if !payload.len().is_multiple_of(2) {
            return false;
        }
        let units = payload
            .chunks_exact(2)
            .map(|pair| {
                if big_endian {
                    u16::from_be_bytes([pair[0], pair[1]])
                } else {
                    u16::from_le_bytes([pair[0], pair[1]])
                }
            })
            .collect::<Vec<_>>();
        let Ok(locale) = String::from_utf16(&units) else {
            return false;
        };
        let mut encoded = value[..2].to_vec();
        encoded.extend(
            cletter::normalize_locale_id(&locale)
                .encode_utf16()
                .flat_map(|unit| {
                    if big_endian {
                        unit.to_be_bytes()
                    } else {
                        unit.to_le_bytes()
                    }
                }),
        );
        encoded
    } else {
        // Locale text is ASCII. Preserve unknown encodings, including BOMless
        // UTF-16 and non-ASCII PDFDocEncoding/UTF-8 strings, without guessing.
        if !value.iter().all(|byte| {
            byte.is_ascii_graphic()
                || matches!(*byte, b' ' | b'\t' | b'\n' | b'\r' | b'\x0b' | b'\x0c')
        }) {
            return false;
        }
        cletter::normalize_locale_id(
            std::str::from_utf8(value).expect("the language bytes were checked as ASCII"),
        )
        .into_bytes()
    };
    if normalized == *value {
        return false;
    }
    *value = normalized;
    true
}

pub struct VerifiedPdf {
    document: Document,
}

impl VerifiedPdf {
    pub fn page_content(&self, page: u32) -> Result<Vec<u8>> {
        let id = self
            .document
            .get_pages()
            .get(&page)
            .copied()
            .context("PDF page is missing")?;
        Ok(self.document.get_page_content(id))
    }
}

pub fn verify(
    path: &Path,
    expected_pages: usize,
    contacts: &[String],
    policy: &serde_json::Value,
) -> Result<VerifiedPdf> {
    let document =
        Document::load(path).with_context(|| format!("cannot parse {}", path.display()))?;
    if let Some(version) = policy.get("version").and_then(serde_json::Value::as_str) {
        ensure!(
            document.version == version,
            "{}: expected PDF {version}",
            path.display()
        );
    }
    ensure!(
        !document.trailer.has(b"Encrypt"),
        "{} is encrypted",
        path.display()
    );
    let pages = document.get_pages();
    ensure!(
        pages.len() == expected_pages,
        "{} rendered {} pages; expected {expected_pages}",
        path.display(),
        pages.len()
    );
    let catalog_id = document.trailer.get(b"Root")?.as_reference()?;
    let catalog = document.get_dictionary(catalog_id)?;
    if let Some(tagged) = policy.get("tagged").and_then(serde_json::Value::as_bool) {
        ensure!(
            catalog.has(b"StructTreeRoot") == tagged,
            "{}: PDF tagging differs from its contract",
            path.display()
        );
    }
    for key in [b"AcroForm".as_slice(), b"OpenAction", b"AA"] {
        ensure!(
            !catalog.has(key),
            "{} contains forbidden catalog entry /{}",
            path.display(),
            String::from_utf8_lossy(key)
        );
    }
    if let Some(names) = get_dict(&document, catalog, b"Names") {
        ensure!(
            !names.has(b"JavaScript") && !names.has(b"EmbeddedFiles"),
            "{} contains JavaScript or embedded files",
            path.display()
        );
    }

    let font_pattern = policy
        .get("font_pattern")
        .and_then(serde_json::Value::as_str)
        .map(Regex::new)
        .transpose()?;
    let mut fonts_seen = 0;
    for (page_number, page_id) in &pages {
        let page = document.get_dictionary(*page_id)?;
        ensure!(
            !page.has(b"AA"),
            "{} page {page_number} contains an automatic action",
            path.display()
        );
        let media = inherited(&document, *page_id, b"MediaBox")
            .context("PDF page has no MediaBox")?
            .as_array()?;
        ensure!(
            media.len() == 4,
            "{} page {page_number} has an invalid MediaBox",
            path.display()
        );
        let width = media[2].as_float()? - media[0].as_float()?;
        let height = media[3].as_float()? - media[1].as_float()?;
        ensure!(
            width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0,
            "{} page {page_number} has invalid dimensions",
            path.display()
        );
        if let Some(size) = policy.get("size_pt").and_then(serde_json::Value::as_array) {
            ensure!(size.len() == 2, "PDF size_pt must contain width and height");
            let expected_width = size[0].as_f64().context("invalid PDF width")?;
            let expected_height = size[1].as_f64().context("invalid PDF height")?;
            ensure!(
                (f64::from(width) - expected_width).abs() <= 0.2
                    && (f64::from(height) - expected_height).abs() <= 0.2,
                "{} page {page_number} does not match its style dimensions ({width}×{height})",
                path.display()
            );
        }
        for font in document.get_page_fonts(*page_id)?.values() {
            fonts_seen += 1;
            let base = font.get(b"BaseFont")?.as_name()?;
            let base = String::from_utf8_lossy(base);
            ensure!(
                font_pattern
                    .as_ref()
                    .is_none_or(|pattern| pattern.is_match(&base)),
                "{} contains a fallback or unsubsetted font: {base}",
                path.display()
            );
            ensure!(
                font.has(b"ToUnicode"),
                "{} contains a font without a Unicode map: {base}",
                path.display()
            );
        }
    }
    ensure!(fonts_seen > 0, "{} contains no fonts", path.display());
    let has_embedded_font = document.objects.values().any(|object| {
        object_dictionary(object).is_some_and(|dictionary| {
            dictionary.has(b"FontFile")
                || dictionary.has(b"FontFile2")
                || dictionary.has(b"FontFile3")
        })
    });
    ensure!(
        has_embedded_font,
        "{} contains no embedded font program",
        path.display()
    );
    if policy
        .get("require_image")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        let has_image = document.objects.values().any(|object| {
            object_dictionary(object)
                .and_then(|dictionary| dictionary.get(b"Subtype").ok())
                .and_then(|value| value.as_name().ok())
                == Some(b"Image")
        });
        ensure!(
            has_image,
            "{} is missing its rendered signature image",
            path.display()
        );
    }
    let page_numbers = pages.keys().copied().collect::<Vec<_>>();
    let text = document
        .extract_text(&page_numbers)
        .context("PDF text extraction failed")?;
    ensure!(
        text.chars().filter(|item| !item.is_whitespace()).count()
            >= usize::try_from(
                policy
                    .get("minimum_text_chars")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(1)
            )?,
        "{} has no usable text layer",
        path.display()
    );
    let normalized_text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for contact in contacts {
        let normalized_contact = contact.split_whitespace().collect::<Vec<_>>().join(" ");
        ensure!(
            normalized_text.contains(&normalized_contact),
            "{} is missing machine-readable contact text: {contact}",
            path.display()
        );
    }
    Ok(VerifiedPdf { document })
}

pub fn semantic_signature(path: &Path) -> Result<Vec<u8>> {
    let document =
        Document::load(path).with_context(|| format!("cannot parse {}", path.display()))?;
    let mut digest = Sha256::new();
    digest.update(b"ccvl-semantic-pdf-v1");
    if let Ok(identifiers) = document.trailer.get(b"ID").and_then(Object::as_array)
        && let Some(document_identifier) = identifiers.first()
    {
        hash_object(&mut digest, document_identifier)?;
    }
    for ((object_number, generation), object) in &document.objects {
        digest.update(object_number.to_be_bytes());
        digest.update(generation.to_be_bytes());
        hash_object(&mut digest, object)?;
    }
    Ok(digest.finalize().to_vec())
}

fn hash_object(digest: &mut Sha256, object: &Object) -> Result<()> {
    match object {
        Object::Null => digest.update(b"null"),
        Object::Boolean(value) => {
            digest.update(b"boolean");
            digest.update([u8::from(*value)]);
        }
        Object::Integer(value) => {
            digest.update(b"integer");
            digest.update(value.to_be_bytes());
        }
        Object::Real(value) => {
            digest.update(b"real");
            digest.update(value.to_bits().to_be_bytes());
        }
        Object::Name(value) => {
            digest.update(b"name");
            hash_bytes(digest, value);
        }
        Object::String(value, _) => {
            digest.update(b"string");
            hash_bytes(digest, value);
        }
        Object::Array(values) => {
            digest.update(b"array");
            digest.update((values.len() as u64).to_be_bytes());
            for value in values {
                hash_object(digest, value)?;
            }
        }
        Object::Dictionary(dictionary) => {
            digest.update(b"dictionary");
            hash_dictionary(digest, dictionary, false)?;
        }
        Object::Stream(stream) => {
            digest.update(b"stream");
            hash_dictionary(digest, &stream.dict, true)?;
            let content = stream
                .get_plain_content()
                .context("cannot decode a PDF stream for semantic comparison")?;
            let normalized =
                INSTANCE_ID.replace_all(&content, b"${1}CCVL-DETERMINISTIC-INSTANCE${2}");
            hash_bytes(digest, &normalized);
        }
        Object::Reference((object_number, generation)) => {
            digest.update(b"reference");
            digest.update(object_number.to_be_bytes());
            digest.update(generation.to_be_bytes());
        }
    }
    Ok(())
}

fn hash_dictionary(digest: &mut Sha256, dictionary: &Dictionary, stream: bool) -> Result<()> {
    let mut entries = dictionary
        .iter()
        .filter(|(key, _)| {
            !stream
                || ![
                    b"DecodeParms".as_slice(),
                    b"Filter".as_slice(),
                    b"Length".as_slice(),
                ]
                .contains(&key.as_slice())
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|(key, _)| *key);
    digest.update((entries.len() as u64).to_be_bytes());
    for (key, value) in entries {
        hash_bytes(digest, key);
        hash_object(digest, value)?;
    }
    Ok(())
}

fn hash_bytes(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn object_dictionary(object: &Object) -> Option<&Dictionary> {
    match object {
        Object::Dictionary(dictionary) => Some(dictionary),
        Object::Stream(stream) => Some(&stream.dict),
        _ => None,
    }
}

fn get_dict<'a>(
    document: &'a Document,
    parent: &'a Dictionary,
    key: &[u8],
) -> Option<&'a Dictionary> {
    match parent.get(key).ok()? {
        Object::Dictionary(dictionary) => Some(dictionary),
        Object::Reference(id) => document.get_dictionary(*id).ok(),
        _ => None,
    }
}

fn inherited<'a>(document: &'a Document, mut id: ObjectId, key: &[u8]) -> Option<&'a Object> {
    loop {
        let dictionary = document.get_dictionary(id).ok()?;
        if let Ok(value) = dictionary.get(key) {
            return Some(value);
        }
        id = dictionary.get(b"Parent").ok()?.as_reference().ok()?;
    }
}

#[cfg(test)]
mod tests;
