//! The same bounded fields are offered to manual editors and chat tools.
use std::borrow::Cow;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};

use crate::StyleDocument;

#[derive(Clone, Debug, Serialize)]
pub struct EditableField {
    /// JSON pointer, prefixed with the input name: `/record/cl/body`.
    pub key: String,
    pub label: String,
    pub structured: bool,
}

impl StyleDocument {
    /// Content shape comes from the selected style and supplied records.
    /// Arrays and objects are edited as JSON, so styles can add or remove entries.
    #[must_use]
    pub fn editable_fields(&self) -> Vec<EditableField> {
        let mut fields = Vec::new();
        for (input, section, value) in [
            (
                "record",
                self.bundle.document.as_str(),
                self.record.get(&self.bundle.document),
            ),
            ("record", "job", self.record.get("job")),
            ("profile", "", Some(&self.profile)),
        ] {
            if let Some(object) = value.and_then(Value::as_object) {
                for (name, value) in object {
                    if input == "record" && section == "job" && name == "id" {
                        continue;
                    }
                    let escaped = name.replace('~', "~0").replace('/', "~1");
                    let key = if section.is_empty() {
                        format!("/{input}/{escaped}")
                    } else {
                        format!("/{input}/{section}/{escaped}")
                    };
                    fields.push(EditableField {
                        key,
                        label: if section.is_empty() {
                            format!("Profile · {name}")
                        } else {
                            format!("{section} · {name}")
                        },
                        structured: !value.is_string(),
                    });
                }
            }
        }
        fields
    }

    pub fn field_value(&self, key: &str) -> Result<Cow<'_, str>> {
        let value = self.input_value(key).context("unknown style field")?;
        Ok(match value.as_str() {
            Some(text) => Cow::Borrowed(text),
            None => Cow::Owned(serde_json::to_string_pretty(value)?),
        })
    }

    /// Reject unknown fields, invalid JSON and type changes before changing an input.
    pub fn set_field(&mut self, key: &str, text: &str) -> Result<()> {
        let field = self
            .editable_fields()
            .into_iter()
            .find(|field| field.key == key)
            .context("field is not editable in this style document")?;
        let previous = self.input_value(key).context("unknown style field")?;
        let value = if field.structured {
            let value: Value =
                serde_json::from_str(text).context("enter valid JSON for this structured field")?;
            ensure!(
                std::mem::discriminant(previous) == std::mem::discriminant(&value),
                "replacement must preserve the field's JSON type"
            );
            value
        } else {
            Value::String(text.to_owned())
        };
        let (input, pointer) = split_key(key).context("invalid style field")?;
        let root = if input == "record" {
            &mut self.record
        } else {
            &mut self.profile
        };
        *root.pointer_mut(pointer).context("unknown style field")? = value;
        Ok(())
    }

    /// Prompt projection deliberately omits binary assets and template source.
    #[must_use]
    pub fn editing_context(&self) -> Value {
        json!({"style": {"id": self.bundle.id, "version": self.bundle.version,
            "sha256": self.bundle_sha256, "contract": self.bundle.contract},
            "record": self.record, "profile": self.profile, "editableFields": self.editable_fields()})
    }

    fn input_value(&self, key: &str) -> Option<&Value> {
        let (input, pointer) = split_key(key)?;
        match input {
            "record" => self.record.pointer(pointer),
            "profile" => self.profile.pointer(pointer),
            _ => None,
        }
    }
}

fn split_key(key: &str) -> Option<(&str, &str)> {
    let key = key.strip_prefix('/')?;
    let slash = key.find('/')?;
    Some((&key[..slash], &key[slash..]))
}
