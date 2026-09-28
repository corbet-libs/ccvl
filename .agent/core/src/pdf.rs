use anyhow::{Context, Result};
use lopdf::{Document, Object, ObjectId};
use regex::bytes::Regex as BytesRegex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

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
            .as_chunks::<2>()
            .0
            .iter()
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
