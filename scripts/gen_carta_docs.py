#!/usr/bin/env python3
"""Generates CartaColibri's per-language READMEs (docs/cartacolibri/README.<lang>.md), the root README.md
and the presentation site (site-carta/index.html) from ONE text table, so the 8 languages never drift.
Languages: en ro fr it es pt de hu — informal address ("tu") everywhere.   Usage: python3 scripts/gen_carta_docs.py"""
import html, os, sys
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LANGS = ["en", "ro", "fr", "it", "es", "pt", "de", "hu"]
NAMES = {"en": "English", "ro": "Română", "fr": "Français", "it": "Italiano", "es": "Español", "pt": "Português", "de": "Deutsch", "hu": "Magyar"}
REPO = "https://github.com/chiuta/CartaColibri"

X = {  # key -> {lang: text}
"tagline": {
 "en": "Your documents, searchable and explained — 100 % local.",
 "ro": "Documentele tale, căutabile și explicate — 100 % local.",
 "fr": "Tes documents, interrogeables et expliqués — 100 % en local.",
 "it": "I tuoi documenti, ricercabili e spiegati — 100 % in locale.",
 "es": "Tus documentos, buscables y explicados — 100 % en local.",
 "pt": "Os teus documentos, pesquisáveis e explicados — 100 % local.",
 "de": "Deine Dokumente, durchsuchbar und erklärt — 100 % lokal.",
 "hu": "A dokumentumaid, kereshetően és érthetően — 100 %-ban helyben."},
"intro": {
 "en": "CartaColibri is a local AI library built on the Colibrì engine. It indexes the folders you choose, lets you search them in a flash, and then asks a local language model to summarize, extract data from, or synthesize groups of your files — always citing the source file. Nothing leaves your computer, and your originals are never modified.",
 "ro": "CartaColibri este o bibliotecă AI locală construită pe motorul Colibrì. Indexează folderele alese de tine, le poți căuta într-o clipă, apoi ceri unui model de limbaj local să rezume, să extragă date sau să sintetizeze grupuri de fișiere — mereu cu citarea fișierului-sursă. Nimic nu pleacă de pe calculatorul tău, iar originalele nu sunt niciodată modificate.",
 "fr": "CartaColibri est une bibliothèque IA locale construite sur le moteur Colibrì. Elle indexe les dossiers que tu choisis, te laisse les interroger en un éclair, puis demande à un modèle de langage local de résumer, d’extraire des données ou de synthétiser des groupes de fichiers — en citant toujours le fichier source. Rien ne quitte ton ordinateur et tes originaux ne sont jamais modifiés.",
 "it": "CartaColibri è una biblioteca IA locale costruita sul motore Colibrì. Indicizza le cartelle che scegli, ti permette di cercarvi in un lampo e poi chiede a un modello linguistico locale di riassumere, estrarre dati o sintetizzare gruppi di file — citando sempre il file sorgente. Niente lascia il tuo computer e gli originali non vengono mai modificati.",
 "es": "CartaColibri es una biblioteca de IA local construida sobre el motor Colibrì. Indexa las carpetas que eliges, te deja buscar en ellas al instante y luego pide a un modelo de lenguaje local que resuma, extraiga datos o sintetice grupos de archivos, citando siempre el archivo de origen. Nada sale de tu ordenador y tus originales nunca se modifican.",
 "pt": "O CartaColibri é uma biblioteca de IA local construída sobre o motor Colibrì. Indexa as pastas que escolheres, deixa-te pesquisá-las num instante e depois pede a um modelo de linguagem local que resuma, extraia dados ou sintetize grupos de ficheiros — citando sempre o ficheiro de origem. Nada sai do teu computador e os teus originais nunca são modificados.",
 "de": "CartaColibri ist eine lokale KI-Bibliothek auf Basis der Colibrì-Engine. Sie indiziert die Ordner, die du auswählst, lässt dich blitzschnell darin suchen und bittet dann ein lokales Sprachmodell, Dateigruppen zusammenzufassen, Daten daraus zu extrahieren oder sie zu synthetisieren — immer mit Quellenangabe. Nichts verlässt deinen Rechner, und deine Originale werden nie verändert.",
 "hu": "A CartaColibri egy helyi MI-könyvtár a Colibrì motorra építve. Indexeli a kiválasztott mappáidat, villámgyorsan kereshetsz bennük, majd egy helyi nyelvi modellel összefoglaltathatsz, adatokat nyerethetsz ki vagy szintetizálhatsz fájlcsoportokat — mindig megadva a forrásfájlt. Semmi sem hagyja el a gépedet, az eredeti fájlokat pedig soha nem módosítja."},
"h_what": {"en": "What it does", "ro": "Ce face", "fr": "Ce qu’elle fait", "it": "Cosa fa", "es": "Qué hace", "pt": "O que faz", "de": "Was sie kann", "hu": "Mit tud"},
"f1": {
 "en": "**Library** — every local document with its location; add or remove folders, upload files, view them, optionally save OCR text.",
 "ro": "**Biblioteca** — toate documentele locale cu locația lor; adaugi sau scoți foldere, încarci fișiere, le vezi, opțional salvezi text OCR.",
 "fr": "**Bibliothèque** — tous les documents locaux avec leur emplacement ; ajoute ou retire des dossiers, envoie des fichiers, consulte-les, enregistre du texte OCR si tu veux.",
 "it": "**Biblioteca** — tutti i documenti locali con la loro posizione; aggiungi o rimuovi cartelle, carica file, visualizzali, salva se vuoi il testo OCR.",
 "es": "**Biblioteca** — todos los documentos locales con su ubicación; añade o quita carpetas, sube archivos, míralos y, si quieres, guarda el texto OCR.",
 "pt": "**Biblioteca** — todos os documentos locais com a sua localização; adiciona ou remove pastas, envia ficheiros, vê-os e, se quiseres, guarda o texto OCR.",
 "de": "**Bibliothek** — alle lokalen Dokumente mit ihrem Speicherort; Ordner hinzufügen oder entfernen, Dateien hochladen, ansehen, auf Wunsch OCR-Text speichern.",
 "hu": "**Könyvtár** — az összes helyi dokumentum a helyével együtt; mappák hozzáadása és eltávolítása, fájlok feltöltése, megtekintése, kérésre OCR-szöveg mentése."},
"f2": {
 "en": "**RAG-fetch** — DocFetcher-style incremental full-text index (words, “phrases”, prefix*, AND/OR/NOT, filters by type, size, location) and reusable **groups** of files.",
 "ro": "**RAG-fetch** — index full-text incremental în stil DocFetcher (cuvinte, „expresii”, prefix*, AND/OR/NOT, filtre după tip, mărime, locație) și **grupuri** reutilizabile de fișiere.",
 "fr": "**RAG-fetch** — index plein texte incrémental façon DocFetcher (mots, « expressions », préfixe*, AND/OR/NOT, filtres par type, taille, emplacement) et **groupes** de fichiers réutilisables.",
 "it": "**RAG-fetch** — indice full-text incrementale in stile DocFetcher (parole, «frasi», prefisso*, AND/OR/NOT, filtri per tipo, dimensione, posizione) e **gruppi** di file riutilizzabili.",
 "es": "**RAG-fetch** — índice de texto completo incremental al estilo DocFetcher (palabras, «frases», prefijo*, AND/OR/NOT, filtros por tipo, tamaño y ubicación) y **grupos** de archivos reutilizables.",
 "pt": "**RAG-fetch** — índice de texto integral incremental ao estilo DocFetcher (palavras, «frases», prefixo*, AND/OR/NOT, filtros por tipo, tamanho e localização) e **grupos** de ficheiros reutilizáveis.",
 "de": "**RAG-fetch** — inkrementeller Volltextindex im DocFetcher-Stil (Wörter, „Phrasen“, Präfix*, AND/OR/NOT, Filter nach Typ, Größe, Ort) und wiederverwendbare **Gruppen** von Dateien.",
 "hu": "**RAG-fetch** — DocFetcher-stílusú inkrementális teljes szövegű index (szavak, „kifejezések”, előtag*, AND/OR/NOT, szűrés típus, méret, hely szerint) és újrafelhasználható fájl**csoportok**."},
"f3": {
 "en": "**RAG-CARTA** — three local tasks over a group: **summarize**, **extract** (table or JSON, “NOT FOUND” instead of guesses), **synthesize** (themes, shared points, contradictions). Every answer cites [S1], [S2]… and lists its sources.",
 "ro": "**RAG-CARTA** — trei taskuri locale pe un grup: **rezumă**, **extrage** (tabel sau JSON, „NOT FOUND” în loc de presupuneri), **sintetizează** (teme, puncte comune, contradicții). Fiecare răspuns citează [S1], [S2]… și își listează sursele.",
 "fr": "**RAG-CARTA** — trois tâches locales sur un groupe : **résumer**, **extraire** (tableau ou JSON, « NOT FOUND » plutôt que des suppositions), **synthétiser** (thèmes, points communs, contradictions). Chaque réponse cite [S1], [S2]… et liste ses sources.",
 "it": "**RAG-CARTA** — tre compiti locali su un gruppo: **riassumere**, **estrarre** (tabella o JSON, «NOT FOUND» invece di supposizioni), **sintetizzare** (temi, punti comuni, contraddizioni). Ogni risposta cita [S1], [S2]… ed elenca le fonti.",
 "es": "**RAG-CARTA** — tres tareas locales sobre un grupo: **resumir**, **extraer** (tabla o JSON, «NOT FOUND» en lugar de suposiciones) y **sintetizar** (temas, puntos comunes, contradicciones). Cada respuesta cita [S1], [S2]… y enumera sus fuentes.",
 "pt": "**RAG-CARTA** — três tarefas locais sobre um grupo: **resumir**, **extrair** (tabela ou JSON, «NOT FOUND» em vez de suposições) e **sintetizar** (temas, pontos comuns, contradições). Cada resposta cita [S1], [S2]… e lista as fontes.",
 "de": "**RAG-CARTA** — drei lokale Aufgaben auf einer Gruppe: **zusammenfassen**, **extrahieren** (Tabelle oder JSON, „NOT FOUND“ statt Vermutungen), **synthetisieren** (Themen, Gemeinsamkeiten, Widersprüche). Jede Antwort zitiert [S1], [S2] … und listet ihre Quellen.",
 "hu": "**RAG-CARTA** — három helyi feladat egy csoporton: **összefoglalás**, **kinyerés** (táblázat vagy JSON, találgatás helyett „NOT FOUND”), **szintézis** (témák, közös pontok, ellentmondások). Minden válasz idézi az [S1], [S2]… jelöléseket és felsorolja a forrásokat."},
"f4": {
 "en": "**Private by design** — no cloud, no telemetry; originals are only read. 8 languages: EN, RO, FR, IT, ES, PT, DE, HU.",
 "ro": "**Privat prin construcție** — fără cloud, fără telemetrie; originalele sunt doar citite. 8 limbi: EN, RO, FR, IT, ES, PT, DE, HU.",
 "fr": "**Privé par conception** — ni cloud ni télémétrie ; les originaux sont seulement lus. 8 langues : EN, RO, FR, IT, ES, PT, DE, HU.",
 "it": "**Privato per progetto** — niente cloud, niente telemetria; gli originali vengono solo letti. 8 lingue: EN, RO, FR, IT, ES, PT, DE, HU.",
 "es": "**Privada por diseño** — sin nube ni telemetría; los originales solo se leen. 8 idiomas: EN, RO, FR, IT, ES, PT, DE, HU.",
 "pt": "**Privada por conceção** — sem nuvem nem telemetria; os originais são apenas lidos. 8 línguas: EN, RO, FR, IT, ES, PT, DE, HU.",
 "de": "**Privat von Grund auf** — keine Cloud, keine Telemetrie; Originale werden nur gelesen. 8 Sprachen: EN, RO, FR, IT, ES, PT, DE, HU.",
 "hu": "**Alapból privát** — nincs felhő, nincs telemetria; az eredetiket csak olvassa. 8 nyelv: EN, RO, FR, IT, ES, PT, DE, HU."},
"f5": {
 "en": "**web-grab** — a small Rust browser (clean-room, replaces Falkon) with tabs that knows your library: a badge when a page is already saved (and when it has changed), related documents while you browse, one-click save, questions to your local Colibrì about a page — or across all open tabs, with citations. A command and a bookmarklet cover other browsers. Also: 🖍 highlights stored with the page, 👁 watched pages with a diff of what changed (`cartacolibri webwatch`), an HTML snapshot of each save, an opt-in searchable history (`cartacolibri history on`), 🗂📝📊 summarize or extract across tabs, and `?question` in the address bar to ask your whole library.",
 "ro": "**web-grab** — un mic browser în Rust (clean-room, înlocuiește Falkon), cu taburi, care îți cunoaște biblioteca: insignă când pagina e deja salvată (și când s-a schimbat), documente înrudite în timp ce navighezi, salvare dintr-un clic, întrebări către Colibrì local despre o pagină — sau peste toate taburile deschise, cu citări. O comandă și un bookmarklet acoperă alte browsere. În plus: 🖍 evidențieri salvate cu pagina, 👁 pagini urmărite cu diferența a ceea ce s-a schimbat (`cartacolibri webwatch`), instantaneu HTML la fiecare salvare, istoric căutabil opțional (`cartacolibri history on`), 🗂📝📊 rezumat sau extragere peste taburi și `?întrebare` în bara de adrese pentru a întreba toată biblioteca.",
 "fr": "**web-grab** — un petit navigateur en Rust (clean-room, remplace Falkon) avec onglets, qui connaît ta bibliothèque : un badge quand la page est déjà enregistrée (et quand elle a changé), des documents liés pendant que tu navigues, enregistrement en un clic, questions à ton Colibrì local sur une page — ou sur tous les onglets ouverts, avec citations. Une commande et un bookmarklet couvrent les autres navigateurs. Aussi : 🖍 surlignages enregistrés avec la page, 👁 pages surveillées avec le détail des changements (`cartacolibri webwatch`), instantané HTML à chaque sauvegarde, historique consultable optionnel (`cartacolibri history on`), 🗂📝📊 résumé ou extraction sur tous les onglets, et `?question` dans la barre d'adresse pour interroger toute ta bibliothèque.",
 "it": "**web-grab** — un piccolo browser in Rust (clean-room, sostituisce Falkon) con schede, che conosce la tua biblioteca: un badge quando la pagina è già salvata (e quando è cambiata), documenti correlati mentre navighi, salvataggio con un clic, domande al tuo Colibrì locale su una pagina — o su tutte le schede aperte, con citazioni. Un comando e un bookmarklet coprono gli altri browser. Inoltre: 🖍 evidenziazioni salvate con la pagina, 👁 pagine monitorate con le differenze (`cartacolibri webwatch`), istantanea HTML a ogni salvataggio, cronologia ricercabile opzionale (`cartacolibri history on`), 🗂📝📊 riassunto o estrazione su tutte le schede e `?domanda` nella barra degli indirizzi per interrogare tutta la biblioteca.",
 "es": "**web-grab** — un pequeño navegador en Rust (clean-room, sustituye a Falkon) con pestañas, que conoce tu biblioteca: una insignia cuando la página ya está guardada (y cuando ha cambiado), documentos relacionados mientras navegas, guardado con un clic, preguntas a tu Colibrì local sobre una página — o sobre todas las pestañas abiertas, con citas. Un comando y un bookmarklet cubren otros navegadores. Además: 🖍 resaltados guardados con la página, 👁 páginas vigiladas con el detalle de los cambios (`cartacolibri webwatch`), instantánea HTML en cada guardado, historial con búsqueda opcional (`cartacolibri history on`), 🗂📝📊 resumen o extracción entre pestañas y `?pregunta` en la barra de direcciones para consultar toda tu biblioteca.",
 "pt": "**web-grab** — um pequeno navegador em Rust (clean-room, substitui o Falkon) com separadores, que conhece a tua biblioteca: um selo quando a página já está guardada (e quando mudou), documentos relacionados enquanto navegas, guardar com um clique, perguntas ao teu Colibrì local sobre uma página — ou sobre todos os separadores abertos, com citações. Um comando e um bookmarklet cobrem outros navegadores. Também: 🖍 destaques guardados com a página, 👁 páginas vigiadas com o detalhe das alterações (`cartacolibri webwatch`), instantâneo HTML em cada gravação, histórico pesquisável opcional (`cartacolibri history on`), 🗂📝📊 resumo ou extração entre separadores e `?pergunta` na barra de endereços para interrogar toda a biblioteca.",
 "de": "**web-grab** — ein kleiner Browser in Rust (Clean-Room, ersetzt Falkon) mit Tabs, der deine Bibliothek kennt: ein Abzeichen, wenn die Seite schon gespeichert ist (und wenn sie sich geändert hat), verwandte Dokumente beim Surfen, Speichern mit einem Klick, Fragen an dein lokales Colibrì zu einer Seite — oder zu allen offenen Tabs, mit Quellenangaben. Ein Befehl und ein Bookmarklet decken andere Browser ab. Außerdem: 🖍 Markierungen, die mit der Seite gespeichert werden, 👁 beobachtete Seiten mit Diff der Änderungen (`cartacolibri webwatch`), HTML-Schnappschuss bei jedem Speichern, optional durchsuchbarer Verlauf (`cartacolibri history on`), 🗂📝📊 Zusammenfassen oder Extrahieren über alle Tabs und `?Frage` in der Adressleiste, um die ganze Bibliothek zu fragen.",
 "hu": "**web-grab** — egy kis Rust böngésző (clean-room, a Falkon helyett) lapokkal, amely ismeri a könyvtáradat: jelvény, ha az oldal már mentve van (és ha megváltozott), kapcsolódó dokumentumok böngészés közben, mentés egy kattintással, kérdések a helyi Colibrìhez egy oldalról — vagy az összes nyitott lapról, hivatkozásokkal. Egy parancs és egy bookmarklet a többi böngészőt fedi le."},
"h_start": {"en": "Quick start", "ro": "Pornire rapidă", "fr": "Démarrage rapide", "it": "Avvio rapido", "es": "Inicio rápido", "pt": "Início rápido", "de": "Schnellstart", "hu": "Gyors kezdés"},
"start1": {
 "en": "Build it (or grab the portable zip) and start the local page. For the AI tasks also run the Colibrì engine with a model that fits your RAM (see its docs):",
 "ro": "Compilează-l (sau ia zip-ul portabil) și pornește pagina locală. Pentru taskurile AI pornește și motorul Colibrì cu un model care încape în RAM-ul tău (vezi documentația lui):",
 "fr": "Compile-la (ou prends le zip portable) et lance la page locale. Pour les tâches IA, lance aussi le moteur Colibrì avec un modèle adapté à ta RAM (voir sa doc) :",
 "it": "Compilala (o prendi lo zip portatile) e avvia la pagina locale. Per i compiti IA avvia anche il motore Colibrì con un modello adatto alla tua RAM (vedi la sua documentazione):",
 "es": "Compílala (o descarga el zip portátil) y arranca la página local. Para las tareas de IA, arranca también el motor Colibrì con un modelo que quepa en tu RAM (mira su documentación):",
 "pt": "Compila-a (ou obtém o zip portátil) e arranca a página local. Para as tarefas de IA, arranca também o motor Colibrì com um modelo que caiba na tua RAM (vê a documentação dele):",
 "de": "Kompiliere sie (oder nimm das portable Zip) und starte die lokale Seite. Für die KI-Aufgaben starte außerdem die Colibrì-Engine mit einem Modell, das in deinen RAM passt (siehe deren Doku):",
 "hu": "Fordítsd le (vagy töltsd le a hordozható zipet), és indítsd el a helyi oldalt. Az MI-feladatokhoz indítsd el a Colibrì motort is a RAM-odba férő modellel (lásd a dokumentációját):"},
"start2": {
 "en": "Or use the command line, with no model needed to try it (`--mock` is an offline fallback):",
 "ro": "Sau folosește linia de comandă, fără model pentru a încerca (`--mock` e un mod offline de rezervă):",
 "fr": "Ou utilise la ligne de commande, sans modèle pour essayer (`--mock` est un repli hors ligne) :",
 "it": "Oppure usa la riga di comando, senza modello per provare (`--mock` è un ripiego offline):",
 "es": "O usa la línea de comandos, sin modelo para probar (`--mock` es un modo sin conexión):",
 "pt": "Ou usa a linha de comandos, sem modelo para experimentar (`--mock` é um modo offline):",
 "de": "Oder nutze die Kommandozeile; zum Ausprobieren braucht es kein Modell (`--mock` ist ein Offline-Fallback):",
 "hu": "Vagy használd a parancssort; kipróbálni modell nélkül is lehet (`--mock` offline tartalék mód):"},
"h_layers": {"en": "How it fits together", "ro": "Cum se leagă", "fr": "Comment ça s’articule", "it": "Come si incastra", "es": "Cómo encaja todo", "pt": "Como tudo encaixa", "de": "Wie alles zusammenspielt", "hu": "Hogyan áll össze"},
"layers": {
 "en": "Layer 0 — your folders (read-only) → RAG-fetch — index, search, groups → RAG-CARTA — AI tasks through the Colibrì engine.",
 "ro": "Stratul 0 — folderele tale (doar citire) → RAG-fetch — index, căutare, grupuri → RAG-CARTA — taskuri AI prin motorul Colibrì.",
 "fr": "Couche 0 — tes dossiers (lecture seule) → RAG-fetch — index, recherche, groupes → RAG-CARTA — tâches IA via le moteur Colibrì.",
 "it": "Livello 0 — le tue cartelle (sola lettura) → RAG-fetch — indice, ricerca, gruppi → RAG-CARTA — compiti IA tramite il motore Colibrì.",
 "es": "Capa 0 — tus carpetas (solo lectura) → RAG-fetch — índice, búsqueda, grupos → RAG-CARTA — tareas de IA con el motor Colibrì.",
 "pt": "Camada 0 — as tuas pastas (só leitura) → RAG-fetch — índice, pesquisa, grupos → RAG-CARTA — tarefas de IA através do motor Colibrì.",
 "de": "Ebene 0 — deine Ordner (nur lesend) → RAG-fetch — Index, Suche, Gruppen → RAG-CARTA — KI-Aufgaben über die Colibrì-Engine.",
 "hu": "0. réteg — a mappáid (csak olvasás) → RAG-fetch — index, keresés, csoportok → RAG-CARTA — MI-feladatok a Colibrì motoron át."},
"h_lic": {"en": "Licence and credits", "ro": "Licență și mulțumiri", "fr": "Licence et crédits", "it": "Licenza e crediti", "es": "Licencia y créditos", "pt": "Licença e créditos", "de": "Lizenz und Danksagung", "hu": "Licenc és köszönet"},
"lic": {
 "en": "CartaColibri is released under the **MIT licence**. It is a clean-room Rust implementation: it talks to the [Colibrì](https://github.com/JustVugg/colibri) engine (JustVugg, Apache-2.0) only through its public HTTP gateway and contains none of its code. The index and search layer follows the behaviour described in DocFetcher’s public manual; no DocFetcher code is used. The web-grab browser is built on the operating system’s own webview and contains no Falkon code.",
 "ro": "CartaColibri este publicat sub **licența MIT**. Este o implementare Rust clean-room: vorbește cu motorul [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) doar prin gateway-ul lui HTTP public și nu conține cod din el. Stratul de indexare și căutare urmează comportamentul descris în manualul public DocFetcher; nu se folosește cod DocFetcher. Browserul web-grab este construit pe webview-ul propriu al sistemului de operare și nu conține cod Falkon.",
 "fr": "CartaColibri est publié sous **licence MIT**. C’est une implémentation Rust clean-room : elle ne parle au moteur [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) que via sa passerelle HTTP publique et n’en contient aucun code. La couche d’index et de recherche suit le comportement décrit dans le manuel public de DocFetcher ; aucun code DocFetcher n’est utilisé. Le navigateur web-grab s’appuie sur la webview du système et ne contient aucun code de Falkon.",
 "it": "CartaColibri è rilasciato con **licenza MIT**. È un’implementazione Rust clean-room: parla con il motore [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) solo tramite il suo gateway HTTP pubblico e non ne contiene codice. Il livello di indicizzazione e ricerca segue il comportamento descritto nel manuale pubblico di DocFetcher; non si usa codice DocFetcher. Il browser web-grab si basa sulla webview del sistema operativo e non contiene codice di Falkon.",
 "es": "CartaColibri se publica bajo **licencia MIT**. Es una implementación Rust clean-room: habla con el motor [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) solo a través de su pasarela HTTP pública y no contiene nada de su código. La capa de índice y búsqueda sigue el comportamiento descrito en el manual público de DocFetcher; no se usa código de DocFetcher. El navegador web-grab usa el webview del propio sistema operativo y no contiene código de Falkon.",
 "pt": "O CartaColibri é publicado sob **licença MIT**. É uma implementação Rust clean-room: fala com o motor [Colibrì](https://github.com/JustVugg/colibri) (JustVugg, Apache-2.0) apenas através do seu gateway HTTP público e não contém código dele. A camada de índice e pesquisa segue o comportamento descrito no manual público do DocFetcher; não é usado código do DocFetcher. O navegador web-grab usa o webview do próprio sistema operativo e não contém código do Falkon.",
 "de": "CartaColibri steht unter der **MIT-Lizenz**. Es ist eine Clean-Room-Implementierung in Rust: Sie spricht mit der [Colibrì](https://github.com/JustVugg/colibri)-Engine (JustVugg, Apache-2.0) nur über deren öffentliches HTTP-Gateway und enthält keinen Code daraus. Die Index- und Suchschicht folgt dem im öffentlichen DocFetcher-Handbuch beschriebenen Verhalten; DocFetcher-Code wird nicht verwendet. Der web-grab-Browser nutzt die Webview des Betriebssystems und enthält keinen Falkon-Code.",
 "hu": "A CartaColibri **MIT licenc** alatt jelenik meg. Clean-room Rust implementáció: a [Colibrì](https://github.com/JustVugg/colibri) motorral (JustVugg, Apache-2.0) kizárólag a nyilvános HTTP-átjárón át beszél, és nem tartalmaz belőle kódot. Az index- és keresőréteg a DocFetcher nyilvános kézikönyvében leírt viselkedést követi; DocFetcher-kód nincs benne. A web-grab böngésző az operációs rendszer saját webview-jára épül, és nem tartalmaz Falkon-kódot."},
"h_docs": {"en": "More", "ro": "Mai multe", "fr": "Plus d’infos", "it": "Per saperne di più", "es": "Más información", "pt": "Mais informação", "de": "Mehr", "hu": "Továbbiak"},
"docs": {
 "en": "[Full documentation](../cartacolibri.md) · [Colibrì engine](https://github.com/JustVugg/colibri) · [Contributing](../../CONTRIBUTING.md)",
 "ro": "[Documentație completă](../cartacolibri.md) · [README motor Colibrì](https://github.com/JustVugg/colibri) · [Contribuții](../../CONTRIBUTING.md)",
 "fr": "[Documentation complète](../cartacolibri.md) · [README du moteur Colibrì](https://github.com/JustVugg/colibri) · [Contribuer](../../CONTRIBUTING.md)",
 "it": "[Documentazione completa](../cartacolibri.md) · [README del motore Colibrì](https://github.com/JustVugg/colibri) · [Contribuire](../../CONTRIBUTING.md)",
 "es": "[Documentación completa](../cartacolibri.md) · [README del motor Colibrì](https://github.com/JustVugg/colibri) · [Contribuir](../../CONTRIBUTING.md)",
 "pt": "[Documentação completa](../cartacolibri.md) · [README do motor Colibrì](https://github.com/JustVugg/colibri) · [Contribuir](../../CONTRIBUTING.md)",
 "de": "[Vollständige Dokumentation](../cartacolibri.md) · [README der Colibrì-Engine](https://github.com/JustVugg/colibri) · [Mitmachen](../../CONTRIBUTING.md)",
 "hu": "[Teljes dokumentáció](../cartacolibri.md) · [A Colibrì motor README-je](https://github.com/JustVugg/colibri) · [Közreműködés](../../CONTRIBUTING.md)"},
"dl": {"en": "Download", "ro": "Descarcă", "fr": "Télécharger", "it": "Scarica", "es": "Descargar", "pt": "Transferir", "de": "Herunterladen", "hu": "Letöltés"},
"dl_sub": {
 "en": "Portable (Windows, Linux, macOS): unzip, run the launcher, done.",
 "ro": "Portabil (Windows, Linux, macOS): dezarhivezi, rulezi lansatorul, gata.",
 "fr": "Portable (Windows, Linux, macOS) : décompresse, lance le lanceur, c’est prêt.",
 "it": "Portatile (Windows, Linux, macOS): decomprimi, avvii il lanciatore, fatto.",
 "es": "Portátil (Windows, Linux, macOS): descomprime, ejecuta el lanzador y listo.",
 "pt": "Portátil (Windows, Linux, macOS): descomprime, executa o lançador e já está.",
 "de": "Portabel (Windows, Linux, macOS): entpacken, Starter ausführen, fertig.",
 "hu": "Hordozható (Windows, Linux, macOS): kicsomagolod, elindítod, kész."},
"h_demo": {"en": "See it in action", "ro": "Vezi-o în acțiune", "fr": "Vois-la en action", "it": "Guardala in azione", "es": "Mírala en acción", "pt": "Vê-a em ação", "de": "Sieh sie in Aktion", "hu": "Nézd meg működés közben"},
"demo_sub": {
 "en": "Short demo with subtitles in 8 languages (pick them in the player's subtitle menu).",
 "ro": "Demo scurt cu subtitrări în 8 limbi (alege-le din meniul de subtitrări al playerului).",
 "fr": "Courte démo sous-titrée en 8 langues (choisis-les dans le menu des sous-titres du lecteur).",
 "it": "Breve demo con sottotitoli in 8 lingue (scegli dal menu dei sottotitoli del player).",
 "es": "Demo breve con subtítulos en 8 idiomas (elígelos en el menú de subtítulos del reproductor).",
 "pt": "Demonstração curta com legendas em 8 línguas (escolhe-as no menu de legendas do leitor).",
 "de": "Kurze Demo mit Untertiteln in 8 Sprachen (im Untertitel-Menü des Players wählbar).",
 "hu": "Rövid bemutató 8 nyelvű felirattal (a lejátszó feliratmenüjében választhatsz)."},
"tut": {"en": "Illustrated PDF tutorial", "ro": "Tutorial PDF ilustrat", "fr": "Tutoriel PDF illustré", "it": "Tutorial PDF illustrato", "es": "Tutorial PDF ilustrado", "pt": "Tutorial PDF ilustrado", "de": "Bebilderte PDF-Anleitung", "hu": "Illusztrált PDF-útmutató"},
}
START_CODE = """```bash
git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
cargo build --release                       # Rust 1.90+, or use the portable zip
./target/release/cartacolibri serve         # then open http://127.0.0.1:8765/
coli web --model ~/colibri-models/<model>   # Colibrì engine, only for the AI tasks
```"""
CLI_CODE = """```bash
cartacolibri add ~/Documents
cartacolibri search "contract*" --ext pdf,docx
cartacolibri group set Contracts ~/Documents/contracts
cartacolibri synthesize "payment terms" --group Contracts --save --mock
cartacolibri grab https://example.org/article      # web-grab (command line)
cartacolibri-browser https://example.org           # web-grab (browser)
```"""


