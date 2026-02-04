use super::AuthorityError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct Authority {
    pub username: Option<String>,
    pub password: Option<String>,
    pub host: String,
    pub port: Option<u16>,
}

impl Authority {
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            username: None,
            password: None,
            host: host.into(),
            port: None,
        }
    }

    pub fn parse(s: &str) -> Result<Self, AuthorityError> {
        let (username, password, host_and_port) = match s.split_once('@') {
            Some((userinfo, rest)) => {
                let (username, password) = match userinfo.split_once(':') {
                    Some((u, p)) => (Some(u.to_string()), Some(p.to_string())),
                    None => (Some(userinfo.to_string()), None),
                };
                (username, password, rest)
            }
            None => (None, None, s),
        };

        let (host, port) = if host_and_port.starts_with('[') {
            let bracket_end = host_and_port
                .find(']')
                .ok_or(AuthorityError::InvalidSyntax)?;
            let host = &host_and_port[1..bracket_end];
            let after_bracket = &host_and_port[bracket_end + 1..];
            let port = if after_bracket.starts_with(':') {
                Some(
                    after_bracket[1..]
                        .parse()
                        .map_err(|_| AuthorityError::InvalidPort)?,
                )
            } else {
                None
            };
            (host.to_string(), port)
        } else {
            match host_and_port.rsplit_once(':') {
                Some((h, p)) => {
                    let port: u16 = p.parse().map_err(|_| AuthorityError::InvalidPort)?;
                    (h.to_string(), Some(port))
                }
                None => (host_and_port.to_string(), None),
            }
        };

        Ok(Self {
            username,
            password,
            host,
            port,
        })
    }
}

impl Authority {
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    pub fn userinfo(&self) -> Option<String> {
        match (&self.username, &self.password) {
            (Some(u), Some(p)) => Some(format!("{}:{}", u, p)),
            (Some(u), None) => Some(u.clone()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Authority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(userinfo) = self.userinfo() {
            write!(f, "{}@", userinfo)?;
        }

        write!(f, "{}", self.host)?;

        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }

        Ok(())
    }
}

impl std::str::FromStr for Authority {
    type Err = AuthorityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
