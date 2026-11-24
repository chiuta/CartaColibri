//! Friendly CLI text: 8 languages (informal "tu"/"du"/"te"), colour helpers.
//! Language order: en ro fr de es it pt hu.

use std::io::IsTerminal;

pub const LANGS: [&str; 8] = ["en", "ro", "fr", "de", "es", "it", "pt", "hu"];

/// Pick a language index from a user value such as `ro`, `ro_RO.UTF-8`, `FR`.
pub fn lang_index(v: Option<&str>) -> usize {
    let v = v.unwrap_or("").trim().to_lowercase();
    let code: String = v.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    LANGS.iter().position(|l| *l == code).unwrap_or(0)
}

/// Resolve from `--lang`/`CARTA_LANG` value, falling back to `LANG`, then English.
pub fn resolve(flag: Option<&str>) -> usize {
    match flag {
        Some(f) if !f.trim().is_empty() => lang_index(Some(f)),
        _ => lang_index(std::env::var("LANG").ok().as_deref()),
    }
}

fn table(key: &str) -> [&'static str; 8] {
    match key {
        "hello1" => [
            "Welcome to CartaColibri: a private AI library that lives on your computer.",
            "Bun venit la CartaColibri: o bibliotecă AI privată, care trăiește pe calculatorul tău.",
            "Bienvenue dans CartaColibri : une bibliothèque IA privée, qui vit sur ton ordinateur.",
            "Willkommen bei CartaColibri: einer privaten KI-Bibliothek, die auf deinem Computer lebt.",
            "Te damos la bienvenida a CartaColibri: una biblioteca de IA privada que vive en tu ordenador.",
            "Benvenuto in CartaColibri: una biblioteca AI privata che vive sul tuo computer.",
            "Bem-vindo ao CartaColibri: uma biblioteca de IA privada que vive no teu computador.",
            "Üdv a CartaColibriban: egy privát MI-könyvtár, amely a te gépeden él.",
        ],
        "hello2" => [
            "Point it at a folder of documents, then search them and ask questions about them.",
            "Arată-i un folder cu documente, apoi caută în ele și pune-le întrebări.",
            "Indique-lui un dossier de documents, puis cherche dedans et pose-lui des questions.",
            "Zeig ihm einen Ordner mit Dokumenten, dann kannst du suchen und Fragen stellen.",
            "Indícale una carpeta de documentos y busca en ellos o hazles preguntas.",
            "Indicagli una cartella di documenti, poi cerca e fai domande sui tuoi file.",
            "Aponta-lhe uma pasta de documentos e depois pesquisa e faz perguntas sobre eles.",
            "Mutass neki egy dokumentummappát, aztán keress benne és kérdezz a tartalmáról.",
        ],
        "hello3" => [
            "Get started in one step:",
            "Începe dintr-un singur pas:",
            "Démarre en une seule étape :",
            "Starte in einem Schritt:",
            "Empieza en un solo paso:",
            "Comincia con un solo passo:",
            "Começa num só passo:",
            "Kezdd el egyetlen lépéssel:",
        ],
        "status" => [
            "Your library: {0} document(s) in {1} folder(s).",
            "Biblioteca ta: {0} document(e) în {1} folder(e).",
            "Ta bibliothèque : {0} document(s) dans {1} dossier(s).",
            "Deine Bibliothek: {0} Dokument(e) in {1} Ordner(n).",
            "Tu biblioteca: {0} documento(s) en {1} carpeta(s).",
            "La tua biblioteca: {0} documento/i in {1} cartella/e.",
            "A tua biblioteca: {0} documento(s) em {1} pasta(s).",
            "A könyvtárad: {0} dokumentum {1} mappában.",
        ],
        "next" => [
            "Things you can do now:",
            "Ce poți face acum:",
            "Ce que tu peux faire maintenant :",
            "Das kannst du jetzt tun:",
            "Lo que puedes hacer ahora:",
            "Cosa puoi fare adesso:",
            "O que podes fazer agora:",
            "Ezt teheted meg most:",
        ],
        "do_search" => [
            "search your documents",
            "caută în documentele tale",
            "cherche dans tes documents",
            "durchsuche deine Dokumente",
            "busca en tus documentos",
            "cerca nei tuoi documenti",
            "pesquisa nos teus documentos",
            "keress a dokumentumaidban",
        ],
        "do_ui" => [
            "open the web interface",
            "deschide interfața web",
            "ouvre l'interface web",
            "öffne die Weboberfläche",
            "abre la interfaz web",
            "apri l'interfaccia web",
            "abre a interface web",
            "nyisd meg a webes felületet",
        ],
        "do_doctor" => [
            "check that everything works",
            "verifică dacă totul funcționează",
            "vérifie que tout fonctionne",
            "prüfe, ob alles funktioniert",
            "comprueba que todo funciona",
            "controlla che tutto funzioni",
            "verifica se tudo funciona",
            "ellenőrizd, hogy minden működik-e",
        ],
        "ask_folder" => [
            "Which folder holds your documents?",
            "În ce folder ai documentele?",
            "Dans quel dossier sont tes documents ?",
            "In welchem Ordner liegen deine Dokumente?",
            "¿En qué carpeta están tus documentos?",
            "In quale cartella si trovano i tuoi documenti?",
            "Em que pasta estão os teus documentos?",
            "Melyik mappában vannak a dokumentumaid?",
        ],
        "need_folder" => [
            "No folder given, and there is no terminal to ask you.",
            "Nu ai dat niciun folder și nu pot să te întreb (nu e terminal).",
            "Aucun dossier indiqué, et pas de terminal pour te le demander.",
            "Kein Ordner angegeben, und es gibt kein Terminal zum Nachfragen.",
            "No indicaste carpeta y no hay terminal para preguntarte.",
            "Nessuna cartella indicata e non c'è un terminale per chiedertelo.",
            "Não indicaste nenhuma pasta e não há terminal para perguntar.",
            "Nem adtál meg mappát, és nincs terminál, ahol megkérdezhetném.",
        ],
        "indexing" => [
            "Reading your documents (the first time can take a while)...",
            "Îți citesc documentele (prima dată poate dura puțin)...",
            "Lecture de tes documents (la première fois, ça peut prendre un moment)...",
            "Ich lese deine Dokumente (beim ersten Mal kann das dauern)...",
            "Leyendo tus documentos (la primera vez puede tardar)...",
            "Leggo i tuoi documenti (la prima volta può volerci un po')...",
            "A ler os teus documentos (da primeira vez pode demorar)...",
            "Olvasom a dokumentumaidat (először eltarthat egy ideig)...",
        ],
        "indexed" => [
            "Done: {0} new, {1} updated, {2} unchanged, {3} failed.",
            "Gata: {0} noi, {1} actualizate, {2} neschimbate, {3} eșuate.",
            "Terminé : {0} nouveaux, {1} mis à jour, {2} inchangés, {3} en échec.",
            "Fertig: {0} neu, {1} aktualisiert, {2} unverändert, {3} fehlgeschlagen.",
            "Listo: {0} nuevos, {1} actualizados, {2} sin cambios, {3} fallidos.",
            "Fatto: {0} nuovi, {1} aggiornati, {2} invariati, {3} falliti.",
            "Pronto: {0} novos, {1} atualizados, {2} sem alterações, {3} falhados.",
            "Kész: {0} új, {1} frissített, {2} változatlan, {3} sikertelen.",
        ],
        "ui_at" => [
            "Your library is at {0}",
            "Biblioteca ta e la {0}",
            "Ta bibliothèque est à {0}",
            "Deine Bibliothek ist unter {0}",
            "Tu biblioteca está en {0}",
            "La tua biblioteca è su {0}",
            "A tua biblioteca está em {0}",
            "A könyvtárad itt érhető el: {0}",
        ],
        "search_how" => [
            "To search from the terminal:",
            "Ca să cauți din terminal:",
            "Pour chercher depuis le terminal :",
            "Zum Suchen im Terminal:",
            "Para buscar desde la terminal:",
            "Per cercare dal terminale:",
            "Para pesquisar no terminal:",
            "Keresés a terminálból:",
        ],
        "stop_hint" => [
            "Press Ctrl+C to stop.",
            "Apasă Ctrl+C ca să oprești.",
            "Appuie sur Ctrl+C pour arrêter.",
            "Drücke Strg+C zum Beenden.",
            "Pulsa Ctrl+C para detener.",
            "Premi Ctrl+C per fermare.",
            "Prime Ctrl+C para parar.",
            "Nyomd meg a Ctrl+C-t a leállításhoz.",
        ],
        "error" => [
            "error", "eroare", "erreur", "Fehler", "error", "errore", "erro", "hiba",
        ],
        "try" => [
            "Try this:",
            "Încearcă asta:",
            "Essaie ça :",
            "Probier das:",
            "Prueba esto:",
            "Prova così:",
            "Tenta isto:",
            "Próbáld ezt:",
        ],
        "folder_missing" => [
            "I can't find the folder {0}.",
            "Nu găsesc folderul {0}.",
            "Je ne trouve pas le dossier {0}.",
            "Ich finde den Ordner {0} nicht.",
            "No encuentro la carpeta {0}.",
            "Non trovo la cartella {0}.",
            "Não encontro a pasta {0}.",
            "Nem találom a(z) {0} mappát.",
        ],
        "folder_hint" => [
            "check the spelling, then run: cartacolibri add \"/path/to/your/documents\"",
            "verifică numele, apoi rulează: cartacolibri add \"/calea/către/documente\"",
            "vérifie l'orthographe, puis lance : cartacolibri add \"/chemin/vers/tes/documents\"",
            "prüfe die Schreibweise und starte: cartacolibri add \"/pfad/zu/deinen/dokumenten\"",
            "revisa el nombre y ejecuta: cartacolibri add \"/ruta/a/tus/documentos\"",
            "controlla il nome e lancia: cartacolibri add \"/percorso/dei/tuoi/documenti\"",
            "verifica o nome e executa: cartacolibri add \"/caminho/para/os/teus/documentos\"",
            "ellenőrizd a nevet, majd futtasd: cartacolibri add \"/utvonal/a/dokumentumokhoz\"",
        ],
        "gateway_down" => [
            "The Colibrì gateway (the local AI model) is not answering.",
            "Gateway-ul Colibrì (modelul AI local) nu răspunde.",
            "La passerelle Colibrì (le modèle IA local) ne répond pas.",
            "Das Colibrì-Gateway (das lokale KI-Modell) antwortet nicht.",
            "El gateway de Colibrì (el modelo de IA local) no responde.",
            "Il gateway Colibrì (il modello AI locale) non risponde.",
            "O gateway Colibrì (o modelo de IA local) não responde.",
            "A Colibrì átjáró (a helyi MI-modell) nem válaszol.",
        ],
        "gateway_hint" => [
            "start it with `coli web --model DIR`, or add --mock for an offline answer without a model",
            "pornește-l cu `coli web --model DIR` sau adaugă --mock pentru un răspuns offline, fără model",
            "lance-la avec `coli web --model DIR`, ou ajoute --mock pour une réponse hors ligne sans modèle",
            "starte es mit `coli web --model DIR` oder nutze --mock für eine Offline-Antwort ohne Modell",
            "arráncalo con `coli web --model DIR` o añade --mock para una respuesta sin conexión y sin modelo",
            "avvialo con `coli web --model DIR` oppure aggiungi --mock per una risposta offline senza modello",
            "inicia-o com `coli web --model DIR` ou junta --mock para uma resposta offline sem modelo",
            "indítsd el: `coli web --model DIR`, vagy add hozzá a --mock kapcsolót modell nélküli, offline válaszhoz",
        ],
        "no_docs_hint" => [
            "index your folder first: cartacolibri index   (or cartacolibri start)",
            "indexează mai întâi folderul: cartacolibri index   (sau cartacolibri start)",
            "indexe d'abord ton dossier : cartacolibri index   (ou cartacolibri start)",
            "indexiere zuerst deinen Ordner: cartacolibri index   (oder cartacolibri start)",
            "indexa primero tu carpeta: cartacolibri index   (o cartacolibri start)",
            "indicizza prima la cartella: cartacolibri index   (oppure cartacolibri start)",
            "indexa primeiro a tua pasta: cartacolibri index   (ou cartacolibri start)",
            "előbb indexeld a mappát: cartacolibri index   (vagy cartacolibri start)",
        ],
        "port_busy" => [
            "Port {0} is already in use (maybe CartaColibri is already running).",
            "Portul {0} e deja ocupat (poate CartaColibri rulează deja).",
            "Le port {0} est déjà utilisé (CartaColibri tourne peut-être déjà).",
            "Port {0} ist schon belegt (vielleicht läuft CartaColibri schon).",
            "El puerto {0} ya está en uso (quizá CartaColibri ya está en marcha).",
            "La porta {0} è già in uso (forse CartaColibri è già in esecuzione).",
            "A porta {0} já está em uso (talvez o CartaColibri já esteja a correr).",
            "A(z) {0} port foglalt (talán a CartaColibri már fut).",
        ],
        "port_hint" => [
            "pick another port: cartacolibri start --port 8766",
            "alege alt port: cartacolibri start --port 8766",
            "choisis un autre port : cartacolibri start --port 8766",
            "nimm einen anderen Port: cartacolibri start --port 8766",
            "elige otro puerto: cartacolibri start --port 8766",
            "scegli un'altra porta: cartacolibri start --port 8766",
            "escolhe outra porta: cartacolibri start --port 8766",
            "válassz másik portot: cartacolibri start --port 8766",
        ],
        "no_results" => [
            "No matches for \"{0}\".",
            "Nicio potrivire pentru \"{0}\".",
            "Aucun résultat pour « {0} ».",
            "Keine Treffer für \"{0}\".",
            "Sin resultados para \"{0}\".",
            "Nessun risultato per \"{0}\".",
            "Sem resultados para \"{0}\".",
            "Nincs találat erre: \"{0}\".",
        ],
        "loosen_hint" => [
            "use fewer or simpler words, or drop the filters; if you added files recently run: cartacolibri index",
            "folosește mai puține cuvinte sau mai simple, ori scoate filtrele; dacă ai adăugat fișiere, rulează: cartacolibri index",
            "utilise moins de mots ou des mots plus simples, ou retire les filtres ; si tu as ajouté des fichiers : cartacolibri index",
            "nimm weniger oder einfachere Wörter oder lass die Filter weg; neue Dateien? Dann: cartacolibri index",
            "usa menos palabras o más simples, o quita los filtros; si añadiste archivos, ejecuta: cartacolibri index",
            "usa meno parole o più semplici, o togli i filtri; se hai aggiunto file, lancia: cartacolibri index",
            "usa menos palavras ou mais simples, ou tira os filtros; se juntaste ficheiros, executa: cartacolibri index",
            "használj kevesebb vagy egyszerűbb szót, vagy hagyd el a szűrőket; ha újabb fájlokat tettél be: cartacolibri index",
        ],
        "empty_lib" => [
            "Your library is empty, so there is nothing to find yet.",
            "Biblioteca ta e goală, deci nu am ce găsi încă.",
            "Ta bibliothèque est vide, il n'y a encore rien à trouver.",
            "Deine Bibliothek ist leer, es gibt noch nichts zu finden.",
            "Tu biblioteca está vacía, aún no hay nada que encontrar.",
            "La tua biblioteca è vuota, non c'è ancora nulla da trovare.",
            "A tua biblioteca está vazia, ainda não há nada para encontrar.",
            "A könyvtárad üres, még nincs mit találni.",
        ],
        "doctor_title" => [
            "CartaColibri health check",
            "Verificarea stării CartaColibri",
            "Bilan de santé de CartaColibri",
            "CartaColibri-Gesundheitscheck",
            "Revisión de CartaColibri",
            "Controllo di CartaColibri",
            "Verificação do CartaColibri",
            "CartaColibri állapotellenőrzés",
        ],
        "doctor_good" => [
            "Everything important works.",
            "Tot ce e important funcționează.",
            "Tout l'essentiel fonctionne.",
            "Alles Wichtige funktioniert.",
            "Todo lo importante funciona.",
            "Tutto ciò che conta funziona.",
            "Tudo o que é importante funciona.",
            "Minden fontos dolog működik.",
        ],
        "doctor_warns" => [
            "Works, with {0} optional suggestion(s) above.",
            "Funcționează, cu {0} sugestie(sugestii) opțională(e) mai sus.",
            "Ça fonctionne, avec {0} suggestion(s) facultative(s) ci-dessus.",
            "Läuft, mit {0} optionalen Hinweis(en) oben.",
            "Funciona, con {0} sugerencia(s) opcional(es) arriba.",
            "Funziona, con {0} suggerimento/i facoltativo/i sopra.",
            "Funciona, com {0} sugestão(ões) opcional(is) acima.",
            "Működik, {0} opcionális javaslattal fent.",
        ],
        "doctor_bad" => [
            "{0} problem(s) must be fixed first (see FAIL above).",
            "{0} problemă(e) trebuie rezolvată(e) întâi (vezi FAIL mai sus).",
            "{0} problème(s) à corriger d'abord (voir FAIL ci-dessus).",
            "{0} Problem(e) müssen zuerst behoben werden (siehe FAIL oben).",
            "Hay que arreglar {0} problema(s) primero (mira FAIL arriba).",
            "{0} problema/i da risolvere prima (vedi FAIL sopra).",
            "{0} problema(s) a resolver primeiro (vê FAIL acima).",
            "{0} problémát előbb ki kell javítani (lásd FAIL fent).",
        ],
        _ => ["", "", "", "", "", "", "", ""],
    }
}

