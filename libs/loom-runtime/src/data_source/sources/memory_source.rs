use std::collections::HashMap;
use std::sync::RwLock;

use crate::path::Path;

use crate::data_source::{DataSource, Document, Id, ReadError, WriteError};

pub struct MemorySource {
    documents: RwLock<HashMap<Id, Document>>,
}

impl MemorySource {
    pub fn new() -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemorySource {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for MemorySource {
    fn exists(&self, path: &Path) -> Result<bool, ReadError> {
        let id = Id::new(path.to_string().as_str());
        let documents = self
            .documents
            .read()
            .map_err(|e| ReadError::Panic(e.to_string()))?;
        Ok(documents.contains_key(&id))
    }

    fn count(&self, path: &Path) -> Result<usize, ReadError> {
        let path_str = path.to_string();
        let documents = self
            .documents
            .read()
            .map_err(|e| ReadError::Panic(e.to_string()))?;

        let count = documents
            .values()
            .filter(|doc| doc.path.to_string().starts_with(&path_str))
            .count();
        Ok(count)
    }

    fn find_one(&self, path: &Path) -> Result<Document, ReadError> {
        let id = Id::new(path.to_string().as_str());
        let documents = self
            .documents
            .read()
            .map_err(|e| ReadError::Panic(e.to_string()))?;

        documents
            .get(&id)
            .cloned()
            .ok_or_else(|| ReadError::Custom(format!("document not found: {}", path)))
    }

    fn find(&self, path: &Path) -> Result<Vec<Document>, ReadError> {
        let path_str = path.to_string();
        let documents = self
            .documents
            .read()
            .map_err(|e| ReadError::Panic(e.to_string()))?;

        let results: Vec<Document> = documents
            .values()
            .filter(|doc| doc.path.to_string().starts_with(&path_str))
            .cloned()
            .collect();
        Ok(results)
    }

    fn create(&self, document: Document) -> Result<(), WriteError> {
        let mut documents = self
            .documents
            .write()
            .map_err(|e| WriteError::Panic(e.to_string()))?;

        if documents.contains_key(&document.id) {
            return Err(WriteError::Custom(format!(
                "document already exists: {}",
                document.path
            )));
        }

        documents.insert(document.id, document);
        Ok(())
    }

    fn update(&self, document: Document) -> Result<(), WriteError> {
        let mut documents = self
            .documents
            .write()
            .map_err(|e| WriteError::Panic(e.to_string()))?;

        if !documents.contains_key(&document.id) {
            return Err(WriteError::Custom(format!(
                "document not found: {}",
                document.path
            )));
        }

        documents.insert(document.id, document);
        Ok(())
    }

    fn upsert(&self, document: Document) -> Result<(), WriteError> {
        let mut documents = self
            .documents
            .write()
            .map_err(|e| WriteError::Panic(e.to_string()))?;
        documents.insert(document.id, document);
        Ok(())
    }

    fn delete(&self, path: &Path) -> Result<(), WriteError> {
        let id = Id::new(path.to_string().as_str());
        let mut documents = self
            .documents
            .write()
            .map_err(|e| WriteError::Panic(e.to_string()))?;

        if documents.remove(&id).is_none() {
            return Err(WriteError::Custom(format!("document not found: {}", path)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, MediaType, path::FieldPath, path::FilePath, value::Value};

    fn make_doc(path: &Path) -> Document {
        let entity = Entity::new(
            FieldPath::parse("root").unwrap(),
            "text",
            Value::String("hello".to_string()),
        );
        Document::new(path.clone(), MediaType::TextPlain, vec![entity])
    }

    #[test]
    fn test_create_and_find_one() {
        let ds = MemorySource::new();
        let path = Path::File(FilePath::parse("/test/file.txt"));
        let doc = make_doc(&path);

        ds.create(doc.clone()).unwrap();
        let read_doc = ds.find_one(&path).unwrap();

        assert_eq!(read_doc, doc);
    }

    #[test]
    fn test_exists() {
        let ds = MemorySource::new();
        let path = Path::File(FilePath::parse("/test/file.txt"));
        let doc = make_doc(&path);

        assert!(!ds.exists(&path).unwrap());
        ds.create(doc).unwrap();
        assert!(ds.exists(&path).unwrap());
    }

    #[test]
    fn test_count() {
        let ds = MemorySource::new();
        let path1 = Path::File(FilePath::parse("/test/file1.txt"));
        let path2 = Path::File(FilePath::parse("/test/file2.txt"));
        let path3 = Path::File(FilePath::parse("/other/file.txt"));

        ds.create(make_doc(&path1)).unwrap();
        ds.create(make_doc(&path2)).unwrap();
        ds.create(make_doc(&path3)).unwrap();

        let test_path = Path::File(FilePath::parse("/test"));
        assert_eq!(ds.count(&test_path).unwrap(), 2);
    }

    #[test]
    fn test_find() {
        let ds = MemorySource::new();
        let path1 = Path::File(FilePath::parse("/test/file1.txt"));
        let path2 = Path::File(FilePath::parse("/test/file2.txt"));
        let path3 = Path::File(FilePath::parse("/other/file.txt"));

        ds.create(make_doc(&path1)).unwrap();
        ds.create(make_doc(&path2)).unwrap();
        ds.create(make_doc(&path3)).unwrap();

        let test_path = Path::File(FilePath::parse("/test"));
        let results = ds.find(&test_path).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_create_duplicate_fails() {
        let ds = MemorySource::new();
        let path = Path::File(FilePath::parse("/test/file.txt"));
        let doc = make_doc(&path);

        ds.create(doc.clone()).unwrap();
        let result = ds.create(doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_update() {
        let ds = MemorySource::new();
        let path = Path::File(FilePath::parse("/test/file.txt"));
        let doc = make_doc(&path);

        ds.create(doc.clone()).unwrap();
        ds.update(doc).unwrap();
    }

    #[test]
    fn test_update_not_found() {
        let ds = MemorySource::new();
        let path = Path::File(FilePath::parse("/test/file.txt"));
        let doc = make_doc(&path);

        let result = ds.update(doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_upsert() {
        let ds = MemorySource::new();
        let path = Path::File(FilePath::parse("/test/file.txt"));
        let doc = make_doc(&path);

        // Works when doesn't exist
        ds.upsert(doc.clone()).unwrap();
        assert!(ds.exists(&path).unwrap());

        // Works when exists
        ds.upsert(doc).unwrap();
        assert!(ds.exists(&path).unwrap());
    }

    #[test]
    fn test_delete() {
        let ds = MemorySource::new();
        let path = Path::File(FilePath::parse("/test/file.txt"));
        let doc = make_doc(&path);

        ds.create(doc).unwrap();
        assert!(ds.exists(&path).unwrap());

        ds.delete(&path).unwrap();
        assert!(!ds.exists(&path).unwrap());
    }

    #[test]
    fn test_delete_not_found() {
        let ds = MemorySource::new();
        let path = Path::File(FilePath::parse("/nonexistent"));
        let result = ds.delete(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_one_not_found() {
        let ds = MemorySource::new();
        let path = Path::File(FilePath::parse("/nonexistent"));
        let result = ds.find_one(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_custom());
    }
}
