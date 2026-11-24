# -*- coding: utf-8 -*-
"""Texts for the "Power features" section, the CLI command table, the SECURITY section and the extra FAQ /
troubleshooting entries (8 languages, informal "tu"). Imported by carta_docs_beginner.py and gen_carta_docs.py.
Inline markup: **bold**, `code`, [text](https://url)."""

LANGS = ["en", "ro", "fr", "it", "es", "pt", "de", "hu"]

H_POWER = {"en": "Power features", "ro": "Funcții avansate", "fr": "Fonctions avancées", "it": "Funzioni avanzate", "es": "Funciones avanzadas", "pt": "Funcionalidades avançadas", "de": "Power-Funktionen", "hu": "Haladó funkciók"}

POWER = [
{ # find
 "en": "**Find** — a live progress bar while indexing; “did you mean…?” for typos, with suggestion chips when nothing matches; duplicates listed, and a plain reason for every skipped file.",
 "ro": "**Găsește** — bară de progres în timpul indexării; „Ai vrut să spui…?” pentru greșeli de scriere, cu sugestii când nu se potrivește nimic; duplicatele listate și un motiv clar pentru fiecare fișier sărit.",
 "fr": "**Trouve** — une barre de progression pendant l’indexation ; « Tu voulais dire… ? » pour les fautes de frappe, avec des suggestions quand rien ne correspond ; les doublons listés et une raison claire pour chaque fichier ignoré.",
 "it": "**Trova** — una barra di avanzamento durante l’indicizzazione; «Intendevi…?» per i refusi, con suggerimenti quando nulla corrisponde; i duplicati elencati e un motivo chiaro per ogni file saltato.",
 "es": "**Encuentra** — una barra de progreso durante la indexación; «¿Querías decir…?» para los errores de escritura, con sugerencias cuando nada coincide; los duplicados listados y un motivo claro para cada archivo omitido.",
 "pt": "**Encontra** — uma barra de progresso durante a indexação; «Querias dizer…?» para os erros de escrita, com sugestões quando nada corresponde; os duplicados listados e um motivo claro para cada ficheiro ignorado.",
 "de": "**Finden** — ein Fortschrittsbalken beim Indizieren; „Meintest du …?“ bei Tippfehlern, mit Vorschlägen, wenn nichts passt; Duplikate aufgelistet und ein klarer Grund für jede übersprungene Datei.",
 "hu": "**Keresés** — folyamatjelző az indexelés közben; „Erre gondoltál…?” elgépeléseknél, javaslatokkal, ha semmi sem egyezik; a duplikátumok listázva, és minden kihagyott fájlhoz érthető ok."},
{ # organize
 "en": "**Organize** — tags, a star and a private note on every document; they survive re-indexing. See related documents, what you opened recently and most often, and save searches as smart collections.",
 "ro": "**Organizează** — etichete, o stea și o notă privată pe fiecare document; rămân și după reindexare. Vezi documente înrudite, ce ai deschis recent și cel mai des, și salvează căutări ca colecții inteligente.",
 "fr": "**Organise** — des étiquettes, une étoile et une note privée sur chaque document ; elles survivent à la réindexation. Vois les documents liés, ce que tu as ouvert récemment et le plus souvent, et enregistre des recherches comme collections intelligentes.",
 "it": "**Organizza** — etichette, una stella e una nota privata su ogni documento; sopravvivono alla reindicizzazione. Vedi i documenti correlati, ciò che hai aperto di recente e più spesso, e salva le ricerche come raccolte intelligenti.",
 "es": "**Organiza** — etiquetas, una estrella y una nota privada en cada documento; sobreviven a la reindexación. Mira los documentos relacionados, lo que abriste hace poco y con más frecuencia, y guarda búsquedas como colecciones inteligentes.",
 "pt": "**Organiza** — etiquetas, uma estrela e uma nota privada em cada documento; sobrevivem à reindexação. Vê os documentos relacionados, o que abriste recentemente e com mais frequência, e guarda pesquisas como coleções inteligentes.",
 "de": "**Ordnen** — Tags, ein Stern und eine private Notiz an jedem Dokument; sie überstehen das Neuindizieren. Sieh verwandte Dokumente, was du zuletzt und am häufigsten geöffnet hast, und speichere Suchen als intelligente Sammlungen.",
 "hu": "**Rendszerezés** — címkék, csillag és privát jegyzet minden dokumentumon; túlélik az újraindexelést. Nézd a kapcsolódó dokumentumokat, a nemrég és a leggyakrabban megnyitottakat, és mentsd el a kereséseket okos gyűjteményként."},
{ # tasks
 "en": "**14 tasks** — compare, timeline, glossary, outline, actions, FAQ, flashcards, quiz, translate, explain, contradictions, plus summarize, extract and synthesize. Tick **verify citations** to flag sentences the sources do not back, and pick the answer language.",
 "ro": "**14 taskuri** — compară, cronologie, glosar, schiță, acțiuni, FAQ, fișe de studiu, test, tradu, explică, contradicții, plus rezumă, extrage și sintetizează. Bifează **verifică citările** ca să fie marcate frazele pe care sursele nu le susțin și alege limba răspunsului.",
 "fr": "**14 tâches** — comparer, chronologie, glossaire, plan, actions, FAQ, fiches, quiz, traduire, expliquer, contradictions, plus résumer, extraire et synthétiser. Coche **vérifier les citations** pour repérer les phrases que les sources n’appuient pas, et choisis la langue de la réponse.",
 "it": "**14 compiti** — confronta, cronologia, glossario, scaletta, azioni, FAQ, flashcard, quiz, traduci, spiega, contraddizioni, più riassumi, estrai e sintetizza. Spunta **verifica le citazioni** per segnalare le frasi che le fonti non sostengono e scegli la lingua della risposta.",
 "es": "**14 tareas** — comparar, cronología, glosario, esquema, acciones, FAQ, tarjetas, cuestionario, traducir, explicar, contradicciones, además de resumir, extraer y sintetizar. Marca **verificar citas** para señalar las frases que las fuentes no respaldan y elige el idioma de la respuesta.",
 "pt": "**14 tarefas** — comparar, cronologia, glossário, esquema, ações, FAQ, cartões, questionário, traduzir, explicar, contradições, além de resumir, extrair e sintetizar. Assinala **verificar citações** para marcar as frases que as fontes não sustentam e escolhe a língua da resposta.",
 "de": "**14 Aufgaben** — vergleichen, Zeitleiste, Glossar, Gliederung, Aufgabenliste, FAQ, Lernkarten, Quiz, übersetzen, erklären, Widersprüche, dazu zusammenfassen, extrahieren und synthetisieren. Hake **Zitate prüfen** an, um Sätze zu markieren, die die Quellen nicht stützen, und wähle die Antwortsprache.",
 "hu": "**14 feladat** — összehasonlítás, idővonal, szószedet, vázlat, teendők, GYIK, kártyák, kvíz, fordítás, magyarázat, ellentmondások, plusz összefoglalás, kinyerés és szintézis. Jelöld be az **idézetek ellenőrzését**, hogy megjelölje a forrásokkal alá nem támasztott mondatokat, és válaszd ki a válasz nyelvét."},
{ # chat
 "en": "**Chat** — ask follow-up questions across your whole library or one group. Answers cite their sources, or honestly say “not found in your documents” instead of guessing.",
 "ro": "**Chat** — pune întrebări succesive peste toată biblioteca sau peste un grup. Răspunsurile își citează sursele sau spun sincer „nu am găsit în documentele tale”, în loc să ghicească.",
 "fr": "**Chat** — pose des questions à la suite sur toute ta bibliothèque ou sur un groupe. Les réponses citent leurs sources, ou disent honnêtement « introuvable dans tes documents » au lieu de deviner.",
 "it": "**Chat** — fai domande una dopo l’altra su tutta la biblioteca o su un gruppo. Le risposte citano le fonti, oppure dicono onestamente «non trovato nei tuoi documenti» invece di tirare a indovinare.",
 "es": "**Chat** — haz preguntas encadenadas sobre toda tu biblioteca o sobre un grupo. Las respuestas citan sus fuentes o dicen con honestidad «no se encontró en tus documentos» en vez de adivinar.",
 "pt": "**Chat** — faz perguntas seguidas sobre toda a biblioteca ou sobre um grupo. As respostas citam as fontes ou dizem com honestidade «não encontrado nos teus documentos» em vez de adivinhar.",
 "de": "**Chat** — stelle Folgefragen zu deiner ganzen Bibliothek oder einer Gruppe. Die Antworten nennen ihre Quellen oder sagen ehrlich „in deinen Dokumenten nicht gefunden“, statt zu raten.",
 "hu": "**Chat** — tegyél fel egymásra épülő kérdéseket az egész könyvtáradról vagy egy csoportról. A válaszok megadják a forrásaikat, vagy őszintén kimondják: „nem található a dokumentumaidban”, találgatás helyett."},
{ # share + safeguard
 "en": "**Share and safeguard** — export a group, tag or search as Markdown, CSV, JSON or zip, with citations; a Stats view with charts and a timeline; a one-file Backup with checksums; Check/Repair for the index.",
 "ro": "**Partajează și protejează** — exportă un grup, o etichetă sau o căutare ca Markdown, CSV, JSON sau zip, cu citări; o vedere Statistici cu grafice și cronologie; o copie de rezervă într-un singur fișier, cu sume de control; Verifică/Repară pentru index.",
 "fr": "**Partage et protège** — exporte un groupe, une étiquette ou une recherche en Markdown, CSV, JSON ou zip, avec citations ; une vue Statistiques avec graphiques et chronologie ; une sauvegarde en un seul fichier avec sommes de contrôle ; Vérifier/Réparer pour l’index.",
 "it": "**Condividi e proteggi** — esporta un gruppo, un’etichetta o una ricerca in Markdown, CSV, JSON o zip, con citazioni; una vista Statistiche con grafici e cronologia; un backup in un solo file con checksum; Controlla/Ripara per l’indice.",
 "es": "**Comparte y protege** — exporta un grupo, una etiqueta o una búsqueda como Markdown, CSV, JSON o zip, con citas; una vista de Estadísticas con gráficos y cronología; una copia de seguridad en un solo archivo con sumas de verificación; Comprobar/Reparar para el índice.",
 "pt": "**Partilha e protege** — exporta um grupo, uma etiqueta ou uma pesquisa em Markdown, CSV, JSON ou zip, com citações; uma vista de Estatísticas com gráficos e cronologia; uma cópia de segurança num único ficheiro com somas de verificação; Verificar/Reparar para o índice.",
 "de": "**Teilen und sichern** — exportiere eine Gruppe, ein Tag oder eine Suche als Markdown, CSV, JSON oder Zip, mit Quellenangaben; eine Statistik-Ansicht mit Diagrammen und Zeitleiste; ein Backup in einer einzigen Datei mit Prüfsummen; Prüfen/Reparieren für den Index.",
 "hu": "**Megosztás és mentés** — exportálj egy csoportot, címkét vagy keresést Markdown, CSV, JSON vagy zip formátumban, hivatkozásokkal; Statisztika nézet diagramokkal és idővonallal; egyfájlos, ellenőrzőösszeges biztonsági mentés; Ellenőrzés/Javítás az indexhez."},
{ # bring things in
 "en": "**Bring things in** — import browser bookmarks, an Obsidian folder, Zotero/BibTeX, Pocket/Raindrop CSV or Evernote exports, and follow RSS/Atom feeds (OPML in and out): they become searchable notes. Email, calendar, subtitles and notebooks are indexed too.",
 "ro": "**Adu lucruri înăuntru** — importă marcaje de browser, un folder Obsidian, Zotero/BibTeX, CSV Pocket/Raindrop sau exporturi Evernote și urmărește fluxuri RSS/Atom (OPML import și export): devin notițe căutabile. Sunt indexate și e-mailurile, calendarele, subtitrările și caietele.",
 "fr": "**Fais entrer des choses** — importe les favoris du navigateur, un dossier Obsidian, Zotero/BibTeX, un CSV Pocket/Raindrop ou des exports Evernote, et suis des flux RSS/Atom (OPML en import et export) : ils deviennent des notes interrogeables. Les e-mails, agendas, sous-titres et notebooks sont aussi indexés.",
 "it": "**Porta dentro le cose** — importa i segnalibri del browser, una cartella Obsidian, Zotero/BibTeX, CSV di Pocket/Raindrop o esportazioni di Evernote, e segui i feed RSS/Atom (OPML in entrata e in uscita): diventano note ricercabili. Anche email, calendari, sottotitoli e notebook vengono indicizzati.",
 "es": "**Trae cosas** — importa los marcadores del navegador, una carpeta de Obsidian, Zotero/BibTeX, CSV de Pocket/Raindrop o exportaciones de Evernote, y sigue fuentes RSS/Atom (OPML de entrada y salida): se convierten en notas con búsqueda. También se indexan correos, calendarios, subtítulos y cuadernos.",
 "pt": "**Traz coisas** — importa os marcadores do navegador, uma pasta do Obsidian, Zotero/BibTeX, CSV do Pocket/Raindrop ou exportações do Evernote, e segue feeds RSS/Atom (OPML de entrada e saída): tornam-se notas pesquisáveis. Também são indexados e-mails, calendários, legendas e cadernos.",
 "de": "**Importieren** — hole Browser-Lesezeichen, einen Obsidian-Ordner, Zotero/BibTeX, Pocket/Raindrop-CSV oder Evernote-Exporte herein und abonniere RSS/Atom-Feeds (OPML rein und raus): Sie werden zu durchsuchbaren Notizen. E-Mails, Kalender, Untertitel und Notebooks werden ebenfalls indiziert.",
 "hu": "**Behozatal** — importálj böngészőkönyvjelzőket, Obsidian-mappát, Zotero/BibTeX-et, Pocket/Raindrop CSV-t vagy Evernote-exportot, és kövess RSS/Atom-csatornákat (OPML be- és kimenet): kereshető jegyzetekké válnak. Az e-mailek, naptárak, feliratok és notebookok is bekerülnek az indexbe."},
]

