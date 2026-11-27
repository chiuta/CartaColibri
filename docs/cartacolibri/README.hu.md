<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>A dokumentumaid, kereshetően és érthetően — 100 %-ban helyben.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · <b>Magyar</b></p>

A CartaColibri egy helyi MI-könyvtár a Colibrì motorra építve. Indexeli a kiválasztott mappáidat, villámgyorsan kereshetsz bennük, majd egy helyi nyelvi modellel összefoglaltathatsz, adatokat nyerethetsz ki vagy szintetizálhatsz fájlcsoportokat — mindig megadva a forrásfájlt. Semmi sem hagyja el a gépedet, az eredeti fájlokat pedig soha nem módosítja.

## Az első 5 perced

Nem kell hozzá előképzettség. Három lépés, és már kerestél és összefoglaltál a saját fájljaidban.

1. **Indítsd el.** Csomagold ki a hordozható zipet és futtasd az indítót, vagy írd be a terminálba: `cartacolibri start`. A böngésződ megmutatja a helyi oldalt: `http://127.0.0.1:8765/`.
2. **Adj hozzá egy mappát.** Az oldalon nyisd meg a **Könyvtár** részt, írd be egy mappa útvonalát, például `~/Documents`, és kattints a **Mappa hozzáadása** gombra. Egy nagy mappa első indexelése eltarthat pár percig. A fájljaidat csak olvassa, soha nem módosítja.
3. **Keress, aztán kérdezz.** Nyisd meg a **RAG-fetch**-et, keress egy szót, jelölj ki pár fájlt, és hozz létre egy csoportot. Utána nyisd meg a **RAG-CARTA**-t, válaszd az **Összefoglalás** lehetőséget, és kattints az **Indítás** gombra. Még nincs MI-modelled? Jelöld be az **Offline mód (modell nélkül)** opciót, és próbáld ki úgy.

Inkább a terminált szereted? Ugyanez a három lépés:

```bash
cartacolibri start                 # step 1 - leave this window open (use a second one for the rest)
cartacolibri add ~/Documents       # step 2
cartacolibri group set Demo ~/Documents
cartacolibri summarize "main points" --group Demo --mock   # step 3 - --mock = no AI model needed
```

## Mit tud

- **Könyvtár** — az összes helyi dokumentum a helyével együtt; mappák hozzáadása és eltávolítása, fájlok feltöltése, megtekintése, kérésre OCR-szöveg mentése.
- **RAG-fetch** — DocFetcher-stílusú inkrementális teljes szövegű index (szavak, „kifejezések”, előtag*, AND/OR/NOT, szűrés típus, méret, hely szerint) és újrafelhasználható fájl**csoportok**.
- **RAG-CARTA** — három helyi feladat egy csoporton: **összefoglalás**, **kinyerés** (táblázat vagy JSON, találgatás helyett „NOT FOUND”), **szintézis** (témák, közös pontok, ellentmondások). Minden válasz idézi az [S1], [S2]… jelöléseket és felsorolja a forrásokat.
- **Alapból privát** — nincs felhő, nincs telemetria; az eredetiket csak olvassa. 8 nyelv: EN, RO, FR, IT, ES, PT, DE, HU.
- **web-grab** — egy kis Rust böngésző (clean-room) lapokkal, amely ismeri a könyvtáradat: jelvény, ha az oldal már mentve van (és ha megváltozott), kapcsolódó dokumentumok böngészés közben, mentés egy kattintással, kérdések a helyi Colibrìhez egy oldalról — vagy az összes nyitott lapról, hivatkozásokkal. Egy parancs és egy bookmarklet a többi böngészőt fedi le.

## Haladó funkciók

- **Keresés** — folyamatjelző az indexelés közben; „Erre gondoltál…?” elgépeléseknél, javaslatokkal, ha semmi sem egyezik; a duplikátumok listázva, és minden kihagyott fájlhoz érthető ok. Írj szűrőket közvetlenül a mezőbe (`ext:pdf`, `path:`, `size:>1mb`, `after:`/`before:` dátumok, `re:` reguláris kifejezések), jelöld be az **Okos keresés** négyzetet rokon szavakhoz, és nézd, hány találat van az egyes fájltípusokból.
- **Rendszerezés** — címkék, csillag és privát jegyzet minden dokumentumon; túlélik az újraindexelést. Nézd a kapcsolódó dokumentumokat, a nemrég és a leggyakrabban megnyitottakat, és mentsd el a kereséseket okos gyűjteményként.
- **14 feladat** — összehasonlítás, idővonal, szószedet, vázlat, teendők, GYIK, kártyák, kvíz, fordítás, magyarázat, ellentmondások, plusz összefoglalás, kinyerés és szintézis. Jelöld be az **idézetek ellenőrzését**, hogy megjelölje a forrásokkal alá nem támasztott mondatokat, és válaszd ki a válasz nyelvét.
- **Chat** — tegyél fel egymásra épülő kérdéseket az egész könyvtáradról vagy egy csoportról. A válaszok megadják a forrásaikat, vagy őszintén kimondják: „nem található a dokumentumaidban”, találgatás helyett. Minden válasz **Forrásokkal alátámasztva** pontszámot mutat, és megjelöli azokat a mondatokat vagy számokat, amelyeket a források nem támasztanak alá. A „Minden kérdés külön” bejelölésével nem emlékezik semmire, vagy időszakra korlátozhatod a dokumentumokat.
- **Megosztás és mentés** — exportálj egy csoportot, címkét vagy keresést Markdown, CSV, JSON vagy zip formátumban, hivatkozásokkal; Statisztika nézet diagramokkal és idővonallal; egyfájlos, ellenőrzőösszeges biztonsági mentés; Ellenőrzés/Javítás az indexhez.
- **Behozatal** — importálj böngészőkönyvjelzőket, Obsidian-mappát, Zotero/BibTeX-et, Pocket/Raindrop CSV-t vagy Evernote-exportot, és kövess RSS/Atom-csatornákat (OPML be- és kimenet): kereshető jegyzetekké válnak. Az e-mailek, naptárak, feliratok és notebookok is bekerülnek az indexbe.
- **Betekintés** — “Ellentmondások” (ugyanaz a mondat, más szám vagy dátum), “Tématérkép” (buborékok), “Mi az új” egy dátum óta, “2 összehasonlítása”, “Nevek és számok” (e-mailek, linkek, összegek, dátumok, telefonszámok), “Dosszié” egy névről és “Szövegem ellenőrzése” a könyvtár alapján.
- **Automatizálj és kapcsolódj** — a `digest` újra lefuttat egy mentett keresést, és csak az újdonságokat jelzi (ismétel, fájlhoz fűz, összefoglal vagy e-mail fájlt ment); az `mcp` lehetővé teszi, hogy más MI-alkalmazások keressenek, olvassanak, kérdezzenek, ellenőrizzenek és dossziét kérjenek a könyvtáradból, csak olvasásra.
- **Másolatok és dátumok** — “Duplikátumok” (másolat vagy majdnem másolat dokumentumok, hasonlóságukkal) és “Kronológia” (az összes dátumos mondat időrendben).

