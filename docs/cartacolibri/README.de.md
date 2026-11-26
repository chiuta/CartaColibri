<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Deine Dokumente, durchsuchbar und erklärt — 100 % lokal.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · <b>Deutsch</b> · [Magyar](README.hu.md)</p>

CartaColibri ist eine lokale KI-Bibliothek auf Basis der Colibrì-Engine. Sie indiziert die Ordner, die du auswählst, lässt dich blitzschnell darin suchen und bittet dann ein lokales Sprachmodell, Dateigruppen zusammenzufassen, Daten daraus zu extrahieren oder sie zu synthetisieren — immer mit Quellenangabe. Nichts verlässt deinen Rechner, und deine Originale werden nie verändert.

## Deine ersten 5 Minuten

Du brauchst keine Vorkenntnisse. Drei Schritte, und du hast deine eigenen Dateien durchsucht und zusammengefasst.

1. **Starte sie.** Entpacke das portable Zip und führe den Starter aus, oder tippe `cartacolibri start` in ein Terminal. Dein Browser zeigt die lokale Seite `http://127.0.0.1:8765/`.
2. **Füge einen Ordner hinzu.** Öffne auf der Seite **Bibliothek**, gib einen Ordnerpfad wie `~/Documents` ein und klicke auf **Ordner hinzufügen**. Das erste Indizieren eines großen Ordners kann ein paar Minuten dauern. Deine Dateien werden nur gelesen, nie verändert.
3. **Suchen, dann fragen.** Öffne **RAG-fetch**, suche ein Wort, hake ein paar Dateien an und erstelle eine Gruppe. Öffne dann **RAG-CARTA**, wähle **Zusammenfassen** und klicke auf **Starten**. Noch kein KI-Modell? Aktiviere **Offline-Modus (ohne Modell)** und probiere es trotzdem aus.

Lieber im Terminal? Dieselben drei Schritte:

```bash
cartacolibri start                 # step 1 - leave this window open (use a second one for the rest)
cartacolibri add ~/Documents       # step 2
cartacolibri group set Demo ~/Documents
cartacolibri summarize "main points" --group Demo --mock   # step 3 - --mock = no AI model needed
```

## Was sie kann

- **Bibliothek** — alle lokalen Dokumente mit ihrem Speicherort; Ordner hinzufügen oder entfernen, Dateien hochladen, ansehen, auf Wunsch OCR-Text speichern.
- **RAG-fetch** — inkrementeller Volltextindex im DocFetcher-Stil (Wörter, „Phrasen“, Präfix*, AND/OR/NOT, Filter nach Typ, Größe, Ort) und wiederverwendbare **Gruppen** von Dateien.
- **RAG-CARTA** — drei lokale Aufgaben auf einer Gruppe: **zusammenfassen**, **extrahieren** (Tabelle oder JSON, „NOT FOUND“ statt Vermutungen), **synthetisieren** (Themen, Gemeinsamkeiten, Widersprüche). Jede Antwort zitiert [S1], [S2] … und listet ihre Quellen.
- **Privat von Grund auf** — keine Cloud, keine Telemetrie; Originale werden nur gelesen. 8 Sprachen: EN, RO, FR, IT, ES, PT, DE, HU.
- **web-grab** — ein kleiner Browser in Rust (Clean-Room) mit Tabs, der deine Bibliothek kennt: ein Abzeichen, wenn die Seite schon gespeichert ist (und wenn sie sich geändert hat), verwandte Dokumente beim Surfen, Speichern mit einem Klick, Fragen an dein lokales Colibrì zu einer Seite — oder zu allen offenen Tabs, mit Quellenangaben. Ein Befehl und ein Bookmarklet decken andere Browser ab. Außerdem: 🖍 Markierungen, die mit der Seite gespeichert werden, 👁 beobachtete Seiten mit Diff der Änderungen (`cartacolibri webwatch`), HTML-Schnappschuss bei jedem Speichern, optional durchsuchbarer Verlauf (`cartacolibri history on`), 🗂📝📊 Zusammenfassen oder Extrahieren über alle Tabs und `?Frage` in der Adressleiste, um die ganze Bibliothek zu fragen.

## Power-Funktionen

