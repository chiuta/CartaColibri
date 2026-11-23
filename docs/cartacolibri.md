# CartaColibri — local AI library on top of Colibrì

<p align="center"><img src="../assets/cartacolibri-logo.png" width="280" alt="CartaColibri — local AI library"></p>

100 % local and private. Two layers over a folder of your documents; the Colibrì C engines
are untouched and the new code lives in `c/ragfetch.py`, `c/ragcarta.py`, `c/carta_cli.py`
(+ `ui/carta_api.py` for the chat UI). Originals are only read, never modified.

```
Layer 0  Library      your folders (read-only)
Layer 1  RAG-fetch    incremental full-text index · search/filters · configurable GROUPS
Layer 2  RAG-CARTA    summarize · extract · synthesize over a group, through the Colibrì gateway
```

## RAG-fetch (DocFetcher-style, clean-room)

* One SQLite/FTS5 index in the state directory (`~/.cartacolibri`, or `$CARTACOLIBRI_HOME`, or `--home`).
  Diacritics are folded (`scoala` finds *școală*); legacy cp1250 text files are decoded.
* **Incremental**: size+mtime, then SHA-1, decide what to re-read; deleted files leave the index.
* **Priority 1**: `.txt .md .markdown .rst .log .pdf` (pypdf → PyMuPDF → `pdftotext`).
  **Priority 2**: `.csv .json .jsonl .yaml .yml .html .htm .rtf .docx .odt .pptx`.
* **Excluded**: `_cartacolibri/`, `.cartacolibri/`, `.git/`, `__pycache__/`, hidden/temp files, archives,
  weights, raw media. A corrupt file is logged (`coli carta problems`) and skipped; the folder carries on.
* Query syntax: words, `"phrases"`, `prefix*`, `AND` / `OR` / `NOT`; filters by extension, size, path, group.

### Groups (configurable lists)

Stored in `<state>/groups.json`:

```json
{ "Contracte": { "description": "toate contractele",
                 "members": ["/home/me/docs/contracte", "/home/me/docs/**/*.pdf"] } }
```

A member is a file, a folder, or a glob (matched on the full path). Groups are re-usable across tasks.

## RAG-CARTA

| task | does | output |
|---|---|---|
| `summarize` | key points per document + overview | Markdown |
| `extract` | the fields you name; `NOT FOUND` instead of guesses | Markdown table or `--json` |
| `synthesize` | one answer across files: themes, shared points, contradictions | Markdown |

Every result cites `[S1]`, `[S2]`… and ends with a **Sources** list mapping them to files.
Long documents are processed map-reduce (`--budget` characters per model call). `--save` writes the
result, labelled with timestamp/group/task, to `<state>/outputs/` (never indexed).
The model is any running Colibrì gateway (`coli web --model DIR` or `coli serve`, default
`http://127.0.0.1:8000`; `--url`, `$COLI_URL`). `--mock` is a deterministic offline extractive fallback.

## Commands

```bash
coli carta add ~/Documente                 # register + index a folder
coli carta search "valoarea totală" --ext pdf,docx --min-kb 10
coli carta group set Contracte ~/Documente/contracte "~/Documente/**/*.pdf" -d "toate contractele"
coli carta group from-search Facturi "factura 2026" --ext pdf
coli carta summarize  --group Contracte --save
coli carta extract    --group Contracte --fields "părți,valoare,termen,data" --save
coli carta synthesize "termene de plată" --group Contracte --save
coli carta index | folders | list | problems | remove DIR
```

## Chat UI API (`coli web` / `coli ui`)

`/carta/library`, `/carta/folders` (POST add / DELETE remove), `/carta/index`, `/carta/search`,
`/carta/doc?id=`, `/carta/groups`, `/carta/run` — local page only (Host/Origin + `X-Colibri-UI` checks,
as for the Models page).

## Tests

`make check-carta` (index, exclusions, errors, incremental updates, search, groups, the three tasks
against a mock and a fake HTTP gateway, CLI end-to-end).

## The screens

`coli web` (or `coli ui`) serves **`/carta.html`**; the chat page has two new icons on its left rail.

* **Library** — all documents with location, add / remove folder, update index, drag-and-drop **upload to library**
  (optionally saving OCR text as TXT and a searchable PDF when `tesseract` is installed), **view** a document, files skipped.
* **RAG-fetch** — search box, filters (type, size, location), groups as the search scope, **Create index from selected files**.
* **RAG-CARTA** (button top-right) — summarize / extract / synthesize over the selected files or a group, with a
  “Message Colibri” box, results with [S#] citations, copy, save to `outputs/`, offline `mock` mode.
* 8 languages (EN RO FR IT ES PT DE HU, informal address), light/dark theme, palette from the logo.

## Portable

`python3 scripts/make_portable.py --version 0.1.0-alpha.1` builds a PortableApps-layout zip
(`CartaColibriPortable.bat` / `.sh`, `Data/` holds the index, groups, outputs and models; nothing is written outside the folder).
The *Tag a CartaColibri release* workflow does it and drafts the pre-release.

## Also included

* **OCR** (needs `tesseract`; `--ocr` or `CARTACOLIBRI_OCR=1`): images and scanned PDF pages (via `pdftoppm`) are read and indexed; uploads can
  also save the text (TXT) and a searchable PDF.
* **Extractors** for `.epub` and `.xlsx`; a typo/inflection-tolerant stem-prefix fallback when a search finds nothing.
* **Folder watching**: `coli carta watch --interval 30`, or `CARTACOLIBRI_WATCH=30` for the UI server.
* **Chat composer**: “Upload to library” in the `+` menu of the chat page.
* **Tutorials**: illustrated PDFs in 8 languages (`docs/cartacolibri/CartaColibri-Tutorial.<lang>.pdf`) and a demo video with 8 subtitle
  tracks (`site-carta/media/`), both rebuilt with `scripts/capture_carta_demo.py` + `scripts/gen_carta_tutorial.py`.

## Not yet (next iterations)

Embeddings / true hybrid retrieval, a voice-over for the demo video, a CartaColibri-specific Windows installer.
