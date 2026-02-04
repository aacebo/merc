mod authority;
mod error;
mod scheme;

pub use authority::*;
pub use error::*;
pub use scheme::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct UriPath {
    pub scheme: Scheme,
    pub authority: Option<Authority>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl UriPath {
    pub fn new(scheme: Scheme, path: impl Into<String>) -> Self {
        Self {
            scheme,
            authority: None,
            path: path.into(),
            query: None,
            fragment: None,
        }
    }

    pub fn parse(input: &str) -> Result<Self, UriError> {
        let s = input.trim();

        if s.is_empty() {
            return Err(UriError::Empty);
        }

        let (scheme_str, rest) = s.split_once(':').ok_or(UriError::MissingScheme)?;
        let scheme = Scheme::parse(scheme_str).map_err(UriError::InvalidScheme)?;
        let (authority, path_and_rest) = if rest.starts_with("//") {
            let after_slashes = &rest[2..];
            let auth_end = after_slashes
                .find(|c| c == '/' || c == '?' || c == '#')
                .unwrap_or(after_slashes.len());
            let auth_str = &after_slashes[..auth_end];
            let authority = if auth_str.is_empty() {
                None
            } else {
                Some(Authority::parse(auth_str)?)
            };

            (authority, &after_slashes[auth_end..])
        } else {
            (None, rest)
        };

        let (path_and_query, fragment) = match path_and_rest.split_once('#') {
            Some((before, frag)) => (before, Some(frag.to_string())),
            None => (path_and_rest, None),
        };

        let (path, query) = match path_and_query.split_once('?') {
            Some((p, q)) => (p.to_string(), Some(q.to_string())),
            None => (path_and_query.to_string(), None),
        };

        Ok(Self {
            scheme,
            authority,
            path,
            query,
            fragment,
        })
    }
}

impl UriPath {
    pub fn with_authority(mut self, authority: Authority) -> Self {
        self.authority = Some(authority);
        self
    }

    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn with_fragment(mut self, fragment: impl Into<String>) -> Self {
        self.fragment = Some(fragment.into());
        self
    }

    pub fn scheme(&self) -> &Scheme {
        &self.scheme
    }

    pub fn host(&self) -> Option<&str> {
        self.authority.as_ref().map(|a| a.host.as_str())
    }

    pub fn port(&self) -> Option<u16> {
        self.authority.as_ref().and_then(|a| a.port)
    }

    pub fn username(&self) -> Option<&str> {
        self.authority.as_ref().and_then(|a| a.username.as_deref())
    }

    pub fn password(&self) -> Option<&str> {
        self.authority.as_ref().and_then(|a| a.password.as_deref())
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }
}

impl std::fmt::Display for UriPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", self.scheme)?;

        if let Some(authority) = &self.authority {
            write!(f, "//{}", authority)?;
        }

        write!(f, "{}", self.path)?;

        if let Some(query) = &self.query {
            write!(f, "?{}", query)?;
        }

        if let Some(fragment) = &self.fragment {
            write!(f, "#{}", fragment)?;
        }

        Ok(())
    }
}

impl std::str::FromStr for UriPath {
    type Err = UriError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_uri() {
        let uri: UriPath = "https://example.com/path".parse().unwrap();
        assert_eq!(uri.scheme, Scheme::Https);
        assert_eq!(uri.host(), Some("example.com"));
        assert_eq!(uri.path(), "/path");
    }

    #[test]
    fn test_parse_uri_with_port() {
        let uri: UriPath = "mqtt://broker.example.com:1883/topic".parse().unwrap();
        assert_eq!(uri.scheme, Scheme::Mqtt);
        assert_eq!(uri.host(), Some("broker.example.com"));
        assert_eq!(uri.port(), Some(1883));
        assert_eq!(uri.path(), "/topic");
    }

    #[test]
    fn test_parse_uri_with_credentials() {
        let uri: UriPath = "amqp://user:pass@localhost:5672/vhost".parse().unwrap();
        assert_eq!(uri.scheme, Scheme::Amqp);
        assert_eq!(uri.username(), Some("user"));
        assert_eq!(uri.password(), Some("pass"));
        assert_eq!(uri.host(), Some("localhost"));
        assert_eq!(uri.port(), Some(5672));
        assert_eq!(uri.path(), "/vhost");
    }

    #[test]
    fn test_parse_uri_with_username_only() {
        let uri: UriPath = "redis://admin@localhost:6379".parse().unwrap();
        assert_eq!(uri.scheme, Scheme::Redis);
        assert_eq!(uri.username(), Some("admin"));
        assert_eq!(uri.password(), None);
        assert_eq!(uri.host(), Some("localhost"));
        assert_eq!(uri.port(), Some(6379));
    }

    #[test]
    fn test_parse_uri_with_query_and_fragment() {
        let uri: UriPath = "https://example.com/path?foo=bar#section".parse().unwrap();
        assert_eq!(uri.query(), Some("foo=bar"));
        assert_eq!(uri.fragment(), Some("section"));
    }

    #[test]
    fn test_parse_file_uri() {
        let uri: UriPath = "file:///home/user/file.txt".parse().unwrap();
        assert_eq!(uri.scheme, Scheme::File);
        assert!(uri.authority.is_none());
        assert_eq!(uri.path(), "/home/user/file.txt");
    }

    #[test]
    fn test_display_roundtrip() {
        let original = "https://user:pass@example.com:8080/path?query=value#frag";
        let uri: UriPath = original.parse().unwrap();
        assert_eq!(uri.to_string(), original);
    }
}
