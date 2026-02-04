#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct FilePath(std::path::PathBuf);

impl FilePath {
    pub fn parse(input: &str) -> Self {
        Self(std::path::PathBuf::from(input))
    }

    pub fn len(&self) -> usize {
        self.0.components().count()
    }

    pub fn is_empty(&self) -> bool {
        self.0.as_os_str().is_empty()
    }
}

impl std::ops::Deref for FilePath {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for FilePath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_absolute_path() {
        let path = FilePath::parse("/home/user/file.txt");
        assert_eq!(path.to_string(), "/home/user/file.txt");
    }

    #[test]
    fn test_parse_relative_path() {
        let path = FilePath::parse("src/main.rs");
        assert_eq!(path.to_string(), "src/main.rs");
    }

    #[test]
    fn test_deref() {
        let path = FilePath::parse("/home/user/file.txt");
        assert_eq!(path.file_name().unwrap(), "file.txt");
    }

    #[test]
    fn test_display() {
        let path = FilePath::parse("./relative/path");
        assert_eq!(format!("{}", path), "./relative/path");
    }
}
