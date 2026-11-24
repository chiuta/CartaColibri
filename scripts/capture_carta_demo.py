#!/usr/bin/env python3
"""Captures the CartaColibri screens in all 8 languages (playwright + a system Chrome), driving the real UI of the Rust
server against a generated sample library and the offline `mock` mode.
Output: docs/cartacolibri/img/<lang>-<n>.png   n = 1 library, 2 search, 3 group, 4 task, 5 document drawer, 6 chat, 7 stats, 8 engine dialog, 9 import dialog (web-grab)
Usage: python3 scripts/capture_carta_demo.py [--lib DIR] [--bin PATH] [--only en,ro]   (without --lib a sample library is generated)"""
import argparse, os, shutil, socket, subprocess, sys, tempfile, time
from playwright.sync_api import sync_playwright
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LANGS = ["en", "ro", "fr", "it", "es", "pt", "de", "hu"]
GROUP = {"en": "Contracts", "ro": "Contracte", "fr": "Contrats", "it": "Contratti", "es": "Contratos", "pt": "Contratos", "de": "Verträge", "hu": "Szerződések"}
ASK = {"en": "Compare the payment terms", "ro": "Compară termenele de plată", "fr": "Compare les délais de paiement", "it": "Confronta i termini di pagamento",
       "es": "Compara los plazos de pago", "pt": "Compara os prazos de pagamento", "de": "Vergleiche die Zahlungsfristen", "hu": "Hasonlítsd össze a fizetési határidőket"}
TAG = {"en": "to-review", "ro": "de-revizuit", "fr": "à-revoir", "it": "da-rivedere", "es": "por-revisar", "pt": "a-rever", "de": "prüfen", "hu": "átnézendő"}
NOTE = {"en": "Check the late-payment penalty with legal.", "ro": "Verifică penalitatea de întârziere cu juriștii.", "fr": "Vérifier la pénalité de retard avec le juridique.",
        "it": "Verificare la penale per ritardo con l’ufficio legale.", "es": "Revisar la penalización por demora con el área legal.", "pt": "Verificar a penalização por atraso com o jurídico.",
        "de": "Verzugsstrafe mit der Rechtsabteilung klären.", "hu": "A késedelmi kötbért egyeztetni a jogásszal."}
CHAT = {"en": "Which contract has the longest payment period?", "ro": "Care contract are cel mai lung termen de plată?", "fr": "Quel contrat a le délai de paiement le plus long ?",
        "it": "Quale contratto ha il termine di pagamento più lungo?", "es": "¿Qué contrato tiene el plazo de pago más largo?", "pt": "Qual contrato tem o prazo de pagamento mais longo?",
        "de": "Welcher Vertrag hat die längste Zahlungsfrist?", "hu": "Melyik szerződésnek van a leghosszabb fizetési határideje?"}
SAMPLE = {
    "contracts/supply-contract.md": "# Supply contract\n\nSupplier: Gamma Foods. Buyer: Beta Logistics SA. Total value: 18,500 EUR. Payment terms: invoices are payable within 45 days. Delivery every second Monday. Penalty for delay: 0.1 % per day.\n",
    "contracts/service-agreement.txt": "Service agreement between Alfa Consulting SRL and Beta Logistics SA.\nThe total value is 42,000 EUR. Payment is due 30 days from receipt of invoice. The agreement runs for 12 months and renews automatically unless cancelled.\n",
    "contracts/lease.txt": "Lease of the warehouse in Ploiesti. Monthly rent 3,200 EUR, payable by the 5th. Term: 36 months. Notice period: 90 days. Payment by bank transfer only.\n",
    "notes/project-notes.md": "# Project notes\n\nKick-off on Monday. Review the contracts: the payment terms differ between the supply contract and the service agreement. Ask legal about the penalty.\n",
    "articles/hummingbirds.md": "# Hummingbirds\n\nHummingbirds hover in front of flowers and drink nectar. Their wings beat dozens of times per second.\n",
}


