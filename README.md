# Recapp Desktop

Recapp is a small desktop app that catches you up on a TV show before you watch the next episode. Give it a series title, a season/episode, and a timerange, and it fetches the relevant episode synopses, summarizes them with a local LLM, and translates the result into the language you prefer.

It's a native desktop port (built with [Tauri](https://tauri.app)) of an earlier web app. The frontend is React/TypeScript; the backend is Rust — there's no server, no REST API, no Node runtime involved.

## What it does

Given a series, a season/episode number, and a timerange, Recapp builds a recap:

| Timerange | Behavior |
| --- | --- |
| All series until now | Every episode from earlier seasons, plus the current season up to (excluding) the selected episode |
| Season until now | Only the current season, up to (excluding) the selected episode |
| Prev episode | Just the previous episode's synopsis |
| This episode | Just the selected episode's synopsis |

For the two "until now" ranges, the collected synopses are summarized by an LLM before translation. For the single-episode ranges, the raw synopsis is translated directly.

## What you need to provide

Recapp doesn't ship with any API keys — you configure everything yourself from the Settings screen (gear icon, top left):

- **TMDB API key** (required). Recapp uses [The Movie Database](https://www.themoviedb.org/) for episode data. Get a free key at https://www.themoviedb.org/settings/api after creating an account.
- **Ollama URL and model** (required for summarization/translation). Recapp currently uses [Ollama](https://ollama.com) running locally as its LLM provider — install it, pull a model (e.g. `ollama pull llama3`), and make sure it's running (`http://localhost:11434` by default) before generating a recap.
- **Default language**. The language recaps are translated into. English needs no translation step; any other language routes through Ollama.

Settings are stored locally on your machine (a JSON file in your OS's app config directory) — nothing is sent anywhere except to TMDB and to your own local Ollama instance.

## Development setup

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain, via `rustup`)
- Platform build tools for Tauri — see the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/) for your OS (Xcode Command Line Tools on macOS, Build Tools for Visual Studio on Windows, standard dev packages on Linux)
- [Ollama](https://ollama.com) running locally, with at least one model pulled

### Running locally

```bash
npm install
npm run tauri dev
```

This starts the Vite dev server and compiles/launches the Rust backend with hot reload. On first run, open Settings and fill in your TMDB API key and Ollama configuration before using the recap form.

### Building a release executable

```bash
npm run tauri build
```

This produces a native installer for the OS you run it on (`.app`/`.dmg` on macOS, `.msi`/`.exe` on Windows, `.deb`/`.rpm`/AppImage on Linux). Tauri does not cross-compile for other operating systems from a single machine — see `.github/workflows` for a CI setup that builds all three platforms.

## Project structure

- `src/` — React/TypeScript frontend (form UI, Settings screen)
- `src-tauri/src/` — Rust backend: `tmdb.rs` (TMDB API client), `llm/` (summarization provider, currently Ollama), `translator/` (translation provider, currently Ollama), `settings.rs` (local config persistence), `commands.rs` (Tauri commands invoked from the frontend)
