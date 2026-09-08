use std::fs;

use tempfile::tempdir;

use super::*;

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
