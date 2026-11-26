<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Documentele tale, căutabile și explicate — 100 % local.</b></p>

<p align="center">[English](README.en.md) · <b>Română</b> · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri este o bibliotecă AI locală construită pe motorul Colibrì. Indexează folderele alese de tine, le poți căuta într-o clipă, apoi ceri unui model de limbaj local să rezume, să extragă date sau să sintetizeze grupuri de fișiere — mereu cu citarea fișierului-sursă. Nimic nu pleacă de pe calculatorul tău, iar originalele nu sunt niciodată modificate.

## Primele tale 5 minute

Nu ai nevoie de experiență. Trei pași și îți vei fi căutat și rezumat propriile fișiere.

1. **Pornește-l.** Dezarhivează zip-ul portabil și rulează lansatorul, sau scrie `cartacolibri start` într-un terminal. Browserul îți arată pagina locală, `http://127.0.0.1:8765/`.
2. **Adaugă un folder.** În pagină deschide **Bibliotecă**, scrie calea unui folder, de exemplu `~/Documents`, și apasă **Adaugă folder**. Prima indexare a unui folder mare poate dura câteva minute. Fișierele tale sunt doar citite, niciodată modificate.
3. **Caută, apoi întreabă.** Deschide **RAG-fetch**, caută un cuvânt, bifează câteva fișiere și creează un grup. Apoi deschide **RAG-CARTA**, alege **Rezumă** și apasă **Rulează**. Nu ai încă un model AI? Bifează **Mod offline (fără model)** și încearcă oricum.

Preferi terminalul? Aceiași trei pași:

```bash
cartacolibri start                 # step 1 - leave this window open (use a second one for the rest)
cartacolibri add ~/Documents       # step 2
cartacolibri group set Demo ~/Documents
cartacolibri summarize "main points" --group Demo --mock   # step 3 - --mock = no AI model needed
```

## Ce face

- **Biblioteca** — toate documentele locale cu locația lor; adaugi sau scoți foldere, încarci fișiere, le vezi, opțional salvezi text OCR.
- **RAG-fetch** — index full-text incremental în stil DocFetcher (cuvinte, „expresii”, prefix*, AND/OR/NOT, filtre după tip, mărime, locație) și **grupuri** reutilizabile de fișiere.
- **RAG-CARTA** — trei taskuri locale pe un grup: **rezumă**, **extrage** (tabel sau JSON, „NOT FOUND” în loc de presupuneri), **sintetizează** (teme, puncte comune, contradicții). Fiecare răspuns citează [S1], [S2]… și își listează sursele.
- **Privat prin construcție** — fără cloud, fără telemetrie; originalele sunt doar citite. 8 limbi: EN, RO, FR, IT, ES, PT, DE, HU.
- **web-grab** — un mic browser în Rust (clean-room), cu taburi, care îți cunoaște biblioteca: insignă când pagina e deja salvată (și când s-a schimbat), documente înrudite în timp ce navighezi, salvare dintr-un clic, întrebări către Colibrì local despre o pagină — sau peste toate taburile deschise, cu citări. O comandă și un bookmarklet acoperă alte browsere. În plus: 🖍 evidențieri salvate cu pagina, 👁 pagini urmărite cu diferența a ceea ce s-a schimbat (`cartacolibri webwatch`), instantaneu HTML la fiecare salvare, istoric căutabil opțional (`cartacolibri history on`), 🗂📝📊 rezumat sau extragere peste taburi și `?întrebare` în bara de adrese pentru a întreba toată biblioteca.

## Funcții avansate

