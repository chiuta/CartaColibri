#!/usr/bin/env python3
"""End-to-end browser tests for CartaColibri: starts the real release server on a free port with a temporary
--home and a generated fixture library, then drives the web UI (playwright for Python + a system Chrome) and
asserts through the page itself.

  python3 tests/e2e/run_e2e.py [--bin target/release/cartacolibri] [--langs en,ro] [--smoke fr,it,es,pt,de,hu]
                               [--shots DIR] [--headed]

Full flow (en and ro): first-run wizard, search, did-you-mean, document drawer (tag / star / note, persisted across
a reload), saved search, Task tab (glossary with the offline mock -> [S1]), Chat tab (cited answer and a "not found"
box), Export download, Stats SVG bars, Backup zip, no uncaught JS errors.
Smoke (the other languages): page loads, language applied, no raw translation keys, no JS errors.
Exit code: 0 when every check passed, 1 otherwise."""
import argparse, contextlib, os, shutil, socket, subprocess, sys, tempfile, time, traceback, zipfile
from playwright.sync_api import sync_playwright, TimeoutError as PWTimeout

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ALL = ["en", "ro", "fr", "it", "es", "pt", "de", "hu"]

FIXTURES = {
    "contract_en.txt": "Supply contract (English)\n\nPayment terms: the invoice is payable within 30 days of delivery. Late payment interest of 2 percent per month applies.\n"
                       "The supplier delivers the goods to the warehouse in Bucharest. Termination requires 60 days written notice.\n"
                       "Glossary: invoice means the document requesting payment. Warranty means the promise to repair defects.\n",
    "contract_ro.txt": "Contract de furnizare (română)\n\nTermenele de plată: factura se plătește în 30 de zile de la livrare. Penalitățile de întârziere sunt de 2 la sută pe lună.\n"
                       "Furnizorul livrează marfa la depozitul din București. Încetarea contractului necesită un preaviz scris de 60 de zile.\n",
    "hummingbirds.md": "# Hummingbirds\n\nHummingbirds are tiny birds that hover in front of flowers and drink nectar. Their wings beat dozens of times per second.\n\n"
                       "## Migration\n\nSome hummingbirds fly across the Gulf of Mexico every spring.\n",
    "colibri.html": "<html><head><meta charset='utf-8'><title>Colibri</title></head><body><h1>Colibrii</h1>"
                    "<p>Colibrii sunt păsări mici care zboară înapoi și bat din aripi foarte repede. Se hrănesc cu nectar.</p></body></html>",
}
GROUP = {"en": "Contracts", "ro": "Contracte"}
QUERY = {"en": "payment", "ro": "plată"}
TYPO = {"en": "paymnet", "ro": "platta"}   # a typo of a word that exists in the fixtures
CHAT_Q = {"en": "What are the payment terms?", "ro": "Care sunt termenele de plată?"}
CHAT_NF = "zxqvk quasar nebulae"


def free_port():
    s = socket.socket(); s.bind(("127.0.0.1", 0)); p = s.getsockname()[1]; s.close(); return p


class Server:
    def __init__(self, binp, home):
        self.port = free_port(); self.home = home
        self.p = subprocess.Popen([binp, "--home", home, "serve", "--port", str(self.port)], stdout=subprocess.DEVNULL, stderr=open(f"/tmp/e2e-server-{self.port}.log", "wb"), env=dict(os.environ, CARTACOLIBRI_DEBUG_REQ="1"))
        for _ in range(100):
            with contextlib.suppress(OSError):
                socket.create_connection(("127.0.0.1", self.port), 0.2).close(); break
            if self.p.poll() is not None: raise RuntimeError("server exited: " + self.p.stderr.read().decode(errors="replace"))
            time.sleep(0.1)
        else:
            raise RuntimeError("server did not start")
        self.base = f"http://127.0.0.1:{self.port}"

    def stop(self):
        self.p.terminate()
        with contextlib.suppress(Exception): self.p.wait(5)


