use keyring::Entry;

use crate::llm::LlmProviderKind;

const SERVICE: &str = "recapp-desktop";

fn entry(provider: LlmProviderKind) -> Result<Entry, String> {
    Entry::new(SERVICE, provider.key_name()).map_err(|e| e.to_string())
}

pub fn get_api_key(provider: LlmProviderKind) -> Result<Option<String>, String> {
    match entry(provider)?.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn set_api_key(provider: LlmProviderKind, key: &str) -> Result<(), String> {
    entry(provider)?.set_password(key).map_err(|e| e.to_string())
}

pub fn delete_api_key(provider: LlmProviderKind) -> Result<(), String> {
    match entry(provider)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn set_llm_api_key(provider: LlmProviderKind, key: String) -> Result<(), String> {
    set_api_key(provider, &key)
}

#[tauri::command]
pub fn has_llm_api_key(provider: LlmProviderKind) -> Result<bool, String> {
    Ok(get_api_key(provider)?.is_some())
}

#[tauri::command]
pub fn delete_llm_api_key(provider: LlmProviderKind) -> Result<(), String> {
    delete_api_key(provider)
}
