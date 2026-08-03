# Recapp Desktop

Recapp is a small desktop app that catches you up on a TV show before you watch the next episode. Give it a series title, a season/episode, a timerange and it fetches the relevant episode synopses, summarizes them with a local LLM and translates the result into the language you prefer.

It's a native desktop port built with Tauri. The frontend is React/TypeScript; the backend is Rust. There's no server, no REST API, no Node runtime involved.

## What it does

Given a series, a season/episode number, and a timerange, Recapp builds a recap (duh!):

| Timerange | Behavior |
| --- | --- |
| All series until now | Every episode from earlier seasons, plus the current season up to (excluding) the selected episode |
| Season until now | Only the current season, up to (excluding) the selected episode |
| Prev episode | Just the previous episode's synopsis |
| This episode | Just the selected episode's synopsis |

For the two "until now" ranges, the collected synopses are summarized by an LLM before translation. For the single-episode ranges, the raw synopsis is translated directly.

## What you need to provide

Recapp doesn't ship with any API keys. You configure everything yourself from the Settings screen (gear icon, top left):

- **TMDB API key** (required). Recapp uses [The Movie Database](https://www.themoviedb.org/) for episode data. Get a free key at https://www.themoviedb.org/settings/api after creating an account.
- **Ollama URL and model** (required for summarization/translation). Recapp currently uses [Ollama](https://ollama.com) running locally as its LLM provider; install it, pull a model (e.g. `ollama pull gemma4`) and make sure it's running (`http://localhost:11434` by default) before generating a recap.
- **Default language**. The language recaps are translated into. Default lang is English which needs no translation step; any other language routes through Ollama.

Settings are stored locally on your machine (a JSON file in your OS's app config directory), nothing is sent anywhere except the request to TMDB and to your own local Ollama instance.

## Installing a prebuilt release

If you just want to use the app, download the installer for your OS from the [Releases page](https://github.com/blitzmartin/recapp-desktop/releases):

- **macOS**: download the `.dmg`, open it, drag Recapp to Applications.
- **Windows**: download the `.msi` or `.exe` and run it.
- **Linux**: download the `.deb`, `.rpm`, or AppImage for your distro.

You'll still need [Ollama](https://ollama.com) running locally with a model pulled, and a free [TMDB API key](https://www.themoviedb.org/settings/api) — see [What you need to provide](#what-you-need-to-provide) above.

### "Unidentified developer" / "Windows protected your PC" warnings

Recapp isn't code-signed or notarized (that requires a paid Apple Developer account and a Windows code-signing certificate), so both macOS and Windows will flag the installer as coming from an unknown publisher. This is expected — here's how to run it anyway:

**macOS:**
1. Try to open the app — you'll get a message saying it "cannot be opened because the developer cannot be verified" (or it's damaged/can't be opened, depending on macOS version).
2. Open **System Settings → Privacy & Security**, scroll down, and click **"Open Anyway"** next to the Recapp warning. Confirm in the dialog that appears.
3. Alternatively, right-click (or Control-click) the app in Finder and choose **Open**, then confirm in the dialog — this bypasses Gatekeeper for that app without touching system settings.
4. If macOS says the app is "damaged and can't be opened" (Gatekeeper quarantine on a downloaded, unsigned app), clear the quarantine flag from Terminal:
   ```bash
   xattr -cr /Applications/recapp-desktop.app
   ```

**Windows:**
1. Running the installer triggers **"Windows protected your PC"** (SmartScreen).
2. Click **"More info"**, then **"Run anyway"**.
3. If SmartScreen doesn't show that option, right-click the installer file → **Properties** → check **"Unblock"** at the bottom of the General tab → **OK**, then run it again.

## Development setup

The following is only needed if you want to run Recapp from source or build it yourself.

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain, via `rustup`)
- Platform build tools for Tauri: see the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/) for your OS (Xcode Command Line Tools on macOS, Build Tools for Visual Studio on Windows, standard dev packages on Linux)
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

## License

[MIT](LICENSE)