## Újdonságok a 0.2.0-alpha.10-ben

- **Mentés a LegatorOCR-ból közvetlenül a könyvtárba** — Ha a LegatorOCR-t a CartaColibriból nyitod meg, a Fájl menüje most már felkínálja a „PDF mentése a CartaColibri könyvtárba” pontot. A PDF fájlpárbeszédablak nélkül a könyvtár `from-legatorocr` mappájába kerül (a meglévő név -2, -3, … végződést kap, így semmi sem íródik felül), és a többivel együtt indexelődik. Ehhez olyan LegatorOCR kell, amely tartalmazza az új menüpontot; a régebbiek tovább működnek, csak kézzel mented ebbe a mappába.

## Újdonságok a 0.2.0-alpha.3-tól az alpha.9-ig

- **Szűrők közvetlenül a keresőmezőben** — Írd a szavaid mellé: `ext:pdf`, `path:szerzodesek`, `size:>1mb`, `after:2025-01`, `before:2025-06-30` vagy `re:SZ-\d+` (reguláris kifejezés), és a találatok szűkülnek. Működnek a weboldalon, a parancssorban és az API-n keresztül.
- **Okos keresés** — Jelöld be az „Okos keresés” négyzetet, és a helyi MI hozzáad néhány szinonimát és rokon szót (egyszerű OR-ként, így a pontos találatok maradnak elöl). A hozzáadott szavak a mező alatt látszanak. Futó motor nélkül a szokásos keresést kapod.
- **Egykérdéses csevegés** — Jelöld be a „Minden kérdés külön” négyzetet, és a csevegés elfelejti a korábbi kérdéseket: minden válasz csak a dokumentumaidból származik, forrásokkal.
- **MCP-szerver más MI-alkalmazásoknak** — A `cartacolibri mcp` lehetővé teszi, hogy más MI-alkalmazások a Model Context Protocolon át keressenek a könyvtáradban, elolvassanak egy dokumentumot és kérdezzenek róla. Csak olvasható, a gépeden fut, és csak a keresés által visszaadott dokumentumokat éri el.
- **Összefoglaló az újdonságokról** — A `cartacolibri digest NÉV` újra lefuttat egy mentett keresést, és csak az utolsó futás óta megjelent dokumentumokat jelzi. A `--every 60` óránként ismétli, az `--out jelentes.md` fájlhoz fűz, a `--summarize` pedig rövid összefoglalót kér a helyi MI-től.
- **Tudd, mennyire bízhatsz egy válaszban** — Minden csevegésválasz mostantól „Forrásokkal alátámasztva” pontszámot mutat: az egyes mondatok mekkora része található meg valóban az idézett forrásokban, az ellenőrzendő mondatokkal és a sehol sem szereplő számokkal.
- **Hasonlíts össze két dokumentumot, és lásd az összes nevet és számot** — Jelölj be két dokumentumot, és nyomd meg a „2 összehasonlítása” gombot: mondatonként látod, mi tér el két változat között. A „Nevek és számok” felsorolja a könyvtár összes e-mailjét, linkjét, összegét, dátumát és telefonszámát, és hogy hol szerepelnek. Parancssorban: `compare` és `entities`.
- **Találd meg az ellentmondásokat, nézd a tématérképet, kövesd a változásokat** — Az „Ellentmondások” felsorolja a különböző dokumentumok azon mondatait, amelyek egy számban vagy dátumban térnek el (500 és 550). A „Tématérkép” téma szerint csoportosítja a dokumentumokat. A „Mi az új” megmutatja, mi változott egy dátum óta. Parancssorban: `conflicts`, `topics`, `changes --since`.
- **Ellenőrizd a saját szövegedet a könyvtárad alapján** — Illessz be egy vázlatot a „Szövegem ellenőrzése” ablakba, és minden mondatot megjelölök: alátámasztva, részben, a számok eltérnek vagy nincs meg, a dokumentumaid legjobb bizonyítékával. A „Dosszié” összegyűjt mindent, amit a könyvtár egy névről vagy fogalomról mond, dátumokkal és összegekkel. Más MI-alkalmazások az MCP-szerveren át érik el mindkettőt.
- **Kérdezz egy időszakról, és kapd az összefoglalókat e-mail fájlként** — A csevegésben válaszd a „Dokumentumok innen … eddig …” mezőket, és a válasz csak az adott időszakban módosult dokumentumokból származik (parancssorban: `chat --since --until`). A tématérkép mostantól kattintható buborékokból álló kép. A `digest --mail-to te@example.org` minden jelentést e-mail fájlként is elment az outbox mappába; a `--sendmail` átadja a gép saját levelezőprogramjának. Más nem hagyja el a gépedet.
- **Kapcsold össze az ismert csevegőszolgáltatásokat, és hasonlítsd össze őket** — A `cartacolibri providers` kilistázza az Ollama, LM Studio, Mistral, OpenAI, Gemini, Cohere, NVIDIA, Claude és Meta szolgáltatást; a `--provider NÉV:MODELL` egyet használ a csevegésben és a feladatokban, a `versus` pedig ugyanazt a kérdést teszi fel többnek ugyanazokra a szövegrészekre. A felhőszolgáltatásokhoz `--allow-cloud` és kulcs kell, mert a szövegrészeid elküldésre kerülnek.
- **Csevegési előzmények és gomb a web-grab böngészőhöz** — A csevegés alatt most ott a korábbi beszélgetéseid listája: kereshetsz bennük, megnyithatsz egyet, hogy elolvasd és ott folytasd, ahol abbahagytad, törölhetsz egyet vagy mindet. Csak a böngésződben tárolódnak. A „Behozatal máshonnan” ablakban egy új gomb megnyitja a web-grab böngészőt ugyanazzal a könyvtárral. Az egész előzményt egyetlen fájlba is mentheted, és vissza is importálhatod; ugyanaz a nyílt formátum, mint a Colibri Companionban ([docs/CHAT-HISTORY.hu.md](../CHAT-HISTORY.hu.md)).
- **OCR a LegatorOCR-rel, „Kijelöltek törlése”, érthető kihagyási okok** — A beolvasott lapokat és fotókat mostantól a LegatorOCR is olvashatja, ugyanaz a program, amit a Colibri Companion használ (a tesseract továbbra is működik). Az új „Kijelöltek törlése” gomb, a `cartacolibri forget` és a kihagyott fájlok melletti ✕ véglegesen kiveszi őket a könyvtárból, a fájlokhoz nem nyúlva. A kihagyás okai most az általad választott nyelven jelennek meg, a 80 MB-os korlát pedig a CARTACOLIBRI_MAX_FILE_MB-vel emelhető.
- **Találd meg a másolatokat, és olvasd a könyvtárat idővonalként** — A „Duplikátumok” felsorolja a másolatnak vagy majdnem másolatnak számító dokumentumokat, hasonlóságukkal. A „Kronológia” időrendbe teszi a könyvtár összes dátumos mondatát, a dokumentumukkal. Parancssorban: `dupes`, `chronology`.
- **Keresés jelentés szerint, a keresés mérése, több formátum** — Az `embed` a helyi motorod embedding-modelljével előkészíti a jelentés szerinti keresést; utána a `search --mode hybrid` olyan szövegeket is megtalál, amelyek más szavakat használnak, mint a kérdésed, az `eval` pedig megméri, mennyire találja meg az egyes mód azt, amit vársz. Új formátumok: tar és tar.gz, OpenDocument táblázatok és prezentációk, makrós Office-fájlok, valamint a régi doc, xls és ppt az antiword, catdoc vagy LibreOffice segítségével. A teljes könyvtáron dolgozó eszközök (nevek és számok, másolatok, ellentmondások, témák) 20 000 dokumentumnál percekről egy másodperc alá gyorsultak, a szűrők pedig most már az egész könyvtárat látják.
- **Őszinte gombok, és nincs több 10 perces megszakítás** — A „Web-grab böngésző megnyitása” most ellenőrzi, hogy a böngésző tényleg megnyílt-e, és ha nem, megmutatja az általa adott okot. Az OCR-beállítások mellett egy új „A LegatorOCR megnyitása” gomb megnyitja a LegatorOCR ablakát. A hosszú feladatot vagy csevegést (egy nagy modell egy nagy csoporton sok percig tarthat) az oldal már nem szakítja meg 10 perc után az „The operation was aborted” üzenettel: csak a motor saját korlátja számít, és egy stopper mutatja, mióta fut.
- **A LegatorOCR könnyebben megtalálható, a széles szkennelések egészben olvashatók** — A CartaColibri most a saját mellett, egy `App` mappában és egy kicsomagolt letöltés mappáiban keresi a LegatorOCR-t, és újraindítás nélkül észreveszi a később bemásolt példányt. A széles JPG-szkenneléseket már nem vágja ketté, így nem vesznek el sorok, a beszkennelt PDF-eket pedig telepített poppler mellett is OCR-rel olvassa. A „A LegatorOCR megnyitása” egy `from-legatorocr` mappát is létrehoz, amely a könyvtárad része: mentsd ide a PDF-et vagy a szöveget, és a következő indexeléskor bekerül a könyvtárba.