CLI_TH = {"en": ["Command", "What it does"], "ro": ["Comandă", "Ce face"], "fr": ["Commande", "Ce qu’elle fait"], "it": ["Comando", "Cosa fa"], "es": ["Comando", "Qué hace"], "pt": ["Comando", "O que faz"], "de": ["Befehl", "Was er macht"], "hu": ["Parancs", "Mit csinál"]}
H_CLI = {"en": "More commands", "ro": "Mai multe comenzi", "fr": "Plus de commandes", "it": "Altri comandi", "es": "Más comandos", "pt": "Mais comandos", "de": "Weitere Befehle", "hu": "További parancsok"}

# (command, {lang: description}); commands are never translated
CLI = [
("`task NAME [request] --group G | --tag T | --query Q [--mock] [--verify] [--lang xx] [--save md|html|txt]`", {
 "en": "the 14 tasks (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` marks sentences whose `[S#]` the text does not back",
 "ro": "cele 14 taskuri (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` marchează frazele al căror `[S#]` nu e susținut de text",
 "fr": "les 14 tâches (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize) ; `--verify` marque les phrases dont le `[S#]` n’est pas appuyé par le texte",
 "it": "i 14 compiti (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` segnala le frasi il cui `[S#]` non è sostenuto dal testo",
 "es": "las 14 tareas (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` marca las frases cuyo `[S#]` el texto no respalda",
 "pt": "as 14 tarefas (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` marca as frases cujo `[S#]` o texto não sustenta",
 "de": "die 14 Aufgaben (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); `--verify` markiert Sätze, deren `[S#]` der Text nicht stützt",
 "hu": "a 14 feladat (compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions, summarize, extract, synthesize); a `--verify` megjelöli azokat a mondatokat, amelyek `[S#]` hivatkozását a szöveg nem támasztja alá"}),
("`chat [--group G]`", {
 "en": "multi-turn questions over your library; answers are cited, or say “not found in your documents”",
 "ro": "întrebări succesive peste bibliotecă; răspunsurile sunt citate sau spun „nu am găsit în documentele tale”",
 "fr": "questions à la suite sur ta bibliothèque ; réponses citées, ou « introuvable dans tes documents »",
 "it": "domande in sequenza sulla biblioteca; risposte citate, oppure «non trovato nei tuoi documenti»",
 "es": "preguntas encadenadas sobre tu biblioteca; respuestas citadas o «no se encontró en tus documentos»",
 "pt": "perguntas seguidas sobre a biblioteca; respostas citadas, ou «não encontrado nos teus documentos»",
 "de": "Folgefragen zu deiner Bibliothek; Antworten mit Quellen oder „in deinen Dokumenten nicht gefunden“",
 "hu": "egymásra épülő kérdések a könyvtáradról; a válaszok hivatkoznak, vagy „nem található a dokumentumaidban”"}),
("`tag add/rm/list/show`, `star`, `note`", {
 "en": "your own organization, kept across re-indexing",
 "ro": "organizarea ta, păstrată și după reindexare",
 "fr": "ton organisation personnelle, conservée après réindexation",
 "it": "la tua organizzazione personale, conservata dopo la reindicizzazione",
 "es": "tu organización personal, que se conserva tras reindexar",
 "pt": "a tua organização pessoal, mantida após reindexar",
 "de": "deine eigene Ordnung, bleibt beim Neuindizieren erhalten",
 "hu": "a saját rendszerezésed, amely az újraindexelés után is megmarad"}),
("`saved add/list/run/rm`", {
 "en": "saved searches / smart collections",
 "ro": "căutări salvate / colecții inteligente",
 "fr": "recherches enregistrées / collections intelligentes",
 "it": "ricerche salvate / raccolte intelligenti",
 "es": "búsquedas guardadas / colecciones inteligentes",
 "pt": "pesquisas guardadas / coleções inteligentes",
 "de": "gespeicherte Suchen / intelligente Sammlungen",
 "hu": "mentett keresések / okos gyűjtemények"}),
("`recent`, `related PATH`", {
 "en": "what you opened lately; similar documents",
 "ro": "ce ai deschis recent; documente similare",
 "fr": "ce que tu as ouvert récemment ; documents similaires",
 "it": "ciò che hai aperto di recente; documenti simili",
 "es": "lo que abriste hace poco; documentos parecidos",
 "pt": "o que abriste recentemente; documentos semelhantes",
 "de": "was du zuletzt geöffnet hast; ähnliche Dokumente",
 "hu": "amit nemrég megnyitottál; hasonló dokumentumok"}),
("`export --format md|csv|json|zip`", {
 "en": "share a group, tag or saved search with citations",
 "ro": "partajează un grup, o etichetă sau o căutare salvată, cu citări",
 "fr": "partage un groupe, une étiquette ou une recherche enregistrée, avec citations",
 "it": "condividi un gruppo, un’etichetta o una ricerca salvata, con citazioni",
 "es": "comparte un grupo, una etiqueta o una búsqueda guardada, con citas",
 "pt": "partilha um grupo, uma etiqueta ou uma pesquisa guardada, com citações",
 "de": "teile eine Gruppe, ein Tag oder eine gespeicherte Suche mit Quellenangaben",
 "hu": "oszd meg egy csoportot, címkét vagy mentett keresést hivatkozásokkal"}),
("`backup OUT [--index]`, `backup-check ZIP`, `restore ZIP`", {
 "en": "one-file backup with checksums; check it; restore it",
 "ro": "copie de rezervă într-un singur fișier, cu sume de control; verific-o; restaureaz-o",
 "fr": "sauvegarde en un seul fichier avec sommes de contrôle ; vérifie-la ; restaure-la",
 "it": "backup in un solo file con checksum; controllalo; ripristinalo",
 "es": "copia de seguridad en un solo archivo con sumas de verificación; compruébala; restáurala",
 "pt": "cópia de segurança num único ficheiro com somas de verificação; verifica-a; restaura-a",
 "de": "Backup in einer Datei mit Prüfsummen; prüfen; wiederherstellen",
 "hu": "egyfájlos, ellenőrzőösszeges mentés; ellenőrzése; visszaállítása"}),
("`verify [--repair]`", {
 "en": "check the index; with `--repair`, rebuild it from your files",
 "ro": "verifică indexul; cu `--repair`, îl reconstruiește din fișierele tale",
 "fr": "vérifie l’index ; avec `--repair`, le reconstruit à partir de tes fichiers",
 "it": "controlla l’indice; con `--repair` lo ricostruisce dai tuoi file",
 "es": "comprueba el índice; con `--repair` lo reconstruye a partir de tus archivos",
 "pt": "verifica o índice; com `--repair` reconstrói-o a partir dos teus ficheiros",
 "de": "prüft den Index; mit `--repair` wird er aus deinen Dateien neu aufgebaut",
 "hu": "ellenőrzi az indexet; a `--repair` a fájljaidból újraépíti"}),
("`stats`, `timeline`", {
 "en": "numbers about your library; documents by month",
 "ro": "cifre despre bibliotecă; documente pe luni",
 "fr": "chiffres sur ta bibliothèque ; documents par mois",
 "it": "numeri sulla tua biblioteca; documenti per mese",
 "es": "cifras de tu biblioteca; documentos por mes",
 "pt": "números da tua biblioteca; documentos por mês",
 "de": "Zahlen zu deiner Bibliothek; Dokumente pro Monat",
 "hu": "számok a könyvtáradról; dokumentumok havonként"}),
("`import PATH`", {
 "en": "bookmarks `.html`, Obsidian folder, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex`",
 "ro": "marcaje `.html`, folder Obsidian, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex`",
 "fr": "favoris `.html`, dossier Obsidian, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex`",
 "it": "segnalibri `.html`, cartella Obsidian, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex`",
 "es": "marcadores `.html`, carpeta de Obsidian, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex`",
 "pt": "marcadores `.html`, pasta do Obsidian, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex`",
 "de": "Lesezeichen `.html`, Obsidian-Ordner, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex`",
 "hu": "könyvjelzők `.html`, Obsidian-mappa, `.bib`, Pocket/Raindrop `.csv`, Evernote `.enex`"}),
("`feeds add/list/rm/poll`, `feeds opml-import/opml-export`", {
 "en": "RSS/Atom subscriptions saved as notes",
 "ro": "abonamente RSS/Atom salvate ca notițe",
 "fr": "abonnements RSS/Atom enregistrés comme notes",
 "it": "iscrizioni RSS/Atom salvate come note",
 "es": "suscripciones RSS/Atom guardadas como notas",
 "pt": "subscrições RSS/Atom guardadas como notas",
 "de": "RSS/Atom-Abos, als Notizen gespeichert",
 "hu": "RSS/Atom-feliratkozások, jegyzetként mentve"}),
("`webwatch`, `history`", {
 "en": "watched web pages with a diff; opt-in browsing history",
 "ro": "pagini web urmărite, cu diferențe; istoric de navigare opțional",
 "fr": "pages web surveillées avec diff ; historique de navigation optionnel",
 "it": "pagine web monitorate con differenze; cronologia di navigazione opzionale",
 "es": "páginas web vigiladas con diferencias; historial de navegación opcional",
 "pt": "páginas web vigiadas com diferenças; histórico de navegação opcional",
 "de": "beobachtete Webseiten mit Diff; optionaler Browserverlauf",
 "hu": "figyelt weboldalak különbségekkel; opcionális böngészési előzmények"}),
]

