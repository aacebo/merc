use crate::path::IdentPath;
use crate::value::Value;
use crate::{Document, Entity, Format, Record};

use super::{Codec, CodecError};

#[derive(Debug, Clone)]
pub struct TomlCodec {
    pub pretty_print: bool,
}

impl Default for TomlCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl TomlCodec {
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

impl Codec for TomlCodec {
    fn format(&self) -> Format {
        Format::Toml
    }

    fn decode(&self, record: Record) -> Result<Document, CodecError> {
        if record.media_type.format() != Format::Toml {
            return Err(CodecError::UnsupportedMediaType(record.media_type));
        }

        let text = String::from_utf8(record.content)?;
        let toml_val: toml::Value = toml::from_str(&text).map_err(CodecError::decode)?;
        let value: Value = toml_val.into();

        let entity = Entity::new(
            IdentPath::parse("root").expect("valid field path"),
            record.media_type.as_mime_str(),
            value,
        );

        Ok(Document::new(record.path, record.media_type, vec![entity]))
    }

    fn encode(&self, document: Document) -> Result<Record, CodecError> {
        if document.media_type.format() != Format::Toml {
            return Err(CodecError::UnsupportedMediaType(document.media_type));
        }

        let content = document
            .content
            .first()
            .ok_or_else(|| CodecError::Encode("document has no content".to_string()))?;

        let toml_val: toml::Value = (&content.content).into();
        let text = if self.pretty_print {
            toml::to_string_pretty(&toml_val)
        } else {
            toml::to_string(&toml_val)
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
    fn test_decode_toml() {
        let codec = TomlCodec::new();
        let path = Path::File(FilePath::parse("/test.toml"));
        let record = Record::from_str(
            path.clone(),
            MediaType::TextToml,
            "name = \"test\"\nvalue = 42",
        );

        let document = codec.decode(record).unwrap();

        assert_eq!(document.media_type, MediaType::TextToml);
        assert!(document.content[0].content.is_object());
        assert_eq!(document.content[0].content["name"].as_str(), Some("test"));
        assert_eq!(document.content[0].content["value"].as_int(), Some(42));
    }

    #[test]
    fn test_encode_toml() {
        let codec = TomlCodec::new();
        let path = Path::File(FilePath::parse("/test.toml"));

        let mut obj = Object::new();
        obj.insert("key".to_string(), Value::String("value".to_string()));
        let entity = Entity::new(
            IdentPath::parse("root").unwrap(),
            "application/toml",
            Value::Object(obj),
        );
        let document = Document::new(path.clone(), MediaType::TextToml, vec![entity]);

        let record = codec.encode(document).unwrap();
        let text = record.content_str().unwrap();

        assert!(text.contains("key"));
        assert!(text.contains("value"));
    }

    #[test]
    fn test_roundtrip() {
        let codec = TomlCodec::new();
        let path = Path::File(FilePath::parse("/test.toml"));
        let original = Record::from_str(path, MediaType::TextToml, "test = 123");

        let document = codec.decode(original).unwrap();
        let record = codec.encode(document).unwrap();
        let text = record.content_str().unwrap();

        // Re-decode to verify
        let path2 = Path::File(FilePath::parse("/test.toml"));
        let record2 = Record::from_str(path2, MediaType::TextToml, text);
        let doc2 = codec.decode(record2).unwrap();

        assert_eq!(doc2.content[0].content["test"].as_int(), Some(123));
    }

    #[test]
    fn test_unsupported_media_type() {
        let codec = TomlCodec::new();
        let path = Path::File(FilePath::parse("/test.json"));
        let record = Record::from_str(path, MediaType::TextJson, "{}");

        let result = codec.decode(record);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_unsupported());
    }
}
