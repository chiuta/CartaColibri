(() => {
  if (window.top !== window) return;
  const N = "__NONCE__", T = __TAB__;
  const post = window.ipc && window.ipc.postMessage.bind(window.ipc);
  if (!post) return;
  const D = {
    en: { save: "Save", ask: "Ask", addr: "Address, or words to search your library", q: "Ask about this page…", think: "Thinking…", newtab: "New tab (Ctrl+T)", sg: "Save all tabs as a library group", aa: "Ask all open tabs", inlib: "In your library", changed: "differs from the saved copy", related: "Related in your library", refresh: "Refresh copy", qall: "Question across all tabs…" },
    ro: { save: "Salvează", ask: "Întreabă", addr: "Adresă sau cuvinte de căutat în bibliotecă", q: "Întreabă despre pagina asta…", think: "Se gândește…", newtab: "Tab nou (Ctrl+T)", sg: "Salvează toate taburile ca grup în bibliotecă", aa: "Întreabă toate taburile deschise", inlib: "În biblioteca ta", changed: "diferă de copia salvată", related: "Înrudite în biblioteca ta", refresh: "Reîmprospătează copia", qall: "Întrebare peste toate taburile…" },
    fr: { save: "Enregistrer", ask: "Demander", addr: "Adresse, ou mots à chercher dans ta bibliothèque", q: "Pose une question sur cette page…", think: "Réflexion…", newtab: "Nouvel onglet (Ctrl+T)", sg: "Enregistrer tous les onglets comme groupe", aa: "Interroger tous les onglets ouverts", inlib: "Dans ta bibliothèque", changed: "diffère de la copie enregistrée", related: "Liés dans ta bibliothèque", refresh: "Actualiser la copie", qall: "Question sur tous les onglets…" },
    it: { save: "Salva", ask: "Chiedi", addr: "Indirizzo, o parole da cercare nella biblioteca", q: "Chiedi di questa pagina…", think: "Sto pensando…", newtab: "Nuova scheda (Ctrl+T)", sg: "Salva tutte le schede come gruppo", aa: "Interroga tutte le schede aperte", inlib: "Nella tua biblioteca", changed: "diversa dalla copia salvata", related: "Correlati nella tua biblioteca", refresh: "Aggiorna la copia", qall: "Domanda su tutte le schede…" },
    es: { save: "Guardar", ask: "Preguntar", addr: "Dirección, o palabras para buscar en tu biblioteca", q: "Pregunta sobre esta página…", think: "Pensando…", newtab: "Nueva pestaña (Ctrl+T)", sg: "Guardar todas las pestañas como grupo", aa: "Preguntar a todas las pestañas", inlib: "En tu biblioteca", changed: "difiere de la copia guardada", related: "Relacionado en tu biblioteca", refresh: "Actualizar la copia", qall: "Pregunta sobre todas las pestañas…" },
    pt: { save: "Guardar", ask: "Perguntar", addr: "Endereço, ou palavras para pesquisar na biblioteca", q: "Pergunta sobre esta página…", think: "A pensar…", newtab: "Novo separador (Ctrl+T)", sg: "Guardar todos os separadores como grupo", aa: "Perguntar a todos os separadores", inlib: "Na tua biblioteca", changed: "difere da cópia guardada", related: "Relacionado na tua biblioteca", refresh: "Atualizar a cópia", qall: "Pergunta sobre todos os separadores…" },
    de: { save: "Speichern", ask: "Fragen", addr: "Adresse oder Suchwörter für deine Bibliothek", q: "Frage zu dieser Seite…", think: "Denke nach…", newtab: "Neuer Tab (Strg+T)", sg: "Alle Tabs als Gruppe speichern", aa: "Alle offenen Tabs befragen", inlib: "In deiner Bibliothek", changed: "weicht von der gespeicherten Kopie ab", related: "Verwandt in deiner Bibliothek", refresh: "Kopie aktualisieren", qall: "Frage über alle Tabs…" },
    hu: { save: "Mentés", ask: "Kérdés", addr: "Cím, vagy keresőszavak a könyvtárhoz", q: "Kérdezz erről az oldalról…", think: "Gondolkodik…", newtab: "Új lap (Ctrl+T)", sg: "Az összes lap mentése csoportként", aa: "Kérdés az összes nyitott laphoz", inlib: "A könyvtáradban", changed: "eltér a mentett példánytól", related: "Kapcsolódó a könyvtáradban", refresh: "Példány frissítése", qall: "Kérdés az összes laphoz…" },
  };
  const L = D[(navigator.language || "en").slice(0, 2)] || D.en;
  const send = (o) => post(JSON.stringify(Object.assign({ n: N, t: T }, o)));
  const pageText = () => String(getSelection()) || (document.body ? document.body.innerText : "");
  let sh, panel, qrow, qin, urlIn, tabsEl, badge, mode = "page", info = null;
  const show = (t, cls) => { panel.textContent = t; panel.className = cls || ""; panel.style.display = t ? "block" : "none"; };
  const act = (cmd, q) => {
    if (cmd === "save") { show("…"); send({ cmd, url: location.href, title: document.title, text: pageText() }); }
    else if (cmd === "ask") { show(L.think); send({ cmd, q, url: location.href, title: document.title, text: pageText() }); }
    else if (cmd === "ask_tabs") { show(L.think); send({ cmd, q }); }
    else if (cmd === "save_tabs") { show("…"); send({ cmd }); }
  };
  const hidden = (name, fn) => Object.defineProperty(window, name + N, { enumerable: false, value: fn });
  hidden("__cc_", (m) => { if (sh) show(m.text, m.ok ? "ok" : "err"); });
  hidden("__cc_do_", (c, q) => act(c, q));
  hidden("__cc_tabs_", (s) => {
    if (!sh) return;
    tabsEl.textContent = "";
    for (const t of s.tabs) {
      const el = document.createElement("div"); el.className = "tab" + (t.id === s.active ? " on" : "");
      const lab = document.createElement("span"); lab.textContent = (t.title || t.url || "·").slice(0, 28); lab.title = t.url;
      const x = document.createElement("b"); x.textContent = "×";
      el.onclick = () => send({ cmd: "activate", id: t.id }); x.onclick = (e) => { e.stopPropagation(); send({ cmd: "close", id: t.id }); };
      el.append(lab, x); tabsEl.append(el);
    }
    const plus = document.createElement("button"); plus.textContent = "+"; plus.title = L.newtab; plus.onclick = () => send({ cmd: "newtab" }); tabsEl.append(plus);
  });
  hidden("__cc_info_", (i) => {
    if (!sh) return; info = i; badge.textContent = ""; badge.style.display = "none";
    if (i.status) {
      badge.style.display = "inline-block";
      badge.textContent = "📚 " + L.inlib + (i.status.changed ? " · ⚠ " + L.changed : "");
      badge.className = i.status.changed ? "warn" : "";
      badge.title = i.status.path;
    } else if (i.related && i.related.length) {
      badge.style.display = "inline-block"; badge.textContent = "🔗 " + i.related.length + " " + L.related.toLowerCase(); badge.className = "";
    }
  });
  const mount = () => {
    const host = document.createElement("div");
    host.style.cssText = "all:initial;position:fixed;top:0;left:0;right:0;z-index:2147483647";
    sh = host.attachShadow({ mode: "closed" });
    sh.innerHTML = `<style>
      *{box-sizing:border-box;font:13px system-ui,Segoe UI,sans-serif}
      #tabs{display:flex;gap:4px;align-items:flex-end;height:32px;padding:4px 8px 0;background:#0a0c10;overflow:hidden}
      .tab{display:flex;gap:8px;align-items:center;max-width:210px;padding:5px 10px;border-radius:9px 9px 0 0;background:#171a21;color:#9aa3b2;cursor:pointer;white-space:nowrap}
      .tab.on{background:#0f1115;color:#eef1f6;border-top:2px solid #fdb93a}.tab b{color:#9aa3b2}.tab b:hover{color:#e5484d}
      #tabs>button{background:none;border:0;color:#5fd7e6;font-size:18px;cursor:pointer;padding:0 8px}
      .bar{display:flex;gap:6px;align-items:center;height:44px;padding:6px 10px;background:#0f1115;border-bottom:2px solid #2f66c4;color:#eef1f6}
      button{background:#171a21;color:#eef1f6;border:1px solid #2a2f3a;border-radius:8px;padding:5px 10px;cursor:pointer}
      button:hover{border-color:#5fd7e6}.go{background:#fdb93a;color:#2b1d00;border-color:#fdb93a;font-weight:700}
      input{flex:1;min-width:80px;background:#171a21;color:#eef1f6;border:1px solid #2a2f3a;border-radius:8px;padding:6px 10px}
      #badge{display:none;padding:3px 9px;border-radius:99px;background:#16324a;color:#5fd7e6;cursor:pointer;white-space:nowrap;max-width:34vw;overflow:hidden;text-overflow:ellipsis}#badge.warn{background:#4a3410;color:#fdb93a}
      .row{display:none;gap:6px;padding:6px 10px;background:#171a21}.row.on{display:flex}
      #panel{display:none;max-height:45vh;overflow:auto;white-space:pre-wrap;padding:10px 14px;background:#171a21;color:#eef1f6;border-bottom:1px solid #2a2f3a}
      #panel.ok{border-left:4px solid #5fd7e6}#panel.err{border-left:4px solid #e5484d}
    </style>
    <div id="tabs"></div>
    <div class="bar"><button id="b">◀</button><button id="f">▶</button><button id="r">⟳</button>
      <input id="u" spellcheck="false"><span id="badge"></span><button id="s" class="go"></button><button id="a"></button><button id="sg"></button></div>
    <div class="row" id="qrow"><input id="q"><button id="qq" class="go">➤</button><button id="qa"></button></div><div id="panel"></div>`;
    const $ = (i) => sh.getElementById(i);
    panel = $("panel"); qrow = $("qrow"); qin = $("q"); urlIn = $("u"); tabsEl = $("tabs"); badge = $("badge");
    urlIn.value = location.href.startsWith("data:") ? "" : location.href; urlIn.placeholder = L.addr;
    $("s").textContent = "💾 " + L.save; $("a").textContent = "✨ " + L.ask; $("sg").textContent = "🗂"; $("sg").title = L.sg; $("qa").textContent = "🗂 ✨"; $("qa").title = L.aa; qin.placeholder = L.q;
    $("b").onclick = () => history.back(); $("f").onclick = () => history.forward(); $("r").onclick = () => location.reload();
    urlIn.onkeydown = (e) => { if (e.key === "Enter") send({ cmd: "go", q: urlIn.value }); };
    $("s").onclick = () => act("save");
    $("sg").onclick = () => act("save_tabs");
    $("a").onclick = () => { qrow.classList.toggle("on"); if (qrow.classList.contains("on")) qin.focus(); };
    const ask = () => qin.value.trim() && act("ask", qin.value.trim());
    const askAll = () => qin.value.trim() && act("ask_tabs", qin.value.trim());
    $("qq").onclick = ask; $("qa").onclick = askAll; qin.onkeydown = (e) => { if (e.key === "Enter") (e.ctrlKey ? askAll : ask)(); };
    badge.onclick = () => {
      if (!info) return;
      const lines = [];
      if (info.status) lines.push("📚 " + info.status.path + (info.status.changed ? "\n⚠ " + L.changed + " — 💾 " + L.refresh : ""));
      if (info.related && info.related.length) { lines.push("", "🔗 " + L.related + ":"); for (const r of info.related) lines.push(" • " + r.score + "%  " + r.path + "\n     " + r.snippet); }
      show(lines.join("\n"), "ok");
    };
    document.documentElement.appendChild(host);
    document.documentElement.style.setProperty("margin-top", "78px", "important");
    addEventListener("keydown", (e) => {
      if (!(e.ctrlKey || e.metaKey)) return;
      if (e.key === "t") { e.preventDefault(); send({ cmd: "newtab" }); }
      else if (e.key === "w") { e.preventDefault(); send({ cmd: "close", id: T }); }
      else if (e.key === "Tab") { e.preventDefault(); send({ cmd: "cycle", back: e.shiftKey }); }
      else if (e.key === "l") { e.preventDefault(); urlIn.focus(); urlIn.select(); }
    }, true);
    send({ cmd: "ready" });
  };
  const report = () => send({ cmd: "loaded", url: location.href, title: document.title, text: (document.body ? document.body.innerText : "").slice(0, 300000) });
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", mount); else mount();
  addEventListener("load", () => setTimeout(report, 400));
})();
