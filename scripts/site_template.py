#!/usr/bin/env python3
"""HTML template + the new (8-language) strings of the CartaColibri presentation page.
Used by gen_carta_docs.py (site()). One self-contained page: no external fonts, scripts or trackers."""

LANGS = ["en", "ro", "fr", "it", "es", "pt", "de", "hu"]

# key -> language -> text.  Informal "tu" everywhere it applies.
NEW = {
    "badge": {
        "en": "Open source · MIT · 100 % on your computer", "ro": "Open source · MIT · 100 % pe calculatorul tău",
        "fr": "Open source · MIT · 100 % sur ton ordinateur", "it": "Open source · MIT · 100 % sul tuo computer",
        "es": "Código abierto · MIT · 100 % en tu ordenador", "pt": "Código aberto · MIT · 100 % no teu computador",
        "de": "Open Source · MIT · 100 % auf deinem Rechner", "hu": "Nyílt forrás · MIT · 100 % a te gépeden"},
    "st_lang": {"en": "languages", "ro": "limbi", "fr": "langues", "it": "lingue", "es": "idiomas", "pt": "idiomas", "de": "Sprachen", "hu": "nyelv"},
    "st_tasks": {"en": "AI tasks", "ro": "sarcini AI", "fr": "tâches IA", "it": "attività IA", "es": "tareas de IA", "pt": "tarefas de IA", "de": "KI-Aufgaben", "hu": "MI-feladat"},
    "st_cloud": {"en": "bytes sent to a cloud", "ro": "octeți trimiși în cloud", "fr": "octet envoyé au cloud", "it": "byte inviati al cloud", "es": "bytes enviados a la nube",
                 "pt": "bytes enviados para a nuvem", "de": "Byte an eine Cloud", "hu": "bájt megy a felhőbe"},
    "st_local": {"en": "local", "ro": "local", "fr": "local", "it": "locale", "es": "local", "pt": "local", "de": "lokal", "hu": "helyi"},
    "n_tour": {"en": "Tour", "ro": "Tur", "fr": "Visite", "it": "Tour", "es": "Recorrido", "pt": "Visita", "de": "Tour", "hu": "Séta"},
    "tour_btn": {"en": "Take the tour", "ro": "Fă turul", "fr": "Fais la visite", "it": "Fai il tour", "es": "Haz el recorrido", "pt": "Faz a visita", "de": "Mach die Tour", "hu": "Indítsd a sétát"},
    "h_tour": {"en": "A guided tour, screen by screen", "ro": "Un tur ghidat, ecran cu ecran", "fr": "Une visite guidée, écran par écran", "it": "Un tour guidato, schermata per schermata",
               "es": "Un recorrido guiado, pantalla a pantalla", "pt": "Uma visita guiada, ecrã a ecrã", "de": "Eine geführte Tour, Bildschirm für Bildschirm", "hu": "Vezetett séta, képernyőről képernyőre"},
    "tour_sub": {"en": "Real screenshots of the real interface, in your language.", "ro": "Capturi reale ale interfeței reale, în limba ta.",
                 "fr": "De vraies captures de la vraie interface, dans ta langue.", "it": "Schermate reali dell’interfaccia reale, nella tua lingua.",
                 "es": "Capturas reales de la interfaz real, en tu idioma.", "pt": "Capturas reais da interface real, na tua língua.",
                 "de": "Echte Screenshots der echten Oberfläche, in deiner Sprache.", "hu": "Valódi képernyőképek a valódi felületről, a te nyelveden."},
    "t1": {"en": "Library", "ro": "Bibliotecă", "fr": "Bibliothèque", "it": "Libreria", "es": "Biblioteca", "pt": "Biblioteca", "de": "Bibliothek", "hu": "Könyvtár"},
    "t2": {"en": "Search", "ro": "Căutare", "fr": "Recherche", "it": "Ricerca", "es": "Búsqueda", "pt": "Pesquisa", "de": "Suche", "hu": "Keresés"},
    "t3": {"en": "Groups", "ro": "Grupuri", "fr": "Groupes", "it": "Gruppi", "es": "Grupos", "pt": "Grupos", "de": "Gruppen", "hu": "Csoportok"},
    "t4": {"en": "Tasks", "ro": "Sarcini", "fr": "Tâches", "it": "Attività", "es": "Tareas", "pt": "Tarefas", "de": "Aufgaben", "hu": "Feladatok"},
    "t5": {"en": "Document", "ro": "Document", "fr": "Document", "it": "Documento", "es": "Documento", "pt": "Documento", "de": "Dokument", "hu": "Dokumentum"},
    "t6": {"en": "Chat", "ro": "Chat", "fr": "Chat", "it": "Chat", "es": "Chat", "pt": "Chat", "de": "Chat", "hu": "Csevegés"},
    "t7": {"en": "Stats", "ro": "Statistici", "fr": "Statistiques", "it": "Statistiche", "es": "Estadísticas", "pt": "Estatísticas", "de": "Statistik", "hu": "Statisztika"},
    "c1": {"en": "Every document you chose, with its location. Add folders once; only what changed is re-read.",
           "ro": "Fiecare document ales de tine, cu locația lui. Adaugi folderele o dată; se recitește doar ce s-a schimbat.",
           "fr": "Chaque document choisi, avec son emplacement. Tu ajoutes les dossiers une fois ; seul ce qui a changé est relu.",
           "it": "Ogni documento scelto, con la sua posizione. Aggiungi le cartelle una volta; si rilegge solo ciò che è cambiato.",
           "es": "Cada documento que elegiste, con su ubicación. Añades las carpetas una vez; solo se relee lo que cambió.",
           "pt": "Cada documento que escolheste, com a sua localização. Adicionas as pastas uma vez; só se relê o que mudou.",
           "de": "Jedes Dokument, das du gewählt hast, mit seinem Ort. Ordner einmal hinzufügen; nur Geändertes wird neu gelesen.",
           "hu": "Minden általad választott dokumentum a helyével együtt. A mappákat egyszer adod hozzá; csak a változás olvasódik újra."},
    "c2": {"en": "Full-text search in a flash: phrases, prefix*, AND/OR/NOT, filters — and “did you mean” when you mistype.",
           "ro": "Căutare full-text instantanee: fraze, prefix*, AND/OR/NOT, filtre — și „ai vrut să scrii” când greșești.",
           "fr": "Recherche plein texte instantanée : expressions, préfixe*, AND/OR/NOT, filtres — et « tu voulais dire » si tu fais une faute.",
           "it": "Ricerca full-text istantanea: frasi, prefisso*, AND/OR/NOT, filtri — e «intendevi» quando sbagli a scrivere.",
           "es": "Búsqueda de texto completo al instante: frases, prefijo*, AND/OR/NOT, filtros — y «¿quisiste decir?» si te equivocas.",
           "pt": "Pesquisa de texto completo num instante: frases, prefixo*, AND/OR/NOT, filtros — e «querias dizer» quando te enganas.",
           "de": "Volltextsuche in Sekunden: Phrasen, Präfix*, AND/OR/NOT, Filter — und „meintest du“ bei Tippfehlern.",
           "hu": "Villámgyors teljes szöveges keresés: kifejezések, előtag*, AND/OR/NOT, szűrők — és „erre gondoltál?” elírás esetén."},
    "c3": {"en": "Turn any search into a reusable group — a folder of knowledge for one question or project.",
           "ro": "Transformă orice căutare într-un grup reutilizabil — un dosar de cunoștințe pentru o întrebare sau un proiect.",
           "fr": "Transforme n’importe quelle recherche en groupe réutilisable — un dossier de connaissances pour une question ou un projet.",
           "it": "Trasforma qualsiasi ricerca in un gruppo riutilizzabile — una cartella di conoscenza per una domanda o un progetto.",
           "es": "Convierte cualquier búsqueda en un grupo reutilizable: una carpeta de conocimiento para una pregunta o un proyecto.",
           "pt": "Transforma qualquer pesquisa num grupo reutilizável — uma pasta de conhecimento para uma pergunta ou um projeto.",
           "de": "Mach aus jeder Suche eine wiederverwendbare Gruppe — ein Wissensordner für eine Frage oder ein Projekt.",
           "hu": "Bármely keresésből újrahasználható csoportot csinálhatsz — tudásmappa egy kérdéshez vagy projekthez."},
    "c4": {"en": "Summarize, extract, synthesize and more — the answer cites its sources as [S1], [S2]… and says so when it does not know.",
           "ro": "Rezumă, extrage, sintetizează și altele — răspunsul își citează sursele ca [S1], [S2]… și spune când nu știe.",
           "fr": "Résume, extrais, synthétise et plus — la réponse cite ses sources [S1], [S2]… et dit quand elle ne sait pas.",
           "it": "Riassumi, estrai, sintetizza e altro — la risposta cita le fonti come [S1], [S2]… e dice quando non sa.",
           "es": "Resume, extrae, sintetiza y más: la respuesta cita sus fuentes como [S1], [S2]… y avisa cuando no lo sabe.",
           "pt": "Resume, extrai, sintetiza e mais — a resposta cita as fontes como [S1], [S2]… e diz quando não sabe.",
           "de": "Zusammenfassen, extrahieren, synthetisieren und mehr — die Antwort zitiert ihre Quellen als [S1], [S2]… und sagt, wenn sie etwas nicht weiß.",
           "hu": "Összegzés, kinyerés, szintézis és több — a válasz [S1], [S2]… jelöléssel hivatkozik a forrásaira, és jelzi, ha nem tudja."},
    "c5": {"en": "Open any document next to its related ones; add tags, a star and your own notes — all saved locally.",
           "ro": "Deschide orice document lângă cele înrudite; adaugă etichete, o stea și notele tale — totul salvat local.",
           "fr": "Ouvre un document à côté de ses voisins ; ajoute des étiquettes, une étoile et tes notes — tout reste en local.",
           "it": "Apri un documento accanto a quelli correlati; aggiungi tag, una stella e le tue note — tutto salvato in locale.",
           "es": "Abre un documento junto a los relacionados; añade etiquetas, una estrella y tus notas: todo se guarda en local.",
           "pt": "Abre um documento ao lado dos relacionados; junta etiquetas, uma estrela e as tuas notas — tudo guardado localmente.",
           "de": "Öffne ein Dokument neben verwandten; füge Tags, einen Stern und eigene Notizen hinzu — alles lokal gespeichert.",
           "hu": "Nyiss meg egy dokumentumot a rokonai mellett; címkék, csillag és saját jegyzetek — minden helyben mentve."},
    "c6": {"en": "Ask your library in plain words. You get a cited answer, or an honest “not found”.",
           "ro": "Întreabă-ți biblioteca pe limba ta. Primești un răspuns cu surse sau un „nu am găsit” sincer.",
           "fr": "Interroge ta bibliothèque avec tes mots. Tu obtiens une réponse sourcée, ou un « introuvable » honnête.",
           "it": "Chiedi alla tua libreria con parole semplici. Ottieni una risposta con fonti, o un onesto «non trovato».",
           "es": "Pregunta a tu biblioteca con tus palabras. Recibes una respuesta con fuentes, o un «no encontrado» honesto.",
           "pt": "Pergunta à tua biblioteca por palavras simples. Recebes uma resposta com fontes, ou um «não encontrado» honesto.",
           "de": "Frag deine Bibliothek in einfachen Worten. Du bekommst eine Antwort mit Quellen — oder ein ehrliches „nicht gefunden“.",
           "hu": "Kérdezd a könyvtáradat egyszerű szavakkal. Forrásokkal ellátott választ kapsz, vagy őszinte „nem találtam”-ot."},
    "c7": {"en": "See what is inside your library: types, sizes and growth over time.",
           "ro": "Vezi ce se află în biblioteca ta: tipuri, dimensiuni și evoluția în timp.",
           "fr": "Vois ce que contient ta bibliothèque : types, tailles et évolution dans le temps.",
           "it": "Guarda cosa c’è nella tua libreria: tipi, dimensioni e crescita nel tempo.",
           "es": "Mira qué hay en tu biblioteca: tipos, tamaños y evolución en el tiempo.",
           "pt": "Vê o que há na tua biblioteca: tipos, tamanhos e evolução ao longo do tempo.",
           "de": "Sieh, was in deiner Bibliothek steckt: Typen, Größen und Entwicklung über die Zeit.",
           "hu": "Nézd meg, mi van a könyvtáradban: típusok, méretek és változás az időben."},
    "h_video": {"en": "Watch it work", "ro": "Vezi-l la lucru", "fr": "Regarde-le en action", "it": "Guardalo in azione", "es": "Míralo en acción", "pt": "Vê-o em ação", "de": "Sieh es in Aktion", "hu": "Nézd meg működés közben"},
    "video_sub": {"en": "Under a minute, the real interface, narrated by captions in your language.", "ro": "Sub un minut, interfața reală, explicată prin subtitrări în limba ta.",
                  "fr": "Moins d’une minute, la vraie interface, commentée par des sous-titres dans ta langue.", "it": "Meno di un minuto, l’interfaccia reale, spiegata da didascalie nella tua lingua.",
                  "es": "Menos de un minuto, la interfaz real, explicada con subtítulos en tu idioma.", "pt": "Menos de um minuto, a interface real, explicada por legendas na tua língua.",
                  "de": "Unter einer Minute, die echte Oberfläche, erklärt durch Untertitel in deiner Sprache.", "hu": "Egy percnél rövidebben, a valódi felület, feliratokkal a te nyelveden."},
    "h_devices": {"en": "Desktop, tablet, phone", "ro": "Desktop, tabletă, telefon", "fr": "Ordinateur, tablette, téléphone", "it": "Desktop, tablet, telefono",
                  "es": "Ordenador, tableta, móvil", "pt": "Computador, tablet, telemóvel", "de": "Desktop, Tablet, Smartphone", "hu": "Asztali gép, tablet, telefon"},
    "dev_sub": {"en": "The interface adapts to every screen — and so does this page.", "ro": "Interfața se adaptează la orice ecran — la fel și această pagină.",
                "fr": "L’interface s’adapte à tous les écrans — cette page aussi.", "it": "L’interfaccia si adatta a ogni schermo — e anche questa pagina.",
                "es": "La interfaz se adapta a cualquier pantalla, y esta página también.", "pt": "A interface adapta-se a qualquer ecrã — e esta página também.",
                "de": "Die Oberfläche passt sich jedem Bildschirm an — diese Seite auch.", "hu": "A felület minden képernyőhöz alkalmazkodik — ez az oldal is."},
    "coauthor": {"en": "co-author", "ro": "coautor", "fr": "co-auteur", "it": "coautore", "es": "coautor", "pt": "coautor", "de": "Mitautor", "hu": "társszerző"},
    "theme": {"en": "Theme", "ro": "Temă", "fr": "Thème", "it": "Tema", "es": "Tema", "pt": "Tema", "de": "Design", "hu": "Téma"},
    "skip": {"en": "Skip to content", "ro": "Sari la conținut", "fr": "Aller au contenu", "it": "Vai al contenuto", "es": "Saltar al contenido", "pt": "Saltar para o conteúdo", "de": "Zum Inhalt springen", "hu": "Ugrás a tartalomhoz"},
    "play": {"en": "Play / pause the tour", "ro": "Pornește / oprește turul", "fr": "Lancer / mettre en pause la visite", "it": "Avvia / ferma il tour", "es": "Iniciar / pausar el recorrido",
             "pt": "Iniciar / pausar a visita", "de": "Tour starten / anhalten", "hu": "Séta indítása / szüneteltetése"},
    "no_video": {"en": "Your browser cannot play this video.", "ro": "Browserul tău nu poate reda acest video.", "fr": "Ton navigateur ne peut pas lire cette vidéo.",
                 "it": "Il tuo browser non può riprodurre questo video.", "es": "Tu navegador no puede reproducir este vídeo.", "pt": "O teu navegador não consegue reproduzir este vídeo.",
                 "de": "Dein Browser kann dieses Video nicht abspielen.", "hu": "A böngésződ nem tudja lejátszani ezt a videót."},
    "alt_phone": {"en": "CartaColibri on a phone", "ro": "CartaColibri pe telefon", "fr": "CartaColibri sur téléphone", "it": "CartaColibri sul telefono", "es": "CartaColibri en el móvil",
                  "pt": "CartaColibri no telemóvel", "de": "CartaColibri auf dem Smartphone", "hu": "CartaColibri telefonon"},
    "alt_tablet": {"en": "CartaColibri on a tablet", "ro": "CartaColibri pe tabletă", "fr": "CartaColibri sur tablette", "it": "CartaColibri sul tablet", "es": "CartaColibri en la tableta",
                   "pt": "CartaColibri no tablet", "de": "CartaColibri auf dem Tablet", "hu": "CartaColibri tableten"},
    "alt_desktop": {"en": "CartaColibri on the desktop", "ro": "CartaColibri pe desktop", "fr": "CartaColibri sur ordinateur", "it": "CartaColibri sul desktop", "es": "CartaColibri en el ordenador",
                    "pt": "CartaColibri no computador", "de": "CartaColibri auf dem Desktop", "hu": "CartaColibri asztali gépen"},
}

