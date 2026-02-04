use crate::MediaType;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct ETag([u8; 32]);

impl ETag {
    pub fn new(media_type: MediaType, content: &str) -> Self {
        Self::from_bytes(media_type, content.as_bytes())
    }

    pub fn from_bytes(media_type: MediaType, content: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(media_type.as_mime_str().as_bytes());
        hasher.update(b"::");
        hasher.update(content);
        Self(*hasher.finalize().as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for ETag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}
