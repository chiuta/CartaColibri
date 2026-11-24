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
