<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>I tuoi documenti, ricercabili e spiegati — 100 % in locale.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · <b>Italiano</b> · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri è una biblioteca IA locale costruita sul motore Colibrì. Indicizza le cartelle che scegli, ti permette di cercarvi in un lampo e poi chiede a un modello linguistico locale di riassumere, estrarre dati o sintetizzare gruppi di file — citando sempre il file sorgente. Niente lascia il tuo computer e gli originali non vengono mai modificati.

## I tuoi primi 5 minuti

Non serve esperienza. Tre passi e avrai cercato e riassunto i tuoi file.

1. **Avviala.** Decomprimi lo zip portatile e avvii il lanciatore, oppure scrivi `cartacolibri start` in un terminale. Il browser mostra la pagina locale, `http://127.0.0.1:8765/`.
2. **Aggiungi una cartella.** Nella pagina apri **Biblioteca**, scrivi il percorso di una cartella come `~/Documents` e clicca **Aggiungi cartella**. La prima indicizzazione di una cartella grande può richiedere qualche minuto. I tuoi file vengono solo letti, mai modificati.
3. **Cerca, poi chiedi.** Apri **RAG-fetch**, cerca una parola, spunta qualche file e crea un gruppo. Poi apri **RAG-CARTA**, scegli **Riassumi** e clicca **Esegui**. Non hai ancora un modello IA? Spunta **Modalità offline (senza modello)** e prova lo stesso.

Preferisci il terminale? Gli stessi tre passi:

```bash
cartacolibri start                 # step 1 - leave this window open (use a second one for the rest)
cartacolibri add ~/Documents       # step 2
cartacolibri group set Demo ~/Documents
cartacolibri summarize "main points" --group Demo --mock   # step 3 - --mock = no AI model needed
```

## Cosa fa

- **Biblioteca** — tutti i documenti locali con la loro posizione; aggiungi o rimuovi cartelle, carica file, visualizzali, salva se vuoi il testo OCR.
- **RAG-fetch** — indice full-text incrementale in stile DocFetcher (parole, «frasi», prefisso*, AND/OR/NOT, filtri per tipo, dimensione, posizione) e **gruppi** di file riutilizzabili.
- **RAG-CARTA** — tre compiti locali su un gruppo: **riassumere**, **estrarre** (tabella o JSON, «NOT FOUND» invece di supposizioni), **sintetizzare** (temi, punti comuni, contraddizioni). Ogni risposta cita [S1], [S2]… ed elenca le fonti.
- **Privato per progetto** — niente cloud, niente telemetria; gli originali vengono solo letti. 8 lingue: EN, RO, FR, IT, ES, PT, DE, HU.
- **web-grab** — un piccolo browser in Rust (clean-room) con schede, che conosce la tua biblioteca: un badge quando la pagina è già salvata (e quando è cambiata), documenti correlati mentre navighi, salvataggio con un clic, domande al tuo Colibrì locale su una pagina — o su tutte le schede aperte, con citazioni. Un comando e un bookmarklet coprono gli altri browser. Inoltre: 🖍 evidenziazioni salvate con la pagina, 👁 pagine monitorate con le differenze (`cartacolibri webwatch`), istantanea HTML a ogni salvataggio, cronologia ricercabile opzionale (`cartacolibri history on`), 🗂📝📊 riassunto o estrazione su tutte le schede e `?domanda` nella barra degli indirizzi per interrogare tutta la biblioteca.

## Funzioni avanzate

- **Trova** — una barra di avanzamento durante l’indicizzazione; «Intendevi…?» per i refusi, con suggerimenti quando nulla corrisponde; i duplicati elencati e un motivo chiaro per ogni file saltato. Scrivi i filtri direttamente nella casella (`ext:pdf`, `path:`, `size:>1mb`, date `after:`/`before:`, espressioni regolari `re:`), spunta **Ricerca intelligente** per aggiungere parole affini e vedi quanti risultati ha ogni tipo di file.
- **Organizza** — etichette, una stella e una nota privata su ogni documento; sopravvivono alla reindicizzazione. Vedi i documenti correlati, ciò che hai aperto di recente e più spesso, e salva le ricerche come raccolte intelligenti.
- **14 compiti** — confronta, cronologia, glossario, scaletta, azioni, FAQ, flashcard, quiz, traduci, spiega, contraddizioni, più riassumi, estrai e sintetizza. Spunta **verifica le citazioni** per segnalare le frasi che le fonti non sostengono e scegli la lingua della risposta.
- **Chat** — fai domande una dopo l’altra su tutta la biblioteca o su un gruppo. Le risposte citano le fonti, oppure dicono onestamente «non trovato nei tuoi documenti» invece di tirare a indovinare. Ogni risposta mostra un punteggio di **Aderenza alle fonti** e segnala le frasi o i numeri che le fonti non sostengono. Spunta «Ogni domanda a sé» per non ricordare nulla, oppure limita i documenti a un intervallo di tempo.
- **Condividi e proteggi** — esporta un gruppo, un’etichetta o una ricerca in Markdown, CSV, JSON o zip, con citazioni; una vista Statistiche con grafici e cronologia; un backup in un solo file con checksum; Controlla/Ripara per l’indice.
- **Porta dentro le cose** — importa i segnalibri del browser, una cartella Obsidian, Zotero/BibTeX, CSV di Pocket/Raindrop o esportazioni di Evernote, e segui i feed RSS/Atom (OPML in entrata e in uscita): diventano note ricercabili. Anche email, calendari, sottotitoli e notebook vengono indicizzati.
- **Analisi** — “Contraddizioni” (stessa frase, numero o data diversi), “Mappa dei temi” (bolle), “Novità” da una data, “Confronta 2” documenti, “Nomi e numeri” (e-mail, link, importi, date, telefoni), “Dossier” su un nome e “Verifica il mio testo” rispetto alla biblioteca.
- **Automatizza e collega** — `digest` rilancia una ricerca salvata e segnala solo le novità (ripete, accoda a un file, riassume o salva un file e-mail); `mcp` permette ad altre app di IA di cercare, leggere, interrogare, verificare e chiedere dossier nella tua biblioteca, in sola lettura.
- **Copie e date** — “Duplicati” (documenti copie o quasi-copie, con quanto si somigliano) e “Cronologia” (tutte le frasi con data in ordine di tempo).

