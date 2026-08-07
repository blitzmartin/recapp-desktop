use crate::llm;
use crate::settings::AppSettings;
use crate::tmdb;
use crate::translator::{ollama::OllamaTranslator, Translator};
use crate::types::{BackendResponse, Episode, Language, SearchData, Timerange};

async fn translate_if_needed(
    text: String,
    language: Language,
    settings: &AppSettings,
) -> Result<String, String> {
    if language == Language::En {
        return Ok(text);
    }

    let translator = OllamaTranslator::new(settings.ollama_url.clone(), settings.ollama_model.clone());
    translator
        .translate(&text, Language::En.full_name(), language.full_name())
        .await
        .map_err(|e| {
            eprintln!("[generate_recap] errore traduzione: {e}");
            e
        })
}

#[tauri::command]
pub async fn generate_recap(
    app: tauri::AppHandle,
    search_data: SearchData,
) -> Result<BackendResponse, String> {
    eprintln!("[generate_recap] received: {search_data:?}");

    let settings = crate::settings::load(&app);

    if settings.tmdb_api_key.is_empty() {
        eprintln!("[generate_recap] TMDB API key non configurata nei Settings");
        return Err("TMDB API key non configurata. Vai su Settings.".to_string());
    }
    let api_key = settings.tmdb_api_key.clone();

    let series_id = tmdb::search_series(&api_key, &search_data.series_title)
        .await
        .map_err(|e| {
            eprintln!("[generate_recap] errore ricerca serie: {e}");
            e.to_string()
        })?
        .ok_or_else(|| {
            eprintln!("[generate_recap] serie non trovata: {}", search_data.series_title);
            "TV Series not found".to_string()
        })?;

    match search_data.timerange {
        Timerange::ThisEpisode | Timerange::PrevEpisode => {
            let episode_number = match search_data.timerange {
                Timerange::PrevEpisode => search_data.episode_number - 1,
                _ => search_data.episode_number,
            };

            let episode_data = tmdb::get_episode(
                &api_key,
                series_id,
                search_data.season_number,
                episode_number,
            )
            .await
            .map_err(|e| {
                eprintln!("[generate_recap] errore recupero episodio: {e}");
                e.to_string()
            })?;

            let episode = Episode {
                synopsis: episode_data.overview,
                season_number: search_data.season_number,
                episode_number: episode_data.episode_number,
            };

            let summary =
                translate_if_needed(episode.synopsis.clone(), settings.default_language, &settings)
                    .await?;

            Ok(BackendResponse {
                episodes: Some(vec![episode]),
                summary: Some(summary),
                error: None,
            })
        }
        Timerange::AllSeries | Timerange::AllSeason => {
            let mut episodes: Vec<Episode> = Vec::new();

            if search_data.timerange == Timerange::AllSeries {
                for season_number in 1..search_data.season_number {
                    let season_episodes = tmdb::get_season(&api_key, series_id, season_number)
                        .await
                        .map_err(|e| {
                            eprintln!("[generate_recap] errore recupero stagione {season_number}: {e}");
                            e.to_string()
                        })?;

                    episodes.extend(season_episodes.into_iter().map(|e| Episode {
                        synopsis: e.synopsis,
                        season_number,
                        episode_number: e.episode_number,
                    }));
                }
            }

            let current_season_episodes = tmdb::get_season(
                &api_key,
                series_id,
                search_data.season_number,
            )
            .await
            .map_err(|e| {
                eprintln!("[generate_recap] errore recupero stagione corrente: {e}");
                e.to_string()
            })?;

            episodes.extend(
                current_season_episodes
                    .into_iter()
                    .filter(|e| e.episode_number < search_data.episode_number)
                    .map(|e| Episode {
                        synopsis: e.synopsis,
                        season_number: search_data.season_number,
                        episode_number: e.episode_number,
                    }),
            );

            let text_to_recap = episodes
                .iter()
                .map(|e| e.synopsis.as_str())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ");

            let summary = if text_to_recap.is_empty() {
                "No summary available".to_string()
            } else {
                let provider = llm::build_provider(&settings)?;
                let recapped_text = provider
                    .summarize(&search_data.series_title, &text_to_recap, "en", 100)
                    .await
                    .map_err(|e| {
                        eprintln!("[generate_recap] errore riassunto LLM: {e}");
                        e
                    })?;
                translate_if_needed(recapped_text, settings.default_language, &settings).await?
            };

            Ok(BackendResponse {
                episodes: Some(episodes),
                summary: Some(summary),
                error: None,
            })
        }
    }
}
