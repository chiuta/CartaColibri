<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>I tuoi documenti, ricercabili e spiegati — 100 % in locale.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · <b>Italiano</b> · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri è una biblioteca IA locale costruita sul motore Colibrì. Indicizza le cartelle che scegli, ti permette di cercarvi in un lampo e poi chiede a un modello linguistico locale di riassumere, estrarre dati o sintetizzare gruppi di file — citando sempre il file sorgente. Niente lascia il tuo computer e gli originali non vengono mai modificati.

## Cosa fa

- **Biblioteca** — tutti i documenti locali con la loro posizione; aggiungi o rimuovi cartelle, carica file, visualizzali, salva se vuoi il testo OCR.
- **RAG-fetch** — indice full-text incrementale in stile DocFetcher (parole, «frasi», prefisso*, AND/OR/NOT, filtri per tipo, dimensione, posizione) e **gruppi** di file riutilizzabili.
- **RAG-CARTA** — tre compiti locali su un gruppo: **riassumere**, **estrarre** (tabella o JSON, «NOT FOUND» invece di supposizioni), **sintetizzare** (temi, punti comuni, contraddizioni). Ogni risposta cita [S1], [S2]… ed elenca le fonti.
- **Privato per progetto** — niente cloud, niente telemetria; gli originali vengono solo letti. 8 lingue: EN, RO, FR, IT, ES, PT, DE, HU.
- **web-grab** — un piccolo browser in Rust (clean-room, sostituisce Falkon) con schede, che conosce la tua biblioteca: un badge quando la pagina è già salvata (e quando è cambiata), documenti correlati mentre navighi, salvataggio con un clic, domande al tuo Colibrì locale su una pagina — o su tutte le schede aperte, con citazioni. Un comando e un bookmarklet coprono gli altri browser. Inoltre: 🖍 evidenziazioni salvate con la pagina, 👁 pagine monitorate con le differenze (`cartacolibri webwatch`), istantanea HTML a ogni salvataggio, cronologia ricercabile opzionale (`cartacolibri history on`), 🗂📝📊 riassunto o estrazione su tutte le schede e `?domanda` nella barra degli indirizzi per interrogare tutta la biblioteca.

## Avvio rapido

Compilala (o prendi lo zip portatile) e avvia la pagina locale. Per i compiti IA avvia anche il motore Colibrì con un modello adatto alla tua RAM (vedi la sua documentazione):

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
cargo build --release                       # Rust 1.90+, or use the portable zip
./target/release/cartacolibri serve         # then open http://127.0.0.1:8765/
coli web --model ~/colibri-models/<model>   # Colibrì engine, only for the AI tasks
```

Oppure usa la riga di comando, senza modello per provare (`--mock` è un ripiego offline):

```bash
cartacolibri add ~/Documents
cartacolibri search "contract*" --ext pdf,docx
cartacolibri group set Contracts ~/Documents/contracts
cartacolibri synthesize "payment terms" --group Contracts --save --mock
cartacolibri grab https://example.org/article      # web-grab (command line)
cartacolibri-browser https://example.org           # web-grab (browser)
```

## Come si incastra

Livello 0 — le tue cartelle (sola lettura) → RAG-fetch — indice, ricerca, gruppi → RAG-CARTA — compiti IA tramite il motore Colibrì.

## Licenza e crediti

CartaColibri è rilasciato con **licenza MIT**. È un’implementazione Rust clean-room: parla con il motore [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) solo tramite il suo gateway HTTP pubblico e non ne contiene codice. Il livello di indicizzazione e ricerca segue il comportamento descritto nel manuale pubblico di DocFetcher; non si usa codice DocFetcher. Il browser web-grab si basa sulla webview del sistema operativo e non contiene codice di Falkon.

## Per saperne di più

[Documentazione completa](../cartacolibri.md) · [README del motore Colibrì](https://github.com/JustVugg/colibri) · [Contribuire](../../CONTRIBUTING.md) · [📄 Tutorial PDF illustrato](CartaColibri-Tutorial.it.pdf)
