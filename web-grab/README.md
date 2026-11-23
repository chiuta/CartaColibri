# web-grab

Bring web pages into your CartaColibri library — 100 % local; the address is kept as the source so
RAG-CARTA citations lead back to the page.

| Way | How | Tested |
|---|---|---|
| **Browser** `cartacolibri-browser [URL]` | A small Rust browser on the system webview (WebKitGTK / WebView2 / WKWebView). A bar on every page: back / forward / reload, address bar (type **words** to search your library instead of the web), 💾 save the page or your selection, ✨ ask your local Colibrì about the page (`--mock` for an offline answer, `--gateway URL`). Replaces the Falkon plugin idea: no Falkon code, MIT. | Linux: CI self-test under Xvfb (`--selftest-save`). Windows / macOS: compiled in CI, **not yet run by hand** |
| Command line | `cartacolibri grab https://example.org/article` | CI |
| Bookmarklet (any other browser) | `cartacolibri bookmarklet` prints a `javascript:` link; save it as a bookmark and click it on a page (a selection is saved instead of the whole page) | the `/carta/clip` endpoint is tested; a page's CSP can block it |
| API | `POST /carta/grab`, `POST /carta/clip?token=…` | CI |

Browser notes
- The bar is injected into every page inside a closed shadow root; its messages to the app carry a random per-run
  nonce, so page scripts cannot trigger saves or questions.
- No tabs yet: links that want a new window open in the same view. No extensions, no sync, no telemetry.
- Build: `cargo build --release -p cartacolibri-browser` (Linux needs `libwebkit2gtk-4.1-dev`; Windows needs the WebView2 runtime).
- Pages are decoded as the site declares; a page without a charset can show wrong accents.