def md_readme(lang, root=False):
    t = lambda k: X[k][lang]
    pre = "docs/cartacolibri/" if root else ""
    langbar = " · ".join(f"<b>{NAMES[l]}</b>" if l == lang else f"[{NAMES[l]}]({pre}README.{l}.md)" if not (root and l == "en") else f"[{NAMES[l]}](README.md)"
                         for l in LANGS) if not root else " · ".join("<b>English</b>" if l == "en" else f"[{NAMES[l]}](docs/cartacolibri/README.{l}.md)" for l in LANGS)
    logo = "assets/cartacolibri-logo.png" if root else "../../assets/cartacolibri-logo.png"
    t_docs = t("docs")
    if root: t_docs = t_docs.replace("](../cartacolibri.md)", "](docs/cartacolibri.md)").replace("](../../", "](")
    return f"""<p align="center"><img src="{logo}" width="320" alt="CartaColibri — Local AI Library"></p>

<p align="center"><b>{t('tagline')}</b></p>

<p align="center">{langbar}</p>

{t('intro')}

## {t('h_what')}

- {t('f1')}
- {t('f2')}
- {t('f3')}
- {t('f4')}
- {t('f5')}

## {t('h_start')}

{t('start1')}

{START_CODE}

{t('start2')}

{CLI_CODE}

## {t('h_layers')}

{t('layers')}

## {t('h_lic')}

{t('lic')}

## {t('h_docs')}

{t_docs} · [📄 {t('tut')}]({'docs/cartacolibri/' if root else ''}CartaColibri-Tutorial.{lang}.pdf)
"""


