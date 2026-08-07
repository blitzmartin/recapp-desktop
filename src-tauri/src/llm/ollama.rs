use serde::{Deserialize, Serialize};

use super::LlmProvider;

pub struct OllamaProvider {
    url: String,
    model: String,
}

impl OllamaProvider {
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

#[derive(Deserialize)]
struct TagsModel {
    name: String,
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TagsModel>,
}

/// Lists models currently pulled in the local Ollama install, so the
/// Settings UI can offer a picker instead of free text. Ollama exposes this
/// over its own local HTTP API, no API key involved.
#[tauri::command]
pub async fn list_ollama_models(url: String) -> Result<Vec<String>, String> {
    let response = reqwest::Client::new()
        .get(format!("{url}/api/tags"))
        .send()
        .await
        .map_err(|e| format!("Could not reach Ollama at {url}: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Could not reach Ollama at {url}: {e}"))?;

    let parsed: TagsResponse = response
        .json()
        .await
        .map_err(|e| format!("Could not parse Ollama model list: {e}"))?;

    Ok(parsed.models.into_iter().map(|m| m.name).collect())
}

impl LlmProvider for OllamaProvider {
    async fn summarize(
        &self,
        series_title: &str,
        text: &str,
        language: &str,
        num_words: u32,
    ) -> Result<String, String> {
        let prompt = super::build_prompt(series_title, text, language, num_words);

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
            .map_err(|e| format!("Ollama summarization failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("Ollama summarization failed: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Ollama summarization failed: {e}"))?;

        let summary = response.message.content.trim().to_string();
        if summary.is_empty() {
            Ok("No summary available".to_string())
        } else {
            Ok(summary)
        }
    }
}
