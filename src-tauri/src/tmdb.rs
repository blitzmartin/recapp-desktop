use serde::Deserialize;

const BASE_URL: &str = "https://api.themoviedb.org/3";

#[derive(Debug, Deserialize)]
struct SearchResult {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
pub struct EpisodeData {
    pub episode_number: i64,
    pub overview: String,
}

pub async fn search_series(api_key: &str, title: &str) -> Result<Option<i64>, reqwest::Error> {
    let url = format!("{BASE_URL}/search/tv");
    let response: SearchResponse = reqwest::Client::new()
        .get(&url)
        .query(&[("api_key", api_key), ("query", title)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response.results.first().map(|r| r.id))
}

#[derive(Debug, Deserialize)]
struct SeasonEpisode {
    episode_number: i64,
    overview: String,
}

#[derive(Debug, Deserialize)]
struct SeasonResponse {
    #[serde(default)]
    episodes: Vec<SeasonEpisode>,
}

pub struct SeasonEpisodeSynopsis {
    pub episode_number: i64,
    pub synopsis: String,
}

pub async fn get_season(
    api_key: &str,
    series_id: i64,
    season_number: i64,
) -> Result<Vec<SeasonEpisodeSynopsis>, reqwest::Error> {
    let url = format!("{BASE_URL}/tv/{series_id}/season/{season_number}");
    let response: SeasonResponse = reqwest::Client::new()
        .get(&url)
        .query(&[("api_key", api_key)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response
        .episodes
        .into_iter()
        .map(|e| SeasonEpisodeSynopsis {
            episode_number: e.episode_number,
            synopsis: e.overview,
        })
        .collect())
}

pub async fn get_episode(
    api_key: &str,
    series_id: i64,
    season_number: i64,
    episode_number: i64,
) -> Result<EpisodeData, reqwest::Error> {
    let url = format!("{BASE_URL}/tv/{series_id}/season/{season_number}/episode/{episode_number}");
    reqwest::Client::new()
        .get(&url)
        .query(&[("api_key", api_key)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}
