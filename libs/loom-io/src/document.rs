use crate::{ETag, Entity, Id, MediaType, path::Path};

#[derive(Debug, Clone, Hash, serde::Deserialize, serde::Serialize)]
pub struct Document {
    pub id: Id,
    pub etag: ETag,
    pub path: Path,
    pub size: usize,
    pub media_type: MediaType,
    pub content: Vec<Entity>,
}

impl Document {
    pub fn new(path: Path, media_type: MediaType, content: Vec<Entity>) -> Self {
        let raw = content
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        Self {
            id: Id::new(path.to_string().as_str()),
            etag: ETag::new(media_type, &raw),
            path,
            size: raw.len(),
            media_type,
            content,
        }
    }
}

impl Eq for Document {}
impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id) && self.etag.eq(&other.etag)
    }
}

impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).expect("should serialize")
        )
    }
}
