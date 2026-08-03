pub mod anthropic;
pub mod gemini;
pub mod ollama;
pub mod openai;

use serde::{Deserialize, Serialize};

use crate::settings::AppSettings;
use anthropic::AnthropicProvider;
use gemini::GeminiProvider;
use ollama::OllamaProvider;
use openai::OpenAiProvider;

pub trait LlmProvider {
    async fn summarize(&self, text: &str, language: &str, num_words: u32) -> Result<String, String>;
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
}

impl LlmProviderKind {
    /// Name used as the system keyring key and in error messages.
    pub fn key_name(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
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
}

impl AnyLlmProvider {
    pub async fn summarize(
        &self,
        text: &str,
        language: &str,
        num_words: u32,
    ) -> Result<String, String> {
        match self {
            Self::Ollama(p) => p.summarize(text, language, num_words).await,
            Self::OpenAi(p) => p.summarize(text, language, num_words).await,
            Self::Anthropic(p) => p.summarize(text, language, num_words).await,
            Self::Gemini(p) => p.summarize(text, language, num_words).await,
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
/// (OpenAI, Anthropic, Gemini): same meaning (auth/quota/timeout) behind
/// different conventions, so one mapping point instead of repeating it in
/// each provider.
pub(crate) async fn ensure_success(
    response: reqwest::Response,
    provider_name: &str,
) -> Result<reqwest::Response, String> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let message = match status.as_u16() {
        401 | 403 => format!("{provider_name}: invalid or unauthorized API key."),
        429 => format!("{provider_name}: rate limit or quota exceeded."),
        408 | 504 => format!("{provider_name}: request timed out."),
        _ => format!("{provider_name}: request failed ({status})."),
    };
    Err(message)
}