- **Finden** — ein Fortschrittsbalken beim Indizieren; „Meintest du …?“ bei Tippfehlern, mit Vorschlägen, wenn nichts passt; Duplikate aufgelistet und ein klarer Grund für jede übersprungene Datei. Tippe Filter direkt ins Feld (`ext:pdf`, `path:`, `size:>1mb`, Daten `after:`/`before:`, reguläre Ausdrücke `re:`), hake **Intelligente Suche** an, um verwandte Wörter zu ergänzen, und sieh, wie viele Treffer es je Dateityp gibt.
- **Ordnen** — Tags, ein Stern und eine private Notiz an jedem Dokument; sie überstehen das Neuindizieren. Sieh verwandte Dokumente, was du zuletzt und am häufigsten geöffnet hast, und speichere Suchen als intelligente Sammlungen.
- **14 Aufgaben** — vergleichen, Zeitleiste, Glossar, Gliederung, Aufgabenliste, FAQ, Lernkarten, Quiz, übersetzen, erklären, Widersprüche, dazu zusammenfassen, extrahieren und synthetisieren. Hake **Zitate prüfen** an, um Sätze zu markieren, die die Quellen nicht stützen, und wähle die Antwortsprache.
- **Chat** — stelle Folgefragen zu deiner ganzen Bibliothek oder einer Gruppe. Die Antworten nennen ihre Quellen oder sagen ehrlich „in deinen Dokumenten nicht gefunden“, statt zu raten. Jede Antwort zeigt einen Wert **Belegt durch Quellen** und markiert Sätze oder Zahlen, die die Quellen nicht stützen. Hake „Jede Frage einzeln“ an, damit nichts gemerkt wird, oder begrenze die Dokumente auf einen Zeitraum.
- **Teilen und sichern** — exportiere eine Gruppe, ein Tag oder eine Suche als Markdown, CSV, JSON oder Zip, mit Quellenangaben; eine Statistik-Ansicht mit Diagrammen und Zeitleiste; ein Backup in einer einzigen Datei mit Prüfsummen; Prüfen/Reparieren für den Index.
- **Importieren** — hole Browser-Lesezeichen, einen Obsidian-Ordner, Zotero/BibTeX, Pocket/Raindrop-CSV oder Evernote-Exporte herein und abonniere RSS/Atom-Feeds (OPML rein und raus): Sie werden zu durchsuchbaren Notizen. E-Mails, Kalender, Untertitel und Notebooks werden ebenfalls indiziert.
- **Einblicke** — “Widersprüche” (gleicher Satz, andere Zahl oder anderes Datum), “Themenkarte” (Blasen), “Was ist neu” seit einem Datum, “2 vergleichen”, “Namen & Zahlen” (E-Mails, Links, Beträge, Daten, Telefonnummern), “Dossier” zu einem Namen und “Meinen Text prüfen” gegen die Bibliothek.
- **Automatisieren und verbinden** — `digest` führt eine gespeicherte Suche erneut aus und meldet nur Neues (wiederholt, hängt an eine Datei an, fasst zusammen oder speichert eine E-Mail-Datei); `mcp` lässt andere KI-Apps in deiner Bibliothek suchen, lesen, fragen, prüfen und Dossiers abrufen, nur lesend.
- **Kopien und Daten** — “Duplikate” (Dokumente, die Kopien oder fast Kopien sind, mit ihrer Ähnlichkeit) und “Chronologie” (alle datierten Sätze in zeitlicher Reihenfolge).

## Neu in 0.2.0-alpha.7

- **Filter direkt im Suchfeld** — Tippe `ext:pdf`, `path:vertraege`, `size:>1mb`, `after:2025-01`, `before:2025-06-30` oder `re:RE-\d+` (regulärer Ausdruck) neben deine Wörter, und die Treffer werden eingegrenzt. Sie funktionieren auf der Webseite, in der Kommandozeile und über die API.
- **Intelligente Suche** — Hake „Intelligente Suche“ an, und deine lokale KI ergänzt ein paar Synonyme und verwandte Wörter (als einfaches OR, exakte Treffer stehen also weiter vorn). Die zusätzlichen Wörter erscheinen unter dem Feld. Ohne laufende Engine bekommst du die normale Suche.
- **Chat mit Einzelfragen** — Hake „Jede Frage einzeln“ an, und der Chat vergisst frühere Fragen: Jede Antwort stammt nur aus deinen Dokumenten, mit ihren Quellen.
- **Ein MCP-Server für andere KI-Apps** — `cartacolibri mcp` lässt andere KI-Anwendungen über das Model Context Protocol in deiner Bibliothek suchen, ein Dokument lesen und Fragen dazu stellen. Er ist nur lesend, läuft auf deinem Rechner und erreicht nur Dokumente, die die Suche geliefert hat.
- **Eine Zusammenfassung des Neuen** — `cartacolibri digest NAME` führt eine gespeicherte Suche erneut aus und meldet nur die Dokumente, die seit dem letzten Mal hinzugekommen sind. Mit `--every 60` wiederholt es stündlich, mit `--out bericht.md` hängt es an eine Datei an, mit `--summarize` liefert deine lokale KI eine kurze Zusammenfassung.
- **Wissen, wie weit du einer Antwort trauen kannst** — Jede Chat-Antwort zeigt jetzt einen Wert „Belegt durch Quellen“: wie viel jedes Satzes tatsächlich in den zitierten Quellen steht, mit den Sätzen zum Nachprüfen und Zahlen, die in keiner Quelle vorkommen.
- **Zwei Dokumente vergleichen und alle Namen und Zahlen sehen** — Hake zwei Dokumente an und drücke „2 vergleichen“, um Satz für Satz zu sehen, was sich zwischen zwei Fassungen unterscheidet. „Namen & Zahlen“ listet alle E-Mails, Links, Beträge, Daten und Telefonnummern deiner Bibliothek und wo sie vorkommen. In der Kommandozeile: `compare` und `entities`.
- **Widersprüche aufspüren, Themenkarte sehen, Änderungen verfolgen** — „Widersprüche“ listet Sätze aus verschiedenen Dokumenten, die bis auf eine Zahl oder ein Datum gleich sind (500 gegen 550). Die „Themenkarte“ gruppiert deine Dokumente nach Thema. „Was ist neu“ zeigt, was sich seit einem Datum geändert hat. In der Kommandozeile: `conflicts`, `topics`, `changes --since`.
- **Den eigenen Text gegen deine Bibliothek prüfen** — Füge einen Entwurf in „Meinen Text prüfen“ ein, und jeder Satz wird als belegt, teilweise, Zahlen weichen ab oder nicht gefunden markiert, mit dem besten Beleg aus deinen Dokumenten. Das „Dossier“ sammelt alles, was deine Bibliothek zu einem Namen oder Begriff sagt, mit Daten und Beträgen. Andere KI-Apps erreichen beides über den MCP-Server.
- **Zeitraum erfragen und Zusammenfassungen als E-Mail-Datei erhalten** — Wähle im Chat „Dokumente ab … bis …“, und die Antwort stammt nur aus Dokumenten, die in dieser Zeit geändert wurden (Kommandozeile: `chat --since --until`). Die Themenkarte ist jetzt ein Bild aus anklickbaren Blasen. `digest --mail-to du@example.org` speichert jeden Bericht auch als E-Mail-Datei im Ordner outbox; mit `--sendmail` übergibst du ihn dem Mailprogramm deines Rechners. Sonst verlässt nichts deinen Rechner.
- **Verbinde die Chat-Dienste, die du kennst, und vergleiche sie** — `cartacolibri providers` listet Ollama, LM Studio, Mistral, OpenAI, Gemini, Cohere, NVIDIA, Claude und Meta; `--provider NAME:MODELL` nutzt einen im Chat und bei Aufgaben, und `versus` stellt mehreren dieselbe Frage zu denselben Textstellen. Cloud-Dienste brauchen `--allow-cloud` und einen Schlüssel, weil deine Textstellen dorthin gesendet werden.
- **Kopien finden und die Bibliothek als Zeitleiste lesen** — „Duplikate“ listet Dokumente, die Kopien oder fast Kopien sind, mit ihrer Ähnlichkeit. „Chronologie“ ordnet alle datierten Sätze deiner Bibliothek zeitlich, mit ihrem Dokument. In der Kommandozeile: `dupes`, `chronology`.
- **Suche nach Bedeutung, Suche messen, mehr Formate** — `embed` bereitet die Bedeutungssuche mit dem Embedding-Modell deiner lokalen Engine vor; danach findet `search --mode hybrid` Texte, die andere Wörter benutzen als deine Frage, und `eval` misst, wie gut jeder Modus findet, was du erwartest. Neue Formate: tar und tar.gz, OpenDocument-Tabellen und -Präsentationen, Office-Dateien mit Makros sowie alte doc, xls und ppt über antiword, catdoc oder LibreOffice. Werkzeuge über die ganze Bibliothek (Namen und Zahlen, Kopien, Widersprüche, Themen) brauchen bei 20.000 Dokumenten jetzt unter einer Sekunde statt Minuten, und Filter sehen jetzt die ganze Bibliothek.

