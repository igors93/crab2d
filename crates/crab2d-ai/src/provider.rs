use crate::AiError;

/// Pluggable backend for any LLM provider.
pub trait AiProvider: Send + Sync {
    /// Send a system prompt + user message; return the assistant's reply as plain text.
    fn complete(&self, system: &str, user: &str) -> Result<String, AiError>;
}
