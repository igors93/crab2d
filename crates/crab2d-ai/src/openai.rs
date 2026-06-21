use serde_json::{json, Value};

use crate::{AiError, AiProvider};

/// GPT / Codex via the OpenAI Chat Completions API.
pub struct OpenAiProvider {
    api_key: String,
    model: String,
    max_tokens: u32,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "gpt-4o".to_owned(),
            max_tokens: 4096,
            base_url: "https://api.openai.com/v1".to_owned(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Override the base URL to point at any OpenAI-compatible endpoint
    /// (e.g. a local Ollama server, Azure OpenAI, etc.).
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

impl AiProvider for OpenAiProvider {
    fn complete(&self, system: &str, user: &str) -> Result<String, AiError> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user",   "content": user}
            ]
        });

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("content-type", "application/json")
            .send_json(&body)
            .map_err(|e| AiError::Http(e.to_string()))?;

        let json: Value = response
            .into_json()
            .map_err(|e| AiError::Http(e.to_string()))?;

        extract_openai_text(&json)
    }
}

fn extract_openai_text(json: &Value) -> Result<String, AiError> {
    if let Some(error) = json.get("error") {
        let msg = error
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        return Err(AiError::Api(msg.to_owned()));
    }

    json["choices"]
        .as_array()
        .and_then(|choices| choices.first())
        .and_then(|choice| choice["message"]["content"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| AiError::Api("unexpected OpenAI response shape".to_owned()))
}
