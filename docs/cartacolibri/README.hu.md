<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>A dokumentumaid, kereshetően és érthetően — 100 %-ban helyben.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · <b>Magyar</b></p>

A CartaColibri egy helyi MI-könyvtár a Colibrì motorra építve. Indexeli a kiválasztott mappáidat, villámgyorsan kereshetsz bennük, majd egy helyi nyelvi modellel összefoglaltathatsz, adatokat nyerethetsz ki vagy szintetizálhatsz fájlcsoportokat — mindig megadva a forrásfájlt. Semmi sem hagyja el a gépedet, az eredeti fájlokat pedig soha nem módosítja.

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

## Licenc és köszönet

A CartaColibri **MIT licenc** alatt jelenik meg. Clean-room Rust implementáció: a [Colibrì](https://github.com/JustVugg/colibri) motorral (JustVugg, Apache-2.0) kizárólag a nyilvános HTTP-átjárón át beszél, és nem tartalmaz belőle kódot. Az index- és keresőréteg a DocFetcher nyilvános kézikönyvében leírt viselkedést követi; DocFetcher-kód nincs benne. A web-grab böngésző az operációs rendszer saját webview-jára épül, és nem tartalmaz Falkon-kódot.

## Továbbiak

[Teljes dokumentáció](../cartacolibri.md) · [A Colibrì motor README-je](https://github.com/JustVugg/colibri) · [Közreműködés](../../CONTRIBUTING.md) · [📄 Illusztrált PDF-útmutató](CartaColibri-Tutorial.hu.pdf)
