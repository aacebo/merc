use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use async_trait::async_trait;

use crate::MediaType;
use crate::path::Path;

use crate::data_source::{DataSource, Id, ReadError, Record, WriteError};

#[derive(Debug, Clone)]
pub struct FileSystemSourceOptions {
    pub path: PathBuf,
}

impl Default for FileSystemSourceOptions {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
        }
    }
}

impl FileSystemSourceOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = path.into();
        self
    }
}

pub struct FileSystemSource {
    options: FileSystemSourceOptions,
    cache: RwLock<HashMap<Id, Record>>,
}

impl FileSystemSource {
    pub fn new() -> Self {
        Self::with_options(FileSystemSourceOptions::default())
    }

    pub fn with_options(options: FileSystemSourceOptions) -> Self {
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
                "FileSystemSource only supports File paths".to_string(),
            )),
        }
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

    pub fn clear(&self) -> Result<(), ReadError> {
        let mut cache = self
            .cache
            .write()
            .map_err(|e| ReadError::Panic(e.to_string()))?;
        cache.clear();
        Ok(())
    }
}

impl Default for FileSystemSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DataSource for FileSystemSource {
    async fn exists(&self, path: &Path) -> Result<bool, ReadError> {
        let full_path = self.full_path(path)?;
        Ok(full_path.exists())
    }

    async fn count(&self, path: &Path) -> Result<usize, ReadError> {
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

    async fn find_one(&self, path: &Path) -> Result<Record, ReadError> {
        let id = Id::new(path.to_string().as_str());

        {
            let cache = self
                .cache
                .read()
                .map_err(|e| ReadError::Panic(e.to_string()))?;
            if let Some(record) = cache.get(&id) {
                return Ok(record.clone());
            }
        }

        let full_path = self.full_path(path)?;
        let content = std::fs::read(&full_path)?;
        let media_type = MediaType::from_path(&full_path);
        let record = Record::new(path.clone(), media_type, content);

        {
            let mut cache = self
                .cache
                .write()
                .map_err(|e| ReadError::Panic(e.to_string()))?;
            cache.insert(id, record.clone());
        }

        Ok(record)
    }

    async fn find(&self, path: &Path) -> Result<Vec<Record>, ReadError> {
        let full_path = self.full_path(path)?;

        if full_path.is_file() {
            return Ok(vec![self.find_one(path).await?]);
        }

        if full_path.is_dir() {
            let files = self.list_files(&full_path)?;
            let mut records = Vec::new();
            for file_path in files {
                let relative = file_path
                    .strip_prefix(&self.options.path)
                    .unwrap_or(&file_path);
                let path = Path::File(crate::path::FilePath::parse(
                    relative.to_str().unwrap_or(""),
                ));
                records.push(self.find_one(&path).await?);
            }
            return Ok(records);
        }

        Ok(Vec::new())
    }

    async fn create(&self, record: Record) -> Result<(), WriteError> {
        let full_path = self.full_path(&record.path).map_err(|e| match e {
            ReadError::Custom(msg) => WriteError::Custom(msg),
            ReadError::IO(io) => WriteError::IO(io),
            ReadError::Panic(msg) => WriteError::Panic(msg),
        })?;

        if full_path.exists() {
            return Err(WriteError::Custom(format!(
                "file already exists: {}",
                record.path
            )));
        }

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&full_path, &record.content)?;

        let id = record.id;
        {
            let mut cache = self
                .cache
                .write()
                .map_err(|e| WriteError::Panic(e.to_string()))?;
            cache.insert(id, record);
        }

        Ok(())
    }

    async fn update(&self, record: Record) -> Result<(), WriteError> {
        let full_path = self.full_path(&record.path).map_err(|e| match e {
            ReadError::Custom(msg) => WriteError::Custom(msg),
            ReadError::IO(io) => WriteError::IO(io),
            ReadError::Panic(msg) => WriteError::Panic(msg),
        })?;

        if !full_path.exists() {
            return Err(WriteError::Custom(format!(
                "file not found: {}",
                record.path
            )));
        }

        std::fs::write(&full_path, &record.content)?;

        let id = record.id;
        {
            let mut cache = self
                .cache
                .write()
                .map_err(|e| WriteError::Panic(e.to_string()))?;
            cache.insert(id, record);
        }

        Ok(())
    }

    async fn upsert(&self, record: Record) -> Result<(), WriteError> {
        let full_path = self.full_path(&record.path).map_err(|e| match e {
            ReadError::Custom(msg) => WriteError::Custom(msg),
            ReadError::IO(io) => WriteError::IO(io),
            ReadError::Panic(msg) => WriteError::Panic(msg),
        })?;

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&full_path, &record.content)?;

        let id = record.id;
        {
            let mut cache = self
                .cache
                .write()
                .map_err(|e| WriteError::Panic(e.to_string()))?;
            cache.insert(id, record);
        }