class Report:
    def __init__(self, shots): self.rows = []; self.shots = shots

    @contextlib.contextmanager
    def check(self, lang, name, page=None):
        t = time.time()
        try:
            yield
            self.rows.append((lang, name, True, "", time.time() - t))
        except Exception as e:  # noqa: BLE001 - any failure is a failed check
            msg = (str(e).strip().splitlines() or [type(e).__name__])[0][:160]
            self.rows.append((lang, name, False, msg, time.time() - t))
            if page is not None:
                with contextlib.suppress(Exception): page.screenshot(path=os.path.join(self.shots, f"FAIL-{lang}-{name.replace(' ', '_')[:40]}.png"))
            if os.environ.get("E2E_TRACE"): traceback.print_exc()


def new_page(browser, base, lang, errors, path="/carta.html#library", accept_downloads=True):
    ctx = browser.new_context(viewport={"width": 1360, "height": 820}, accept_downloads=accept_downloads)
    ctx.add_init_script(f"localStorage.setItem('carta.lang','{lang}')")
    pg = ctx.new_page(); pg.set_default_timeout(10000)

    def on_console(m):
        if m.type != "error": return
        t = m.text + " " + (m.location.get("url", "") if m.location else "")
        if "503" in t or "/health" in t: return   # the engine is not running: the expected 503 on /health
        errors.append("console: " + m.text[:200])
    pg.on("console", on_console)
    pg.on("pageerror", lambda e: errors.append("pageerror: " + str(e)[:200]))
    pg.goto(base + path)
    pg.wait_for_selector("#lang")
    return ctx, pg


def raw_keys(pg):
    """data-i elements whose visible text is still the translation key (e.g. 'k_something')."""
    return pg.evaluate("""()=>[...document.querySelectorAll('[data-i]')].filter(e=>e.dataset.tip===undefined&&e.textContent.trim()===e.dataset.i&&/^[a-z0-9]+(_[a-z0-9]+)+$/.test(e.dataset.i)).map(e=>e.dataset.i).slice(0,10)""")


def reload_ready(pg):
    pg.reload(); pg.wait_for_selector("#lang"); pg.wait_for_selector("#libRows tr", state="attached")


