# web-grab

Bring web pages into your CartaColibri library — 100 % local, the address is kept as the source so
RAG-CARTA citations lead back to the page.

| Way | How | Status |
|---|---|---|
| Command line | `cartacolibri grab https://example.org/article` | tested (CI) |
| Bookmarklet (any browser, **Falkon included**) | `cartacolibri bookmarklet` prints a `javascript:` link; save it as a bookmark, then click it on any page (a text selection is saved instead of the whole page) | the `/carta/clip` endpoint is tested; page CSP can block it on a few strict sites |
| Falkon plugin | `falkon/` — a Python plugin template adding **Save to CartaColibri** | **template, not yet run against a real Falkon build** — see the notes inside |
| API | `POST /carta/grab {"url": "...", "title": "...", "text": "..."}` (UI header required) or `POST /carta/clip?token=…` | tested |

The token for `/carta/clip` lives in `<home>/clip.token` (printed by `cartacolibri serve`). It is the only
route that accepts cross-origin calls, and only with that secret.

Falkon (https://www.falkon.org) is GPL-3.0. This module never links or copies Falkon code: the plugin is a
separate, optional file that talks to CartaColibri over local HTTP, so CartaColibri itself stays MIT.
