use lopdf::dictionary;
use std::fs;

use tempfile::tempdir;

use super::*;

#[test]
fn exported_locale_metadata_is_lowercase_without_rewriting_content() {
    let mixed = "de-ch".to_ascii_uppercase();
    let mut document = Document::with_version("1.7");
    let indirect = document.add_object(Object::string_literal(mixed.as_str()));
    let xml = format!(
        "<x:xmpmeta><dc:language><rdf:Bag><rdf:li>{mixed}</rdf:li></rdf:Bag></dc:language><dc:title>{mixed}</dc:title></x:xmpmeta>"
    );
    let metadata = document.add_object(lopdf::Stream::new(
        lopdf::dictionary! { "Type" => "Metadata", "Subtype" => "XML" },
        xml.into_bytes(),
    ));
    let prose = document.add_object(lopdf::Stream::new(
        Dictionary::new(),
        mixed.as_bytes().to_vec(),
    ));
    let root = document.add_object(lopdf::dictionary! {
        "Type" => "Catalog",
        "Lang" => indirect,
        "Metadata" => metadata,
        "Contents" => prose,
        "Nested" => lopdf::dictionary! { "Lang" => Object::string_literal(mixed.as_str()) },
    });
    document.trailer.set("Root", root);
    let mut original = Vec::new();
    document.save_to(&mut original).unwrap();

    let normalized = lowercase_locales(&original).unwrap();
    let parsed = Document::load_mem(&normalized).unwrap();
    assert_eq!(
        parsed.get_object(indirect).unwrap().as_str().unwrap(),
        b"de-ch"
    );
    assert_eq!(
        parsed
            .get_dictionary(root)
            .unwrap()
            .get(b"Nested")
            .unwrap()
            .as_dict()
            .unwrap()
            .get(b"Lang")
            .unwrap()
            .as_str()
            .unwrap(),
        b"de-ch"
    );
    let actual_xml = parsed
        .get_object(metadata)
        .unwrap()
        .as_stream()
        .unwrap()
        .get_plain_content()
        .unwrap();
    assert_eq!(
        String::from_utf8(actual_xml).unwrap(),
        format!(
            "<x:xmpmeta><dc:language><rdf:Bag><rdf:li>de-ch</rdf:li></rdf:Bag></dc:language><dc:title>{mixed}</dc:title></x:xmpmeta>"
        )
    );
    assert_eq!(
        parsed
            .get_object(prose)
            .unwrap()
            .as_stream()
            .unwrap()
            .content,
        mixed.as_bytes()
    );
    assert_eq!(lowercase_locales(&normalized).unwrap(), normalized);
}

#[test]
fn missing_pdf_is_rejected() {
    assert!(
        verify(
            Path::new("definitely-missing.pdf"),
            1,
            &[],
            &serde_json::json!({})
        )
        .is_err()
    );
}

#[test]
fn rendition_identifier_is_not_document_content() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let original = root.join("cvl/cv/harvard/standard/de/ch/pdf/cv-2.pdf");
    let original_bytes = fs::read(&original).unwrap();
    let trailer_id = BytesRegex::new(r"(/ID\[\([^)]*\)\()[^)]*(\)\]\s*>>)").unwrap();
    let changed = INSTANCE_ID
        .replace_all(&original_bytes, b"${1}AAAAAAAAAAAAAAAAAAAAAA==${2}")
        .into_owned();
    let changed = trailer_id
        .replace_all(&changed, b"${1}AAAAAAAAAAAAAAAAAAAAAA==${2}")
        .into_owned();
    assert_ne!(original_bytes, changed);

    let directory = tempdir().unwrap();
    let equivalent = directory.path().join("equivalent.pdf");
    fs::write(&equivalent, changed).unwrap();
    assert_eq!(
        semantic_signature(&original).unwrap(),
        semantic_signature(&equivalent).unwrap()
    );
}

#[test]
fn metadata_change_is_detected() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let original = root.join("cvl/cv/harvard/standard/de/ch/pdf/cv-2.pdf");
    let original_bytes = fs::read(&original).unwrap();
    let metadata = BytesRegex::new("<dc:language>").unwrap();
    let changed = metadata
        .replacen(&original_bytes, 1, b"<dc:languagf>")
        .into_owned();
    assert_ne!(original_bytes, changed);

    let directory = tempdir().unwrap();
    let modified = directory.path().join("modified.pdf");
    fs::write(&modified, changed).unwrap();
    assert_ne!(
        semantic_signature(&original).unwrap(),
        semantic_signature(&modified).unwrap()
    );
}
