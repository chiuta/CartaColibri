<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>I tuoi documenti, ricercabili e spiegati — 100 % in locale.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · <b>Italiano</b> · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri è una biblioteca IA locale costruita sul motore Colibrì. Indicizza le cartelle che scegli, ti permette di cercarvi in un lampo e poi chiede a un modello linguistico locale di riassumere, estrarre dati o sintetizzare gruppi di file — citando sempre il file sorgente. Niente lascia il tuo computer e gli originali non vengono mai modificati.

## Cosa fa

- **Biblioteca** — tutti i documenti locali con la loro posizione; aggiungi o rimuovi cartelle, carica file, visualizzali, salva se vuoi il testo OCR.
- **RAG-fetch** — indice full-text incrementale in stile DocFetcher (parole, «frasi», prefisso*, AND/OR/NOT, filtri per tipo, dimensione, posizione) e **gruppi** di file riutilizzabili.
- **RAG-CARTA** — tre compiti locali su un gruppo: **riassumere**, **estrarre** (tabella o JSON, «NOT FOUND» invece di supposizioni), **sintetizzare** (temi, punti comuni, contraddizioni). Ogni risposta cita [S1], [S2]… ed elenca le fonti.
- **Privato per progetto** — niente cloud, niente telemetria; gli originali vengono solo letti. 8 lingue: EN, RO, FR, IT, ES, PT, DE, HU.

## Avvio rapido

Scarica il codice e avvia il motore con un modello adatto alla tua RAM (vedi la documentazione Colibrì), poi apri la pagina CartaColibri:

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
./start-here.sh            # Windows: START-HERE.bat
coli web --model ~/colibri-models/<model>   # then open http://127.0.0.1:8790/carta.html
```

Oppure usa la riga di comando, senza modello per provare (`--mock` è un ripiego offline):

```bash
coli carta add ~/Documents
coli carta search "contract*" --ext pdf,docx
coli carta group set Contracts ~/Documents/contracts
coli carta synthesize "payment terms" --group Contracts --save --mock
```

## Come si incastra

Livello 0 — le tue cartelle (sola lettura) → RAG-fetch — indice, ricerca, gruppi → RAG-CARTA — compiti IA tramite il motore Colibrì.

## Licenza e crediti

CartaColibri è un fork di [Colibrì](https://github.com/JustVugg/colibri) di JustVugg e mantiene la licenza Apache-2.0. Il livello di indicizzazione e ricerca è un progetto clean-room ispirato al comportamento descritto nel manuale pubblico di DocFetcher; non si usa codice di DocFetcher.

## Per saperne di più

[Documentazione completa](../cartacolibri.md) · [README del motore Colibrì](../../README.colibri.md) · [Contribuire](../../CONTRIBUTING-CARTA.md)