## Neu in 0.2.0-alpha.2

- **Die Engine-Adresse funktioniert einfach** — `http://127.0.0.1:8000` und `http://127.0.0.1:8000/v1` sind beide richtig. CartaColibri liest die Modellliste von `/v1/models`, wählt das geladene Modell selbst und braucht keine `/health`-Seite mehr. Das behebt die Meldung „AI engine not running (404)“.
- **Er findet deine KI selbst** — CartaColibri sucht auf diesem Rechner nach einer laufenden Engine: die eigenen Einstellungen von Colibri Companion, eine Discovery-Datei, danach die üblichen lokalen Ports (8000, 8080, 11434, 1234, 8001, 5000). Geprüft werden nur Loopback-Adressen. Öffne ⚙, um zu sehen, was gefunden wurde, oder drücke „Colibri Companion starten“.
- **Live-Status und Tempo** — Die Kopfzeile zeigt das laufende Modell und ob es automatisch gefunden wurde, alle paar Sekunden aktualisiert. Unter jeder Antwort siehst du die verbrauchten Tokens und die Geschwindigkeit; unter 3 Tokens/s gibt es den Hinweis auf ein kleineres Modell. Anfragen gehen nacheinander an die Engine.
- **Webseiten aus dem Import-Dialog** — Unter „Etwas hereinholen“ haben die Nachrichten-Feeds web-grab Platz gemacht: Adresse einfügen und die Seite als eine einzige HTML- oder MHTML-Datei in der Bibliothek speichern. Die Feeds sind weiterhin da, eingeklappt.
- **Drei Schaltflächen im web-grab-Browser** — „Save to folder“ legt die Textkopie in deiner Bibliothek ab, „Grab website HTML“ speichert die Seite (Umschalt+Klick für MHTML), „PDF“ druckt sie als PDF. Unter Linux läuft PDF lautlos; unter Windows und macOS öffnet sich der Druckdialog, und du wählst „Als PDF speichern“.
- **Gespeicherte Seiten sind durchsuchbar** — `.mhtml`- und `.html`-Dateien werden mit dem richtigen Zeichensatz indiziert, eine gespeicherte Seite wird also wie jedes Dokument gefunden. Teile einer Seite auf privaten Adressen werden nie geladen oder verlinkt.

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

## Weitere Befehle

