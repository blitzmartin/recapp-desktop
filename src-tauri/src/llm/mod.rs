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

/// Options controlling how `summarize` generates the recap: target length,
/// creativity (`temperature`), and an optional user-provided prompt
/// template overriding `DEFAULT_PROMPT_TEMPLATE`.
pub struct SummarizeOptions<'a> {
    pub num_words: u32,
    pub temperature: f32,
    pub custom_prompt_template: Option<&'a str>,
}

pub trait LlmProvider {
    async fn summarize(
        &self,
        series_title: &str,
        text: &str,
        language: &str,
        options: &SummarizeOptions,
    ) -> Result<String, String>;

    async fn translate(
        &self,
        text: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<String, String>;
}

/// Default task instructions template, without the source synopses
/// (Anthropic sends this as a separate `system` field; other providers
/// prepend it to the user message via `build_prompt`). Centralized so every
/// provider sends the same wording instead of each duplicating it, and
/// exposed as the baseline the "Reset to default" action in Advanced AI
/// settings restores.
///
/// Explicitly asks for one synthesized narrative rather than a per-episode
/// rundown: without this, models tend to default to "in episode 1... in
/// episode 2..." rather than extracting the throughlines that actually
/// matter for someone about to watch the next one.
pub const DEFAULT_PROMPT_TEMPLATE: &str = "You are preparing a viewer to watch the next episode of the TV series \"{series_title}\". \
Based on the episode synopses provided, write a single cohesive recap in {language} of about {num_words} words. \
Synthesize the most important plot developments and ongoing character/story threads into one flowing narrative aimed at refreshing the viewer's memory. \
Do not summarize episode by episode or list events one by one, and do not invent details beyond what's in the synopses.";

/// A user-editable template must still reference these so the generated
/// recap stays grounded in the right series and language; `{num_words}` is
/// optional since the target length is also enforced via `SummarizeOptions`.
pub fn validate_prompt_template(template: &str) -> Result<(), String> {
    if !template.contains("{series_title}") || !template.contains("{language}") {
        return Err(
            "The prompt must contain the {series_title} and {language} placeholders.".to_string(),
        );
    }
    Ok(())
}

fn render_template(template: &str, series_title: &str, language: &str, num_words: u32) -> String {
    template
        .replace("{series_title}", series_title)
        .replace("{language}", language)
        .replace("{num_words}", &num_words.to_string())
}

pub(crate) fn build_instructions(
    series_title: &str,
    language: &str,
    num_words: u32,
    custom_template: Option<&str>,
) -> String {
    let template = custom_template.unwrap_or(DEFAULT_PROMPT_TEMPLATE);
    render_template(template, series_title, language, num_words)
}

/// Full prompt (instructions + source text) for providers whose API takes a
/// single message rather than a separate system field.
pub(crate) fn build_prompt(
    series_title: &str,
    text: &str,
    language: &str,
    num_words: u32,
    custom_template: Option<&str>,
) -> String {
    format!(
        "{}\n\nEpisode synopses:\n\"{text}\"",
        build_instructions(series_title, language, num_words, custom_template)
    )
}

/// Prompt used by every provider's `translate`, so the wording (and the
/// "no explanation" instruction that keeps the reply free of preamble) is
/// defined once instead of duplicated per provider.
pub(crate) fn build_translate_prompt(text: &str, source_language: &str, target_language: &str) -> String {
    format!(
        "Translate the following text from {source_language} to {target_language}. Provide only the translated text without any explanation:\n\n\"{text}\""
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
        options: &SummarizeOptions<'_>,
    ) -> Result<String, String> {
        match self {
            Self::Ollama(p) => p.summarize(series_title, text, language, options).await,
            Self::OpenAi(p) => p.summarize(series_title, text, language, options).await,
            Self::Anthropic(p) => p.summarize(series_title, text, language, options).await,
            Self::Gemini(p) => p.summarize(series_title, text, language, options).await,
            Self::DeepSeek(p) => p.summarize(series_title, text, language, options).await,
        }
    }

    pub async fn translate(
        &self,
        text: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<String, String> {
        match self {
            Self::Ollama(p) => p.translate(text, source_language, target_language).await,
            Self::OpenAi(p) => p.translate(text, source_language, target_language).await,
            Self::Anthropic(p) => p.translate(text, source_language, target_language).await,
            Self::Gemini(p) => p.translate(text, source_language, target_language).await,
            Self::DeepSeek(p) => p.translate(text, source_language, target_language).await,
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
