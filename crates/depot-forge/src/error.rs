use std::fmt;

#[derive(Debug)]
pub enum ForgeError {
    Http(u16, String),
    Network(String),
    Json(String),
    MissingAuth(String),
    Api(String),
}

impl fmt::Display for ForgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(status, msg) => write!(f, "HTTP {}: {}", status, msg),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Json(msg) => write!(f, "JSON serialization error: {}", msg),
            Self::MissingAuth(msg) => write!(f, "Missing authentication: {}", msg),
            Self::Api(msg) => write!(f, "API error: {}", msg),
        }
    }
}

impl std::error::Error for ForgeError {}
