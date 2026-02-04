use crate::path::FieldPath;
use crate::value::Value;
use crate::{Document, Entity, Format, Record};

use super::{Codec, CodecError};

#[derive(Debug, Clone, Default)]
pub struct TextCodec;

impl TextCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Codec for TextCodec {
    fn format(&self) -> Format {
        Format::Text
    }

    fn decode(&self, record: Record) -> Result<Document, CodecError> {
        if record.media_type.format() != Format::Text {
            return Err(CodecError::UnsupportedMediaType(record.media_type));
        }

        let text = String::from_utf8(record.content)?;
        let entity = Entity::new(
            FieldPath::parse("root").expect("valid field path"),
            record.media_type.as_mime_str(),
            Value::String(text),
        );

        Ok(Document::new(record.path, record.media_type, vec![entity]))
    }

    fn encode(&self, document: Document) -> Result<Record, CodecError> {
        if document.media_type.format() != Format::Text {
            return Err(CodecError::UnsupportedMediaType(document.media_type));
        }

        let content = document
            .content
            .first()
            .ok_or_else(|| CodecError::Encode("document has no content".to_string()))?;

        let text = content
            .content
            .as_str()
            .ok_or_else(|| CodecError::Encode("content is not a string".to_string()))?;

        Ok(Record::from_str(document.path, document.media_type, text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MediaType;
    use crate::path::FilePath;
    use crate::path::Path;

    #[test]
    fn test_decode_text() {
        let codec = TextCodec::new();
        let path = Path::File(FilePath::parse("/test.txt"));
        let record = Record::from_str(path.clone(), MediaType::TextPlain, "hello world");

        let document = codec.decode(record).unwrap();

        assert_eq!(document.media_type, MediaType::TextPlain);
        assert_eq!(document.content[0].content.as_str(), Some("hello world"));
    }

    #[test]
    fn test_encode_text() {
        let codec = TextCodec::new();
        let path = Path::File(FilePath::parse("/test.txt"));
        let entity = Entity::new(
            FieldPath::parse("root").unwrap(),
            "text/plain",
            Value::String("hello world".to_string()),
        );
        let document = Document::new(path.clone(), MediaType::TextPlain, vec![entity]);

        let record = codec.encode(document).unwrap();

        assert_eq!(record.content_str().unwrap(), "hello world");
    }

    #[test]
    fn test_roundtrip() {
        let codec = TextCodec::new();
        let path = Path::File(FilePath::parse("/test.txt"));
        let original = Record::from_str(path, MediaType::TextPlain, "test content");

        let document = codec.decode(original.clone()).unwrap();
        let record = codec.encode(document).unwrap();

        assert_eq!(original.content, record.content);
    }

    #[test]
    fn test_unsupported_media_type() {
        let codec = TextCodec::new();
        let path = Path::File(FilePath::parse("/test.bin"));
        let record = Record::new(path, MediaType::Binary, vec![0, 1, 2, 3]);

        let result = codec.decode(record);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_unsupported());
    }
}
