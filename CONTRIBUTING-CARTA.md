# Contributing to CartaColibri

Welcome! CartaColibri is a fork of [Colibrì](https://github.com/JustVugg/colibri); the C engines stay upstream's, and all new
work lives in small, separate modules so syncing with upstream stays easy:

| area | files |
|---|---|
| RAG-fetch (index, search, groups) | `c/ragfetch.py` |
| RAG-CARTA (summarize / extract / synthesize) | `c/ragcarta.py` |
| CLI (`coli carta …`) | `c/carta_cli.py` (+ 4 lines in `c/coli`) |
| Chat-UI API and page | `ui/carta_api.py`, `ui/carta.html` |
| Docs, 8-language READMEs, site | `docs/cartacolibri.md`, `scripts/gen_carta_docs.py` → `README.md`, `docs/cartacolibri/`, `site-carta/` |

```bash
python3 -I tests/test_cartacolibri.py        # or: make check-carta
node scripts/check_carta_ui.js               # Node ≥ 24: UI syntax + 8-language completeness
python3 scripts/gen_carta_docs.py            # after editing any text in the generator
```

Rules of the house: pure standard library in the Python layers; originals are only ever read; every UI string exists in
EN, RO, FR, IT, ES, PT, DE, HU with informal address ("tu"); no cloud calls, no telemetry; clean-room — never copy code
from DocFetcher or other third-party products, only public standards and your own design.
Good first issues: more extractors (`.epub`, `.xlsx`), embeddings/hybrid retrieval, folder watching, OCR for scanned PDFs.
