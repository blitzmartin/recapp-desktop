mod commands;
mod llm;
mod secrets;
mod settings;
mod tmdb;
mod translator;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::generate_recap,
            settings::get_settings,
            settings::save_settings,
            secrets::set_llm_api_key,
            secrets::has_llm_api_key,
            secrets::delete_llm_api_key,
            llm::ollama::list_ollama_models
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
