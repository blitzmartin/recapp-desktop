pub mod anthropic;
pub mod deepseek;
pub mod gemini;
pub mod ollama;
pub mod openai;

use serde::{Deserialize, Serialize};

use crate::settings::AppSettings;
use anthropic::AnthropicProvider;
use deepseek::DeepSeekProvider;
use gemini::GeminiProvider;
use ollama::OllamaProvider;
use openai::OpenAiProvider;

pub trait LlmProvider {
    async fn summarize(
        &self,
        series_title: &str,
        text: &str,
        language: &str,
        num_words: u32,
    ) -> Result<String, String>;
}

/// Shared task instructions, without the source synopses (Anthropic sends
/// this as a separate `system` field; other providers prepend it to the
/// user message via `build_prompt`). Centralized so every provider sends
/// the same wording instead of each duplicating it.
///
/// Explicitly asks for one synthesized narrative rather than a per-episode
/// rundown: without this, models tend to default to "in episode 1... in
/// episode 2..." rather than extracting the throughlines that actually
/// matter for someone about to watch the next one.
pub(crate) fn build_instructions(series_title: &str, language: &str, num_words: u32) -> String {
    format!(
        "You are preparing a viewer to watch the next episode of the TV series \"{series_title}\". \
        Based on the episode synopses provided, write a single cohesive recap in {language} of about {num_words} words. \
        Synthesize the most important plot developments and ongoing character/story threads into one flowing narrative aimed at refreshing the viewer's memory. \
        Do not summarize episode by episode or list events one by one, and do not invent details beyond what's in the synopses."
    )
}

/// Full prompt (instructions + source text) for providers whose API takes a
/// single message rather than a separate system field.
pub(crate) fn build_prompt(series_title: &str, text: &str, language: &str, num_words: u32) -> String {
    format!(
        "{}\n\nEpisode synopses:\n\"{text}\"",
        build_instructions(series_title, language, num_words)
    )
}

/// LLM provider selectable by the user. `Ollama` runs locally and needs no
/// API key; the others are remote, token-based services.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LlmProviderKind {
    #[default]
    Ollama,
    OpenAi,
    Anthropic,
    Gemini,
    DeepSeek,
}

impl LlmProviderKind {
    /// Name used as the system keyring key and in error messages.
    pub fn key_name(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::DeepSeek => "deepseek",
        }
    }
}

/// Rust doesn't allow `Box<dyn LlmProvider>` here: the trait uses native
/// `async fn`, which isn't "object safe" without the `async-trait` crate.
/// With a closed set of providers known at compile time, an enum with
/// `match`-based dispatch is the idiomatic alternative: it plays the same
/// role as a Strategy pattern, with no extra dependency and no allocation.
pub enum AnyLlmProvider {
    Ollama(OllamaProvider),
    OpenAi(OpenAiProvider),
    Anthropic(AnthropicProvider),
    Gemini(GeminiProvider),
    DeepSeek(DeepSeekProvider),
}

impl AnyLlmProvider {
    pub async fn summarize(
        &self,
        series_title: &str,
        text: &str,
        language: &str,
        num_words: u32,
    ) -> Result<String, String> {
        match self {
            Self::Ollama(p) => p.summarize(series_title, text, language, num_words).await,
            Self::OpenAi(p) => p.summarize(series_title, text, language, num_words).await,
            Self::Anthropic(p) => p.summarize(series_title, text, language, num_words).await,
            Self::Gemini(p) => p.summarize(series_title, text, language, num_words).await,
            Self::DeepSeek(p) => p.summarize(series_title, text, language, num_words).await,
        }
    }
}

/// Factory: builds the provider configured in settings, fetching the API
/// key from the keyring for remote providers.
pub fn build_provider(settings: &AppSettings) -> Result<AnyLlmProvider, String> {
    match settings.llm_provider {
        LlmProviderKind::Ollama => Ok(AnyLlmProvider::Ollama(OllamaProvider::new(
            settings.ollama_url.clone(),
            settings.ollama_model.clone(),
        ))),
        LlmProviderKind::OpenAi => {
            let key = require_api_key(LlmProviderKind::OpenAi)?;
            Ok(AnyLlmProvider::OpenAi(OpenAiProvider::new(
                key,
                settings.openai_model.clone(),
            )))
        }
        LlmProviderKind::Anthropic => {
            let key = require_api_key(LlmProviderKind::Anthropic)?;
            Ok(AnyLlmProvider::Anthropic(AnthropicProvider::new(
                key,
                settings.anthropic_model.clone(),
            )))
        }
        LlmProviderKind::Gemini => {
            let key = require_api_key(LlmProviderKind::Gemini)?;
            Ok(AnyLlmProvider::Gemini(GeminiProvider::new(
                key,
                settings.gemini_model.clone(),
            )))
        }
        LlmProviderKind::DeepSeek => {
            let key = require_api_key(LlmProviderKind::DeepSeek)?;
            Ok(AnyLlmProvider::DeepSeek(DeepSeekProvider::new(
                key,
                settings.deepseek_model.clone(),
            )))
        }
    }
}

fn require_api_key(provider: LlmProviderKind) -> Result<String, String> {
    crate::secrets::get_api_key(provider)?.ok_or_else(|| {
        format!(
            "No API key configured for {}. Go to Settings.",
            provider.key_name()
        )
    })
}

/// Shared interpretation of error HTTP statuses for remote providers
/// (OpenAI, Anthropic, Gemini, DeepSeek): same meaning (auth/quota/model
/// access/timeout) behind different conventions, so one mapping point
/// instead of repeating it in each provider.
pub(crate) async fn ensure_success(
    response: reqwest::Response,
    provider_name: &str,
) -> Result<reqwest::Response, String> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    // Providers put the actual reason ("model not found", "insufficient
    // quota", ...) in the response body; a status code alone is too coarse
    // to tell "bad model name" apart from other 400s. Best-effort read, so a
    // body-read failure still falls back to a status-based message.
    let body = response.text().await.unwrap_or_default();
    let detail = extract_error_message(&body);

    let message = match status.as_u16() {
        401 | 403 => format!(
            "{provider_name}: invalid or unauthorized API key.{}",
            detail_suffix(&detail)
        ),
        404 => format!(
            "{provider_name}: model not found, or your API key doesn't have access to it. \
             Check the model name in Settings.{}",
            detail_suffix(&detail)
        ),
        400 if detail
            .as_deref()
            .is_some_and(|d| d.to_lowercase().contains("model")) => format!(
            "{provider_name}: the request was rejected because of the model. \
             Check the model name in Settings.{}",
            detail_suffix(&detail)
        ),
        429 => format!(
            "{provider_name}: rate limit or quota exceeded.{}",
            detail_suffix(&detail)
        ),
        408 | 504 => format!("{provider_name}: request timed out."),
        _ => format!(
            "{provider_name}: request failed ({status}).{}",
            detail_suffix(&detail)
        ),
    };
    Err(message)
}

/// Best-effort extraction of a human-readable message from a provider's
/// JSON error body. OpenAI/DeepSeek nest it under `error.message`;
/// Anthropic and Gemini use the same shape or a top-level `message`.
fn extract_error_message(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value
        .get("error")
        .and_then(|e| e.get("message"))
        .or_else(|| value.get("message"))
        .and_then(|m| m.as_str())
        .map(str::to_string)
}

fn detail_suffix(detail: &Option<String>) -> String {
    match detail {
        Some(d) => format!(" Details: {d}"),
        None => String::new(),
    }
}
