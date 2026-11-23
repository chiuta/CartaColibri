<p align="center"><img src="../../assets/cartacolibri-logo.png" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>Documentele tale, căutabile și explicate — 100 % local.</b></p>

<p align="center">[English](README.en.md) · <b>Română</b> · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)</p>

CartaColibri este o bibliotecă AI locală construită pe motorul Colibrì. Indexează folderele alese de tine, le poți căuta într-o clipă, apoi ceri unui model de limbaj local să rezume, să extragă date sau să sintetizeze grupuri de fișiere — mereu cu citarea fișierului-sursă. Nimic nu pleacă de pe calculatorul tău, iar originalele nu sunt niciodată modificate.

## Ce face

- **Biblioteca** — toate documentele locale cu locația lor; adaugi sau scoți foldere, încarci fișiere, le vezi, opțional salvezi text OCR.
- **RAG-fetch** — index full-text incremental în stil DocFetcher (cuvinte, „expresii”, prefix*, AND/OR/NOT, filtre după tip, mărime, locație) și **grupuri** reutilizabile de fișiere.
- **RAG-CARTA** — trei taskuri locale pe un grup: **rezumă**, **extrage** (tabel sau JSON, „NOT FOUND” în loc de presupuneri), **sintetizează** (teme, puncte comune, contradicții). Fiecare răspuns citează [S1], [S2]… și își listează sursele.
- **Privat prin construcție** — fără cloud, fără telemetrie; originalele sunt doar citite. 8 limbi: EN, RO, FR, IT, ES, PT, DE, HU.

## Pornire rapidă

Ia codul și pornește motorul cu un model care încape în RAM-ul tău (vezi documentația Colibrì), apoi deschide pagina CartaColibri:

```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
./start-here.sh            # Windows: START-HERE.bat
coli web --model ~/colibri-models/<model>   # then open http://127.0.0.1:8790/carta.html
```

Sau folosește linia de comandă, fără model pentru a încerca (`--mock` e un mod offline de rezervă):

```bash
coli carta add ~/Documents
coli carta search "contract*" --ext pdf,docx
coli carta group set Contracts ~/Documents/contracts
coli carta synthesize "payment terms" --group Contracts --save --mock
```

## Cum se leagă

Stratul 0 — folderele tale (doar citire) → RAG-fetch — index, căutare, grupuri → RAG-CARTA — taskuri AI prin motorul Colibrì.

## Licență și mulțumiri

CartaColibri este un fork al [Colibrì](https://github.com/JustVugg/colibri) de JustVugg și păstrează licența Apache-2.0. Stratul de indexare și căutare este un design clean-room, inspirat de comportamentul descris în manualul public DocFetcher; nu se folosește cod DocFetcher.

## Mai multe

[Documentație completă](../cartacolibri.md) · [README motor Colibrì](../../README.colibri.md) · [Contribuții](../../CONTRIBUTING-CARTA.md) · [📄 Tutorial PDF ilustrat](CartaColibri-Tutorial.ro.pdf)