def full_flow(browser, binp, lang, rep, fixdir, shots):
    home = tempfile.mkdtemp(prefix="cc-e2e-home-"); srv = Server(binp, home); errors = []
    ctx = pg = None
    L = lang
    try:
        # ---- first-run wizard (fresh, empty library) ----
        ctx, pg = new_page(browser, srv.base, lang, errors, "/carta.html?welcome=1")
        with rep.check(L, "language applied", pg):
            assert pg.evaluate("document.documentElement.lang") == lang
            assert pg.input_value("#lang") == lang
            bad = raw_keys(pg); assert not bad, f"raw keys visible: {bad}"
        with rep.check(L, "wizard flows and closes", pg):
            pg.wait_for_selector("#wiz.on")
            pg.click("#wNext")                                   # step 1: add documents
            pg.fill("#wRoot", fixdir); pg.click("#wAdd")
            pg.wait_for_function("()=>document.querySelector('#wReady').classList.contains('ok')", timeout=30000)
            pg.wait_for_function("()=>!document.querySelector('#wNext').disabled")
            pg.click("#wNext")                                   # step 2: search
            pg.fill("#wQ", QUERY[lang]); pg.press("#wQ", "Enter"); pg.wait_for_selector("#wRes li b", timeout=10000)
            pg.screenshot(path=f"{shots}/{lang}-wizard-search.png")
            pg.click("#wNext")                                   # step 3: ask
            pg.click("#wSum"); pg.wait_for_selector("#wOut .src, #wOut p, #wOut li", timeout=20000)
            pg.wait_for_function("()=>!document.querySelector('#wSum').disabled", timeout=20000)
            pg.screenshot(path=f"{shots}/{lang}-wizard-ask.png")
            pg.click("#wFinish"); pg.wait_for_selector("#wiz", state="hidden")
        # ---- library + search ----
        ctx.close(); ctx, pg = new_page(browser, srv.base, lang, errors, "/carta.html#library")
        with rep.check(L, "library lists fixtures", pg):
            pg.wait_for_selector("#libRows tr", timeout=15000)
            assert pg.locator("#libRows tr").count() >= len(FIXTURES), "fewer rows than fixture files"
            pg.screenshot(path=f"{shots}/{lang}-library.png")
        with rep.check(L, "search finds a doc", pg):
            pg.click('.rb[data-view=fetch]'); pg.fill("#q", QUERY[lang]); pg.press("#q", "Enter")
            pg.wait_for_selector("#resRows tr", timeout=10000)
            assert pg.locator("#resRows tr").count() >= 1
            pg.screenshot(path=f"{shots}/{lang}-search.png")
        with rep.check(L, "typo shows did-you-mean banner", pg):
            pg.fill("#q", TYPO[lang]); pg.press("#q", "Enter")
            pg.wait_for_function("()=>{const d=document.querySelector('#dym');return d&&!d.hidden&&d.textContent.trim().length>0}", timeout=10000)
            pg.screenshot(path=f"{shots}/{lang}-dym.png")
        # ---- document drawer ----
        pg.fill("#q", "payment OR plată OR pay*"); pg.press("#q", "Enter"); pg.wait_for_selector("#resRows tr")
        pg.locator("#resRows tr").first.dblclick(position={"x": 60, "y": 10})
        with rep.check(L, "document drawer opens", pg):
            pg.wait_for_selector("#viewer.on", timeout=10000); pg.wait_for_function("()=>document.querySelector('#vText').textContent.length>10")
            pg.wait_for_function("()=>document.querySelector('#vTags').textContent.trim().length>0")
        with rep.check(L, "tag chip appears and persists after reload", pg):
            pg.fill("#vTagIn", "e2etag"); pg.click("#vTagAdd")
            pg.wait_for_selector("#vTags .tagc", timeout=5000)
            assert "e2etag" in pg.inner_text("#vTags")
            pg.screenshot(path=f"{shots}/{lang}-drawer.png")
            pg.click("#vClose") if pg.locator("#vClose").is_visible() else pg.keyboard.press("Escape")
            reload_ready(pg)
            pg.click('.rb[data-view=fetch]'); pg.fill("#q", "payment OR plată OR pay*"); pg.press("#q", "Enter"); pg.wait_for_selector("#resRows tr")
            pg.locator("#resRows tr").first.dblclick(position={"x": 60, "y": 10})
            pg.wait_for_selector("#vTags .tagc", timeout=8000)
            assert "e2etag" in pg.inner_text("#vTags"), "tag lost after reload"
        with rep.check(L, "star toggles", pg):
            before = pg.get_attribute("#vStar", "aria-pressed")
            pg.click("#vStar"); pg.wait_for_function(f"()=>document.querySelector('#vStar').getAttribute('aria-pressed')!=='{before}'", timeout=5000)
            pg.click("#vStar"); pg.wait_for_function(f"()=>document.querySelector('#vStar').getAttribute('aria-pressed')==='{before}'", timeout=5000)
        with rep.check(L, "note autosaves and persists", pg):
            pg.fill("#vNote", "e2e note 12345"); pg.wait_for_function("()=>document.querySelector('#vNoteSt').textContent.trim().length>0")
            pg.wait_for_timeout(1800)   # debounce 800 ms + request
            pg.click("#vClose") if pg.locator("#vClose").is_visible() else pg.keyboard.press("Escape")
            reload_ready(pg)
            pg.click('.rb[data-view=fetch]'); pg.fill("#q", "payment OR plată OR pay*"); pg.press("#q", "Enter"); pg.wait_for_selector("#resRows tr")
            pg.locator("#resRows tr").first.dblclick(position={"x": 60, "y": 10})
            pg.wait_for_function("()=>document.querySelector('#vNote').value.includes('e2e note 12345')", timeout=8000)
            pg.click("#vClose") if pg.locator("#vClose").is_visible() else pg.keyboard.press("Escape")
        # ---- saved search ----
        with rep.check(L, "saved search created and run", pg):
            pg.click('.rb[data-view=fetch]'); pg.fill("#q", QUERY[lang]); pg.press("#q", "Enter"); pg.wait_for_selector("#resRows tr")
            pg.fill("#svName", "e2e-saved"); pg.click("#svSave")
            pg.wait_for_selector('#savedList [data-svrun="e2e-saved"]', timeout=5000)
            pg.fill("#q", "zzzzzz"); pg.press("#q", "Enter"); pg.wait_for_timeout(600)
            pg.click('#savedList [data-svrun="e2e-saved"]'); pg.wait_for_selector("#resRows tr", timeout=8000)
            assert pg.input_value("#q") == QUERY[lang]
        # ---- group + task ----
        pg.fill("#q", "payment OR plată OR pay* OR hummingbirds OR colibrii"); pg.press("#q", "Enter"); pg.wait_for_selector("#resRows tr")
        # select every result; a debounced/auto search may still land and reset the selection, so retry until it sticks
        for _ in range(12):
            pg.wait_for_selector("#resRows tr")
            pg.evaluate("()=>{const b=document.querySelector('#resAll');b.checked=true;b.dispatchEvent(new Event('change',{bubbles:true}))}")
            pg.wait_for_timeout(500)
            if pg.evaluate("()=>S.selRes.size") > 0:
                break
        pg.fill("#gName", GROUP[lang]); pg.click("#gFromSel")
        try:
            pg.wait_for_selector('#groupList input[name=grp]:not([value=""])', timeout=15000)
        except Exception:
            print("DEBUG group not created:", pg.evaluate("()=>({toast:(document.querySelector('#toast')||{}).textContent,sel:S.selRes.size,name:document.querySelector('#gName').value,groups:Object.keys(S.lib.groups)})")); raise
        with rep.check(L, "task glossary (mock) cites [S1]", pg):
            pg.click("#cartaBtn"); pg.wait_for_selector("#drawer.on")
            pg.wait_for_selector("#taskSel option[value=glossary]", state="attached", timeout=15000)
            pg.select_option("#taskSel", "glossary", timeout=15000); pg.check("#mock")
            assert pg.locator('#taskSeg button[data-t=synthesize]').count() == 1 and pg.locator('#taskSel option').count() == 14, 'expected 14 tasks'
            pg.check('input[name=scope][value=group]'); self_dbg = lambda: pg.evaluate("()=>({opts:[...document.querySelectorAll('#scopeGroup option')].map(o=>o.textContent),groups:Object.keys(S.lib.groups),rows:document.querySelectorAll('#groupList .row').length})")
            try:
                pg.wait_for_function("(n)=>[...document.querySelectorAll('#scopeGroup option')].some(o=>o.value===n||o.textContent===n)", arg=GROUP[lang], timeout=15000)
            except Exception:
                print("DEBUG scopeGroup:", GROUP[lang], self_dbg()); raise
            pg.select_option("#scopeGroup", GROUP[lang])
            pg.fill("#req", QUERY[lang]); pg.click("#run")
            pg.wait_for_function("()=>document.querySelector('#out').textContent.includes('[S1]')", timeout=20000)
            pg.screenshot(path=f"{shots}/{lang}-task.png")
        with rep.check(L, "task verify-citations + answer language controls present", pg):
            assert pg.locator("#verify").count() == 1 and pg.locator("#ansLang option").count() >= 8
        # ---- chat ----
        with rep.check(L, "chat answers with a cited source", pg):
            pg.click('#dTabs button[data-tab=chat]'); pg.wait_for_selector("#chatPane:not([hidden])")
            if pg.locator("#chatGroup option").count(): pg.select_option("#chatGroup", index=0)
            pg.check("#mock"); pg.fill("#chatIn", CHAT_Q[lang]); pg.click("#chatSend")
            pg.wait_for_selector("#chatLog .msg.b .src", timeout=20000)
            assert pg.locator("#chatLog .msg.b .src .cite").count() >= 1
            pg.screenshot(path=f"{shots}/{lang}-chat.png")
        with rep.check(L, "chat unrelated question shows 'not found'", pg):
            pg.fill("#chatIn", CHAT_NF); pg.click("#chatSend")
            pg.wait_for_selector("#chatLog .nf", timeout=20000)
        pg.click("#dClose") if pg.locator("#dClose").is_visible() else pg.keyboard.press("Escape")
        # ---- export ----
        with rep.check(L, "export dialog downloads a non-empty file", pg):
            pg.click('.rb[data-view=fetch]'); pg.fill("#q", QUERY[lang]); pg.press("#q", "Enter"); pg.wait_for_selector("#resRows tr")
            pg.click("#expRes"); pg.wait_for_selector("#exp.on")
            pg.select_option("#expFmt", "md")
            with pg.expect_download(timeout=15000) as dl:
                pg.click("#expGo")
            path = dl.value.path(); assert path and os.path.getsize(path) > 0, "empty export"
            pg.click("#expClose"); pg.wait_for_selector("#exp", state="hidden")
        # ---- stats + backup ----
        with rep.check(L, "stats renders SVG bars", pg):
            pg.click('.rb[data-view=stats]'); pg.wait_for_selector("#statsOut svg.bars rect.bb", timeout=10000)
            assert pg.locator("#statsOut svg.bars").count() >= 2
            pg.screenshot(path=f"{shots}/{lang}-stats.png")
        with rep.check(L, "backup download is a valid zip", pg):
            with pg.expect_download(timeout=30000) as dl:
                pg.click("#bkGo")
            path = dl.value.path(); assert zipfile.is_zipfile(path), "not a zip"
            with zipfile.ZipFile(path) as z: assert z.testzip() is None and z.namelist(), "zip empty or corrupt"
        with rep.check(L, "index check (verify) reports OK", pg):
            pg.click("#vfGo"); pg.wait_for_function("()=>document.querySelector('#vfOut .ok')", timeout=15000)
        with rep.check(L, "no raw translation keys after navigating", pg):
            bad = raw_keys(pg); assert not bad, f"raw keys visible: {bad}"
        with rep.check(L, "no uncaught JS errors", pg):
            assert not errors, "; ".join(errors[:3])
    finally:
        with contextlib.suppress(Exception): ctx and ctx.close()
        srv.stop(); shutil.rmtree(home, ignore_errors=True)


