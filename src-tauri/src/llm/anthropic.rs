use serde::{Deserialize, Serialize};

use super::LlmProvider;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

#[derive(Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    system: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
}

impl LlmProvider for AnthropicProvider {
    async fn summarize(
        &self,
        series_title: &str,
        text: &str,
        language: &str,
        num_words: u32,
    ) -> Result<String, String> {
        let system = super::build_instructions(series_title, language, num_words);

        let response = reqwest::Client::new()
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&MessagesRequest {
                model: self.model.clone(),
                system,
                messages: vec![Message {
                    role: "user",
                    content: text.to_string(),
                }],
                max_tokens: 1024,
            })
            .send()
            .await
            .map_err(|e| format!("Anthropic summarization failed: {e}"))?;

        let response = super::ensure_success(response, "Anthropic").await?;

        let parsed: MessagesResponse = response
            .json()
            .await
            .map_err(|e| format!("Anthropic summarization failed: {e}"))?;

        let summary = parsed
            .content
            .into_iter()
            .next()
            .map(|c| c.text.trim().to_string())
            .unwrap_or_default();

        if summary.is_empty() {
            Ok("No summary available".to_string())
        } else {
            Ok(summary)
        }
    }
}
