<p align="center"><img src="assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Your documents, searchable and explained — 100 % local.</b></p>

<p align="center"><b>English</b> · [Română](docs/cartacolibri/README.ro.md) · [Français](docs/cartacolibri/README.fr.md) · [Italiano](docs/cartacolibri/README.it.md) · [Español](docs/cartacolibri/README.es.md) · [Português](docs/cartacolibri/README.pt.md) · [Deutsch](docs/cartacolibri/README.de.md) · [Magyar](docs/cartacolibri/README.hu.md)</p>

CartaColibri is a local AI library built on the Colibrì engine. It indexes the folders you choose, lets you search them in a flash, and then asks a local language model to summarize, extract data from, or synthesize groups of your files — always citing the source file. Nothing leaves your computer, and your originals are never modified.

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

## Licence and credits

CartaColibri is released under the **MIT licence**. It is a clean-room Rust implementation: it talks to the [Colibrì](https://github.com/JustVugg/colibri) engine (JustVugg, Apache-2.0) only through its public HTTP gateway and contains none of its code. The index and search layer follows the behaviour described in DocFetcher’s public manual; no DocFetcher code is used. The web-grab browser is built on the operating system’s own webview and contains no Falkon code.

## More

[Full documentation](docs/cartacolibri.md) · [Colibrì engine](https://github.com/JustVugg/colibri) · [Contributing](CONTRIBUTING.md) · [📄 Illustrated PDF tutorial](docs/cartacolibri/CartaColibri-Tutorial.en.pdf)