H_SEC = {"en": "Security", "ro": "Securitate", "fr": "Sécurité", "it": "Sicurezza", "es": "Seguridad", "pt": "Segurança", "de": "Sicherheit", "hu": "Biztonság"}
# {sec} and {tm} are replaced by the two links (SECURITY.md, docs/threat-model.md)
SEC = {
 "en": "Found a vulnerability? Please report it privately, as described in [{sec}]; please do not open a public issue first. How the server defends itself (loopback only, the `X-Colibri-UI` header, no private addresses in web-grab, read-only originals) is written down in the [{tm}].",
 "ro": "Ai găsit o vulnerabilitate? Raportează-o în privat, așa cum e descris în [{sec}]; te rugăm să nu deschizi mai întâi un issue public. Cum se apără serverul (doar loopback, antetul `X-Colibri-UI`, fără adrese private în web-grab, originale doar în citire) e descris în [{tm}].",
 "fr": "Tu as trouvé une faille ? Signale-la en privé, comme décrit dans [{sec}] ; n’ouvre pas d’abord un ticket public. La façon dont le serveur se défend (loopback uniquement, en-tête `X-Colibri-UI`, pas d’adresses privées dans web-grab, originaux en lecture seule) est décrite dans [{tm}].",
 "it": "Hai trovato una vulnerabilità? Segnalala in privato, come descritto in [{sec}]; per favore non aprire prima una issue pubblica. Come si difende il server (solo loopback, intestazione `X-Colibri-UI`, nessun indirizzo privato in web-grab, originali in sola lettura) è descritto in [{tm}].",
 "es": "¿Has encontrado una vulnerabilidad? Infórmala en privado, como se describe en [{sec}]; por favor, no abras antes una incidencia pública. Cómo se defiende el servidor (solo loopback, cabecera `X-Colibri-UI`, sin direcciones privadas en web-grab, originales de solo lectura) está descrito en [{tm}].",
 "pt": "Encontraste uma vulnerabilidade? Comunica-a em privado, como descrito em [{sec}]; por favor, não abras primeiro uma issue pública. A forma como o servidor se defende (só loopback, cabeçalho `X-Colibri-UI`, sem endereços privados no web-grab, originais só de leitura) está descrita em [{tm}].",
 "de": "Du hast eine Sicherheitslücke gefunden? Melde sie bitte privat, wie in [{sec}] beschrieben; eröffne bitte nicht zuerst ein öffentliches Issue. Wie sich der Server schützt (nur Loopback, Header `X-Colibri-UI`, keine privaten Adressen in web-grab, Originale nur lesend), steht im [{tm}].",
 "hu": "Sebezhetőséget találtál? Jelezd privátban, ahogy a [{sec}] leírja; kérlek, ne nyiss előbb nyilvános hibajegyet. Azt, hogyan védekezik a szerver (csak loopback, `X-Colibri-UI` fejléc, a web-grab nem megy privát címekre, az eredetik csak olvashatók), a [{tm}] írja le."}
