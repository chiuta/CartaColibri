#!/usr/bin/env python3
"""Builds the illustrated tutorial PDFs (8 languages), the subtitled demo video (8 soft subtitle tracks) and the
.vtt files for the site, from the screenshots made by scripts/capture_carta_demo.py.
Output: docs/cartacolibri/CartaColibri-Tutorial.<lang>.pdf · site-carta/media/cartacolibri-demo.mp4 + demo.<lang>.vtt
Usage: python3 scripts/gen_carta_tutorial.py"""
import os, subprocess, sys, tempfile
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen_carta_docs import X, LANGS, NAMES, ROOT
from reportlab.lib.pagesizes import A4
from reportlab.lib.units import mm
from reportlab.lib.utils import ImageReader
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.pdfgen import canvas
from PIL import Image, ImageDraw, ImageFont

# step -> lang -> (title, caption(short, for subtitles), text(long, for the PDF))
S = [
{"img": 1,
 "title": {"en": "1 · Add your folders", "ro": "1 · Adaugă folderele tale", "fr": "1 · Ajoute tes dossiers", "it": "1 · Aggiungi le tue cartelle", "es": "1 · Añade tus carpetas", "pt": "1 · Adiciona as tuas pastas", "de": "1 · Füge deine Ordner hinzu", "hu": "1 · Add hozzá a mappáidat"},
 "cap": {"en": "Open Library and add a folder: every document appears with its location.", "ro": "Deschide Biblioteca și adaugă un folder: fiecare document apare cu locația lui.", "fr": "Ouvre la Bibliothèque et ajoute un dossier : chaque document apparaît avec son emplacement.", "it": "Apri la Biblioteca e aggiungi una cartella: ogni documento appare con la sua posizione.", "es": "Abre la Biblioteca y añade una carpeta: cada documento aparece con su ubicación.", "pt": "Abre a Biblioteca e adiciona uma pasta: cada documento aparece com a sua localização.", "de": "Öffne die Bibliothek und füge einen Ordner hinzu: Jedes Dokument erscheint mit seinem Speicherort.", "hu": "Nyisd meg a Könyvtárat és adj hozzá egy mappát: minden dokumentum a helyével együtt megjelenik."},
 "text": {"en": "In the Library screen type the path of a folder and press “Add folder”. CartaColibri indexes it once and later updates only what changed. You can also drop files on the dashed box to upload them; with tesseract installed you can save OCR text or a searchable PDF. Your originals are never modified.",
          "ro": "În ecranul Bibliotecă scrie calea unui folder și apasă „Adaugă folder”. CartaColibri îl indexează o dată, apoi actualizează doar ce s-a schimbat. Poți și să tragi fișiere în caseta punctată ca să le încarci; dacă ai tesseract instalat, poți salva text OCR sau un PDF căutabil. Originalele tale nu sunt modificate niciodată.",
          "fr": "Dans l’écran Bibliothèque, saisis le chemin d’un dossier et appuie sur « Ajouter un dossier ». CartaColibri l’indexe une fois, puis ne met à jour que ce qui a changé. Tu peux aussi déposer des fichiers dans la zone en pointillés ; avec tesseract, tu peux enregistrer le texte OCR ou un PDF interrogeable. Tes originaux ne sont jamais modifiés.",
          "it": "Nella schermata Biblioteca scrivi il percorso di una cartella e premi «Aggiungi cartella». CartaColibri la indicizza una volta e poi aggiorna solo ciò che è cambiato. Puoi anche trascinare file nel riquadro tratteggiato per caricarli; con tesseract puoi salvare il testo OCR o un PDF ricercabile. I tuoi originali non vengono mai modificati.",
          "es": "En la pantalla Biblioteca escribe la ruta de una carpeta y pulsa «Añadir carpeta». CartaColibri la indexa una vez y después solo actualiza lo que cambió. También puedes soltar archivos en el recuadro de puntos para subirlos; con tesseract puedes guardar el texto OCR o un PDF con texto buscable. Tus originales nunca se modifican.",
          "pt": "No ecrã Biblioteca escreve o caminho de uma pasta e carrega em «Adicionar pasta». O CartaColibri indexa-a uma vez e depois só atualiza o que mudou. Podes também largar ficheiros na caixa tracejada para os enviar; com o tesseract instalado, podes guardar o texto OCR ou um PDF pesquisável. Os teus originais nunca são modificados.",
          "de": "Gib im Bildschirm Bibliothek den Pfad eines Ordners ein und klicke auf „Ordner hinzufügen“. CartaColibri indiziert ihn einmal und aktualisiert danach nur das Geänderte. Du kannst auch Dateien auf das gestrichelte Feld ziehen, um sie hochzuladen; mit Tesseract kannst du OCR-Text oder ein durchsuchbares PDF speichern. Deine Originale werden nie verändert.",
          "hu": "A Könyvtár képernyőn írd be egy mappa útvonalát, és nyomd meg a „Mappa hozzáadása” gombot. A CartaColibri egyszer indexeli, utána csak a változásokat frissíti. A szaggatott dobozba fájlokat is húzhatsz feltöltéshez; tesseract telepítése esetén OCR-szöveget vagy kereshető PDF-et is menthetsz. Az eredeti fájljaidat soha nem módosítja."}},
{"img": 2,
 "title": {"en": "2 · Search with RAG-fetch", "ro": "2 · Caută cu RAG-fetch", "fr": "2 · Cherche avec RAG-fetch", "it": "2 · Cerca con RAG-fetch", "es": "2 · Busca con RAG-fetch", "pt": "2 · Procura com o RAG-fetch", "de": "2 · Suche mit RAG-fetch", "hu": "2 · Keress a RAG-fetch-csel"},
 "cap": {"en": "Search words, phrases, prefix* and AND / OR / NOT; filter by type, size and location.", "ro": "Caută cuvinte, expresii, prefix* și AND / OR / NOT; filtrează după tip, mărime și locație.", "fr": "Cherche des mots, expressions, préfixe* et AND / OR / NOT ; filtre par type, taille et emplacement.", "it": "Cerca parole, frasi, prefisso* e AND / OR / NOT; filtra per tipo, dimensione e posizione.", "es": "Busca palabras, frases, prefijo* y AND / OR / NOT; filtra por tipo, tamaño y ubicación.", "pt": "Procura palavras, frases, prefixo* e AND / OR / NOT; filtra por tipo, tamanho e localização.", "de": "Suche Wörter, Phrasen, Präfix* und AND / OR / NOT; filtere nach Typ, Größe und Ort.", "hu": "Keress szavakra, kifejezésekre, előtag*-ra és AND / OR / NOT-tal; szűrj típus, méret és hely szerint."},
 "text": {"en": "Open RAG-fetch and type your query. Use “quotes” for an exact phrase, a trailing * for prefixes, and AND / OR / NOT to combine terms. Diacritics are ignored, so “scoala” finds “școală”. Narrow the results with the type chips, the min / max size, the location filter or a group. Double-click a result to read the document with your words highlighted.",
          "ro": "Deschide RAG-fetch și scrie interogarea. Folosește „ghilimele” pentru o expresie exactă, * la final pentru prefix și AND / OR / NOT pentru a combina termeni. Diacriticele sunt ignorate, deci „scoala” găsește „școală”. Restrânge rezultatele cu butoanele de tip, mărimea min / max, filtrul de locație sau un grup. Dublu-clic pe un rezultat ca să citești documentul cu cuvintele tale evidențiate.",
          "fr": "Ouvre RAG-fetch et saisis ta requête. Utilise « guillemets » pour une expression exacte, * en fin de mot pour les préfixes et AND / OR / NOT pour combiner. Les diacritiques sont ignorés : « scoala » trouve « școală ». Affine avec les pastilles de type, la taille min / max, le filtre d’emplacement ou un groupe. Double-clique sur un résultat pour lire le document avec tes mots surlignés.",
          "it": "Apri RAG-fetch e scrivi la tua ricerca. Usa «virgolette» per una frase esatta, * finale per i prefissi e AND / OR / NOT per combinare i termini. I segni diacritici vengono ignorati: «scoala» trova «școală». Restringi con i tipi, la dimensione min / max, il filtro di posizione o un gruppo. Doppio clic su un risultato per leggere il documento con le tue parole evidenziate.",
          "es": "Abre RAG-fetch y escribe tu consulta. Usa «comillas» para una frase exacta, * al final para prefijos y AND / OR / NOT para combinar términos. Se ignoran los signos diacríticos: «scoala» encuentra «școală». Acota con las etiquetas de tipo, el tamaño mín. / máx., el filtro de ubicación o un grupo. Haz doble clic en un resultado para leer el documento con tus palabras resaltadas.",
          "pt": "Abre o RAG-fetch e escreve a tua pesquisa. Usa «aspas» para uma frase exata, * no fim para prefixos e AND / OR / NOT para combinar termos. Os diacríticos são ignorados: «scoala» encontra «școală». Restringe com as etiquetas de tipo, o tamanho mín. / máx., o filtro de localização ou um grupo. Faz duplo clique num resultado para ler o documento com as tuas palavras realçadas.",
          "de": "Öffne RAG-fetch und gib deine Suche ein. Nutze „Anführungszeichen“ für exakte Phrasen, ein * am Ende für Präfixe und AND / OR / NOT zum Kombinieren. Diakritika werden ignoriert: „scoala“ findet „școală“. Grenze mit den Typ-Chips, Min.-/Max.-Größe, Ortsfilter oder einer Gruppe ein. Doppelklick auf einen Treffer zeigt das Dokument mit hervorgehobenen Suchwörtern.",
          "hu": "Nyisd meg a RAG-fetch-et és írd be a keresést. Használj „idézőjelet” pontos kifejezéshez, záró *-ot előtaghoz, AND / OR / NOT-ot a kombináláshoz. Az ékezeteket figyelmen kívül hagyja: a „scoala” megtalálja a „școală” szót. Szűkíts a típus-címkékkel, a min. / max. mérettel, a helyszűrővel vagy egy csoporttal. Dupla kattintással megnyithatod a dokumentumot a keresett szavak kiemelésével."}},
{"img": 3,
 "title": {"en": "3 · Make a group", "ro": "3 · Fă un grup", "fr": "3 · Crée un groupe", "it": "3 · Crea un gruppo", "es": "3 · Crea un grupo", "pt": "3 · Cria um grupo", "de": "3 · Erstelle eine Gruppe", "hu": "3 · Hozz létre egy csoportot"},
 "cap": {"en": "Tick the files you need and create a reusable group — your index for a task.", "ro": "Bifează fișierele de care ai nevoie și creează un grup reutilizabil — indexul tău pentru un task.", "fr": "Coche les fichiers voulus et crée un groupe réutilisable — ton index pour une tâche.", "it": "Spunta i file che ti servono e crea un gruppo riutilizzabile: il tuo indice per un compito.", "es": "Marca los archivos que necesitas y crea un grupo reutilizable: tu índice para una tarea.", "pt": "Assinala os ficheiros de que precisas e cria um grupo reutilizável — o teu índice para uma tarefa.", "de": "Hake die gewünschten Dateien an und erstelle eine wiederverwendbare Gruppe — dein Index für eine Aufgabe.", "hu": "Jelöld ki a szükséges fájlokat és hozz létre újrafelhasználható csoportot — az indexed egy feladathoz."},
 "text": {"en": "Tick the files you want, type a group name and press “Create index from selected files”. A group is a reusable list (a project, a theme, the result of a query) stored in groups.json. You can later add folders or glob patterns to it from the command line, and reuse it for as many tasks as you like.",
          "ro": "Bifează fișierele dorite, scrie numele grupului și apasă „Creează index din fișierele selectate”. Un grup e o listă reutilizabilă (un proiect, o temă, rezultatul unei căutări) păstrată în groups.json. Mai târziu îi poți adăuga foldere sau tipare glob din linia de comandă și îl poți refolosi pentru oricâte taskuri vrei.",
          "fr": "Coche les fichiers voulus, saisis un nom de groupe et appuie sur « Créer un index depuis la sélection ». Un groupe est une liste réutilisable (un projet, un thème, le résultat d’une recherche) stockée dans groups.json. Tu pourras y ajouter des dossiers ou des motifs glob en ligne de commande et le réutiliser pour autant de tâches que tu veux.",
          "it": "Spunta i file desiderati, scrivi il nome del gruppo e premi «Crea un indice dai file selezionati». Un gruppo è un elenco riutilizzabile (un progetto, un tema, il risultato di una ricerca) salvato in groups.json. In seguito potrai aggiungere cartelle o pattern glob dalla riga di comando e riusarlo per quanti compiti vuoi.",
          "es": "Marca los archivos que quieras, escribe un nombre de grupo y pulsa «Crear índice con los archivos seleccionados». Un grupo es una lista reutilizable (un proyecto, un tema, el resultado de una búsqueda) guardada en groups.json. Después podrás añadirle carpetas o patrones glob desde la línea de comandos y reutilizarlo en todas las tareas que quieras.",
          "pt": "Assinala os ficheiros que queres, escreve o nome do grupo e carrega em «Criar índice com os ficheiros selecionados». Um grupo é uma lista reutilizável (um projeto, um tema, o resultado de uma pesquisa) guardada em groups.json. Depois podes acrescentar-lhe pastas ou padrões glob na linha de comandos e reutilizá-lo em quantas tarefas quiseres.",
          "de": "Hake die gewünschten Dateien an, gib einen Gruppennamen ein und klicke auf „Index aus ausgewählten Dateien erstellen“. Eine Gruppe ist eine wiederverwendbare Liste (ein Projekt, ein Thema, ein Suchergebnis), gespeichert in groups.json. Später kannst du per Kommandozeile Ordner oder Glob-Muster hinzufügen und sie für beliebig viele Aufgaben wiederverwenden.",
          "hu": "Jelöld ki a kívánt fájlokat, írd be a csoport nevét, és nyomd meg az „Index létrehozása a kijelölt fájlokból” gombot. A csoport egy újrafelhasználható lista (projekt, téma, keresés eredménye), a groups.json fájlban tárolva. Később parancssorból mappákat vagy glob mintákat adhatsz hozzá, és annyi feladathoz használhatod, amennyihez csak szeretnéd."}},
{"img": 4,
 "title": {"en": "4 · Ask RAG-CARTA", "ro": "4 · Întreabă RAG-CARTA", "fr": "4 · Interroge RAG-CARTA", "it": "4 · Interroga RAG-CARTA", "es": "4 · Pregunta a RAG-CARTA", "pt": "4 · Pergunta ao RAG-CARTA", "de": "4 · Frage RAG-CARTA", "hu": "4 · Kérdezd a RAG-CARTÁ-t"},
 "cap": {"en": "RAG-CARTA summarizes, extracts or synthesizes — every answer cites its sources.", "ro": "RAG-CARTA rezumă, extrage sau sintetizează — fiecare răspuns își citează sursele.", "fr": "RAG-CARTA résume, extrait ou synthétise — chaque réponse cite ses sources.", "it": "RAG-CARTA riassume, estrae o sintetizza: ogni risposta cita le sue fonti.", "es": "RAG-CARTA resume, extrae o sintetiza: cada respuesta cita sus fuentes.", "pt": "O RAG-CARTA resume, extrai ou sintetiza — cada resposta cita as suas fontes.", "de": "RAG-CARTA fasst zusammen, extrahiert oder synthetisiert — jede Antwort nennt ihre Quellen.", "hu": "A RAG-CARTA összefoglal, kinyer vagy szintetizál — minden válasz megadja a forrásait."},
 "text": {"en": "Press RAG-CARTA (top right). Pick a task — summarize, extract or synthesize — choose your selected files or a group, type your question in the “Message Colibri” box and press Run. Answers keep [S1], [S2] citations and end with the list of source files; tick “Save result” to keep them in outputs/. Start the engine with coli web for real answers; “Offline mode” is only a fast extractive fallback.",
          "ro": "Apasă RAG-CARTA (sus, dreapta). Alege un task — rezumă, extrage sau sintetizează — alege fișierele selectate sau un grup, scrie întrebarea în caseta „Message Colibri” și apasă Rulează. Răspunsurile păstrează citările [S1], [S2] și se încheie cu lista fișierelor-sursă; bifează „Salvează rezultatul” ca să le păstrezi în outputs/. Pornește motorul cu coli web pentru răspunsuri reale; „Mod offline” e doar o variantă extractivă rapidă de rezervă.",
          "fr": "Appuie sur RAG-CARTA (en haut à droite). Choisis une tâche — résumer, extraire ou synthétiser —, puis tes fichiers sélectionnés ou un groupe, écris ta question dans la zone « Message Colibri » et lance. Les réponses gardent les citations [S1], [S2] et finissent par la liste des fichiers sources ; coche « Enregistrer le résultat » pour le garder dans outputs/. Lance le moteur avec coli web pour de vraies réponses ; le « Mode hors ligne » n’est qu’un repli extractif rapide.",
          "it": "Premi RAG-CARTA (in alto a destra). Scegli un compito — riassumere, estrarre o sintetizzare —, poi i file selezionati o un gruppo, scrivi la domanda nella casella «Message Colibri» e premi Esegui. Le risposte mantengono le citazioni [S1], [S2] e finiscono con l’elenco dei file sorgente; spunta «Salva il risultato» per conservarle in outputs/. Avvia il motore con coli web per risposte vere; la «Modalità offline» è solo un ripiego estrattivo veloce.",
          "es": "Pulsa RAG-CARTA (arriba a la derecha). Elige una tarea —resumir, extraer o sintetizar—, luego tus archivos seleccionados o un grupo, escribe la pregunta en el cuadro «Message Colibri» y pulsa Ejecutar. Las respuestas mantienen las citas [S1], [S2] y terminan con la lista de archivos de origen; marca «Guardar el resultado» para conservarlas en outputs/. Arranca el motor con coli web para respuestas reales; el «Modo sin conexión» es solo un recurso extractivo rápido.",
          "pt": "Carrega em RAG-CARTA (em cima, à direita). Escolhe uma tarefa — resumir, extrair ou sintetizar —, depois os ficheiros selecionados ou um grupo, escreve a pergunta na caixa «Message Colibri» e carrega em Executar. As respostas mantêm as citações [S1], [S2] e terminam com a lista dos ficheiros de origem; assinala «Guardar o resultado» para os manter em outputs/. Arranca o motor com coli web para respostas reais; o «Modo offline» é apenas um recurso extrativo rápido.",
          "de": "Klicke oben rechts auf RAG-CARTA. Wähle eine Aufgabe — zusammenfassen, extrahieren oder synthetisieren —, dann deine ausgewählten Dateien oder eine Gruppe, tippe deine Frage ins Feld „Message Colibri“ und klicke auf Starten. Antworten behalten die Zitate [S1], [S2] und enden mit der Liste der Quelldateien; mit „Ergebnis speichern“ landen sie in outputs/. Starte die Engine mit coli web für echte Antworten; der „Offline-Modus“ ist nur ein schneller extraktiver Notbehelf.",
          "hu": "Nyomd meg a RAG-CARTA gombot (jobb felül). Válassz feladatot — összefoglalás, kinyerés vagy szintézis —, majd a kijelölt fájlokat vagy egy csoportot, írd be a kérdést a „Message Colibri” mezőbe, és indítsd el. A válaszok megtartják az [S1], [S2] hivatkozásokat, és a forrásfájlok listájával végződnek; az „Eredmény mentése” bejelölésével az outputs/ mappába kerülnek. Valódi válaszokhoz indítsd a motort a coli web paranccsal; az „Offline mód” csak gyors, kivonatoló tartalék."}},
]
END = {  # closing card: command line + portable
 "title": {"en": "5 · Command line and portable", "ro": "5 · Linia de comandă și versiunea portabilă", "fr": "5 · Ligne de commande et version portable", "it": "5 · Riga di comando e versione portatile", "es": "5 · Línea de comandos y versión portátil", "pt": "5 · Linha de comandos e versão portátil", "de": "5 · Kommandozeile und portable Version", "hu": "5 · Parancssor és hordozható változat"},
 "cap": {"en": "Everything also works from the command line — and as a portable app.", "ro": "Totul merge și din linia de comandă — și ca aplicație portabilă.", "fr": "Tout fonctionne aussi en ligne de commande — et en version portable.", "it": "Tutto funziona anche da riga di comando — e come app portatile.", "es": "Todo funciona también desde la línea de comandos — y como app portátil.", "pt": "Tudo funciona também na linha de comandos — e como app portátil.", "de": "Alles funktioniert auch per Kommandozeile — und als portable App.", "hu": "Minden működik parancssorból is — és hordozható alkalmazásként."},
 "text": {"en": "The same features are available as cartacolibri commands (see below). The portable zip keeps your index, groups and outputs in its own Data folder: unzip, run the launcher, done.",
          "ro": "Aceleași funcții sunt disponibile ca bomenzi cartacolibri (vezi mai jos). Arhiva portabilă ține indexul, grupurile și rezultatele în propriul folder Data: dezarhivezi, rulezi lansatorul, gata.".replace("bomenzi", "comenzi"),
          "fr": "Les mêmes fonctions existent en commandes cartacolibri (voir ci-dessous). Le zip portable garde ton index, tes groupes et tes résultats dans son propre dossier Data : décompresse, lance, c’est prêt.",
          "it": "Le stesse funzioni sono disponibili come comandi cartacolibri (vedi sotto). Lo zip portatile conserva indice, gruppi e risultati nella propria cartella Data: decomprimi, avvia, fatto.",
          "es": "Las mismas funciones están disponibles como comandos cartacolibri (abajo). El zip portátil guarda tu índice, tus grupos y tus resultados en su propia carpeta Data: descomprime, ejecuta y listo.",
          "pt": "As mesmas funções estão disponíveis como comandos cartacolibri (abaixo). O zip portátil guarda o teu índice, os grupos e os resultados na sua própria pasta Data: descomprime, executa e já está.",
          "de": "Dieselben Funktionen gibt es als cartacolibri-Befehle (siehe unten). Das portable Zip speichert Index, Gruppen und Ergebnisse im eigenen Ordner Data: entpacken, Starter ausführen, fertig.",
          "hu": "Ugyanezek a funkciók cartacolibri parancsokként is elérhetők (lásd alább). A hordozható zip az indexet, a csoportokat és az eredményeket a saját Data mappájában tartja: kicsomagolod, elindítod, kész."}}