TEMPLATE = r"""<!doctype html>
<html lang="en" data-theme="dark"><head><meta charset="utf-8"><meta name="author" content="Alexandru-Ionuț Chiuță, Alexandru Dumbrava">
<meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">
<meta name="color-scheme" content="dark light"><meta name="theme-color" content="#0b0e14">
<title>CartaColibri — Local AI Library</title>
<meta name="description" content="Local AI library on the Colibrì engine: index, search, summarize, extract, synthesize — 100% on your computer. Open source, MIT.">
<meta property="og:title" content="CartaColibri — Local AI Library"><meta property="og:description" content="Your documents, searchable and explained — 100% local.">
<meta property="og:type" content="website"><meta property="og:image" content="cartacolibri-logo.png"><meta name="twitter:card" content="summary_large_image">
<link rel="icon" href="cartacolibri-mark.png">
<style>
:root{--bg:#0b0e14;--bg2:#10141d;--s:#151a25;--s2:#1b2230;--line:#263045;--ink:#eef2f9;--mute:#9aa7bd;--blue:#3b76e0;--cyan:#5fd7e6;--amber:#fdb93a;--vio:#8b6cf0;--glow:rgba(95,215,230,.22);--sh:0 30px 80px -30px rgba(0,0,0,.7)}
[data-theme=light]{--bg:#f6f8fc;--bg2:#eef2f9;--s:#ffffff;--s2:#f1f4fa;--line:#d7dfec;--ink:#121826;--mute:#51607a;--blue:#2a62c9;--cyan:#0a7f93;--amber:#b87400;--vio:#5a3fd0;--glow:rgba(42,98,201,.16);--sh:0 30px 70px -30px rgba(20,40,90,.35)}
*{box-sizing:border-box}html{scroll-behavior:smooth;-webkit-text-size-adjust:100%}
body{margin:0;background:var(--bg);color:var(--ink);font:17px/1.65 system-ui,-apple-system,"Segoe UI",Roboto,"Helvetica Neue",sans-serif;overflow-x:hidden}
a{color:var(--cyan)}code{background:var(--s2);border:1px solid var(--line);border-radius:6px;padding:1px 6px;font-size:.9em;overflow-wrap:anywhere}
img,video{max-width:100%}:focus-visible{outline:3px solid var(--amber);outline-offset:2px;border-radius:6px}
.skip{position:absolute;left:-999px;top:8px;background:var(--amber);color:#000;padding:8px 14px;border-radius:8px;z-index:50}.skip:focus{left:8px}
nav{display:flex;gap:4px 16px;align-items:center;padding:10px max(4vw,env(safe-area-inset-left));border-bottom:1px solid var(--line);position:sticky;top:0;z-index:20;background:color-mix(in srgb,var(--bg) 82%,transparent);backdrop-filter:blur(14px) saturate(1.4);-webkit-backdrop-filter:blur(14px) saturate(1.4)}
nav .brand{display:flex;align-items:center;gap:8px;font-weight:800;letter-spacing:.2px;margin-right:auto;color:var(--ink);text-decoration:none}nav .brand img{width:28px;height:28px}
nav .links{display:flex;gap:2px 14px;flex-wrap:wrap}nav .links a{color:var(--ink);text-decoration:none;padding:6px 4px;opacity:.85;border-bottom:2px solid transparent}nav .links a:hover{opacity:1;border-color:var(--cyan)}
select,.ib{background:var(--s);color:var(--ink);border:1px solid var(--line);border-radius:12px;padding:7px 12px;font:inherit;font-size:15px;cursor:pointer}.ib{width:40px;height:38px;padding:0;display:grid;place-items:center}
select:hover,.ib:hover{border-color:var(--cyan)}
section{scroll-margin-top:84px}
/* hero */
.hero{position:relative;text-align:center;padding:clamp(40px,8vw,96px) 5vw 0;isolation:isolate;overflow:hidden}
.aurora{position:absolute;inset:-20% -10% auto -10%;height:120%;z-index:-1;filter:blur(70px) saturate(1.3);opacity:.85;pointer-events:none}
.aurora i{position:absolute;border-radius:50%;mix-blend-mode:screen;animation:float 18s ease-in-out infinite alternate}
[data-theme=light] .aurora i{mix-blend-mode:multiply;opacity:.35}
.aurora i:nth-child(1){left:8%;top:6%;width:44vw;height:44vw;background:radial-gradient(circle,#2f66c4 0,transparent 65%)}
.aurora i:nth-child(2){right:4%;top:0;width:38vw;height:38vw;background:radial-gradient(circle,#8b6cf0 0,transparent 65%);animation-delay:-6s}
.aurora i:nth-child(3){left:32%;top:26%;width:34vw;height:34vw;background:radial-gradient(circle,#2cc7d8 0,transparent 65%);animation-delay:-11s}
@keyframes float{from{transform:translate3d(-4%,-3%,0) scale(1)}to{transform:translate3d(5%,6%,0) scale(1.12)}}
.grain{position:absolute;inset:0;z-index:-1;background-image:radial-gradient(var(--line) 1px,transparent 1px);background-size:26px 26px;opacity:.35;mask-image:radial-gradient(70% 60% at 50% 20%,#000,transparent)}
.badge{display:inline-flex;gap:8px;align-items:center;padding:6px 14px;border:1px solid var(--line);border-radius:99px;background:color-mix(in srgb,var(--s) 75%,transparent);font-size:14px;color:var(--mute);backdrop-filter:blur(6px)}
.badge:before{content:"";width:8px;height:8px;border-radius:50%;background:#45d483;box-shadow:0 0 0 4px rgba(69,212,131,.18)}
.hero .logo{width:min(150px,32vw);height:auto;margin:18px auto 4px;display:block;filter:drop-shadow(0 12px 28px var(--glow))}
.hero h1{font-size:clamp(30px,6vw,62px);line-height:1.08;margin:.3em auto .35em;max-width:16em;letter-spacing:-.02em;background:linear-gradient(120deg,var(--ink) 20%,var(--cyan) 55%,var(--vio));-webkit-background-clip:text;background-clip:text;color:transparent}
.hero .lead{max-width:760px;margin:0 auto 22px;color:var(--mute);font-size:clamp(16px,2.2vw,19px)}
.btn{display:inline-flex;align-items:center;gap:8px;margin:6px;padding:14px 26px;border-radius:14px;font-weight:700;text-decoration:none;color:#fff;background:linear-gradient(135deg,var(--blue),var(--vio));box-shadow:0 10px 30px -10px var(--blue);transition:transform .2s,box-shadow .2s}
.btn:hover{transform:translateY(-2px);box-shadow:0 16px 36px -10px var(--blue)}.btn.a{background:linear-gradient(135deg,#ffcb5c,var(--amber));color:#2b1d00;box-shadow:0 10px 30px -12px var(--amber)}
.btn.g{background:var(--s);color:var(--ink);border:1px solid var(--line);box-shadow:none}
.stats{display:flex;flex-wrap:wrap;gap:10px 34px;justify-content:center;margin:30px auto 8px;padding:0;list-style:none}
.stats li{text-align:center;min-width:92px}.stats b{display:block;font-size:clamp(26px,4vw,38px);line-height:1.1;background:linear-gradient(120deg,var(--cyan),var(--amber));-webkit-background-clip:text;background-clip:text;color:transparent}.stats span{font-size:14px;color:var(--mute)}
/* device stage */
.stage{position:relative;max-width:1080px;margin:34px auto 0;padding:0 0 clamp(20px,5vw,60px);perspective:1800px}
.win{position:relative;background:var(--s);border:1px solid var(--line);border-radius:14px;box-shadow:var(--sh),0 0 0 1px rgba(255,255,255,.03) inset;overflow:hidden;transition:transform .35s ease-out}
.win .bar{display:flex;align-items:center;gap:7px;padding:10px 14px;background:var(--s2);border-bottom:1px solid var(--line)}
.win .bar i{width:11px;height:11px;border-radius:50%;background:#ff5f57}.win .bar i:nth-child(2){background:#febc2e}.win .bar i:nth-child(3){background:#28c840}
.win .bar u{margin-left:12px;flex:1;max-width:340px;text-decoration:none;font-size:12.5px;color:var(--mute);background:var(--bg2);border:1px solid var(--line);border-radius:8px;padding:2px 10px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.win img{display:block;width:100%;height:auto;aspect-ratio:1360/820;background:var(--bg2)}
.stage .main{transform:rotateX(3deg) rotateY(-2deg);transform-origin:50% 100%}
.dev{position:absolute;background:#0a0c12;border:2px solid #2a3144;box-shadow:var(--sh);overflow:hidden}
.dev img{display:block;width:100%;height:100%;object-fit:cover;object-position:top}
.phone{width:clamp(96px,17%,190px);aspect-ratio:390/844;border-radius:clamp(14px,2.4vw,28px);right:2%;bottom:0;border-width:3px;transform:translateY(0) rotate(3deg)}
.tablet{width:clamp(150px,27%,300px);aspect-ratio:820/1180;border-radius:clamp(12px,1.8vw,22px);left:1%;bottom:2%;transform:rotate(-3deg)}
.dev:before{content:"";position:absolute;top:6px;left:50%;width:26%;height:4px;margin-left:-13%;border-radius:4px;background:#2a3144;z-index:2}
.float1{animation:bob 7s ease-in-out infinite}.float2{animation:bob 9s ease-in-out -3s infinite}
@keyframes bob{0%,100%{translate:0 0}50%{translate:0 -10px}}
/* sections */
main{max-width:1120px;margin:0 auto;padding:0 5vw 90px}
h2{margin:0 0 .35em;font-size:clamp(26px,4vw,40px);line-height:1.15;letter-spacing:-.015em}
.sec{padding:clamp(48px,8vw,96px) 0 0}.sub{color:var(--mute);max-width:720px;margin:0 0 26px;font-size:18px}.center{text-align:center}.center .sub{margin-left:auto;margin-right:auto}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,290px),1fr));gap:16px}
.card{position:relative;background:linear-gradient(180deg,var(--s),var(--bg2));border:1px solid var(--line);border-radius:20px;padding:22px;overflow-wrap:anywhere;transition:transform .25s,border-color .25s,box-shadow .25s}
.card:hover{transform:translateY(-4px);border-color:color-mix(in srgb,var(--cyan) 55%,var(--line));box-shadow:0 18px 40px -22px var(--glow)}
.grid{align-items:start}@media(min-width:900px){#f5{grid-column:span 2}}
.card svg.ic{width:34px;height:34px;stroke:var(--cyan);fill:none;stroke-width:1.7;stroke-linecap:round;stroke-linejoin:round;margin-bottom:10px;display:block}
.card.p svg.ic{stroke:var(--amber)}
/* tour */
.tabs{display:flex;gap:8px;overflow-x:auto;padding:4px 2px 12px;margin:0 -2px;scrollbar-width:thin;scroll-snap-type:x proximity;justify-content:center;flex-wrap:wrap}
.tabs button{scroll-snap-align:center;flex:none;display:flex;align-items:center;gap:8px;padding:9px 16px;border-radius:99px;border:1px solid var(--line);background:var(--s);color:var(--ink);font:inherit;font-size:15px;font-weight:600;cursor:pointer;position:relative;overflow:hidden}
.tabs button em{font-style:normal;width:22px;height:22px;border-radius:50%;display:grid;place-items:center;font-size:12px;background:var(--s2);color:var(--mute)}
.tabs button[aria-selected=true]{border-color:var(--cyan);background:color-mix(in srgb,var(--cyan) 14%,var(--s))}
.tabs button[aria-selected=true] em{background:var(--cyan);color:#001a1f}
.tabs button .pg{position:absolute;left:0;bottom:0;height:3px;width:0;background:linear-gradient(90deg,var(--cyan),var(--amber))}
.tabs button.run .pg{animation:pg var(--dur,6s) linear forwards}@keyframes pg{to{width:100%}}
.tour .win{margin-top:6px}.tour .win img{transition:opacity .35s}.tour .cap{max-width:760px;margin:18px auto 0;text-align:center;color:var(--mute);font-size:18px;min-height:3.4em}
.play{margin-left:6px}
/* video */
.vwrap{max-width:980px;margin:0 auto;position:relative}
.vwrap video{width:100%;aspect-ratio:16/9;background:#000;display:block;border-radius:14px}
.vwrap .win{padding:0}.vwrap .win .bar u{max-width:none}
.glow{position:absolute;inset:-6% 4%;background:radial-gradient(60% 60% at 50% 50%,var(--glow),transparent);filter:blur(30px);z-index:-1}
/* flow, code, tables */
.flow{display:flex;flex-wrap:wrap;gap:10px;align-items:center;justify-content:center;margin:24px 0}.flow span{padding:12px 18px;border-radius:14px;border:1px solid var(--line);background:var(--s);font-weight:700}.flow i{color:var(--cyan);font-style:normal}
.cb{position:relative}pre{background:var(--s);border:1px solid var(--line);border-radius:14px;padding:16px;padding-right:90px;overflow:auto;font-size:14px;margin:.6em 0;line-height:1.55}
.cp{position:absolute;top:9px;right:9px;background:var(--bg);color:var(--ink);border:1px solid var(--line);border-radius:8px;padding:3px 10px;font:inherit;font-size:13px;cursor:pointer}.cp:hover{border-color:var(--cyan)}
ol.steps{padding-left:0;list-style:none;counter-reset:s;display:grid;gap:12px;margin:18px 0}ol.steps li{counter-increment:s;position:relative;background:var(--s);border:1px solid var(--line);border-radius:16px;padding:14px 16px 14px 60px}
ol.steps li:before{content:counter(s);position:absolute;left:14px;top:50%;translate:0 -50%;width:34px;height:34px;border-radius:50%;display:grid;place-items:center;font-weight:800;color:#001a1f;background:linear-gradient(135deg,var(--cyan),var(--amber))}
details{background:var(--s);border:1px solid var(--line);border-radius:14px;padding:12px 18px;margin:9px 0;transition:border-color .2s}details[open]{border-color:var(--cyan)}summary{cursor:pointer;font-weight:650}details p{margin:.6em 0 .2em;color:var(--mute)}
.tw{overflow-x:auto;border-radius:14px;border:1px solid var(--line)}table{border-collapse:collapse;width:100%;font-size:15px}th,td{border-bottom:1px solid var(--line);padding:10px 12px;text-align:left;vertical-align:top}th{background:var(--s2)}tr:last-child td{border-bottom:0}
dt{font-weight:700;margin-top:.9em}dd{margin:.1em 0 0;color:var(--mute)}
.note{border-left:4px solid var(--amber);background:var(--s);border-radius:0 14px 14px 0;padding:12px 18px}
footer{border-top:1px solid var(--line);padding:30px 5vw 50px;text-align:center;color:var(--mute);font-size:15px;background:var(--bg2)}
/* reveal */
.rv{opacity:0;transform:translateY(26px);transition:opacity .7s ease,transform .7s ease}.rv.in{opacity:1;transform:none}
.no-js .rv{opacity:1;transform:none}
@media(max-width:820px){nav .links{display:none}.stage .main{transform:none}.tablet{width:30%}.phone{width:20%}}
@media(max-width:560px){body{font-size:16px}.tablet{display:none}.phone{width:30%;right:3%;bottom:-6px}.hero{padding-top:34px}.btn{padding:13px 20px}th,td{padding:7px}.tabs{justify-content:flex-start;flex-wrap:nowrap}.win .bar u{display:none}ol.steps li{padding-left:56px}}
@media(prefers-reduced-motion:reduce){*{animation:none!important;transition:none!important;scroll-behavior:auto!important}.rv{opacity:1;transform:none}}
@media print{nav,.aurora,.grain,.stage,.vwrap{display:none}body{background:#fff;color:#000}}
</style></head><body class="no-js">
<a class="skip" href="#start" data-n="skip"></a>
<nav><a class="brand" href="#top"><img src="cartacolibri-mark.png" alt="" width="28" height="28">CartaColibri</a>
<div class="links"><a href="#tour" id="n_tour"></a><a href="#start" id="n_start"></a><a href="#features" id="n_features"></a><a href="#faq" id="n_faq"></a><a href="#privacy" id="n_privacy"></a><a href="#download" id="n_download"></a></div>
<button class="ib" id="th" type="button" data-nlabel="theme" aria-label="Theme"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></svg></button>
<select id="l" aria-label="Language"></select></nav>
<header class="hero" id="top"><div class="aurora" aria-hidden="true"><i></i><i></i><i></i></div><div class="grain" aria-hidden="true"></div>
<span class="badge" data-n="badge"></span>
<img class="logo" src="cartacolibri-mark.png" alt="" width="150" height="150">
<h1 id="tagline"></h1><p class="lead" id="intro"></p>
<a class="btn a" href="#download" id="dl"></a><a class="btn g" href="#tour" id="h_first_btn"></a><a class="btn g" href="__REPO__">GitHub</a>
<p id="dl_sub" style="font-size:15px;color:var(--mute);margin:6px 0 0"></p>
<ul class="stats"><li><b>8</b><span data-n="st_lang"></span></li><li><b>14</b><span data-n="st_tasks"></span></li><li><b>0</b><span data-n="st_cloud"></span></li><li><b>100%</b><span data-n="st_local"></span></li></ul>
<div class="stage" id="stage">
<div class="win main"><div class="bar"><i></i><i></i><i></i><u>127.0.0.1:8765</u></div><img id="hero_img" alt="" width="1360" height="820" fetchpriority="high" decoding="async"></div>
<div class="dev tablet float2" id="hero_tab"><img id="hero_t" alt="" loading="lazy" decoding="async"></div>
<div class="dev phone float1" id="hero_ph"><img id="hero_p" alt="" loading="lazy" decoding="async"></div>
</div></header>
<main id="main">
<section class="sec tour rv" id="tour"><div class="center"><h2 data-n="h_tour"></h2><p class="sub" data-n="tour_sub"></p></div>
<div class="tabs" id="tabs" role="tablist"></div>
<div class="win"><div class="bar"><i></i><i></i><i></i><u id="tour_url">127.0.0.1:8765</u><button class="ib play" id="play" type="button" data-nlabel="play" aria-label="Play" style="width:30px;height:26px;margin-left:auto">⏯</button></div><img id="tour_img" alt="" width="1360" height="820" decoding="async"></div>
<p class="cap" id="tour_cap" aria-live="polite"></p></section>

<section class="sec rv" id="video"><div class="center"><h2 id="h_demo"></h2><p class="sub" data-n="video_sub"></p></div>
<div class="vwrap"><div class="glow" aria-hidden="true"></div><div class="win"><div class="bar"><i></i><i></i><i></i><u>CartaColibri</u></div>
<video id="vid" controls playsinline preload="none" muted></video></div></div>
<p class="center" style="margin-top:18px"><a id="tutpdf" class="btn g" href="#"></a></p></section>

<section class="sec rv center" id="devices"><h2 data-n="h_devices"></h2><p class="sub" data-n="dev_sub"></p>
<div class="stage" style="max-width:900px"><div class="win main"><div class="bar"><i></i><i></i><i></i><u>127.0.0.1:8765</u></div><img id="dv_d" alt="" loading="lazy" decoding="async"></div>
<div class="dev tablet" style="left:-1%"><img id="dv_t" alt="" loading="lazy" decoding="async"></div><div class="dev phone" style="right:-1%"><img id="dv_p" alt="" loading="lazy" decoding="async"></div></div></section>

<section class="sec" id="features"><div class="rv"><h2 id="h_what"></h2></div>
<div class="grid rv" id="fgrid"><div class="card" id="f1"></div><div class="card" id="f2"></div><div class="card" id="f3"></div><div class="card" id="f4"></div><div class="card" id="f5"></div></div>
<div class="rv"><h2 id="h_power" style="margin-top:1.6em"></h2></div>
<div class="grid rv" id="pgrid"><div class="card p" id="p1"></div><div class="card p" id="p2"></div><div class="card p" id="p3"></div><div class="card p" id="p4"></div><div class="card p" id="p5"></div><div class="card p" id="p6"></div><div class="card p" id="p7"></div><div class="card p" id="p8"></div></div>
<div class="rv"><h2 id="h_layers" style="margin-top:1.6em"></h2><div class="flow"><span>📁 Layer 0</span><i>→</i><span>RAG-fetch</span><i>→</i><span>RAG-CARTA</span><i>→</i><span>Colibrì</span></div><p id="layers" class="center sub"></p></div></section>

<section class="sec" id="new8"><div class="rv"><h2 id="h_new8"></h2></div>
__N8GRID__

<section class="sec" id="new4"><div class="rv"><h2 id="h_new4"></h2></div>
__N4GRID__

<section class="sec" id="new"><div class="rv"><h2 id="h_new"></h2></div>
<div class="grid rv" id="nwgrid"><div class="card p" id="nw1"></div><div class="card p" id="nw2"></div><div class="card p" id="nw3"></div><div class="card p" id="nw4"></div><div class="card p" id="nw5"></div><div class="card p" id="nw6"></div></div></section>

<section class="sec rv" id="start"><h2 id="h_first"></h2><p class="sub" id="first_intro"></p><ol class="steps" id="steps"></ol><p id="first_note"></p>
<div class="cb"><pre>__FIRST_CODE__</pre></div>
<h2 id="h_start" style="margin-top:1.6em"></h2><p id="start1"></p><div class="cb"><pre>git clone __REPO__
cd CartaColibri
cargo build --release
./target/release/cartacolibri serve
# http://127.0.0.1:8765/</pre></div><p id="start2"></p><div class="cb"><pre>cartacolibri add ~/Documents
cartacolibri search "contract*" --ext pdf,docx
cartacolibri group set Contracts ~/Documents/contracts
cartacolibri synthesize "payment terms" --group Contracts --save --mock
cartacolibri grab https://example.org/article
cartacolibri-browser https://example.org</pre></div>
<h2 id="h_cli" style="margin-top:1.6em"></h2><div class="tw"><table><thead><tr id="clih"></tr></thead><tbody id="clib"></tbody></table></div></section>

<section class="sec rv" id="faq"><h2 id="h_faq"></h2><div id="faqs"></div>
<h2 id="h_trouble" style="margin-top:1.6em"></h2><p id="trouble_intro"></p><div class="tw"><table><thead><tr id="thr"></tr></thead><tbody id="tbl"></tbody></table></div>
<h2 id="h_gloss" style="margin-top:1.6em"></h2><dl id="gloss"></dl></section>
<section class="sec rv" id="privacy"><h2 id="h_priv"></h2><ul id="priv"></ul></section>
<section class="sec rv" id="security"><h2 id="h_sec"></h2><p id="sec"></p></section>
<section class="sec rv center" id="download"><h2 id="h_dl"></h2><p><a class="btn a" href="__REPO__/releases" id="dl2"></a><a class="btn g" href="__REPO__">GitHub</a></p><p id="dl_sub2" style="color:var(--mute)"></p>
<div style="text-align:left"><h2 id="h_status" style="margin-top:1.4em"></h2><p class="note" id="status"></p><h2 id="h_lic" style="margin-top:1.4em"></h2><p id="lic"></p></div></section>
</main>
<footer>© 2026 Alexandru-Ionuț Chiuță &amp; Alexandru Dumbrava (<span id="coauth"></span>) · CartaColibri · MIT · <a href="__REPO__">__REPOHOST__</a></footer>
<script>
const D=__DATA__,N=__NAMES__,X=__NEW__,IDS=__IDS__;
const $=id=>document.getElementById(id),sel=$('l'),root=document.documentElement;
document.body.classList.remove('no-js');
let L=(()=>{try{const s=localStorage.getItem('cc.lang');if(s&&N[s])return s}catch(e){}const n=(navigator.language||'en').slice(0,2);return N[n]?n:'en'})();
sel.innerHTML=Object.entries(N).map(([k,v])=>'<option value="'+k+'">'+v+'</option>').join('');
/* theme */
const mq=matchMedia('(prefers-color-scheme: light)');let th=null;try{th=localStorage.getItem('cc.theme')}catch(e){}
function setTheme(t){root.dataset.theme=t;document.querySelector('meta[name=theme-color]').content=t==='light'?'#f6f8fc':'#0b0e14'}
setTheme(th||(mq.matches?'light':'dark'));
$('th').onclick=()=>{const t=root.dataset.theme==='dark'?'light':'dark';setTheme(t);try{localStorage.setItem('cc.theme',t)}catch(e){}};
/* icons */
const IC={f1:'<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',f2:'<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>',f3:'<path d="M12 3l1.9 4.6L19 9l-4.1 3.2L16 17l-4-2.7L8 17l1.1-4.8L5 9l5.1-1.4z"/>',f4:'<rect x="4" y="11" width="16" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/>',f5:'<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c3 3.2 3 14.8 0 18M12 3c-3 3.2-3 14.8 0 18"/>',
p1:'<path d="M4 7V5a1 1 0 0 1 1-1h2M17 4h2a1 1 0 0 1 1 1v2M20 17v2a1 1 0 0 1-1 1h-2M7 20H5a1 1 0 0 1-1-1v-2M8 12h8"/>',p2:'<path d="M20 12 12 20 4 12V4h8z"/><circle cx="8.5" cy="8.5" r="1"/>',p3:'<path d="M4 6h16M4 12h16M4 18h10"/>',p4:'<path d="M4 4h16v12H8l-4 4z"/>',p5:'<path d="M12 3v12m0 0-4-4m4 4 4-4M5 21h14"/>',p6:'<path d="M12 21V9m0 0-4 4m4-4 4 4M5 3h14"/>'};
for(const k in IC){const e=$(k);e.dataset.ic=k}
function icons(){for(const k in IC){const e=$(k);if(!e.querySelector('svg.ic'))e.insertAdjacentHTML('afterbegin','<svg class="ic" viewBox="0 0 24 24" aria-hidden="true">'+IC[k]+'</svg>')}}
/* copy buttons */
function addCopy(){document.querySelectorAll('.cb').forEach(b=>{let c=b.querySelector('.cp');if(!c){c=document.createElement('button');c.type='button';c.className='cp';b.appendChild(c);
c.onclick=()=>{const t=b.querySelector('pre').innerText;const ok=()=>{c.textContent=D.copied[L];setTimeout(()=>c.textContent=D.copy[L],1500)};
if(navigator.clipboard&&window.isSecureContext)navigator.clipboard.writeText(t).then(ok,()=>{});else{const a=document.createElement('textarea');a.value=t;a.style.position='fixed';a.style.opacity='0';document.body.appendChild(a);a.select();try{document.execCommand('copy');ok()}catch(e){}a.remove()}}}
c.textContent=D.copy[L]})}
/* tour */
let cur=0,timer=null,playing=!matchMedia('(prefers-reduced-motion: reduce)').matches,userStopped=false;const DUR=6500;
function buildTabs(){$('tabs').innerHTML=[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19].map((n,i)=>'<button type="button" role="tab" id="tab'+n+'" aria-selected="'+(i===cur)+'" aria-controls="tour_img" tabindex="'+(i===cur?0:-1)+'"><em>'+n+'</em>'+X['t'+n][L]+'<span class="pg"></span></button>').join('');
document.querySelectorAll('#tabs button').forEach((b,i)=>{b.onclick=()=>{userStopped=true;show(i)};b.onkeydown=e=>{if(e.key==='ArrowRight'||e.key==='ArrowLeft'){e.preventDefault();userStopped=true;show((cur+(e.key==='ArrowRight'?1:13))%14);$('tab'+(cur+1)).focus()}}})}
function show(i,auto){cur=i;const n=i+1;const im=$('tour_img');im.style.opacity=.15;im.onload=()=>{im.style.opacity=1};im.src='img/'+L+'-'+n+'.webp';im.alt=X['c'+n][L];$('tour_cap').textContent=X['c'+n][L];
document.querySelectorAll('#tabs button').forEach((b,j)=>{b.setAttribute('aria-selected',j===i);b.tabIndex=j===i?0:-1;b.classList.remove('run');void b.offsetWidth});
const b=$('tab'+n);if(b&&b.scrollIntoView&&!auto)b.scrollIntoView({inline:'center',block:'nearest',behavior:'smooth'});
clearTimeout(timer);if(playing&&!userStopped&&inView){b.style.setProperty('--dur',DUR+'ms');b.classList.add('run');timer=setTimeout(()=>show((cur+1)%19,true),DUR)}}
let inView=false;
$('play').onclick=()=>{playing=!playing;userStopped=false;show(cur,true)};
/* language */
function go(){root.lang=L;sel.value=L;for(const k of IDS){const e=$(k);if(e)e.innerHTML=D[k][L]}
document.querySelectorAll('[data-n]').forEach(e=>e.textContent=X[e.dataset.n][L]);$('coauth').textContent=X.coauthor[L];document.querySelectorAll('[data-nlabel]').forEach(e=>e.setAttribute('aria-label',X[e.dataset.nlabel][L]));
for(const k of ['n_start','n_features','n_faq','n_privacy','n_download'])$(k).textContent=D[k][L];$('n_tour').textContent=X.n_tour[L];
$('h_first_btn').textContent=X.tour_btn[L];$('h_dl').textContent=D.dl[L];$('dl').textContent=D.dl[L];$('dl2').textContent=D.dl[L];
$('steps').innerHTML=[1,2,3].map(i=>'<li>'+D['first'+i][L]+'</li>').join('');
$('faqs').innerHTML=D.faq[L].map(x=>'<details><summary>'+x[0]+'</summary><p>'+x[1]+'</p></details>').join('');
$('thr').innerHTML=D.th[L].map(x=>'<th>'+x+'</th>').join('');
$('tbl').innerHTML=D.trouble[L].map(r=>'<tr>'+r.map(c=>'<td>'+c+'</td>').join('')+'</tr>').join('');
$('clih').innerHTML=D.cli_th[L].map(x=>'<th>'+x+'</th>').join('');
$('clib').innerHTML=D.cli[L].map(r=>'<tr>'+r.map(c=>'<td>'+c+'</td>').join('')+'</tr>').join('');
$('gloss').innerHTML=D.gloss[L].map(x=>'<dt>'+x[0]+'</dt><dd>'+x[1]+'</dd>').join('');
$('priv').innerHTML=D.priv[L].map(x=>'<li>'+x+'</li>').join('');
const tp=$('tutpdf');tp.textContent='📄 '+D.tut[L];tp.href='media/CartaColibri-Tutorial.'+L+'.pdf';
$('hero_img').src='img/'+L+'-2.webp';$('hero_img').alt=X.alt_desktop[L];
for(const [id,f,a] of [['hero_t','t1','alt_tablet'],['hero_p','m1','alt_phone'],['dv_t','t1','alt_tablet'],['dv_p','m2','alt_phone'],['dv_d','1','alt_desktop']]){const e=$(id);e.alt=X[a][L];e.src='img/'+L+'-'+f+'.webp';e.onerror=()=>{(e.closest('.dev')||e).style.display='none'};e.onload=()=>{(e.closest('.dev')||e).style.display=''}}
const v=$('vid');v.innerHTML='';const pv=v.paused;v.poster='video/'+L+'-demo.jpg';v.src='video/'+L+'-demo.mp4';v.setAttribute('aria-label',X.h_video[L]);v.onerror=()=>{const s=$('video');if(s)s.style.display='none'};v.load();
$('dl_sub').innerHTML=D.dl_sub[L];$('dl_sub2').innerHTML=D.dl_sub[L];buildTabs();show(cur,true);icons();addCopy()}
sel.onchange=()=>{L=sel.value;try{localStorage.setItem('cc.lang',L)}catch(e){}go()};
/* reveal + autoplay only while the tour is visible */
const io='IntersectionObserver'in window?new IntersectionObserver(es=>es.forEach(e=>{if(e.isIntersecting){e.target.classList.add('in');io.unobserve(e.target)}}),{threshold:.08}):null;
document.querySelectorAll('.rv').forEach(e=>io?io.observe(e):e.classList.add('in'));
if('IntersectionObserver'in window){new IntersectionObserver(es=>{inView=es[0].isIntersecting;show(cur,true)},{threshold:.35}).observe($('tour'))}
/* gentle pointer tilt on the hero (desktop only) */
const st=$('stage');if(matchMedia('(hover:hover) and (min-width:900px)').matches&&!matchMedia('(prefers-reduced-motion: reduce)').matches){const m=st.querySelector('.main');
st.addEventListener('pointermove',e=>{const r=st.getBoundingClientRect(),x=(e.clientX-r.left)/r.width-.5,y=(e.clientY-r.top)/r.height-.5;m.style.transform='rotateX('+(3-y*5)+'deg) rotateY('+(-2+x*6)+'deg)'});st.addEventListener('pointerleave',()=>m.style.transform='')}
go();
</script></body></html>
"""
