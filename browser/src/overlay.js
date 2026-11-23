(() => {
  if (window.top !== window) return;
  const N = "__NONCE__";
  const post = window.ipc && window.ipc.postMessage.bind(window.ipc);
  if (!post) return;
  const D = {
    en: ["Save to library", "Ask Colibrì", "Address, or words to search your library", "Ask about this page…", "Saved", "Error", "Thinking…", "Library hits"],
    ro: ["Salvează în bibliotecă", "Întreabă Colibrì", "Adresă sau cuvinte de căutat în bibliotecă", "Întreabă despre pagina asta…", "Salvat", "Eroare", "Se gândește…", "Rezultate din bibliotecă"],
    fr: ["Enregistrer dans la bibliothèque", "Demander à Colibrì", "Adresse, ou mots à chercher dans ta bibliothèque", "Pose une question sur cette page…", "Enregistré", "Erreur", "Réflexion…", "Résultats de la bibliothèque"],
    it: ["Salva nella biblioteca", "Chiedi a Colibrì", "Indirizzo, o parole da cercare nella biblioteca", "Chiedi di questa pagina…", "Salvato", "Errore", "Sto pensando…", "Risultati dalla biblioteca"],
    es: ["Guardar en la biblioteca", "Preguntar a Colibrì", "Dirección, o palabras para buscar en tu biblioteca", "Pregunta sobre esta página…", "Guardado", "Error", "Pensando…", "Resultados de la biblioteca"],
    pt: ["Guardar na biblioteca", "Perguntar ao Colibrì", "Endereço, ou palavras para pesquisar na biblioteca", "Pergunta sobre esta página…", "Guardado", "Erro", "A pensar…", "Resultados da biblioteca"],
    de: ["In Bibliothek speichern", "Colibrì fragen", "Adresse oder Suchwörter für deine Bibliothek", "Frage zu dieser Seite…", "Gespeichert", "Fehler", "Denke nach…", "Treffer aus der Bibliothek"],
    hu: ["Mentés a könyvtárba", "Kérdezd a Colibrìt", "Cím, vagy keresőszavak a könyvtárhoz", "Kérdezz erről az oldalról…", "Mentve", "Hiba", "Gondolkodik…", "Találatok a könyvtárból"],
  };
  const L = D[(navigator.language || "en").slice(0, 2)] || D.en;
  const send = (o) => post(JSON.stringify(Object.assign({ n: N }, o)));
  const pageText = () => String(getSelection()) || (document.body ? document.body.innerText : "");
  let sh, panel, qrow, qin, url;
  const show = (t, cls) => { panel.textContent = t; panel.className = cls || ""; panel.style.display = t ? "block" : "none"; };
  const act = (cmd, q) => {
    if (cmd === "save") { show("…"); send({ cmd, url: location.href, title: document.title, text: pageText() }); }
    else if (cmd === "ask") { show(L[6]); send({ cmd, q, url: location.href, title: document.title, text: pageText() }); }
  };
  Object.defineProperty(window, "__cc_" + N, { enumerable: false, value: (m) => { if (!sh) return; show(m.text, m.ok ? "ok" : "err"); } });
  Object.defineProperty(window, "__cc_do_" + N, { enumerable: false, value: (c, q) => act(c, q) });
  const mount = () => {
    const host = document.createElement("div");
    host.style.cssText = "all:initial;position:fixed;top:0;left:0;right:0;z-index:2147483647";
    sh = host.attachShadow({ mode: "closed" });
    sh.innerHTML = `<style>
      *{box-sizing:border-box;font:14px system-ui,Segoe UI,sans-serif}
      .bar{display:flex;gap:6px;align-items:center;height:44px;padding:6px 10px;background:#0f1115;border-bottom:2px solid #2f66c4;color:#eef1f6}
      button{background:#171a21;color:#eef1f6;border:1px solid #2a2f3a;border-radius:8px;padding:5px 10px;cursor:pointer}
      button:hover{border-color:#5fd7e6}.go{background:#fdb93a;color:#2b1d00;border-color:#fdb93a;font-weight:700}
      input{flex:1;min-width:80px;background:#171a21;color:#eef1f6;border:1px solid #2a2f3a;border-radius:8px;padding:6px 10px}
      .row{display:none;gap:6px;padding:6px 10px;background:#171a21}.row.on{display:flex}
      #panel{display:none;max-height:45vh;overflow:auto;white-space:pre-wrap;padding:10px 14px;background:#171a21;color:#eef1f6;border-bottom:1px solid #2a2f3a}
      #panel.ok{border-left:4px solid #5fd7e6}#panel.err{border-left:4px solid #e5484d}
    </style>
    <div class="bar"><button id="b">◀</button><button id="f">▶</button><button id="r">⟳</button>
      <input id="u" spellcheck="false" placeholder=""><button id="s" class="go"></button><button id="a"></button></div>
    <div class="row" id="qrow"><input id="q" placeholder=""><button id="qq" class="go">➤</button></div><div id="panel"></div>`;
    const $ = (i) => sh.getElementById(i);
    panel = $("panel"); qrow = $("qrow"); qin = $("q"); url = $("u");
    url.value = location.href.startsWith("data:") ? "" : location.href; url.placeholder = L[2];
    $("s").textContent = "💾 " + L[0]; $("a").textContent = "✨ " + L[1]; qin.placeholder = L[3];
    $("b").onclick = () => history.back(); $("f").onclick = () => history.forward(); $("r").onclick = () => location.reload();
    url.onkeydown = (e) => { if (e.key === "Enter") send({ cmd: "go", q: url.value }); };
    $("s").onclick = () => act("save");
    $("a").onclick = () => { qrow.classList.toggle("on"); if (qrow.classList.contains("on")) qin.focus(); };
    const ask = () => qin.value.trim() && act("ask", qin.value.trim());
    $("qq").onclick = ask; qin.onkeydown = (e) => { if (e.key === "Enter") ask(); };
    document.documentElement.appendChild(host);
    document.documentElement.style.setProperty("margin-top", "44px", "important");
  };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", mount); else mount();
})();
