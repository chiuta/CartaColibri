<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Deine Dokumente, durchsuchbar und erklärt — 100 % lokal.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · <b>Deutsch</b> · [Magyar](README.hu.md)</p>

CartaColibri ist eine lokale KI-Bibliothek auf Basis der Colibrì-Engine. Sie indiziert die Ordner, die du auswählst, lässt dich blitzschnell darin suchen und bittet dann ein lokales Sprachmodell, Dateigruppen zusammenzufassen, Daten daraus zu extrahieren oder sie zu synthetisieren — immer mit Quellenangabe. Nichts verlässt deinen Rechner, und deine Originale werden nie verändert.

## Was sie kann

- **Bibliothek** — alle lokalen Dokumente mit ihrem Speicherort; Ordner hinzufügen oder entfernen, Dateien hochladen, ansehen, auf Wunsch OCR-Text speichern.
- **RAG-fetch** — inkrementeller Volltextindex im DocFetcher-Stil (Wörter, „Phrasen“, Präfix*, AND/OR/NOT, Filter nach Typ, Größe, Ort) und wiederverwendbare **Gruppen** von Dateien.
- **RAG-CARTA** — drei lokale Aufgaben auf einer Gruppe: **zusammenfassen**, **extrahieren** (Tabelle oder JSON, „NOT FOUND“ statt Vermutungen), **synthetisieren** (Themen, Gemeinsamkeiten, Widersprüche). Jede Antwort zitiert [S1], [S2] … und listet ihre Quellen.
- **Privat von Grund auf** — keine Cloud, keine Telemetrie; Originale werden nur gelesen. 8 Sprachen: EN, RO, FR, IT, ES, PT, DE, HU.
- **web-grab** — ein kleiner Browser in Rust (Clean-Room, ersetzt Falkon) mit Tabs, der deine Bibliothek kennt: ein Abzeichen, wenn die Seite schon gespeichert ist (und wenn sie sich geändert hat), verwandte Dokumente beim Surfen, Speichern mit einem Klick, Fragen an dein lokales Colibrì zu einer Seite — oder zu allen offenen Tabs, mit Quellenangaben. Ein Befehl und ein Bookmarklet decken andere Browser ab.

## Schnellstart

Kompiliere sie (oder nimm das portable Zip) und starte die lokale Seite. Für die KI-Aufgaben starte außerdem die Colibrì-Engine mit einem Modell, das in deinen RAM passt (siehe deren Doku):

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
cargo build --release                       # Rust 1.90+, or use the portable zip
./target/release/cartacolibri serve         # then open http://127.0.0.1:8765/
coli web --model ~/colibri-models/<model>   # Colibrì engine, only for the AI tasks
```

Oder nutze die Kommandozeile; zum Ausprobieren braucht es kein Modell (`--mock` ist ein Offline-Fallback):

```bash
cartacolibri add ~/Documents
cartacolibri search "contract*" --ext pdf,docx
cartacolibri group set Contracts ~/Documents/contracts
cartacolibri synthesize "payment terms" --group Contracts --save --mock
cartacolibri grab https://example.org/article      # web-grab (command line)
cartacolibri-browser https://example.org           # web-grab (browser)
```

## Wie alles zusammenspielt

Ebene 0 — deine Ordner (nur lesend) → RAG-fetch — Index, Suche, Gruppen → RAG-CARTA — KI-Aufgaben über die Colibrì-Engine.

## Lizenz und Danksagung

CartaColibri steht unter der **MIT-Lizenz**. Es ist eine Clean-Room-Implementierung in Rust: Sie spricht mit der [Colibrì](https://github.com/JustVugg/colibri)-Engine (JustVugg, Apache-2.0) nur über deren öffentliches HTTP-Gateway und enthält keinen Code daraus. Die Index- und Suchschicht folgt dem im öffentlichen DocFetcher-Handbuch beschriebenen Verhalten; DocFetcher-Code wird nicht verwendet. Der web-grab-Browser nutzt die Webview des Betriebssystems und enthält keinen Falkon-Code.

## Mehr

[Vollständige Dokumentation](../cartacolibri.md) · [README der Colibrì-Engine](https://github.com/JustVugg/colibri) · [Mitmachen](../../CONTRIBUTING.md) · [📄 Bebilderte PDF-Anleitung](CartaColibri-Tutorial.de.pdf)
