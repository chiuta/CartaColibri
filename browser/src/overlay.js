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
  const Y = {
    en: { rd: "Reader view (Ctrl+Shift+R)", rdA: "Read aloud", rdStop: "Stop reading", rdNoTts: "Read aloud is not available in this webview.", rdEmpty: "No article text found on this page.", rdTheme: "Dark / light / sepia", rdSmall: "Smaller text", rdBig: "Larger text", selAsk: "Ask", selNote: "Note", selHl: "Highlight", selTr: "Translate", selEx: "Explain simply", selAskPre: "About this passage: ", noteok: "Note saved in your library ✔", tabColor: "Color of this tab", noColor: "No color", tabSave: "Save this group", tabAsk: "Ask this group", qgrp: "Question about this tab group…", discard: "Put this tab to sleep", discardAll: "Put inactive tabs to sleep", dupes: "Close duplicate tabs", dupTab: "Duplicate of another tab", slept: "{n} tab(s) put to sleep ✔ (they wake when you open them)", dedup: "{n} duplicate tab(s) closed ✔", tsearch: "Search tabs and library (Ctrl+Shift+K)", tsPh: "Search open tabs and your library…", tsTabs: "Open tabs", tsLib: "In your library", tsNone: "Nothing found", copyPath: "Click to copy the file path", showDiff: "Show what changed", refreshCopy: "Refresh my saved copy", noDiff: "No differences found.", cite: "Cite this page (APA / MLA / BibTeX)", copied: "Copied ✔", copyClean: "Copy clean link (no tracking parameters)", cleaned: "Link copied without tracking parameters ✔", nocopy: "Could not copy — select the text and press Ctrl+C.", later: "Read later (saves the page in your library)", laterOk: "Added to your reading list ✔ — it is saved in your library", laterOff: "Removed from your reading list", sLater: "Reading list", sNoLater: "Nothing to read later — press ☰ → 📖 on a page.", dlOffer: "This is a document ({f}). Add it to your library?", dlAdd: "Add to library", dlOpen: "Open anyway", dlOk: "Added to your library ✔ — {p}", dlBusy: "Downloading…", find: "Find in page (Ctrl+F)", findPh: "Find in page…", findN: "{i} of {n}", findNone: "No matches", trackers: "Block known trackers", trkOn: "Tracker blocking on — known tracking addresses are not opened", trkOff: "Tracker blocking off", trkBlk: "Blocked a tracking address: {h}", more: "More tools", on: "on", off: "off" },
    ro: { rd: "Mod citire (Ctrl+Shift+R)", rdA: "Citește cu voce tare", rdStop: "Oprește citirea", rdNoTts: "Citirea cu voce tare nu este disponibilă în acest webview.", rdEmpty: "Nu am găsit text de articol pe pagină.", rdTheme: "Închis / deschis / sepia", rdSmall: "Text mai mic", rdBig: "Text mai mare", selAsk: "Întreabă", selNote: "Notă", selHl: "Evidențiază", selTr: "Traduce", selEx: "Explică simplu", selAskPre: "Despre acest pasaj: ", noteok: "Nota a fost salvată în bibliotecă ✔", tabColor: "Culoarea acestui tab", noColor: "Fără culoare", tabSave: "Salvează acest grup", tabAsk: "Întreabă acest grup", qgrp: "Întrebare despre acest grup de taburi…", discard: "Adoarme acest tab", discardAll: "Adoarme taburile inactive", dupes: "Închide taburile duplicate", dupTab: "Duplicat al altui tab", slept: "{n} tab(uri) adormite ✔ (se trezesc când le deschizi)", dedup: "{n} tab(uri) duplicate închise ✔", tsearch: "Caută în taburi și bibliotecă (Ctrl+Shift+K)", tsPh: "Caută în taburile deschise și în bibliotecă…", tsTabs: "Taburi deschise", tsLib: "În biblioteca ta", tsNone: "Nu am găsit nimic", copyPath: "Clic pentru a copia calea fișierului", showDiff: "Arată ce s-a schimbat", refreshCopy: "Reîmprospătează copia mea salvată", noDiff: "Nu am găsit diferențe.", cite: "Citează această pagină (APA / MLA / BibTeX)", copied: "Copiat ✔", copyClean: "Copiază linkul curat (fără parametri de urmărire)", cleaned: "Linkul a fost copiat fără parametri de urmărire ✔", nocopy: "Nu am putut copia — selectează textul și apasă Ctrl+C.", later: "Citește mai târziu (salvează pagina în bibliotecă)", laterOk: "Adăugat în lista de citit ✔ — e salvat în bibliotecă", laterOff: "Scos din lista de citit", sLater: "Listă de citit", sNoLater: "Nimic de citit mai târziu — apasă ☰ → 📖 pe o pagină.", dlOffer: "Acesta este un document ({f}). Îl adaug în bibliotecă?", dlAdd: "Adaugă în bibliotecă", dlOpen: "Deschide oricum", dlOk: "Adăugat în bibliotecă ✔ — {p}", dlBusy: "Se descarcă…", find: "Caută în pagină (Ctrl+F)", findPh: "Caută în pagină…", findN: "{i} din {n}", findNone: "Nicio potrivire", trackers: "Blochează trackerele cunoscute", trkOn: "Blocarea trackerelor pornită — adresele de urmărire cunoscute nu se deschid", trkOff: "Blocarea trackerelor oprită", trkBlk: "Am blocat o adresă de urmărire: {h}", more: "Mai multe unelte", on: "pornit", off: "oprit" },
    fr: { rd: "Mode lecture (Ctrl+Maj+R)", rdA: "Lire à voix haute", rdStop: "Arrêter la lecture", rdNoTts: "La lecture à voix haute n'est pas disponible dans ce webview.", rdEmpty: "Aucun texte d'article trouvé sur cette page.", rdTheme: "Sombre / clair / sépia", rdSmall: "Texte plus petit", rdBig: "Texte plus grand", selAsk: "Demander", selNote: "Note", selHl: "Surligner", selTr: "Traduire", selEx: "Expliquer simplement", selAskPre: "À propos de ce passage : ", noteok: "Note enregistrée dans ta bibliothèque ✔", tabColor: "Couleur de cet onglet", noColor: "Sans couleur", tabSave: "Enregistrer ce groupe", tabAsk: "Interroger ce groupe", qgrp: "Question sur ce groupe d'onglets…", discard: "Mettre cet onglet en veille", discardAll: "Mettre les onglets inactifs en veille", dupes: "Fermer les onglets en double", dupTab: "Doublon d'un autre onglet", slept: "{n} onglet(s) mis en veille ✔ (ils se réveillent à l'ouverture)", dedup: "{n} onglet(s) en double fermé(s) ✔", tsearch: "Chercher dans les onglets et la bibliothèque (Ctrl+Maj+K)", tsPh: "Chercher dans les onglets ouverts et ta bibliothèque…", tsTabs: "Onglets ouverts", tsLib: "Dans ta bibliothèque", tsNone: "Rien trouvé", copyPath: "Clique pour copier le chemin du fichier", showDiff: "Voir ce qui a changé", refreshCopy: "Actualiser ma copie enregistrée", noDiff: "Aucune différence.", cite: "Citer cette page (APA / MLA / BibTeX)", copied: "Copié ✔", copyClean: "Copier le lien propre (sans paramètres de suivi)", cleaned: "Lien copié sans paramètres de suivi ✔", nocopy: "Copie impossible — sélectionne le texte et appuie sur Ctrl+C.", later: "Lire plus tard (enregistre la page dans ta bibliothèque)", laterOk: "Ajouté à ta liste de lecture ✔ — enregistré dans ta bibliothèque", laterOff: "Retiré de ta liste de lecture", sLater: "Liste de lecture", sNoLater: "Rien à lire plus tard — appuie sur ☰ → 📖 sur une page.", dlOffer: "C'est un document ({f}). L'ajouter à ta bibliothèque ?", dlAdd: "Ajouter à la bibliothèque", dlOpen: "Ouvrir quand même", dlOk: "Ajouté à ta bibliothèque ✔ — {p}", dlBusy: "Téléchargement…", find: "Chercher dans la page (Ctrl+F)", findPh: "Chercher dans la page…", findN: "{i} sur {n}", findNone: "Aucun résultat", trackers: "Bloquer les traceurs connus", trkOn: "Blocage des traceurs activé — les adresses de suivi connues ne s'ouvrent pas", trkOff: "Blocage des traceurs désactivé", trkBlk: "Adresse de suivi bloquée : {h}", more: "Plus d'outils", on: "activé", off: "désactivé" },
    it: { rd: "Modalità lettura (Ctrl+Maiusc+R)", rdA: "Leggi ad alta voce", rdStop: "Ferma la lettura", rdNoTts: "La lettura ad alta voce non è disponibile in questo webview.", rdEmpty: "Nessun testo di articolo trovato in questa pagina.", rdTheme: "Scuro / chiaro / seppia", rdSmall: "Testo più piccolo", rdBig: "Testo più grande", selAsk: "Chiedi", selNote: "Nota", selHl: "Evidenzia", selTr: "Traduci", selEx: "Spiega semplice", selAskPre: "Su questo passaggio: ", noteok: "Nota salvata nella tua biblioteca ✔", tabColor: "Colore di questa scheda", noColor: "Nessun colore", tabSave: "Salva questo gruppo", tabAsk: "Interroga questo gruppo", qgrp: "Domanda su questo gruppo di schede…", discard: "Metti in pausa questa scheda", discardAll: "Metti in pausa le schede inattive", dupes: "Chiudi le schede duplicate", dupTab: "Duplicato di un'altra scheda", slept: "{n} scheda/e in pausa ✔ (si riattivano quando le apri)", dedup: "{n} scheda/e duplicata/e chiusa/e ✔", tsearch: "Cerca in schede e biblioteca (Ctrl+Maiusc+K)", tsPh: "Cerca nelle schede aperte e nella biblioteca…", tsTabs: "Schede aperte", tsLib: "Nella tua biblioteca", tsNone: "Nessun risultato", copyPath: "Clicca per copiare il percorso del file", showDiff: "Mostra cosa è cambiato", refreshCopy: "Aggiorna la mia copia salvata", noDiff: "Nessuna differenza.", cite: "Cita questa pagina (APA / MLA / BibTeX)", copied: "Copiato ✔", copyClean: "Copia il link pulito (senza parametri di tracciamento)", cleaned: "Link copiato senza parametri di tracciamento ✔", nocopy: "Impossibile copiare — seleziona il testo e premi Ctrl+C.", later: "Leggi dopo (salva la pagina nella biblioteca)", laterOk: "Aggiunto alla lista di lettura ✔ — salvato nella biblioteca", laterOff: "Rimosso dalla lista di lettura", sLater: "Lista di lettura", sNoLater: "Niente da leggere dopo — premi ☰ → 📖 su una pagina.", dlOffer: "Questo è un documento ({f}). Lo aggiungo alla biblioteca?", dlAdd: "Aggiungi alla biblioteca", dlOpen: "Apri comunque", dlOk: "Aggiunto alla biblioteca ✔ — {p}", dlBusy: "Download in corso…", find: "Trova nella pagina (Ctrl+F)", findPh: "Trova nella pagina…", findN: "{i} di {n}", findNone: "Nessun risultato", trackers: "Blocca i tracker noti", trkOn: "Blocco dei tracker attivo — gli indirizzi di tracciamento noti non si aprono", trkOff: "Blocco dei tracker disattivato", trkBlk: "Indirizzo di tracciamento bloccato: {h}", more: "Altri strumenti", on: "attivo", off: "disattivo" },
    es: { rd: "Modo lectura (Ctrl+Mayús+R)", rdA: "Leer en voz alta", rdStop: "Detener la lectura", rdNoTts: "La lectura en voz alta no está disponible en este webview.", rdEmpty: "No se encontró texto de artículo en esta página.", rdTheme: "Oscuro / claro / sepia", rdSmall: "Texto más pequeño", rdBig: "Texto más grande", selAsk: "Preguntar", selNote: "Nota", selHl: "Resaltar", selTr: "Traducir", selEx: "Explicar fácil", selAskPre: "Sobre este pasaje: ", noteok: "Nota guardada en tu biblioteca ✔", tabColor: "Color de esta pestaña", noColor: "Sin color", tabSave: "Guardar este grupo", tabAsk: "Preguntar a este grupo", qgrp: "Pregunta sobre este grupo de pestañas…", discard: "Dormir esta pestaña", discardAll: "Dormir las pestañas inactivas", dupes: "Cerrar pestañas duplicadas", dupTab: "Duplicada de otra pestaña", slept: "{n} pestaña(s) dormida(s) ✔ (se despiertan al abrirlas)", dedup: "{n} pestaña(s) duplicada(s) cerrada(s) ✔", tsearch: "Buscar en pestañas y biblioteca (Ctrl+Mayús+K)", tsPh: "Buscar en las pestañas abiertas y en tu biblioteca…", tsTabs: "Pestañas abiertas", tsLib: "En tu biblioteca", tsNone: "No se encontró nada", copyPath: "Haz clic para copiar la ruta del archivo", showDiff: "Ver qué ha cambiado", refreshCopy: "Actualizar mi copia guardada", noDiff: "No hay diferencias.", cite: "Citar esta página (APA / MLA / BibTeX)", copied: "Copiado ✔", copyClean: "Copiar enlace limpio (sin parámetros de rastreo)", cleaned: "Enlace copiado sin parámetros de rastreo ✔", nocopy: "No se pudo copiar — selecciona el texto y pulsa Ctrl+C.", later: "Leer después (guarda la página en tu biblioteca)", laterOk: "Añadido a tu lista de lectura ✔ — guardado en tu biblioteca", laterOff: "Quitado de tu lista de lectura", sLater: "Lista de lectura", sNoLater: "Nada para leer después — pulsa ☰ → 📖 en una página.", dlOffer: "Esto es un documento ({f}). ¿Lo añado a tu biblioteca?", dlAdd: "Añadir a la biblioteca", dlOpen: "Abrir de todos modos", dlOk: "Añadido a tu biblioteca ✔ — {p}", dlBusy: "Descargando…", find: "Buscar en la página (Ctrl+F)", findPh: "Buscar en la página…", findN: "{i} de {n}", findNone: "Sin coincidencias", trackers: "Bloquear rastreadores conocidos", trkOn: "Bloqueo de rastreadores activado — no se abren las direcciones de rastreo conocidas", trkOff: "Bloqueo de rastreadores desactivado", trkBlk: "Dirección de rastreo bloqueada: {h}", more: "Más herramientas", on: "activado", off: "desactivado" },
    pt: { rd: "Modo de leitura (Ctrl+Shift+R)", rdA: "Ler em voz alta", rdStop: "Parar a leitura", rdNoTts: "A leitura em voz alta não está disponível neste webview.", rdEmpty: "Não foi encontrado texto de artigo nesta página.", rdTheme: "Escuro / claro / sépia", rdSmall: "Texto mais pequeno", rdBig: "Texto maior", selAsk: "Perguntar", selNote: "Nota", selHl: "Destacar", selTr: "Traduzir", selEx: "Explicar simples", selAskPre: "Sobre esta passagem: ", noteok: "Nota guardada na tua biblioteca ✔", tabColor: "Cor deste separador", noColor: "Sem cor", tabSave: "Guardar este grupo", tabAsk: "Perguntar a este grupo", qgrp: "Pergunta sobre este grupo de separadores…", discard: "Adormecer este separador", discardAll: "Adormecer separadores inativos", dupes: "Fechar separadores duplicados", dupTab: "Duplicado de outro separador", slept: "{n} separador(es) adormecido(s) ✔ (acordam quando os abrires)", dedup: "{n} separador(es) duplicado(s) fechado(s) ✔", tsearch: "Pesquisar separadores e biblioteca (Ctrl+Shift+K)", tsPh: "Pesquisar nos separadores abertos e na biblioteca…", tsTabs: "Separadores abertos", tsLib: "Na tua biblioteca", tsNone: "Nada encontrado", copyPath: "Clica para copiar o caminho do ficheiro", showDiff: "Ver o que mudou", refreshCopy: "Atualizar a minha cópia guardada", noDiff: "Sem diferenças.", cite: "Citar esta página (APA / MLA / BibTeX)", copied: "Copiado ✔", copyClean: "Copiar ligação limpa (sem parâmetros de rastreio)", cleaned: "Ligação copiada sem parâmetros de rastreio ✔", nocopy: "Não foi possível copiar — seleciona o texto e carrega em Ctrl+C.", later: "Ler mais tarde (guarda a página na biblioteca)", laterOk: "Adicionado à lista de leitura ✔ — guardado na biblioteca", laterOff: "Removido da lista de leitura", sLater: "Lista de leitura", sNoLater: "Nada para ler mais tarde — carrega em ☰ → 📖 numa página.", dlOffer: "Isto é um documento ({f}). Adiciono à biblioteca?", dlAdd: "Adicionar à biblioteca", dlOpen: "Abrir mesmo assim", dlOk: "Adicionado à biblioteca ✔ — {p}", dlBusy: "A transferir…", find: "Procurar na página (Ctrl+F)", findPh: "Procurar na página…", findN: "{i} de {n}", findNone: "Sem resultados", trackers: "Bloquear rastreadores conhecidos", trkOn: "Bloqueio de rastreadores ligado — os endereços de rastreio conhecidos não abrem", trkOff: "Bloqueio de rastreadores desligado", trkBlk: "Endereço de rastreio bloqueado: {h}", more: "Mais ferramentas", on: "ligado", off: "desligado" },
    de: { rd: "Lesemodus (Strg+Umschalt+R)", rdA: "Vorlesen", rdStop: "Vorlesen beenden", rdNoTts: "Vorlesen ist in diesem Webview nicht verfügbar.", rdEmpty: "Auf dieser Seite wurde kein Artikeltext gefunden.", rdTheme: "Dunkel / hell / Sepia", rdSmall: "Kleinerer Text", rdBig: "Größerer Text", selAsk: "Fragen", selNote: "Notiz", selHl: "Markieren", selTr: "Übersetzen", selEx: "Einfach erklären", selAskPre: "Zu dieser Passage: ", noteok: "Notiz in deiner Bibliothek gespeichert ✔", tabColor: "Farbe dieses Tabs", noColor: "Keine Farbe", tabSave: "Diese Gruppe speichern", tabAsk: "Diese Gruppe befragen", qgrp: "Frage zu dieser Tab-Gruppe…", discard: "Diesen Tab schlafen legen", discardAll: "Inaktive Tabs schlafen legen", dupes: "Doppelte Tabs schließen", dupTab: "Doppelt zu einem anderen Tab", slept: "{n} Tab(s) schlafen gelegt ✔ (sie wachen beim Öffnen auf)", dedup: "{n} doppelte(r) Tab(s) geschlossen ✔", tsearch: "Tabs und Bibliothek durchsuchen (Strg+Umschalt+K)", tsPh: "Offene Tabs und deine Bibliothek durchsuchen…", tsTabs: "Offene Tabs", tsLib: "In deiner Bibliothek", tsNone: "Nichts gefunden", copyPath: "Klicken, um den Dateipfad zu kopieren", showDiff: "Änderungen anzeigen", refreshCopy: "Meine gespeicherte Kopie aktualisieren", noDiff: "Keine Unterschiede gefunden.", cite: "Diese Seite zitieren (APA / MLA / BibTeX)", copied: "Kopiert ✔", copyClean: "Sauberen Link kopieren (ohne Tracking-Parameter)", cleaned: "Link ohne Tracking-Parameter kopiert ✔", nocopy: "Kopieren nicht möglich — Text markieren und Strg+C drücken.", later: "Später lesen (speichert die Seite in der Bibliothek)", laterOk: "Zur Leseliste hinzugefügt ✔ — in deiner Bibliothek gespeichert", laterOff: "Aus der Leseliste entfernt", sLater: "Leseliste", sNoLater: "Nichts zum späteren Lesen — drücke ☰ → 📖 auf einer Seite.", dlOffer: "Das ist ein Dokument ({f}). Zur Bibliothek hinzufügen?", dlAdd: "Zur Bibliothek hinzufügen", dlOpen: "Trotzdem öffnen", dlOk: "Zur Bibliothek hinzugefügt ✔ — {p}", dlBusy: "Wird heruntergeladen…", find: "In Seite suchen (Strg+F)", findPh: "In Seite suchen…", findN: "{i} von {n}", findNone: "Keine Treffer", trackers: "Bekannte Tracker blockieren", trkOn: "Tracker-Blockierung an — bekannte Tracking-Adressen werden nicht geöffnet", trkOff: "Tracker-Blockierung aus", trkBlk: "Tracking-Adresse blockiert: {h}", more: "Weitere Werkzeuge", on: "an", off: "aus" },
    hu: { rd: "Olvasó nézet (Ctrl+Shift+R)", rdA: "Felolvasás", rdStop: "Felolvasás leállítása", rdNoTts: "A felolvasás nem érhető el ebben a webview-ban.", rdEmpty: "Nem találtam cikkszöveget ezen az oldalon.", rdTheme: "Sötét / világos / szépia", rdSmall: "Kisebb szöveg", rdBig: "Nagyobb szöveg", selAsk: "Kérdés", selNote: "Jegyzet", selHl: "Kiemelés", selTr: "Fordítás", selEx: "Egyszerű magyarázat", selAskPre: "Erről a részletről: ", noteok: "Jegyzet elmentve a könyvtáradba ✔", tabColor: "A lap színe", noColor: "Nincs szín", tabSave: "A csoport mentése", tabAsk: "Kérdés a csoporthoz", qgrp: "Kérdés erről a laccsoportról…", discard: "A lap elaltatása", discardAll: "Az inaktív lapok elaltatása", dupes: "Ismétlődő lapok bezárása", dupTab: "Egy másik lap másolata", slept: "{n} lap elaltatva ✔ (megnyitáskor felébrednek)", dedup: "{n} ismétlődő lap bezárva ✔", tsearch: "Keresés a lapokban és a könyvtárban (Ctrl+Shift+K)", tsPh: "Keresés a nyitott lapokban és a könyvtárban…", tsTabs: "Nyitott lapok", tsLib: "A könyvtáradban", tsNone: "Nincs találat", copyPath: "Kattints a fájl útvonalának másolásához", showDiff: "Mi változott?", refreshCopy: "Mentett példányom frissítése", noDiff: "Nincs különbség.", cite: "Az oldal hivatkozása (APA / MLA / BibTeX)", copied: "Kimásolva ✔", copyClean: "Tiszta link másolása (követőparaméterek nélkül)", cleaned: "Link kimásolva követőparaméterek nélkül ✔", nocopy: "Nem sikerült másolni — jelöld ki a szöveget, és nyomd meg a Ctrl+C-t.", later: "Olvasd később (az oldal a könyvtárba kerül)", laterOk: "Felvéve az olvasólistára ✔ — a könyvtáradba mentve", laterOff: "Eltávolítva az olvasólistáról", sLater: "Olvasólista", sNoLater: "Nincs mit később olvasni — nyomd meg a ☰ → 📖 gombot egy oldalon.", dlOffer: "Ez egy dokumentum ({f}). Hozzáadjam a könyvtárhoz?", dlAdd: "Hozzáadás a könyvtárhoz", dlOpen: "Megnyitás mégis", dlOk: "Hozzáadva a könyvtárhoz ✔ — {p}", dlBusy: "Letöltés…", find: "Keresés az oldalon (Ctrl+F)", findPh: "Keresés az oldalon…", findN: "{i} / {n}", findNone: "Nincs találat", trackers: "Ismert követők blokkolása", trkOn: "Követőblokkolás bekapcsolva — az ismert követőcímek nem nyílnak meg", trkOff: "Követőblokkolás kikapcsolva", trkBlk: "Követőcím blokkolva: {h}", more: "További eszközök", on: "be", off: "ki" },
  };
  const lang = (navigator.language || "en").slice(0, 2);
  const L = Object.assign({}, D.en, X.en, Y.en, D[lang] || {}, X[lang] || {}, Y[lang] || {});
  const web = () => /^https?:$/.test(location.protocol);
  const send = (o) => post(JSON.stringify(Object.assign({ n: N, t: T }, o)));
  const pageHtml = () => { try { return document.documentElement.outerHTML.slice(0, 3000000); } catch (e) { return ""; } };
  const pageText = () => String(getSelection()) || (document.body ? document.body.innerText : "");
  const TRK = /^(utm_|oly_|fbclid$|gclid$|dclid$|msclkid$|mc_eid$|mc_cid$|igshid$|yclid$|_hsenc$|_hsmi$|mkt_tok$|ref_src$|spm$|vero_id$)/i;
  const cleanUrl = (u) => { try { const x = new URL(u); for (const k of [...x.searchParams.keys()]) if (TRK.test(k)) x.searchParams.delete(k); return x.href; } catch (e) { return u; } };
  const COL = ["", "#e5484d", "#fdb93a", "#3dd68c", "#5fd7e6", "#8e7cf0", "#f06fb2"];
  const copy = (t) => {
    const done = () => toast(L.copied), fail = () => toast(L.nocopy, "err");
    try { if (navigator.clipboard && navigator.clipboard.writeText) return navigator.clipboard.writeText(t).then(done, () => legacy()); } catch (e) { /* fall through */ }
    function legacy() { try { const a = document.createElement("textarea"); a.value = t; a.style.cssText = "position:fixed;opacity:0"; document.body.appendChild(a); a.select(); const ok = document.execCommand("copy"); a.remove(); ok ? done() : fail(); } catch (e) { fail(); } }
    legacy();
  };
  const el = (tag, txt, cls) => { const e = document.createElement(tag); if (txt != null) e.textContent = txt; if (cls) e.className = cls; return e; };
  let $, $hist, sh, hostEl, panel, selm, menuEl, frow, fin, fcnt, rdEl, tsEl, ctxEl, prefs = {}, tabList = [], activeTab = 0, groupColor = 0, selTxt = "", fi = 0, fn = 0, rdText = "", rdSize = 19, rdTheme = "", qrow, qin, urlIn, tabsEl, badge, toastEl, helpEl, tourEl, startEl, startIn, info = null, mock = false, tt, ti = -1;
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
    if (["save", "highlight", "watch", "ask", "later"].includes(cmd) && !web()) return toast(L.nohttp, "err");
    if (cmd === "later") { send({ cmd, url: cleanUrl(location.href), title: document.title, text: pageText(), html: pageHtml() }); return; }
    if (cmd === "save") { show("…"); send({ cmd, url: cleanUrl(location.href), title: document.title, text: pageText(), html: pageHtml() }); }
    else if (cmd === "highlight") { const sel = String(getSelection()).trim(); if (!sel) return toast(L.hlnone, "err"); send({ cmd, url: cleanUrl(location.href), title: document.title, text: document.body ? document.body.innerText : "", sel }); applyHL([sel]); }
    else if (cmd === "watch") { send({ cmd, url: cleanUrl(location.href) }); }
    else if (cmd === "history") { send({ cmd }); }
    else if (cmd === "ask") { show(L.think); send({ cmd, q, url: location.href, title: document.title, text: pageText() }); }
    else if (cmd === "ask_tabs") { show(L.think); send({ cmd, q, task: arguments[2] || "synthesize", color: groupColor }); }
    else if (cmd === "save_tabs") { show("…"); send({ cmd, color: arguments[2] || 0 }); }
  };
  const hidden = (name, fn) => Object.defineProperty(window, name + N, { enumerable: false, value: fn });
  hidden("__cc_", (m) => {
    if (!sh) return;
    if (m.k === "ts") return renderTs(m.x || []);
    if (!m.ok) return show(friendly(m.text), "err");
    const x = m.x;
    if (m.k === "dl") return offerDoc(x);
    if (m.k === "blk") return toast(L.trkBlk.replace("{h}", x), "err");
    const f = { later: () => (x && x.on ? L.laterOk : L.laterOff), note: () => L.noteok, dlok: () => L.dlOk.replace("{p}", (x && x.path) || ""), trk: () => { prefs.trackers = !!x; return x ? L.trkOn : L.trkOff; }, slept: () => L.slept.replace("{n}", x), dedup: () => L.dedup.replace("{n}", x), save: () => L.saved.replace("{p}", (x && x.path) || ""), hl: () => L.hlok, watch: () => L.watchok, hist: () => (x ? L.histon : L.histoff), tabs: () => L.tabsok.replace("{n}", x) }[m.k];
    if (f) { if (m.k !== "trk") show(""); toast(f()); } else show(m.text, "ok");
  });
  hidden("__cc_do_", (c, q, k) => act(c, q, k));
  hidden("__cc_tabs_", (s) => {
    if (!sh) return;
    tabsEl.textContent = ""; tabList = s.tabs; activeTab = s.active;
    for (const t of s.tabs) {
      const e = el("div", null, "tab" + (t.id === s.active ? " on" : "") + (t.sleep ? " zz" : ""));
      if (t.color) e.style.boxShadow = "inset 0 -3px 0 " + COL[t.color];
      e.oncontextmenu = (ev) => { ev.preventDefault(); ev.stopPropagation(); tabMenu(t, ev); };
      const lab = el("span", (t.sleep ? "💤 " : "") + (t.dup ? "⧉ " : "") + (t.title || t.url || "·").slice(0, 28)); lab.title = (t.dup ? L.dupTab + "\n" : "") + t.url;
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
    const lc = col("📖 " + L.sLater, d.later || [], L.sNoLater, go);
    lc.querySelectorAll(".item").forEach((r) => { const u = r.title; const x = el("b", "✓"); x.title = L.laterOff; x.onclick = (ev) => { ev.stopPropagation(); send({ cmd: "later_done", url: u }); r.remove(); }; r.append(x); });
    cols.append(lc, col("📚 " + L.sRecent, d.recent || [], L.sEmpty, go), col("👁 " + L.sWatch, (d.watched || []).map((u) => ({ url: u })), L.sNoWatch, go));
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
    table(L.sc, [["Ctrl+S", L.save], ["Ctrl+Shift+A", L.ask], ["Ctrl+H", L.hl], ["F1", L.helpT], ["Esc", L.esc], ["Ctrl+Shift+T", L.reopen], ["Ctrl+T · Ctrl+W · Ctrl+Tab", L.tabsNav], ["Ctrl+L", L.ab], ["Ctrl+F", L.find], ["Ctrl+Shift+R", L.rd], ["Ctrl+Shift+K", L.tsearch], ["Ctrl+🖱", L.cc]]);
    table(L.btn, [["💾", L.t1], ["✨", L.t2], ["🖍", L.t3], ["👁", L.t4], ["🕘", L.hist], ["🗂", L.sg], ["🗂✨", L.aa], ["🗂📝", L.sall], ["🗂📊", L.xall], ["📚 / 🔗", L.inlib + " · " + L.related], ["?…", L.asklib], ["☰", L.more], ["📖", L.later], ["❝", L.cite], ["🛡", L.trackers]]);
    if (mock) helpEl.append(el("p", "⚠ " + L.mockn, "hint"));
    const again = el("button", "🎓 " + L.tourT); again.onclick = () => { helpEl.className = ""; tour(0); }; helpEl.append(again);
    helpEl.className = "on";
  };
  // ---- rich panel helpers, documents, diff, citations
  const showNode = (...nodes) => { panel.textContent = ""; panel.append(...nodes); panel.className = "ok"; panel.style.display = "block"; };
  const btn = (txt, fn, cls) => { const b = el("button", txt, cls); b.onclick = fn; return b; };
  const offerDoc = (x) => {
    const f = file0(x.url);
    const row = el("div", null, "acts");
    row.append(btn("➕ " + L.dlAdd, () => { show(L.dlBusy); send({ cmd: "dl", url: x.url }); }, "go"));
    if (x.open) row.append(btn(L.dlOpen, () => { show(""); send({ cmd: "open_raw", url: x.url }); }));
    showNode(el("div", "📄 " + L.dlOffer.replace("{f}", f)), row);
  };
  const file0 = (u) => { try { return decodeURIComponent(new URL(u).pathname.split("/").pop() || u); } catch (e) { return u; } };
  const showStatus = () => {
    if (!info) return;
    const box = el("div");
    if (info.status) {
      box.append(el("div", "📚 " + info.status.path + (info.status.snapshot ? "\n🖼 " + L.snap + ": " + info.status.snapshot : ""), "pre"));
      if (info.status.changed) {
        box.append(el("div", "⚠ " + L.changed, "warnl"), el("div", L.diffh + ":", "dh"));
        const d = el("div", null, "diff");
        for (const ln of info.status.diff || []) d.append(el("div", ln, ln[0] === "+" ? "add" : ln[0] === "-" ? "del" : ""));
        if (!(info.status.diff || []).length) d.append(el("div", L.noDiff, "hint"));
        const acts = el("div", null, "acts"); acts.append(btn("💾 " + L.refreshCopy, () => act("save"), "go")); box.append(d, acts);
      }
    }
    if (info.related && info.related.length) {
      box.append(el("div", "🔗 " + L.related + ":", "dh"));
      for (const r of info.related) box.append(el("div", " • " + r.score + "%  " + r.path + "\n     " + r.snippet, "pre"));
    }
    showNode(box);
  };
  const meta = (...names) => { for (const n of names) { const m = document.querySelector('meta[name="' + n + '"],meta[property="' + n + '"],meta[itemprop="' + n + '"]'); if (m && m.content) return m.content.trim(); } return ""; };
  const citeData = () => {
    const url = cleanUrl(location.href), now = new Date();
    let author = meta("author", "article:author", "dc.creator", "citation_author");
    let pub = meta("article:published_time", "datePublished", "dc.date", "citation_publication_date", "date");
    let d = pub ? new Date(pub) : null; if (d && isNaN(d)) d = null;
    const site = meta("og:site_name", "application-name") || location.hostname.replace(/^www\./, "");
    return { url, title: (meta("og:title") || document.title || url).trim(), author, site, d, now };
  };
  const fmtDate = (d, f) => d.toLocaleDateString(lang, f);
  const citations = () => {
    const c = citeData(), y = c.d ? c.d.getFullYear() : "n.d.";
    const acc = c.now.toISOString().slice(0, 10), accLong = fmtDate(c.now, { year: "numeric", month: "long", day: "numeric" });
    const who = c.author || c.site;
    const apa = who + ". (" + (c.d ? y + ", " + fmtDate(c.d, { month: "long", day: "numeric" }) : "n.d.") + "). " + c.title + ". " + (c.author ? c.site + ". " : "") + c.url;
    const mla = (c.author ? c.author + ". " : "") + "“" + c.title + ".” " + c.site + ", " + (c.d ? fmtDate(c.d, { day: "numeric", month: "short", year: "numeric" }) + ", " : "") + c.url + ". Accessed " + c.now.getDate() + " " + c.now.toLocaleDateString("en", { month: "short" }) + " " + c.now.getFullYear() + ".";
    const key = ((c.author || c.site).split(/[ ,]/)[0].toLowerCase().replace(/[^a-z0-9]/g, "") || "web") + (c.d ? y : "nd");
    const esc = (t) => String(t).replace(/[{}]/g, "");
    const bib = "@misc{" + key + ",\n  title = {" + esc(c.title) + "},\n" + (c.author ? "  author = {" + esc(c.author) + "},\n" : "") + "  howpublished = {" + esc(c.site) + "},\n" + (c.d ? "  year = {" + y + "},\n" : "") + "  url = {" + c.url + "},\n  note = {Accessed: " + acc + "}\n}";
    return [["APA", apa], ["MLA", mla], ["BibTeX", bib]].map(([k, v]) => [k, v, accLong]);
  };
  const citeOpen = () => {
    if (!web()) return toast(L.nohttp, "err");
    const box = el("div"); box.append(el("div", "❝ " + L.cite, "dh"));
    for (const [k, v] of citations()) { const r = el("div", null, "cite"); r.append(btn("📋 " + k, () => copy(v)), el("div", v, "pre")); box.append(r); }
    showNode(box);
  };
  const copyClean = () => { if (!web()) return toast(L.nohttp, "err"); const u = cleanUrl(location.href); try { if (navigator.clipboard) return navigator.clipboard.writeText(u).then(() => toast(L.cleaned), () => copy(u)); } catch (e) { /* */ } copy(u); };

  // ---- reader view and read aloud
  const readerBlocks = () => {
    const cands = [...document.querySelectorAll("article,main,[role=main],#content,.post,.entry-content,.article-body")].concat([document.body]);
    let best = document.body, bs = -1;
    for (const c of cands) { if (!c) continue; const sc = [...c.querySelectorAll("p")].reduce((a, p) => a + (p.innerText || "").length, 0); if (sc > bs) { bs = sc; best = c; } }
    const out = [];
    for (const n of best.querySelectorAll("h1,h2,h3,h4,p,li,blockquote,pre")) {
      if (n.closest("nav,aside,footer,form,script,style,noscript,[aria-hidden=true]")) continue;
      if (n.parentElement && n.parentElement.closest("blockquote,li") && n.tagName === "P") continue;
      const t = (n.innerText || "").replace(/\s+/g, " ").trim();
      if (t.length < (n.tagName[0] === "H" ? 2 : 25)) continue;
      out.push([n.tagName.toLowerCase(), t]);
    }
    if (out.reduce((a, b) => a + b[1].length, 0) < 120) return (document.body.innerText || "").split(/\n{2,}/).map((t) => ["p", t.trim()]).filter((b) => b[1].length > 1).slice(0, 2000);
    return out.slice(0, 3000);
  };
  const tts = () => (typeof speechSynthesis !== "undefined" && typeof SpeechSynthesisUtterance !== "undefined" ? speechSynthesis : null);
  const speakStop = () => { try { tts() && tts().cancel(); } catch (e) { /* */ } };
  const speakToggle = () => {
    const t = tts(); if (!t) return toast(L.rdNoTts, "err");
    if (t.speaking) { t.cancel(); return; }
    const parts = rdText.match(/[^.!?\n]+[.!?]*\s*/g) || [rdText];
    for (const p of parts) { if (!p.trim()) continue; const u = new SpeechSynthesisUtterance(p.trim()); u.lang = document.documentElement.lang || navigator.language; t.speak(u); }
  };
  const readerOn = () => rdEl.style.display === "block";
  const readerClose = () => { speakStop(); rdEl.style.display = "none"; rdEl.textContent = ""; };
  const readerPaint = () => {
    const th = rdTheme || (matchMedia("(prefers-color-scheme:light)").matches ? "light" : "dark");
    rdEl.className = "rd-" + th; const c = rdEl.querySelector(".rdc"); if (c) c.style.fontSize = rdSize + "px";
  };
  const readerOpen = () => {
    if (readerOn()) return readerClose();
    if (!web()) return toast(L.nohttp, "err");
    const blocks = readerBlocks();
    if (!blocks.length) return toast(L.rdEmpty, "err");
    rdText = blocks.map((b) => b[1]).join("\n");
    rdEl.textContent = "";
    const bar = el("div", null, "rdbar");
    const th = () => { const o = ["dark", "light", "sepia"]; rdTheme = o[(o.indexOf(rdTheme || (matchMedia("(prefers-color-scheme:light)").matches ? "light" : "dark")) + 1) % 3]; send({ cmd: "pref", k: "rd_theme", v: rdTheme }); readerPaint(); };
    const sz = (d) => { rdSize = Math.max(12, Math.min(40, rdSize + d)); send({ cmd: "pref", k: "rd_size", v: rdSize }); readerPaint(); };
    const b1 = btn("A−", () => sz(-2)); b1.title = L.rdSmall; const b2 = btn("A+", () => sz(2)); b2.title = L.rdBig;
    const b3 = btn("◐", th); b3.title = L.rdTheme;
    bar.append(b1, b2, b3);
    if (tts()) { const b4 = btn("🔊", speakToggle); b4.title = L.rdA + " / " + L.rdStop; bar.append(b4); }
    const x = btn("✕", readerClose); x.title = "Esc"; bar.append(el("span", "", "sp"), x);
    const art = el("div", null, "rdc");
    art.append(el("h1", (document.title || location.hostname).slice(0, 200)));
    for (const [tag, t] of blocks) art.append(el(tag === "li" ? "p" : /^h[1-4]$/.test(tag) ? "h2" : tag === "pre" ? "pre" : tag === "blockquote" ? "blockquote" : "p", (tag === "li" ? "• " : "") + t));
    rdEl.append(bar, art); rdEl.style.display = "block"; readerPaint(); rdEl.scrollTop = 0;
  };

  // ---- find in page
  const findCount = (q) => { if (!q) return 0; const t = (document.body ? document.body.innerText : "").toLowerCase(); const needle = q.toLowerCase(); let n = 0, i = 0; while ((i = t.indexOf(needle, i)) >= 0) { n++; i += needle.length; } return n; };
  const findUpdate = () => { fcnt.textContent = !fin.value ? "" : fn ? (fi ? L.findN.replace("{i}", fi).replace("{n}", fn) : String(fn)) : L.findNone; };
  const findGo = (back, fresh) => {
    const q = fin.value; fn = findCount(q); if (fresh) fi = 0;
    if (!q || !fn) { fi = 0; return findUpdate(); }
    let ok = false; try { ok = window.find(q, false, !!back, true, false, false, false); } catch (e) { /* */ }
    if (ok) fi = back ? (fi <= 1 ? fn : fi - 1) : (fi >= fn ? 1 : fi + 1);
    findUpdate();
  };
  const findOpen = () => { if (!web()) return toast(L.nohttp, "err"); frow.classList.add("on"); const s0 = String(getSelection()).trim(); if (s0 && s0.length < 80) fin.value = s0; fin.focus(); fin.select(); if (fin.value) findGo(false, true); };
  const findClose = () => { frow.classList.remove("on"); fi = 0; try { getSelection().removeAllRanges(); } catch (e) { /* */ } };

  // ---- tab search (open tabs + library), tab colour menu, more-tools menu
  const tsOpen = () => { closeMenus(); tsEl.style.display = "block"; const i = tsEl.querySelector("input"); i.value = ""; renderTs(null); i.focus(); };
  const tsClose = () => { tsEl.style.display = "none"; };
  let tsHits = null, tsTimer;
  const renderTs = (hits) => {
    if (hits) tsHits = hits; else tsHits = null;
    if (tsEl.style.display !== "block") return;
    const q = tsEl.querySelector("input").value.trim().toLowerCase(), list = tsEl.querySelector(".list"); list.textContent = "";
    const tl = tabList.filter((t) => !q || (t.title + " " + t.url).toLowerCase().includes(q));
    list.append(el("h3", L.tsTabs));
    for (const t of tl) { const r = el("div", null, "item"); r.append(el("span", (t.color ? "● " : "") + (t.title || t.url || "·")), el("small", t.url)); if (t.color) r.firstChild.style.color = COL[t.color]; r.onclick = () => { tsClose(); send({ cmd: "activate", id: t.id }); }; list.append(r); }
    if (!tl.length) list.append(el("p", L.tsNone, "hint"));
    if (q) {
      list.append(el("h3", L.tsLib));
      for (const h of tsHits || []) { const r = el("div", null, "item"); r.append(el("span", h.path.split("/").pop()), el("small", h.snippet.slice(0, 140))); r.title = L.copyPath; r.onclick = () => copy(h.path); list.append(r); }
      if (tsHits && !tsHits.length) list.append(el("p", L.tsNone, "hint"));
    }
  };
  const closeMenus = () => { menuEl.style.display = "none"; ctxEl.style.display = "none"; };
  const placeMenu = (m, x, y) => { m.style.display = "block"; m.style.left = Math.max(4, Math.min(innerWidth - 270, x)) + "px"; m.style.top = y + "px"; };
  const tabMenu = (t, ev) => {
    closeMenus(); ctxEl.textContent = "";
    const dots = el("div", null, "dots2"); dots.append(el("small", L.tabColor));
    for (let k = 0; k <= 6; k++) { const d = el("i", k ? "" : "∅"); d.style.background = COL[k] || "transparent"; d.title = k ? "" : L.noColor; d.onclick = () => { closeMenus(); send({ cmd: "tabcolor", id: t.id, color: k }); }; dots.append(d); }
    ctxEl.append(dots);
    const item = (txt, fn) => { const b = el("div", txt, "mi"); b.onclick = () => { closeMenus(); fn(); }; ctxEl.append(b); };
    if (t.color) { item("💾 " + L.tabSave, () => act("save_tabs", null, t.color)); item("✨ " + L.tabAsk, () => { groupColor = t.color; qrow.classList.add("on"); qin.placeholder = L.qgrp; qin.focus(); }); }
    if (t.id !== activeTab && !t.sleep) item("💤 " + L.discard, () => send({ cmd: "discard", id: t.id }));
    placeMenu(ctxEl, ev.clientX, 40);
  };
  const moreMenu = () => {
    if (menuEl.style.display === "block") return closeMenus();
    closeMenus(); menuEl.textContent = "";
    const item = (ico, txt, fn) => { const b = el("div", ico + " " + txt, "mi"); b.onclick = () => { closeMenus(); fn(); }; menuEl.append(b); };
    item("📖", L.later, () => act("later")); item("📄", L.rd, readerOpen); item("🔎", L.find, findOpen); item("❝", L.cite, citeOpen); item("🔗", L.copyClean, copyClean);
    item("🗂", L.tsearch, tsOpen); item("💤", L.discardAll, () => send({ cmd: "discard" })); item("⧉", L.dupes, () => send({ cmd: "dedupe" }));
    item("🛡", L.trackers + " (" + (prefs.trackers ? L.on : L.off) + ")", () => send({ cmd: "trackers" }));
    placeMenu(menuEl, innerWidth - 280, 78);
  };
  // ---- floating mini-menu on a text selection
  const selHide = () => { if (selm) selm.style.display = "none"; };
  const selShow = () => {
    if (!sh || !web()) return;
    const s0 = getSelection(), t = String(s0 || "").trim();
    if (t.length < 2 || t.length > 4000 || !s0.rangeCount) return selHide();
    const r = s0.getRangeAt(0).getBoundingClientRect();
    if (!r.width && !r.height) return selHide();
    selTxt = t; selm.style.display = "flex";
    selm.style.left = Math.max(6, Math.min(innerWidth - 330, r.left)) + "px"; selm.style.top = Math.max(84, Math.min(innerHeight - 50, r.bottom + 8)) + "px";
  };
  const selDo = (k) => {
    const t = selTxt; selHide(); if (!t) return;
    if (k === "ask") { qrow.classList.add("on"); qin.value = L.selAskPre + "«" + t.slice(0, 240) + "» "; qin.focus(); }
    else if (k === "note") send({ cmd: "note", url: cleanUrl(location.href), title: document.title, sel: t });
    else if (k === "hl") act("highlight");
    else { show(L.think); send({ cmd: "sel", mode: k, sel: t, lang, title: document.title }); }
  };
  hidden("__cc_boot_", (b) => {
    if (!sh) return;
    mock = !!b.mock; prefs = b.prefs || {}; if (prefs.rd_size) rdSize = +prefs.rd_size || 19; rdTheme = prefs.rd_theme || "";
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
      .acts{display:flex;gap:8px;margin-top:8px}.pre{white-space:pre-wrap;margin:4px 0}.warnl{color:#fdb93a;margin:6px 0}.dh{font-weight:700;color:var(--acc);margin:8px 0 4px}
      .diff{max-height:30vh;overflow:auto;border:1px solid var(--bd);border-radius:8px;padding:6px 8px;font:12px ui-monospace,monospace;background:var(--bg)}.diff div{white-space:pre-wrap;font:12px ui-monospace,monospace}.add{color:#3dd68c}.del{color:#e5484d}
      .cite{margin:6px 0;display:flex;gap:8px;align-items:flex-start}.cite .pre{flex:1;font-size:12px}
      .tab.zz{opacity:.55}.item b{position:absolute;right:10px;color:var(--acc)}.item{position:relative}
      #selm,#menu,#ctx{display:none;position:fixed;z-index:6;background:var(--bg2);color:var(--fg);border:1px solid var(--bd);border-radius:10px;box-shadow:0 8px 28px var(--sh)}
      #selm{gap:4px;padding:5px}#menu,#ctx{width:262px;padding:4px}.mi{padding:7px 10px;border-radius:7px;cursor:pointer}.mi:hover{background:var(--bgb)}
      .dots2{display:flex;gap:6px;align-items:center;padding:6px 8px;flex-wrap:wrap}.dots2 small{width:100%;color:var(--mut)}.dots2 i{width:20px;height:20px;border-radius:50%;border:1px solid var(--bd);cursor:pointer;text-align:center;font-style:normal;font-size:11px;line-height:18px}
      #frow input{max-width:320px}#fcnt{color:var(--mut);min-width:80px}
      #reader{display:none;position:fixed;inset:0;z-index:1;overflow:auto;padding-top:80px}
      #reader.rd-dark{background:#14161b;color:#e6e9ef}#reader.rd-light{background:#fbfbf8;color:#1b1f2a}#reader.rd-sepia{background:#f4ecd8;color:#433422}
      .rdbar{position:sticky;top:0;display:flex;gap:8px;padding:8px 14px;background:inherit;border-bottom:1px solid #8884}.rdbar .sp{flex:1}
      .rdc{max-width:42em;margin:0 auto;padding:12px 22px 80px;line-height:1.65;font-family:Georgia,serif;font-size:19px}.rdc *{font-family:inherit;font-size:inherit}.rdc h1{font-size:1.7em;line-height:1.25}.rdc h2{font-size:1.3em;margin-top:1.4em}.rdc blockquote{border-left:3px solid #8886;margin:1em 0;padding-left:1em;font-style:italic}.rdc pre{white-space:pre-wrap;font-family:ui-monospace,monospace}
      #tsrch{display:none;position:fixed;z-index:7;top:90px;left:50%;transform:translateX(-50%);width:min(620px,94vw);max-height:70vh;overflow:auto;background:var(--bg2);color:var(--fg);border:1px solid var(--bd);border-radius:12px;box-shadow:0 8px 28px var(--sh);padding:12px}#tsrch input{width:100%;flex:none}
      .item span,.item small{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.item small{color:var(--mut)}
    </style>
    <div id="start"></div><div id="tabs"></div>
    <div class="bar"><button id="b">◀</button><button id="f">▶</button><button id="r">⟳</button>
      <input id="u" spellcheck="false"><span id="badge"></span><button id="s" class="go"></button><button id="a"></button><button id="hl"></button><button id="wt"></button><button id="hi"></button><button id="sg"></button><button id="hp"></button><button id="mm">☰</button></div>
    <div class="row" id="frow"><input id="fin" spellcheck="false"><button id="fp">▲</button><button id="fnx">▼</button><span id="fcnt"></span><button id="fx">✕</button></div>
    <div class="row" id="qrow"><input id="q"><button id="qq" class="go">➤</button><button id="qa"></button><button id="qs"></button><button id="qx"></button></div><div id="panel"></div>
    <div id="toast"></div><div id="tour"></div><div id="help"></div>
    <div id="selm"></div><div id="menu"></div><div id="ctx"></div><div id="reader"></div><div id="tsrch"><input id="tsin"><div class="list"></div></div>`;
    $ = (i) => sh.getElementById(i);
    panel = $("panel"); qrow = $("qrow"); qin = $("q"); urlIn = $("u"); tabsEl = $("tabs"); badge = $("badge");
    toastEl = $("toast"); helpEl = $("help"); tourEl = $("tour"); startEl = $("start");
    hostEl = host; selm = $("selm"); menuEl = $("menu"); ctxEl = $("ctx"); rdEl = $("reader"); tsEl = $("tsrch"); frow = $("frow"); fin = $("fin"); fcnt = $("fcnt");
    $("mm").title = L.more; $("mm").onclick = moreMenu;
    fin.placeholder = L.findPh; $("fnx").onclick = () => findGo(false); $("fp").onclick = () => findGo(true); $("fx").onclick = findClose;
    fin.oninput = () => findGo(false, true);
    fin.onkeydown = (e) => { if (e.key === "Enter") { e.preventDefault(); findGo(e.shiftKey); } else if (e.key === "Escape") { e.stopPropagation(); findClose(); } };
    const tin = $("tsin"); tin.placeholder = L.tsPh; tin.oninput = () => { renderTs(null); clearTimeout(tsTimer); tsTimer = setTimeout(() => tin.value.trim() && send({ cmd: "tsearch", q: tin.value.trim() }), 250); };
    tin.onkeydown = (e) => { if (e.key === "Escape") { e.stopPropagation(); tsClose(); } };
    selm.onmousedown = (e) => e.preventDefault();
    for (const [k, ico, tip] of [["ask", "✨", L.selAsk], ["note", "📝", L.selNote], ["hl", "🖍", L.selHl], ["translate", "🌐", L.selTr], ["explain", "💡", L.selEx]]) { const b = el("button", ico + " " + tip); b.onclick = () => selDo(k); selm.append(b); }
    addEventListener("mouseup", (e) => { if (e.target !== host) setTimeout(selShow, 12); }, true);
    addEventListener("keyup", (e) => { if (e.shiftKey && /^Arrow|Home|End/.test(e.key)) setTimeout(selShow, 12); }, true);
    addEventListener("mousedown", (e) => { if (e.target !== host) { selHide(); closeMenus(); } }, true);
    addEventListener("scroll", selHide, true);
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
    $("a").onclick = () => { if (qrow.classList.contains("on")) { qrow.classList.remove("on"); groupColor = 0; qin.placeholder = L.q; } else openAsk(); };
    const ask = () => qin.value.trim() && act("ask", qin.value.trim());
    const askAll = (k) => qin.value.trim() && act("ask_tabs", qin.value.trim(), k);
    $("qq").onclick = ask; $("qa").onclick = () => askAll("synthesize"); $("qs").onclick = () => askAll("summarize"); $("qx").onclick = () => askAll("extract");
    qin.onkeydown = (e) => { if (e.key === "Enter") (e.ctrlKey ? () => askAll("synthesize") : ask)(); };
    badge.onclick = showStatus;
    document.documentElement.appendChild(host);
    document.documentElement.style.setProperty("margin-top", "78px", "important");
    addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        if (tsEl.style.display === "block") { e.preventDefault(); tsClose(); return; }
        if (readerOn()) { e.preventDefault(); readerClose(); return; }
        if (frow.classList.contains("on")) { e.preventDefault(); findClose(); return; }
        if (menuEl.style.display === "block" || ctxEl.style.display === "block") { closeMenus(); return; }
        const open = ti >= 0 || helpOpen() || panel.style.display === "block" || toastEl.className || qrow.classList.contains("on");
        if (!open) return;
        e.preventDefault(); closeTour(); helpEl.className = ""; toastEl.className = ""; show(""); qrow.classList.remove("on"); groupColor = 0; qin.placeholder = L.q;
        return;
      }
      if (e.key === "F1") { e.preventDefault(); help(); return; }
      if (!(e.ctrlKey || e.metaKey)) return;
      const k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
      if (k === "t") { e.preventDefault(); send({ cmd: e.shiftKey ? "reopen" : "newtab" }); }
      else if (k === "w") { e.preventDefault(); send({ cmd: "close", id: T }); }
      else if (k === "Tab") { e.preventDefault(); send({ cmd: "cycle", back: e.shiftKey }); }
      else if (k === "l") { e.preventDefault(); urlIn.focus(); urlIn.select(); }
      else if (k === "f" && !e.shiftKey) { e.preventDefault(); findOpen(); }
      else if (k === "r" && e.shiftKey) { e.preventDefault(); readerOpen(); }
      else if (k === "k" && e.shiftKey) { e.preventDefault(); tsOpen(); }
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
