<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Your documents, searchable and explained — 100 % local.</b></p>

<p align="center"><b>English</b> · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri is a local AI library built on the Colibrì engine. It indexes the folders you choose, lets you search them in a flash, and then asks a local language model to summarize, extract data from, or synthesize groups of your files — always citing the source file. Nothing leaves your computer, and your originals are never modified.

## Your first 5 minutes

No experience needed. Three steps and you will have searched and summarized your own files.

1. **Start it.** Unzip the portable zip and run the launcher, or type `cartacolibri start` in a terminal. Your browser shows the local page, `http://127.0.0.1:8765/`.
2. **Add a folder.** On the page open **Library**, type a folder path such as `~/Documents` and click **Add folder**. The first indexing of a big folder can take a few minutes. Your files are only read, never changed.
3. **Search, then ask.** Open **RAG-fetch**, search a word, tick a few files and create a group. Then open **RAG-CARTA**, choose **Summarize** and click **Run**. No AI model installed yet? Tick **Offline mode (no model)** and try anyway.

Prefer the terminal? The same three steps:

```bash
cartacolibri start                 # step 1 - leave this window open (use a second one for the rest)
cartacolibri add ~/Documents       # step 2
cartacolibri group set Demo ~/Documents
cartacolibri summarize "main points" --group Demo --mock   # step 3 - --mock = no AI model needed
```

## What it does

- **Library** — every local document with its location; add or remove folders, upload files, view them, optionally save OCR text.
- **RAG-fetch** — DocFetcher-style incremental full-text index (words, “phrases”, prefix*, AND/OR/NOT, filters by type, size, location) and reusable **groups** of files.
- **RAG-CARTA** — three local tasks over a group: **summarize**, **extract** (table or JSON, “NOT FOUND” instead of guesses), **synthesize** (themes, shared points, contradictions). Every answer cites [S1], [S2]… and lists its sources.
- **Private by design** — no cloud, no telemetry; originals are only read. 8 languages: EN, RO, FR, IT, ES, PT, DE, HU.
- **web-grab** — a small Rust browser (clean-room, replaces Falkon) with tabs that knows your library: a badge when a page is already saved (and when it has changed), related documents while you browse, one-click save, questions to your local Colibrì about a page — or across all open tabs, with citations. A command and a bookmarklet cover other browsers. Also: 🖍 highlights stored with the page, 👁 watched pages with a diff of what changed (`cartacolibri webwatch`), an HTML snapshot of each save, an opt-in searchable history (`cartacolibri history on`), 🗂📝📊 summarize or extract across tabs, and `?question` in the address bar to ask your whole library.

## Quick start

Build it (or grab the portable zip) and start the local page. For the AI tasks also run the Colibrì engine with a model that fits your RAM (see its docs):

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
cargo build --release                       # Rust 1.90+, or use the portable zip
./target/release/cartacolibri serve         # then open http://127.0.0.1:8765/
coli web --model ~/colibri-models/<model>   # Colibrì engine, only for the AI tasks
```

Or use the command line, with no model needed to try it (`--mock` is an offline fallback):

```bash
cartacolibri add ~/Documents
cartacolibri search "contract*" --ext pdf,docx
cartacolibri group set Contracts ~/Documents/contracts
cartacolibri synthesize "payment terms" --group Contracts --save --mock
cartacolibri grab https://example.org/article      # web-grab (command line)
cartacolibri-browser https://example.org           # web-grab (browser)
```

## How it fits together

Layer 0 — your folders (read-only) → RAG-fetch — index, search, groups → RAG-CARTA — AI tasks through the Colibrì engine.

## Frequently asked questions

**What does “local” mean?**  
Everything runs on your own computer: the page, the search index and the AI model. It works with the network cable unplugged. The only exception is web-grab, which fetches the web pages you ask for.

**Is my data sent anywhere?**  
No. There is no cloud, no account, no telemetry and no analytics. The only traffic leaving your machine is the web pages you explicitly ask web-grab to fetch. Details are in “Privacy and safety”.

**Colibrì is not running. Can I still use it?**  
Yes. The Library and searching never need a model. For summarize, extract and synthesize, tick **Offline mode (no model)** (`--mock` on the command line): you get a plain answer built from your files, with citations, but without real AI wording. For real answers, start the Colibrì engine.

**What is Colibrì, and do I need it?**  
[Colibrì](https://github.com/JustVugg/colibri) is a separate local engine that runs the language model. You only need it for the AI answers. Pick a model whose file is smaller than your free RAM, and start with a small one. See Colibrì’s own docs.

**Which files can it read?**  
Text and notes (txt, md, rst, log, csv, json, yaml, html, rtf), PDF, Word, OpenDocument, PowerPoint and Excel files (docx, odt, pptx, xlsx), epub, and, with OCR on, images and scanned PDFs.

**What about scans and photos of documents?**  
Turn on OCR (`--ocr`, or `CARTACOLIBRI_OCR=1`). It needs `tesseract` installed on your computer, plus `pdftoppm` for scanned PDFs. The page tells you when tesseract is missing.

**How do I update?**  
Portable version: download the newer zip and unpack it. From source: `git pull`, then `cargo build --release`. Your data in `~/.cartacolibri` stays where it is, and the index refreshes by itself, only for what changed.

**How do I remove everything?**  
Delete the program folder, then delete the state folder `~/.cartacolibri` (or wherever `CARTACOLIBRI_HOME` points). That is all: your own documents were never touched.

**Anything special on Windows or macOS?**  
The portable zip exists for both, but Linux is the best-tested system. The web-grab browser is compiled in CI for Windows and macOS and has never been run by hand there. Windows needs the WebView2 runtime for it. If your system asks you to confirm opening a downloaded program, confirm only if you got it from the project’s page.

**Why “alpha”?**  
Because it is young: the core works and is tested, but not on every system, and the automatic checks on GitHub are not yet verified. Expect rough edges, and tell us about them.

## When something goes wrong

First try `cartacolibri doctor`: it checks your setup and tells you what to fix.

| Symptom | Likely cause | What to do |
|---|---|---|
| `cartacolibri: command not found` | The program’s folder is not in your PATH. | Run it with its full path, for example `./target/release/cartacolibri`, or use the launcher from the portable zip. |
| The page does not open at `http://127.0.0.1:8765/` | The local page is not running, or the port is taken. | Run `cartacolibri start`. If the port is busy, run `cartacolibri serve --port 8766` and open that address. |
| You do not know what is wrong | Something in the setup is missing. | Run `cartacolibri doctor`: it checks the setup and tells you what to fix. |
| The AI tasks say they cannot reach Colibrì | The Colibrì engine is not running, or listens on another address (default `http://127.0.0.1:8000`). | Start it (`coli web --model ~/colibri-models/<model>`), or pass its address with `--url`. Meanwhile tick **Offline mode (no model)**. |
| New files do not show up in search | The index has not seen them yet. | Click **Update index**, or run `cartacolibri index`. `cartacolibri watch` keeps it current automatically. |
| Some files are listed under “Files skipped” | They are corrupt, protected, unsupported or scans. | Run `cartacolibri problems` to see why. For scans, turn on OCR. |
| Scans and images give no text | OCR is off, or `tesseract` is not installed. | Install `tesseract` (and `pdftoppm` for PDFs) and add `--ocr`, or set `CARTACOLIBRI_OCR=1`. |
| The answer says NOT FOUND | The chosen files do not contain it. This is on purpose: it never guesses. | Check that the right group is selected, rephrase the question, or add the missing files to the group. |
| The web-grab browser does not open | A system component is missing. | Linux: install `libwebkit2gtk-4.1`. Windows: install the WebView2 runtime. Meanwhile `cartacolibri grab URL` works without it. |
| Your own script gets `403 missing X-Colibri-UI header` | This is a safety check, not a bug. | Send the header `X-Colibri-UI: 1` with every call to `/carta/*`. |

