use serde_json::{json, Value};

use crate::{AiError, AiProvider};

/// Claude via the Anthropic Messages API.
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    max_tokens: u32,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "claude-sonnet-4-6".to_owned(),
            max_tokens: 4096,
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
}

impl AiProvider for AnthropicProvider {
    fn complete(&self, system: &str, user: &str) -> Result<String, AiError> {
        let body = json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "system": system,
            "messages": [{"role": "user", "content": user}]
        });

        // Trim the key: copy-paste often adds leading/trailing whitespace.
        let key = self.api_key.trim();

        let response = ureq::post("https://api.anthropic.com/v1/messages")
            .set("x-api-key", key)
            .set("anthropic-version", "2023-06-01")
            .set("content-type", "application/json")
            .send_json(&body)
            .map_err(|e| {
                // ureq v2 turns 4xx/5xx into Err(Status(code, response)).
                // Extract the API error body so the user sees the real message.
                if let ureq::Error::Status(code, resp) = e {
                    let body: Value = resp.into_json().unwrap_or(Value::Null);
                    let msg = body["error"]["message"]
                        .as_str()
                        .unwrap_or("API rejected the request")
                        .to_owned();
                    AiError::Api(format!("HTTP {code}: {msg}"))
                } else {
                    AiError::Http(e.to_string())
                }
            })?;

        let json: Value = response
            .into_json()
            .map_err(|e| AiError::Http(e.to_string()))?;

        extract_anthropic_text(&json)
    }
}

fn extract_anthropic_text(json: &Value) -> Result<String, AiError> {
    if let Some(error) = json.get("error") {
        let msg = error
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        return Err(AiError::Api(msg.to_owned()));
    }

    json["content"]
        .as_array()
        .and_then(|blocks| blocks.first())
        .and_then(|block| block["text"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| AiError::Api("unexpected Anthropic response shape".to_owned()))
}
