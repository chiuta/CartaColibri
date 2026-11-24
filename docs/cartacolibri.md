# CartaColibri — reference

> New here? Start with **Your first 5 minutes**, the FAQ and the troubleshooting table in the [README](../README.md) (also in [Română](cartacolibri/README.ro.md), [Français](cartacolibri/README.fr.md), [Italiano](cartacolibri/README.it.md), [Español](cartacolibri/README.es.md), [Português](cartacolibri/README.pt.md), [Deutsch](cartacolibri/README.de.md), [Magyar](cartacolibri/README.hu.md)). This page is the technical reference.

## Layers
`your folders (read-only)` → **RAG-fetch** (SQLite FTS5 index, filters, groups) → **RAG-CARTA** (summarize / extract / synthesize through the Colibrì gateway) → the local UI.

State lives in one folder: `$CARTACOLIBRI_HOME` (default `~/.cartacolibri`) — `index.sqlite`, `groups.json`,
`library/uploads/`, `outputs/` (never indexed), `clip.token`.

## Formats
txt md markdown rst log csv json jsonl yaml yml html htm rtf · pdf · docx odt pptx xlsx epub ·
images (png jpg tif bmp webp) and scanned PDF pages when OCR is on (`--ocr` / `CARTACOLIBRI_OCR=1`, needs `tesseract`, and `pdftoppm` for PDFs).
Legacy 8-bit text is read as Windows-1250 (Romanian / Central-European). A broken or hostile file (zip bomb, corrupt PDF)
is reported under `problems` and never stops the folder.

## Search syntax
`word`, `"exact phrase"`, `prefix*`, `AND` / `OR` / `NOT`. Diacritics are folded (`bilanț` finds `bilant`).
If nothing matches, words are retried as stems (≈70 % of the word) so inflections still hit.
Filters: `--ext pdf,txt`, `--min-kb`, `--max-kb`, `--path TEXT`, `--group NAME`. Score is a percentage of the best hit.

## Groups
`cartacolibri group set NAME PATH_OR_FOLDER_OR_GLOB…` (`--add` to extend, `from-search` to snapshot a query).

## RAG-CARTA
`summarize | extract | synthesize` over `--group` (or `--query`). Long sources are processed map-reduce within
`--budget` characters per model call. Every answer carries `[S#]` citations and a Sources list. `--save` writes to `outputs/`.
`--mock` is a deterministic offline fallback (no model). The engine is any OpenAI-compatible gateway
(`--url`, default `http://127.0.0.1:8000`, or `$COLI_URL`).

## Server and API
`cartacolibri serve [--port 8765]` binds to 127.0.0.1 only. `/carta/*` requires the `X-Colibri-UI` header, a loopback
`Host` and a same-origin `Origin`. `CARTACOLIBRI_WATCH=<seconds>` keeps the index current while serving.
Routes: `GET library|status|search|doc`, `POST folders|index|upload|groups|run|grab`, `DELETE folders|groups`;
`POST /carta/clip?token=` for the bookmarklet.

## web-grab
See [`web-grab/README.md`](../web-grab/README.md).

## More commands (power features)

| Command | What it does |
|---|---|
| `task NAME [request] --group G / --tag T / --query Q [--mock] [--verify] [--lang xx] [--save md\|html\|txt]` | compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions (plus summarize / extract / synthesize); `--verify` marks sentences whose `[S#]` is not backed by the text |
| `chat [--group G]` | multi-turn questions over your library; answers are cited, or say "not found in your documents" |
| `tag add/rm/list/show`, `star`, `note` | your own organization, kept across re-indexing |
| `saved add/list/run/rm` | saved searches / smart collections |
| `recent`, `related PATH` | what you opened lately, similar documents |
| `export --format md\|csv\|json\|zip` | share a group, tag or saved search with citations |
| `backup OUT [--index]`, `backup-check ZIP`, `restore ZIP`, `verify [--repair]` | one-file backup with checksums; index check and rebuild |
| `stats`, `timeline` | numbers about your library, documents by month |
| `import PATH` | bookmarks `.html`, Obsidian folder, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex` |
| `feeds add/list/rm/poll`, `feeds opml-import/opml-export` | RSS/Atom subscriptions saved as notes |
| `webwatch`, `history` | watched pages with diff; opt-in browsing history |

Also indexed now: `.eml .mbox .ics .vcf .tsv .srt .vtt .ipynb .tex .org .adoc .ndjson` and the text inside `.zip` files.
Web-grab refuses loopback / private addresses unless `CARTACOLIBRI_GRAB_LOCAL=1` is set.

## New in 0.2.0-alpha.2

- **Engine address:** both `http://127.0.0.1:8000` and `…/v1` work; the model is read from `/v1/models` and picked automatically; no `/health` page is needed.
- **Auto-detection:** Colibri Companion's `settings.json`, a discovery file, then local ports 8000, 8080, 11434, 1234, 8001, 5000 (loopback only) — see [engine-discovery.md](engine-discovery.md). The ⚙ dialog can also start Colibri Companion.
- **Live status and usage:** running model in the header, tokens and tokens/s under each answer, one request at a time.
- **web-grab pages:** `POST /carta/grab` accepts `"fmt": "html" | "mhtml"`; the import dialog has a web-grab section; `.mhtml` files are indexed.
- **web-grab browser:** toolbar buttons *Save to folder*, *Grab website HTML* (Shift+click: MHTML) and *PDF* (silent on Linux; the print dialog on Windows and macOS).

## New in 0.2.0-alpha.4

- **Inline filters:** `ext:pdf,docx`, `path:text`, `size:>500kb` / `size:<2mb` in any query (web, CLI, API). A filter-only query is an "empty query" error.
- **Smart search:** `GET /carta/find?q=…&smart=1` asks the engine for up to 8 synonyms (plain words only, added with OR); the response lists them under `expanded`. No engine, no expansion.
- **Chat mode:** `POST /carta/chat` accepts `"mode": "query"` — a single question, no memory kept.
- **MCP server:** `cartacolibri mcp [--url URL] [--mock]` speaks MCP over stdio (JSON-RPC 2.0, one message per line). Tools: `search`, `read_document` (by id from `search`), `list_groups`, `ask`. Read-only. Example for an MCP client: `{"command": "cartacolibri", "args": ["mcp"]}`.
- **Date and regex filters:** `after:2025-01`, `before:2025-06-30` (YYYY, YYYY-MM or YYYY-MM-DD, UTC) and `re:PATTERN` (no spaces; case-insensitive Rust regex tested against the matching passage; a broken pattern is an error). API: `after`, `before`, `re`. `GET /carta/find` also returns `facets.ext` (hit counts per file type).
- **Digest:** `cartacolibri digest NAME [--every MINUTES] [--out FILE.md] [--summarize] [--all] [--mock]` re-runs a saved search and reports only documents not seen in earlier runs (the first run just records the current matches unless `--all`). State: `digest-NAME.json` in the library folder.