## Novità della 0.2.0-alpha.10

- **Salva da LegatorOCR direttamente nella biblioteca** — Quando LegatorOCR è aperto da CartaColibri, il suo menu File offre ora «Salva il PDF nella biblioteca CartaColibri». Il PDF va nella cartella `from-legatorocr` della biblioteca senza finestra di dialogo (un nome esistente riceve -2, -3, … così nulla viene sostituito) e viene indicizzato con il resto. Serve una versione di LegatorOCR che contenga la nuova voce di menu; le più vecchie funzionano ancora, salvi semplicemente a mano in quella cartella.

## Novità dalla 0.2.0-alpha.3 alla alpha.9

- **Filtri direttamente nella casella di ricerca** — Scrivi `ext:pdf`, `path:contratti`, `size:>1mb`, `after:2025-01`, `before:2025-06-30` o `re:FATT-\d+` (espressione regolare) accanto alle tue parole e i risultati si restringono. Funzionano nella pagina web, da riga di comando e tramite API.
- **Ricerca intelligente** — Spunta «Ricerca intelligente» e la tua IA locale aggiunge qualche sinonimo e parola affine (come semplice OR: le corrispondenze esatte restano in testa). Le parole aggiunte compaiono sotto la casella. Senza motore attivo ottieni la normale ricerca.
- **Chat a domanda singola** — Spunta «Ogni domanda a sé» e la chat dimentica le domande precedenti: ogni risposta viene solo dai tuoi documenti, con le sue fonti.
- **Un server MCP per altre app di IA** — `cartacolibri mcp` permette ad altre applicazioni di IA di cercare nella tua biblioteca, leggere un documento e fargli domande tramite il Model Context Protocol. È di sola lettura, gira sul tuo computer e raggiunge solo i documenti restituiti dalla ricerca.
- **Un riepilogo delle novità** — `cartacolibri digest NOME` rilancia una ricerca salvata e segnala solo i documenti comparsi dall’ultima volta. Aggiungi `--every 60` per ripetere ogni ora, `--out rapporto.md` per accodare a un file e `--summarize` per un breve riassunto della tua IA locale.
- **Sapere quanto fidarsi di una risposta** — Ogni risposta della chat mostra ora un punteggio «Aderenza alle fonti»: quanta parte di ogni frase si trova davvero nelle fonti citate, con le frasi da ricontrollare e i numeri che non compaiono in nessuna fonte.
- **Confronta due documenti e vedi tutti i nomi e i numeri** — Spunta due documenti e premi «Confronta 2» per vedere, frase per frase, cosa differisce tra due versioni. «Nomi e numeri» elenca tutte le e-mail, i link, gli importi, le date e i telefoni della tua biblioteca e dove compaiono. Da riga di comando: `compare` ed `entities`.
- **Individua le contraddizioni, vedi la mappa dei temi, segui cosa è cambiato** — «Contraddizioni» elenca frasi di documenti diversi uguali tranne un numero o una data (500 contro 550). «Mappa dei temi» raggruppa i documenti per argomento. «Novità» elenca cosa è cambiato da una data. Da riga di comando: `conflicts`, `topics`, `changes --since`.
- **Verifica il tuo testo rispetto alla biblioteca** — Incolla una bozza in «Verifica il mio testo» e ogni frase è segnata come confermata, in parte, numeri diversi o non trovata, con la migliore prova dai tuoi documenti. «Dossier» raccoglie tutto ciò che la biblioteca dice di un nome o termine, con date e importi. Le altre app di IA li ottengono tramite il server MCP.
- **Chiedi su un intervallo di tempo e ricevi i riepiloghi come file e-mail** — Nella chat scegli «Documenti dal … al …» e la risposta viene solo dai documenti modificati in quel periodo (da riga di comando: `chat --since --until`). La mappa dei temi è ora un’immagine di bolle cliccabili. `digest --mail-to te@example.org` salva ogni rapporto anche come file e-mail nella cartella outbox; aggiungi `--sendmail` per passarlo al programma di posta del tuo computer. Nient’altro lascia il tuo computer.
- **Collega i servizi di chat che conosci e confrontali** — `cartacolibri providers` elenca Ollama, LM Studio, Mistral, OpenAI, Gemini, Cohere, NVIDIA, Claude e Meta; `--provider NOME:MODELLO` ne usa uno per chat e attività, e `versus` pone la stessa domanda a più servizi sugli stessi passaggi. I servizi cloud richiedono `--allow-cloud` e una chiave, perché i tuoi passaggi vengono inviati.
- **Cronologia della chat e pulsante verso il browser web-grab** — Sotto la chat c’è ora l’elenco delle tue conversazioni: cercale, aprine una per rileggerla e continuare da dove eri rimasto, eliminane una o tutte. Restano solo nel tuo browser. In «Importa da altrove» un nuovo pulsante apre il browser web-grab sulla stessa libreria. Puoi anche salvare tutta la cronologia in un file e reimportarla; è lo stesso formato aperto di Colibri Companion ([docs/CHAT-HISTORY.it.md](../CHAT-HISTORY.it.md)).
- **OCR con LegatorOCR, «Elimina i selezionati», motivi di esclusione leggibili** — Scansioni e foto si possono ora leggere con LegatorOCR, lo stesso programma di Colibri Companion (tesseract funziona ancora). Un nuovo pulsante «Elimina i selezionati», `cartacolibri forget` e una ✕ accanto ai file saltati li tolgono per sempre dalla libreria, senza toccare i file. I motivi per cui un file è stato saltato appaiono nella tua lingua e il limite di 80 MB si alza con CARTACOLIBRI_MAX_FILE_MB.
- **Trova le copie e leggi la biblioteca come una cronologia** — «Duplicati» elenca i documenti che sono copie o quasi-copie, con quanto si somigliano. «Cronologia» mette tutte le frasi con data della biblioteca in ordine di tempo, con il documento di provenienza. Da riga di comando: `dupes`, `chronology`.
- **Ricerca per significato, misura della ricerca, più formati** — `embed` prepara la ricerca per significato con il modello di embedding del tuo motore locale; poi `search --mode hybrid` trova testi che usano parole diverse dalla tua domanda e `eval` misura quanto bene ogni modalità trova ciò che ti aspetti. Nuovi formati: tar e tar.gz, fogli e presentazioni OpenDocument, file Office con macro e i vecchi doc, xls e ppt tramite antiword, catdoc o LibreOffice. Gli strumenti su tutta la biblioteca (nomi e numeri, copie, conflitti, argomenti) sono passati da minuti a meno di un secondo su 20.000 documenti e i filtri ora vedono tutta la biblioteca.
- **Pulsanti sinceri e niente più stop dopo 10 minuti** — «Apri il browser web-grab» ora controlla che il browser si sia davvero aperto e, se no, mostra il motivo che ha dato. Un nuovo pulsante «Apri LegatorOCR», vicino alle opzioni OCR, apre la finestra di LegatorOCR. Un’attività o una chat lunga (un modello grande su un gruppo grande può richiedere molti minuti) non viene più fermata dalla pagina dopo 10 minuti con «The operation was aborted»: conta solo il limite del motore e un cronometro mostra da quanto sta andando.
- **LegatorOCR si trova meglio e le scansioni larghe si leggono intere** — CartaColibri ora cerca LegatorOCR accanto a sé, in una cartella `App` e nelle cartelle di un archivio decompresso, e nota una copia aggiunta senza riavvio. Le scansioni JPG larghe non sono più tagliate in due, quindi non si perdono righe, e i PDF scansionati passano dall’OCR anche con poppler installato. «Apri LegatorOCR» crea anche una cartella `from-legatorocr` che fa parte della biblioteca: salvi lì un PDF o un testo ed entra nella biblioteca alla prossima indicizzazione.

