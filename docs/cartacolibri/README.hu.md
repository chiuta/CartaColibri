<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>A dokumentumaid, kereshetően és érthetően — 100 %-ban helyben.</b></p>

<p align="center">[English](README.en.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · <b>Magyar</b></p>

A CartaColibri egy helyi MI-könyvtár a Colibrì motorra építve. Indexeli a kiválasztott mappáidat, villámgyorsan kereshetsz bennük, majd egy helyi nyelvi modellel összefoglaltathatsz, adatokat nyerethetsz ki vagy szintetizálhatsz fájlcsoportokat — mindig megadva a forrásfájlt. Semmi sem hagyja el a gépedet, az eredeti fájlokat pedig soha nem módosítja.

## Mit tud

- **Könyvtár** — az összes helyi dokumentum a helyével együtt; mappák hozzáadása és eltávolítása, fájlok feltöltése, megtekintése, kérésre OCR-szöveg mentése.
- **RAG-fetch** — DocFetcher-stílusú inkrementális teljes szövegű index (szavak, „kifejezések”, előtag*, AND/OR/NOT, szűrés típus, méret, hely szerint) és újrafelhasználható fájl**csoportok**.
- **RAG-CARTA** — három helyi feladat egy csoporton: **összefoglalás**, **kinyerés** (táblázat vagy JSON, találgatás helyett „NOT FOUND”), **szintézis** (témák, közös pontok, ellentmondások). Minden válasz idézi az [S1], [S2]… jelöléseket és felsorolja a forrásokat.
- **Alapból privát** — nincs felhő, nincs telemetria; az eredetiket csak olvassa. 8 nyelv: EN, RO, FR, IT, ES, PT, DE, HU.

## Gyors kezdés

Töltsd le a kódot, és indítsd a motort a RAM-odba férő modellel (lásd a Colibrì dokumentációt), majd nyisd meg a CartaColibri oldalt:

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
./start-here.sh            # Windows: START-HERE.bat
coli web --model ~/colibri-models/<model>   # then open http://127.0.0.1:8790/carta.html
```

Vagy használd a parancssort; kipróbálni modell nélkül is lehet (`--mock` offline tartalék mód):

```bash
coli carta add ~/Documents
coli carta search "contract*" --ext pdf,docx
coli carta group set Contracts ~/Documents/contracts
coli carta synthesize "payment terms" --group Contracts --save --mock
```

## Hogyan áll össze

0. réteg — a mappáid (csak olvasás) → RAG-fetch — index, keresés, csoportok → RAG-CARTA — MI-feladatok a Colibrì motoron át.

## Licenc és köszönet

A CartaColibri a JustVugg [Colibrì](https://github.com/JustVugg/colibri) projektjének forkja, és megtartja annak Apache-2.0 licencét. Az index- és keresőréteg clean-room tervezésű, a DocFetcher nyilvános kézikönyvében leírt viselkedés ihlette; DocFetcher-kódot nem használ.

## Továbbiak

[Teljes dokumentáció](../cartacolibri.md) · [A Colibrì motor README-je](../../README.colibri.md) · [Közreműködés](../../CONTRIBUTING-CARTA.md) · [📄 Illusztrált PDF-útmutató](CartaColibri-Tutorial.hu.pdf)