SEC_NAMES = {"sec": "SECURITY.md", "tm": "docs/threat-model.md"}

FAQ = {
"en": [
 ("How do tags, stars and notes survive re-indexing?", "They are not stored in the search index. They live in a small file of their own, `organize.json`, in your `<home>` folder, tied to each document’s path. So **Update index**, a full re-index and even **Check/Repair** (which rebuilds the index) leave them untouched. If you rename or move a file, it counts as a new document, so the tags stay with the old path."),
 ("How do I back up and restore?", "Open **Stats** and press **Download a backup** (tick the box to include the index), or run `cartacolibri backup my.zip`. It is a single zip with checksums, holding your tags, notes, groups, saved searches and imported notes, never your original documents. `cartacolibri backup-check my.zip` tests it, and `cartacolibri restore my.zip` puts everything back. Keep the zip somewhere other than this computer."),
 ("How do I import bookmarks, Obsidian or Zotero?", "Click **Bring things in** in the Library, or run `cartacolibri import PATH`. It understands a browser bookmarks export (`.html`), an Obsidian vault folder, a Zotero/BibTeX `.bib` file, Pocket or Raindrop `.csv` files and Evernote `.enex` exports. Each item becomes a small note in your library that you can search, tag and cite. Your originals are only read."),
 ("How do feeds work?", "In **Bring things in → News feeds**, paste an RSS or Atom address and press **Subscribe**. When you press **Check now** (or run `cartacolibri feeds poll`), new items are saved as notes and indexed, so you can search and ask about them. Nothing runs in the background by itself. You can bring your subscriptions in and out as OPML."),
 ("What does “verify citations” do?", "After the AI answers, CartaColibri re-reads every sentence. If a sentence has no `[S#]` citation, or the cited source does not actually contain what the sentence says, it is marked `[unsupported]`. It is a quick safety net, not a guarantee: still open the source for anything important."),
 ("Why was a web page refused?", "web-grab only fetches public pages. Addresses on your own machine or private network (`localhost`, `192.168.x.x`, `10.x.x.x` and the like) are refused, so a web page can never trick it into reading your internal services. If you really want that, for example for your own intranet, start CartaColibri with `CARTACOLIBRI_GRAB_LOCAL=1`."),
],
"ro": [
 ("Cum rămân etichetele, stelele și notele după reindexare?", "Nu sunt păstrate în indexul de căutare, ci într-un mic fișier separat, `organize.json`, în folderul tău `<home>`, legate de calea fiecărui document. Deci **Actualizează indexul**, o reindexare completă și chiar **Verifică/Repară** (care reface indexul) nu le ating. Dacă redenumești sau muți un fișier, el contează ca document nou, iar etichetele rămân la vechea cale."),
 ("Cum fac o copie de rezervă și cum o restaurez?", "Deschide **Statistici** și apasă **Descarcă copia de rezervă** (bifează căsuța ca să incluzi și indexul), sau rulează `cartacolibri backup my.zip`. Este un singur zip cu sume de control, cu etichetele, notele, grupurile, căutările salvate și notițele importate, niciodată cu documentele tale originale. `cartacolibri backup-check my.zip` îl verifică, iar `cartacolibri restore my.zip` pune totul la loc. Păstrează zip-ul în altă parte decât pe acest calculator."),
 ("Cum import marcaje, Obsidian sau Zotero?", "Apasă **Adu lucruri** în Bibliotecă sau rulează `cartacolibri import CALEA`. Înțelege un export de marcaje din browser (`.html`), un folder Obsidian, un fișier Zotero/BibTeX `.bib`, fișiere `.csv` Pocket sau Raindrop și exporturi Evernote `.enex`. Fiecare element devine o mică notiță în biblioteca ta, pe care o poți căuta, eticheta și cita. Originalele sunt doar citite."),
 ("Cum funcționează fluxurile (feeds)?", "În **Adu lucruri → Fluxuri**, lipește o adresă RSS sau Atom și apasă **Adaugă**. Când apeși **Verifică acum** (sau rulezi `cartacolibri feeds poll`), elementele noi sunt salvate ca notițe și indexate, deci le poți căuta și întreba despre ele. Nimic nu rulează singur în fundal. Abonamentele le poți aduce și scoate ca OPML."),
 ("Ce face „verifică citările”?", "După ce răspunde AI-ul, CartaColibri recitește fiecare frază. Dacă o frază nu are citare `[S#]`, sau sursa citată nu conține de fapt ce spune fraza, este marcată `[unsupported]`. E o plasă de siguranță rapidă, nu o garanție: deschide sursa pentru tot ce e important."),
 ("De ce a fost refuzată o pagină web?", "web-grab aduce doar pagini publice. Adresele de pe propriul calculator sau din rețeaua privată (`localhost`, `192.168.x.x`, `10.x.x.x` și altele) sunt refuzate, ca nicio pagină web să nu o poată păcăli să-ți citească serviciile interne. Dacă chiar vrei asta, de exemplu pentru propriul intranet, pornește CartaColibri cu `CARTACOLIBRI_GRAB_LOCAL=1`."),
],
"fr": [
 ("Comment les étiquettes, étoiles et notes survivent-elles à la réindexation ?", "Elles ne sont pas stockées dans l’index de recherche mais dans un petit fichier à part, `organize.json`, dans ton dossier `<home>`, lié au chemin de chaque document. **Mettre à jour l’index**, une réindexation complète et même **Vérifier/Réparer** (qui reconstruit l’index) n’y touchent pas. Si tu renommes ou déplaces un fichier, il compte comme un nouveau document : les étiquettes restent sur l’ancien chemin."),
 ("Comment sauvegarder et restaurer ?", "Ouvre **Statistiques** et clique sur **Télécharger la sauvegarde** (coche la case pour inclure l’index), ou lance `cartacolibri backup my.zip`. C’est un seul zip avec sommes de contrôle, qui contient tes étiquettes, notes, groupes, recherches enregistrées et notes importées, jamais tes documents originaux. `cartacolibri backup-check my.zip` le teste et `cartacolibri restore my.zip` remet tout en place. Garde le zip ailleurs que sur cet ordinateur."),
 ("Comment importer des favoris, Obsidian ou Zotero ?", "Clique sur **Importer** dans la Bibliothèque, ou lance `cartacolibri import CHEMIN`. Il comprend un export de favoris du navigateur (`.html`), un dossier Obsidian, un fichier Zotero/BibTeX `.bib`, des fichiers `.csv` Pocket ou Raindrop et des exports Evernote `.enex`. Chaque élément devient une petite note de ta bibliothèque, que tu peux chercher, étiqueter et citer. Tes originaux sont seulement lus."),
 ("Comment fonctionnent les flux ?", "Dans **Importer → Flux**, colle une adresse RSS ou Atom et clique sur **Ajouter**. Quand tu cliques sur **Vérifier maintenant** (ou lances `cartacolibri feeds poll`), les nouveaux éléments sont enregistrés comme notes et indexés : tu peux les chercher et les interroger. Rien ne tourne tout seul en arrière-plan. Tu peux importer et exporter tes abonnements en OPML."),
 ("Que fait « vérifier les citations » ?", "Après la réponse de l’IA, CartaColibri relit chaque phrase. Si une phrase n’a pas de citation `[S#]`, ou si la source citée ne contient pas vraiment ce que dit la phrase, elle est marquée `[unsupported]`. C’est un filet de sécurité rapide, pas une garantie : ouvre la source pour tout ce qui compte."),
 ("Pourquoi une page web a-t-elle été refusée ?", "web-grab ne récupère que des pages publiques. Les adresses de ta propre machine ou d’un réseau privé (`localhost`, `192.168.x.x`, `10.x.x.x`, etc.) sont refusées, afin qu’une page web ne puisse jamais le piéger pour lire tes services internes. Si tu le veux vraiment, par exemple pour ton propre intranet, lance CartaColibri avec `CARTACOLIBRI_GRAB_LOCAL=1`."),
],
"it": [
 ("Come sopravvivono etichette, stelle e note alla reindicizzazione?", "Non sono salvate nell’indice di ricerca, ma in un piccolo file a parte, `organize.json`, nella tua cartella `<home>`, legato al percorso di ogni documento. Quindi **Aggiorna indice**, una reindicizzazione completa e perfino **Controlla/Ripara** (che ricostruisce l’indice) non le toccano. Se rinomini o sposti un file, conta come un nuovo documento: le etichette restano sul vecchio percorso."),
 ("Come faccio il backup e il ripristino?", "Apri **Statistiche** e premi **Scarica backup** (spunta la casella per includere l’indice), oppure esegui `cartacolibri backup my.zip`. È un unico zip con checksum, che contiene etichette, note, gruppi, ricerche salvate e note importate, mai i tuoi documenti originali. `cartacolibri backup-check my.zip` lo verifica e `cartacolibri restore my.zip` rimette tutto a posto. Tieni lo zip lontano da questo computer."),
 ("Come importo segnalibri, Obsidian o Zotero?", "Clicca **Porta dentro** nella Biblioteca, oppure esegui `cartacolibri import PERCORSO`. Capisce un’esportazione di segnalibri del browser (`.html`), una cartella Obsidian, un file Zotero/BibTeX `.bib`, file `.csv` di Pocket o Raindrop ed esportazioni Evernote `.enex`. Ogni elemento diventa una piccola nota nella tua biblioteca, che puoi cercare, etichettare e citare. Gli originali vengono solo letti."),
 ("Come funzionano i feed?", "In **Porta dentro → Feed**, incolla un indirizzo RSS o Atom e premi **Aggiungi**. Quando premi **Controlla ora** (o esegui `cartacolibri feeds poll`), le novità vengono salvate come note e indicizzate, così puoi cercarle e fare domande. Niente gira da solo in background. Puoi importare ed esportare le iscrizioni in OPML."),
 ("Cosa fa «verifica le citazioni»?", "Dopo la risposta dell’IA, CartaColibri rilegge ogni frase. Se una frase non ha una citazione `[S#]`, o la fonte citata non contiene davvero ciò che dice la frase, viene contrassegnata `[unsupported]`. È una rete di sicurezza veloce, non una garanzia: apri la fonte per tutto ciò che conta."),
 ("Perché una pagina web è stata rifiutata?", "web-grab recupera solo pagine pubbliche. Gli indirizzi della tua macchina o di una rete privata (`localhost`, `192.168.x.x`, `10.x.x.x` e simili) vengono rifiutati, così nessuna pagina web può indurlo a leggere i tuoi servizi interni. Se lo vuoi davvero, per esempio per la tua intranet, avvia CartaColibri con `CARTACOLIBRI_GRAB_LOCAL=1`."),
],
"es": [
 ("¿Cómo sobreviven las etiquetas, estrellas y notas a la reindexación?", "No se guardan en el índice de búsqueda, sino en un pequeño archivo aparte, `organize.json`, en tu carpeta `<home>`, ligado a la ruta de cada documento. Así que **Actualizar índice**, una reindexación completa e incluso **Comprobar/Reparar** (que reconstruye el índice) no las tocan. Si renombras o mueves un archivo, cuenta como un documento nuevo y las etiquetas se quedan con la ruta antigua."),
 ("¿Cómo hago una copia de seguridad y la restauro?", "Abre **Estadísticas** y pulsa **Descargar copia de seguridad** (marca la casilla para incluir el índice), o ejecuta `cartacolibri backup my.zip`. Es un único zip con sumas de verificación que contiene tus etiquetas, notas, grupos, búsquedas guardadas y notas importadas, nunca tus documentos originales. `cartacolibri backup-check my.zip` la comprueba y `cartacolibri restore my.zip` lo devuelve todo a su sitio. Guarda el zip en otro sitio que no sea este ordenador."),
 ("¿Cómo importo marcadores, Obsidian o Zotero?", "Haz clic en **Traer cosas** en la Biblioteca, o ejecuta `cartacolibri import RUTA`. Entiende una exportación de marcadores del navegador (`.html`), una carpeta de Obsidian, un archivo Zotero/BibTeX `.bib`, archivos `.csv` de Pocket o Raindrop y exportaciones de Evernote `.enex`. Cada elemento se convierte en una pequeña nota de tu biblioteca que puedes buscar, etiquetar y citar. Tus originales solo se leen."),
 ("¿Cómo funcionan las fuentes (feeds)?", "En **Traer cosas → Fuentes**, pega una dirección RSS o Atom y pulsa **Añadir**. Cuando pulsas **Comprobar ahora** (o ejecutas `cartacolibri feeds poll`), las novedades se guardan como notas y se indexan, así que puedes buscarlas y preguntar sobre ellas. Nada se ejecuta solo en segundo plano. Puedes traer y llevar tus suscripciones como OPML."),
 ("¿Qué hace «verificar citas»?", "Cuando la IA responde, CartaColibri relee cada frase. Si una frase no tiene cita `[S#]`, o la fuente citada no contiene en realidad lo que dice la frase, se marca como `[unsupported]`. Es una red de seguridad rápida, no una garantía: abre la fuente para todo lo importante."),
 ("¿Por qué se rechazó una página web?", "web-grab solo descarga páginas públicas. Las direcciones de tu propia máquina o de una red privada (`localhost`, `192.168.x.x`, `10.x.x.x` y similares) se rechazan, para que ninguna página web pueda engañarlo y hacerle leer tus servicios internos. Si de verdad lo quieres, por ejemplo para tu propia intranet, arranca CartaColibri con `CARTACOLIBRI_GRAB_LOCAL=1`."),
],
"pt": [
 ("Como sobrevivem as etiquetas, estrelas e notas à reindexação?", "Não ficam guardadas no índice de pesquisa, mas num pequeno ficheiro à parte, `organize.json`, na tua pasta `<home>`, ligado ao caminho de cada documento. Por isso **Atualizar índice**, uma reindexação completa e até **Verificar/Reparar** (que reconstrói o índice) não lhes tocam. Se mudares o nome ou moveres um ficheiro, conta como documento novo e as etiquetas ficam no caminho antigo."),
 ("Como faço uma cópia de segurança e a restauro?", "Abre **Estatísticas** e carrega em **Transferir cópia de segurança** (assinala a caixa para incluir o índice), ou executa `cartacolibri backup my.zip`. É um único zip com somas de verificação, com as tuas etiquetas, notas, grupos, pesquisas guardadas e notas importadas, nunca os teus documentos originais. `cartacolibri backup-check my.zip` testa-o e `cartacolibri restore my.zip` repõe tudo. Guarda o zip noutro sítio que não este computador."),
 ("Como importo marcadores, Obsidian ou Zotero?", "Clica em **Trazer coisas** na Biblioteca, ou executa `cartacolibri import CAMINHO`. Entende uma exportação de marcadores do navegador (`.html`), uma pasta do Obsidian, um ficheiro Zotero/BibTeX `.bib`, ficheiros `.csv` do Pocket ou Raindrop e exportações do Evernote `.enex`. Cada item torna-se uma pequena nota na tua biblioteca, que podes pesquisar, etiquetar e citar. Os teus originais são apenas lidos."),
 ("Como funcionam os feeds?", "Em **Trazer coisas → Feeds**, cola um endereço RSS ou Atom e carrega em **Adicionar**. Quando carregas em **Verificar agora** (ou executas `cartacolibri feeds poll`), as novidades são guardadas como notas e indexadas, por isso podes pesquisá-las e fazer perguntas sobre elas. Nada corre sozinho em segundo plano. Podes trazer e levar as tuas subscrições em OPML."),
 ("O que faz «verificar citações»?", "Depois de a IA responder, o CartaColibri relê cada frase. Se uma frase não tem citação `[S#]`, ou a fonte citada não contém realmente o que a frase diz, fica marcada `[unsupported]`. É uma rede de segurança rápida, não uma garantia: abre a fonte para tudo o que for importante."),
 ("Porque foi recusada uma página web?", "O web-grab só obtém páginas públicas. Os endereços da tua própria máquina ou de uma rede privada (`localhost`, `192.168.x.x`, `10.x.x.x` e semelhantes) são recusados, para que nenhuma página web o possa enganar e levá-lo a ler os teus serviços internos. Se mesmo assim quiseres, por exemplo para a tua intranet, arranca o CartaColibri com `CARTACOLIBRI_GRAB_LOCAL=1`."),
],
"de": [
 ("Wie überleben Tags, Sterne und Notizen das Neuindizieren?", "Sie stehen nicht im Suchindex, sondern in einer kleinen eigenen Datei, `organize.json`, in deinem `<home>`-Ordner, gebunden an den Pfad jedes Dokuments. **Index aktualisieren**, ein vollständiges Neuindizieren und sogar **Prüfen/Reparieren** (das den Index neu aufbaut) fassen sie nicht an. Benennst du eine Datei um oder verschiebst sie, zählt sie als neues Dokument; die Tags bleiben beim alten Pfad."),
 ("Wie sichere ich und stelle wieder her?", "Öffne **Statistik** und klicke auf **Backup herunterladen** (Haken setzen, um den Index mitzunehmen), oder führe `cartacolibri backup my.zip` aus. Das ist ein einziges Zip mit Prüfsummen, das deine Tags, Notizen, Gruppen, gespeicherten Suchen und importierten Notizen enthält, nie deine Originaldokumente. `cartacolibri backup-check my.zip` prüft es, `cartacolibri restore my.zip` stellt alles wieder her. Bewahre das Zip nicht nur auf diesem Rechner auf."),
 ("Wie importiere ich Lesezeichen, Obsidian oder Zotero?", "Klicke in der Bibliothek auf **Importieren** oder führe `cartacolibri import PFAD` aus. Verstanden werden ein Lesezeichen-Export des Browsers (`.html`), ein Obsidian-Ordner, eine Zotero/BibTeX-Datei `.bib`, `.csv`-Dateien von Pocket oder Raindrop und Evernote-Exporte `.enex`. Jeder Eintrag wird zu einer kleinen Notiz in deiner Bibliothek, die du durchsuchen, taggen und zitieren kannst. Deine Originale werden nur gelesen."),
 ("Wie funktionieren Feeds?", "Füge unter **Importieren → Feeds** eine RSS- oder Atom-Adresse ein und klicke auf **Hinzufügen**. Wenn du auf **Jetzt prüfen** klickst (oder `cartacolibri feeds poll` ausführst), werden neue Einträge als Notizen gespeichert und indiziert, sodass du sie durchsuchen und befragen kannst. Nichts läuft von selbst im Hintergrund. Deine Abos kannst du als OPML hinein- und herausholen."),
 ("Was macht „Zitate prüfen“?", "Nachdem die KI geantwortet hat, liest CartaColibri jeden Satz noch einmal. Hat ein Satz kein `[S#]`-Zitat oder enthält die zitierte Quelle gar nicht, was der Satz sagt, wird er mit `[unsupported]` markiert. Das ist ein schnelles Sicherheitsnetz, keine Garantie: Öffne bei allem Wichtigen die Quelle."),
 ("Warum wurde eine Webseite abgelehnt?", "web-grab ruft nur öffentliche Seiten ab. Adressen auf deinem eigenen Rechner oder in einem privaten Netz (`localhost`, `192.168.x.x`, `10.x.x.x` und Ähnliches) werden abgelehnt, damit keine Webseite es dazu bringen kann, deine internen Dienste zu lesen. Wenn du das wirklich willst, etwa für dein eigenes Intranet, starte CartaColibri mit `CARTACOLIBRI_GRAB_LOCAL=1`."),
],
"hu": [
 ("Hogyan élik túl a címkék, csillagok és jegyzetek az újraindexelést?", "Nem a keresőindexben vannak, hanem egy külön kis fájlban, az `organize.json`-ban, a `<home>` mappádban, minden dokumentum útvonalához kötve. Az **Index frissítése**, a teljes újraindexelés és még az **Ellenőrzés/Javítás** (amely újraépíti az indexet) sem nyúl hozzájuk. Ha átnevezel vagy áthelyezel egy fájlt, új dokumentumnak számít, a címkék pedig a régi útvonalnál maradnak."),
 ("Hogyan készítek mentést, és hogyan állítom vissza?", "Nyisd meg a **Statisztika** nézetet, és nyomd meg a **Mentés letöltése** gombot (jelöld be a négyzetet, ha az indexet is szeretnéd), vagy futtasd: `cartacolibri backup my.zip`. Egyetlen zip, ellenőrzőösszegekkel, a címkéiddel, jegyzeteiddel, csoportjaiddal, mentett keresésekkel és importált jegyzetekkel, az eredeti dokumentumaid soha nincsenek benne. A `cartacolibri backup-check my.zip` ellenőrzi, a `cartacolibri restore my.zip` mindent visszaállít. A zipet tartsd máshol is, ne csak ezen a gépen."),
 ("Hogyan importálok könyvjelzőket, Obsidiant vagy Zotero-t?", "Kattints a Könyvtárban a **Behozatal** gombra, vagy futtasd: `cartacolibri import ÚTVONAL`. Érti a böngésző könyvjelző-exportját (`.html`), az Obsidian-mappát, a Zotero/BibTeX `.bib` fájlt, a Pocket vagy Raindrop `.csv` fájlokat és az Evernote `.enex` exportokat. Minden elem kis jegyzet lesz a könyvtáradban, amelyben kereshetsz, címkézhetsz és hivatkozhatsz. Az eredeti fájlokat csak olvassa."),
 ("Hogyan működnek a csatornák (feedek)?", "A **Behozatal → Csatornák** résznél illessz be egy RSS- vagy Atom-címet, és nyomd meg a **Hozzáadás** gombot. Amikor a **Ellenőrzés most** gombra kattintasz (vagy lefuttatod a `cartacolibri feeds poll` parancsot), az új elemek jegyzetként mentődnek és indexelődnek, így kereshetsz bennük és kérdezhetsz róluk. A háttérben semmi nem fut magától. A feliratkozásaidat OPML-ként hozhatod be és viheted ki."),
 ("Mit csinál az „idézetek ellenőrzése”?", "Miután az MI válaszolt, a CartaColibri újra elolvassa minden mondatot. Ha egy mondatnak nincs `[S#]` hivatkozása, vagy a hivatkozott forrás valójában nem tartalmazza, amit a mondat állít, `[unsupported]` jelölést kap. Ez gyors biztonsági háló, nem garancia: ami fontos, ott nyisd meg a forrást."),
 ("Miért utasítottak el egy weboldalt?", "A web-grab csak nyilvános oldalakat tölt le. A saját gépeden vagy privát hálózaton lévő címeket (`localhost`, `192.168.x.x`, `10.x.x.x` és hasonlók) elutasítja, hogy egy weboldal ne tudja rávenni a belső szolgáltatásaid olvasására. Ha tényleg ezt szeretnéd, például a saját intranetedhez, indítsd a CartaColibrit a `CARTACOLIBRI_GRAB_LOCAL=1` beállítással."),
],
}

