mod error;
mod field;
mod file;
mod uri;

pub use field::*;
pub use file::*;
pub use uri::*;

/// Creates a `Path` from a string literal.
///
/// # Variants
///
/// - `path!(file "path/to/file")` - Creates a `Path::File`
/// - `path!(uri "https://example.com")` - Creates a `Path::Uri` (panics on invalid URI)
/// - `path!(field "object.field[0]")` - Creates a `Path::Field` (panics on invalid field path)
///
/// # Examples
///
/// ```
/// use loom_runtime::path;
///
/// let file = path!(file "/home/user/file.txt");
/// let uri = path!(uri "https://example.com/path");
/// let field = path!(field "data.items[0].name");
/// ```
#[macro_export]
macro_rules! path {
    (file $path:expr) => {
        $crate::path::Path::File($crate::path::FilePath::parse($path))
    };
    (uri $path:expr) => {
        $crate::path::Path::Uri($crate::path::UriPath::parse($path).expect("invalid URI"))
    };
    (field $path:expr) => {
        $crate::path::Path::Field(
            $crate::path::FieldPath::parse($path).expect("invalid field path"),
        )
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum Path {
    File(FilePath),
    Uri(UriPath),
    Field(FieldPath),
}

impl Path {
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    pub fn is_uri(&self) -> bool {
        matches!(self, Self::Uri(_))
    }

    pub fn is_field(&self) -> bool {
        matches!(self, Self::Field(_))
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(v) => write!(f, "{}", v),
            Self::Uri(v) => write!(f, "{}", v),
            Self::Field(v) => write!(f, "{}", v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_macro_file() {
        let path = path!(file "/home/user/file.txt");
        assert!(path.is_file());
        assert_eq!(path.to_string(), "/home/user/file.txt");
    }

    #[test]
    fn test_path_macro_uri() {
        let path = path!(uri "https://example.com/path");
        assert!(path.is_uri());
        assert_eq!(path.to_string(), "https://example.com/path");
    }

    #[test]
    fn test_path_macro_field() {
        let path = path!(field "object.field[0]");
        assert!(path.is_field());
        assert_eq!(path.to_string(), "object.field[0]");
    }

    #[test]
    fn test_path_is_file() {
        let path = Path::File(FilePath::parse("/home/user/file.txt"));
        assert!(path.is_file());
        assert!(!path.is_uri());
        assert!(!path.is_field());
    }

    #[test]
    fn test_path_is_uri() {
        let path = Path::Uri(UriPath::parse("https://example.com").unwrap());
        assert!(!path.is_file());
        assert!(path.is_uri());
        assert!(!path.is_field());
    }

    #[test]
    fn test_path_is_field() {
        let path = Path::Field(FieldPath::parse("object.field").unwrap());
        assert!(!path.is_file());
        assert!(!path.is_uri());
        assert!(path.is_field());
    }

    #[test]
    fn test_path_display_file() {
        let path = Path::File(FilePath::parse("/home/user/file.txt"));
        assert_eq!(path.to_string(), "/home/user/file.txt");
    }

    #[test]
    fn test_path_display_field() {
        let path = Path::Field(FieldPath::parse("object.field[0]").unwrap());
        assert_eq!(path.to_string(), "object.field[0]");
    }
}