def fake_engine(tmp):
    """A tiny OpenAI-style engine (only GET /v1/models) published through a discovery file, so the engine dialog has something to find.
    Returns the environment for the server process."""
    import http.server, json, threading
    class H(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            ok = self.path.rstrip("/") == "/v1/models"
            body = json.dumps({"data": [{"id": "qwen3-8b"}]} if ok else {}).encode()
            self.send_response(200 if ok else 404); self.send_header("Content-Type", "application/json"); self.send_header("Content-Length", str(len(body))); self.end_headers(); self.wfile.write(body)
        def log_message(self, *a): pass
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), H); threading.Thread(target=srv.serve_forever, daemon=True).start()
    f = os.path.join(tmp, "endpoint.json"); open(f, "w").write(json.dumps({"url": f"http://127.0.0.1:{srv.server_address[1]}/v1", "model": "qwen3-8b"}))
    empty = os.path.join(tmp, "no-companion"); os.makedirs(empty, exist_ok=True)
    return dict(os.environ, COLIBRI_ENDPOINT_FILE=f, COLIBRI_COMPANION_HOME=empty)


def engine_dialog(pg, pause=500):
    pg.click("#gwCfg"); pg.wait_for_selector("#gwDlg", state="visible"); pg.click("#gwDetect"); pg.wait_for_selector("#gwFound [data-use]"); pg.wait_for_timeout(pause)


def import_dialog(pg, pause=500):
    pg.click(".rb[data-view=library]"); pg.wait_for_timeout(300); pg.click("#bringBtn"); pg.wait_for_selector("#wgUrl", state="visible"); pg.fill("#wgUrl", "https://example.org/article"); pg.wait_for_timeout(pause)


def free_port():
    s = socket.socket(); s.bind(("127.0.0.1", 0)); p = s.getsockname()[1]; s.close(); return p