CMD = ["cartacolibri add ~/Documents", 'cartacolibri search "payment OR pay*" --ext txt,md', "cartacolibri group set Contracts ~/Documents/contracts",
       'cartacolibri synthesize "payment terms" --group Contracts --save', "cartacolibri watch --interval 30      # keep the index current",
       "cartacolibri grab https://example.org/article      # web-grab", "cartacolibri bookmarklet               # one-click save from any browser"]
TITLE_CARD = {"en": "Tutorial", "ro": "Tutorial", "fr": "Tutoriel", "it": "Tutorial", "es": "Tutorial", "pt": "Tutorial", "de": "Anleitung", "hu": "Útmutató"}
PDFSEARCH = None


def wrap(c, text, font, size, width):
    words, lines, cur = text.split(), [], ""
    for w in words:
        t = (cur + " " + w).strip()
        if c.stringWidth(t, font, size) <= width: cur = t
        else: lines.append(cur); cur = w
    return lines + ([cur] if cur else [])


def pdf(lang):
    dj = "/usr/share/fonts/truetype/dejavu/"
    pdfmetrics.registerFont(TTFont("DJ", dj + "DejaVuSans.ttf")); pdfmetrics.registerFont(TTFont("DJB", dj + "DejaVuSans-Bold.ttf")); pdfmetrics.registerFont(TTFont("DJM", dj + "DejaVuSansMono.ttf"))
    out = f"{ROOT}/docs/cartacolibri/CartaColibri-Tutorial.{lang}.pdf"
    c = canvas.Canvas(out, pagesize=A4); W, H = A4; M = 20 * mm
    c.setTitle(f"CartaColibri — {TITLE_CARD[lang]} ({NAMES[lang]})"); c.setAuthor("Alexio · CartaColibri")
    logo = ImageReader(f"{ROOT}/assets/cartacolibri-logo.png")
    c.drawImage(logo, W / 2 - 55 * mm, H - 120 * mm, 110 * mm, 110 * mm * 1.0, preserveAspectRatio=True, mask="auto")
    c.setFont("DJB", 26); c.drawCentredString(W / 2, H - 135 * mm, f"CartaColibri — {TITLE_CARD[lang]}")
    c.setFont("DJ", 12); y = H - 150 * mm
    for ln in wrap(c, X["tagline"][lang], "DJ", 12, W - 2 * M): c.drawCentredString(W / 2, y, ln); y -= 6 * mm
    y -= 6 * mm; c.setFont("DJ", 10.5)
    for ln in wrap(c, X["intro"][lang].replace("**", ""), "DJ", 10.5, W - 2 * M): c.drawString(M, y, ln); y -= 5.4 * mm
    c.showPage()
    for st in S:
        c.setFont("DJB", 18); c.drawString(M, H - 25 * mm, st["title"][lang])
        img = f"{ROOT}/docs/cartacolibri/img/{lang}-{st['img']}.png"; iw, ih = Image.open(img).size
        w = W - 2 * M; h = w * ih / iw
        c.drawImage(ImageReader(img), M, H - 32 * mm - h, w, h)
        c.setStrokeColorRGB(.3, .3, .3); c.rect(M, H - 32 * mm - h, w, h)
        y = H - 32 * mm - h - 9 * mm; c.setFont("DJ", 10.5)
        for ln in wrap(c, st["text"][lang], "DJ", 10.5, W - 2 * M): c.drawString(M, y, ln); y -= 5.4 * mm
        c.setFont("DJ", 8); c.setFillGray(.5); c.drawRightString(W - M, 12 * mm, "CartaColibri · github.com/chiuta/CartaColibri"); c.setFillGray(0)
        c.showPage()
    c.setFont("DJB", 18); c.drawString(M, H - 25 * mm, END["title"][lang]); y = H - 38 * mm; c.setFont("DJ", 10.5)
    for ln in wrap(c, END["text"][lang], "DJ", 10.5, W - 2 * M): c.drawString(M, y, ln); y -= 5.4 * mm
    y -= 6 * mm; c.setFont("DJM", 9.5)
    for cmd in CMD: c.drawString(M, y, cmd); y -= 6 * mm
    c.showPage(); c.save(); return out


