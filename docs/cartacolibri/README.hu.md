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
- **web-grab** — egy kis Rust böngésző (clean-room, a Falkon helyett) lapokkal, amely ismeri a könyvtáradat: jelvény, ha az oldal már mentve van (és ha megváltozott), kapcsolódó dokumentumok böngészés közben, mentés egy kattintással, kérdések a helyi Colibrìhez egy oldalról — vagy az összes nyitott lapról, hivatkozásokkal. Egy parancs és egy bookmarklet a többi böngészőt fedi le.

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
Szöveget és jegyzeteket (txt, md, rst, log, csv, json, yaml, html, rtf), PDF-et, Word-, OpenDocument-, PowerPoint- és Excel-fájlokat (docx, odt, pptx, xlsx), epubot, bekapcsolt OCR mellett pedig képeket és beszkennelt PDF-eket.

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

A CartaColibri **alfa** állapotú szoftver (0.2.0-alpha verzió): működik, de számíts apróbb hibákra, és a fontos dolgaidról tarts mentést. A Linux a legjobban tesztelt platform: ott a web-grab böngészőt a CI automatikusan kipróbálja. **Windowson és macOS-en** a böngészőt a CI lefordítja, de **kézzel még soha nem futtattuk**. A GitHubon az automatikus ellenőrzések (CI) **még nincsenek megerősítve**: a feladatok elakadtak a sorban. A dokumentumaid így is biztonságban vannak, mert csak olvassa őket.

## Licenc és köszönet

A CartaColibri **MIT licenc** alatt jelenik meg. Clean-room Rust implementáció: a [Colibrì](https://github.com/JustVugg/colibri) motorral (JustVugg, Apache-2.0) kizárólag a nyilvános HTTP-átjárón át beszél, és nem tartalmaz belőle kódot. Az index- és keresőréteg a DocFetcher nyilvános kézikönyvében leírt viselkedést követi; DocFetcher-kód nincs benne. A web-grab böngésző az operációs rendszer saját webview-jára épül, és nem tartalmaz Falkon-kódot.

## Továbbiak

[Teljes dokumentáció](../cartacolibri.md) · [A Colibrì motor README-je](https://github.com/JustVugg/colibri) · [Közreműködés](../../CONTRIBUTING.md) · [📄 Illusztrált PDF-útmutató](CartaColibri-Tutorial.hu.pdf)