## Novità della 0.2.0-alpha.2

- **L’indirizzo del motore funziona da solo** — `http://127.0.0.1:8000` e `http://127.0.0.1:8000/v1` vanno entrambi bene. CartaColibri legge l’elenco dei modelli da `/v1/models`, sceglie da solo il modello caricato e non ha più bisogno di una pagina `/health`. Questo risolve il messaggio «AI engine not running (404)».
- **Trova da solo la tua IA** — CartaColibri cerca un motore in esecuzione su questo computer: le impostazioni di Colibri Companion, un file di scoperta, poi le solite porte locali (8000, 8080, 11434, 1234, 8001, 5000). Vengono sondati solo indirizzi loopback. Apri ⚙ per vedere cosa ha trovato, oppure premi «Avvia Colibri Companion».
- **Stato e velocità in tempo reale** — L’intestazione mostra il modello in esecuzione e se è stato trovato in automatico, aggiornato ogni pochi secondi. Sotto ogni risposta vedi i token usati e la velocità; sotto i 3 token/s ricevi il consiglio di scegliere un modello più piccolo. Le richieste vanno al motore una alla volta.
- **Pagine web dalla finestra di importazione** — In «Importa da altrove» i feed di notizie hanno lasciato il posto a web-grab: incolla un indirizzo e salva la pagina come un unico file HTML o MHTML nella biblioteca. I feed ci sono ancora, ripiegati.
- **Tre pulsanti nel browser web-grab** — «Save to folder» conserva la copia di testo nella biblioteca, «Grab website HTML» salva la pagina (Maiusc+clic per MHTML), «PDF» la stampa in PDF. Il PDF è silenzioso su Linux; su Windows e macOS si apre la finestra di stampa e scegli «Salva come PDF».
- **Le pagine salvate si possono cercare** — I file `.mhtml` e `.html` sono indicizzati con il set di caratteri giusto, così una pagina salvata si trova come qualsiasi documento. Le parti di una pagina su indirizzi privati non vengono mai scaricate né collegate.

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