def ts(sec):
    return f"{int(sec // 3600):02}:{int(sec % 3600 // 60):02}:{int(sec % 60):02},{int(sec % 1 * 1000):03}"


def video():
    D = 7.0                                   # seconds per slide
    media = f"{ROOT}/site-carta/media"; os.makedirs(media, exist_ok=True)
    tmp = tempfile.mkdtemp(); slides = []
    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 54)
    card = Image.new("RGB", (1280, 720), (15, 17, 21)); lg = Image.open(f"{ROOT}/assets/cartacolibri-logo.png").convert("RGBA"); lg.thumbnail((520, 520))
    card.paste(lg, ((1280 - lg.width) // 2, 70), lg); d = ImageDraw.Draw(card)
    t = "CartaColibri — Local AI Library"; w = d.textlength(t, font=font); d.text(((1280 - w) / 2, 620), t, font=font, fill=(238, 241, 246))
    p = f"{tmp}/s0.png"; card.save(p); slides.append(p)
    for i, st in enumerate(S, 1):
        im = Image.open(f"{ROOT}/docs/cartacolibri/img/en-{st['img']}.png").convert("RGB"); im = im.resize((1280, int(1280 * im.height / im.width)))
        bg = Image.new("RGB", (1280, 720), (15, 17, 21)); bg.paste(im, (0, (720 - im.height) // 2)); p = f"{tmp}/s{i}.png"; bg.save(p); slides.append(p)
    steps = [None] + S + [END]
    p = f"{tmp}/s{len(slides)}.png"; end = Image.new("RGB", (1280, 720), (15, 17, 21)); d = ImageDraw.Draw(end); f2 = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", 30)
    for k, cmd in enumerate(CMD): d.text((70, 120 + k * 80), cmd[:62], font=f2, fill=(95, 215, 230))
    end.save(p); slides.append(p)
    caps = {l: [X["tagline"][l]] + [s["cap"][l] for s in S] + [END["cap"][l]] for l in LANGS}
    srts = []
    for l in LANGS:
        with open(f"{media}/demo.{l}.vtt", "w", encoding="utf-8") as v, open(f"{tmp}/{l}.srt", "w", encoding="utf-8") as r:
            v.write("WEBVTT\n\n")
            for i, cap in enumerate(caps[l]):
                a, b = i * D + .4, (i + 1) * D - .4
                r.write(f"{i+1}\n{ts(a)} --> {ts(b)}\n{cap}\n\n"); v.write(f"{ts(a).replace(',', '.')} --> {ts(b).replace(',', '.')}\n{cap}\n\n")
        srts.append(f"{tmp}/{l}.srt")
    lst = f"{tmp}/list.txt"
    with open(lst, "w") as f:
        for s in slides: f.write(f"file '{s}'\nduration {D}\n")
        f.write(f"file '{slides[-1]}'\n")
    code = dict(en="eng", ro="ron", fr="fra", it="ita", es="spa", pt="por", de="deu", hu="hun")
    cmd = ["ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", lst]
    for s in srts: cmd += ["-i", s]
    cmd += ["-map", "0:v"] + sum([["-map", f"{i+1}:0"] for i in range(len(srts))], [])
    cmd += ["-c:v", "libx264", "-preset", "veryfast", "-crf", "28", "-pix_fmt", "yuv420p", "-r", "15", "-c:s", "mov_text"]
    for i, l in enumerate(LANGS): cmd += [f"-metadata:s:s:{i}", f"language={code[l]}", f"-metadata:s:s:{i}", f"title={NAMES[l]}"]
    cmd += ["-movflags", "+faststart", f"{media}/cartacolibri-demo.mp4"]
    subprocess.run(cmd, check=True)
    return f"{media}/cartacolibri-demo.mp4"


if __name__ == "__main__":
    for l in LANGS: print(pdf(l))
    print(video())
