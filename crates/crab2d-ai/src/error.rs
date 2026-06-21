use std::fmt;

#[derive(Debug)]
pub enum AiError {
    Http(String),
    Api(String),
    NoJsonFound,
    Deserialize(serde_json::Error),
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(msg) => write!(f, "HTTP error: {msg}"),
            Self::Api(msg) => write!(f, "AI provider error: {msg}"),
            Self::NoJsonFound => f.write_str("AI response contained no JSON block"),
            Self::Deserialize(e) => write!(f, "Failed to parse AI response: {e}"),
        }
    }
}

impl std::error::Error for AiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Deserialize(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for AiError {
    fn from(e: serde_json::Error) -> Self {
        Self::Deserialize(e)
    }
}
