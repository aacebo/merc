use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::MediaType;
use crate::path::{FieldPath, Path};
use crate::value::Value;

use crate::data_source::{DataSource, Document, Entity, Id, ReadError, WriteError};

#[derive(Debug, Clone)]
pub struct JsonFileSourceOptions {
    pub path: PathBuf,
    pub pretty_print: bool,
}

impl Default for JsonFileSourceOptions {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            pretty_print: false,
        }
    }
}

impl JsonFileSourceOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = path.into();
        self
    }

    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }
}

pub struct JsonFileSource {
    options: JsonFileSourceOptions,
    cache: RwLock<HashMap<Id, Document>>,
}

impl JsonFileSource {
    pub fn new() -> Self {
        Self::with_options(JsonFileSourceOptions::default())
    }

    pub fn with_options(options: JsonFileSourceOptions) -> Self {
        Self {
            options,
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn full_path(&self, path: &Path) -> Result<PathBuf, ReadError> {
        match path {
            Path::File(file_path) => {
                let path_buf: &std::path::Path = file_path;
                if path_buf.is_absolute() {
                    Ok(path_buf.to_path_buf())
                } else {
                    Ok(self.options.path.join(path_buf))
                }
            }
            _ => Err(ReadError::Custom(
                "JsonFileSource only supports File paths".to_string(),
            )),
        }
    }

    pub fn clear(&self) -> Result<(), ReadError> {
        let mut cache = self
            .cache
            .write()
            .map_err(|e| ReadError::Panic(e.to_string()))?;
        cache.clear();
        Ok(())
    }
}

impl Default for JsonFileSource {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonFileSource {
    fn read_file(&self, path: &Path) -> Result<Document, ReadError> {
        let id = Id::new(path.to_string().as_str());

        {
            let cache = self
                .cache
                .read()
                .map_err(|e| ReadError::Panic(e.to_string()))?;
            if let Some(doc) = cache.get(&id) {
                return Ok(doc.clone());
            }
        }

        let full_path = self.full_path(path)?;
        let content_str = std::fs::read_to_string(&full_path)?;
        let media_type = MediaType::from_path(&full_path);
        let content = if media_type == MediaType::TextJson {
            let json: serde_json::Value = serde_json::from_str(&content_str)
                .map_err(|e| ReadError::Custom(format!("JSON parse error: {}", e)))?;
            json.into()
        } else if media_type.is_textlike() {
            Value::String(content_str)
        } else {
            return Err(ReadError::Custom(format!(
                "Unsupported media type: {}",
                media_type
            )));
        };

        let entity = Entity::new(
            FieldPath::parse("root").expect("valid field path"),
            media_type.as_mime_str(),
            content,
        );

        let document = Document::new(path.clone(), media_type, vec![entity]);

        {
            let mut cache = self
                .cache
                .write()
                .map_err(|e| ReadError::Panic(e.to_string()))?;
            cache.insert(id, document.clone());
        }

        Ok(document)
    }

    fn write_file(&self, document: &Document) -> Result<(), WriteError> {
        let full_path = self.full_path(&document.path).map_err(|e| match e {
            ReadError::Custom(msg) => WriteError::Custom(msg),
            ReadError::IO(io) => WriteError::IO(io),
            ReadError::Panic(msg) => WriteError::Panic(msg),
        })?;

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = document
            .content
            .first()
            .ok_or_else(|| WriteError::Custom("Document has no content".to_string()))?;

        let output = if document.media_type == MediaType::TextJson {
            let json: serde_json::Value = (&content.content).into();
            if self.options.pretty_print {
                serde_json::to_string_pretty(&json)
            } else {
                serde_json::to_string(&json)
            }
            .map_err(|e| WriteError::Custom(format!("JSON serialize error: {}", e)))?
        } else if document.media_type.is_textlike() {
            content
                .content
                .as_str()
                .ok_or_else(|| {
                    WriteError::Custom("Text content must be a string Value".to_string())
                })?
                .to_string()
        } else {
            return Err(WriteError::Custom(format!(
                "Unsupported media type: {}",
                document.media_type
            )));
        };

        std::fs::write(&full_path, &output)?;

        let id = document.id;
        {
            let mut cache = self
                .cache
                .write()
                .map_err(|e| WriteError::Panic(e.to_string()))?;
            cache.insert(id, document.clone());
        }

        Ok(())
    }

    fn list_files(&self, dir_path: &std::path::Path) -> Result<Vec<PathBuf>, ReadError> {
        let mut files = Vec::new();
        if dir_path.is_dir() {
            for entry in std::fs::read_dir(dir_path)? {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_file() {
                    files.push(entry_path);
                } else if entry_path.is_dir() {
                    files.extend(self.list_files(&entry_path)?);
                }
            }
        }
        Ok(files)
    }
}

impl DataSource for JsonFileSource {
    fn exists(&self, path: &Path) -> Result<bool, ReadError> {
        let full_path = self.full_path(path)?;
        Ok(full_path.exists())
    }

    fn count(&self, path: &Path) -> Result<usize, ReadError> {
        let full_path = self.full_path(path)?;
        if full_path.is_file() {
            return Ok(1);
        }
        if full_path.is_dir() {
            let files = self.list_files(&full_path)?;
            return Ok(files.len());
        }
        Ok(0)
    }

    fn find_one(&self, path: &Path) -> Result<Document, ReadError> {
        self.read_file(path)
    }

    fn find(&self, path: &Path) -> Result<Vec<Document>, ReadError> {
        let full_path = self.full_path(path)?;

        if full_path.is_file() {
            return Ok(vec![self.read_file(path)?]);
        }

        if full_path.is_dir() {
            let files = self.list_files(&full_path)?;
            let mut documents = Vec::new();
            for file_path in files {
                let relative = file_path
                    .strip_prefix(&self.options.path)
                    .unwrap_or(&file_path);
                let path = Path::File(crate::path::FilePath::parse(
                    relative.to_str().unwrap_or(""),
                ));
                documents.push(self.read_file(&path)?);
            }
            return Ok(documents);
        }

        Ok(Vec::new())
    }

    fn create(&self, document: Document) -> Result<(), WriteError> {
        let full_path = self.full_path(&document.path).map_err(|e| match e {
            ReadError::Custom(msg) => WriteError::Custom(msg),
            ReadError::IO(io) => WriteError::IO(io),
            ReadError::Panic(msg) => WriteError::Panic(msg),
        })?;

        if full_path.exists() {
            return Err(WriteError::Custom(format!(
                "file already exists: {}",
                document.path
            )));
        }

        self.write_file(&document)
    }

    fn update(&self, document: Document) -> Result<(), WriteError> {
        let full_path = self.full_path(&document.path).map_err(|e| match e {
            ReadError::Custom(msg) => WriteError::Custom(msg),
            ReadError::IO(io) => WriteError::IO(io),
            ReadError::Panic(msg) => WriteError::Panic(msg),
        })?;

        if !full_path.exists() {
            return Err(WriteError::Custom(format!(
                "file not found: {}",
                document.path
            )));
        }

        self.write_file(&document)
    }

    fn upsert(&self, document: Document) -> Result<(), WriteError> {
        self.write_file(&document)
    }

    fn delete(&self, path: &Path) -> Result<(), WriteError> {
        let full_path = self.full_path(path).map_err(|e| match e {
            ReadError::Custom(msg) => WriteError::Custom(msg),
            ReadError::IO(io) => WriteError::IO(io),
            ReadError::Panic(msg) => WriteError::Panic(msg),
        })?;

        if !full_path.exists() {
            return Err(WriteError::Custom(format!("file not found: {}", path)));
        }

        // Remove from cache
        let id = Id::new(path.to_string().as_str());
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(&id);
        }

        std::fs::remove_file(&full_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::FilePath;
    use std::env::temp_dir;

    fn test_dir() -> PathBuf {
        temp_dir().join("loom_json_source_test")
    }

    fn test_options() -> JsonFileSourceOptions {
        JsonFileSourceOptions::new()
            .with_path(test_dir())
            .with_pretty_print(true)
    }

    fn make_json_doc(path: &Path) -> Document {
        let mut obj = crate::value::Object::new();
        obj.insert("key".to_string(), Value::String("value".to_string()));
        let content = Value::Object(obj);
        let entity = Entity::new(
            FieldPath::parse("root").unwrap(),
            "application/json",
            content,
        );
        Document::new(path.clone(), MediaType::TextJson, vec![entity])
    }

    #[test]
    fn test_find_one_json_file() {
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.json");
        std::fs::write(&file_path, r#"{"name": "test", "value": 42}"#).unwrap();

        let ds = JsonFileSource::with_options(test_options());
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));

        let doc = ds.find_one(&path).unwrap();

        assert_eq!(doc.media_type, MediaType::TextJson);
        assert!(doc.content[0].content.is_object());
        assert_eq!(doc.content[0].content["name"].as_str(), Some("test"));
        assert_eq!(doc.content[0].content["value"].as_int(), Some(42));
        assert_eq!(doc.content[0].otype, "application/json");

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_find_one_text_file() {
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.txt");
        std::fs::write(&file_path, "Hello, World!").unwrap();

        let ds = JsonFileSource::with_options(test_options());
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));

        let doc = ds.find_one(&path).unwrap();

        assert_eq!(doc.media_type, MediaType::TextPlain);
        assert_eq!(doc.content[0].content.as_str(), Some("Hello, World!"));
        assert_eq!(doc.content[0].otype, "text/plain");

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_exists() {
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("exists_test.json");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));

        let ds = JsonFileSource::with_options(test_options());

        assert!(!ds.exists(&path).unwrap());
        std::fs::write(&file_path, "{}").unwrap();
        assert!(ds.exists(&path).unwrap());

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_create_json_file() {
        let ds = JsonFileSource::with_options(test_options());
        let file_path = test_dir().join("create_test.json");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));
        let doc = make_json_doc(&path);

        // Ensure file doesn't exist
        let _ = std::fs::remove_file(&file_path);

        ds.create(doc).unwrap();

        let written = std::fs::read_to_string(&file_path).unwrap();
        assert!(written.contains("\"key\""));
        assert!(written.contains("\"value\""));

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_create_duplicate_fails() {
        let ds = JsonFileSource::with_options(test_options());
        let file_path = test_dir().join("create_dup_test.json");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));
        let doc = make_json_doc(&path);