- **Găsește** — bară de progres în timpul indexării; „Ai vrut să spui…?” pentru greșeli de scriere, cu sugestii când nu se potrivește nimic; duplicatele listate și un motiv clar pentru fiecare fișier sărit. Scrie filtre direct în căsuță (`ext:pdf`, `path:`, `size:>1mb`, date `after:`/`before:`, expresii regulate `re:`), bifează **Căutare inteligentă** ca să adaugi cuvinte înrudite și vezi câte potriviri are fiecare tip de fișier.
- **Organizează** — etichete, o stea și o notă privată pe fiecare document; rămân și după reindexare. Vezi documente înrudite, ce ai deschis recent și cel mai des, și salvează căutări ca colecții inteligente.
- **14 taskuri** — compară, cronologie, glosar, schiță, acțiuni, FAQ, fișe de studiu, test, tradu, explică, contradicții, plus rezumă, extrage și sintetizează. Bifează **verifică citările** ca să fie marcate frazele pe care sursele nu le susțin și alege limba răspunsului.
- **Chat** — pune întrebări succesive peste toată biblioteca sau peste un grup. Răspunsurile își citează sursele sau spun sincer „nu am găsit în documentele tale”, în loc să ghicească. Fiecare răspuns arată un scor de **Susținere în surse** și marchează propozițiile sau numerele pe care sursele nu le susțin. Bifează „Fiecare întrebare separat” ca să nu rețină nimic sau limitează documentele la un interval de timp.
- **Partajează și protejează** — exportă un grup, o etichetă sau o căutare ca Markdown, CSV, JSON sau zip, cu citări; o vedere Statistici cu grafice și cronologie; o copie de rezervă într-un singur fișier, cu sume de control; Verifică/Repară pentru index.
- **Adu lucruri înăuntru** — importă marcaje de browser, un folder Obsidian, Zotero/BibTeX, CSV Pocket/Raindrop sau exporturi Evernote și urmărește fluxuri RSS/Atom (OPML import și export): devin notițe căutabile. Sunt indexate și e-mailurile, calendarele, subtitrările și caietele.
- **Perspicacitate** — “Contradicții” (aceeași propoziție, alt număr sau altă dată), “Harta subiectelor” (bule), “Ce e nou” de la o dată, “Compară 2” documente, “Nume și numere” (emailuri, linkuri, sume, date, telefoane), “Dosar” despre un nume și “Verifică textul meu” față de bibliotecă.
- **Automatizezi și conectezi** — `digest` reia o căutare salvată și raportează doar ce e nou (repetă, adaugă într-un fișier, rezumă sau salvează un fișier email); `mcp` lasă alte aplicații AI să caute, citească, întrebe, verifice fapte și ceară dosare din biblioteca ta, doar în citire.
- **Copii și date** — “Duplicate” (documente copii sau aproape-copii, cu cât de asemănătoare sunt) și “Cronologie” (toate propozițiile cu dată, în ordine).

## Noutăți în 0.2.0-alpha.7

- **Filtre direct în căsuța de căutare** — Scrie `ext:pdf`, `path:contracte`, `size:>1mb`, `after:2025-01`, `before:2025-06-30` sau `re:FACT-\d+` (expresie regulată) lângă cuvintele tale și rezultatele se restrâng. Merg în pagina web, în linia de comandă și prin API.
- **Căutare inteligentă** — Bifează „Căutare inteligentă” și AI-ul tău local adaugă câteva sinonime și cuvinte înrudite (ca un simplu OR, deci potrivirile exacte rămân primele). Cuvintele în plus apar sub căsuță. Fără motor pornit primești pur și simplu căutarea normală.
- **Chat cu întrebări separate** — Bifează „Fiecare întrebare separat” și chatul uită întrebările anterioare: fiecare răspuns vine doar din documentele tale, cu sursele lui.
- **Server MCP pentru alte aplicații AI** — `cartacolibri mcp` lasă alte aplicații AI să-ți caute în bibliotecă, să citească un document și să pună întrebări despre el prin Model Context Protocol. E doar în citire, rulează pe calculatorul tău și ajunge doar la documentele returnate de căutare.
- **Rezumat cu ce e nou** — `cartacolibri digest NUME` reia o căutare salvată și raportează doar documentele apărute de la ultima rulare. Adaugă `--every 60` ca să se repete din oră în oră, `--out raport.md` ca să adauge într-un fișier și `--summarize` pentru un scurt rezumat de la AI-ul tău local.
- **Afli cât de mult poți avea încredere într-un răspuns** — Fiecare răspuns din chat arată acum un scor „Susținere în surse”: cât din fiecare propoziție se găsește cu adevărat în sursele citate, cu propozițiile de verificat și numerele care nu apar în nicio sursă.
- **Compară două documente și vezi toate numele și numerele** — Bifează două documente și apasă „Compară 2” ca să vezi, propoziție cu propoziție, ce diferă între două versiuni. „Nume și numere” listează toate emailurile, linkurile, sumele, datele și telefoanele din bibliotecă și unde apar. În linia de comandă: `compare` și `entities`.
- **Găsești contradicții, vezi harta subiectelor, urmărești ce s-a schimbat** — „Contradicții” listează propoziții din documente diferite care sunt la fel, mai puțin un număr sau o dată (500 față de 550). „Harta subiectelor” grupează documentele după ce tratează. „Ce e nou” arată ce s-a schimbat de la o dată. În linia de comandă: `conflicts`, `topics`, `changes --since`.
- **Verifici textul tău față de bibliotecă** — Lipește o ciornă în „Verifică textul meu” și fiecare propoziție e marcată: susținută, parțial, numerele diferă sau negăsită, cu cea mai bună dovadă din documentele tale. „Dosar” adună tot ce spune biblioteca despre un nume sau termen, cu datele și sumele lui. Alte aplicații AI le primesc pe ambele prin serverul MCP.
- **Întrebi pe un interval de timp și primești rezumatele ca fișier email** — În chat alegi „Documente din … până în …” și răspunsul vine doar din documentele modificate în perioada aceea (în linia de comandă: `chat --since --until`). Harta subiectelor e acum o imagine cu bule pe care poți apăsa. `digest --mail-to tu@example.org` salvează fiecare raport și ca fișier email în folderul outbox; adaugă `--sendmail` ca să-l dai programului de poștă al calculatorului tău. Nimic altceva nu iese din calculator.
- **Conectează serviciile de chat pe care le știi și compară-le** — `cartacolibri providers` listează Ollama, LM Studio, Mistral, OpenAI, Gemini, Cohere, NVIDIA, Claude și Meta; `--provider NUME:MODEL` folosește unul la chat și sarcini, iar `versus` pune aceeași întrebare mai multora pe aceleași pasaje. Serviciile din cloud cer `--allow-cloud` și o cheie, fiindcă pasajele tale le sunt trimise.
- **Găsești copiile și citești biblioteca ca o cronologie** — „Duplicate” listează documentele care sunt copii sau aproape-copii, cu cât de asemănătoare sunt. „Cronologie” pune toate propozițiile cu dată din bibliotecă în ordine, cu documentul din care vin. În linia de comandă: `dupes`, `chronology`.
- **Căutare după sens, măsurarea căutării, mai multe formate** — `embed` pregătește căutarea după sens cu modelul de embeddings al motorului tău local; apoi `search --mode hybrid` găsește texte care folosesc alte cuvinte decât întrebarea ta, iar `eval` măsoară cât de bine găsește fiecare mod ce te aștepți. Formate noi: tar și tar.gz, foi și prezentări OpenDocument, fișiere Office cu macrouri și vechile doc, xls și ppt prin antiword, catdoc sau LibreOffice. Uneltele pe toată biblioteca (nume și numere, copii, conflicte, subiecte) au trecut de la minute la sub o secundă la 20.000 de documente, iar filtrele văd acum toată biblioteca.

