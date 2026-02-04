use crate::path::FieldPath;
use crate::value::Value;
use crate::{Document, Entity, Format, Record};

use super::{Codec, CodecError};

#[derive(Debug, Clone)]
pub struct JsonCodec {
    pub pretty_print: bool,
}

impl Default for JsonCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonCodec {
    pub fn new() -> Self {
        Self {
            pretty_print: false,
        }
    }

    pub fn pretty() -> Self {
        Self { pretty_print: true }
    }

    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }
}

impl Codec for JsonCodec {
    fn format(&self) -> Format {
        Format::Json
    }

    fn decode(&self, record: Record) -> Result<Document, CodecError> {
        if record.media_type.format() != Format::Json {
            return Err(CodecError::UnsupportedMediaType(record.media_type));
        }

        let text = String::from_utf8(record.content)?;
        let json: serde_json::Value = serde_json::from_str(&text).map_err(CodecError::decode)?;
        let value: Value = json.into();

        let entity = Entity::new(
            FieldPath::parse("root").expect("valid field path"),
            record.media_type.as_mime_str(),
            value,
        );

        Ok(Document::new(record.path, record.media_type, vec![entity]))
    }

    fn encode(&self, document: Document) -> Result<Record, CodecError> {
        if document.media_type.format() != Format::Json {
            return Err(CodecError::UnsupportedMediaType(document.media_type));
        }

        let content = document
            .content
            .first()
            .ok_or_else(|| CodecError::Encode("document has no content".to_string()))?;

        let json: serde_json::Value = (&content.content).into();
        let text = if self.pretty_print {
            serde_json::to_string_pretty(&json)
        } else {
            serde_json::to_string(&json)
        }
        .map_err(CodecError::encode)?;

        Ok(Record::from_str(document.path, document.media_type, &text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MediaType;
    use crate::path::FilePath;
    use crate::path::Path;
    use crate::value::Object;

    #[test]
    fn test_decode_json() {
        let codec = JsonCodec::new();
        let path = Path::File(FilePath::parse("/test.json"));
        let record = Record::from_str(
            path.clone(),
            MediaType::TextJson,
            r#"{"name": "test", "value": 42}"#,
        );

        let document = codec.decode(record).unwrap();

        assert_eq!(document.media_type, MediaType::TextJson);
        assert!(document.content[0].content.is_object());
        assert_eq!(document.content[0].content["name"].as_str(), Some("test"));
        assert_eq!(document.content[0].content["value"].as_int(), Some(42));
    }

    #[test]
    fn test_encode_json() {
        let codec = JsonCodec::new();
        let path = Path::File(FilePath::parse("/test.json"));

        let mut obj = Object::new();
        obj.insert("key".to_string(), Value::String("value".to_string()));
        let entity = Entity::new(
            FieldPath::parse("root").unwrap(),
            "application/json",
            Value::Object(obj),
        );
        let document = Document::new(path.clone(), MediaType::TextJson, vec![entity]);

        let record = codec.encode(document).unwrap();
        let text = record.content_str().unwrap();

        assert!(text.contains("\"key\""));
        assert!(text.contains("\"value\""));
    }

    #[test]
    fn test_roundtrip() {
        let codec = JsonCodec::new();
        let path = Path::File(FilePath::parse("/test.json"));
        let original = Record::from_str(path, MediaType::TextJson, r#"{"test":123}"#);

        let document = codec.decode(original.clone()).unwrap();
        let record = codec.encode(document).unwrap();

        // Parse both as JSON to compare (formatting may differ)
        let orig_json: serde_json::Value = serde_json::from_slice(&original.content).unwrap();
        let round_json: serde_json::Value = serde_json::from_slice(&record.content).unwrap();
        assert_eq!(orig_json, round_json);
    }

    #[test]
    fn test_pretty_print() {
        let codec = JsonCodec::pretty();
        let path = Path::File(FilePath::parse("/test.json"));

        let mut obj = Object::new();
        obj.insert("key".to_string(), Value::String("value".to_string()));
        let entity = Entity::new(
            FieldPath::parse("root").unwrap(),
            "application/json",
            Value::Object(obj),
        );
        let document = Document::new(path.clone(), MediaType::TextJson, vec![entity]);

        let record = codec.encode(document).unwrap();
        let text = record.content_str().unwrap();

        // Pretty printed JSON has newlines
        assert!(text.contains('\n'));
    }

    #[test]
    fn test_unsupported_media_type() {
        let codec = JsonCodec::new();
        let path = Path::File(FilePath::parse("/test.txt"));
        let record = Record::from_str(path, MediaType::TextPlain, "not json");

        let result = codec.decode(record);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_unsupported());
    }
}
