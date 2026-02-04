use saphyr::{Yaml, YamlEmitter};

use crate::path::FieldPath;
use crate::value::Value;
use crate::{Document, Entity, Format, Record};

use super::{Codec, CodecError};

#[derive(Debug, Clone, Default)]
pub struct YamlCodec;

impl YamlCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Codec for YamlCodec {
    fn format(&self) -> Format {
        Format::Yaml
    }

    fn decode(&self, record: Record) -> Result<Document, CodecError> {
        if record.media_type.format() != Format::Yaml {
            return Err(CodecError::UnsupportedMediaType(record.media_type));
        }

        let text = String::from_utf8(record.content)?;
        let docs = Yaml::load_from_str(&text).map_err(|e| CodecError::Decode(e.to_string()))?;
        let yaml = docs.into_iter().next().unwrap_or(Yaml::Null);
        let value = Value::from(yaml);

        let entity = Entity::new(
            FieldPath::parse("root").expect("valid field path"),
            record.media_type.as_mime_str(),
            value,
        );

        Ok(Document::new(record.path, record.media_type, vec![entity]))
    }

    fn encode(&self, document: Document) -> Result<Record, CodecError> {
        if document.media_type.format() != Format::Yaml {
            return Err(CodecError::UnsupportedMediaType(document.media_type));
        }

        let content = document
            .content
            .first()
            .ok_or_else(|| CodecError::Encode("document has no content".to_string()))?;

        let yaml = Yaml::from(&content.content);
        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter
            .dump(&yaml)
            .map_err(|e| CodecError::Encode(e.to_string()))?;

        Ok(Record::from_str(
            document.path,
            document.media_type,
            &out_str,
        ))
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
    fn test_decode_yaml() {
        let codec = YamlCodec::new();
        let path = Path::File(FilePath::parse("/test.yaml"));
        let record = Record::from_str(path.clone(), MediaType::TextYaml, "name: test\nvalue: 42");

        let document = codec.decode(record).unwrap();

        assert_eq!(document.media_type, MediaType::TextYaml);
        assert!(document.content[0].content.is_object());
        assert_eq!(document.content[0].content["name"].as_str(), Some("test"));
        assert_eq!(document.content[0].content["value"].as_int(), Some(42));
    }

    #[test]
    fn test_encode_yaml() {
        let codec = YamlCodec::new();
        let path = Path::File(FilePath::parse("/test.yaml"));

        let mut obj = Object::new();
        obj.insert("key".to_string(), Value::String("value".to_string()));
        let entity = Entity::new(
            FieldPath::parse("root").unwrap(),
            "application/yaml",
            Value::Object(obj),
        );
        let document = Document::new(path.clone(), MediaType::TextYaml, vec![entity]);

        let record = codec.encode(document).unwrap();
        let text = record.content_str().unwrap();

        assert!(text.contains("key"));
        assert!(text.contains("value"));
    }

    #[test]
    fn test_roundtrip() {
        let codec = YamlCodec::new();
        let path = Path::File(FilePath::parse("/test.yaml"));
        let original = Record::from_str(path, MediaType::TextYaml, "test: 123");

        let document = codec.decode(original).unwrap();
        let record = codec.encode(document).unwrap();
        let text = record.content_str().unwrap();

        // Re-decode to verify
        let path2 = Path::File(FilePath::parse("/test.yaml"));
        let record2 = Record::from_str(path2, MediaType::TextYaml, text);
        let doc2 = codec.decode(record2).unwrap();

        assert_eq!(doc2.content[0].content["test"].as_int(), Some(123));
    }

    #[test]
    fn test_unsupported_media_type() {
        let codec = YamlCodec::new();
        let path = Path::File(FilePath::parse("/test.json"));
        let record = Record::from_str(path, MediaType::TextJson, "{}");

        let result = codec.decode(record);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_unsupported());
    }
}
