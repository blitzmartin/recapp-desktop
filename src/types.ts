export type SearchData = {
  series_title: string;
  season_number: number;
  episode_number: number;
  timerange: Timerange;
};

export enum Timerange {
    ALL_SERIES = "all_series",
    ALL_SEASON = "all_season",
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

export enum LlmProviderKind {
  OLLAMA = "ollama",
  OPENAI = "openai",
  ANTHROPIC = "anthropic",
  GEMINI = "gemini",
  DEEPSEEK = "deepseek",
}

export type PrecisionSetting =
  | { kind: "precise" }
  | { kind: "balanced" }
  | { kind: "creative" }
  | { kind: "custom"; value: number };

export type LengthSetting =
  | { kind: "short" }
  | { kind: "medium" }
  | { kind: "long" }
  | { kind: "custom"; value: number };

export type AppSettings = {
  tmdb_api_key: string;
  llm_provider: LlmProviderKind;
  ollama_url: string;
  ollama_model: string;
  openai_model: string;
  anthropic_model: string;
  gemini_model: string;
  deepseek_model: string;
  default_language: Language;
  custom_prompt_template: string | null;
  precision: PrecisionSetting;
  length: LengthSetting;
};
