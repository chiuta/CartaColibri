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
- **web-grab** — un mic browser în Rust (clean-room, înlocuiește Falkon), cu taburi, care îți cunoaște biblioteca: insignă când pagina e deja salvată (și când s-a schimbat), documente înrudite în timp ce navighezi, salvare dintr-un clic, întrebări către Colibrì local despre o pagină — sau peste toate taburile deschise, cu citări. O comandă și un bookmarklet acoperă alte browsere. În plus: 🖍 evidențieri salvate cu pagina, 👁 pagini urmărite cu diferența a ceea ce s-a schimbat (`cartacolibri webwatch`), instantaneu HTML la fiecare salvare, istoric căutabil opțional (`cartacolibri history on`), 🗂📝📊 rezumat sau extragere peste taburi și `?întrebare` în bara de adrese pentru a întreba toată biblioteca.

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
Text și notițe (txt, md, rst, log, csv, json, yaml, html, rtf), PDF, fișiere Word, OpenDocument, PowerPoint și Excel (docx, odt, pptx, xlsx), epub și, cu OCR pornit, imagini și PDF-uri scanate.

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

CartaColibri este un software **alfa** (versiunea 0.2.0-alpha): funcționează, dar așteaptă-te la asperități și păstrează copii de rezervă pentru ce e important. Linux e platforma cel mai bine testată: browserul web-grab e autotestat acolo în CI. Pe **Windows și macOS** browserul este compilat în CI, dar **n-a fost niciodată rulat manual**. Verificările automate de pe GitHub (CI) **nu sunt încă confirmate**: joburile au rămas blocate în coadă. Documentele tale sunt în siguranță oricum, fiindcă sunt doar citite.

## Licență și mulțumiri

CartaColibri este publicat sub **licența MIT**. Este o implementare Rust clean-room: vorbește cu motorul [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) doar prin gateway-ul lui HTTP public și nu conține cod din el. Stratul de indexare și căutare urmează comportamentul descris în manualul public DocFetcher; nu se folosește cod DocFetcher. Browserul web-grab este construit pe webview-ul propriu al sistemului de operare și nu conține cod Falkon.

## Mai multe

[Documentație completă](../cartacolibri.md) · [README motor Colibrì](https://github.com/JustVugg/colibri) · [Contribuții](../../CONTRIBUTING.md) · [📄 Tutorial PDF ilustrat](CartaColibri-Tutorial.ro.pdf)