        Ok(())
    }

    async fn delete(&self, path: &Path) -> Result<(), WriteError> {
        let full_path = self.full_path(path).map_err(|e| match e {
            ReadError::Custom(msg) => WriteError::Custom(msg),
            ReadError::IO(io) => WriteError::IO(io),
            ReadError::Panic(msg) => WriteError::Panic(msg),
        })?;

        if !full_path.exists() {
            return Err(WriteError::Custom(format!("file not found: {}", path)));
        }

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
        temp_dir().join("loom_file_system_source_test")
    }

    fn test_options() -> FileSystemSourceOptions {
        FileSystemSourceOptions::new().with_path(test_dir())
    }

    fn make_record(path: &Path, content: &str) -> Record {
        Record::from_str(path.clone(), MediaType::TextPlain, content)
    }

    #[tokio::test]
    async fn test_exists() {
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("exists_test.txt");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));

        let ds = FileSystemSource::with_options(test_options());

        assert!(!ds.exists(&path).await.unwrap());
        std::fs::write(&file_path, "test").unwrap();
        assert!(ds.exists(&path).await.unwrap());

        let _ = std::fs::remove_file(&file_path);
    }

    #[tokio::test]
    async fn test_find_one() {
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("find_one_test.txt");
        std::fs::write(&file_path, "hello world").unwrap();

        let ds = FileSystemSource::with_options(test_options());
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));

        let record = ds.find_one(&path).await.unwrap();

        assert_eq!(record.media_type, MediaType::TextPlain);
        assert_eq!(record.content_str().unwrap(), "hello world");

        let _ = std::fs::remove_file(&file_path);
    }

    #[tokio::test]
    async fn test_create() {
        let ds = FileSystemSource::with_options(test_options());
        let file_path = test_dir().join("create_test.txt");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));
        let record = make_record(&path, "test content");

        let _ = std::fs::remove_file(&file_path);

        ds.create(record).await.unwrap();

        let written = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(written, "test content");

        let _ = std::fs::remove_file(&file_path);
    }

    #[tokio::test]
    async fn test_create_duplicate_fails() {
        let ds = FileSystemSource::with_options(test_options());
        let file_path = test_dir().join("create_dup_test.txt");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));
        let record = make_record(&path, "test");

        let _ = std::fs::remove_file(&file_path);

        ds.create(record.clone()).await.unwrap();
        let result = ds.create(record).await;
        assert!(result.is_err());

        let _ = std::fs::remove_file(&file_path);
    }

    #[tokio::test]
    async fn test_update() {
        let ds = FileSystemSource::with_options(test_options());
        let file_path = test_dir().join("update_test.txt");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));

        std::fs::create_dir_all(test_dir()).unwrap();
        std::fs::write(&file_path, "old").unwrap();

        let record = make_record(&path, "new");
        ds.update(record).await.unwrap();

        let written = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(written, "new");

        let _ = std::fs::remove_file(&file_path);
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let ds = FileSystemSource::with_options(test_options());
        let file_path = test_dir().join("update_not_found.txt");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));
        let record = make_record(&path, "test");

        let _ = std::fs::remove_file(&file_path);

        let result = ds.update(record).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_upsert() {
        let ds = FileSystemSource::with_options(test_options());
        let file_path = test_dir().join("upsert_test.txt");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));
        let record = make_record(&path, "test");

        let _ = std::fs::remove_file(&file_path);

        ds.upsert(record.clone()).await.unwrap();
        assert!(ds.exists(&path).await.unwrap());

        ds.upsert(record).await.unwrap();
        assert!(ds.exists(&path).await.unwrap());

        let _ = std::fs::remove_file(&file_path);
    }

    #[tokio::test]
    async fn test_delete() {
        let ds = FileSystemSource::with_options(test_options());
        let file_path = test_dir().join("delete_test.txt");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));
        let record = make_record(&path, "test");

        let _ = std::fs::remove_file(&file_path);

        ds.create(record).await.unwrap();
        assert!(ds.exists(&path).await.unwrap());

        ds.delete(&path).await.unwrap();
        assert!(!ds.exists(&path).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let ds = FileSystemSource::with_options(test_options());
        let path = Path::File(FilePath::parse("/nonexistent/file.txt"));

        let result = ds.delete(&path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_roundtrip() {
        let ds = FileSystemSource::with_options(test_options());
        let file_path = test_dir().join("roundtrip.txt");
        let path = Path::File(FilePath::parse(file_path.to_str().unwrap()));

        let _ = std::fs::remove_file(&file_path);

        let record = make_record(&path, "roundtrip content");
        ds.create(record).await.unwrap();
        ds.clear().unwrap();

        let read_record = ds.find_one(&path).await.unwrap();
        assert_eq!(read_record.content_str().unwrap(), "roundtrip content");

        let _ = std::fs::remove_file(&file_path);
    }

    #[tokio::test]
    async fn test_find_one_not_found() {
        let ds = FileSystemSource::with_options(test_options());
        let path = Path::File(FilePath::parse("/nonexistent/file.txt"));

        let result = ds.find_one(&path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().is_io());
    }

    #[test]
    fn test_options_builder() {
        let options = FileSystemSourceOptions::new().with_path("/custom/path");

        assert_eq!(options.path, PathBuf::from("/custom/path"));
    }
}
