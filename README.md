<p align="center"><img src="assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Your documents, searchable and explained — 100 % local.</b></p>

<p align="center"><b>English</b> · [Română](docs/cartacolibri/README.ro.md) · [Français](docs/cartacolibri/README.fr.md) · [Italiano](docs/cartacolibri/README.it.md) · [Español](docs/cartacolibri/README.es.md) · [Português](docs/cartacolibri/README.pt.md) · [Deutsch](docs/cartacolibri/README.de.md) · [Magyar](docs/cartacolibri/README.hu.md)</p>

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
- **web-grab** — a small Rust browser (clean-room) with tabs that knows your library: a badge when a page is already saved (and when it has changed), related documents while you browse, one-click save, questions to your local Colibrì about a page — or across all open tabs, with citations. A command and a bookmarklet cover other browsers. Also: 🖍 highlights stored with the page, 👁 watched pages with a diff of what changed (`cartacolibri webwatch`), an HTML snapshot of each save, an opt-in searchable history (`cartacolibri history on`), 🗂📝📊 summarize or extract across tabs, and `?question` in the address bar to ask your whole library.

## Power features

- **Find** — a live progress bar while indexing; “did you mean…?” for typos, with suggestion chips when nothing matches; duplicates listed, and a plain reason for every skipped file.
- **Organize** — tags, a star and a private note on every document; they survive re-indexing. See related documents, what you opened recently and most often, and save searches as smart collections.
- **14 tasks** — compare, timeline, glossary, outline, actions, FAQ, flashcards, quiz, translate, explain, contradictions, plus summarize, extract and synthesize. Tick **verify citations** to flag sentences the sources do not back, and pick the answer language.
- **Chat** — ask follow-up questions across your whole library or one group. Answers cite their sources, or honestly say “not found in your documents” instead of guessing.
- **Share and safeguard** — export a group, tag or search as Markdown, CSV, JSON or zip, with citations; a Stats view with charts and a timeline; a one-file Backup with checksums; Check/Repair for the index.
- **Bring things in** — import browser bookmarks, an Obsidian folder, Zotero/BibTeX, Pocket/Raindrop CSV or Evernote exports, and follow RSS/Atom feeds (OPML in and out): they become searchable notes. Email, calendar, subtitles and notebooks are indexed too.

## New in 0.2.0-alpha.4

- **Filters right in the search box** — Type `ext:pdf`, `path:contracts`, `size:>1mb`, `after:2025-01`, `before:2025-06-30` or `re:INV-\d+` (a regular expression) next to your words and the results narrow down. They work in the web page, on the command line and through the API.
- **Smart search** — Tick “Smart search” and your local AI adds a few synonyms and related words (as a plain OR, so exact matches still rank first). The extra words are shown under the box. With no engine running, you simply get the normal search.
- **Single-question chat** — Tick “Each question on its own” and the chat forgets earlier questions: every answer comes only from your documents, with its sources.
- **An MCP server for other AI apps** — `cartacolibri mcp` lets other AI applications search your library, read a document and ask questions about it through the Model Context Protocol. It is read-only, runs on your computer and only reaches documents that search has returned.
- **A digest of what is new** — `cartacolibri digest NAME` re-runs a saved search and reports only the documents that appeared since last time. Add `--every 60` to repeat hourly, `--out report.md` to append to a file and `--summarize` to get a short summary from your local AI.

## New in 0.2.0-alpha.2

- **The engine address just works** — `http://127.0.0.1:8000` and `http://127.0.0.1:8000/v1` are both fine. CartaColibri reads the model list from `/v1/models`, picks the loaded model by itself and no longer needs a `/health` page. That fixes the “AI engine not running (404)” message.
- **It finds your AI by itself** — CartaColibri looks for a running engine on this computer: Colibri Companion’s own settings, a discovery file, then the usual local ports (8000, 8080, 11434, 1234, 8001, 5000). Only loopback addresses are ever probed. Open ⚙ to see what was found, or press “Start Colibri Companion”.
- **Live status and speed** — The header shows the running model and whether it was found automatically, refreshed every few seconds. Under each answer you see the tokens used and the speed; below 3 tokens/s you get a hint to pick a smaller model. Requests go to the engine one at a time.
- **Web pages from the import dialog** — In “Bring things in” the news feeds gave way to web-grab: paste an address and save the page as one self-contained HTML or MHTML file in your library. Feeds are still there, folded away.
- **Three buttons in the web-grab browser** — “Save to folder” keeps the text copy in your library, “Grab website HTML” saves the page (Shift+click for MHTML), “PDF” prints it to PDF. PDF is silent on Linux; on Windows and macOS the print dialog opens and you choose “Save as PDF”.
- **Saved pages are searchable** — `.mhtml` and `.html` files are indexed with the right character set, so a saved page is found like any document. Pieces of a page that live on private addresses are never fetched or linked.

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

## More commands

