use crate::{ETag, Id, MediaType, path::Path};

#[derive(Debug, Clone, Hash, serde::Deserialize, serde::Serialize)]
pub struct Record {
    pub id: Id,
    pub etag: ETag,
    pub path: Path,
    pub size: usize,
    pub media_type: MediaType,
    pub content: Vec<u8>,
}

impl Record {
    pub fn new(path: Path, media_type: MediaType, content: Vec<u8>) -> Self {
        Self {
            id: Id::new(path.to_string().as_str()),
            etag: ETag::from_bytes(media_type, &content),
            size: content.len(),
            path,
            media_type,
            content,
        }
    }

    pub fn from_str(path: Path, media_type: MediaType, content: &str) -> Self {
        Self::new(path, media_type, content.as_bytes().to_vec())
    }

    pub fn content_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.content)
    }
}

impl Eq for Record {}
impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id) && self.etag.eq(&other.etag)
    }
}

#[cfg(feature = "json")]
impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).expect("should serialize")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::FilePath;

    #[test]
    fn test_record_new() {
        let path = Path::File(FilePath::parse("/test/file.json"));
        let content = b"{\"key\": \"value\"}".to_vec();
        let record = Record::new(path.clone(), MediaType::TextJson, content.clone());

        assert_eq!(record.path, path);
        assert_eq!(record.media_type, MediaType::TextJson);
        assert_eq!(record.content, content);
        assert_eq!(record.size, content.len());
    }

    #[test]
    fn test_record_from_str() {
        let path = Path::File(FilePath::parse("/test/file.txt"));
        let record = Record::from_str(path.clone(), MediaType::TextPlain, "hello world");

        assert_eq!(record.content_str().unwrap(), "hello world");
    }

    #[test]
    fn test_record_equality() {
        let path = Path::File(FilePath::parse("/test/file.txt"));
        let record1 = Record::from_str(path.clone(), MediaType::TextPlain, "hello");
        let record2 = Record::from_str(path.clone(), MediaType::TextPlain, "hello");
        let record3 = Record::from_str(path.clone(), MediaType::TextPlain, "world");

        assert_eq!(record1, record2);
        assert_ne!(record1, record3);
    }
}