TROUBLE = {
"en": [
 ("The index looks broken: odd results, missing files, an error about the database", "The index file was damaged (for example by a crash or a full disk), or fell out of step with your folders.", "Open **Stats**, press **Check the index** and then **Repair the index**, or run `cartacolibri verify --repair`. It rebuilds the index from your files; your tags, notes and groups are kept."),
 ("A file was skipped because it was “too slow”", "Reading it took longer than the time limit (default 120 seconds), usually a huge or damaged PDF or a very large scan.", "Raise the limit, for example `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, then **Update index**. Or split / re-export the file. The reason is shown under “Files skipped”."),
],
"ro": [
 ("Indexul pare stricat: rezultate ciudate, fișiere lipsă, o eroare despre baza de date", "Fișierul de index a fost deteriorat (de pildă de o cădere sau un disc plin) sau nu mai este sincron cu folderele tale.", "Deschide **Statistici → Verifică / Repară** sau rulează `cartacolibri verify --repair`. Reface indexul din fișierele tale; etichetele, notele și grupurile rămân."),
 ("Un fișier a fost sărit pentru că a fost „prea lent”", "Citirea lui a depășit limita de timp (implicit 120 de secunde), de obicei un PDF uriaș sau deteriorat ori o scanare foarte mare.", "Mărește limita, de exemplu `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, apoi **Actualizează indexul**. Sau împarte / reexportă fișierul. Motivul apare la „Fișiere sărite”."),
],
"fr": [
 ("L’index semble cassé : résultats bizarres, fichiers manquants, une erreur sur la base de données", "Le fichier d’index a été endommagé (par exemple par un plantage ou un disque plein) ou n’est plus synchronisé avec tes dossiers.", "Ouvre **Statistiques → Vérifier / Réparer**, ou lance `cartacolibri verify --repair`. L’index est reconstruit à partir de tes fichiers ; tes étiquettes, notes et groupes sont conservés."),
 ("Un fichier a été ignoré car « trop lent »", "Sa lecture a dépassé la limite de temps (120 secondes par défaut), en général un PDF énorme ou abîmé, ou un très gros scan.", "Augmente la limite, par exemple `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, puis **Mettre à jour l’index**. Ou découpe / réexporte le fichier. La raison s’affiche sous « Fichiers ignorés »."),
],
"it": [
 ("L’indice sembra rotto: risultati strani, file mancanti, un errore sul database", "Il file dell’indice si è danneggiato (per esempio per un arresto anomalo o un disco pieno) o non è più allineato alle tue cartelle.", "Apri **Statistiche → Controlla / Ripara**, oppure esegui `cartacolibri verify --repair`. L’indice viene ricostruito dai tuoi file; etichette, note e gruppi restano."),
 ("Un file è stato saltato perché «troppo lento»", "La lettura ha superato il limite di tempo (predefinito 120 secondi), di solito un PDF enorme o danneggiato, o una scansione molto grande.", "Alza il limite, per esempio `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, poi **Aggiorna indice**. Oppure dividi / riesporta il file. Il motivo compare sotto «File saltati»."),
],
"es": [
 ("El índice parece roto: resultados raros, archivos que faltan, un error sobre la base de datos", "El archivo del índice se dañó (por ejemplo por un fallo o un disco lleno) o se desincronizó de tus carpetas.", "Abre **Estadísticas → Comprobar / Reparar**, o ejecuta `cartacolibri verify --repair`. Reconstruye el índice a partir de tus archivos; tus etiquetas, notas y grupos se conservan."),
 ("Se omitió un archivo por ser «demasiado lento»", "Leerlo superó el límite de tiempo (120 segundos por defecto), normalmente un PDF enorme o dañado, o un escaneo muy grande.", "Sube el límite, por ejemplo `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, y luego **Actualizar índice**. O divide / vuelve a exportar el archivo. El motivo aparece en «Archivos omitidos»."),
],
"pt": [
 ("O índice parece estragado: resultados estranhos, ficheiros em falta, um erro sobre a base de dados", "O ficheiro do índice ficou danificado (por exemplo por uma falha ou um disco cheio) ou perdeu a sincronia com as tuas pastas.", "Abre **Estatísticas → Verificar / Reparar**, ou executa `cartacolibri verify --repair`. Reconstrói o índice a partir dos teus ficheiros; as etiquetas, notas e grupos mantêm-se."),
 ("Um ficheiro foi ignorado por ser «demasiado lento»", "A leitura excedeu o limite de tempo (120 segundos por omissão), normalmente um PDF enorme ou danificado, ou uma digitalização muito grande.", "Aumenta o limite, por exemplo `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, e depois **Atualizar índice**. Ou divide / volta a exportar o ficheiro. O motivo aparece em «Ficheiros ignorados»."),
],
"de": [
 ("Der Index wirkt kaputt: seltsame Treffer, fehlende Dateien, ein Fehler zur Datenbank", "Die Indexdatei wurde beschädigt (etwa durch einen Absturz oder eine volle Festplatte) oder ist nicht mehr im Takt mit deinen Ordnern.", "Öffne **Statistik → Prüfen / Reparieren** oder führe `cartacolibri verify --repair` aus. Der Index wird aus deinen Dateien neu aufgebaut; Tags, Notizen und Gruppen bleiben erhalten."),
 ("Eine Datei wurde übersprungen, weil sie „zu langsam“ war", "Das Lesen dauerte länger als das Zeitlimit (standardmäßig 120 Sekunden), meist bei einem riesigen oder beschädigten PDF oder einem sehr großen Scan.", "Erhöhe das Limit, zum Beispiel `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, und klicke dann auf **Index aktualisieren**. Oder teile / exportiere die Datei neu. Der Grund steht unter „Übersprungene Dateien“."),
],
"hu": [
 ("Az index hibásnak tűnik: furcsa találatok, hiányzó fájlok, adatbázis-hiba", "Az indexfájl megsérült (például összeomlás vagy tele lemez miatt), vagy elcsúszott a mappáidhoz képest.", "Nyisd meg a **Statisztika → Ellenőrzés / Javítás** részt, vagy futtasd: `cartacolibri verify --repair`. A fájljaidból újraépíti az indexet; a címkéid, jegyzeteid és csoportjaid megmaradnak."),
 ("Egy fájlt kihagyott, mert „túl lassú” volt", "Az olvasása átlépte az időkorlátot (alapból 120 másodperc), általában egy hatalmas vagy sérült PDF, esetleg egy nagyon nagy szkennelés.", "Emeld meg a korlátot, például `CARTACOLIBRI_EXTRACT_TIMEOUT=600`, majd **Index frissítése**. Vagy darabold fel / exportáld újra a fájlt. Az ok a „Kihagyott fájlok” alatt látszik."),
],
}

STATUS_EXTRA = {
 "en": "Portable downloads for Linux, Windows and macOS are on the GitHub **Releases** page (marked pre-release).",
 "ro": "Descărcările portabile pentru Linux, Windows și macOS sunt pe pagina **Releases** de pe GitHub (marcate pre-lansare).",
 "fr": "Les téléchargements portables pour Linux, Windows et macOS sont sur la page **Releases** de GitHub (marqués préversion).",
 "it": "I download portatili per Linux, Windows e macOS sono nella pagina **Releases** di GitHub (contrassegnati come pre-release).",
 "es": "Las descargas portátiles para Linux, Windows y macOS están en la página **Releases** de GitHub (marcadas como versión preliminar).",
 "pt": "As transferências portáteis para Linux, Windows e macOS estão na página **Releases** do GitHub (marcadas como pré-versão).",
 "de": "Die portablen Downloads für Linux, Windows und macOS findest du auf der GitHub-Seite **Releases** (als Pre-Release markiert).",
 "hu": "A hordozható letöltések Linuxra, Windowsra és macOS-re a GitHub **Releases** oldalán vannak (előzetes kiadásként jelölve).",
}
