import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { AppSettings, Language, LlmProviderKind } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { Eye, EyeOff } from "lucide-react";
import { useEffect, useState } from "react";

const languageLabels: Record<Language, string> = {
  [Language.EN]: "English",
  [Language.IT]: "Italian",
  [Language.FR]: "French",
  [Language.DE]: "German",
  [Language.ES]: "Spanish",
  [Language.PT]: "Portuguese",
  [Language.RU]: "Russian",
  [Language.ZH]: "Chinese",
  [Language.JA]: "Japanese",
};

const providerLabels: Record<LlmProviderKind, string> = {
  [LlmProviderKind.OLLAMA]: "Ollama (local)",
  [LlmProviderKind.OPENAI]: "OpenAI",
  [LlmProviderKind.ANTHROPIC]: "Anthropic",
  [LlmProviderKind.GEMINI]: "Gemini",
  [LlmProviderKind.DEEPSEEK]: "DeepSeek",
};

// Every remote provider stores its model in its own `AppSettings` field, so
// each entry here maps the selected provider to the field it should edit.
const providerModelField: Record<LlmProviderKind, keyof AppSettings | null> = {
  [LlmProviderKind.OLLAMA]: "ollama_model",
  [LlmProviderKind.OPENAI]: "openai_model",
  [LlmProviderKind.ANTHROPIC]: "anthropic_model",
  [LlmProviderKind.GEMINI]: "gemini_model",
  [LlmProviderKind.DEEPSEEK]: "deepseek_model",
};

const requiresApiKey = (provider: LlmProviderKind) =>
  provider !== LlmProviderKind.OLLAMA;

// Curated list of common models per remote provider. There's no reliable
// API to list "models this key can actually use" up front (see Settings.tsx
// history), so this is a maintained shortlist rather than a live query.
// Ollama is handled separately: it's queried live from the local install.
const CUSTOM_MODEL = "__custom__";

const staticModelOptions: Partial<Record<LlmProviderKind, string[]>> = {
  [LlmProviderKind.OPENAI]: ["gpt-4o", "gpt-4o-mini", "gpt-4.1", "gpt-4.1-mini", "o3-mini"],
  [LlmProviderKind.ANTHROPIC]: [
    "claude-opus-5",
    "claude-sonnet-5",
    "claude-haiku-4-5-20251001",
    "claude-3-5-haiku-latest",
  ],
  [LlmProviderKind.GEMINI]: ["gemini-1.5-flash", "gemini-1.5-pro", "gemini-2.0-flash"],
  [LlmProviderKind.DEEPSEEK]: ["deepseek-chat", "deepseek-reasoner"],
};

type SettingsProps = {
  onNavigateAdvanced: () => void;
};