## Noutăți în 0.2.0-alpha.2

- **Adresa motorului merge direct** — `http://127.0.0.1:8000` și `http://127.0.0.1:8000/v1` sunt ambele bune. CartaColibri citește lista de modele din `/v1/models`, alege singur modelul încărcat și nu mai are nevoie de o pagină `/health`. Asta rezolvă mesajul „AI engine not running (404)”.
- **Își găsește singur AI-ul** — CartaColibri caută un motor pornit pe acest calculator: setările proprii ale Colibri Companion, un fișier de descoperire, apoi porturile locale obișnuite (8000, 8080, 11434, 1234, 8001, 5000). Se verifică doar adrese loopback. Deschide ⚙ ca să vezi ce a găsit sau apasă „Pornește Colibri Companion”.
- **Stare și viteză în direct** — Antetul arată modelul care rulează și dacă a fost găsit automat, reîmprospătat la câteva secunde. Sub fiecare răspuns vezi tokenii folosiți și viteza; sub 3 tokeni/s primești un sfat să alegi un model mai mic. Cererile merg la motor pe rând.
- **Pagini web din dialogul de import** — În „Adu lucruri din altă parte” fluxurile de știri au lăsat locul web-grab: lipești o adresă și salvezi pagina ca un singur fișier HTML sau MHTML în bibliotecă. Fluxurile sunt tot acolo, pliate.
- **Trei butoane în browserul web-grab** — „Save to folder” păstrează copia text în bibliotecă, „Grab website HTML” salvează pagina (Shift+clic pentru MHTML), „PDF” o tipărește în PDF. Pe Linux PDF-ul se face în tăcere; pe Windows și macOS se deschide dialogul de tipărire și alegi „Salvează ca PDF”.
- **Paginile salvate sunt căutabile** — Fișierele `.mhtml` și `.html` sunt indexate cu setul de caractere corect, deci o pagină salvată se găsește ca orice document. Părțile unei pagini aflate pe adrese private nu sunt niciodată descărcate sau legate.

## Pornire rapidă

