#!/usr/bin/env python3
"""Captures the CartaColibri screens in all 8 languages (playwright + the system Chrome), driving the real UI against a
sample library and the offline `mock` mode. Output: docs/cartacolibri/img/<lang>-<n>.png   (n = 1..4)
Usage: python3 scripts/capture_carta_demo.py [--lib /home/user/Documents]"""
import argparse, os, subprocess, sys, tempfile, time
from playwright.sync_api import sync_playwright
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LANGS = ["en", "ro", "fr", "it", "es", "pt", "de", "hu"]
GROUP = {"en": "Contracts", "ro": "Contracte", "fr": "Contrats", "it": "Contratti", "es": "Contratos", "pt": "Contratos", "de": "Verträge", "hu": "Szerződések"}
ASK = {"en": "Compare the payment terms", "ro": "Compară termenele de plată", "fr": "Compare les délais de paiement", "it": "Confronta i termini di pagamento",
       "es": "Compara los plazos de pago", "pt": "Compara os prazos de pagamento", "de": "Vergleiche die Zahlungsfristen", "hu": "Hasonlítsd össze a fizetési határidőket"}

def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--lib", default="/home/user/Documents"); ap.add_argument("--port", type=int, default=8793); a = ap.parse_args()
    out = os.path.join(ROOT, "docs", "cartacolibri", "img"); os.makedirs(out, exist_ok=True)
    home = tempfile.mkdtemp()
    env = dict(os.environ, CARTACOLIBRI_HOME=home)
    srv = subprocess.Popen([sys.executable, os.path.join(ROOT, "ui", "server.py"), "--port", str(a.port)], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(1.5)
    try:
        base = f"http://127.0.0.1:{a.port}"
        with sync_playwright() as p:
            b = p.chromium.launch(executable_path="/usr/bin/google-chrome", args=["--no-sandbox"])
            for lang in LANGS:
                ctx = b.new_context(viewport={"width": 1360, "height": 820}); ctx.add_init_script(f"localStorage.setItem('carta.lang','{lang}')")
                pg = ctx.new_page(); errs = []; pg.on("pageerror", lambda e: errs.append(str(e)))
                pg.goto(base + "/carta.html#library"); pg.wait_for_selector("#newRoot")
                if lang == LANGS[0]:
                    pg.fill("#newRoot", a.lib); pg.click("#addRoot"); pg.wait_for_selector("#libRows tr")
                else:
                    pg.reload(); pg.wait_for_selector("#libRows tr")
                pg.wait_for_timeout(500); pg.screenshot(path=f"{out}/{lang}-1.png")
                pg.click('.rb[data-view=fetch]'); pg.fill("#q", "payment OR pay*"); pg.press("#q", "Enter"); pg.wait_for_selector("#resRows tr"); pg.wait_for_timeout(300)
                pg.screenshot(path=f"{out}/{lang}-2.png")
                for i in range(3): pg.locator("#resRows input[type=checkbox]").nth(i).check()   # the table re-renders on each tick
                pg.fill("#gName", GROUP[lang]); pg.click("#gFromSel"); pg.wait_for_selector("#groupList .row"); pg.wait_for_timeout(300)
                pg.screenshot(path=f"{out}/{lang}-3.png")
                pg.click("#cartaBtn"); pg.click('#taskSeg button[data-t=synthesize]'); pg.check("#mock")
                pg.check('input[name=scope][value=group]'); pg.fill("#req", ASK[lang]); pg.click("#run"); pg.wait_for_selector("#out .cite, #out li", timeout=15000); pg.wait_for_timeout(400)
                pg.screenshot(path=f"{out}/{lang}-4.png")
                pg.evaluate("n=>fetch('/carta/groups',{method:'DELETE',headers:{'X-Colibri-UI':'1','Content-Type':'application/json'},body:JSON.stringify({name:n})})", GROUP[lang])
                print(lang, "ok", "JS errors:" + str(errs) if errs else "")
                ctx.close()
            # the chat page must load without errors and offer the new upload entry
            ctx = b.new_context(viewport={"width": 1360, "height": 820}); pg = ctx.new_page(); errs = []; pg.on("pageerror", lambda e: errs.append(str(e)))
            pg.goto(base + "/"); pg.wait_for_timeout(1500)
            has = pg.evaluate("!!document.querySelector('#plusPop [data-k=lib]')"); print("chat page: upload-to-library entry:", has, "errors:", errs)
            b.close()
    finally:
        srv.terminate()

if __name__ == "__main__":
    main()