| Befehl | Was er macht |
|---|---|
| `start [--folder F] [--port N] [--no-open] [--no-serve]` | Geführter Start: Ordner wählen, indizieren, Webseite öffnen. |
| `doctor` | Prüft Engine, OCR, PDF-Werkzeuge, Datenträger und Port und nennt für jedes Problem die genaue Lösung. |
| `add FOLDER`, `remove FOLDER`, `folders`, `index [FOLDER]`, `problems` | Ordner hinzufügen oder vergessen (Dateien bleiben unberührt), Ordner auflisten, neu indizieren und nicht lesbare Dateien anzeigen. |
| `watch [--interval S]` | Hält den Index aktuell: durchsucht die Ordner alle S Sekunden neu. |
| `search QUERY [--ext pdf,txt] [--min-kb N] [--max-kb N] [--path P] [--group G] [--mode bm25\|semantic\|hybrid] [--json]` | Durchsucht die Bibliothek. Wörter, „Phrasen“, Präfix*, AND/OR/NOT und Filter direkt in der Anfrage: `ext:pdf` `path:vertraege` `size:>1mb` `after:2025-01` `before:2025-06-30` `re:RE-\d+`. |
| `list [--group G] [--json]`, `group set/list/show/rm/from-search` | Listet Dokumente; Gruppen sind benannte Dokumentmengen, in denen du suchen oder über die du fragen kannst. |
| `summarize`, `extract`, `synthesize` (same options as `task`, with `--fields`) | Abkürzungen für die drei Hauptaufgaben: jedes Dokument zusammenfassen, Felder extrahieren oder mehrere zu einer Antwort verbinden. |
| `grab URL`, `bookmarklet` | Speichert eine Webseite in der Bibliothek (HTML, MHTML oder PDF); das Bookmarklet erledigt das in jedem Browser. |
| `serve [--port N] [--url ENGINE]` | Startet die Webseite auf diesem Rechner (http://127.0.0.1:PORT). |
| `completions bash\|zsh\|fish\|powershell`, `import-json FILE` | Gibt ein Tab-Vervollständigungsskript für deine Shell aus; importiert als JSON exportierte Tags, Sterne, Notizen und Gruppen erneut. |
| `task NAME [request] --group G \| --tag T \| --query Q [--mock] [--verify] [--lang xx] [--save md\|html\|txt]` | die 14 Aufgaben (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` markiert Sätze, deren `[S#]` der Text nicht stützt |
| `chat [--group G] [--since DATE] [--until DATE] [--lang xx] [--fallback]` | Folgefragen zu deiner Bibliothek; Antworten mit Quellen oder „in deinen Dokumenten nicht gefunden“ |
| `tag add/rm/list/show`, `star [--off]`, `note` | deine eigene Ordnung, bleibt beim Neuindizieren erhalten |
| `saved add/list/run/rm` | gespeicherte Suchen / intelligente Sammlungen |
| `recent`, `related PATH` | was du zuletzt geöffnet hast; ähnliche Dokumente |
| `export [--saved NAME] --format md\|csv\|json\|zip` | teile eine Gruppe, ein Tag oder eine gespeicherte Suche mit Quellenangaben |
| `backup OUT [--index]`, `backup-check ZIP`, `restore ZIP` | Backup in einer Datei mit Prüfsummen; prüfen; wiederherstellen |
| `verify [--repair]` | prüft den Index; mit `--repair` wird er aus deinen Dateien neu aufgebaut |
| `stats`, `timeline` | Zahlen zu deiner Bibliothek; Dokumente pro Monat |
| `import PATH` | Lesezeichen `.html`, Obsidian-Ordner, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex` |
| `feeds add/list/rm/poll`, `feeds opml-import/opml-export` | RSS/Atom-Abos, als Notizen gespeichert |
| `webwatch`, `history` | beobachtete Webseiten mit Diff; optionaler Browserverlauf |
| `providers [--json]` | Listet die Chat-Dienste, mit denen du dich verbinden kannst (Ollama, LM Studio, Mistral, OpenAI, Gemini, Cohere, NVIDIA, Claude, Meta), und welche bereit sind. Nutze einen mit `--provider NAME[:MODELL]` bei `chat`, `task`, `summarize` …; Cloud-Dienste brauchen zusätzlich `--allow-cloud` und einen Schlüssel, weil deine Textstellen dorthin gesendet werden. |
| `versus "QUESTION" [--providers mock,colibri,ollama:llama3.2,mistral:MODEL…] [--group G] [--lang xx] [--allow-cloud] [--json]` | Stellt mehreren Modellen dieselbe Frage zu denselben Textstellen und vergleicht sie: Zeit, Länge, Zitate und wie viele unbelegt wirken, dann jede Antwort. |
| `digest NAME [--every MIN] [--out FILE.md] [--summarize] [--all] [--mail-to ADDR [--sendmail]]` | Führt eine gespeicherte Suche erneut aus und meldet nur neue Dokumente seit dem letzten Mal; wiederholt sich, hängt an eine Datei an, fasst mit deiner lokalen KI zusammen oder speichert eine E-Mail-Datei im Ordner outbox. |
| `entities [--kind email\|url\|money\|date\|phone] [--group G] [--json]` | Alle E-Mails, Links, Beträge, Daten und Telefonnummern deiner Dokumente, mit den Dokumenten, in denen sie vorkommen. |
| `compare A B [--json]` | Vergleicht zwei Dokumente Satz für Satz: wie ähnlich sie sind und was nur in einem steht (A und B sind Pfade oder deren Ende). |
| `conflicts [--group G] [--json]` | Satzpaare aus verschiedenen Dokumenten, die bis auf eine Zahl oder ein Datum gleich sind (500 gegen 550): Kandidaten zum Prüfen, keine Urteile. |
| `dupes [--group G] [--min 0.6] [--json]` | Findet Dokumente, die Kopien oder fast Kopien voneinander sind (Exporte, Entwürfe, dieselbe Datei zweimal gespeichert), mit ihrer Ähnlichkeit und wie viel des kleineren im anderen steckt. |
| `chronology [--group G] [--limit N] [--json]` | Alle datierten Sätze der Bibliothek in zeitlicher Reihenfolge, mit ihrem Dokument (Daten wie 2025-03-12, 12.03.2025 oder 12/03/2025, Tag zuerst; oder mit Monatsnamen in einer der 8 Sprachen: 12. März 2025, 12 March 2025). Die monatliche Dokumentzahl ist `timeline`. |
| `embed [--embed-model M] [--batch 32] [--limit N] [--reset] [--mock] [--json]` | Bereitet die Bedeutungssuche vor: Das Embedding-Modell deiner lokalen Engine macht aus jeder Passage einen Vektor, der im Index der Bibliothek liegt. Inkrementell (nur neue oder geänderte Passagen), fortsetzbar (`--limit`), und nichts verlässt deinen Rechner. Danach findet `search --mode semantic` oder `--mode hybrid` (Wörter und Bedeutung zusammen) Texte, die andere Wörter benutzen als deine Frage. |
| `eval FILE [--modes bm25,semantic,hybrid] [--json]` | Misst, wie gut die Suche findet, was du erwartest. FILE enthält pro Zeile eine Frage mit den erwarteten Dateien (`Frage<TAB>datei1,datei2`) oder ist JSON (`[{"q": "...", "expect": ["lease.txt"]}]`). Gibt hit@1, hit@5, hit@10 und MRR je Modus aus, dazu die nicht gefundenen Fragen. |
| `topics [--json]` | Gruppiert die Dokumente nach Thema (ihre charakteristischsten Wörter). |
| `dossier TERM [--json]` | Alles, was die Bibliothek zu einem Namen oder Begriff sagt: Passagen je Dokument, dazu gefundene Daten und Beträge. |
| `changes --since DATE [--json]` | Seit einem Datum geänderte Dokumente (YYYY, YYYY-MM oder YYYY-MM-DD), neueste zuerst. |
| `factcheck FILE\|- [--json]` | Prüft einen eigenen Text gegen die Bibliothek: Jeder Satz wird als belegt, teilweise, Zahlen weichen ab oder nicht gefunden markiert, mit dem besten Beleg. |
| `mcp [--url ENGINE] [--mock]` | MCP-Server über stdio, damit andere KI-Apps in deiner Bibliothek suchen, lesen, fragen, prüfen und Dossiers abrufen (nur lesend). |

## Wie alles zusammenspielt

Ebene 0 — deine Ordner (nur lesend) → RAG-fetch — Index, Suche, Gruppen → RAG-CARTA — KI-Aufgaben über die Colibrì-Engine.

## Häufige Fragen

**Was bedeutet „lokal“?**  
Alles läuft auf deinem eigenen Rechner: die Seite, der Suchindex und das KI-Modell. Es funktioniert auch ohne Netzwerk. Die einzige Ausnahme ist web-grab, das die Webseiten abruft, die du ihm nennst.

**Werden meine Daten irgendwohin gesendet?**  
Nein. Keine Cloud, kein Konto, keine Telemetrie, keine Nutzungsstatistik. Das Einzige, was deinen Rechner verlässt, sind die Webseiten, die du web-grab ausdrücklich abrufen lässt. Details unter „Datenschutz und Sicherheit“.

**Colibrì läuft nicht. Kann ich trotzdem loslegen?**  
Ja. Bibliothek und Suche brauchen nie ein Modell. Für Zusammenfassen, Extrahieren und Synthetisieren aktiviere **Offline-Modus (ohne Modell)** (`--mock` auf der Kommandozeile): Du bekommst eine schlichte Antwort aus deinen Dateien, mit Quellenangaben, aber ohne echte KI-Formulierung. Für richtige Antworten starte die Colibrì-Engine.

**Was ist Colibrì, und brauche ich es?**  
[Colibrì](https://github.com/JustVugg/colibri) ist eine eigene lokale Engine, die das Sprachmodell ausführt. Du brauchst sie nur für die KI-Antworten. Wähle ein Modell, dessen Datei kleiner ist als dein freier RAM, und fang mit einem kleinen an. Siehe die Doku von Colibrì.

**Welche Dateien kann sie lesen?**  
Text und Notizen (txt, md, rst, log, csv, json, yaml, html, rtf), PDF, Word-, OpenDocument-, PowerPoint- und Excel-Dateien (docx, odt, pptx, xlsx), epub und, mit aktivierter OCR, Bilder und gescannte PDFs. Außerdem E-Mails (eml, mbox), Kalender und Kontakte (ics, vcf), zip-, tar- und tar.gz-Archive, OpenDocument-Tabellen und -Präsentationen (ods, odp) sowie die alten doc, xls und ppt, wenn antiword, catdoc oder LibreOffice installiert ist.

**Und Scans und Fotos von Dokumenten?**  
Schalte OCR ein (`--ocr` oder `CARTACOLIBRI_OCR=1`). Dafür muss `tesseract` auf deinem Rechner installiert sein, für gescannte PDFs außerdem `pdftoppm`. Die Seite sagt dir, wenn tesseract fehlt.

**Wie aktualisiere ich?**  
Portable Version: das neuere Zip herunterladen und entpacken. Aus dem Quellcode: `git pull`, dann `cargo build --release`. Deine Daten in `~/.cartacolibri` bleiben, wo sie sind, und der Index aktualisiert sich von selbst, nur für Geändertes.

**Wie entferne ich alles?**  
Lösche den Programmordner und dann den Zustandsordner `~/.cartacolibri` (oder wohin `CARTACOLIBRI_HOME` zeigt). Das ist alles: Deine eigenen Dokumente wurden nie angetastet.

**Gibt es etwas Besonderes unter Windows oder macOS?**  
Das portable Zip gibt es für beide, aber Linux ist das am besten getestete System. Der web-grab-Browser wird in CI für Windows und macOS kompiliert und wurde dort noch nie von Hand ausgeführt. Unter Windows braucht er die WebView2-Runtime. Wenn dein System dich bittet, das Öffnen eines heruntergeladenen Programms zu bestätigen, bestätige nur, wenn du es von der Projektseite hast.

**Warum „Alpha“?**  
Weil sie noch jung ist: Der Kern funktioniert und ist getestet, aber nicht auf jedem System, und die automatischen Prüfungen auf GitHub sind noch nicht bestätigt. Rechne mit Ecken und Kanten und sag uns Bescheid.

**Kann sie nach Bedeutung suchen, nicht nur nach Wörtern?**  
Ja, optional. Führe einmal `cartacolibri embed` aus: Das Embedding-Modell deiner lokalen Engine macht aus jeder Passage Zahlen, gespeichert in deiner eigenen Indexdatei (später werden nur neue oder geänderte Passagen neu berechnet). Danach findet `search --mode hybrid` (oder `semantic`) einen Text auch dann, wenn er andere Wörter benutzt als deine Frage. Dafür braucht die Engine ein Embedding-Modell (bei Ollama: `ollama pull nomic-embed-text`). Ohne es funktioniert die Wortsuche wie bisher. `cartacolibri eval` misst, wie gut jeder Modus findet, was du erwartest.

**Was kann sie nicht?**  
Es gibt keine Agenten, die selbstständig handeln, keine Websuche, keine Benutzerkonten, keine Handy-App und keinen Abgleich zwischen Computern. Die Wortsuche ist exakt, aber die Analysewerkzeuge (Widersprüche, Duplikate, Chronologie, Namen und Zahlen) arbeiten mit Wort- und Zahlenregeln, nicht mit Verständnis: Sie zeigen, was zu prüfen ist, und sind keine Urteile. Antworten sind so gut wie das Modell, das du lokal laufen lässt. Die 8 Übersetzungen wurden nicht von Muttersprachlern geprüft, und das Projekt ist Alpha. E-Mails als .msg und .pst werden nicht gelesen (exportiere sie als .eml oder .mbox), Audio und Video werden nicht transkribiert.

**Wie überleben Tags, Sterne und Notizen das Neuindizieren?**  
Sie stehen nicht im Suchindex, sondern in einer kleinen eigenen Datei, `organize.json`, in deinem `<home>`-Ordner, gebunden an den Pfad jedes Dokuments. **Index aktualisieren**, ein vollständiges Neuindizieren und sogar **Prüfen/Reparieren** (das den Index neu aufbaut) fassen sie nicht an. Benennst du eine Datei um oder verschiebst sie, zählt sie als neues Dokument; die Tags bleiben beim alten Pfad.

**Wie sichere ich und stelle wieder her?**  
Öffne **Statistik** und klicke auf **Backup herunterladen** (Haken setzen, um den Index mitzunehmen), oder führe `cartacolibri backup my.zip` aus. Das ist ein einziges Zip mit Prüfsummen, das deine Tags, Notizen, Gruppen, gespeicherten Suchen und importierten Notizen enthält, nie deine Originaldokumente. `cartacolibri backup-check my.zip` prüft es, `cartacolibri restore my.zip` stellt alles wieder her. Bewahre das Zip nicht nur auf diesem Rechner auf.

**Wie importiere ich Lesezeichen, Obsidian oder Zotero?**  
Klicke in der Bibliothek auf **Importieren** oder führe `cartacolibri import PFAD` aus. Verstanden werden ein Lesezeichen-Export des Browsers (`.html`), ein Obsidian-Ordner, eine Zotero/BibTeX-Datei `.bib`, `.csv`-Dateien von Pocket oder Raindrop und Evernote-Exporte `.enex`. Jeder Eintrag wird zu einer kleinen Notiz in deiner Bibliothek, die du durchsuchen, taggen und zitieren kannst. Deine Originale werden nur gelesen.

**Wie funktionieren Feeds?**  
Füge unter **Importieren → Feeds** eine RSS- oder Atom-Adresse ein und klicke auf **Hinzufügen**. Wenn du auf **Jetzt prüfen** klickst (oder `cartacolibri feeds poll` ausführst), werden neue Einträge als Notizen gespeichert und indiziert, sodass du sie durchsuchen und befragen kannst. Nichts läuft von selbst im Hintergrund. Deine Abos kannst du als OPML hinein- und herausholen.

**Was macht „Zitate prüfen“?**  
Nachdem die KI geantwortet hat, liest CartaColibri jeden Satz noch einmal. Hat ein Satz kein `[S#]`-Zitat oder enthält die zitierte Quelle gar nicht, was der Satz sagt, wird er mit `[unsupported]` markiert. Das ist ein schnelles Sicherheitsnetz, keine Garantie: Öffne bei allem Wichtigen die Quelle.

**Warum wurde eine Webseite abgelehnt?**  
web-grab ruft nur öffentliche Seiten ab. Adressen auf deinem eigenen Rechner oder in einem privaten Netz (`localhost`, `192.168.x.x`, `10.x.x.x` und Ähnliches) werden abgelehnt, damit keine Webseite es dazu bringen kann, deine internen Dienste zu lesen. Wenn du das wirklich willst, etwa für dein eigenes Intranet, starte CartaColibri mit `CARTACOLIBRI_GRAB_LOCAL=1`.

## Wenn etwas nicht klappt

Probiere zuerst `cartacolibri doctor`: Es prüft deine Einrichtung und sagt dir, was zu beheben ist.

| Symptom | Wahrscheinliche Ursache | Was tun |
|---|---|---|
| `cartacolibri: command not found` | Der Programmordner liegt nicht in deinem PATH. | Starte es mit vollem Pfad, zum Beispiel `./target/release/cartacolibri`, oder nutze den Starter aus dem portablen Zip. |
| Die Seite öffnet sich nicht unter `http://127.0.0.1:8765/` | Die lokale Seite läuft nicht, oder der Port ist belegt. | Führe `cartacolibri start` aus. Ist der Port belegt, führe `cartacolibri serve --port 8766` aus und öffne diese Adresse. |
| Du weißt nicht, was nicht stimmt | In der Einrichtung fehlt etwas. | Führe `cartacolibri doctor` aus: Es prüft die Einrichtung und sagt dir, was zu beheben ist. |
| Die KI-Aufgaben erreichen Colibrì nicht | Die Colibrì-Engine läuft nicht oder lauscht auf einer anderen Adresse (Standard `http://127.0.0.1:8000`). | Starte sie (`coli web --model ~/colibri-models/<model>`) oder gib ihre Adresse mit `--url` an. Aktiviere solange **Offline-Modus (ohne Modell)**. |
| Neue Dateien tauchen in der Suche nicht auf | Der Index hat sie noch nicht gesehen. | Klicke auf **Index aktualisieren** oder führe `cartacolibri index` aus. `cartacolibri watch` hält ihn automatisch aktuell. |
| Einige Dateien stehen unter „Übersprungene Dateien“ | Die Dateien sind beschädigt, geschützt, nicht unterstützt oder Scans. | Führe `cartacolibri problems` aus, um den Grund zu sehen. Bei Scans schalte OCR ein. |
| Scans und Bilder liefern keinen Text | OCR ist aus oder `tesseract` ist nicht installiert. | Installiere `tesseract` (und `pdftoppm` für PDFs) und füge `--ocr` hinzu, oder setze `CARTACOLIBRI_OCR=1`. |
| Die Antwort lautet NOT FOUND | Die gewählten Dateien enthalten die Information nicht. Das ist Absicht: Es wird nie geraten. | Prüfe, ob die richtige Gruppe gewählt ist, formuliere die Frage um oder füge der Gruppe die fehlenden Dateien hinzu. |
| Der web-grab-Browser öffnet sich nicht | Eine Systemkomponente fehlt. | Linux: installiere `libwebkit2gtk-4.1`. Windows: installiere die WebView2-Runtime. Bis dahin funktioniert `cartacolibri grab URL` auch ohne. |
| Dein eigenes Skript bekommt `403 missing X-Colibri-UI header` | Das ist eine Sicherheitsprüfung, kein Fehler. | Sende den Header `X-Colibri-UI: 1` bei jedem Aufruf von `/carta/*`. |
| Der Index wirkt kaputt: seltsame Treffer, fehlende Dateien, ein Fehler zur Datenbank | Die Indexdatei wurde beschädigt (etwa durch einen Absturz oder eine volle Festplatte) oder ist nicht mehr im Takt mit deinen Ordnern. | Öffne **Statistik → Prüfen / Reparieren** oder führe `cartacolibri verify --repair` aus. Der Index wird aus deinen Dateien neu aufgebaut; Tags, Notizen und Gruppen bleiben erhalten. |
| Eine Datei wurde übersprungen, weil sie „zu langsam“ war | Das Lesen dauerte länger als das Zeitlimit (standardmäßig 120 Sekunden), meist bei einem riesigen oder beschädigten PDF oder einem sehr großen Scan. | Erhöhe das Limit, zum Beispiel `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, und klicke dann auf **Index aktualisieren**. Oder teile / exportiere die Datei neu. Der Grund steht unter „Übersprungene Dateien“. |
| „AI engine not running … (404)“ | Die Adresse zeigt auf etwas, das nicht die Engine ist, oder es ist kein Modell geladen. | Öffne ⚙, drücke „Engine suchen“ und wähle die angezeigte Adresse; `…:8000` und `…:8000/v1` funktionieren beide. Prüfe, ob in Colibri Companion ein Modell geladen ist. |

## Glossar in einfachen Worten

- **Index** — Ein Verzeichnis der Wörter in deinen Dateien, damit eine Suche einen Augenblick dauert, statt alles neu zu lesen.
- **RAG** — „Retrieval-augmented generation“: Zuerst werden die passenden Stellen in deinen Dateien gefunden, dann antwortet die KI nur anhand dieser Stellen.
- **Quellenangabe [S1]** — Eine Markierung wie [S1] oder [S2] in einer Antwort. Das Kürzel verweist auf die Quelldatei, aus der der Satz stammt, damit du es prüfen kannst.
- **Gruppe** — Eine benannte Liste von Dateien (ein Projekt, ein Thema), in der du immer wieder suchen oder fragen kannst.
- **Gateway** — Die kleine Webadresse (Standard `http://127.0.0.1:8000`), über die CartaColibri mit der Colibrì-Engine spricht.
- **Schnappschuss (Snapshot)** — Eine gespeicherte HTML-Kopie einer Webseite, bei jedem Speichern mit web-grab angelegt, damit du behältst, was an dem Tag dort stand.
- **Watch (Beobachten)** — Zweierlei: `cartacolibri watch` hält deinen Index aktuell, und `cartacolibri webwatch` ruft Seiten, die du verfolgst, erneut ab und zeigt, was sich geändert hat.

## Datenschutz und Sicherheit

- **Wo was liegt.** Alles, was CartaColibri schreibt, liegt in einem einzigen Ordner, `<home>`: standardmäßig `~/.cartacolibri` oder der Ordner aus `CARTACOLIBRI_HOME`. Darin: `index.sqlite` (der Suchindex), `groups.json`, `library/uploads/` (hochgeladene Dateien), `outputs/` (Ergebnisse, die du speichern wolltest), `snapshots/` und `web.json` (gespeicherte Webseiten), `highlights.json`, `watch.json`, `settings.json` und `clip.token` (ein Geheimnis für das Bookmarklet: nicht weitergeben).
- **Deine Originale sind nur lesbar.** CartaColibri liest deine Dokumente und verändert, verschiebt oder löscht sie nie.
- **Die Seite ist rein lokal.** Der Server lauscht nur auf `127.0.0.1`, andere Rechner in deinem Netzwerk kommen also nicht heran. Anfragen, deren Adresse oder Herkunft nicht lokal ist, weist er ebenfalls ab.
- **Der Header `X-Colibri-UI`.** Jeder Aufruf der Seite an ihren eigenen Server trägt diesen Header. Eine Webseite in einem anderen Tab kann ihn nicht hinzufügen und deshalb deinen Browser nicht dazu bringen, die lokale App hinter deinem Rücken anzustoßen.
- **Nichts verlässt `127.0.0.1`, außer Seiten, die du abrufen lässt.** Keine Telemetrie, keine Statistik, keine Cloud. web-grab ruft nur die Adressen ab, die du angibst (und Seiten, die du beobachten wolltest). Auch die Präsentationsseite lädt nichts von anderen Servern.
- **Der Verlauf ist standardmäßig aus.** Der Verlauf des web-grab-Browsers existiert nur, wenn du ihn einschaltest (`cartacolibri history on`), bleibt auf deiner Festplatte, und `cartacolibri history clear` löscht ihn.

## Ehrlicher Stand

CartaColibri ist **Alpha**-Software (Version 0.2.0-alpha): Es funktioniert, aber rechne mit Ecken und Kanten und sichere Wichtiges extra. Linux ist die am besten getestete Plattform: Dort wird der web-grab-Browser in CI selbst getestet. Unter **Windows und macOS** wird der Browser in CI zwar kompiliert, wurde aber **noch nie von Hand ausgeführt**. Die automatischen Prüfungen auf GitHub (CI) laufen unter Linux, Windows und macOS durch. Deine Dokumente sind in jedem Fall sicher, weil sie nur gelesen werden. Die portablen Downloads für Linux, Windows und macOS findest du auf der GitHub-Seite **Releases** (als Pre-Release markiert).

## Sicherheit

Du hast eine Sicherheitslücke gefunden? Melde sie bitte privat, wie in [SECURITY.md](../../SECURITY.md) beschrieben; eröffne bitte nicht zuerst ein öffentliches Issue. Wie sich der Server schützt (nur Loopback, Header `X-Colibri-UI`, keine privaten Adressen in web-grab, Originale nur lesend), steht im [docs/threat-model.md](../threat-model.md).

## Lizenz und Danksagung

CartaColibri steht unter der **MIT-Lizenz**. Es ist eine Clean-Room-Implementierung in Rust: Sie spricht mit der [Colibrì](https://github.com/JustVugg/colibri)-Engine (JustVugg, Apache-2.0) nur über deren öffentliches HTTP-Gateway und enthält keinen Code daraus. Die Index- und Suchschicht folgt dem im öffentlichen DocFetcher-Handbuch beschriebenen Verhalten; DocFetcher-Code wird nicht verwendet. Der web-grab-Browser nutzt die Webview des Betriebssystems und enthält keinen Falkon-Code.

**Alexandru-Ionuț Chiuță** · **Alexandru Dumbrava** (Mitautor)

## Mehr

[Vollständige Dokumentation](../cartacolibri.md) · [README der Colibrì-Engine](https://github.com/JustVugg/colibri) · [Mitmachen](../../CONTRIBUTING.md) · [📄 Bebilderte PDF-Anleitung](CartaColibri-Tutorial.de.pdf)