## Altri comandi

| Comando | Cosa fa |
|---|---|
| `start [--folder F] [--port N] [--no-open] [--no-serve]` | Avvio guidato: scegli una cartella, viene indicizzata e si apre la pagina web. |
| `doctor` | Controlla motore, OCR, strumenti PDF, disco e porta e indica la correzione esatta per ogni problema. |
| `add FOLDER`, `remove FOLDER`, `folders`, `index [FOLDER]`, `problems` | Registra o dimentica una cartella (i file non vengono mai toccati), elenca le cartelle, reindicizza ed elenca i file illeggibili. |
| `watch [--interval S]` | Mantiene l’indice aggiornato: riscansiona le cartelle ogni S secondi. |
| `search QUERY [--ext pdf,txt] [--min-kb N] [--max-kb N] [--path P] [--group G] [--mode bm25\|semantic\|hybrid] [--json]` | Cerca nella biblioteca. Parole, «frasi», prefisso*, AND/OR/NOT e filtri nella query: `ext:pdf` `path:contratti` `size:>1mb` `after:2025-01` `before:2025-06-30` `re:FATT-\d+`. |
| `list [--group G] [--json]`, `group set/list/show/rm/from-search` | Elenca i documenti; i gruppi sono insiemi di documenti con nome in cui cercare o su cui fare domande. |
| `summarize`, `extract`, `synthesize` (same options as `task`, with `--fields`) | Scorciatoie per le tre attività principali: riassumere ogni documento, estrarre campi o combinarne più di uno in una risposta. |
| `grab URL`, `bookmarklet` | Salva una pagina web nella biblioteca (HTML, MHTML o PDF); il bookmarklet lo fa da qualsiasi browser. |
| `serve [--port N] [--url ENGINE]` | Avvia la pagina web su questo computer (http://127.0.0.1:PORT). |
| `completions bash\|zsh\|fish\|powershell`, `import-json FILE` | Stampa uno script di completamento con Tab per la tua shell; reimporta tag, stelle, note e gruppi esportati in JSON. |
| `task NAME [request] --group G \| --tag T \| --query Q [--mock] [--verify] [--lang xx] [--save md\|html\|txt]` | i 14 compiti (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` segnala le frasi il cui `[S#]` non è sostenuto dal testo |
| `chat [--group G] [--since DATE] [--until DATE] [--lang xx] [--fallback]` | domande in sequenza sulla biblioteca; risposte citate, oppure «non trovato nei tuoi documenti» |
| `tag add/rm/list/show`, `star [--off]`, `note` | la tua organizzazione personale, conservata dopo la reindicizzazione |
| `saved add/list/run/rm` | ricerche salvate / raccolte intelligenti |
| `recent`, `related PATH` | ciò che hai aperto di recente; documenti simili |
| `export [--saved NAME] --format md\|csv\|json\|zip` | condividi un gruppo, un’etichetta o una ricerca salvata, con citazioni |
| `backup OUT [--index]`, `backup-check ZIP`, `restore ZIP` | backup in un solo file con checksum; controllalo; ripristinalo |
| `verify [--repair]` | controlla l’indice; con `--repair` lo ricostruisce dai tuoi file |
| `stats`, `timeline` | numeri sulla tua biblioteca; documenti per mese |
| `import PATH` | segnalibri `.html`, cartella Obsidian, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex` |
| `feeds add/list/rm/poll`, `feeds opml-import/opml-export` | iscrizioni RSS/Atom salvate come note |
| `webwatch`, `history` | pagine web monitorate con differenze; cronologia di navigazione opzionale |
| `forget PATH… \| --list \| --undo PATH…` | Toglie file dalla libreria in modo definitivo (come «Elimina i selezionati» nella pagina web): i file stessi non vengono toccati e una reindicizzazione non li riporta. `--list` li mostra, `--undo` li riammette. |
| `providers [--json]` | Elenca i servizi di chat a cui collegarti (Ollama, LM Studio, Mistral, OpenAI, Gemini, Cohere, NVIDIA, Claude, Meta) e quali sono pronti. Usane uno con `--provider NOME[:MODELLO]` su `chat`, `task`, `summarize`…; quelli cloud richiedono anche `--allow-cloud` e una chiave, perché i tuoi passaggi vengono inviati. |
| `versus "QUESTION" [--providers mock,colibri,ollama:llama3.2,mistral:MODEL…] [--group G] [--lang xx] [--allow-cloud] [--json]` | Pone la stessa domanda a più modelli sugli stessi passaggi e li confronta: tempo, lunghezza, citazioni e quante sembrano non supportate, poi ogni risposta. |
| `digest NAME [--every MIN] [--out FILE.md] [--summarize] [--all] [--mail-to ADDR [--sendmail]]` | Rilancia una ricerca salvata e segnala solo i documenti nuovi dall’ultima volta; può ripetersi, accodare a un file, riassumere con la tua IA locale o salvare un file e-mail nella cartella outbox. |
| `entities [--kind email\|url\|money\|date\|phone] [--group G] [--json]` | Tutte le e-mail, i link, gli importi, le date e i telefoni dei tuoi documenti, con i documenti in cui compaiono. |
| `compare A B [--json]` | Confronta due documenti frase per frase: quanto sono simili e cosa c’è solo in uno (A e B sono percorsi o le loro terminazioni). |
| `conflicts [--group G] [--json]` | Coppie di frasi di documenti diversi uguali tranne un numero o una data (500 contro 550): candidati da verificare, non verdetti. |
| `dupes [--group G] [--min 0.6] [--json]` | Trova i documenti che sono copie o quasi-copie l’uno dell’altro (esportazioni, bozze, lo stesso file salvato due volte), con quanto si somigliano e quanta parte del più piccolo è contenuta nell’altro. |
| `chronology [--group G] [--limit N] [--json]` | Tutte le frasi con data della biblioteca in ordine di tempo, con il documento di provenienza (date come 2025-03-12, 12.03.2025 o 12/03/2025, prima il giorno; o con il nome del mese in una delle 8 lingue: 12 marzo 2025, 12 March 2025). Il conteggio mensile dei documenti è `timeline`. |
| `embed [--embed-model M] [--batch 32] [--limit N] [--reset] [--mock] [--json]` | Prepara la ricerca per significato: il modello di embedding del tuo motore locale trasforma ogni passaggio in un vettore conservato nell’indice della biblioteca. Incrementale (solo passaggi nuovi o cambiati), riprendibile (`--limit`), e nulla lascia il tuo computer. Poi `search --mode semantic` o `--mode hybrid` (parole e significato insieme) trova testi che usano parole diverse dalla tua domanda. |
| `eval FILE [--modes bm25,semantic,hybrid] [--json]` | Misura quanto bene la ricerca trova ciò che ti aspetti. FILE ha una domanda per riga con i file attesi (`domanda<TAB>file1,file2`) oppure è JSON (`[{"q": "...", "expect": ["lease.txt"]}]`). Stampa hit@1, hit@5, hit@10 e MRR per ogni modalità e le domande non trovate. |
| `topics [--json]` | Raggruppa i documenti per argomento (le loro parole più caratteristiche). |
| `dossier TERM [--json]` | Tutto ciò che la biblioteca dice di un nome o termine: passaggi per documento, più date e importi trovati. |
| `changes --since DATE [--json]` | Documenti modificati da una data (YYYY, YYYY-MM o YYYY-MM-DD), i più recenti prima. |
| `factcheck FILE\|- [--json]` | Verifica un tuo testo rispetto alla biblioteca: ogni frase è segnata come confermata, in parte, numeri diversi o non trovata, con la migliore prova. |
| `mcp [--url ENGINE] [--mock]` | Server MCP via stdio perché altre app di IA cerchino, leggano, interroghino, verifichino e chiedano dossier nella tua biblioteca (sola lettura). |

## Come si incastra

Livello 0 — le tue cartelle (sola lettura) → RAG-fetch — indice, ricerca, gruppi → RAG-CARTA — compiti IA tramite il motore Colibrì.

## Domande frequenti

**Cosa significa «locale»?**  
Tutto gira sul tuo computer: la pagina, l’indice di ricerca e il modello IA. Funziona anche senza rete. L’unica eccezione è web-grab, che scarica le pagine web che gli chiedi.

**I miei dati vengono inviati da qualche parte?**  
No. Niente cloud, niente account, niente telemetria, niente statistiche d’uso. L’unico traffico che esce dal tuo computer sono le pagine web che chiedi esplicitamente a web-grab di recuperare. Dettagli in «Privacy e sicurezza».

**Colibrì non è in esecuzione. Posso usarla lo stesso?**  
Sì. La Biblioteca e la ricerca non hanno mai bisogno di un modello. Per riassumere, estrarre e sintetizzare spunta **Modalità offline (senza modello)** (`--mock` da riga di comando): ottieni una risposta semplice costruita dai tuoi file, con citazioni, ma senza vera formulazione IA. Per risposte vere avvia il motore Colibrì.

**Cos’è Colibrì e mi serve?**  
[Colibrì](https://github.com/JustVugg/colibri) è un motore locale separato che esegue il modello linguistico. Ti serve solo per le risposte IA. Scegli un modello il cui file sia più piccolo della tua RAM libera e parti da uno piccolo. Vedi la documentazione di Colibrì.

**Quali file sa leggere?**  
Testo e appunti (txt, md, rst, log, csv, json, yaml, html, rtf), PDF, file Word, OpenDocument, PowerPoint ed Excel (docx, odt, pptx, xlsx), epub e, con l’OCR attivo, immagini e PDF scansionati. Anche e-mail (eml, mbox), calendari e contatti (ics, vcf), archivi zip, tar e tar.gz, fogli e presentazioni OpenDocument (ods, odp) e i vecchi doc, xls e ppt se sono installati antiword, catdoc o LibreOffice.

**E le scansioni e le foto di documenti?**  
Attiva l’OCR (`--ocr`, oppure `CARTACOLIBRI_OCR=1`). Serve `tesseract` installato sul tuo computer, più `pdftoppm` per i PDF scansionati. La pagina ti avvisa quando manca tesseract.

**Come aggiorno?**  
Versione portatile: scarica lo zip più recente e decomprimilo. Dai sorgenti: `git pull`, poi `cargo build --release`. I tuoi dati in `~/.cartacolibri` restano dove sono e l’indice si aggiorna da solo, solo per ciò che è cambiato.

**Come elimino tutto?**  
Elimina la cartella del programma, poi la cartella di stato `~/.cartacolibri` (o quella indicata da `CARTACOLIBRI_HOME`). Fine: i tuoi documenti non sono mai stati toccati.

**Qualcosa di particolare su Windows o macOS?**  
Lo zip portatile esiste per entrambi, ma Linux è il sistema meglio testato. Il browser web-grab viene compilato in CI per Windows e macOS e lì non è mai stato avviato a mano. Su Windows richiede il runtime WebView2. Se il sistema ti chiede di confermare l’apertura di un programma scaricato, conferma solo se lo hai preso dalla pagina del progetto.

**Perché «alfa»?**  
Perché è giovane: il nucleo funziona ed è testato, ma non su ogni sistema, e i controlli automatici su GitHub non sono ancora verificati. Aspettati qualche spigolo e segnalacelo.

**Può cercare per significato, non solo per parole?**  
Sì, in modo facoltativo. Esegui una volta `cartacolibri embed`: il modello di embedding del tuo motore locale trasforma ogni passaggio in numeri, salvati nel tuo file di indice (poi si rifanno solo i passaggi nuovi o cambiati). Poi `search --mode hybrid` (o `semantic`) trova un testo anche se usa parole diverse dalla tua domanda. Serve un modello di embedding nel motore (con Ollama: `ollama pull nomic-embed-text`). Senza, la ricerca per parole funziona come prima. `cartacolibri eval` misura quanto bene ogni modalità trova ciò che ti aspetti.

**Cosa non sa fare?**  
Non ha agenti che agiscono da soli, ricerca sul web, account utente, app per telefono né sincronizzazione tra computer. La ricerca per parole è esatta, ma gli strumenti di analisi (conflitti, duplicati, cronologia, nomi e numeri) usano regole su parole e numeri, non comprensione: indicano cose da controllare, non sono verdetti. Le risposte valgono quanto il modello che esegui in locale. Le 8 traduzioni non sono state riviste da madrelingua e il progetto è in alpha. Le e-mail .msg e .pst non si leggono (esportale in .eml o .mbox) e audio e video non vengono trascritti.

**Come sopravvivono etichette, stelle e note alla reindicizzazione?**  
Non sono salvate nell’indice di ricerca, ma in un piccolo file a parte, `organize.json`, nella tua cartella `<home>`, legato al percorso di ogni documento. Quindi **Aggiorna indice**, una reindicizzazione completa e perfino **Controlla/Ripara** (che ricostruisce l’indice) non le toccano. Se rinomini o sposti un file, conta come un nuovo documento: le etichette restano sul vecchio percorso.

**Come faccio il backup e il ripristino?**  
Apri **Statistiche** e premi **Scarica backup** (spunta la casella per includere l’indice), oppure esegui `cartacolibri backup my.zip`. È un unico zip con checksum, che contiene etichette, note, gruppi, ricerche salvate e note importate, mai i tuoi documenti originali. `cartacolibri backup-check my.zip` lo verifica e `cartacolibri restore my.zip` rimette tutto a posto. Tieni lo zip lontano da questo computer.

**Come importo segnalibri, Obsidian o Zotero?**  
Clicca **Porta dentro** nella Biblioteca, oppure esegui `cartacolibri import PERCORSO`. Capisce un’esportazione di segnalibri del browser (`.html`), una cartella Obsidian, un file Zotero/BibTeX `.bib`, file `.csv` di Pocket o Raindrop ed esportazioni Evernote `.enex`. Ogni elemento diventa una piccola nota nella tua biblioteca, che puoi cercare, etichettare e citare. Gli originali vengono solo letti.

**Come funzionano i feed?**  
In **Porta dentro → Feed**, incolla un indirizzo RSS o Atom e premi **Aggiungi**. Quando premi **Controlla ora** (o esegui `cartacolibri feeds poll`), le novità vengono salvate come note e indicizzate, così puoi cercarle e fare domande. Niente gira da solo in background. Puoi importare ed esportare le iscrizioni in OPML.

**Cosa fa «verifica le citazioni»?**  
Dopo la risposta dell’IA, CartaColibri rilegge ogni frase. Se una frase non ha una citazione `[S#]`, o la fonte citata non contiene davvero ciò che dice la frase, viene contrassegnata `[unsupported]`. È una rete di sicurezza veloce, non una garanzia: apri la fonte per tutto ciò che conta.

**Perché una pagina web è stata rifiutata?**  
web-grab recupera solo pagine pubbliche. Gli indirizzi della tua macchina o di una rete privata (`localhost`, `192.168.x.x`, `10.x.x.x` e simili) vengono rifiutati, così nessuna pagina web può indurlo a leggere i tuoi servizi interni. Se lo vuoi davvero, per esempio per la tua intranet, avvia CartaColibri con `CARTACOLIBRI_GRAB_LOCAL=1`.

## Quando qualcosa non va

Prova prima `cartacolibri doctor`: controlla la tua installazione e ti dice cosa sistemare.

| Sintomo | Causa probabile | Cosa fare |
|---|---|---|
| `cartacolibri: command not found` | La cartella del programma non è nel tuo PATH. | Avvialo con il percorso completo, ad esempio `./target/release/cartacolibri`, oppure usa il lanciatore dello zip portatile. |
| La pagina non si apre su `http://127.0.0.1:8765/` | La pagina locale non è in esecuzione, oppure la porta è occupata. | Esegui `cartacolibri start`. Se la porta è occupata, esegui `cartacolibri serve --port 8766` e apri quell’indirizzo. |
| Non sai cosa non va | Manca qualcosa nella configurazione. | Esegui `cartacolibri doctor`: controlla la configurazione e ti dice cosa sistemare. |
| I compiti IA dicono di non raggiungere Colibrì | Il motore Colibrì non è in esecuzione, oppure ascolta su un altro indirizzo (predefinito `http://127.0.0.1:8000`). | Avvialo (`coli web --model ~/colibri-models/<model>`) o passa il suo indirizzo con `--url`. Nel frattempo spunta **Modalità offline (senza modello)**. |
| I file nuovi non compaiono nella ricerca | L’indice non li ha ancora visti. | Clicca **Aggiorna indice** o esegui `cartacolibri index`. `cartacolibri watch` lo tiene aggiornato da solo. |
| Alcuni file sono elencati in «File saltati» | Sono corrotti, protetti, non supportati o scansioni. | Esegui `cartacolibri problems` per vedere perché. Per le scansioni attiva l’OCR. |
| Scansioni e immagini non danno testo | L’OCR è spento oppure `tesseract` non è installato. | Installa `tesseract` (e `pdftoppm` per i PDF) e aggiungi `--ocr`, oppure imposta `CARTACOLIBRI_OCR=1`. |
| La risposta dice NOT FOUND | I file scelti non contengono l’informazione. È voluto: non tira mai a indovinare. | Controlla che sia selezionato il gruppo giusto, riformula la domanda o aggiungi al gruppo i file mancanti. |
| Il browser web-grab non si apre | Manca un componente di sistema. | Linux: installa `libwebkit2gtk-4.1`. Windows: installa il runtime WebView2. Nel frattempo `cartacolibri grab URL` funziona senza. |
| Il tuo script riceve `403 missing X-Colibri-UI header` | È un controllo di sicurezza, non un errore. | Invia l’intestazione `X-Colibri-UI: 1` a ogni chiamata a `/carta/*`. |
| L’indice sembra rotto: risultati strani, file mancanti, un errore sul database | Il file dell’indice si è danneggiato (per esempio per un arresto anomalo o un disco pieno) o non è più allineato alle tue cartelle. | Apri **Statistiche → Controlla / Ripara**, oppure esegui `cartacolibri verify --repair`. L’indice viene ricostruito dai tuoi file; etichette, note e gruppi restano. |
| Un file è stato saltato perché «troppo lento» | La lettura ha superato il limite di tempo (predefinito 120 secondi), di solito un PDF enorme o danneggiato, o una scansione molto grande. | Alza il limite, per esempio `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, poi **Aggiorna indice**. Oppure dividi / riesporta il file. Il motivo compare sotto «File saltati». |
| «AI engine not running … (404)» | L’indirizzo punta a qualcosa che non è il motore, oppure non c’è nessun modello caricato. | Apri ⚙, premi «Cerca il motore» e scegli l’indirizzo indicato; vanno bene sia `…:8000` sia `…:8000/v1`. Controlla che in Colibri Companion sia caricato un modello. |

## Glossario in parole semplici

- **Indice** — Un catalogo delle parole contenute nei tuoi file, conservato perché una ricerca richieda un attimo invece di rileggere tutto.
- **RAG** — «Retrieval-augmented generation»: prima si trovano i passaggi rilevanti nei tuoi file, poi l’IA risponde usando solo quelli.
- **Citazione [S1]** — Un segnaposto come [S1] o [S2] in una risposta. Rimanda al file sorgente da cui viene la frase, così puoi verificare.
- **Gruppo** — Un elenco con nome di file (un progetto, un tema) su cui puoi cercare o fare domande ancora e ancora.
- **Gateway** — Il piccolo indirizzo web (predefinito `http://127.0.0.1:8000`) tramite cui CartaColibri parla con il motore Colibrì.
- **Istantanea (snapshot)** — Una copia HTML salvata di una pagina web, fatta a ogni salvataggio con web-grab, per tenere ciò che diceva quel giorno.
- **Watch (monitoraggio)** — Due cose: `cartacolibri watch` tiene aggiornato l’indice, e `cartacolibri webwatch` riscarica le pagine che segui e mostra cosa è cambiato.

## Privacy e sicurezza

- **Dove stanno le cose.** Tutto ciò che CartaColibri scrive sta in una sola cartella, `<home>`: per impostazione predefinita `~/.cartacolibri`, o la cartella indicata da `CARTACOLIBRI_HOME`. Dentro: `index.sqlite` (l’indice di ricerca), `groups.json`, `library/uploads/` (i file che carichi), `outputs/` (i risultati che hai scelto di salvare), `snapshots/` e `web.json` (le pagine web salvate), `highlights.json`, `watch.json`, `settings.json` e `clip.token` (un segreto per il bookmarklet: non condividerlo).
- **I tuoi originali sono in sola lettura.** CartaColibri legge i tuoi documenti e non li modifica, sposta o elimina mai.
- **La pagina è solo locale.** Il server ascolta soltanto su `127.0.0.1`, quindi gli altri computer della tua rete non possono raggiungerlo. Rifiuta anche le richieste il cui indirizzo o la cui origine non sia locale.
- **L’intestazione `X-Colibri-UI`.** Ogni chiamata della pagina al proprio server porta questa intestazione. Una pagina web visitata in un’altra scheda non può aggiungerla, quindi non può far contattare l’app locale al tuo browser alle tue spalle.
- **Niente esce da `127.0.0.1`, tranne le pagine che chiedi di recuperare.** Niente telemetria, niente statistiche, niente cloud. web-grab recupera solo gli indirizzi che gli dai (e le pagine che hai scelto di monitorare). Anche il sito di presentazione non carica nulla da altri server.
- **La cronologia è spenta per impostazione predefinita.** La cronologia del browser web-grab esiste solo se la attivi (`cartacolibri history on`), resta sul tuo disco, e `cartacolibri history clear` la cancella.

## Stato reale

CartaColibri è un software **alfa** (versione 0.2.0-alpha): funziona, ma aspettati qualche spigolo e tieni copie di sicurezza di ciò che conta. Linux è la piattaforma meglio testata: lì il browser web-grab è autotestato in CI. Su **Windows e macOS** il browser viene compilato in CI ma **non è mai stato avviato a mano**. I controlli automatici su GitHub (CI) passano su Linux, Windows e macOS. I tuoi documenti sono al sicuro comunque, perché vengono solo letti. I download portatili per Linux, Windows e macOS sono nella pagina **Releases** di GitHub (contrassegnati come pre-release).

## Sicurezza

Hai trovato una vulnerabilità? Segnalala in privato, come descritto in [SECURITY.md](../../SECURITY.md); per favore non aprire prima una issue pubblica. Come si difende il server (solo loopback, intestazione `X-Colibri-UI`, nessun indirizzo privato in web-grab, originali in sola lettura) è descritto in [docs/threat-model.md](../threat-model.md).

## Licenza e crediti

CartaColibri è rilasciato con **licenza MIT**. È un’implementazione Rust clean-room: parla con il motore [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) solo tramite il suo gateway HTTP pubblico e non ne contiene codice. Il livello di indicizzazione e ricerca segue il comportamento descritto nel manuale pubblico di DocFetcher; non si usa codice DocFetcher. Il browser web-grab si basa sulla webview del sistema operativo e non contiene codice di Falkon.

**Alexandru-Ionuț Chiuță** · **Alexandru Dumbrava** (coautore)

## Per saperne di più

[Documentazione completa](../cartacolibri.md) · [README del motore Colibrì](https://github.com/JustVugg/colibri) · [Contribuire](../../CONTRIBUTING.md) · [📄 Tutorial PDF illustrato](CartaColibri-Tutorial.it.pdf)