## Újdonságok a 0.2.0-alpha.2-ben

- **A motor címe magától működik** — A `http://127.0.0.1:8000` és a `http://127.0.0.1:8000/v1` is jó. A CartaColibri a `/v1/models` címről olvassa a modellek listáját, magától kiválasztja a betöltött modellt, és nincs szüksége `/health` oldalra. Ez megoldja az „AI engine not running (404)” üzenetet.
- **Magától megtalálja az MI-det** — A CartaColibri keres egy futó motort ezen a gépen: a Colibri Companion saját beállításait, egy felfedezőfájlt, majd a szokásos helyi portokat (8000, 8080, 11434, 1234, 8001, 5000). Csak loopback címeket vizsgál. Nyisd meg a ⚙-t, hogy lásd, mit talált, vagy nyomd meg a „Colibri Companion indítása” gombot.
- **Élő állapot és sebesség** — A fejléc mutatja a futó modellt és azt, hogy automatikusan találta-e meg, néhány másodpercenként frissítve. Minden válasz alatt látod a felhasznált tokeneket és a sebességet; 3 token/s alatt javaslatot kapsz kisebb modellre. A kérések egyenként mennek a motorhoz.
- **Weboldalak az importálás ablakából** — A „Behozatal máshonnan” ablakban a hírcsatornák helyét a web-grab vette át: illessz be egy címet, és mentsd az oldalt egyetlen önálló HTML- vagy MHTML-fájlként a könyvtárba. A csatornák továbbra is ott vannak, összecsukva.
- **Három gomb a web-grab böngészőben** — A „Save to folder” a szöveges másolatot a könyvtárba menti, a „Grab website HTML” az oldalt menti (Shift+kattintás: MHTML), a „PDF” PDF-be nyomtatja. Linuxon a PDF csendben készül; Windowson és macOS-en megnyílik a nyomtatási ablak, és te választod a „Mentés PDF-ként” lehetőséget.
- **A mentett oldalak kereshetők** — A `.mhtml` és `.html` fájlok a helyes karakterkészlettel kerülnek az indexbe, így a mentett oldal úgy található meg, mint bármely dokumentum. Az oldal privát címeken lévő részeit sosem tölti le és nem hivatkozza.

