#!/usr/bin/env python3
"""CartaColibri promo capture: one ~28 s demo video per language (1280x720, captioned, human-paced) of the REAL UI plus
mobile (390x844 @2x) and tablet (820x1180) screenshots. Same approach as capture_carta_demo.py: playwright + system Chrome,
release binary, generated sample library, offline `mock` backend, --home temp dir, localStorage presets (lang/theme/welcomed).
Output: docs/cartacolibri/video/<lang>-demo.mp4 (H.264 yuv420p, crf 30, faststart, no audio) + <lang>-demo.jpg (poster)
        docs/cartacolibri/img/<lang>-m1.png (library) -m2 (search) -m3 (chat drawer) and <lang>-t1.png (tablet library)
Needs: python3 playwright, google-chrome, ffmpeg, target/release/cartacolibri.
Usage: python3 scripts/capture_carta_video.py [--only en,ro] [--bin PATH] [--lib DIR] [--no-video] [--no-mobile] [--crf 30]
Mobile layout problems found (overflow of the page / of elements wider than the viewport) are printed as 'MOBILE-ISSUE' lines."""
import argparse, os, shutil, socket, subprocess, sys, tempfile, time
from playwright.sync_api import sync_playwright
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import capture_carta_demo as D
ROOT = D.ROOT; LANGS = D.LANGS
# 7 short captions per language (informal 'tu'): library, search, group, synthesis, chat, document, stats
CAP = {
 "en": ["Your whole library, searchable in one place.", "Search with a typo? You still find what you meant.", "Turn the results into a group: your own search scope.",
        "RAG-CARTA writes a synthesis with [S1] source citations.", "Chat with your documents: every answer cites its sources.", "Open a document, add a tag and a note.", "Your library at a glance."],
 "ro": ["Toată biblioteca ta, căutabilă într-un singur loc.", "Ai greșit o literă? Tot găsești ce căutai.", "Transformă rezultatele într-un grup: propriul tău domeniu de căutare.",
        "RAG-CARTA scrie o sinteză cu citări de surse [S1].", "Discută cu documentele tale: fiecare răspuns citează sursele.", "Deschide un document, adaugă o etichetă și o notă.", "Biblioteca ta dintr-o privire."],
 "fr": ["Toute ta bibliothèque, consultable au même endroit.", "Une faute de frappe ? Tu trouves quand même ce que tu cherchais.", "Transforme les résultats en groupe : ton propre périmètre de recherche.",
        "RAG-CARTA rédige une synthèse avec des citations de sources [S1].", "Discute avec tes documents : chaque réponse cite ses sources.", "Ouvre un document, ajoute une étiquette et une note.", "Ta bibliothèque en un coup d’œil."],
 "it": ["Tutta la tua biblioteca, ricercabile in un unico posto.", "Hai sbagliato una lettera? Trovi comunque ciò che cercavi.", "Trasforma i risultati in un gruppo: il tuo ambito di ricerca.",
        "RAG-CARTA scrive una sintesi con citazioni delle fonti [S1].", "Chatta con i tuoi documenti: ogni risposta cita le fonti.", "Apri un documento, aggiungi un tag e una nota.", "La tua biblioteca a colpo d’occhio."],
 "es": ["Toda tu biblioteca, con búsqueda en un solo lugar.", "¿Te equivocaste al escribir? Aun así encuentras lo que buscabas.", "Convierte los resultados en un grupo: tu propio ámbito de búsqueda.",
        "RAG-CARTA redacta una síntesis con citas de fuentes [S1].", "Chatea con tus documentos: cada respuesta cita sus fuentes.", "Abre un documento, añade una etiqueta y una nota.", "Tu biblioteca de un vistazo."],
 "pt": ["Toda a tua biblioteca, pesquisável num só lugar.", "Enganaste-te numa letra? Encontras na mesma o que procuravas.", "Transforma os resultados num grupo: o teu próprio âmbito de pesquisa.",
        "O RAG-CARTA escreve uma síntese com citações das fontes [S1].", "Conversa com os teus documentos: cada resposta cita as fontes.", "Abre um documento, adiciona uma etiqueta e uma nota.", "A tua biblioteca num relance."],
 "de": ["Deine ganze Bibliothek, an einem Ort durchsuchbar.", "Tippfehler? Du findest trotzdem, was du meintest.", "Mach aus den Treffern eine Gruppe: dein eigener Suchbereich.",
        "RAG-CARTA schreibt eine Synthese mit Quellenangaben [S1].", "Chatte mit deinen Dokumenten: Jede Antwort nennt ihre Quellen.", "Öffne ein Dokument, füge ein Tag und eine Notiz hinzu.", "Deine Bibliothek auf einen Blick."],
 "hu": ["Az egész könyvtárad, egy helyen kereshetően.", "Elgépelted? Így is megtalálod, amit kerestél.", "Alakítsd a találatokat csoporttá: a saját keresési köröd.",
        "A RAG-CARTA összefoglalót ír [S1] forráshivatkozásokkal.", "Beszélgess a dokumentumaidval: minden válasz megnevezi a forrását.", "Nyiss meg egy dokumentumot, adj hozzá címkét és jegyzetet.", "A könyvtárad egy pillantásra."],
}
T0 = [0]
SPEED = 1.35   # mild speed-up of the raw recording
TYPO = "paymnt"
INIT = """
localStorage.setItem('carta.lang','%s');localStorage.setItem('carta.theme','dark');localStorage.setItem('carta.welcomed','1');
addEventListener('DOMContentLoaded',()=>{
 const c=document.createElement('div');c.id='__cur';c.style.cssText='position:fixed;left:0;top:0;width:18px;height:18px;margin:-3px 0 0 -3px;z-index:2147483646;pointer-events:none;border-radius:50%%;background:rgba(255,200,40,.85);border:2px solid #fff;box-shadow:0 0 8px #000;transition:transform .05s';document.documentElement.appendChild(c);
 addEventListener('mousemove',e=>{c.style.transform='translate('+e.clientX+'px,'+e.clientY+'px)'},true);
 const b=document.createElement('div');b.id='__cap';b.style.cssText='position:fixed;left:50%%;bottom:18px;transform:translateX(-50%%);max-width:88vw;z-index:2147483647;pointer-events:none;background:#000;color:#fff;font:600 22px/1.3 system-ui,sans-serif;padding:10px 20px;border-radius:10px;border:2px solid #ffc828;text-align:center;opacity:0;transition:opacity .4s';document.documentElement.appendChild(b)})
"""


