# CartaColibri — reference

> New here? Start with **Your first 5 minutes**, the FAQ and the troubleshooting table in the [README](../README.md) (also in [Română](cartacolibri/README.ro.md), [Français](cartacolibri/README.fr.md), [Italiano](cartacolibri/README.it.md), [Español](cartacolibri/README.es.md), [Português](cartacolibri/README.pt.md), [Deutsch](cartacolibri/README.de.md), [Magyar](cartacolibri/README.hu.md)). This page is the technical reference.

## Layers
`your folders (read-only)` → **RAG-fetch** (SQLite FTS5 index, filters, groups) → **RAG-CARTA** (summarize / extract / synthesize through the Colibrì gateway) → the local UI.

State lives in one folder: `$CARTACOLIBRI_HOME` (default `~/.cartacolibri`) — `index.sqlite`, `groups.json`,
`library/uploads/`, `outputs/` (never indexed), `clip.token`.

## Formats
txt md markdown rst log csv json jsonl yaml yml html htm rtf · pdf · docx odt ods odp pptx xlsx epub (and the macro/template variants docm dotx xlsm pptm ppsx …) ·
eml mbox ics vcf mhtml · zip, tar, tgz / tar.gz (listing plus the text inside) ·
old `.doc` `.xls` `.ppt` only when `antiword`/`catdoc`/`xls2csv`/`catppt` or LibreOffice (`soffice`) is installed (otherwise they are skipped, not failed) ·
images (png jpg tif bmp webp) and scanned PDF pages when OCR is on (`--ocr` / `CARTACOLIBRI_OCR=1`, needs `tesseract`, and `pdftoppm` for PDFs).
Legacy 8-bit text is read as Windows-1250 (Romanian / Central-European). A broken or hostile file (zip bomb, corrupt PDF)
is reported under `problems` and never stops the folder.

## Search by meaning (optional)
`cartacolibri embed` asks the engine's embedding model (`POST /v1/embeddings`: Colibrì, Ollama, LM Studio) for a vector per
passage and stores it in the same SQLite index; it is incremental and restartable (`--limit`). Then
`search QUERY --mode semantic|hybrid` compares the question's vector with the passages (cosine); `hybrid` fuses the BM25 and the
vector ranking with reciprocal rank fusion (k=60), so an exact word match still wins. The same `mode` is a parameter of
`GET /carta/search` and of the MCP `search` tool. Without vectors or without an embedding model, `hybrid` falls back to words with a note.
`--embed-model` / `COLI_EMBED_MODEL` choose the model; changing it replaces the vectors. `--mock` is a lexical stand-in for trying the pipeline offline.
Brute-force cosine over all passages: fine for hundreds of thousands, not for tens of millions.

`cartacolibri eval FILE` measures retrieval on your own questions (`question<TAB>file1,file2` or JSON) per mode: hit@1/5/10 and MRR, plus the missed questions.

## Measured speed (scripts/bench.py, synthetic 20,000 documents, 30 MB of text, one core)
first index 6.4 s · re-index with no change 0.13 s · word / phrase / prefix / boolean search 0.02–0.07 s · regex search 0.3 s ·
entities, chronology, duplicates, conflicts, topics 0.3–0.5 s each · index 53 MB · about 25 MB of memory for searching (67 MB for stats).
Synthetic text is not your text: run `python3 scripts/bench.py --docs N` on your machine.

## Limits (honest)
No agents, no web search, no accounts or multi-user mode, no phone app, no sync. The insight tools (conflicts, duplicates, chronology, entities,
fact-check, grounding) are word and number heuristics with caps, not understanding: they point at things to check. Answer quality is the quality of the
local model. `.msg` and `.pst` mail, audio and video are not read. Translations are not reviewed by native speakers. The project is alpha; the engine
protocol is tested against fakes of Colibrì/Ollama/LM Studio, not against every real build.

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
- **Grounding of answers:** every `POST /carta/chat` reply has `grounding` = `{score 0-100, claims[{sentence, cited, support, status supported|partial|weak, unmatched_numbers}]}`. It is a lexical check (stems of content words against the cited passages, all passages when a sentence cites nothing), not a proof; unmatched numbers lower the score.
- **Entities:** `GET /carta/entities?kind=email|url|money|date|phone[&group=NAME]`, CLI `cartacolibri entities [--kind K] [--group G] [--json]`: what your documents mention and in which documents.
- **Compare:** `GET /carta/compare?a=ID&b=ID`, CLI `cartacolibri compare A B [--json]` (a path or its ending): similarity % plus sentences found only in one of them.
- **Conflicts:** `GET /carta/conflicts[?group=]`, CLI `conflicts` — sentence pairs from different documents that share at least four content words (Jaccard ≥ 0.5) but carry different numbers. Candidates to check, not verdicts.
- **Topic map:** `GET /carta/topics`, CLI `topics` — documents grouped by shared TF-IDF top terms.
- **What's new:** `GET /carta/changes?since=YYYY-MM-DD`, CLI `changes --since` — documents modified since a date (file modification time).
- **Fact-check:** `POST /carta/factcheck {"text"}`, CLI `factcheck FILE|-` — each sentence of your text is searched in the library and judged `supported` / `numbers differ` / `partial` / `not found` against the best evidence sentence.
- **Dossier:** `GET /carta/dossier?q=TERM`, CLI `dossier TERM` — passages per document, plus dates and amounts found in them.
- **MCP:** new tools `factcheck`, `dossier`, `conflicts`, `dupes`, `chronology` (all read-only).
- **Copies and dates:** `dupes` finds documents that are copies or near-copies (5-word phrase overlap; reports similarity and containment); `chronology` lists every dated sentence in time order (ISO, `12.03.2025`, `12/03/2025` day first, or with a month name in any of the 8 languages: `12 March 2025`, `12 martie 2025`, `2025. március 12.`). Also in the Library (“Duplicates”, “Chronology”) and the API (`/carta/dupes`, `/carta/chronology`).
- **Time range in chat:** `POST /carta/chat` accepts `"after"` and `"before"` (YYYY, YYYY-MM or YYYY-MM-DD); CLI `chat --since DATE --until DATE`. Only documents whose modification time falls in the range are retrieved.
- **E-mail digests:** `digest NAME --mail-to ADDR [--sendmail]` writes `outbox/NAME-<time>.eml` in the library folder; with `--sendmail` it also pipes the message to the local `sendmail -t` (no SMTP login is ever stored). The address must be a plain `user@host`.
