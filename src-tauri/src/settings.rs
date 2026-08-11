use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::llm::LlmProviderKind;
use crate::types::Language;

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    "llama3".to_string()
}

fn default_openai_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_anthropic_model() -> String {
    "claude-3-5-haiku-latest".to_string()
}

fn default_gemini_model() -> String {
    "gemini-1.5-flash".to_string()
}

fn default_deepseek_model() -> String {
    "deepseek-chat".to_string()
}

/// How closely the LLM should stick to the source synopses versus elaborate
/// freely, mapped to the provider's `temperature` param. Exposed in the
/// Advanced AI settings screen as presets plus a free-form `Custom` value,
/// so most users never need to know what "temperature" means.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PrecisionSetting {
    Precise,
    Balanced,
    Creative,
    Custom { value: f32 },
}

impl PrecisionSetting {
    pub fn temperature(self) -> f32 {
        match self {
            Self::Precise => 0.1,
            Self::Balanced => 0.5,
            Self::Creative => 0.9,
            Self::Custom { value } => value.clamp(0.0, 1.0),
        }
    }
}

impl Default for PrecisionSetting {
    fn default() -> Self {
        Self::Balanced
    }
}

/// Target length of the generated recap, in words. Same preset + `Custom`
/// shape as `PrecisionSetting`.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum LengthSetting {
    Short,
    Medium,
    Long,
    Custom { value: u32 },
}

impl LengthSetting {
    pub fn num_words(self) -> u32 {
        match self {
            Self::Short => 50,
            Self::Medium => 100,
            Self::Long => 200,
            Self::Custom { value } => value.clamp(20, 500),
        }
    }
}

impl Default for LengthSetting {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    #[serde(default)]
    pub tmdb_api_key: String,
    #[serde(default)]
    pub llm_provider: LlmProviderKind,
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,
    #[serde(default = "default_openai_model")]
    pub openai_model: String,
    #[serde(default = "default_anthropic_model")]
    pub anthropic_model: String,
    #[serde(default = "default_gemini_model")]
    pub gemini_model: String,
    #[serde(default = "default_deepseek_model")]
    pub deepseek_model: String,
    #[serde(default)]
    pub default_language: Language,
    /// `None` means "use the built-in default prompt" (see
    /// `llm::DEFAULT_PROMPT_TEMPLATE`), so future changes to the default
    /// wording still apply to users who never customized it.
    #[serde(default)]
    pub custom_prompt_template: Option<String>,
    #[serde(default)]
    pub precision: PrecisionSetting,
    #[serde(default)]
    pub length: LengthSetting,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            tmdb_api_key: String::new(),
            llm_provider: LlmProviderKind::default(),
            ollama_url: default_ollama_url(),
            ollama_model: default_ollama_model(),
            openai_model: default_openai_model(),
            anthropic_model: default_anthropic_model(),
            gemini_model: default_gemini_model(),
            deepseek_model: default_deepseek_model(),
            default_language: Language::En,
            custom_prompt_template: None,
            precision: PrecisionSetting::default(),
            length: LengthSetting::default(),
        }
    }
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

pub fn load(app: &AppHandle) -> AppSettings {
    let Ok(path) = config_path(app) else {
        return AppSettings::default();
    };
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return AppSettings::default();
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> AppSettings {
    load(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    if let Some(template) = &settings.custom_prompt_template {
        crate::llm::validate_prompt_template(template)?;
    }
    save(&app, &settings)
}
