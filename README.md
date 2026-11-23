<p align="center"><img src="assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Your documents, searchable and explained — 100 % local.</b></p>

<p align="center"><b>English</b> · [Română](docs/cartacolibri/README.ro.md) · [Français](docs/cartacolibri/README.fr.md) · [Italiano](docs/cartacolibri/README.it.md) · [Español](docs/cartacolibri/README.es.md) · [Português](docs/cartacolibri/README.pt.md) · [Deutsch](docs/cartacolibri/README.de.md) · [Magyar](docs/cartacolibri/README.hu.md)</p>

CartaColibri is a local AI library built on the Colibrì engine. It indexes the folders you choose, lets you search them in a flash, and then asks a local language model to summarize, extract data from, or synthesize groups of your files — always citing the source file. Nothing leaves your computer, and your originals are never modified.

## What it does

- **Library** — every local document with its location; add or remove folders, upload files, view them, optionally save OCR text.
- **RAG-fetch** — DocFetcher-style incremental full-text index (words, “phrases”, prefix*, AND/OR/NOT, filters by type, size, location) and reusable **groups** of files.
- **RAG-CARTA** — three local tasks over a group: **summarize**, **extract** (table or JSON, “NOT FOUND” instead of guesses), **synthesize** (themes, shared points, contradictions). Every answer cites [S1], [S2]… and lists its sources.
- **Private by design** — no cloud, no telemetry; originals are only read. 8 languages: EN, RO, FR, IT, ES, PT, DE, HU.

## Quick start

Get the code and start the engine with a model that fits your RAM (see the Colibrì docs), then open the CartaColibri page:

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
./start-here.sh            # Windows: START-HERE.bat
coli web --model ~/colibri-models/<model>   # then open http://127.0.0.1:8790/carta.html
```

Or use the command line, with no model needed to try it (`--mock` is an offline fallback):

```bash
coli carta add ~/Documents
coli carta search "contract*" --ext pdf,docx
coli carta group set Contracts ~/Documents/contracts
coli carta synthesize "payment terms" --group Contracts --save --mock
```

## How it fits together

Layer 0 — your folders (read-only) → RAG-fetch — index, search, groups → RAG-CARTA — AI tasks through the Colibrì engine.

## Licence and credits

CartaColibri is a fork of [Colibrì](https://github.com/JustVugg/colibri) by JustVugg and keeps its Apache-2.0 licence. The index and search layer is a clean-room design inspired by the behaviour described in DocFetcher’s public manual; no DocFetcher code is used.

## More

[Full documentation](docs/cartacolibri.md) · [Colibrì engine README](README.colibri.md) · [Contributing](CONTRIBUTING-CARTA.md)