def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--lib", default=None); ap.add_argument("--port", type=int, default=0)
    ap.add_argument("--bin", default=os.path.join(ROOT, "target", "release", "cartacolibri")); ap.add_argument("--only", default=",".join(LANGS)); a = ap.parse_args()
    langs = [l for l in a.only.split(",") if l in LANGS]
    out = os.path.join(ROOT, "docs", "cartacolibri", "img"); os.makedirs(out, exist_ok=True)
    home = tempfile.mkdtemp(prefix="cc-cap-home-"); lib = a.lib
    if not lib:
        lib = os.path.join(tempfile.mkdtemp(prefix="cc-cap-"), "Documents"); os.makedirs(lib)
        for rel, txt in SAMPLE.items():
            os.makedirs(os.path.dirname(os.path.join(lib, rel)), exist_ok=True); open(os.path.join(lib, rel), "w", encoding="utf-8").write(txt)
    port = a.port or free_port()
    srv = subprocess.Popen([a.bin, "--home", home, "serve", "--port", str(port)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=fake_engine(home))
    for _ in range(100):
        try: socket.create_connection(("127.0.0.1", port), 0.2).close(); break
        except OSError: time.sleep(0.1)
    try:
        base = f"http://127.0.0.1:{port}"
        with sync_playwright() as p:
            try: b = p.chromium.launch(channel="chrome", args=["--no-sandbox", "--disable-background-networking", "--disable-sync", "--no-first-run", "--disable-component-update"])
            except Exception: b = p.chromium.launch(executable_path="/usr/bin/google-chrome", args=["--no-sandbox", "--disable-background-networking", "--disable-sync", "--no-first-run", "--disable-component-update"])
            for lang in langs:
                ctx = b.new_context(viewport={"width": 1360, "height": 820})
                ctx.add_init_script(f"localStorage.setItem('carta.lang','{lang}');localStorage.setItem('carta.theme','dark');localStorage.setItem('carta.welcomed','1')")
                pg = ctx.new_page(); pg.set_default_timeout(20000); errs = []; pg.on("pageerror", lambda e: errs.append(str(e)))
                pg.goto(base + "/carta.html#library")
                try: pg.wait_for_selector("#newRoot", timeout=6000)
                except Exception: pg.reload(); pg.wait_for_selector("#newRoot")   # rare: one request of a fresh page never returns (see tests/e2e notes)
                if lang == langs[0] and not pg.locator("#libRows tr").count():
                    pg.fill("#newRoot", lib); pg.click("#addRoot")
                pg.wait_for_selector("#libRows tr", timeout=60000); pg.wait_for_timeout(600)
                pg.mouse.move(700, 790); pg.screenshot(path=f"{out}/{lang}-1.png")
                pg.click('.rb[data-view=fetch]'); pg.fill("#q", "payment OR pay* OR plată OR plată*"); pg.press("#q", "Enter"); pg.wait_for_selector("#resRows tr"); pg.mouse.move(700, 790); pg.wait_for_timeout(300)
                pg.screenshot(path=f"{out}/{lang}-2.png")
                pg.check("#resAll"); pg.fill("#gName", GROUP[lang]); pg.click("#gFromSel"); pg.wait_for_selector("#groupList .row"); pg.wait_for_timeout(300)
                pg.screenshot(path=f"{out}/{lang}-3.png")
                pg.click("#cartaBtn"); pg.click('#taskSeg button[data-t=synthesize]'); pg.check("#mock")
                pg.check('input[name=scope][value=group]'); pg.select_option("#scopeGroup", GROUP[lang]); pg.fill("#req", ASK[lang]); pg.click("#run")
                pg.wait_for_selector("#out .cite, #out li", timeout=20000); pg.wait_for_timeout(400)
                pg.screenshot(path=f"{out}/{lang}-4.png")
                # chat (still in the drawer)
                pg.click('#dTabs button[data-tab=chat]'); pg.select_option("#chatGroup", index=0); pg.fill("#chatIn", CHAT[lang]); pg.click("#chatSend")
                pg.wait_for_selector("#chatLog .msg.b .src, #chatLog .msg.b .nf", timeout=20000); pg.wait_for_timeout(400)
                pg.screenshot(path=f"{out}/{lang}-6.png")
                pg.click("#dClose"); pg.wait_for_selector("#drawer", state="hidden")
                # document drawer with a tag and a note
                pg.locator("#resRows tr").first.dblclick(position={"x": 70, "y": 12}); pg.wait_for_selector("#viewer.on")
                pg.wait_for_selector("#vRel li"); pg.fill("#vTagIn", TAG[lang]); pg.click("#vTagAdd"); pg.wait_for_selector("#vTags .tagc")
                pg.click("#vStar"); pg.fill("#vNote", NOTE[lang]); pg.wait_for_timeout(1600)
                pg.screenshot(path=f"{out}/{lang}-5.png")
                # undo the demo tag / star / note so the next language starts from the same library
                pg.click("#vStar"); pg.click("#vTags [data-tagrm]"); pg.wait_for_selector("#vTags .tagc", state="detached")
                pg.fill("#vNote", ""); pg.wait_for_timeout(1600)
                pg.click("#vClose"); pg.wait_for_selector("#viewer", state="hidden")
                # stats
                pg.click('.rb[data-view=stats]'); pg.wait_for_selector("#statsOut svg.bars rect.bb"); pg.mouse.move(700, 600); pg.wait_for_timeout(500)
                pg.screenshot(path=f"{out}/{lang}-7.png")
                # engine dialog (found automatically) and import dialog (web-grab)
                engine_dialog(pg); pg.screenshot(path=f"{out}/{lang}-8.png"); pg.click("#gwX"); pg.wait_for_selector("#gwDlg", state="hidden")
                import_dialog(pg); pg.screenshot(path=f"{out}/{lang}-9.png"); pg.click("#brClose"); pg.wait_for_selector("#bring", state="hidden")
                # clean up so the next language starts alike
                pg.evaluate("async n=>{const h={'X-Colibri-UI':'1','Content-Type':'application/json'};await fetch('/carta/groups',{method:'DELETE',headers:h,body:JSON.stringify({name:n})})}", GROUP[lang])
                print(lang, "ok", "JS errors:" + str(errs) if errs else "")
                ctx.close()
            b.close()
    finally:
        srv.terminate(); shutil.rmtree(home, ignore_errors=True)


if __name__ == "__main__":
    main()
