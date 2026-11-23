<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Deine Dokumente, durchsuchbar und erklärt — 100 % lokal.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · <b>Deutsch</b> · [Magyar](README.hu.md)</p>

CartaColibri ist eine lokale KI-Bibliothek auf Basis der Colibrì-Engine. Sie indiziert die Ordner, die du auswählst, lässt dich blitzschnell darin suchen und bittet dann ein lokales Sprachmodell, Dateigruppen zusammenzufassen, Daten daraus zu extrahieren oder sie zu synthetisieren — immer mit Quellenangabe. Nichts verlässt deinen Rechner, und deine Originale werden nie verändert.

## Was sie kann

- **Bibliothek** — alle lokalen Dokumente mit ihrem Speicherort; Ordner hinzufügen oder entfernen, Dateien hochladen, ansehen, auf Wunsch OCR-Text speichern.
- **RAG-fetch** — inkrementeller Volltextindex im DocFetcher-Stil (Wörter, „Phrasen“, Präfix*, AND/OR/NOT, Filter nach Typ, Größe, Ort) und wiederverwendbare **Gruppen** von Dateien.
- **RAG-CARTA** — drei lokale Aufgaben auf einer Gruppe: **zusammenfassen**, **extrahieren** (Tabelle oder JSON, „NOT FOUND“ statt Vermutungen), **synthetisieren** (Themen, Gemeinsamkeiten, Widersprüche). Jede Antwort zitiert [S1], [S2] … und listet ihre Quellen.
- **Privat von Grund auf** — keine Cloud, keine Telemetrie; Originale werden nur gelesen. 8 Sprachen: EN, RO, FR, IT, ES, PT, DE, HU.

## Schnellstart

Hol dir den Code und starte die Engine mit einem Modell, das in deinen RAM passt (siehe Colibrì-Doku), und öffne dann die CartaColibri-Seite:

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
./start-here.sh            # Windows: START-HERE.bat
coli web --model ~/colibri-models/<model>   # then open http://127.0.0.1:8790/carta.html
```

Oder nutze die Kommandozeile; zum Ausprobieren braucht es kein Modell (`--mock` ist ein Offline-Fallback):

```bash
coli carta add ~/Documents
coli carta search "contract*" --ext pdf,docx
coli carta group set Contracts ~/Documents/contracts
coli carta synthesize "payment terms" --group Contracts --save --mock
```

## Wie alles zusammenspielt

Ebene 0 — deine Ordner (nur lesend) → RAG-fetch — Index, Suche, Gruppen → RAG-CARTA — KI-Aufgaben über die Colibrì-Engine.

## Lizenz und Danksagung

CartaColibri ist ein Fork von [Colibrì](https://github.com/JustVugg/colibri) von JustVugg und behält dessen Apache-2.0-Lizenz. Die Index- und Suchschicht ist ein Clean-Room-Entwurf, inspiriert vom im öffentlichen DocFetcher-Handbuch beschriebenen Verhalten; DocFetcher-Code wird nicht verwendet.

## Mehr

[Vollständige Dokumentation](../cartacolibri.md) · [README der Colibrì-Engine](../../README.colibri.md) · [Mitmachen](../../CONTRIBUTING-CARTA.md) · [📄 Bebilderte PDF-Anleitung](CartaColibri-Tutorial.de.pdf)
