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
import { Textarea } from "@/components/ui/textarea";
import { AppSettings, LengthSetting, PrecisionSetting } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

// Mirrors the Rust constant `llm::DEFAULT_PROMPT_TEMPLATE` (src-tauri/src/llm/mod.rs).
// Keep the two in sync if the default wording changes.
export const DEFAULT_PROMPT_TEMPLATE =
  'You are preparing a viewer to watch the next episode of the TV series "{series_title}". ' +
  "Based on the episode synopses provided, write a single cohesive recap in {language} of about {num_words} words. " +
  "Synthesize the most important plot developments and ongoing character/story threads into one flowing narrative aimed at refreshing the viewer's memory. " +
  "Do not summarize episode by episode or list events one by one, and do not invent details beyond what's in the synopses.";

const precisionLabels: Record<PrecisionSetting["kind"], string> = {
  precise: "Preciso",
  balanced: "Bilanciato",
  creative: "Creativo",
  custom: "Custom",
};

const lengthLabels: Record<LengthSetting["kind"], string> = {
  short: "Breve (~50 parole)",
  medium: "Medio (~100 parole)",
  long: "Lungo (~200 parole)",
  custom: "Custom",
};

const validatePrompt = (template: string): string | null => {
  if (!template.includes("{series_title}") || !template.includes("{language}")) {
    return "Il prompt deve contenere i placeholder {series_title} e {language}.";
  }
  return null;
};

export const AdvancedAiSettings = () => {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [prompt, setPrompt] = useState("");
  const [promptError, setPromptError] = useState<string | null>(null);
  const [status, setStatus] = useState<"idle" | "saving" | "saved">("idle");

  useEffect(() => {
    invoke<AppSettings>("get_settings").then((s) => {
      setSettings(s);
      setPrompt(s.custom_prompt_template ?? DEFAULT_PROMPT_TEMPLATE);
    });
  }, []);

  if (!settings) return null;

  const onPromptChange = (value: string) => {
    setPrompt(value);
    setPromptError(validatePrompt(value));
  };

  const onResetPrompt = () => {
    setPrompt(DEFAULT_PROMPT_TEMPLATE);
    setPromptError(null);
  };

  const onSave = async () => {
    const error = validatePrompt(prompt);
    if (error) {
      setPromptError(error);
      return;
    }
    setStatus("saving");
    const custom_prompt_template = prompt === DEFAULT_PROMPT_TEMPLATE ? null : prompt;
    await invoke("save_settings", {
      settings: { ...settings, custom_prompt_template },
    });
    setStatus("saved");
  };

  return (
    <div className="flex flex-col gap-6 w-[420px]">
      <div className="grid gap-2">
        <Label htmlFor="prompt">Prompt:</Label>
        <p className="text-xs text-muted-foreground">
          Il prompt inviato all'LLM per generare il recap. Usa i placeholder{" "}
          <code>{"{series_title}"}</code>, <code>{"{language}"}</code> e{" "}
          <code>{"{num_words}"}</code>.
        </p>
        <Textarea
          id="prompt"
          rows={8}
          value={prompt}
          aria-invalid={promptError ? true : undefined}
          onChange={(e) => onPromptChange(e.target.value)}
        />
        {promptError && <p className="text-xs text-destructive">{promptError}</p>}
        <Button
          type="button"
          variant="secondary"
          size="sm"
          className="self-start"
          onClick={onResetPrompt}
        >
          Reset to default
        </Button>
      </div>

      <div className="grid gap-2">
        <Label>Precisione:</Label>
        <p className="text-xs text-muted-foreground">
          Controlla quanto l'LLM si attiene ai fatti (Preciso) o quanto elabora liberamente
          (Creativo).
        </p>
        <Select
          value={settings.precision.kind}
          onValueChange={(value) => {
            const kind = value as PrecisionSetting["kind"];
            const precision: PrecisionSetting =
              kind === "custom" ? { kind: "custom", value: 0.5 } : { kind };
            setSettings({ ...settings, precision });
          }}
        >
          <SelectTrigger className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {Object.keys(precisionLabels).map((kind) => (
              <SelectItem key={kind} value={kind}>
                {precisionLabels[kind as PrecisionSetting["kind"]]}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {settings.precision.kind === "custom" && (
          <Input
            type="number"
            min={0}
            max={1}
            step={0.1}
            value={settings.precision.value}
            onChange={(e) =>
              setSettings({
                ...settings,
                precision: { kind: "custom", value: Number(e.target.value) },
              })
            }
          />
        )}
      </div>

      <div className="grid gap-2">
        <Label>Lunghezza:</Label>
        <p className="text-xs text-muted-foreground">
          Lunghezza approssimativa del recap generato, in parole.
        </p>
        <Select
          value={settings.length.kind}
          onValueChange={(value) => {
            const kind = value as LengthSetting["kind"];
            const length: LengthSetting =
              kind === "custom" ? { kind: "custom", value: 100 } : { kind };
            setSettings({ ...settings, length });
          }}
        >
          <SelectTrigger className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {Object.keys(lengthLabels).map((kind) => (
              <SelectItem key={kind} value={kind}>
                {lengthLabels[kind as LengthSetting["kind"]]}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {settings.length.kind === "custom" && (
          <Input
            type="number"
            min={20}
            max={500}
            step={10}
            value={settings.length.value}
            onChange={(e) =>
              setSettings({
                ...settings,
                length: { kind: "custom", value: Number(e.target.value) },
              })
            }
          />
        )}
      </div>

      <Button variant="secondary" onClick={onSave}>
        {status === "saving" ? "Saving..." : status === "saved" ? "Saved" : "Save"}
      </Button>
    </div>
  );
};
