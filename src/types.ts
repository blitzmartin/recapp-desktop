export type SearchData = {
  series_title: string;
  season_number: number;
  episode_number: number;
  timerange: Timerange;
};

export enum Timerange {
    ALL_SERIES = "all_series",
    ALL_SEASON = "all_season",
    PREV_EPISODE = "prev_episode",
    THIS_EPISODE = "this_episode"
}

export enum Language {
    EN = "en",
    IT = "it",
    FR = "fr",
    DE = "de",
    ES = "es",
    PT = "pt",
    RU = "ru",
    ZH = "zh",
    JA = "ja"
}

export type BackendResponse = {
  episodes?: Episode[];
  summary?: string;
  error?: string;
};

export type Episode = {
  synopsis: string;
  season_number: number;
  episode_number: number;
};

export type AppSettings = {
  tmdb_api_key: string;
  ollama_url: string;
  ollama_model: string;
  default_language: Language;
};
