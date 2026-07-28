use serde::{Deserialize, Serialize};

use super::Translator;

pub struct OllamaTranslator {
    url: String,
    model: String,
}

impl OllamaTranslator {
    pub fn new(url: String, model: String) -> Self {
        Self { url, model }
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
    stream: bool,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ChatResponseMessage,
}

impl Translator for OllamaTranslator {
    async fn translate(
        &self,
        text: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<String, String> {
        let prompt = format!(
            "Translate the following text from {source_language} to {target_language}. Provide only the translated text without any explanation:\n\n\"{text}\""
        );

        let response: ChatResponse = reqwest::Client::new()
            .post(format!("{}/api/chat", self.url))
            .json(&ChatRequest {
                model: self.model.clone(),
                messages: vec![ChatMessage {
                    role: "user",
                    content: prompt,
                }],
                stream: false,
            })
            .send()
            .await
            .map_err(|e| format!("Ollama translation failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("Ollama translation failed: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Ollama translation failed: {e}"))?;

        let translated = response.message.content.trim().to_string();
        if translated.is_empty() {
            Ok(text.to_string())
        } else {
            Ok(translated)
        }
    }
}
