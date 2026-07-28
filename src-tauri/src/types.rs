use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Timerange {
    AllSeries,
    AllSeason,
    PrevEpisode,
    ThisEpisode,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    En,
    It,
    Fr,
    De,
    Es,
    Pt,
    Ru,
    Zh,
    Ja,
}

impl Language {
    pub fn full_name(&self) -> &'static str {
        match self {
            Language::En => "English",
            Language::It => "Italian",
            Language::Fr => "French",
            Language::De => "German",
            Language::Es => "Spanish",
            Language::Pt => "Portuguese",
            Language::Ru => "Russian",
            Language::Zh => "Chinese",
            Language::Ja => "Japanese",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchData {
    pub series_title: String,
    pub season_number: i64,
    pub episode_number: i64,
    pub timerange: Timerange,
}

#[derive(Debug, Serialize, Clone)]
pub struct Episode {
    pub synopsis: String,
    pub season_number: i64,
    pub episode_number: i64,
}

#[derive(Debug, Serialize, Default)]
pub struct BackendResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episodes: Option<Vec<Episode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