/// Translate `key` into language index `lang`, replacing `{0}`, `{1}`... with `args`.
pub fn t(lang: usize, key: &str, args: &[&str]) -> String {
    // one pass over the template: an argument that itself contains "{1}" is not substituted again
    let tpl = table(key)[lang.min(7)];
    let mut out = String::with_capacity(tpl.len());
    let mut rest = tpl;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let hit = tail.find('}').and_then(|j| {
            tail[1..j]
                .parse::<usize>()
                .ok()
                .and_then(|n| args.get(n))
                .map(|a| (j, *a))
        });
        match hit {
            Some((j, a)) => {
                out.push_str(a);
                rest = &tail[j + 1..];
            }
            None => {
                out.push('{');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Text that is safe to print on a terminal: control characters (ANSI escape sequences, bell,
/// backspace...) coming from documents or file names are dropped; newline and tab stay.
pub fn plain(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .collect()
}

/// ANSI colour, only on a terminal and when `NO_COLOR` is unset.
#[derive(Clone, Copy)]
pub struct Paint {
    pub on: bool,
}

impl Paint {
    pub fn stdout() -> Paint {
        Paint {
            on: std::io::stdout().is_terminal() && color_allowed(),
        }
    }
    pub fn stderr() -> Paint {
        Paint {
            on: std::io::stderr().is_terminal() && color_allowed(),
        }
    }
    fn wrap(&self, code: &str, s: &str) -> String {
        if self.on {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
    pub fn bold(&self, s: &str) -> String {
        self.wrap("1", s)
    }
    pub fn dim(&self, s: &str) -> String {
        self.wrap("2", s)
    }
    pub fn green(&self, s: &str) -> String {
        self.wrap("32", s)
    }
    pub fn yellow(&self, s: &str) -> String {
        self.wrap("33", s)
    }
    pub fn red(&self, s: &str) -> String {
        self.wrap("31", s)
    }
    pub fn cyan(&self, s: &str) -> String {
        self.wrap("36", s)
    }
}

fn color_allowed() -> bool {
    std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty())
        && std::env::var("TERM").as_deref() != Ok("dumb")
}

/// Turn a raw library error into (friendly message, optional hint).
pub fn friendly(lang: usize, raw: &str) -> (String, Option<String>) {
    if let Some(p) = raw.strip_prefix("not a folder: ") {
        return (
            t(lang, "folder_missing", &[p]),
            Some(t(lang, "folder_hint", &[])),
        );
    }
    if raw.contains("Colibrì gateway") {
        return (
            format!("{} ({raw})", t(lang, "gateway_down", &[])),
            Some(t(lang, "gateway_hint", &[])),
        );
    }
    if raw.contains("no indexed documents") {
        return (raw.to_string(), Some(t(lang, "no_docs_hint", &[])));
    }
    if let Some(rest) = raw.strip_prefix("cannot listen on ") {
        let port = rest
            .split(':')
            .nth(1)
            .and_then(|r| r.split(|c: char| !c.is_ascii_digit()).next())
            .unwrap_or("");
        return (
            t(lang, "port_busy", &[port]),
            Some(t(lang, "port_hint", &[])),
        );
    }
    (raw.to_string(), None)
}

/// Print `error: msg` and the `Try this:` hint to stderr.
pub fn print_error(lang: usize, raw: &str) {
    let p = Paint::stderr();
    let (m, hint) = friendly(lang, &plain(raw));
    eprintln!("{} {m}", p.red(&format!("{}:", t(lang, "error", &[]))));
    let hint = hint.unwrap_or_else(|| "cartacolibri doctor".to_string());
    eprintln!("{} {hint}", p.bold(&t(lang, "try", &[])));
}