| Command | What it does |
|---|---|
| `task NAME [request] --group G \| --tag T \| --query Q [--mock] [--verify] [--lang xx] [--save md\|html\|txt]` | the 14 tasks (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` marks sentences whose `[S#]` the text does not back |
| `chat [--group G]` | multi-turn questions over your library; answers are cited, or say “not found in your documents” |
| `tag add/rm/list/show`, `star`, `note` | your own organization, kept across re-indexing |
| `saved add/list/run/rm` | saved searches / smart collections |
| `recent`, `related PATH` | what you opened lately; similar documents |
| `export --format md\|csv\|json\|zip` | share a group, tag or saved search with citations |
| `backup OUT [--index]`, `backup-check ZIP`, `restore ZIP` | one-file backup with checksums; check it; restore it |
| `verify [--repair]` | check the index; with `--repair`, rebuild it from your files |
| `stats`, `timeline` | numbers about your library; documents by month |
| `import PATH` | bookmarks `.html`, Obsidian folder, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex` |
| `feeds add/list/rm/poll`, `feeds opml-import/opml-export` | RSS/Atom subscriptions saved as notes |
| `webwatch`, `history` | watched web pages with a diff; opt-in browsing history |

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

**How do tags, stars and notes survive re-indexing?**  
They are not stored in the search index. They live in a small file of their own, `organize.json`, in your `<home>` folder, tied to each document’s path. So **Update index**, a full re-index and even **Check/Repair** (which rebuilds the index) leave them untouched. If you rename or move a file, it counts as a new document, so the tags stay with the old path.

**How do I back up and restore?**  
Open **Stats** and press **Download a backup** (tick the box to include the index), or run `cartacolibri backup my.zip`. It is a single zip with checksums, holding your tags, notes, groups, saved searches and imported notes, never your original documents. `cartacolibri backup-check my.zip` tests it, and `cartacolibri restore my.zip` puts everything back. Keep the zip somewhere other than this computer.

**How do I import bookmarks, Obsidian or Zotero?**  
Click **Bring things in** in the Library, or run `cartacolibri import PATH`. It understands a browser bookmarks export (`.html`), an Obsidian vault folder, a Zotero/BibTeX `.bib` file, Pocket or Raindrop `.csv` files and Evernote `.enex` exports. Each item becomes a small note in your library that you can search, tag and cite. Your originals are only read.

**How do feeds work?**  
In **Bring things in → News feeds**, paste an RSS or Atom address and press **Subscribe**. When you press **Check now** (or run `cartacolibri feeds poll`), new items are saved as notes and indexed, so you can search and ask about them. Nothing runs in the background by itself. You can bring your subscriptions in and out as OPML.

**What does “verify citations” do?**  
After the AI answers, CartaColibri re-reads every sentence. If a sentence has no `[S#]` citation, or the cited source does not actually contain what the sentence says, it is marked `[unsupported]`. It is a quick safety net, not a guarantee: still open the source for anything important.

**Why was a web page refused?**  
web-grab only fetches public pages. Addresses on your own machine or private network (`localhost`, `192.168.x.x`, `10.x.x.x` and the like) are refused, so a web page can never trick it into reading your internal services. If you really want that, for example for your own intranet, start CartaColibri with `CARTACOLIBRI_GRAB_LOCAL=1`.

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
| The index looks broken: odd results, missing files, an error about the database | The index file was damaged (for example by a crash or a full disk), or fell out of step with your folders. | Open **Stats**, press **Check the index** and then **Repair the index**, or run `cartacolibri verify --repair`. It rebuilds the index from your files; your tags, notes and groups are kept. |
| A file was skipped because it was “too slow” | Reading it took longer than the time limit (default 120 seconds), usually a huge or damaged PDF or a very large scan. | Raise the limit, for example `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, then **Update index**. Or split / re-export the file. The reason is shown under “Files skipped”. |
| “AI engine not running … (404)” | The address points at something that is not the engine, or no model is loaded. | Open ⚙, press “Find the engine” and choose the address it lists; both `…:8000` and `…:8000/v1` work. Make sure a model is loaded in Colibri Companion. |

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

CartaColibri is **alpha** software (version 0.2.0-alpha): it works, but expect rough edges and keep backups of anything important. Linux is the best-tested platform: the web-grab browser is self-tested there in CI. On **Windows and macOS** the browser is compiled in CI but has **never been run by hand**. The automatic checks on GitHub (CI) pass on Linux, Windows and macOS. Your documents are safe either way, because they are only ever read. Portable downloads for Linux, Windows and macOS are on the GitHub **Releases** page (marked pre-release).

## Security

Found a vulnerability? Please report it privately, as described in [SECURITY.md](SECURITY.md); please do not open a public issue first. How the server defends itself (loopback only, the `X-Colibri-UI` header, no private addresses in web-grab, read-only originals) is written down in the [docs/threat-model.md](docs/threat-model.md).

## Licence and credits

CartaColibri is released under the **MIT licence**. It is a clean-room Rust implementation: it talks to the [Colibrì](https://github.com/JustVugg/colibri) engine (JustVugg, Apache-2.0) only through its public HTTP gateway and contains none of its code. The index and search layer follows the behaviour described in DocFetcher’s public manual; no DocFetcher code is used. The web-grab browser is built on the operating system’s own webview and contains no Falkon code.

**Alexandru-Ionuț Chiuță** · **Alexandru Dumbrava** (co-author)

## More

[Full documentation](docs/cartacolibri.md) · [Colibrì engine](https://github.com/JustVugg/colibri) · [Contributing](CONTRIBUTING.md) · [📄 Illustrated PDF tutorial](docs/cartacolibri/CartaColibri-Tutorial.en.pdf)