## Plain-words glossary

- **Index** — A catalogue of the words inside your files, kept so searches take a blink instead of reading everything again.
- **RAG** — “Retrieval-augmented generation”: first find the relevant passages in your files, then let the AI answer using only those.
- **Citation [S1]** — A marker like [S1] or [S2] in an answer. It points to the source file the sentence came from, so you can check it.
- **Group** — A named list of files (a project, a theme) that you can search or ask about again and again.
- **Gateway** — The small web address (default `http://127.0.0.1:8000`) through which CartaColibri talks to the Colibrì engine.
- **Snapshot** — A saved HTML copy of a web page, made each time you save it with web-grab, so you keep what it said that day.
- **Watch** — Two things: `cartacolibri watch` keeps your index current, and `cartacolibri webwatch` re-fetches pages you follow and shows what changed.

## Privacy and safety

- **Where things are kept.** Everything CartaColibri writes lives in one folder, `<home>`: by default `~/.cartacolibri`, or the folder in `CARTACOLIBRI_HOME`. Inside: `index.sqlite` (the search index), `groups.json`, `library/uploads/` (files you upload), `outputs/` (results you chose to save), `snapshots/` and `web.json` (saved web pages), `highlights.json`, `watch.json`, `settings.json` and `clip.token` (a secret for the bookmarklet: do not share it).
- **Your originals are read-only.** CartaColibri reads your documents and never changes, moves or deletes them.
- **The page is local-only.** The server listens on `127.0.0.1` only, so other computers on your network cannot reach it. It also refuses requests whose address or origin is not local.
- **The `X-Colibri-UI` header.** Every call the page makes to its own server carries this header. A web page you visit in another tab cannot add it, so it cannot make your browser poke the local app behind your back.
- **Nothing leaves `127.0.0.1` except pages you ask to grab.** No telemetry, no analytics, no cloud. web-grab only fetches the addresses you give it (and the pages you chose to watch). The presentation site loads nothing from other servers either.
- **History is off by default.** The web-grab browser history only exists if you turn it on (`cartacolibri history on`), stays on your disk, and `cartacolibri history clear` wipes it.

## Honest status

CartaColibri is **alpha** software (version 0.2.0-alpha): it works, but expect rough edges and keep backups of anything important. Linux is the best-tested platform: the web-grab browser is self-tested there in CI. On **Windows and macOS** the browser is compiled in CI but has **never been run by hand**. The automatic checks on GitHub (CI) are **not yet verified**: the jobs have been stuck in the queue. Your documents are safe either way, because they are only ever read.

## Licence and credits

CartaColibri is released under the **MIT licence**. It is a clean-room Rust implementation: it talks to the [Colibrì](https://github.com/JustVugg/colibri) engine (JustVugg, Apache-2.0) only through its public HTTP gateway and contains none of its code. The index and search layer follows the behaviour described in DocFetcher’s public manual; no DocFetcher code is used. The web-grab browser is built on the operating system’s own webview and contains no Falkon code.

## More

[Full documentation](../cartacolibri.md) · [Colibrì engine](https://github.com/JustVugg/colibri) · [Contributing](../../CONTRIBUTING.md) · [📄 Illustrated PDF tutorial](CartaColibri-Tutorial.en.pdf)
