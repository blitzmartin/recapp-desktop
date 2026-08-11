use serde::{Deserialize, Serialize};

use super::LlmProvider;

pub struct DeepSeekProvider {
    api_key: String,
    model: String,
}

impl DeepSeekProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

impl LlmProvider for DeepSeekProvider {
    async fn summarize(
        &self,
        series_title: &str,
        text: &str,
        language: &str,
        num_words: u32,
    ) -> Result<String, String> {
        let prompt = super::build_prompt(series_title, text, language, num_words);

        let response = reqwest::Client::new()
            .post("https://api.deepseek.com/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&ChatRequest {
                model: self.model.clone(),
                messages: vec![ChatMessage {
                    role: "user",
                    content: prompt,
                }],
            })
            .send()
            .await
            .map_err(|e| format!("DeepSeek summarization failed: {e}"))?;

        let response = super::ensure_success(response, "DeepSeek").await?;

        let parsed: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("DeepSeek summarization failed: {e}"))?;

        let summary = parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content.trim().to_string())
            .unwrap_or_default();

        if summary.is_empty() {
            Ok("No summary available".to_string())
        } else {
            Ok(summary)
        }
    }

    async fn translate(
        &self,
        text: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<String, String> {
        let prompt = super::build_translate_prompt(text, source_language, target_language);

        let response = reqwest::Client::new()
            .post("https://api.deepseek.com/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&ChatRequest {
                model: self.model.clone(),
                messages: vec![ChatMessage {
                    role: "user",
                    content: prompt,
                }],
            })
            .send()
            .await
            .map_err(|e| format!("DeepSeek translation failed: {e}"))?;

        let response = super::ensure_success(response, "DeepSeek").await?;

        let parsed: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("DeepSeek translation failed: {e}"))?;

        let translated = parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content.trim().to_string())
            .unwrap_or_default();

        if translated.is_empty() {
            Ok(text.to_string())
        } else {
            Ok(translated)
        }
    }
}
