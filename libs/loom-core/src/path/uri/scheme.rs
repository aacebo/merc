#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum Scheme {
    // File
    File,
    // Web
    Http,
    Https,
    Ws,
    Wss,
    // Messaging
    Mqtt,
    Mqtts,
    Amqp,
    Amqps,
    Kafka,
    Nats,
    // Databases
    S3,
    Pg,
    Redis,
    Rediss,
    Mongo,
    Mysql,
    // Remote access
    Ftp,
    Ftps,
    Sftp,
    Ssh,
    Ldap,
    Ldaps,
    Telnet,
    // RPC
    Grpc,
    Grpcs,
    // Other
    Data,
    Mailto,
    Tel,
    Dns,
    Index,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemeError {
    Empty,
    InvalidSyntax,
}

impl Scheme {
    /// Parse + validate scheme syntax, then map into a known variant (or Unknown).
    pub fn parse(input: &str) -> Result<Self, SchemeError> {
        let s = input.trim();
        if s.is_empty() {
            return Err(SchemeError::Empty);
        }

        // RFC-ish: ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
        let mut chars = s.chars();
        let Some(first) = chars.next() else {
            return Err(SchemeError::Empty);
        };

        if !first.is_ascii_alphabetic() {
            return Err(SchemeError::InvalidSyntax);
        }
        if !chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
            return Err(SchemeError::InvalidSyntax);
        }

        let lower = s.to_ascii_lowercase();
        Ok(match lower.as_str() {
            // File
            "file" => Self::File,
            // Web
            "http" => Self::Http,
            "https" => Self::Https,
            "ws" => Self::Ws,
            "wss" => Self::Wss,
            // Messaging
            "mqtt" => Self::Mqtt,
            "mqtts" => Self::Mqtts,
            "amqp" => Self::Amqp,
            "amqps" => Self::Amqps,
            "kafka" => Self::Kafka,
            "nats" => Self::Nats,
            // Databases
            "s3" => Self::S3,
            "pg" | "postgres" | "postgresql" => Self::Pg,
            "redis" => Self::Redis,
            "rediss" => Self::Rediss,
            "mongo" | "mongodb" | "mongodb+srv" => Self::Mongo,
            "mysql" => Self::Mysql,
            // Remote access
            "ftp" => Self::Ftp,
            "ftps" => Self::Ftps,
            "sftp" => Self::Sftp,
            "ssh" => Self::Ssh,
            "ldap" => Self::Ldap,
            "ldaps" => Self::Ldaps,
            "telnet" => Self::Telnet,
            // RPC
            "grpc" => Self::Grpc,
            "grpcs" => Self::Grpcs,
            // Other
            "data" => Self::Data,
            "mailto" => Self::Mailto,
            "tel" => Self::Tel,
            "dns" => Self::Dns,
            "index" => Self::Index,
            _ => Self::Unknown(s.to_string()),
        })
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::File => "file",
            Self::Http => "http",
            Self::Https => "https",
            Self::Ws => "ws",
            Self::Wss => "wss",
            Self::Mqtt => "mqtt",
            Self::Mqtts => "mqtts",
            Self::Amqp => "amqp",
            Self::Amqps => "amqps",
            Self::Kafka => "kafka",
            Self::Nats => "nats",
            Self::S3 => "s3",
            Self::Pg => "pg",
            Self::Redis => "redis",
            Self::Rediss => "rediss",
            Self::Mongo => "mongodb",
            Self::Mysql => "mysql",
            Self::Ftp => "ftp",
            Self::Ftps => "ftps",
            Self::Sftp => "sftp",
            Self::Ssh => "ssh",
            Self::Ldap => "ldap",
            Self::Ldaps => "ldaps",
            Self::Telnet => "telnet",
            Self::Grpc => "grpc",
            Self::Grpcs => "grpcs",
            Self::Data => "data",
            Self::Mailto => "mailto",
            Self::Tel => "tel",
            Self::Dns => "dns",
            Self::Index => "index",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Scheme {
    type Err = SchemeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