def site():
    def li(md):  # **bold** -> <b>, `code` -> <code>, [text](url) -> <a>
        import re
        s = html.escape(md)
        s = re.sub(r"\*\*(.+?)\*\*", r"<b>\1</b>", s)
        s = re.sub(r"`(.+?)`", r"<code>\1</code>", s)
        return re.sub(r"\[(.+?)\]\((https?://[^)]+)\)", r'<a href="\2">\1</a>', s)
    blocks = {}
    for k in X:
        blocks[k] = {l: li(X[k][l]) for l in LANGS}
    import json
    data = json.dumps(blocks, ensure_ascii=False)
    names = json.dumps(NAMES, ensure_ascii=False)
    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>CartaColibri — Local AI Library</title><meta name="description" content="Local AI library on the Colibrì engine: index, search, summarize, extract, synthesize — 100% local.">
<link rel="icon" href="cartacolibri-mark.png"><meta property="og:image" content="cartacolibri-logo.png">
<style>
:root{{--bg:#0f1115;--s:#171a21;--line:#2a2f3a;--ink:#eef1f6;--mute:#9aa3b2;--blue:#2f66c4;--cyan:#5fd7e6;--amber:#fdb93a}}
*{{box-sizing:border-box}}body{{margin:0;background:var(--bg);color:var(--ink);font:17px/1.65 system-ui,"Segoe UI",Roboto,sans-serif}}
a{{color:var(--cyan)}}code{{background:var(--s);border:1px solid var(--line);border-radius:6px;padding:1px 6px;font-size:.9em}}
nav{{display:flex;gap:12px;align-items:center;padding:14px 5vw;border-bottom:1px solid var(--line);position:sticky;top:0;background:rgba(15,17,21,.92);backdrop-filter:blur(8px)}}
nav b{{flex:1}}select{{background:var(--s);color:var(--ink);border:1px solid var(--line);border-radius:10px;padding:6px 10px;font:inherit}}
.hero{{text-align:center;padding:64px 5vw 40px;background:radial-gradient(60% 60% at 50% 0%,rgba(47,102,196,.28),transparent)}}
.hero img{{width:min(340px,70vw)}}.hero h1{{font-size:clamp(26px,4vw,40px);margin:.4em 0 .2em}}.hero p{{max-width:760px;margin:.6em auto;color:var(--mute)}}
.btn{{display:inline-block;margin:10px 6px;padding:13px 26px;border-radius:12px;font-weight:700;text-decoration:none;color:#fff;background:var(--blue)}}.btn.a{{background:var(--amber);color:#2b1d00}}
main{{max-width:980px;margin:0 auto;padding:10px 5vw 80px}}h2{{margin:2.2em 0 .6em;font-size:24px}}
.grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:16px}}.card{{background:var(--s);border:1px solid var(--line);border-radius:16px;padding:18px}}
.flow{{display:flex;flex-wrap:wrap;gap:10px;align-items:center;justify-content:center;margin:24px 0}}.flow span{{padding:12px 18px;border-radius:12px;border:1px solid var(--line);background:var(--s);font-weight:600}}.flow i{{color:var(--cyan);font-style:normal}}
pre{{background:var(--s);border:1px solid var(--line);border-radius:12px;padding:14px;overflow:auto;font-size:14px}}
footer{{border-top:1px solid var(--line);padding:26px 5vw;color:var(--mute);text-align:center;font-size:14px}}
</style></head><body>
<nav><b>CartaColibri</b><a href="{REPO}">GitHub</a><select id="l" aria-label="Language"></select></nav>
<header class="hero"><img src="cartacolibri-logo.png" alt="CartaColibri — Local AI Library"><h1 id="tagline"></h1><p id="intro"></p>
<a class="btn a" href="{REPO}/releases" id="dl"></a><a class="btn" href="{REPO}">GitHub</a><p id="dl_sub" style="font-size:15px"></p></header>
<main><h2 id="h_demo"></h2><p id="demo_sub" style="color:var(--mute)"></p>
<video id="vid" controls preload="metadata" poster="img/en-4.png" style="width:100%;border-radius:14px;border:1px solid var(--line)"><source src="media/cartacolibri-demo.mp4" type="video/mp4">{"".join(f'<track kind="subtitles" src="media/demo.{l}.vtt" srclang="{l}" label="{NAMES[l]}">' for l in LANGS)}</video>
<div class="grid" style="margin-top:16px"><img id="g1" style="width:100%;border-radius:12px;border:1px solid var(--line)"><img id="g2" style="width:100%;border-radius:12px;border:1px solid var(--line)"><img id="g4" style="width:100%;border-radius:12px;border:1px solid var(--line)"></div>
<p style="margin-top:14px"><a id="tutpdf" class="btn" href="#"></a></p>
<h2 id="h_what"></h2><div class="grid"><div class="card" id="f1"></div><div class="card" id="f2"></div><div class="card" id="f3"></div><div class="card" id="f4"></div><div class="card" id="f5"></div></div>
<h2 id="h_layers"></h2><div class="flow"><span>📁 Layer 0</span><i>→</i><span>RAG-fetch</span><i>→</i><span>RAG-CARTA</span><i>→</i><span>Colibrì</span></div><p id="layers" style="text-align:center;color:var(--mute)"></p>
<h2 id="h_start"></h2><p id="start1"></p><pre>git clone https://github.com/chiuta/CartaColibri
cd CartaColibri
cargo build --release
./target/release/cartacolibri serve
# http://127.0.0.1:8765/</pre><p id="start2"></p><pre>cartacolibri add ~/Documents
cartacolibri search "contract*" --ext pdf,docx
cartacolibri group set Contracts ~/Documents/contracts
cartacolibri synthesize "payment terms" --group Contracts --save --mock
cartacolibri grab https://example.org/article
cartacolibri-browser https://example.org</pre>
<h2 id="h_lic"></h2><p id="lic"></p></main>
<footer>© CartaColibri · MIT · <a href="{REPO}">{REPO.split('//')[1]}</a></footer>
<script>
const D={data},N={names};let L=(()=>{{try{{const s=localStorage.getItem('cc.lang');if(s&&N[s])return s}}catch(e){{}};const n=(navigator.language||'en').slice(0,2);return N[n]?n:'en'}})();
const sel=document.getElementById('l');sel.innerHTML=Object.entries(N).map(([k,v])=>'<option value="'+k+'">'+v+'</option>').join('');
function go(){{document.documentElement.lang=L;sel.value=L;for(const k of ['tagline','intro','h_demo','demo_sub','h_what','f1','f2','f3','f4','f5','h_layers','layers','h_start','start1','start2','h_lic','lic'])document.getElementById(k).innerHTML=D[k][L];
document.getElementById('dl').textContent=D.dl[L];
const tp=document.getElementById('tutpdf');tp.textContent='📄 '+D.tut[L];tp.href='media/CartaColibri-Tutorial.'+L+'.pdf';
[['g1',1],['g2',2],['g4',4]].forEach(([id,n])=>document.getElementById(id).src='img/'+L+'-'+n+'.png');
const tr=[...document.querySelectorAll('#vid track')];tr.forEach(t=>t.track.mode=t.srclang===L?'showing':'disabled');document.getElementById('dl_sub').innerHTML=D.dl_sub[L];}}
sel.onchange=()=>{{L=sel.value;try{{localStorage.setItem('cc.lang',L)}}catch(e){{}}go()}};go();
</script></body></html>
"""


def main():
    os.makedirs(f"{ROOT}/docs/cartacolibri", exist_ok=True)
    for l in LANGS:
        open(f"{ROOT}/docs/cartacolibri/README.{l}.md", "w", encoding="utf-8").write(md_readme(l))
    open(f"{ROOT}/README.md", "w", encoding="utf-8").write(md_readme("en", root=True))
    os.makedirs(f"{ROOT}/site-carta", exist_ok=True)
    open(f"{ROOT}/site-carta/index.html", "w", encoding="utf-8").write(site())
    import shutil
    os.makedirs(f"{ROOT}/site-carta/img", exist_ok=True); os.makedirs(f"{ROOT}/site-carta/media", exist_ok=True)
    for src, dst in [(f"{ROOT}/docs/cartacolibri/img/{l}-{n}.png", f"{ROOT}/site-carta/img/{l}-{n}.png") for l in LANGS for n in (1, 2, 3, 4)] + \
                    [(f"{ROOT}/docs/cartacolibri/CartaColibri-Tutorial.{l}.pdf", f"{ROOT}/site-carta/media/CartaColibri-Tutorial.{l}.pdf") for l in LANGS]:
        if os.path.exists(src): shutil.copyfile(src, dst)
    for f in ("cartacolibri-logo.png", "cartacolibri-mark.png"):
        with open(f"{ROOT}/assets/{f}", "rb") as a, open(f"{ROOT}/site-carta/{f}", "wb") as b: b.write(a.read())
    miss = [(k, l) for k in X for l in LANGS if not X[k].get(l)]
    if miss: sys.exit(f"missing translations: {miss}")
    print("generated", len(LANGS), "READMEs, README.md and site-carta/")


if __name__ == "__main__":
    main()