def cap(pg, txt):
    if os.environ.get('CAPDEBUG'): print('  t=%.1f' % (time.time() - T0[0]), txt)
    pg.evaluate("t=>{const b=document.getElementById('__cap');if(!b)return;if(!t){b.style.opacity=0;return}b.textContent=t;b.style.opacity=1}", txt)


def goto_el(pg, sel, steps=14):
    pg.wait_for_selector(sel); el = pg.locator(sel).first; el.scroll_into_view_if_needed(); bb = el.bounding_box()
    pg.mouse.move(bb["x"] + min(bb["width"] / 2, 60), bb["y"] + bb["height"] / 2, steps=steps); pg.wait_for_timeout(150)


def click(pg, sel, pause=500, **kw):
    goto_el(pg, sel); pg.click(sel, **kw); pg.wait_for_timeout(pause)


def typ(pg, sel, txt, delay=55):
    goto_el(pg, sel); pg.click(sel); pg.fill(sel, ""); pg.type(sel, txt, delay=delay); pg.wait_for_timeout(350)


def load(ctx, base, lib, first):
    pg = ctx.new_page(); pg.set_default_timeout(20000); errs = []; pg.on("pageerror", lambda e: errs.append(str(e)))
    pg.goto(base + "/carta.html#library")
    try: pg.wait_for_selector("#newRoot", state="attached", timeout=6000)
    except Exception: pg.reload(); pg.wait_for_selector("#newRoot", state="attached")   # rare first-request stall
    if first and not pg.locator("#libRows tr").count():
        pg.fill("#newRoot", lib); pg.click("#addRoot")
    pg.wait_for_selector("#libRows tr", timeout=60000); pg.wait_for_timeout(500)
    return pg, errs


