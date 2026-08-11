use serde::{Deserialize, Serialize};

use super::{LlmProvider, SummarizeOptions};

pub struct GeminiProvider {
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f32,
}

#[derive(Serialize)]
struct GenerateRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "generationConfig")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct GenerateResponse {
    candidates: Vec<Candidate>,
}

impl LlmProvider for GeminiProvider {
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

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = reqwest::Client::new()
            .post(url)
            .json(&GenerateRequest {
                contents: vec![Content {
                    parts: vec![Part { text: prompt }],
                }],
                generation_config: Some(GenerationConfig {
                    temperature: options.temperature,
                }),
            })
            .send()
            .await
            .map_err(|e| format!("Gemini summarization failed: {e}"))?;

        let response = super::ensure_success(response, "Gemini").await?;

        let parsed: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Gemini summarization failed: {e}"))?;

        let summary = parsed
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text.trim().to_string())
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

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = reqwest::Client::new()
            .post(url)
            .json(&GenerateRequest {
                contents: vec![Content {
                    parts: vec![Part { text: prompt }],
                }],
                generation_config: None,
            })
            .send()
            .await
            .map_err(|e| format!("Gemini translation failed: {e}"))?;

        let response = super::ensure_success(response, "Gemini").await?;

        let parsed: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Gemini translation failed: {e}"))?;

        let translated = parsed
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text.trim().to_string())
            .unwrap_or_default();

        if translated.is_empty() {
            Ok(text.to_string())
        } else {
            Ok(translated)
        }
    }
}