## Gyors kezdés

Fordítsd le (vagy töltsd le a hordozható zipet), és indítsd el a helyi oldalt. Az MI-feladatokhoz indítsd el a Colibrì motort is a RAM-odba férő modellel (lásd a dokumentációját):

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
cargo build --release                       # Rust 1.90+, or use the portable zip
./target/release/cartacolibri serve         # then open http://127.0.0.1:8765/
coli web --model ~/colibri-models/<model>   # Colibrì engine, only for the AI tasks
```

Vagy használd a parancssort; kipróbálni modell nélkül is lehet (`--mock` offline tartalék mód):

```bash
cartacolibri add ~/Documents
cartacolibri search "contract*" --ext pdf,docx
cartacolibri group set Contracts ~/Documents/contracts
cartacolibri synthesize "payment terms" --group Contracts --save --mock
cartacolibri grab https://example.org/article      # web-grab (command line)
cartacolibri-browser https://example.org           # web-grab (browser)
```

## További parancsok

| Parancs | Mit csinál |
|---|---|
| `start [--folder F] [--port N] [--no-open] [--no-serve]` | Vezetett indítás: kiválasztasz egy mappát, indexeli, és megnyílik a weboldal. |
| `doctor` | Ellenőrzi a motort, az OCR-t, a PDF-eszközöket, a lemezt és a portot, és minden hibához megadja a pontos megoldást. |
| `add FOLDER`, `remove FOLDER`, `folders`, `index [FOLDER]`, `problems` | Mappa felvétele vagy elfelejtése (a fájlokhoz soha nem nyúl), mappák listázása, újraindexelés és az olvashatatlan fájlok listája. |
| `watch [--interval S]` | Naprakészen tartja az indexet: S másodpercenként újraolvassa a mappákat. |
| `search QUERY [--ext pdf,txt] [--min-kb N] [--max-kb N] [--path P] [--group G] [--mode bm25\|semantic\|hybrid] [--json]` | Keres a könyvtárban. Szavak, „kifejezések”, előtag*, AND/OR/NOT és szűrők közvetlenül a lekérdezésben: `ext:pdf` `path:szerzodesek` `size:>1mb` `after:2025-01` `before:2025-06-30` `re:SZ-\d+`. |
| `list [--group G] [--json]`, `group set/list/show/rm/from-search` | Listázza a dokumentumokat; a csoportok elnevezett dokumentumhalmazok, amelyekben kereshetsz vagy kérdezhetsz. |
| `summarize`, `extract`, `synthesize` (same options as `task`, with `--fields`) | Gyorsparancsok a három fő feladathoz: minden dokumentum összefoglalása, mezők kinyerése, vagy több dokumentum egy válaszba vonása. |
| `grab URL`, `bookmarklet` | Weboldalt ment a könyvtárba (HTML, MHTML vagy PDF); a bookmarklet bármely böngészőből megteszi. |
| `serve [--port N] [--url ENGINE]` | Elindítja a weboldalt ezen a gépen (http://127.0.0.1:PORT). |
| `completions bash\|zsh\|fish\|powershell`, `import-json FILE` | Tab-kiegészítő szkriptet ír ki a shelledhez; újraimportálja a JSON-ként exportált címkéket, csillagokat, jegyzeteket és csoportokat. |
| `task NAME [request] --group G \| --tag T \| --query Q [--mock] [--verify] [--lang xx] [--save md\|html\|txt]` | a 14 feladat (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); a `--verify` megjelöli azokat a mondatokat, amelyek `[S#]` hivatkozását a szöveg nem támasztja alá |
| `chat [--group G] [--since DATE] [--until DATE] [--lang xx] [--fallback]` | egymásra épülő kérdések a könyvtáradról; a válaszok hivatkoznak, vagy „nem található a dokumentumaidban” |
| `tag add/rm/list/show`, `star [--off]`, `note` | a saját rendszerezésed, amely az újraindexelés után is megmarad |
| `saved add/list/run/rm` | mentett keresések / okos gyűjtemények |
| `recent`, `related PATH` | amit nemrég megnyitottál; hasonló dokumentumok |
| `export [--saved NAME] --format md\|csv\|json\|zip` | oszd meg egy csoportot, címkét vagy mentett keresést hivatkozásokkal |
| `backup OUT [--index]`, `backup-check ZIP`, `restore ZIP` | egyfájlos, ellenőrzőösszeges mentés; ellenőrzése; visszaállítása |
| `verify [--repair]` | ellenőrzi az indexet; a `--repair` a fájljaidból újraépíti |
| `stats`, `timeline` | számok a könyvtáradról; dokumentumok havonként |
| `import PATH` | könyvjelzők `.html`, Obsidian-mappa, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex` |
| `feeds add/list/rm/poll`, `feeds opml-import/opml-export` | RSS/Atom-feliratkozások, jegyzetként mentve |
| `webwatch`, `history` | figyelt weboldalak különbségekkel; opcionális böngészési előzmények |
| `forget PATH… \| --list \| --undo PATH…` | Véglegesen kiveszi a fájlokat a könyvtárból (mint a „Kijelöltek törlése” a weboldalon): maguk a fájlok érintetlenek maradnak, és az újraindexelés sem hozza vissza őket. A `--list` megmutatja, az `--undo` visszaengedi őket. |
| `providers [--json]` | Kilistázza a csevegőszolgáltatásokat, amelyekhez kapcsolódhatsz (Ollama, LM Studio, Mistral, OpenAI, Gemini, Cohere, NVIDIA, Claude, Meta), és hogy melyik kész. A `chat`, `task`, `summarize`… parancsoknál a `--provider NÉV[:MODELL]` használja; a felhőalapúak mellé `--allow-cloud` és kulcs is kell, mert a szövegrészeid elküldésre kerülnek. |
| `versus "QUESTION" [--providers mock,colibri,ollama:llama3.2,mistral:MODEL…] [--group G] [--lang xx] [--allow-cloud] [--json]` | Ugyanazt a kérdést teszi fel több modellnek ugyanazokra a szövegrészekre, és összeveti őket: idő, hossz, idézetek és hány tűnik alátámasztatlannak, majd az összes válasz. |
| `digest NAME [--every MIN] [--out FILE.md] [--summarize] [--all] [--mail-to ADDR [--sendmail]]` | Újra lefuttat egy mentett keresést, és csak az utolsó futás óta újakat jelzi; ismételhető, fájlhoz fűzhető, a helyi MI összefoglalhatja, vagy e-mail fájlként menthető az outbox mappába. |
| `entities [--kind email\|url\|money\|date\|phone] [--group G] [--json]` | A dokumentumaid összes e-mailje, linkje, összege, dátuma és telefonszáma, azokkal a dokumentumokkal, ahol szerepelnek. |
| `compare A B [--json]` | Mondatonként összehasonlít két dokumentumot: mennyire hasonlók, és mi van csak az egyikben (A és B útvonal vagy annak vége). |
| `conflicts [--group G] [--json]` | Különböző dokumentumok olyan mondatpárjai, amelyek egy számban vagy dátumban térnek el (500 és 550): ellenőrizendő jelöltek, nem ítéletek. |
| `dupes [--group G] [--min 0.6] [--json]` | Megkeresi azokat a dokumentumokat, amelyek egymás másolatai vagy majdnem másolatai (exportok, piszkozatok, kétszer mentett fájl), hasonlóságukkal és azzal, hogy a kisebbik mekkora része van a másikban. |
| `chronology [--group G] [--limit N] [--json]` | A könyvtár összes dátumos mondata időrendben, a dokumentumukkal (dátumok: 2025-03-12, 12.03.2025 vagy 12/03/2025, előbb a nap; vagy hónapnévvel a 8 nyelv bármelyikén: 2025. március 12., 12 March 2025). A havi dokumentumszám a `timeline`. |
| `embed [--embed-model M] [--batch 32] [--limit N] [--reset] [--mock] [--json]` | Előkészíti a jelentés szerinti keresést: a helyi motorod embedding-modellje vektort készít minden szövegrészből, és a könyvtár saját indexében tárolja. Inkrementális (csak az új vagy módosult részek), folytatható (`--limit`), és semmi nem hagyja el a gépedet. Ezután a `search --mode semantic` vagy `--mode hybrid` (szavak és jelentés együtt) olyan szövegeket is megtalál, amelyek más szavakat használnak, mint a kérdésed. |
| `eval FILE [--modes bm25,semantic,hybrid] [--json]` | Megméri, mennyire találja meg a keresés azt, amit vársz. A FILE soronként egy kérdést tartalmaz a várt fájlokkal (`kérdés<TAB>fájl1,fájl2`), vagy JSON (`[{"q": "...", "expect": ["lease.txt"]}]`). Módonként kiírja a hit@1, hit@5, hit@10 és MRR értékeket, és a nem talált kérdéseket. |
| `topics [--json]` | Téma szerint csoportosítja a dokumentumokat (legjellemzőbb szavaik alapján). |
| `dossier TERM [--json]` | Mindaz, amit a könyvtár egy névről vagy fogalomról mond: szövegrészek dokumentumonként, a bennük talált dátumokkal és összegekkel. |
| `changes --since DATE [--json]` | Egy dátum óta módosult dokumentumok (YYYY, YYYY-MM vagy YYYY-MM-DD), a legfrissebbek elöl. |
| `factcheck FILE\|- [--json]` | Ellenőrzi a saját szövegedet a könyvtár alapján: minden mondat alátámasztott, részben, a számok eltérnek vagy nincs meg, a legjobb bizonyítékkal. |
| `mcp [--url ENGINE] [--mock]` | MCP-szerver stdio-n, hogy más MI-alkalmazások keressenek, olvassanak, kérdezzenek, ellenőrizzenek és dossziét kérjenek a könyvtáradból (csak olvasás). |

## Hogyan áll össze

0. réteg — a mappáid (csak olvasás) → RAG-fetch — index, keresés, csoportok → RAG-CARTA — MI-feladatok a Colibrì motoron át.

## Gyakori kérdések

**Mit jelent az, hogy „helyi”?**  
Minden a saját gépeden fut: az oldal, a keresőindex és az MI-modell. Hálózat nélkül is működik. Az egyetlen kivétel a web-grab, amely letölti azokat a weboldalakat, amelyeket kérsz tőle.

**Küld valahová adatot?**  
Nem. Nincs felhő, nincs fiók, nincs telemetria, nincs használati statisztika. Az egyetlen forgalom, ami elhagyja a gépedet, azok a weboldalak, amelyeket kifejezetten kérsz a web-grabtól. Részletek az „Adatvédelem és biztonság” részben.

**A Colibrì nem fut. Használhatom így is?**  
Igen. A Könyvtárhoz és a kereséshez soha nincs szükség modellre. Az összefoglaláshoz, kinyeréshez és szintézishez jelöld be az **Offline mód (modell nélkül)** opciót (parancssorban `--mock`): egyszerű, a fájljaidból épített választ kapsz hivatkozásokkal, de valódi MI-megfogalmazás nélkül. Igazi válaszokhoz indítsd el a Colibrì motort.

**Mi az a Colibrì, és kell nekem?**  
A [Colibrì](https://github.com/JustVugg/colibri) egy különálló helyi motor, amely a nyelvi modellt futtatja. Csak az MI-válaszokhoz kell. Olyan modellt válassz, amelynek a fájlja kisebb a szabad RAM-odnál, és kezdj egy kicsivel. Lásd a Colibrì saját dokumentációját.

**Milyen fájlokat tud olvasni?**  
Szöveget és jegyzeteket (txt, md, rst, log, csv, json, yaml, html, rtf), PDF-et, Word-, OpenDocument-, PowerPoint- és Excel-fájlokat (docx, odt, pptx, xlsx), epubot, bekapcsolt OCR mellett pedig képeket és beszkennelt PDF-eket. Valamint e-mailt (eml, mbox), naptárat és névjegyet (ics, vcf), zip-, tar- és tar.gz-archívumot, OpenDocument táblázatot és prezentációt (ods, odp), továbbá a régi doc, xls és ppt fájlokat, ha az antiword, a catdoc vagy a LibreOffice telepítve van.

**És a szkennelt vagy lefényképezett dokumentumok?**  
Kapcsold be az OCR-t (`--ocr`, vagy `CARTACOLIBRI_OCR=1`). Ehhez a gépeden telepítve kell lennie a `tesseract`-nak, a szkennelt PDF-ekhez pedig a `pdftoppm`-nek is. Az oldal szól, ha a tesseract hiányzik.

**Hogyan frissítek?**  
Hordozható verzió: töltsd le az újabb zipet és csomagold ki. Forrásból: `git pull`, majd `cargo build --release`. Az adataid a `~/.cartacolibri` mappában ott maradnak, az index pedig magától frissül, csak ami megváltozott.

**Hogyan törlök mindent?**  
Töröld a program mappáját, aztán az állapotmappát: `~/.cartacolibri` (vagy amerre a `CARTACOLIBRI_HOME` mutat). Ennyi: a saját dokumentumaidhoz soha nem nyúlt.

**Van valami különleges Windowson vagy macOS-en?**  
A hordozható zip mindkettőre létezik, de a Linux a legjobban tesztelt rendszer. A web-grab böngészőt a CI Windowsra és macOS-re lefordítja, de ott kézzel még soha nem futtattuk. Windowson szüksége van a WebView2 futtatókörnyezetre. Ha a rendszered megerősítést kér egy letöltött program megnyitásához, csak akkor erősítsd meg, ha a projekt oldaláról szerezted.

**Miért „alfa”?**  
Mert fiatal: a mag működik és tesztelt, de nem minden rendszeren, és a GitHub automatikus ellenőrzései még nincsenek megerősítve. Számíts apróbb hibákra, és jelezd nekünk őket.

**Tud jelentés alapján keresni, nem csak szavak szerint?**  
Igen, opcionálisan. Futtasd egyszer a `cartacolibri embed` parancsot: a helyi motorod embedding-modellje számokká alakít minden szövegrészt, és a saját indexfájlodban tárolja (később csak az új vagy módosult részeket készíti újra). Ezután a `search --mode hybrid` (vagy `semantic`) akkor is megtalál egy szöveget, ha más szavakat használ, mint a kérdésed. Ehhez a motorban embedding-modell kell (Ollamánál: `ollama pull nomic-embed-text`). Nélküle a szószerinti keresés ugyanúgy működik, mint eddig. A `cartacolibri eval` megméri, mennyire találja meg az egyes módok azt, amit vársz.

**Mit nem tud?**  
Nincsenek önállóan cselekvő ügynökei, nincs webes keresés, felhasználói fiók, telefonos app és számítógépek közti szinkron. A szószerinti keresés pontos, de az elemző eszközök (ellentmondások, duplikátumok, kronológia, nevek és számok) szó- és számszabályokat használnak, nem megértést: ellenőrizendő dolgokra mutatnak, nem ítéletek. A válaszok annyit érnek, amennyit a helyben futtatott modell. A 8 fordítást nem nézték át anyanyelvi beszélők, a projekt pedig alfa. A .msg és .pst levelek nem olvashatók (exportáld .eml vagy .mbox formátumba), a hang és a videó nincs átírva.

**Hogyan élik túl a címkék, csillagok és jegyzetek az újraindexelést?**  
Nem a keresőindexben vannak, hanem egy külön kis fájlban, az `organize.json`-ban, a `<home>` mappádban, minden dokumentum útvonalához kötve. Az **Index frissítése**, a teljes újraindexelés és még az **Ellenőrzés/Javítás** (amely újraépíti az indexet) sem nyúl hozzájuk. Ha átnevezel vagy áthelyezel egy fájlt, új dokumentumnak számít, a címkék pedig a régi útvonalnál maradnak.

**Hogyan készítek mentést, és hogyan állítom vissza?**  
Nyisd meg a **Statisztika** nézetet, és nyomd meg a **Mentés letöltése** gombot (jelöld be a négyzetet, ha az indexet is szeretnéd), vagy futtasd: `cartacolibri backup my.zip`. Egyetlen zip, ellenőrzőösszegekkel, a címkéiddel, jegyzeteiddel, csoportjaiddal, mentett keresésekkel és importált jegyzetekkel, az eredeti dokumentumaid soha nincsenek benne. A `cartacolibri backup-check my.zip` ellenőrzi, a `cartacolibri restore my.zip` mindent visszaállít. A zipet tartsd máshol is, ne csak ezen a gépen.

**Hogyan importálok könyvjelzőket, Obsidiant vagy Zotero-t?**  
Kattints a Könyvtárban a **Behozatal** gombra, vagy futtasd: `cartacolibri import ÚTVONAL`. Érti a böngésző könyvjelző-exportját (`.html`), az Obsidian-mappát, a Zotero/BibTeX `.bib` fájlt, a Pocket vagy Raindrop `.csv` fájlokat és az Evernote `.enex` exportokat. Minden elem kis jegyzet lesz a könyvtáradban, amelyben kereshetsz, címkézhetsz és hivatkozhatsz. Az eredeti fájlokat csak olvassa.

**Hogyan működnek a csatornák (feedek)?**  
A **Behozatal → Csatornák** résznél illessz be egy RSS- vagy Atom-címet, és nyomd meg a **Hozzáadás** gombot. Amikor a **Ellenőrzés most** gombra kattintasz (vagy lefuttatod a `cartacolibri feeds poll` parancsot), az új elemek jegyzetként mentődnek és indexelődnek, így kereshetsz bennük és kérdezhetsz róluk. A háttérben semmi nem fut magától. A feliratkozásaidat OPML-ként hozhatod be és viheted ki.

**Mit csinál az „idézetek ellenőrzése”?**  
Miután az MI válaszolt, a CartaColibri újra elolvassa minden mondatot. Ha egy mondatnak nincs `[S#]` hivatkozása, vagy a hivatkozott forrás valójában nem tartalmazza, amit a mondat állít, `[unsupported]` jelölést kap. Ez gyors biztonsági háló, nem garancia: ami fontos, ott nyisd meg a forrást.

**Miért utasítottak el egy weboldalt?**  
A web-grab csak nyilvános oldalakat tölt le. A saját gépeden vagy privát hálózaton lévő címeket (`localhost`, `192.168.x.x`, `10.x.x.x` és hasonlók) elutasítja, hogy egy weboldal ne tudja rávenni a belső szolgáltatásaid olvasására. Ha tényleg ezt szeretnéd, például a saját intranetedhez, indítsd a CartaColibrit a `CARTACOLIBRI_GRAB_LOCAL=1` beállítással.

## Ha valami nem működik

Először próbáld ki a `cartacolibri doctor` parancsot: ellenőrzi a beállításaidat, és megmondja, mit kell javítani.

| Tünet | Valószínű ok | Teendő |
|---|---|---|
| `cartacolibri: command not found` | A program mappája nincs a PATH-odban. | Futtasd a teljes útvonalával, például `./target/release/cartacolibri`, vagy használd a hordozható zip indítóját. |
| Az oldal nem nyílik meg a `http://127.0.0.1:8765/` címen | A helyi oldal nem fut, vagy foglalt a port. | Futtasd: `cartacolibri start`. Ha foglalt a port, futtasd: `cartacolibri serve --port 8766`, és nyisd meg azt a címet. |
| Nem tudod, mi a baj | Valami hiányzik a beállításokból. | Futtasd: `cartacolibri doctor`: ellenőrzi a beállításokat, és megmondja, mit kell javítani. |
| Az MI-feladatok szerint nem érik el a Colibrìt | A Colibrì motor nem fut, vagy más címen figyel (alapértelmezés: `http://127.0.0.1:8000`). | Indítsd el (`coli web --model ~/colibri-models/<model>`), vagy add meg a címét a `--url` kapcsolóval. Addig jelöld be az **Offline mód (modell nélkül)** opciót. |
| Az új fájlok nem jelennek meg a keresésben | Az index még nem látta őket. | Kattints az **Index frissítése** gombra, vagy futtasd: `cartacolibri index`. A `cartacolibri watch` automatikusan naprakészen tartja. |
| Néhány fájl a „Kihagyott fájlok” között szerepel | Sérültek, védettek, nem támogatottak, vagy szkennelések. | Futtasd: `cartacolibri problems`, és megtudod az okot. Szkenneléseknél kapcsold be az OCR-t. |
| A szkennelések és képek nem adnak szöveget | Az OCR ki van kapcsolva, vagy a `tesseract` nincs telepítve. | Telepítsd a `tesseract`-ot (PDF-hez a `pdftoppm`-et is), és add hozzá a `--ocr` kapcsolót, vagy állítsd be: `CARTACOLIBRI_OCR=1`. |
| A válasz: NOT FOUND | A kiválasztott fájlok nem tartalmazzák az információt. Ez szándékos: soha nem találgat. | Nézd meg, hogy a megfelelő csoport van-e kiválasztva, fogalmazd át a kérdést, vagy add hozzá a hiányzó fájlokat a csoporthoz. |
| A web-grab böngésző nem nyílik meg | Hiányzik egy rendszerkomponens. | Linux: telepítsd a `libwebkit2gtk-4.1` csomagot. Windows: telepítsd a WebView2 futtatókörnyezetet. Addig a `cartacolibri grab URL` nélküle is működik. |
| A saját szkripted `403 missing X-Colibri-UI header` hibát kap | Ez biztonsági ellenőrzés, nem hiba. | Küldd el az `X-Colibri-UI: 1` fejlécet minden `/carta/*` hívással. |
| Az index hibásnak tűnik: furcsa találatok, hiányzó fájlok, adatbázis-hiba | Az indexfájl megsérült (például összeomlás vagy tele lemez miatt), vagy elcsúszott a mappáidhoz képest. | Nyisd meg a **Statisztika → Ellenőrzés / Javítás** részt, vagy futtasd: `cartacolibri verify --repair`. A fájljaidból újraépíti az indexet; a címkéid, jegyzeteid és csoportjaid megmaradnak. |
| Egy fájlt kihagyott, mert „túl lassú” volt | Az olvasása átlépte az időkorlátot (alapból 120 másodperc), általában egy hatalmas vagy sérült PDF, esetleg egy nagyon nagy szkennelés. | Emeld meg a korlátot, például `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, majd **Index frissítése**. Vagy darabold fel / exportáld újra a fájlt. Az ok a „Kihagyott fájlok” alatt látszik. |
| „AI engine not running … (404)” | A cím valami másra mutat, nem a motorra, vagy nincs betöltött modell. | Nyisd meg a ⚙-t, nyomd meg a „Motor keresése” gombot, és válaszd a listázott címet; a `…:8000` és a `…:8000/v1` is jó. Ellenőrizd, hogy a Colibri Companionban be van töltve egy modell. |

## Szószedet egyszerű szavakkal

- **Index** — A fájljaidban található szavak katalógusa, hogy egy keresés egy pillanat legyen ahelyett, hogy mindent újraolvasna.
- **RAG** — „Retrieval-augmented generation”: először megkeresi a releváns részeket a fájljaidban, aztán az MI csak ezek alapján válaszol.
- **Hivatkozás [S1]** — Egy jelölés, például [S1] vagy [S2] egy válaszban. A forrásfájlra mutat, ahonnan a mondat származik, így ellenőrizheted.
- **Csoport** — Fájlok névvel ellátott listája (egy projekt, egy téma), amelyben újra és újra kereshetsz vagy kérdezhetsz.
- **Gateway (átjáró)** — A kis webcím (alapértelmezés: `http://127.0.0.1:8000`), amelyen át a CartaColibri a Colibrì motorral beszél.
- **Pillanatkép (snapshot)** — Egy weboldal elmentett HTML-másolata, amelyet minden web-grabbel végzett mentésnél készít, hogy megmaradjon, mi állt rajta aznap.
- **Watch (figyelés)** — Két dolog: a `cartacolibri watch` naprakészen tartja az indexet, a `cartacolibri webwatch` pedig újra letölti a követett oldalakat, és megmutatja, mi változott.

## Adatvédelem és biztonság

- **Hol vannak a dolgok.** Minden, amit a CartaColibri ír, egyetlen mappában él, a `<home>`-ban: alapértelmezés szerint `~/.cartacolibri`, vagy a `CARTACOLIBRI_HOME` által megadott mappa. Benne: `index.sqlite` (a keresőindex), `groups.json`, `library/uploads/` (a feltöltött fájlok), `outputs/` (az általad mentett eredmények), `snapshots/` és `web.json` (a mentett weboldalak), `highlights.json`, `watch.json`, `settings.json` és `clip.token` (a bookmarklet titka: ne oszd meg).
- **Az eredetiek csak olvashatók.** A CartaColibri olvassa a dokumentumaidat, de soha nem módosítja, mozgatja vagy törli őket.
- **Az oldal kizárólag helyi.** A szerver csak a `127.0.0.1` címen figyel, így a hálózatod többi gépe nem éri el. Azokat a kéréseket is elutasítja, amelyeknek a címe vagy eredete nem helyi.
- **Az `X-Colibri-UI` fejléc.** Az oldal minden, a saját szerverének küldött hívása hordozza ezt a fejlécet. Egy másik lapon megnyitott weboldal nem tudja hozzáadni, így nem tudja rávenni a böngésződet, hogy a hátad mögött megbökje a helyi alkalmazást.
- **Semmi nem hagyja el a `127.0.0.1` címet, kivéve azokat az oldalakat, amelyeket letöltetsz.** Nincs telemetria, nincs statisztika, nincs felhő. A web-grab csak azokat a címeket tölti le, amelyeket megadsz (és azokat az oldalakat, amelyeket figyelni kértél). A bemutatóoldal sem tölt be semmit más szerverekről.
- **Az előzmények alapból ki vannak kapcsolva.** A web-grab böngésző előzményei csak akkor léteznek, ha bekapcsolod (`cartacolibri history on`), a lemezeden maradnak, és a `cartacolibri history clear` törli őket.

## Őszinte állapotjelentés

A CartaColibri **alfa** állapotú szoftver (0.2.0-alpha verzió): működik, de számíts apróbb hibákra, és a fontos dolgaidról tarts mentést. A Linux a legjobban tesztelt platform: ott a web-grab böngészőt a CI automatikusan kipróbálja. **Windowson és macOS-en** a böngészőt a CI lefordítja, de **kézzel még soha nem futtattuk**. A GitHubon az automatikus ellenőrzések (CI) Linuxon, Windowson és macOS-en átmennek. A dokumentumaid így is biztonságban vannak, mert csak olvassa őket. A hordozható letöltések Linuxra, Windowsra és macOS-re a GitHub **Releases** oldalán vannak (előzetes kiadásként jelölve).

## Biztonság

Sebezhetőséget találtál? Jelezd privátban, ahogy a [SECURITY.md](../../SECURITY.md) leírja; kérlek, ne nyiss előbb nyilvános hibajegyet. Azt, hogyan védekezik a szerver (csak loopback, `X-Colibri-UI` fejléc, a web-grab nem megy privát címekre, az eredetik csak olvashatók), a [docs/threat-model.md](../threat-model.md) írja le.

## Licenc és köszönet

A CartaColibri **MIT licenc** alatt jelenik meg. Clean-room Rust implementáció: a [Colibrì](https://github.com/JustVugg/colibri) motorral (JustVugg, Apache-2.0) kizárólag a nyilvános HTTP-átjárón át beszél, és nem tartalmaz belőle kódot. Az index- és keresőréteg a DocFetcher nyilvános kézikönyvében leírt viselkedést követi; DocFetcher-kód nincs benne. A web-grab böngésző az operációs rendszer saját webview-jára épül, és nem tartalmaz Falkon-kódot.

**Alexandru-Ionuț Chiuță** · **Alexandru Dumbrava** (társszerző)

## Továbbiak

[Teljes dokumentáció](../cartacolibri.md) · [A Colibrì motor README-je](https://github.com/JustVugg/colibri) · [Közreműködés](../../CONTRIBUTING.md) · [📄 Illusztrált PDF-útmutató](CartaColibri-Tutorial.hu.pdf)
