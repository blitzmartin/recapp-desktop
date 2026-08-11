use serde::{Deserialize, Serialize};

use super::{LlmProvider, SummarizeOptions};

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
struct ChatOptions {
    temperature: f32,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<ChatOptions>,
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
        options: &SummarizeOptions<'_>,
    ) -> Result<String, String> {
        let prompt = super::build_prompt(
            series_title,
            text,
            language,
            options.num_words,
            options.custom_prompt_template,
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
                options: Some(ChatOptions {
                    temperature: options.temperature,
                }),
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

    async fn translate(
        &self,
        text: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<String, String> {
        let prompt = super::build_translate_prompt(text, source_language, target_language);

        let response: ChatResponse = reqwest::Client::new()
            .post(format!("{}/api/chat", self.url))
            .json(&ChatRequest {
                model: self.model.clone(),
                messages: vec![ChatMessage {
                    role: "user",
                    content: prompt,
                }],
                stream: false,
                options: None,
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
