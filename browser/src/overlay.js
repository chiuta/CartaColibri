(() => {
  if (window.top !== window) return;
  const N = "__NONCE__", T = __TAB__;
  const post = window.ipc && window.ipc.postMessage.bind(window.ipc);
  if (!post) return;
  const D = {
    en: { hl: "Highlight selection", watch: "Watch this page for changes", hist: "Searchable history (on/off)", sall: "Summarize all tabs", xall: "Extract from all tabs (comma-separated fields)", snap: "Snapshot", diffh: "What changed", asklib: "Ask your whole library: start with ?", save: "Save", ask: "Ask", addr: "Address, or words to search your library", q: "Ask about this page…", think: "Thinking…", newtab: "New tab (Ctrl+T)", sg: "Save all tabs as a library group", aa: "Ask all open tabs", inlib: "In your library", changed: "differs from the saved copy", related: "Related in your library", refresh: "Refresh copy", qall: "Question across all tabs…" },
    ro: { hl: "Evidențiază selecția", watch: "Urmărește schimbările paginii", hist: "Istoric căutabil (pornit/oprit)", sall: "Rezumă toate taburile", xall: "Extrage din toate taburile (câmpuri separate prin virgulă)", snap: "Instantaneu", diffh: "Ce s-a schimbat", asklib: "Întreabă toată biblioteca: începe cu ?", save: "Salvează", ask: "Întreabă", addr: "Adresă sau cuvinte de căutat în bibliotecă", q: "Întreabă despre pagina asta…", think: "Se gândește…", newtab: "Tab nou (Ctrl+T)", sg: "Salvează toate taburile ca grup în bibliotecă", aa: "Întreabă toate taburile deschise", inlib: "În biblioteca ta", changed: "diferă de copia salvată", related: "Înrudite în biblioteca ta", refresh: "Reîmprospătează copia", qall: "Întrebare peste toate taburile…" },
    fr: { hl: "Surligner la sélection", watch: "Surveiller les changements de cette page", hist: "Historique consultable (oui/non)", sall: "Résumer tous les onglets", xall: "Extraire de tous les onglets (champs séparés par des virgules)", snap: "Instantané", diffh: "Ce qui a changé", asklib: "Interroger toute ta bibliothèque : commence par ?", save: "Enregistrer", ask: "Demander", addr: "Adresse, ou mots à chercher dans ta bibliothèque", q: "Pose une question sur cette page…", think: "Réflexion…", newtab: "Nouvel onglet (Ctrl+T)", sg: "Enregistrer tous les onglets comme groupe", aa: "Interroger tous les onglets ouverts", inlib: "Dans ta bibliothèque", changed: "diffère de la copie enregistrée", related: "Liés dans ta bibliothèque", refresh: "Actualiser la copie", qall: "Question sur tous les onglets…" },
    it: { hl: "Evidenzia la selezione", watch: "Controlla le modifiche della pagina", hist: "Cronologia ricercabile (sì/no)", sall: "Riassumi tutte le schede", xall: "Estrai da tutte le schede (campi separati da virgole)", snap: "Istantanea", diffh: "Cosa è cambiato", asklib: "Chiedi a tutta la biblioteca: inizia con ?", save: "Salva", ask: "Chiedi", addr: "Indirizzo, o parole da cercare nella biblioteca", q: "Chiedi di questa pagina…", think: "Sto pensando…", newtab: "Nuova scheda (Ctrl+T)", sg: "Salva tutte le schede come gruppo", aa: "Interroga tutte le schede aperte", inlib: "Nella tua biblioteca", changed: "diversa dalla copia salvata", related: "Correlati nella tua biblioteca", refresh: "Aggiorna la copia", qall: "Domanda su tutte le schede…" },
    es: { hl: "Resaltar la selección", watch: "Vigilar los cambios de esta página", hist: "Historial con búsqueda (sí/no)", sall: "Resumir todas las pestañas", xall: "Extraer de todas las pestañas (campos separados por comas)", snap: "Instantánea", diffh: "Qué ha cambiado", asklib: "Pregunta a toda tu biblioteca: empieza con ?", save: "Guardar", ask: "Preguntar", addr: "Dirección, o palabras para buscar en tu biblioteca", q: "Pregunta sobre esta página…", think: "Pensando…", newtab: "Nueva pestaña (Ctrl+T)", sg: "Guardar todas las pestañas como grupo", aa: "Preguntar a todas las pestañas", inlib: "En tu biblioteca", changed: "difiere de la copia guardada", related: "Relacionado en tu biblioteca", refresh: "Actualizar la copia", qall: "Pregunta sobre todas las pestañas…" },
    pt: { hl: "Destacar a seleção", watch: "Vigiar alterações desta página", hist: "Histórico pesquisável (ligado/desligado)", sall: "Resumir todos os separadores", xall: "Extrair de todos os separadores (campos separados por vírgulas)", snap: "Instantâneo", diffh: "O que mudou", asklib: "Pergunta a toda a biblioteca: começa com ?", save: "Guardar", ask: "Perguntar", addr: "Endereço, ou palavras para pesquisar na biblioteca", q: "Pergunta sobre esta página…", think: "A pensar…", newtab: "Novo separador (Ctrl+T)", sg: "Guardar todos os separadores como grupo", aa: "Perguntar a todos os separadores", inlib: "Na tua biblioteca", changed: "difere da cópia guardada", related: "Relacionado na tua biblioteca", refresh: "Atualizar a cópia", qall: "Pergunta sobre todos os separadores…" },
    de: { hl: "Auswahl markieren", watch: "Seite auf Änderungen beobachten", hist: "Durchsuchbarer Verlauf (an/aus)", sall: "Alle Tabs zusammenfassen", xall: "Aus allen Tabs extrahieren (Felder mit Komma)", snap: "Schnappschuss", diffh: "Was sich geändert hat", asklib: "Ganze Bibliothek fragen: mit ? beginnen", save: "Speichern", ask: "Fragen", addr: "Adresse oder Suchwörter für deine Bibliothek", q: "Frage zu dieser Seite…", think: "Denke nach…", newtab: "Neuer Tab (Strg+T)", sg: "Alle Tabs als Gruppe speichern", aa: "Alle offenen Tabs befragen", inlib: "In deiner Bibliothek", changed: "weicht von der gespeicherten Kopie ab", related: "Verwandt in deiner Bibliothek", refresh: "Kopie aktualisieren", qall: "Frage über alle Tabs…" },
    hu: { hl: "Kijelölés kiemelése", watch: "Az oldal változásainak figyelése", hist: "Kereshető előzmények (be/ki)", sall: "Az összes lap összefoglalása", xall: "Kinyerés az összes lapból (vesszővel elválasztott mezők)", snap: "Pillanatkép", diffh: "Mi változott", asklib: "Kérdezd az egész könyvtárat: kezdd ?-lel", save: "Mentés", ask: "Kérdés", addr: "Cím, vagy keresőszavak a könyvtárhoz", q: "Kérdezz erről az oldalról…", think: "Gondolkodik…", newtab: "Új lap (Ctrl+T)", sg: "Az összes lap mentése csoportként", aa: "Kérdés az összes nyitott laphoz", inlib: "A könyvtáradban", changed: "eltér a mentett példánytól", related: "Kapcsolódó a könyvtáradban", refresh: "Példány frissítése", qall: "Kérdés az összes laphoz…" },
  };
  const X = {
    en: { tourT: "Welcome — a 30-second tour", next: "Next", back: "Back", skip: "Skip", done: "Got it", t1: "Press 💾 (or Ctrl+S) to keep this page in your library. Saving again refreshes the same file.", t2: "Press ✨ (or Ctrl+Shift+A) and ask anything about this page. The answer comes from your local Colibrì.", t3: "Select some text and press 🖍 (or Ctrl+H). It stays highlighted whenever you come back.", t4: "Press 👁 to watch the page: you will see when it changes. Press F1 any time for help.", helpT: "Help", sc: "Keyboard shortcuts", btn: "What the buttons do", esc: "Close panels", reopen: "Reopen closed tab", tabsNav: "New tab · close tab · next tab", ab: "Address bar", cc: "Ctrl+click or middle-click a link: open it in a new tab", saved: "Saved ✔ — find it in your library: {p}", hlok: "Highlighted ✔ — kept with this page", hlnone: "Select some text first, then press 🖍", watchok: "Watching ✔ — you will see when this page changes", histon: "History on — visited pages stay on this computer only", histoff: "History off — nothing new is recorded", tabsok: "{n} tabs saved together as one group ✔", nohttp: "Only web pages (http/https) can be saved or asked about.", nopage: "This page has no readable text yet. Wait for it to finish loading, or select some text and try again.", gw: "Your local Colibrì is not answering. Start it, or relaunch the browser with --mock to get offline demo answers (made up by a built-in mock, not by a real model).", oops: "Something went wrong: {e}", mockn: "Demo mode (--mock): answers come from a built-in mock, not from a real model.", sTitle: "Your library", sRecent: "Recently saved", sWatch: "Watched pages", sEmpty: "Nothing saved yet — open a page and press 💾.", sNoWatch: "No watched pages yet — press 👁 on a page.", sHint: "Tip: type ? and a question to ask your whole library.", helpb: "Help (F1)" },
    ro: { tourT: "Bun venit — un tur de 30 de secunde", next: "Înainte", back: "Înapoi", skip: "Sari peste", done: "Am înțeles", t1: "Apasă 💾 (sau Ctrl+S) ca să păstrezi pagina în bibliotecă. Dacă salvezi din nou, același fișier se reîmprospătează.", t2: "Apasă ✨ (sau Ctrl+Shift+A) și întreabă orice despre pagină. Răspunsul vine de la Colibrì-ul tău local.", t3: "Selectează un text și apasă 🖍 (sau Ctrl+H). Rămâne evidențiat de fiecare dată când revii.", t4: "Apasă 👁 ca să urmărești pagina: vei vedea când se schimbă. Apasă F1 oricând pentru ajutor.", helpT: "Ajutor", sc: "Scurtături de tastatură", btn: "Ce face fiecare buton", esc: "Închide panourile", reopen: "Redeschide ultimul tab închis", tabsNav: "Tab nou · închide tab · tabul următor", ab: "Bara de adrese", cc: "Ctrl+clic sau clic de mijloc pe un link: îl deschide într-un tab nou", saved: "Salvat ✔ — îl găsești în bibliotecă: {p}", hlok: "Evidențiat ✔ — păstrat împreună cu pagina", hlnone: "Selectează mai întâi un text, apoi apasă 🖍", watchok: "Urmărit ✔ — vei vedea când se schimbă pagina", histon: "Istoric pornit — paginile vizitate rămân doar pe acest calculator", histoff: "Istoric oprit — nu se mai înregistrează nimic nou", tabsok: "{n} taburi salvate împreună ca un grup ✔", nohttp: "Doar paginile web (http/https) pot fi salvate sau întrebate.", nopage: "Pagina nu are încă text lizibil. Așteaptă să se încarce sau selectează un text și încearcă din nou.", gw: "Colibrì-ul tău local nu răspunde. Pornește-l sau relansează browserul cu --mock pentru răspunsuri demo offline (inventate de un model simulat, nu de unul real).", oops: "A apărut o problemă: {e}", mockn: "Mod demo (--mock): răspunsurile vin de la un model simulat, nu de la unul real.", sTitle: "Biblioteca ta", sRecent: "Salvate recent", sWatch: "Pagini urmărite", sEmpty: "Nimic salvat încă — deschide o pagină și apasă 💾.", sNoWatch: "Nicio pagină urmărită încă — apasă 👁 pe o pagină.", sHint: "Sfat: scrie ? și o întrebare ca să întrebi toată biblioteca.", helpb: "Ajutor (F1)" },
    fr: { tourT: "Bienvenue — une visite de 30 secondes", next: "Suivant", back: "Retour", skip: "Passer", done: "Compris", t1: "Appuie sur 💾 (ou Ctrl+S) pour garder cette page dans ta bibliothèque. Enregistrer à nouveau met à jour le même fichier.", t2: "Appuie sur ✨ (ou Ctrl+Maj+A) et pose n'importe quelle question sur la page. La réponse vient de ton Colibrì local.", t3: "Sélectionne du texte et appuie sur 🖍 (ou Ctrl+H). Il reste surligné à chaque retour.", t4: "Appuie sur 👁 pour surveiller la page : tu verras quand elle change. Appuie sur F1 à tout moment pour l'aide.", helpT: "Aide", sc: "Raccourcis clavier", btn: "À quoi servent les boutons", esc: "Fermer les panneaux", reopen: "Rouvrir l'onglet fermé", tabsNav: "Nouvel onglet · fermer · onglet suivant", ab: "Barre d'adresse", cc: "Ctrl+clic ou clic du milieu sur un lien : l'ouvre dans un nouvel onglet", saved: "Enregistré ✔ — retrouve-le dans ta bibliothèque : {p}", hlok: "Surligné ✔ — conservé avec cette page", hlnone: "Sélectionne d'abord du texte, puis appuie sur 🖍", watchok: "Surveillance active ✔ — tu verras quand cette page change", histon: "Historique activé — les pages visitées restent sur cet ordinateur uniquement", histoff: "Historique désactivé — rien de nouveau n'est enregistré", tabsok: "{n} onglets enregistrés ensemble en un groupe ✔", nohttp: "Seules les pages web (http/https) peuvent être enregistrées ou interrogées.", nopage: "Cette page n'a pas encore de texte lisible. Attends la fin du chargement, ou sélectionne du texte et réessaie.", gw: "Ton Colibrì local ne répond pas. Démarre-le, ou relance le navigateur avec --mock pour des réponses de démonstration hors ligne (simulées, pas d'un vrai modèle).", oops: "Un problème est survenu : {e}", mockn: "Mode démo (--mock) : les réponses viennent d'une simulation, pas d'un vrai modèle.", sTitle: "Ta bibliothèque", sRecent: "Enregistrées récemment", sWatch: "Pages surveillées", sEmpty: "Rien d'enregistré pour l'instant — ouvre une page et appuie sur 💾.", sNoWatch: "Aucune page surveillée — appuie sur 👁 sur une page.", sHint: "Astuce : tape ? et une question pour interroger toute ta bibliothèque.", helpb: "Aide (F1)" },
    it: { tourT: "Benvenuto — un giro di 30 secondi", next: "Avanti", back: "Indietro", skip: "Salta", done: "Capito", t1: "Premi 💾 (o Ctrl+S) per tenere questa pagina nella tua biblioteca. Salvando di nuovo si aggiorna lo stesso file.", t2: "Premi ✨ (o Ctrl+Maiusc+A) e chiedi qualsiasi cosa sulla pagina. La risposta arriva dal tuo Colibrì locale.", t3: "Seleziona del testo e premi 🖍 (o Ctrl+H). Resta evidenziato ogni volta che torni.", t4: "Premi 👁 per controllare la pagina: vedrai quando cambia. Premi F1 in qualsiasi momento per l'aiuto.", helpT: "Aiuto", sc: "Scorciatoie da tastiera", btn: "A cosa servono i pulsanti", esc: "Chiudi i pannelli", reopen: "Riapri la scheda chiusa", tabsNav: "Nuova scheda · chiudi · scheda successiva", ab: "Barra degli indirizzi", cc: "Ctrl+clic o clic centrale su un link: lo apre in una nuova scheda", saved: "Salvato ✔ — lo trovi nella tua biblioteca: {p}", hlok: "Evidenziato ✔ — conservato con questa pagina", hlnone: "Prima seleziona del testo, poi premi 🖍", watchok: "Controllo attivo ✔ — vedrai quando la pagina cambia", histon: "Cronologia attiva — le pagine visitate restano solo su questo computer", histoff: "Cronologia disattivata — non si registra più nulla", tabsok: "{n} schede salvate insieme come gruppo ✔", nohttp: "Si possono salvare o interrogare solo pagine web (http/https).", nopage: "Questa pagina non ha ancora testo leggibile. Aspetta che finisca di caricarsi, o seleziona del testo e riprova.", gw: "Il tuo Colibrì locale non risponde. Avvialo, oppure riavvia il browser con --mock per risposte dimostrative offline (simulate, non di un vero modello).", oops: "Qualcosa è andato storto: {e}", mockn: "Modalità demo (--mock): le risposte arrivano da una simulazione, non da un vero modello.", sTitle: "La tua biblioteca", sRecent: "Salvate di recente", sWatch: "Pagine controllate", sEmpty: "Ancora nulla di salvato — apri una pagina e premi 💾.", sNoWatch: "Nessuna pagina controllata — premi 👁 su una pagina.", sHint: "Suggerimento: scrivi ? e una domanda per interrogare tutta la biblioteca.", helpb: "Aiuto (F1)" },
    es: { tourT: "Te damos la bienvenida — un recorrido de 30 segundos", next: "Siguiente", back: "Atrás", skip: "Omitir", done: "Entendido", t1: "Pulsa 💾 (o Ctrl+S) para guardar esta página en tu biblioteca. Si guardas de nuevo, se actualiza el mismo archivo.", t2: "Pulsa ✨ (o Ctrl+Mayús+A) y pregunta lo que quieras sobre la página. La respuesta viene de tu Colibrì local.", t3: "Selecciona texto y pulsa 🖍 (o Ctrl+H). Se queda resaltado cada vez que vuelvas.", t4: "Pulsa 👁 para vigilar la página: verás cuándo cambia. Pulsa F1 en cualquier momento para ver la ayuda.", helpT: "Ayuda", sc: "Atajos de teclado", btn: "Qué hace cada botón", esc: "Cerrar paneles", reopen: "Reabrir la pestaña cerrada", tabsNav: "Nueva pestaña · cerrar · siguiente", ab: "Barra de direcciones", cc: "Ctrl+clic o clic central en un enlace: lo abre en una pestaña nueva", saved: "Guardado ✔ — lo encuentras en tu biblioteca: {p}", hlok: "Resaltado ✔ — se guarda con esta página", hlnone: "Primero selecciona texto y luego pulsa 🖍", watchok: "Vigilando ✔ — verás cuándo cambia esta página", histon: "Historial activado — las páginas visitadas se quedan solo en este equipo", histoff: "Historial desactivado — no se registra nada nuevo", tabsok: "{n} pestañas guardadas juntas como grupo ✔", nohttp: "Solo se pueden guardar o consultar páginas web (http/https).", nopage: "Esta página aún no tiene texto legible. Espera a que termine de cargar, o selecciona texto e inténtalo de nuevo.", gw: "Tu Colibrì local no responde. Arráncalo, o relanza el navegador con --mock para obtener respuestas de demostración sin conexión (simuladas, no de un modelo real).", oops: "Algo ha salido mal: {e}", mockn: "Modo demo (--mock): las respuestas vienen de una simulación, no de un modelo real.", sTitle: "Tu biblioteca", sRecent: "Guardadas recientemente", sWatch: "Páginas vigiladas", sEmpty: "Aún no hay nada guardado — abre una página y pulsa 💾.", sNoWatch: "Aún no vigilas ninguna página — pulsa 👁 en una página.", sHint: "Consejo: escribe ? y una pregunta para consultar toda tu biblioteca.", helpb: "Ayuda (F1)" },
    pt: { tourT: "Bem-vindo — uma visita de 30 segundos", next: "Seguinte", back: "Voltar", skip: "Saltar", done: "Percebi", t1: "Carrega em 💾 (ou Ctrl+S) para guardar esta página na tua biblioteca. Se guardares de novo, o mesmo ficheiro é atualizado.", t2: "Carrega em ✨ (ou Ctrl+Shift+A) e pergunta o que quiseres sobre a página. A resposta vem do teu Colibrì local.", t3: "Seleciona texto e carrega em 🖍 (ou Ctrl+H). Fica destacado sempre que voltares.", t4: "Carrega em 👁 para vigiar a página: vais ver quando mudar. Carrega em F1 a qualquer momento para ajuda.", helpT: "Ajuda", sc: "Atalhos de teclado", btn: "O que faz cada botão", esc: "Fechar painéis", reopen: "Reabrir o separador fechado", tabsNav: "Novo separador · fechar · seguinte", ab: "Barra de endereços", cc: "Ctrl+clique ou clique do meio numa ligação: abre-a num novo separador", saved: "Guardado ✔ — encontras na tua biblioteca: {p}", hlok: "Destacado ✔ — guardado com esta página", hlnone: "Seleciona primeiro algum texto e depois carrega em 🖍", watchok: "A vigiar ✔ — vais ver quando esta página mudar", histon: "Histórico ligado — as páginas visitadas ficam só neste computador", histoff: "Histórico desligado — nada de novo é registado", tabsok: "{n} separadores guardados juntos como grupo ✔", nohttp: "Só se podem guardar ou consultar páginas web (http/https).", nopage: "Esta página ainda não tem texto legível. Espera que acabe de carregar, ou seleciona texto e tenta de novo.", gw: "O teu Colibrì local não responde. Inicia-o, ou reinicia o navegador com --mock para respostas de demonstração offline (simuladas, não de um modelo real).", oops: "Algo correu mal: {e}", mockn: "Modo demo (--mock): as respostas vêm de uma simulação, não de um modelo real.", sTitle: "A tua biblioteca", sRecent: "Guardadas recentemente", sWatch: "Páginas vigiadas", sEmpty: "Ainda nada guardado — abre uma página e carrega em 💾.", sNoWatch: "Ainda não vigias nenhuma página — carrega em 👁 numa página.", sHint: "Dica: escreve ? e uma pergunta para consultar toda a biblioteca.", helpb: "Ajuda (F1)" },
    de: { tourT: "Willkommen — eine Tour in 30 Sekunden", next: "Weiter", back: "Zurück", skip: "Überspringen", done: "Verstanden", t1: "Drücke 💾 (oder Strg+S), um diese Seite in deiner Bibliothek zu behalten. Erneutes Speichern aktualisiert dieselbe Datei.", t2: "Drücke ✨ (oder Strg+Umschalt+A) und frag alles zu dieser Seite. Die Antwort kommt von deinem lokalen Colibrì.", t3: "Markiere Text und drücke 🖍 (oder Strg+H). Er bleibt markiert, wenn du wiederkommst.", t4: "Drücke 👁, um die Seite zu beobachten: Du siehst, wann sie sich ändert. Mit F1 öffnest du jederzeit die Hilfe.", helpT: "Hilfe", sc: "Tastenkürzel", btn: "Was die Schaltflächen tun", esc: "Fenster schließen", reopen: "Geschlossenen Tab wieder öffnen", tabsNav: "Neuer Tab · Tab schließen · nächster Tab", ab: "Adressleiste", cc: "Strg+Klick oder Mittelklick auf einen Link: öffnet ihn in einem neuen Tab", saved: "Gespeichert ✔ — du findest es in deiner Bibliothek: {p}", hlok: "Markiert ✔ — wird mit dieser Seite gespeichert", hlnone: "Markiere zuerst Text und drücke dann 🖍", watchok: "Wird beobachtet ✔ — du siehst, wann sich die Seite ändert", histon: "Verlauf an — besuchte Seiten bleiben nur auf diesem Computer", histoff: "Verlauf aus — nichts Neues wird aufgezeichnet", tabsok: "{n} Tabs zusammen als Gruppe gespeichert ✔", nohttp: "Nur Webseiten (http/https) können gespeichert oder befragt werden.", nopage: "Diese Seite hat noch keinen lesbaren Text. Warte, bis sie fertig geladen ist, oder markiere Text und versuche es erneut.", gw: "Dein lokaler Colibrì antwortet nicht. Starte ihn oder starte den Browser mit --mock neu, um Offline-Demoantworten zu bekommen (simuliert, nicht von einem echten Modell).", oops: "Etwas ist schiefgelaufen: {e}", mockn: "Demomodus (--mock): Die Antworten stammen aus einer Simulation, nicht von einem echten Modell.", sTitle: "Deine Bibliothek", sRecent: "Zuletzt gespeichert", sWatch: "Beobachtete Seiten", sEmpty: "Noch nichts gespeichert — öffne eine Seite und drücke 💾.", sNoWatch: "Noch keine beobachteten Seiten — drücke 👁 auf einer Seite.", sHint: "Tipp: Tippe ? und eine Frage, um deine ganze Bibliothek zu fragen.", helpb: "Hilfe (F1)" },
    hu: { tourT: "Üdv — egy 30 másodperces körút", next: "Tovább", back: "Vissza", skip: "Kihagyom", done: "Értem", t1: "Nyomd meg a 💾 gombot (vagy Ctrl+S), hogy az oldal a könyvtáradba kerüljön. Újramentéskor ugyanaz a fájl frissül.", t2: "Nyomd meg a ✨ gombot (vagy Ctrl+Shift+A), és kérdezz bármit az oldalról. A választ a helyi Colibrìd adja.", t3: "Jelölj ki szöveget, és nyomd meg a 🖍 gombot (vagy Ctrl+H). Kiemelve marad, valahányszor visszatérsz.", t4: "A 👁 gombbal figyelheted az oldalt: látni fogod, ha változik. Az F1 bármikor megnyitja a súgót.", helpT: "Súgó", sc: "Billentyűparancsok", btn: "Mit csinálnak a gombok", esc: "Panelek bezárása", reopen: "Bezárt lap újranyitása", tabsNav: "Új lap · lap bezárása · következő lap", ab: "Címsor", cc: "Ctrl+kattintás vagy középső kattintás egy linkre: új lapon nyitja meg", saved: "Elmentve ✔ — megtalálod a könyvtáradban: {p}", hlok: "Kiemelve ✔ — az oldallal együtt megmarad", hlnone: "Előbb jelölj ki szöveget, aztán nyomd meg a 🖍 gombot", watchok: "Figyelem ✔ — látni fogod, ha az oldal megváltozik", histon: "Előzmények bekapcsolva — a meglátogatott oldalak csak ezen a gépen maradnak", histoff: "Előzmények kikapcsolva — semmi új nem kerül rögzítésre", tabsok: "{n} lap együtt mentve egy csoportként ✔", nohttp: "Csak weboldalakat (http/https) lehet menteni vagy megkérdezni.", nopage: "Ennek az oldalnak még nincs olvasható szövege. Várd meg, amíg betölt, vagy jelölj ki szöveget, és próbáld újra.", gw: "A helyi Colibrìd nem válaszol. Indítsd el, vagy indítsd újra a böngészőt a --mock kapcsolóval offline bemutató válaszokhoz (szimuláltak, nem valódi modelltől).", oops: "Valami elromlott: {e}", mockn: "Bemutató mód (--mock): a válaszok szimulációból jönnek, nem valódi modellből.", sTitle: "A könyvtárad", sRecent: "Nemrég mentett", sWatch: "Figyelt oldalak", sEmpty: "Még nincs mentve semmi — nyiss meg egy oldalt, és nyomd meg a 💾 gombot.", sNoWatch: "Még nincs figyelt oldal — nyomd meg a 👁 gombot egy oldalon.", sHint: "Tipp: írj ?-et és egy kérdést, hogy az egész könyvtáradtól kérdezz.", helpb: "Súgó (F1)" },
  };
  const lang = (navigator.language || "en").slice(0, 2);
  const L = Object.assign({}, D.en, X.en, D[lang] || {}, X[lang] || {});
  const web = () => /^https?:$/.test(location.protocol);
  const send = (o) => post(JSON.stringify(Object.assign({ n: N, t: T }, o)));
  const pageHtml = () => { try { return document.documentElement.outerHTML.slice(0, 3000000); } catch (e) { return ""; } };
  const pageText = () => String(getSelection()) || (document.body ? document.body.innerText : "");
  const el = (tag, txt, cls) => { const e = document.createElement(tag); if (txt != null) e.textContent = txt; if (cls) e.className = cls; return e; };
  let $, $hist, sh, panel, qrow, qin, urlIn, tabsEl, badge, toastEl, helpEl, tourEl, startEl, startIn, info = null, mock = false, tt, ti = -1;
  const show = (t, cls) => { panel.textContent = t; panel.className = cls || ""; panel.style.display = t ? "block" : "none"; };
  const toast = (t, cls) => {
    if (!sh) return;
    toastEl.textContent = t; toastEl.className = "on " + (cls || "ok");
    clearTimeout(tt); tt = setTimeout(() => { toastEl.className = ""; }, cls === "err" ? 8000 : 4500);
  };
  const friendly = (raw) => {
    raw = String(raw || "");
    if (/readable text/i.test(raw)) return L.nopage;
    if (/connect|refused|gateway|unreachable|error sending|timed out|timeout|dns|os error|127\.0\.0\.1|colibr/i.test(raw)) return L.gw + "\n(" + raw.slice(0, 200) + ")";
    return L.oops.replace("{e}", raw.slice(0, 300));
  };
  const act = function (cmd, q) {
    if (["save", "highlight", "watch", "ask"].includes(cmd) && !web()) return toast(L.nohttp, "err");
    if (cmd === "save") { show("…"); send({ cmd, url: location.href, title: document.title, text: pageText(), html: pageHtml() }); }
    else if (cmd === "highlight") { const sel = String(getSelection()).trim(); if (!sel) return toast(L.hlnone, "err"); send({ cmd, url: location.href, title: document.title, text: document.body ? document.body.innerText : "", sel }); applyHL([sel]); }
    else if (cmd === "watch") { send({ cmd, url: location.href }); }
    else if (cmd === "history") { send({ cmd }); }
    else if (cmd === "ask") { show(L.think); send({ cmd, q, url: location.href, title: document.title, text: pageText() }); }
    else if (cmd === "ask_tabs") { show(L.think); send({ cmd, q, task: arguments[2] || "synthesize" }); }
    else if (cmd === "save_tabs") { show("…"); send({ cmd }); }
  };
  const hidden = (name, fn) => Object.defineProperty(window, name + N, { enumerable: false, value: fn });
  hidden("__cc_", (m) => {
    if (!sh) return;
    if (!m.ok) return show(friendly(m.text), "err");
    const x = m.x;
    const f = { save: () => L.saved.replace("{p}", (x && x.path) || ""), hl: () => L.hlok, watch: () => L.watchok, hist: () => (x ? L.histon : L.histoff), tabs: () => L.tabsok.replace("{n}", x) }[m.k];
    if (f) { show(""); toast(f()); } else show(m.text, "ok");
  });
  hidden("__cc_do_", (c, q, k) => act(c, q, k));
  hidden("__cc_tabs_", (s) => {
    if (!sh) return;
    tabsEl.textContent = "";
    for (const t of s.tabs) {
      const e = el("div", null, "tab" + (t.id === s.active ? " on" : ""));
      const lab = el("span", (t.title || t.url || "·").slice(0, 28)); lab.title = t.url;
      const x = el("b", "×");
      e.onclick = () => send({ cmd: "activate", id: t.id }); x.onclick = (ev) => { ev.stopPropagation(); send({ cmd: "close", id: t.id }); };
      e.append(lab, x); tabsEl.append(e);
    }
    const plus = el("button", "+"); plus.title = L.newtab; plus.onclick = () => send({ cmd: "newtab" }); tabsEl.append(plus);
  });
  const applyHL = (list) => {
    for (const h of list || []) {
      if (!h || h.length < 3) continue;
      const w = document.createTreeWalker(document.body || document.documentElement, NodeFilter.SHOW_TEXT);
      let n;
      while ((n = w.nextNode())) {
        const raw = n.nodeValue.indexOf(h);
        if (raw >= 0 && n.parentElement && !/^(SCRIPT|STYLE|MARK)$/.test(n.parentElement.tagName)) {
          const r = document.createRange(); r.setStart(n, raw); r.setEnd(n, raw + h.length);
          const m = document.createElement("mark"); m.style.cssText = "background:#fdb93a88;color:inherit"; r.surroundContents(m); break;
        }
      }
    }
  };
  hidden("__cc_info_", (i) => {
    if (!sh) return; info = i; applyHL(i.highlights); $hist.className = i.history ? "go" : "";
    badge.textContent = ""; badge.style.display = "none";
    if (i.status) {
      badge.style.display = "inline-block";
      badge.textContent = "📚 " + L.inlib + (i.status.changed ? " · ⚠ " + L.changed : "");
      badge.className = i.status.changed ? "warn" : "";
      badge.title = i.status.path;
    } else if (i.related && i.related.length) {
      badge.style.display = "inline-block"; badge.textContent = "🔗 " + i.related.length + " " + L.related.toLowerCase(); badge.className = "";
    }
  });
  // ---- new-tab page (drawn inside the closed shadow root: the page itself learns nothing)
  const renderStart = (d) => {
    startEl.textContent = "";
    const box = el("div", null, "sbox");
    box.append(el("h1", "CartaColibri"));
    startIn = el("input"); startIn.placeholder = L.addr; startIn.spellcheck = false;
    startIn.onkeydown = (e) => { if (e.key === "Enter" && startIn.value.trim()) send({ cmd: "go", q: startIn.value }); };
    box.append(startIn, el("p", L.sHint, "hint"));
    const cols = el("div", null, "cols");
    const col = (title, items, empty, click) => {
      const c = el("div", null, "col"); c.append(el("h2", title));
      if (!items.length) c.append(el("p", empty, "hint"));
      for (const it of items) {
        const r = el("div", null, "item"); r.append(el("span", it.title || it.url)); if (it.title) r.append(el("small", it.url));
        r.title = it.url; r.onclick = () => click(it.url); c.append(r);
      }
      return c;
    };
    const go = (u) => send({ cmd: "go", q: u });
    cols.append(col("📚 " + L.sRecent, d.recent || [], L.sEmpty, go), col("👁 " + L.sWatch, (d.watched || []).map((u) => ({ url: u })), L.sNoWatch, go));
    box.append(cols); startEl.append(box);
    startEl.style.display = "block";
    setTimeout(() => startIn.focus(), 50);
  };
  // ---- first-run tour (4 steps, shown once) and help panel
  const steps = [["s", "t1", "💾 " + L.save], ["a", "t2", "✨ " + L.ask], ["hl", "t3", "🖍 " + L.hl], ["wt", "t4", "👁 " + L.watch]];
  const closeTour = () => { ti = -1; tourEl.className = ""; tourEl.textContent = ""; sh.querySelectorAll(".spot").forEach((b) => b.classList.remove("spot")); };
  const tour = (i) => {
    closeTour();
    if (i < 0 || i >= steps.length) return;
    ti = i; const [id, k, title] = steps[i];
    $(id).classList.add("spot");
    const head = el("div", null, "hd"); head.append(el("b", i === 0 ? L.tourT : title), (() => { const x = el("button", "×"); x.title = L.skip; x.onclick = closeTour; return x; })());
    const p = el("p", i === 0 ? title + " — " + L[k] : L[k]);
    const nav = el("div", null, "nav");
    const dots = el("span", steps.map((_, j) => (j === i ? "●" : "○")).join(" "), "dots");
    const sk = el("button", L.skip); sk.onclick = closeTour;
    nav.append(dots, sk);
    if (i > 0) { const b = el("button", L.back); b.onclick = () => tour(i - 1); nav.append(b); }
    const nx = el("button", i === steps.length - 1 ? L.done : L.next, "go"); nx.onclick = () => tour(i + 1); nav.append(nx);
    tourEl.className = "on"; tourEl.append(head, p, nav);
  };
  const helpOpen = () => helpEl.className === "on";
  const help = () => {
    if (helpOpen()) { helpEl.className = ""; return; }
    closeTour(); helpEl.textContent = "";
    const head = el("div", null, "hd"); const x = el("button", "×"); x.onclick = () => { helpEl.className = ""; };
    head.append(el("b", "❓ " + L.helpT), x); helpEl.append(head);
    const table = (title, rows) => {
      helpEl.append(el("h3", title));
      for (const [a, b] of rows) { const r = el("div", null, "hr"); r.append(el("kbd", a), el("span", b)); helpEl.append(r); }
    };
    table(L.sc, [["Ctrl+S", L.save], ["Ctrl+Shift+A", L.ask], ["Ctrl+H", L.hl], ["F1", L.helpT], ["Esc", L.esc], ["Ctrl+Shift+T", L.reopen], ["Ctrl+T · Ctrl+W · Ctrl+Tab", L.tabsNav], ["Ctrl+L", L.ab], ["Ctrl+🖱", L.cc]]);
    table(L.btn, [["💾", L.t1], ["✨", L.t2], ["🖍", L.t3], ["👁", L.t4], ["🕘", L.hist], ["🗂", L.sg], ["🗂✨", L.aa], ["🗂📝", L.sall], ["🗂📊", L.xall], ["📚 / 🔗", L.inlib + " · " + L.related], ["?…", L.asklib]]);
    if (mock) helpEl.append(el("p", "⚠ " + L.mockn, "hint"));
    const again = el("button", "🎓 " + L.tourT); again.onclick = () => { helpEl.className = ""; tour(0); }; helpEl.append(again);
    helpEl.className = "on";
  };
  hidden("__cc_boot_", (b) => {
    if (!sh) return;
    mock = !!b.mock;
    if (b.start && !web()) renderStart(b.start);
    if (b.tour) tour(0);
  });
  const mount = () => {
    const host = document.createElement("div");
    host.style.cssText = "all:initial;position:fixed;top:0;left:0;right:0;z-index:2147483647";
    sh = host.attachShadow({ mode: "closed" });
    sh.innerHTML = `<style>
      :host{--bg0:#0a0c10;--bg:#0f1115;--bg2:#171a21;--fg:#eef1f6;--mut:#9aa3b2;--bd:#2a2f3a;--acc:#5fd7e6;--bgb:#16324a;--sh:#0008}
      @media (prefers-color-scheme:light){:host{--bg0:#e4e7ee;--bg:#ffffff;--bg2:#f0f2f7;--fg:#1b1f2a;--mut:#5b6474;--bd:#cdd2dc;--acc:#0a7f8f;--bgb:#d6eef3;--sh:#0003}}
      *{box-sizing:border-box;font:13px system-ui,Segoe UI,sans-serif}
      #tabs,.bar,.row,#panel{position:relative;z-index:2}
      #tabs{display:flex;gap:4px;align-items:flex-end;height:32px;padding:4px 8px 0;background:var(--bg0);overflow:hidden}
      .tab{display:flex;gap:8px;align-items:center;max-width:210px;padding:5px 10px;border-radius:9px 9px 0 0;background:var(--bg2);color:var(--mut);cursor:pointer;white-space:nowrap}
      .tab.on{background:var(--bg);color:var(--fg);border-top:2px solid #fdb93a}.tab b{color:var(--mut)}.tab b:hover{color:#e5484d}
      #tabs>button{background:none;border:0;color:var(--acc);font-size:18px;cursor:pointer;padding:0 8px}
      .bar{display:flex;gap:6px;align-items:center;height:44px;padding:6px 10px;background:var(--bg);border-bottom:2px solid #2f66c4;color:var(--fg)}
      button{background:var(--bg2);color:var(--fg);border:1px solid var(--bd);border-radius:8px;padding:5px 10px;cursor:pointer}
      button:hover{border-color:var(--acc)}.go{background:#fdb93a;color:#2b1d00;border-color:#fdb93a;font-weight:700}
      .spot{outline:3px solid var(--acc);outline-offset:2px}
      input{flex:1;min-width:80px;background:var(--bg2);color:var(--fg);border:1px solid var(--bd);border-radius:8px;padding:6px 10px}
      #badge{display:none;padding:3px 9px;border-radius:99px;background:var(--bgb);color:var(--acc);cursor:pointer;white-space:nowrap;max-width:34vw;overflow:hidden;text-overflow:ellipsis}#badge.warn{background:#4a3410;color:#fdb93a}
      .row{display:none;gap:6px;padding:6px 10px;background:var(--bg2)}.row.on{display:flex}
      #panel{display:none;max-height:45vh;overflow:auto;white-space:pre-wrap;padding:10px 14px;background:var(--bg2);color:var(--fg);border-bottom:1px solid var(--bd)}
      #panel.ok{border-left:4px solid var(--acc)}#panel.err{border-left:4px solid #e5484d}
      #toast,#tour,#help{display:none;position:fixed;z-index:5;background:var(--bg2);color:var(--fg);border:1px solid var(--bd);border-radius:12px;box-shadow:0 8px 28px var(--sh);padding:12px 14px}
      #toast.on{display:block;right:14px;bottom:14px;max-width:min(480px,90vw);white-space:pre-wrap;cursor:pointer;border-left:4px solid var(--acc)}#toast.err{border-left-color:#e5484d}
      #tour.on{display:block;top:86px;right:14px;width:min(340px,92vw)}#tour p,#help p{margin:8px 0;line-height:1.45;white-space:pre-wrap}
      #help.on{display:block;top:86px;right:14px;width:min(520px,94vw);max-height:calc(100vh - 110px);overflow:auto}
      .hd{display:flex;justify-content:space-between;align-items:center;gap:8px}.hd b{font-size:14px}
      .nav{display:flex;gap:6px;align-items:center;justify-content:flex-end}.dots{margin-right:auto;color:var(--acc);letter-spacing:2px}
      h3{margin:12px 0 4px;color:var(--acc)}.hr{display:flex;gap:10px;padding:3px 0;align-items:baseline}.hr kbd{min-width:120px;color:var(--acc);font-weight:600;white-space:nowrap}
      .hint{color:var(--mut)}
      #start{display:none;position:fixed;inset:0;z-index:0;padding-top:96px;overflow:auto;background:var(--bg);color:var(--fg)}
      .sbox{max-width:880px;margin:0 auto;padding:16px 20px}.sbox h1{font-size:28px;color:var(--acc);margin:6px 0 14px}.sbox input{width:100%;font-size:16px;padding:12px 16px;border-radius:99px}
      .cols{display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:20px;margin-top:20px}.col h2{font-size:15px;margin:0 0 8px}
      .item{display:flex;flex-direction:column;padding:8px 12px;margin-bottom:6px;border-radius:10px;background:var(--bg2);border:1px solid var(--bd);cursor:pointer;overflow:hidden}.item:hover{border-color:var(--acc)}
      .item span,.item small{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.item small{color:var(--mut)}
    </style>
    <div id="start"></div><div id="tabs"></div>
    <div class="bar"><button id="b">◀</button><button id="f">▶</button><button id="r">⟳</button>
      <input id="u" spellcheck="false"><span id="badge"></span><button id="s" class="go"></button><button id="a"></button><button id="hl"></button><button id="wt"></button><button id="hi"></button><button id="sg"></button><button id="hp"></button></div>
    <div class="row" id="qrow"><input id="q"><button id="qq" class="go">➤</button><button id="qa"></button><button id="qs"></button><button id="qx"></button></div><div id="panel"></div>
    <div id="toast"></div><div id="tour"></div><div id="help"></div>`;
    $ = (i) => sh.getElementById(i);
    panel = $("panel"); qrow = $("qrow"); qin = $("q"); urlIn = $("u"); tabsEl = $("tabs"); badge = $("badge");
    toastEl = $("toast"); helpEl = $("help"); tourEl = $("tour"); startEl = $("start");
    toastEl.onclick = () => { toastEl.className = ""; };
    urlIn.value = web() ? location.href : ""; urlIn.placeholder = L.addr;
    $("s").textContent = "💾 " + L.save; $("s").title = L.t1 + " (Ctrl+S)"; $("a").textContent = "✨ " + L.ask; $("a").title = L.t2 + " (Ctrl+Shift+A)"; $("sg").textContent = "🗂"; $("sg").title = L.sg; $("qa").textContent = "🗂✨"; $("qa").title = L.aa; $("qs").textContent = "🗂📝"; $("qs").title = L.sall; $("qx").textContent = "🗂📊"; $("qx").title = L.xall;
    $("hl").textContent = "🖍"; $("hl").title = L.hl + " (Ctrl+H)"; $("wt").textContent = "👁"; $("wt").title = L.watch; $("hi").textContent = "🕘"; $("hi").title = L.hist; $hist = $("hi"); urlIn.title = L.asklib;
    $("hp").textContent = "❓"; $("hp").title = L.helpb; $("hp").onclick = help;
    $("hl").onclick = () => act("highlight"); $("wt").onclick = () => act("watch"); $("hi").onclick = () => act("history"); qin.placeholder = L.q;
    $("b").onclick = () => history.back(); $("f").onclick = () => history.forward(); $("r").onclick = () => location.reload();
    urlIn.onkeydown = (e) => { if (e.key === "Enter") send({ cmd: "go", q: urlIn.value }); };
    $("s").onclick = () => act("save");
    $("sg").onclick = () => act("save_tabs");
    const openAsk = () => { if (!web()) return toast(L.nohttp, "err"); qrow.classList.add("on"); qin.focus(); };
    $("a").onclick = () => { if (qrow.classList.contains("on")) qrow.classList.remove("on"); else openAsk(); };
    const ask = () => qin.value.trim() && act("ask", qin.value.trim());
    const askAll = (k) => qin.value.trim() && act("ask_tabs", qin.value.trim(), k);
    $("qq").onclick = ask; $("qa").onclick = () => askAll("synthesize"); $("qs").onclick = () => askAll("summarize"); $("qx").onclick = () => askAll("extract");
    qin.onkeydown = (e) => { if (e.key === "Enter") (e.ctrlKey ? () => askAll("synthesize") : ask)(); };
    badge.onclick = () => {
      if (!info) return;
      const lines = [];
      if (info.status) {
        lines.push("📚 " + info.status.path + (info.status.snapshot ? "\n🖼 " + L.snap + ": " + info.status.snapshot : ""));
        if (info.status.changed) { lines.push("⚠ " + L.changed + " — 💾 " + L.refresh); lines.push("", L.diffh + ":", ...(info.status.diff || []).map((d) => "  " + d)); }
      }
      if (info.related && info.related.length) { lines.push("", "🔗 " + L.related + ":"); for (const r of info.related) lines.push(" • " + r.score + "%  " + r.path + "\n     " + r.snippet); }
      show(lines.join("\n"), "ok");
    };
    document.documentElement.appendChild(host);
    document.documentElement.style.setProperty("margin-top", "78px", "important");
    addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        const open = ti >= 0 || helpOpen() || panel.style.display === "block" || toastEl.className || qrow.classList.contains("on");
        if (!open) return;
        e.preventDefault(); closeTour(); helpEl.className = ""; toastEl.className = ""; show(""); qrow.classList.remove("on");
        return;
      }
      if (e.key === "F1") { e.preventDefault(); help(); return; }
      if (!(e.ctrlKey || e.metaKey)) return;
      const k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
      if (k === "t") { e.preventDefault(); send({ cmd: e.shiftKey ? "reopen" : "newtab" }); }
      else if (k === "w") { e.preventDefault(); send({ cmd: "close", id: T }); }
      else if (k === "Tab") { e.preventDefault(); send({ cmd: "cycle", back: e.shiftKey }); }
      else if (k === "l") { e.preventDefault(); urlIn.focus(); urlIn.select(); }
      else if (k === "s" && !e.shiftKey) { e.preventDefault(); act("save"); }
      else if (k === "a" && e.shiftKey) { e.preventDefault(); openAsk(); }
      else if (k === "h" && !e.shiftKey) { e.preventDefault(); act("highlight"); }
    }, true);
    // Ctrl/Cmd+click or middle-click on a link: open it in a new tab
    const newTabLink = (e) => {
      const a = e.target && e.target.closest && e.target.closest("a[href]");
      if (!a || !/^https?:/i.test(a.href)) return;
      if (e.type === "click" ? !(e.ctrlKey || e.metaKey) : e.button !== 1) return;
      e.preventDefault(); e.stopPropagation(); send({ cmd: "newtab", url: a.href });
    };
    addEventListener("click", newTabLink, true); addEventListener("auxclick", newTabLink, true);
    send({ cmd: "ready", start: !web() });
  };
  const report = () => send({ cmd: "loaded", url: location.href, title: document.title, text: (document.body ? document.body.innerText : "").slice(0, 300000) });
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", mount); else mount();
  addEventListener("load", () => setTimeout(report, 400));
})();