Compilează-l (sau ia zip-ul portabil) și pornește pagina locală. Pentru taskurile AI pornește și motorul Colibrì cu un model care încape în RAM-ul tău (vezi documentația lui):

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
cargo build --release                       # Rust 1.90+, or use the portable zip
./target/release/cartacolibri serve         # then open http://127.0.0.1:8765/
coli web --model ~/colibri-models/<model>   # Colibrì engine, only for the AI tasks
```

Sau folosește linia de comandă, fără model pentru a încerca (`--mock` e un mod offline de rezervă):

```bash
cartacolibri add ~/Documents
cartacolibri search "contract*" --ext pdf,docx
cartacolibri group set Contracts ~/Documents/contracts
cartacolibri synthesize "payment terms" --group Contracts --save --mock
cartacolibri grab https://example.org/article      # web-grab (command line)
cartacolibri-browser https://example.org           # web-grab (browser)
```

## Mai multe comenzi

| Comandă | Ce face |
|---|---|
| `start [--folder F] [--port N] [--no-open] [--no-serve]` | Pornire ghidată: alegi un folder, îl indexează și deschide pagina web. |
| `doctor` | Verifică motorul, OCR-ul, uneltele PDF, discul și portul și îți spune exact cum repari fiecare problemă. |
| `add FOLDER`, `remove FOLDER`, `folders`, `index [FOLDER]`, `problems` | Adaugi sau uiți un folder (fișierele nu sunt atinse niciodată), listezi folderele, reindexezi și vezi fișierele care n-au putut fi citite. |
| `watch [--interval S]` | Ține indexul la zi: rescanează folderele la fiecare S secunde. |
| `search QUERY [--ext pdf,txt] [--min-kb N] [--max-kb N] [--path P] [--group G] [--mode bm25\|semantic\|hybrid] [--json]` | Caută în bibliotecă. Cuvinte, „fraze”, prefix*, AND/OR/NOT și filtre direct în interogare: `ext:pdf` `path:contracte` `size:>1mb` `after:2025-01` `before:2025-06-30` `re:FACT-\d+`. |
| `list [--group G] [--json]`, `group set/list/show/rm/from-search` | Listează documentele; grupurile sunt seturi de documente cu nume, în care poți căuta sau despre care poți întreba. |
| `summarize`, `extract`, `synthesize` (same options as `task`, with `--fields`) | Scurtături pentru cele trei sarcini principale: rezumi fiecare document, extragi câmpuri sau combini mai multe într-un singur răspuns. |
| `grab URL`, `bookmarklet` | Salvează o pagină web în bibliotecă (HTML, MHTML sau PDF); bookmarklet-ul face asta din orice browser. |
| `serve [--port N] [--url ENGINE]` | Pornește pagina web pe acest calculator (http://127.0.0.1:PORT). |
| `completions bash\|zsh\|fish\|powershell`, `import-json FILE` | Tipărește un script de completare cu Tab pentru shell-ul tău; reimportă etichete, stele, notițe și grupuri exportate ca JSON. |
| `task NAME [request] --group G \| --tag T \| --query Q [--mock] [--verify] [--lang xx] [--save md\|html\|txt]` | cele 14 taskuri (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` marchează frazele al căror `[S#]` nu e susținut de text |
| `chat [--group G] [--since DATE] [--until DATE] [--lang xx] [--fallback]` | întrebări succesive peste bibliotecă; răspunsurile sunt citate sau spun „nu am găsit în documentele tale” |
| `tag add/rm/list/show`, `star [--off]`, `note` | organizarea ta, păstrată și după reindexare |
| `saved add/list/run/rm` | căutări salvate / colecții inteligente |
| `recent`, `related PATH` | ce ai deschis recent; documente similare |
| `export [--saved NAME] --format md\|csv\|json\|zip` | partajează un grup, o etichetă sau o căutare salvată, cu citări |
| `backup OUT [--index]`, `backup-check ZIP`, `restore ZIP` | copie de rezervă într-un singur fișier, cu sume de control; verific-o; restaureaz-o |
| `verify [--repair]` | verifică indexul; cu `--repair`, îl reconstruiește din fișierele tale |
| `stats`, `timeline` | cifre despre bibliotecă; documente pe luni |
| `import PATH` | marcaje `.html`, folder Obsidian, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex` |
| `feeds add/list/rm/poll`, `feeds opml-import/opml-export` | abonamente RSS/Atom salvate ca notițe |
| `webwatch`, `history` | pagini web urmărite, cu diferențe; istoric de navigare opțional |
| `providers [--json]` | Listează serviciile de chat la care te poți conecta (Ollama, LM Studio, Mistral, OpenAI, Gemini, Cohere, NVIDIA, Claude, Meta) și care sunt gata. Folosește unul cu `--provider NUME[:MODEL]` la `chat`, `task`, `summarize`…; cele din cloud cer și `--allow-cloud` și o cheie, fiindcă pasajele tale le sunt trimise. |
| `versus "QUESTION" [--providers mock,colibri,ollama:llama3.2,mistral:MODEL…] [--group G] [--lang xx] [--allow-cloud] [--json]` | Pune aceeași întrebare mai multor modele pe aceleași pasaje și le compară: timp, lungime, citări și câte citări par nesusținute, apoi fiecare răspuns. |
| `digest NAME [--every MIN] [--out FILE.md] [--summarize] [--all] [--mail-to ADDR [--sendmail]]` | Reia o căutare salvată și raportează doar documentele noi de la ultima rulare; poate repeta, adăuga într-un fișier, rezuma cu AI-ul tău local sau salva un fișier email în folderul outbox. |
| `entities [--kind email\|url\|money\|date\|phone] [--group G] [--json]` | Toate emailurile, linkurile, sumele, datele și telefoanele din documente, cu documentele în care apar. |
| `compare A B [--json]` | Compară două documente propoziție cu propoziție: cât de asemănătoare sunt și ce e doar într-unul dintre ele (A și B sunt căi sau terminațiile lor). |
| `conflicts [--group G] [--json]` | Perechi de propoziții din documente diferite care sunt la fel, mai puțin un număr sau o dată (500 față de 550): candidate de verificat, nu verdicte. |
| `dupes [--group G] [--min 0.6] [--json]` | Găsește documente care sunt copii sau aproape-copii unele ale altora (exporturi, ciorne, același fișier salvat de două ori), cu cât de asemănătoare sunt și cât din cel mai mic se află în celălalt. |
| `chronology [--group G] [--limit N] [--json]` | Toate propozițiile cu dată din bibliotecă, în ordine cronologică, cu documentul din care vin (date ca 2025-03-12, 12.03.2025 sau 12/03/2025, ziua întâi; sau cu numele lunii în oricare din cele 8 limbi: 12 March 2025, 12 martie 2025). Numărul lunar de documente este `timeline`. |
| `embed [--embed-model M] [--batch 32] [--limit N] [--reset] [--mock] [--json]` | Pregătește căutarea după sens: modelul de embeddings al motorului tău local transformă fiecare pasaj într-un vector păstrat în indexul bibliotecii. Incremental (doar pasajele noi sau schimbate), reluabil (`--limit`), iar nimic nu iese de pe calculator. Apoi `search --mode semantic` sau `--mode hybrid` (cuvinte și sens împreună) găsește texte care folosesc alte cuvinte decât întrebarea ta. |
| `eval FILE [--modes bm25,semantic,hybrid] [--json]` | Măsoară cât de bine găsește căutarea ce te aștepți. FILE are o întrebare pe linie cu fișierele așteptate (`întrebare<TAB>fișier1,fișier2`) sau e JSON (`[{"q": "...", "expect": ["lease.txt"]}]`). Afișează hit@1, hit@5, hit@10 și MRR pentru fiecare mod și întrebările negăsite. |
| `topics [--json]` | Grupează documentele după ce tratează (cele mai caracteristice cuvinte ale lor). |
| `dossier TERM [--json]` | Tot ce spune biblioteca despre un nume sau termen: pasaje pe documente, plus datele și sumele găsite în ele. |
| `changes --since DATE [--json]` | Documentele modificate de la o dată (YYYY, YYYY-MM sau YYYY-MM-DD), cele mai noi întâi. |
| `factcheck FILE\|- [--json]` | Verifică un text al tău față de bibliotecă: fiecare propoziție e marcată susținută, parțial, numerele diferă sau negăsită, cu cea mai bună dovadă. |
| `mcp [--url ENGINE] [--mock]` | Server MCP prin stdio, ca alte aplicații AI să caute, citească, întrebe, verifice fapte și ceară dosare din biblioteca ta (doar citire). |

## Cum se leagă

Stratul 0 — folderele tale (doar citire) → RAG-fetch — index, căutare, grupuri → RAG-CARTA — taskuri AI prin motorul Colibrì.

## Întrebări frecvente

**Ce înseamnă „local”?**  
Totul rulează pe propriul tău calculator: pagina, indexul de căutare și modelul AI. Funcționează și cu cablul de rețea scos. Singura excepție este web-grab, care aduce paginile web pe care i le ceri.

**Se trimit datele mele undeva?**  
Nu. Nu există cloud, cont, telemetrie sau analiză de utilizare. Singurul trafic care iese de pe calculator sunt paginile web pe care îi ceri explicit lui web-grab să le aducă. Detalii în „Confidențialitate și siguranță”.

**Colibrì nu rulează. Pot totuși să folosesc aplicația?**  
Da. Biblioteca și căutarea nu au nevoie de model. Pentru rezumat, extragere și sinteză bifează **Mod offline (fără model)** (`--mock` în linia de comandă): primești un răspuns simplu, construit din fișierele tale, cu citări, dar fără formulări AI reale. Pentru răspunsuri adevărate, pornește motorul Colibrì.

**Ce este Colibrì și am nevoie de el?**  
[Colibrì](https://github.com/JustVugg/colibri) este un motor local separat care rulează modelul de limbaj. Ai nevoie de el doar pentru răspunsurile AI. Alege un model al cărui fișier e mai mic decât RAM-ul tău liber și începe cu unul mic. Vezi documentația Colibrì.

**Ce fișiere poate citi?**  
Text și notițe (txt, md, rst, log, csv, json, yaml, html, rtf), PDF, fișiere Word, OpenDocument, PowerPoint și Excel (docx, odt, pptx, xlsx), epub și, cu OCR pornit, imagini și PDF-uri scanate. Plus e-mail (eml, mbox), calendare și contacte (ics, vcf), arhive zip, tar și tar.gz, foi și prezentări OpenDocument (ods, odp) și vechile doc, xls și ppt dacă ai instalat antiword, catdoc sau LibreOffice.

**Dar scanările și pozele cu documente?**  
Pornește OCR (`--ocr` sau `CARTACOLIBRI_OCR=1`). Are nevoie de `tesseract` instalat pe calculator, plus `pdftoppm` pentru PDF-uri scanate. Pagina îți spune când lipsește tesseract.

**Cum actualizez?**  
Versiunea portabilă: descarcă zip-ul mai nou și dezarhivează-l. Din sursă: `git pull`, apoi `cargo build --release`. Datele tale din `~/.cartacolibri` rămân unde sunt, iar indexul se reîmprospătează singur, doar pentru ce s-a schimbat.

**Cum șterg totul?**  
Șterge folderul programului, apoi folderul de stare `~/.cartacolibri` (sau unde indică `CARTACOLIBRI_HOME`). Atât: documentele tale nu au fost atinse niciodată.

**Ceva special pe Windows sau macOS?**  
Zip-ul portabil există pentru ambele, dar Linux e sistemul cel mai bine testat. Browserul web-grab este compilat în CI pentru Windows și macOS și nu a fost niciodată rulat manual acolo. Pe Windows are nevoie de WebView2 runtime. Dacă sistemul îți cere confirmarea pentru un program descărcat, confirmă doar dacă l-ai luat de pe pagina proiectului.

**De ce „alfa”?**  
Pentru că e tânăr: nucleul funcționează și e testat, dar nu pe orice sistem, iar verificările automate de pe GitHub nu sunt încă confirmate. Așteaptă-te la asperități și spune-ne despre ele.

**Poate căuta după sens, nu doar după cuvinte?**  
Da, opțional. Rulează o dată `cartacolibri embed`: modelul de embeddings al motorului tău local transformă fiecare pasaj în numere, păstrate în propriul tău fișier de index (apoi se refac doar pasajele noi sau schimbate). După aceea `search --mode hybrid` (sau `semantic`) găsește un text chiar dacă folosește alte cuvinte decât întrebarea ta. Are nevoie de un model de embeddings în motor (pentru Ollama: `ollama pull nomic-embed-text`). Fără el, căutarea după cuvinte merge exact ca înainte. `cartacolibri eval` măsoară cât de bine găsește fiecare mod ce te aștepți.

**Ce nu poate face?**  
Nu are agenți care acționează singuri, căutare pe web, conturi de utilizatori, aplicație de telefon sau sincronizare între calculatoare. Căutarea după cuvinte e exactă, dar uneltele de analiză (conflicte, duplicate, cronologie, nume și numere) folosesc reguli de cuvinte și numere, nu înțelegere: arată lucruri de verificat, nu sunt verdicte. Răspunsurile sunt doar la fel de bune ca modelul pe care îl rulezi local. Cele 8 traduceri nu au fost verificate de vorbitori nativi, iar proiectul e alfa. E-mailurile .msg și .pst nu se citesc (exportă-le în .eml sau .mbox), iar audio și video nu se transcriu.

**Cum rămân etichetele, stelele și notele după reindexare?**  
Nu sunt păstrate în indexul de căutare, ci într-un mic fișier separat, `organize.json`, în folderul tău `<home>`, legate de calea fiecărui document. Deci **Actualizează indexul**, o reindexare completă și chiar **Verifică/Repară** (care reface indexul) nu le ating. Dacă redenumești sau muți un fișier, el contează ca document nou, iar etichetele rămân la vechea cale.

**Cum fac o copie de rezervă și cum o restaurez?**  
Deschide **Statistici** și apasă **Descarcă copia de rezervă** (bifează căsuța ca să incluzi și indexul), sau rulează `cartacolibri backup my.zip`. Este un singur zip cu sume de control, cu etichetele, notele, grupurile, căutările salvate și notițele importate, niciodată cu documentele tale originale. `cartacolibri backup-check my.zip` îl verifică, iar `cartacolibri restore my.zip` pune totul la loc. Păstrează zip-ul în altă parte decât pe acest calculator.

**Cum import marcaje, Obsidian sau Zotero?**  
Apasă **Adu lucruri** în Bibliotecă sau rulează `cartacolibri import CALEA`. Înțelege un export de marcaje din browser (`.html`), un folder Obsidian, un fișier Zotero/BibTeX `.bib`, fișiere `.csv` Pocket sau Raindrop și exporturi Evernote `.enex`. Fiecare element devine o mică notiță în biblioteca ta, pe care o poți căuta, eticheta și cita. Originalele sunt doar citite.

**Cum funcționează fluxurile (feeds)?**  
În **Adu lucruri → Fluxuri**, lipește o adresă RSS sau Atom și apasă **Adaugă**. Când apeși **Verifică acum** (sau rulezi `cartacolibri feeds poll`), elementele noi sunt salvate ca notițe și indexate, deci le poți căuta și întreba despre ele. Nimic nu rulează singur în fundal. Abonamentele le poți aduce și scoate ca OPML.

**Ce face „verifică citările”?**  
După ce răspunde AI-ul, CartaColibri recitește fiecare frază. Dacă o frază nu are citare `[S#]`, sau sursa citată nu conține de fapt ce spune fraza, este marcată `[unsupported]`. E o plasă de siguranță rapidă, nu o garanție: deschide sursa pentru tot ce e important.

**De ce a fost refuzată o pagină web?**  
web-grab aduce doar pagini publice. Adresele de pe propriul calculator sau din rețeaua privată (`localhost`, `192.168.x.x`, `10.x.x.x` și altele) sunt refuzate, ca nicio pagină web să nu o poată păcăli să-ți citească serviciile interne. Dacă chiar vrei asta, de exemplu pentru propriul intranet, pornește CartaColibri cu `CARTACOLIBRI_GRAB_LOCAL=1`.

## Când ceva nu merge

Încearcă mai întâi `cartacolibri doctor`: îți verifică instalarea și îți spune ce trebuie reparat.

| Simptom | Cauză probabilă | Ce faci |
|---|---|---|
| `cartacolibri: command not found` | Folderul programului nu e în PATH. | Rulează-l cu calea completă, de exemplu `./target/release/cartacolibri`, sau folosește lansatorul din zip-ul portabil. |
| Pagina nu se deschide la `http://127.0.0.1:8765/` | Pagina locală nu rulează sau portul e ocupat. | Rulează `cartacolibri start`. Dacă portul e ocupat, rulează `cartacolibri serve --port 8766` și deschide acea adresă. |
| Nu știi ce nu merge | Lipsește ceva din configurare. | Rulează `cartacolibri doctor`: verifică configurarea și îți spune ce trebuie reparat. |
| Taskurile AI spun că nu ajung la Colibrì | Motorul Colibrì nu rulează sau ascultă pe altă adresă (implicit `http://127.0.0.1:8000`). | Pornește-l (`coli web --model ~/colibri-models/<model>`) sau dă-i adresa cu `--url`. Între timp bifează **Mod offline (fără model)**. |
| Fișierele noi nu apar în căutare | Indexul nu le-a văzut încă. | Apasă **Actualizează indexul** sau rulează `cartacolibri index`. `cartacolibri watch` îl ține la zi automat. |
| Unele fișiere apar la „Fișiere omise” | Sunt corupte, protejate, neacceptate sau scanări. | Rulează `cartacolibri problems` ca să vezi de ce. Pentru scanări, pornește OCR. |
| Scanările și imaginile nu dau text | OCR e oprit sau `tesseract` nu e instalat. | Instalează `tesseract` (și `pdftoppm` pentru PDF-uri) și adaugă `--ocr`, sau setează `CARTACOLIBRI_OCR=1`. |
| Răspunsul spune NOT FOUND | Fișierele alese nu conțin informația. E intenționat: nu face presupuneri. | Verifică dacă e ales grupul potrivit, reformulează întrebarea sau adaugă în grup fișierele care lipsesc. |
| Browserul web-grab nu se deschide | Lipsește o componentă de sistem. | Linux: instalează `libwebkit2gtk-4.1`. Windows: instalează WebView2 runtime. Între timp `cartacolibri grab URL` merge fără el. |
| Scriptul tău primește `403 missing X-Colibri-UI header` | E o verificare de siguranță, nu o greșeală. | Trimite antetul `X-Colibri-UI: 1` la fiecare apel către `/carta/*`. |
| Indexul pare stricat: rezultate ciudate, fișiere lipsă, o eroare despre baza de date | Fișierul de index a fost deteriorat (de pildă de o cădere sau un disc plin) sau nu mai este sincron cu folderele tale. | Deschide **Statistici → Verifică / Repară** sau rulează `cartacolibri verify --repair`. Reface indexul din fișierele tale; etichetele, notele și grupurile rămân. |
| Un fișier a fost sărit pentru că a fost „prea lent” | Citirea lui a depășit limita de timp (implicit 120 de secunde), de obicei un PDF uriaș sau deteriorat ori o scanare foarte mare. | Mărește limita, de exemplu `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, apoi **Actualizează indexul**. Sau împarte / reexportă fișierul. Motivul apare la „Fișiere sărite”. |
| „AI engine not running … (404)” | Adresa duce la altceva decât motorul, sau nu e încărcat niciun model. | Deschide ⚙, apasă „Caută motorul” și alege adresa listată; merg și `…:8000`, și `…:8000/v1`. Verifică să fie încărcat un model în Colibri Companion. |

## Glosar pe înțelesul tuturor

- **Index** — Un catalog al cuvintelor din fișierele tale, păstrat ca să dureze o clipă o căutare, în loc să se recitească tot.
- **RAG** — „Retrieval-augmented generation”: mai întâi găsește pasajele relevante din fișierele tale, apoi lasă AI-ul să răspundă folosind doar aceste pasaje.
- **Citare [S1]** — Un marcaj precum [S1] sau [S2] într-un răspuns. Arată fișierul-sursă din care provine propoziția, ca să poți verifica.
- **Grup** — O listă cu nume de fișiere (un proiect, o temă) pe care o poți căuta sau întreba din nou și din nou.
- **Gateway** — Mica adresă web (implicit `http://127.0.0.1:8000`) prin care CartaColibri vorbește cu motorul Colibrì.
- **Instantaneu (snapshot)** — O copie HTML salvată a unei pagini web, făcută de fiecare dată când o salvezi cu web-grab, ca să păstrezi ce scria în ziua aceea.
- **Watch (urmărire)** — Două lucruri: `cartacolibri watch` ține indexul la zi, iar `cartacolibri webwatch` aduce din nou paginile pe care le urmărești și arată ce s-a schimbat.

## Confidențialitate și siguranță

- **Unde se păstrează lucrurile.** Tot ce scrie CartaColibri stă într-un singur folder, `<home>`: implicit `~/.cartacolibri`, sau folderul din `CARTACOLIBRI_HOME`. Înăuntru: `index.sqlite` (indexul de căutare), `groups.json`, `library/uploads/` (fișierele încărcate), `outputs/` (rezultatele pe care ai ales să le salvezi), `snapshots/` și `web.json` (paginile web salvate), `highlights.json`, `watch.json`, `settings.json` și `clip.token` (un secret pentru bookmarklet: nu-l da nimănui).
- **Originalele tale sunt doar pentru citire.** CartaColibri îți citește documentele și nu le modifică, nu le mută și nu le șterge niciodată.
- **Pagina e doar locală.** Serverul ascultă numai pe `127.0.0.1`, deci alte calculatoare din rețea nu îl pot atinge. Refuză și cererile a căror adresă sau origine nu e locală.
- **Antetul `X-Colibri-UI`.** Fiecare apel al paginii către propriul server poartă acest antet. O pagină web vizitată într-un alt tab nu îl poate adăuga, deci nu poate face browserul tău să atingă aplicația locală pe la spatele tău.
- **Nimic nu iese de pe `127.0.0.1`, în afară de paginile pe care le ceri să fie aduse.** Fără telemetrie, fără analiză de utilizare, fără cloud. web-grab aduce doar adresele pe care i le dai (și paginile pe care ai ales să le urmărești). Nici site-ul de prezentare nu încarcă nimic de pe alte servere.
- **Istoricul e oprit implicit.** Istoricul browserului web-grab există doar dacă îl pornești (`cartacolibri history on`), rămâne pe discul tău, iar `cartacolibri history clear` îl șterge.

## Starea reală

CartaColibri este un software **alfa** (versiunea 0.2.0-alpha): funcționează, dar așteaptă-te la asperități și păstrează copii de rezervă pentru ce e important. Linux e platforma cel mai bine testată: browserul web-grab e autotestat acolo în CI. Pe **Windows și macOS** browserul este compilat în CI, dar **n-a fost niciodată rulat manual**. Verificările automate de pe GitHub (CI) trec pe Linux, Windows și macOS. Documentele tale sunt în siguranță oricum, fiindcă sunt doar citite. Descărcările portabile pentru Linux, Windows și macOS sunt pe pagina **Releases** de pe GitHub (marcate pre-lansare).

## Securitate

Ai găsit o vulnerabilitate? Raportează-o în privat, așa cum e descris în [SECURITY.md](../../SECURITY.md); te rugăm să nu deschizi mai întâi un issue public. Cum se apără serverul (doar loopback, antetul `X-Colibri-UI`, fără adrese private în web-grab, originale doar în citire) e descris în [docs/threat-model.md](../threat-model.md).

## Licență și mulțumiri

CartaColibri este publicat sub **licența MIT**. Este o implementare Rust clean-room: vorbește cu motorul [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) doar prin gateway-ul lui HTTP public și nu conține cod din el. Stratul de indexare și căutare urmează comportamentul descris în manualul public DocFetcher; nu se folosește cod DocFetcher. Browserul web-grab este construit pe webview-ul propriu al sistemului de operare și nu conține cod Falkon.

**Alexandru-Ionuț Chiuță** · **Alexandru Dumbrava** (coautor)

## Mai multe

[Documentație completă](../cartacolibri.md) · [README motor Colibrì](https://github.com/JustVugg/colibri) · [Contribuții](../../CONTRIBUTING.md) · [📄 Tutorial PDF ilustrat](CartaColibri-Tutorial.ro.pdf)