def smoke(browser, srv, lang, rep, shots):
    errors = []; ctx = pg = None
    try:
        ctx, pg = new_page(browser, srv.base, lang, errors, "/carta.html?welcome=1")
        with rep.check(lang, "smoke: page loads, language applied", pg):
            try: pg.wait_for_selector("#wiz.on", timeout=6000)
            except PWTimeout:   # seen rarely: the first API calls of a fresh page stall for seconds; one reload, but say so
                with contextlib.suppress(Exception):
                    dbg = pg.evaluate("""()=>({rs:document.readyState,hasS:typeof S,view:[...document.querySelectorAll('.view.on,[id^=v-].on')].map(x=>x.id),wiz:document.querySelector('#wiz')&&document.querySelector('#wiz').className,
                        res:performance.getEntriesByType('resource').map(e=>e.name.split('/').slice(-2).join('/')+':'+Math.round(e.duration)).slice(0,30),url:location.href})""")
                    print("DEBUG stall:", dbg, file=sys.stderr)
                print(f"note: {lang} page init stalled once, reloading", file=sys.stderr); pg.reload(); pg.wait_for_selector("#wiz.on", timeout=15000)
            assert pg.evaluate("document.documentElement.lang") == lang and pg.input_value("#lang") == lang
            assert pg.inner_text("#wTitle").strip(), "empty wizard title"
            pg.screenshot(path=f"{shots}/{lang}-smoke-wizard.png")
        with rep.check(lang, "smoke: no raw translation keys", pg):
            bad = raw_keys(pg); assert not bad, f"raw keys visible: {bad}"
            for v in ("fetch", "stats"):
                pg.evaluate("()=>document.querySelector('#wiz').classList.remove('on')")
                pg.click(f'.rb[data-view={v}]'); pg.wait_for_timeout(300)
                bad = raw_keys(pg); assert not bad, f"raw keys in {v}: {bad}"
            vis = pg.evaluate("()=>[...document.querySelectorAll('button,label,h2,h3,option')].map(e=>e.textContent.trim()).filter(t=>/^[a-z0-9]+(_[a-z0-9]+)+$/.test(t)).slice(0,8)")
            assert not vis, f"raw keys as text: {vis}"
        with rep.check(lang, "smoke: no JS errors", pg):
            assert not errors, "; ".join(errors[:3])
    finally:
        with contextlib.suppress(Exception): ctx and ctx.close()