export const Settings = ({ onNavigateAdvanced }: SettingsProps) => {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [status, setStatus] = useState<"idle" | "saving" | "saved">("idle");
  const [showApiKey, setShowApiKey] = useState(false);
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [hasApiKey, setHasApiKey] = useState(false);
  const [apiKeyStatus, setApiKeyStatus] = useState<"idle" | "saving" | "saved">("idle");
  const [ollamaModels, setOllamaModels] = useState<string[]>([]);
  const [ollamaModelsError, setOllamaModelsError] = useState("");
  const [customModel, setCustomModel] = useState(false);

  useEffect(() => {
    invoke<AppSettings>("get_settings").then(setSettings);
  }, []);

  useEffect(() => {
    if (!settings || settings.llm_provider !== LlmProviderKind.OLLAMA) return;
    setOllamaModelsError("");
    invoke<string[]>("list_ollama_models", { url: settings.ollama_url })
      .then(setOllamaModels)
      .catch((err) => {
        setOllamaModels([]);
        setOllamaModelsError(
          typeof err === "string" ? err : "Could not reach Ollama to list installed models.",
        );
      });
  }, [settings?.llm_provider, settings?.ollama_url]);

  useEffect(() => {
    if (!settings || !requiresApiKey(settings.llm_provider)) {
      setHasApiKey(false);
      return;
    }
    invoke<boolean>("has_llm_api_key", { provider: settings.llm_provider }).then(
      setHasApiKey,
    );
    setApiKeyInput("");
  }, [settings?.llm_provider]);

  useEffect(() => {
    setCustomModel(false);
  }, [settings?.llm_provider]);

  if (!settings) return null;

  const onSave = async () => {
    setStatus("saving");
    await invoke("save_settings", { settings });
    setStatus("saved");
  };

  const onSaveApiKey = async () => {
    if (!apiKeyInput) return;
    setApiKeyStatus("saving");
    await invoke("set_llm_api_key", {
      provider: settings.llm_provider,
      key: apiKeyInput,
    });
    setApiKeyInput("");
    setHasApiKey(true);
    setApiKeyStatus("saved");
  };

  const modelField = providerModelField[settings.llm_provider];
  const modelOptions =
    settings.llm_provider === LlmProviderKind.OLLAMA
      ? ollamaModels
      : staticModelOptions[settings.llm_provider] ?? [];
  const currentModel = modelField ? (settings[modelField] as string) : "";
  const isKnownModel = modelOptions.includes(currentModel);
  const showCustomInput = customModel || (currentModel !== "" && !isKnownModel);

  return (
    <div className="flex flex-col gap-6 w-[280px]">
      <div className="grid gap-2">
        <Label htmlFor="tmdb_api_key">TMDB API Key:</Label>
        <div className="relative">
          <Input
            id="tmdb_api_key"
            type={showApiKey ? "text" : "password"}
            className="pr-9"
            value={settings.tmdb_api_key}
            onChange={(e) =>
              setSettings({ ...settings, tmdb_api_key: e.target.value })
            }
          />
          <button
            type="button"
            aria-label={showApiKey ? "Hide API key" : "Show API key"}
            onClick={() => setShowApiKey(!showApiKey)}
            className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
          >
            {showApiKey ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
          </button>
        </div>
      </div>
      <div className="grid gap-2">
        <Label>LLM Provider:</Label>
        <Select
          value={settings.llm_provider}
          onValueChange={(value) =>
            setSettings({ ...settings, llm_provider: value as LlmProviderKind })
          }
        >
          <SelectTrigger className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {Object.values(LlmProviderKind).map((provider) => (
              <SelectItem key={provider} value={provider}>
                {providerLabels[provider]}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      {settings.llm_provider === LlmProviderKind.OLLAMA && (
        <div className="grid gap-2">
          <Label htmlFor="ollama_url">Ollama URL:</Label>
          <Input
            id="ollama_url"
            value={settings.ollama_url}
            onChange={(e) =>
              setSettings({ ...settings, ollama_url: e.target.value })
            }
          />
        </div>
      )}
      {modelField && (
        <div className="grid gap-2">
          <Label htmlFor="llm_model">Model:</Label>
          {showCustomInput ? (
            <>
              <Input
                id="llm_model"
                placeholder="Model name"
                value={currentModel}
                onChange={(e) =>
                  setSettings({ ...settings, [modelField]: e.target.value })
                }
              />
              {modelOptions.length > 0 && (
                <button
                  type="button"
                  className="text-xs text-muted-foreground hover:text-foreground text-left"
                  onClick={() => {
                    setCustomModel(false);
                    setSettings({ ...settings, [modelField]: modelOptions[0] });
                  }}
                >
                  Choose from list instead
                </button>
              )}
            </>
          ) : (
            <Select
              value={currentModel}
              onValueChange={(value) => {
                if (value === CUSTOM_MODEL) {
                  setCustomModel(true);
                  setSettings({ ...settings, [modelField]: "" });
                  return;
                }
                setSettings({ ...settings, [modelField]: value });
              }}
            >
              <SelectTrigger className="w-full" id="llm_model">
                <SelectValue placeholder="Select a model" />
              </SelectTrigger>
              <SelectContent>
                {modelOptions.map((model) => (
                  <SelectItem key={model} value={model}>
                    {model}
                  </SelectItem>
                ))}
                <SelectItem value={CUSTOM_MODEL}>Custom…</SelectItem>
              </SelectContent>
            </Select>
          )}
          {settings.llm_provider === LlmProviderKind.OLLAMA && ollamaModelsError && (
            <p className="text-xs text-destructive">{ollamaModelsError}</p>
          )}
          {settings.llm_provider === LlmProviderKind.OLLAMA &&
            !ollamaModelsError &&
            ollamaModels.length === 0 && (
              <p className="text-xs text-muted-foreground">
                No models found. Pull one with `ollama pull &lt;model&gt;`.
              </p>
            )}
        </div>
      )}
      {requiresApiKey(settings.llm_provider) && (
        <div className="grid gap-2">
          <Label htmlFor="llm_api_key">
            {providerLabels[settings.llm_provider]} API Key:
          </Label>
          <Input
            id="llm_api_key"
            type="password"
            placeholder={hasApiKey ? "Key is set" : "No key set"}
            value={apiKeyInput}
            onChange={(e) => setApiKeyInput(e.target.value)}
          />
          <Button
            type="button"
            variant="secondary"
            size="sm"
            disabled={!apiKeyInput}
            onClick={onSaveApiKey}
          >
            {apiKeyStatus === "saving"
              ? "Saving..."
              : apiKeyStatus === "saved"
                ? "Saved"
                : "Save API Key"}
          </Button>
        </div>
      )}
      <div className="grid gap-2">
        <Label>Default Language:</Label>
        <Select
          value={settings.default_language}
          onValueChange={(value) =>
            setSettings({ ...settings, default_language: value as Language })
          }
        >
          <SelectTrigger className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {Object.values(Language).map((language) => (
              <SelectItem key={language} value={language}>
                {languageLabels[language]}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <Button variant="secondary" onClick={onSave}>
        {status === "saving"
          ? "Saving..."
          : status === "saved"
            ? "Saved"
            : "Save"}
      </Button>
      <button
        type="button"
        className="text-sm text-muted-foreground hover:text-foreground text-left"
        onClick={onNavigateAdvanced}
      >
        Advanced AI settings →
      </button>
    </div>
  );
};
