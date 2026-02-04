use std::path::PathBuf;

use crate::Format;
use crate::path::{FilePath, Path};
use crate::value::Value;

use super::{ConfigError, Provider};

/// File-based configuration provider
pub struct FileProvider {
    path: PathBuf,
    format: Format,
    is_optional: bool,
}

impl FileProvider {
    pub fn new(path: impl Into<PathBuf>, format: Format, optional: bool) -> Self {
        Self {
            path: path.into(),
            format,
            is_optional: optional,
        }
    }

    #[cfg(feature = "json")]
    pub fn json(path: impl Into<PathBuf>, optional: bool) -> Self {
        Self::new(path, Format::Json, optional)
    }

    #[cfg(feature = "yaml")]
    pub fn yaml(path: impl Into<PathBuf>, optional: bool) -> Self {
        Self::new(path, Format::Yaml, optional)
    }

    fn parse_content(&self, content: &str) -> Result<Value, ConfigError> {
        match self.format {
            #[cfg(feature = "json")]
            Format::Json => {
                let json: serde_json::Value =
                    serde_json::from_str(content).map_err(ConfigError::parse)?;
                Ok(json.into())
            }
            #[cfg(feature = "yaml")]
            Format::Yaml => {
                let docs = saphyr::Yaml::load_from_str(content).map_err(ConfigError::parse)?;
                if let Some(doc) = docs.into_iter().next() {
                    Ok(doc.into())
                } else {
                    Ok(Value::Null)
                }
            }
            _ => Err(ConfigError::provider(format!(
                "unsupported format: {:?}",
                self.format
            ))),
        }
    }
}

impl Provider for FileProvider {
    fn name(&self) -> &str {
        self.path.to_str().unwrap_or("file")
    }

    fn optional(&self) -> bool {
        self.is_optional
    }

    fn path(&self) -> Option<Path> {
        Some(Path::File(FilePath::parse(
            self.path.to_str().unwrap_or(""),
        )))
    }

    fn format(&self) -> Option<crate::Format> {
        Some(self.format)
    }

    fn load(&self) -> Result<Option<Value>, ConfigError> {
        if !self.path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&self.path)?;
        let value = self.parse_content(&content)?;

        Ok(Some(value))
    }
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use super::*;

    #[test]
    fn test_file_provider_not_found_optional() {
        let provider = FileProvider::json("/nonexistent/path.json", true);
        let result = provider.load().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_file_provider_not_found_required() {
        let provider = FileProvider::json("/nonexistent/path.json", false);
        let result = provider.load().unwrap();
        assert!(result.is_none());
    }
}
