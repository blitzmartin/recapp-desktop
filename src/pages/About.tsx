import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "react";

export const About = () => {
  const [version, setVersion] = useState("");

  useEffect(() => {
    getVersion().then(setVersion);
  }, []);

  return (
    <div className="flex flex-col gap-6 w-full max-w-lg text-sm">
      {version && (
        <p className="text-xs text-muted-foreground text-center -mt-2">
          Version {version}
        </p>
      )}
      <section className="flex flex-col gap-2">
        <h2 className="text-lg font-semibold text-primary">What Recapp does</h2>
        <p className="text-muted-foreground">
          Recapp generates a quick recap of a TV show so you can jump back in
          without rewatching everything. Pick a series, a season/episode, and
          how much history you want summarized — Recapp pulls the episode
          synopses from TMDB, optionally asks an LLM to condense them into a
          single narrative, and translates the result if needed.
        </p>
      </section>

      <section className="flex flex-col gap-2">
        <h2 className="text-lg font-semibold text-primary">
          Choosing a timerange
        </h2>
        <ul className="flex flex-col gap-2 text-muted-foreground">
          <li>
            <span className="font-medium text-foreground">
              All series until now
            </span>{" "}
            — summarizes every previous season plus the current season up to
            your episode. Great for "I stopped watching a while ago," but with
            many seasons the LLM is condensing a lot of plot into a short recap,
            so expect broader strokes and fewer details.
          </li>
          <li>
            <span className="font-medium text-foreground">
              Season until now
            </span>{" "}
            — summarizes only the current season up to your episode. More detail
            per event than the full-series option, since there's less to
            compress.
          </li>
          <li>
            <span className="font-medium text-foreground">This Episode</span> —
            uses the single episode's TMDB synopsis directly, no LLM involved.
            Fastest option and the most detail, but limited to that one episode.
          </li>
        </ul>
      </section>

      <section className="flex flex-col gap-2">
        <h2 className="text-lg font-semibold text-primary">Choosing a model</h2>
        <p className="text-muted-foreground">
          Model choice affects both speed and recap quality:
        </p>
        <ul className="flex flex-col gap-2 text-muted-foreground">
          <li>
            <span className="font-medium text-foreground">Ollama (local)</span>{" "}
            — runs on your machine, no API key or internet needed. Speed and
            quality depend on your hardware and the model you've pulled; smaller
            local models may produce rougher summaries than a hosted frontier
            model.
          </li>
          <li>
            <span className="font-medium text-foreground">
              OpenAI / Anthropic / Gemini / DeepSeek
            </span>{" "}
            — hosted models, generally faster and higher quality, but require an
            API key and send episode synopses to that provider.
          </li>
        </ul>
      </section>

      <p className="text-xs text-muted-foreground text-center">
        <button
          type="button"
          className="underline underline-offset-2 hover:text-foreground"
          onClick={() => openUrl("https://paperboardlabs.com")}
        >
          Paper Board Labs
        </button>{" "}
        © 2026
      </p>
    </div>
  );
};