def launch(p, headed):
    for kw in ({"channel": "chrome"}, {"executable_path": "/usr/bin/google-chrome"}, {}):
        try: return p.chromium.launch(headless=not headed, args=["--no-sandbox", "--disable-background-networking", "--disable-sync", "--no-first-run", "--disable-component-update"], **kw)
        except Exception as e:  # noqa: BLE001
            last = e
    raise last


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bin", default=os.environ.get("CARTACOLIBRI_BIN") or os.path.join(os.environ.get("CARGO_TARGET_DIR", os.path.join(ROOT, "target")), "release", "cartacolibri"))
    ap.add_argument("--langs", default="en,ro"); ap.add_argument("--smoke", default="fr,it,es,pt,de,hu")
    ap.add_argument("--shots", default=None); ap.add_argument("--headed", action="store_true")
    a = ap.parse_args()
    if not os.path.exists(a.bin): sys.exit(f"binary not found: {a.bin} (cargo build --release -p cartacolibri)")
    shots = a.shots or tempfile.mkdtemp(prefix="cc-e2e-shots-"); os.makedirs(shots, exist_ok=True)
    fixdir = tempfile.mkdtemp(prefix="cc-e2e-lib-")
    for n, t in FIXTURES.items(): open(os.path.join(fixdir, n), "w", encoding="utf-8").write(t)
    rep = Report(shots); full = [x for x in a.langs.split(",") if x]; sm = [x for x in a.smoke.split(",") if x]
    with sync_playwright() as p:
        browser = launch(p, a.headed)
        try:
            for lang in full:
                try: full_flow(browser, a.bin, lang, rep, fixdir, shots)
                except Exception as e:  # noqa: BLE001
                    rep.rows.append((lang, "harness", False, f"{type(e).__name__}: {e}"[:160], 0))
            if sm:
                home = tempfile.mkdtemp(prefix="cc-e2e-home-"); srv = Server(a.bin, home)
                try:
                    for lang in sm: smoke(browser, srv, lang, rep, shots)
                finally:
                    srv.stop(); shutil.rmtree(home, ignore_errors=True)
        finally:
            browser.close()
    shutil.rmtree(fixdir, ignore_errors=True)
    w = max(len(r[1]) for r in rep.rows) if rep.rows else 10
    print(f"\n{'lang':<5} {'check':<{w}}  result  secs")
    for lang, name, ok, msg, dt in rep.rows:
        print(f"{lang:<5} {name:<{w}}  {'PASS' if ok else 'FAIL'}    {dt:4.1f}" + ("" if ok else "  <- " + msg))
    bad = [r for r in rep.rows if not r[2]]
    print(f"\n{len(rep.rows) - len(bad)}/{len(rep.rows)} checks passed; screenshots in {shots}")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
