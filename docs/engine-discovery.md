# How CartaColibri finds the AI engine (Colibrì / Colibri Companion)

CartaColibri talks to any OpenAI-compatible local engine. Colibrì serves it at `http://127.0.0.1:8000/v1`
(`/v1/models`, `/v1/chat/completions`). It has **no `/health` route**, so CartaColibri checks `/v1/models` instead.

## Order of preference

1. The address you saved in the page (⚙ next to the engine status). Accepts `http://host:8000` or `http://host:8000/v1`.
2. `COLI_URL` / `--url` on the command line.
3. Automatic detection (only when you saved nothing): the launcher's discovery file, then these loopback ports:
   `8000` (Colibrì), `8080`, `11434` (Ollama), `1234` (LM Studio), `8001`, `5000`. The first engine that answers
   `/v1/models` with JSON is used, and the status chip says "found automatically".

The model is taken from `/v1/models` (the first one) unless you pass `--model`. If the engine needs a key,
set `COLI_API_KEY` (never written to disk or logs).

## Discovery file (for Colibri Companion or any launcher)

When the engine starts, write this file; delete it (or rewrite it) when the engine stops:

```json
{ "url": "http://127.0.0.1:8000/v1", "model": "qwen3-14b", "pid": 12345 }
```

Looked up, in order: `$COLIBRI_ENDPOINT_FILE`, `%APPDATA%\colibri-companion\endpoint.json`,
`~/.config/colibri-companion/endpoint.json`, `~/Library/Application Support/colibri-companion/endpoint.json`,
`~/.colibri/endpoint.json`. Only `http://` addresses on `127.0.0.1`, `localhost` or `::1` are accepted, and the file must be under 4 KB.

## What the page shows

The status chip shows the model in use; each answer shows tokens in/out, seconds and tokens per second,
and a hint to try a smaller model or Offline mode when the engine is slow (under 3 tokens/s).
Requests to the engine are sent one at a time, because a local model answers parallel requests no faster.
