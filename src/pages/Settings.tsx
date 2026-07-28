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
import { AppSettings, Language } from "@/types";
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

export const Settings = () => {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [status, setStatus] = useState<"idle" | "saving" | "saved">("idle");
  const [showApiKey, setShowApiKey] = useState(false);

  useEffect(() => {
    invoke<AppSettings>("get_settings").then(setSettings);
  }, []);

  if (!settings) return null;

  const onSave = async () => {
    setStatus("saving");
    await invoke("save_settings", { settings });
    setStatus("saved");
  };

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
        <Label htmlFor="ollama_url">Ollama URL:</Label>
        <Input
          id="ollama_url"
          value={settings.ollama_url}
          onChange={(e) =>
            setSettings({ ...settings, ollama_url: e.target.value })
          }
        />
      </div>
      <div className="grid gap-2">
        <Label htmlFor="ollama_model">Ollama Model:</Label>
        <Input
          id="ollama_model"
          value={settings.ollama_model}
          onChange={(e) =>
            setSettings({ ...settings, ollama_model: e.target.value })
          }
        />
      </div>
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
    </div>
  );
};
