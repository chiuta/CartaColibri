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
`--embed-model` / `COLI_EMBED_MODEL` choose the model; every model keeps its own vectors (a trial with `--mock` never erases real ones). If an engine starts answering with vectors of another size, search says so and `embed --reset` makes them again. Document filters (`ext:`, `path:`, `size:`, dates, groups) are applied while comparing, so they see every passage. `--mock` is a lexical stand-in for trying the pipeline offline.
Brute-force cosine over all passages: 31,000 passages take about 0.2 s per question with the offline mock (256 dimensions); real models have 3-6x more dimensions. Fine for hundreds of thousands of passages, not for tens of millions. A question times out after 30 s, so a hung engine cannot freeze a search.

`cartacolibri eval FILE` measures retrieval on your own questions (`question<TAB>file1,file2` or JSON) per mode: hit@1/5/10 and MRR, plus the missed questions.

## Other engines (Ollama, LM Studio, any OpenAI-compatible server)
Colibrì is the reference engine, but every call is plain OpenAI-style HTTP (`/v1/models`, `/v1/chat/completions`, `/v1/embeddings`), so:
`--url http://127.0.0.1:11434 --model llama3.2` (Ollama; `ollama pull nomic-embed-text` for `embed`) or `--url http://127.0.0.1:1234` (LM Studio; load an embedding model there).
`COLI_URL` / `COLI_API_KEY` / `COLI_EMBED_MODEL` set them once. `cartacolibri doctor` finds a running engine on the usual ports.
Honest status: the protocol is tested against fake servers (good answers, 401/403/500, broken JSON, empty and null content, timeouts, wrong vector counts);
it has not been run against every real Ollama / LM Studio build, so tell us what breaks.

## Connect the chat services you know (`providers`, `--provider`, `versus`)
Local is the default and stays the default. But the same OpenAI-style protocol is spoken by the services in most courses, so one name connects them:
`cartacolibri providers` lists Ollama, LM Studio, Mistral (Le Chat), OpenAI (ChatGPT), Google Gemini, Cohere (Coral), NVIDIA build (Nemotron, Llama), Claude and Meta (Llama API), and which are ready.
Use one on `chat`, `task`, `summarize`… with `--provider NAME:MODEL` (for example `--provider ollama:llama3.2`).
Cloud services also need `--allow-cloud` (or `COLI_ALLOW_CLOUD=1`), a key in their own variable (`MISTRAL_API_KEY`, `OPENAI_API_KEY`, `GEMINI_API_KEY`, `COHERE_API_KEY`, `NVIDIA_API_KEY`, `ANTHROPIC_API_KEY`, `LLAMA_API_KEY`, or `COLI_API_KEY`) and a model name:
the question **and the passages found in your documents are sent to that company**. Without both, nothing leaves your computer. Keys are never written to disk or logs, and a key is only sent to the service it belongs to.
Not connectable: Microsoft Copilot and the chat websites themselves (meta.ai, chatgpt.com, gemini.google.com…) have no public key-based API for this; use the API names above. Meaning search (`embed`) stays on local engines on purpose: it would send the whole library out.
Honest status: the protocol, the path handling (`/v1`, Gemini's `/v1beta/openai`, Cohere's `/compatibility/v1`) and the consent rules are tested against fakes; the cloud endpoints themselves were not called with real keys.

`cartacolibri versus "question" --providers mock,colibri,ollama:llama3.2,mistral:MODEL --allow-cloud` asks each model the same question over the same retrieved passages, then prints time, length, number of [S#] citations and how many look unsupported, followed by every answer.
It compares how models handle your documents; it is not a general benchmark.

### CartaColibri next to those chat services (honest)
| | ChatGPT, Gemini, Le Chat, Claude, Copilot, Meta AI, Coral | CartaColibri |
|---|---|---|
| Where your files go | uploaded to the company (limits on size and count) | stay on your computer; indexed in place |
| Library size | a few files per chat | tens of thousands of files, searched in milliseconds |
| Answers | general knowledge plus the files you attach, sources not always shown | only from your documents, every claim with [S#], unsupported ones marked |
| "Not found" | may guess | says so and stops |
| Quality of the writing and reasoning | usually higher (largest models) | the quality of the model you connect; the best of them can be connected |
| Web search, images, voice, agents, apps, accounts, sync | yes | no (honest limit) |
| Cost | free tiers with limits, paid plans | free; a cloud model, if you connect one, bills you |
| Works offline | no | yes (with a local model, or `--mock` extractive answers) |
Use them together: CartaColibri finds and cites the passages in your own files; the model you connect, local or cloud, writes the answer.

## Measured speed (scripts/bench.py, synthetic 20,000 documents, 30 MB of text, one core)
first index 6.4 s · re-index with no change 0.13 s · word / phrase / prefix / boolean search 0.02–0.07 s · regex search 0.3 s ·
entities, chronology, duplicates, conflicts, topics 0.3–0.5 s each · index 53 MB · about 25 MB of memory for searching (67 MB for stats).
Synthetic text is not your text: run `python3 scripts/bench.py --docs N` on your machine.

## Limits (honest)
No agents, no web search, no accounts or multi-user mode, no phone app, no sync. The insight tools (conflicts, duplicates, chronology, entities,
fact-check, grounding) are word and number heuristics with caps, not understanding: they point at things to check. Answer quality is the quality of the
local model. The whole-library analyses load at most 400 million characters of text (use `--group` to analyse a part of a bigger library). `.msg` and `.pst` mail, audio and video are not read. Translations are not reviewed by native speakers. The project is alpha; the engine
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