def demo(b, base, lib, lang, vdir, crf, first):
    G, C = D.GROUP[lang], CAP[lang]
    ctx = b.new_context(viewport={"width": 1280, "height": 720}, record_video_dir=vdir, record_video_size={"width": 1280, "height": 720})
    ctx.add_init_script(INIT % lang); t0 = time.time()
    pg, errs = load(ctx, base, lib, first); T0[0] = time.time(); pg.mouse.move(640, 400); start = time.time() - t0 - 0.3
    marks = {}
    cap(pg, C[0]); pg.mouse.move(500, 300, steps=25); pg.wait_for_timeout(2400)
    cap(pg, C[1]); click(pg, '.rb[data-view=fetch]', 600)
    typ(pg, "#q", TYPO); pg.press("#q", "Enter"); pg.wait_for_selector("#resRows tr"); pg.wait_for_timeout(600)
    dym = pg.locator("#dym").is_visible(); pg.wait_for_timeout(1800)
    cap(pg, C[2]); goto_el(pg, "#resAll"); pg.check("#resAll"); pg.wait_for_timeout(500)
    typ(pg, "#gName", G); click(pg, "#gFromSel", 400); pg.wait_for_selector("#groupList .row"); pg.wait_for_timeout(1500)
    cap(pg, C[3]); click(pg, "#cartaBtn", 700); click(pg, '#taskSeg button[data-t=synthesize]', 300)
    goto_el(pg, "#mock"); pg.check("#mock"); click(pg, 'input[name=scope][value=group]', 200); pg.select_option("#scopeGroup", G); pg.wait_for_timeout(300)
    typ(pg, "#req", ASKS(lang)); click(pg, "#run", 300)
    pg.wait_for_selector("#out .cite, #out li", timeout=20000); marks["poster"] = time.time() - t0 - start + 0.8; pg.wait_for_timeout(2600)
    cap(pg, C[4]); click(pg, '#dTabs button[data-tab=chat]', 500); pg.select_option("#chatGroup", index=0)
    typ(pg, "#chatIn", D.CHAT[lang], 45); click(pg, "#chatSend", 300)
    pg.wait_for_selector("#chatLog .msg.b .src, #chatLog .msg.b .nf", timeout=20000); pg.wait_for_timeout(2200)
    click(pg, "#dClose", 500); pg.wait_for_selector("#drawer", state="hidden")
    cap(pg, C[5]); goto_el(pg, "#resRows tr"); pg.locator("#resRows tr").first.dblclick(position={"x": 70, "y": 12}); pg.wait_for_selector("#viewer.on")
    pg.wait_for_selector("#vRel li"); pg.wait_for_timeout(700)
    typ(pg, "#vTagIn", D.TAG[lang]); click(pg, "#vTagAdd", 400); pg.wait_for_selector("#vTags .tagc")
    click(pg, "#vStar", 300); typ(pg, "#vNote", D.NOTE[lang], 30); pg.wait_for_timeout(1900)
    click(pg, "#vClose", 400); pg.wait_for_selector("#viewer", state="hidden")
    cap(pg, C[6]); click(pg, '.rb[data-view=stats]', 300); pg.wait_for_selector("#statsOut svg.bars rect.bb"); pg.mouse.move(900, 300, steps=20); pg.wait_for_timeout(2600)
    cap(pg, ""); pg.wait_for_timeout(400)
    dur = time.time() - t0 - start
    # undo the demo tag/star/note (not recorded: the video is cut at `dur`) so the next language starts alike
    pg.click('.rb[data-view=fetch]'); pg.locator("#resRows tr").first.dblclick(position={"x": 70, "y": 12}); pg.wait_for_selector("#viewer.on"); pg.wait_for_selector("#vTags .tagc")
    pg.click("#vStar"); pg.click("#vTags [data-tagrm]"); pg.wait_for_selector("#vTags .tagc", state="detached"); pg.fill("#vNote", ""); pg.wait_for_timeout(1600); pg.click("#vClose"); pg.wait_for_selector("#viewer", state="hidden")
    pg.evaluate("async n=>{const h={'X-Colibri-UI':'1','Content-Type':'application/json'};await fetch('/carta/groups',{method:'DELETE',headers:h,body:JSON.stringify({name:n})})}", G)
    vid = pg.video; ctx.close(); raw = vid.path()
    out = os.path.join(ROOT, "docs", "cartacolibri", "video"); os.makedirs(out, exist_ok=True)
    mp4 = f"{out}/{lang}-demo.mp4"
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-ss", f"{start:.2f}", "-t", f"{dur:.2f}", "-i", raw, "-vf", "setpts=PTS/%s,fps=25,scale=1280:720" % SPEED, "-c:v", "libx264", "-preset", "slow",
                    "-crf", str(crf), "-pix_fmt", "yuv420p", "-movflags", "+faststart", "-an", mp4], check=True)
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-ss", f"{marks.get('poster', 15):.2f}", "-i", mp4, "-frames:v", "1", "-q:v", "4", f"{out}/{lang}-demo.jpg"], check=True)
    os.remove(raw)
    print(lang, "video ok %.1fs %d KB" % (dur, os.path.getsize(mp4) // 1024), "did-you-mean shown" if dym else "(no did-you-mean)", "JS errors:" + str(errs) if errs else "")


def ASKS(lang): return D.ASK[lang]


OVER = """()=>{const W=innerWidth,r=[];document.querySelectorAll('body *').forEach(e=>{const s=getComputedStyle(e);if(s.display==='none'||s.visibility==='hidden'||e.closest('[hidden]'))return;
 const b=e.getBoundingClientRect();if(b.width&&b.right>W+1&&!e.closest('.scroll,.tablewrap'))r.push((e.id?'#'+e.id:e.tagName.toLowerCase()+(e.className&&e.className.baseVal===undefined?'.'+String(e.className).split(' ')[0]:''))+' right='+Math.round(b.right))});
 return {docW:document.documentElement.scrollWidth,W,bodyW:document.body.scrollWidth,els:r.slice(0,8)}}"""


def mobile(b, base, lang, issues):
    out = os.path.join(ROOT, "docs", "cartacolibri", "img")
    for tag, vp, shots in (("m", {"width": 390, "height": 844}, 3), ("t", {"width": 820, "height": 1180}, 1)):
        ctx = b.new_context(viewport=vp, device_scale_factor=2 if tag == "m" else 1, is_mobile=True, has_touch=True)
        ctx.add_init_script(f"localStorage.setItem('carta.lang','{lang}');localStorage.setItem('carta.theme','dark');localStorage.setItem('carta.welcomed','1')")
        pg, errs = load(ctx, base, None, False)
        def chk(name):
            r = pg.evaluate(OVER)
            if r["docW"] > r["W"] + 1 or r["els"]: issues.add(f"{tag}/{name}: page width {r['docW']} > viewport {r['W']}; wide elements {r['els']}")
        pg.screenshot(path=f"{out}/{lang}-{tag}1.png"); chk("library")
        if shots == 3:
            pg.tap('.rb[data-view=fetch]'); pg.wait_for_timeout(300); pg.tap("#q"); pg.fill("#q", "payment OR pay*"); pg.press("#q", "Enter")
            pg.wait_for_selector("#resRows tr"); pg.wait_for_timeout(500); pg.screenshot(path=f"{out}/{lang}-m2.png"); chk("search")
            pg.tap("#cartaBtn"); pg.wait_for_timeout(500); pg.tap('#dTabs button[data-tab=chat]'); pg.wait_for_timeout(500)
            pg.check("#mock"); pg.select_option("#chatGroup", index=0); pg.fill("#chatIn", D.CHAT[lang]); pg.tap("#chatSend")
            pg.wait_for_selector("#chatLog .msg.b .src, #chatLog .msg.b .nf", timeout=20000); pg.wait_for_timeout(500)
            r = pg.evaluate("()=>{const d=document.getElementById('drawer').getBoundingClientRect();return [d.left,d.right,d.width,innerWidth]}")
            if r[1] > r[3] + 1 or r[0] < -1: issues.add(f"m/drawer: drawer box left={r[0]:.0f} right={r[1]:.0f} width={r[2]:.0f} vs viewport {r[3]}")
            pg.screenshot(path=f"{out}/{lang}-m3.png"); chk("chat drawer")
            # touch target sizes of key controls
            small = pg.evaluate("()=>[...document.querySelectorAll('.rail .rb,#cartaBtn,#dClose,#chatSend')].filter(e=>e.offsetParent&&(e.getBoundingClientRect().width<32||e.getBoundingClientRect().height<32)).map(e=>(e.id||e.className)+' '+Math.round(e.getBoundingClientRect().width)+'x'+Math.round(e.getBoundingClientRect().height))")
            if small: issues.add("m: touch targets under 32px: " + ", ".join(small))
        if errs: issues.add(f"{tag}: JS errors {errs}")
        ctx.close()
    print(lang, "mobile/tablet ok")


def free_port():
    s = socket.socket(); s.bind(("127.0.0.1", 0)); p = s.getsockname()[1]; s.close(); return p


def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--lib"); ap.add_argument("--bin", default=os.path.join(ROOT, "target", "release", "cartacolibri"))
    ap.add_argument("--only", default=",".join(LANGS)); ap.add_argument("--crf", type=int, default=30); ap.add_argument("--no-video", action="store_true"); ap.add_argument("--no-mobile", action="store_true"); a = ap.parse_args()
    langs = [l for l in a.only.split(",") if l in LANGS]
    tmp = tempfile.mkdtemp(prefix="cc-vid-"); home = os.path.join(tmp, "home"); os.makedirs(home); vdir = os.path.join(tmp, "rec"); lib = a.lib
    if not lib:
        lib = os.path.join(tmp, "Documents")
        for rel, txt in D.SAMPLE.items():
            os.makedirs(os.path.dirname(os.path.join(lib, rel)), exist_ok=True); open(os.path.join(lib, rel), "w", encoding="utf-8").write(txt)
    port = free_port(); issues = set()
    srv = subprocess.Popen([a.bin, "--home", home, "serve", "--port", str(port)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for _ in range(100):
        try: socket.create_connection(("127.0.0.1", port), 0.2).close(); break
        except OSError: time.sleep(0.1)
    try:
        base = f"http://127.0.0.1:{port}"
        with sync_playwright() as p:
            args = ["--no-sandbox", "--disable-background-networking", "--disable-sync", "--no-first-run", "--disable-component-update"]
            try: b = p.chromium.launch(channel="chrome", args=args)
            except Exception: b = p.chromium.launch(executable_path="/usr/bin/google-chrome", args=args)
            # register the library once, outside any recording
            c0 = b.new_context(viewport={"width": 1280, "height": 720}); c0.add_init_script(INIT % "en"); pg0, _ = load(c0, base, lib, True); c0.close()
            for lang in langs:
                if not a.no_video: demo(b, base, lib, lang, vdir, a.crf, False)
                if not a.no_mobile: mobile(b, base, lang, issues)
            b.close()
    finally:
        srv.terminate(); shutil.rmtree(tmp, ignore_errors=True)
    for i in sorted(issues): print("MOBILE-ISSUE", i)


if __name__ == "__main__":
    main()
