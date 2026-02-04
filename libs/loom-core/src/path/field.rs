pub use super::error::FieldPathError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct FieldPath(Vec<FieldSegment>);

impl FieldPath {
    pub fn parse(input: &str) -> Result<Self, FieldPathError> {
        let s = input.trim();

        if s.is_empty() {
            return Err(FieldPathError::Empty);
        }

        let mut segments = Vec::new();
        let mut chars = s.chars().peekable();
        let mut first = true;

        while let Some(segment) = FieldSegment::parse_next(&mut chars, !first)? {
            segments.push(segment);
            first = false;
        }

        if segments.is_empty() {
            return Err(FieldPathError::Empty);
        }

        Ok(Self(segments))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn segments(&self) -> &[FieldSegment] {
        &self.0
    }
}

impl std::fmt::Display for FieldPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, segment) in self.0.iter().enumerate() {
            match segment {
                FieldSegment::Key(v) if i == 0 => write!(f, "{}", v)?,
                FieldSegment::Key(v) => write!(f, ".{}", v)?,
                FieldSegment::Index(v) => write!(f, "[{}]", v)?,
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum FieldSegment {
    Key(String),
    Index(usize),
}

impl FieldSegment {
    fn parse_next(
        chars: &mut std::iter::Peekable<std::str::Chars>,
        expect_separator: bool,
    ) -> Result<Option<Self>, FieldPathError> {
        if expect_separator {
            match chars.peek() {
                None => return Ok(None),
                Some(&'.') => {
                    chars.next();
                    if chars.peek().is_none() {
                        return Err(FieldPathError::EmptySegment);
                    }
                }
                Some(&'[') => {}
                Some(&']') => return Err(FieldPathError::UnmatchedBracket),
                Some(_) => return Err(FieldPathError::EmptySegment),
            }
        }

        match chars.peek() {
            None => Ok(None),
            Some(&'.') => Err(FieldPathError::EmptySegment),
            Some(&'[') => Self::parse_index(chars).map(Some),
            Some(&']') => Err(FieldPathError::UnmatchedBracket),
            Some(_) => Self::parse_key(chars).map(Some),
        }
    }

    fn parse_key(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Self, FieldPathError> {
        let mut key = String::new();

        while let Some(&c) = chars.peek() {
            match c {
                '.' | '[' => break,
                ']' => return Err(FieldPathError::UnmatchedBracket),
                _ => {
                    key.push(c);
                    chars.next();
                }
            }
        }

        if key.is_empty() {
            return Err(FieldPathError::EmptySegment);
        }

        Ok(Self::Key(key))
    }

    fn parse_index(
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Result<Self, FieldPathError> {
        chars.next(); // consume '['

        let mut index = String::new();

        loop {
            match chars.next() {
                Some(']') => break,
                Some(c) => index.push(c),
                None => return Err(FieldPathError::UnmatchedBracket),
            }
        }

        if index.is_empty() {
            return Err(FieldPathError::EmptyBracket);
        }

        let value = index.parse().map_err(|_| FieldPathError::InvalidIndex)?;
        Ok(Self::Index(value))
    }
}

impl std::fmt::Display for FieldSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(v) => write!(f, ".{}", v),
            Self::Index(v) => write!(f, "[{}]", v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_key() {
        let path = FieldPath::parse("object").unwrap();
        assert_eq!(path.to_string(), "object");
    }

    #[test]
    fn test_parse_dotted_path() {
        let path = FieldPath::parse("object.field").unwrap();
        assert_eq!(path.to_string(), "object.field");
    }

    #[test]
    fn test_parse_index() {
        let path = FieldPath::parse("arr[0]").unwrap();
        assert_eq!(path.to_string(), "arr[0]");
    }

    #[test]
    fn test_parse_complex() {
        let path = FieldPath::parse("object.field[2].test").unwrap();
        assert_eq!(path.to_string(), "object.field[2].test");
    }

    #[test]
    fn test_parse_consecutive_indices() {
        let path = FieldPath::parse("arr[0][1]").unwrap();
        assert_eq!(path.to_string(), "arr[0][1]");
    }

    #[test]
    fn test_parse_index_after_dot() {
        let path = FieldPath::parse("a[0].b").unwrap();
        assert_eq!(path.to_string(), "a[0].b");
    }

    #[test]
    fn test_parse_empty_error() {
        let err = FieldPath::parse("").unwrap_err();
        assert_eq!(err, FieldPathError::Empty);
    }

    #[test]
    fn test_parse_empty_segment_error() {
        let err = FieldPath::parse("a..b").unwrap_err();
        assert_eq!(err, FieldPathError::EmptySegment);
    }

    #[test]
    fn test_parse_trailing_dot_error() {
        let err = FieldPath::parse("a.").unwrap_err();
        assert_eq!(err, FieldPathError::EmptySegment);
    }

    #[test]
    fn test_parse_leading_dot_error() {
        let err = FieldPath::parse(".a").unwrap_err();
        assert_eq!(err, FieldPathError::EmptySegment);
    }

    #[test]
    fn test_parse_unmatched_open_bracket_error() {
        let err = FieldPath::parse("a[0").unwrap_err();
        assert_eq!(err, FieldPathError::UnmatchedBracket);
    }

    #[test]
    fn test_parse_unmatched_close_bracket_error() {
        let err = FieldPath::parse("a]0").unwrap_err();
        assert_eq!(err, FieldPathError::UnmatchedBracket);
    }

    #[test]
    fn test_parse_empty_bracket_error() {
        let err = FieldPath::parse("a[]").unwrap_err();
        assert_eq!(err, FieldPathError::EmptyBracket);
    }

    #[test]
    fn test_parse_invalid_index_error() {
        let err = FieldPath::parse("a[abc]").unwrap_err();
        assert_eq!(err, FieldPathError::InvalidIndex);
    }

    #[test]
    fn test_display_roundtrip() {
        let inputs = [
            "object",
            "object.field",
            "arr[0]",
            "object.field[2].test",
            "arr[0][1]",
            "a[0].b",
        ];

        for input in inputs {
            let path = FieldPath::parse(input).unwrap();
            assert_eq!(path.to_string(), input);
        }
    }
}