        // Ensure file doesn't exist
        let _ = std::fs::remove_file(&file_path);

        ds.create(doc.clone()).unwrap();
        let result = ds.create(doc);
        assert!(result.is_err());

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_update() {
        let ds = JsonFileSource::with_options(test_options());
        let file_path = test_dir().join("update_test.json");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));

        // Create file first
        std::fs::create_dir_all(test_dir()).unwrap();
        std::fs::write(&file_path, "{}").unwrap();

        let doc = make_json_doc(&path);
        ds.update(doc).unwrap();

        let written = std::fs::read_to_string(&file_path).unwrap();
        assert!(written.contains("\"key\""));

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_update_not_found() {
        let ds = JsonFileSource::with_options(test_options());
        let file_path = test_dir().join("update_not_found.json");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));
        let doc = make_json_doc(&path);

        // Ensure file doesn't exist
        let _ = std::fs::remove_file(&file_path);

        let result = ds.update(doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_upsert() {
        let ds = JsonFileSource::with_options(test_options());
        let file_path = test_dir().join("upsert_test.json");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));
        let doc = make_json_doc(&path);

        // Ensure file doesn't exist
        let _ = std::fs::remove_file(&file_path);

        // Works when doesn't exist
        ds.upsert(doc.clone()).unwrap();
        assert!(ds.exists(&path).unwrap());

        // Works when exists
        ds.upsert(doc).unwrap();
        assert!(ds.exists(&path).unwrap());

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_delete() {
        let ds = JsonFileSource::with_options(test_options());
        let file_path = test_dir().join("delete_test.json");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));
        let doc = make_json_doc(&path);

        // Ensure clean state
        let _ = std::fs::remove_file(&file_path);

        ds.create(doc).unwrap();
        assert!(ds.exists(&path).unwrap());

        ds.delete(&path).unwrap();
        assert!(!ds.exists(&path).unwrap());
    }

    #[test]
    fn test_delete_not_found() {
        let ds = JsonFileSource::with_options(test_options());
        let path = Path::File(FilePath::parse("/nonexistent/file.json"));

        let result = ds.delete(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip() {
        let ds = JsonFileSource::with_options(test_options());
        let file_path = test_dir().join("roundtrip.json");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));

        let mut obj = crate::value::Object::new();
        obj.insert("test".to_string(), Value::from(123));
        let content = Value::Object(obj);

        let entity = Entity::new(
            FieldPath::parse("root").unwrap(),
            "application/json",
            content.clone(),
        );
        let doc = Document::new(path.clone(), MediaType::TextJson, vec![entity]);

        // Ensure clean state
        let _ = std::fs::remove_file(&file_path);

        ds.create(doc).unwrap();
        ds.clear().unwrap();

        let read_doc = ds.find_one(&path).unwrap();
        assert_eq!(read_doc.content[0].content["test"].as_int(), Some(123));

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_find_one_not_found() {
        let ds = JsonFileSource::with_options(test_options());
        let path = Path::File(FilePath::parse("/nonexistent/file.txt"));

        let result = ds.find_one(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_io());
    }

    #[test]
    fn test_options_builder() {
        let options = JsonFileSourceOptions::new()
            .with_path("/custom/path")
            .with_pretty_print(true);

        assert_eq!(options.path, PathBuf::from("/custom/path"));
        assert!(options.pretty_print);
    }
}
