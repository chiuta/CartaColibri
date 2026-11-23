"""CartaColibri: RAG-fetch (index, search, groups) and RAG-CARTA (summarize/extract/synthesize).
Run:  python3 -m unittest tests.test_cartacolibri   (or `make check-carta`)"""
import json, os, sys, tempfile, threading, time, unittest, zipfile
from http.server import BaseHTTPRequestHandler, HTTPServer

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "c"))
import ragfetch, ragcarta, carta_cli


def w(path, text, mode="w"):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, mode, **({} if "b" in mode else {"encoding": "utf-8"})) as f: f.write(text)


class FakeGateway(BaseHTTPRequestHandler):
    seen = []
    def do_GET(self):
        self.send_response(200); self.end_headers(); self.wfile.write(b"ok")
    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        FakeGateway.seen.append(body)
        out = json.dumps({"choices": [{"message": {"content": "- fact [S1]\n| Source | Suma |\n|---|---|\n| S1 | 42 |"}}]}).encode()
        self.send_response(200); self.send_header("Content-Type", "application/json"); self.end_headers(); self.wfile.write(out)
    def log_message(self, *a): pass


class Carta(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        t = self.tmp.name
        self.lib_dir = os.path.join(t, "docs")
        w(os.path.join(self.lib_dir, "contracte", "c1.txt"), "Contract de prestări servicii. Valoarea totală este 42 de lei. Termen: 30 de zile.")
        w(os.path.join(self.lib_dir, "contracte", "c2.md"), "# Contract 2\nPlata se face în 45 de zile. Valoarea: 100 lei.")
        w(os.path.join(self.lib_dir, "articole", "a1.html"), "<html><script>var x=1</script><body><p>Articol despre colibri și zbor.</p></body></html>")
        with open(os.path.join(self.lib_dir, "ro1250.txt"), "wb") as f: f.write("Şcoală şi ţară în Bucureşti".encode("cp1250"))
        with zipfile.ZipFile(os.path.join(self.lib_dir, "note.docx"), "w") as z:
            z.writestr("word/document.xml", '<w:document xmlns:w="x"><w:body><w:p><w:r><w:t>Notă de proiect Colibri</w:t></w:r></w:p></w:body></w:document>')
        w(os.path.join(self.lib_dir, "broken.pdf"), b"not a pdf at all", "wb")
        w(os.path.join(self.lib_dir, "_cartacolibri", "outputs", "old.md"), "colibri output that must not be indexed")
        w(os.path.join(self.lib_dir, ".git", "x.txt"), "colibri hidden")
        w(os.path.join(self.lib_dir, "movie.mp4"), b"\0\0", "wb")
        self.lib = ragfetch.Library(os.path.join(t, "state"))
        self.stats = self.lib.update([self.lib_dir])

    def tearDown(self):
        self.lib.db.close(); self.tmp.cleanup()

    def test_index_exclusions_and_errors(self):
        names = {os.path.basename(d["path"]) for d in self.lib.documents()}
        self.assertIn("c1.txt", names); self.assertIn("note.docx", names)
        self.assertNotIn("old.md", names); self.assertNotIn("x.txt", names); self.assertNotIn("movie.mp4", names)
        self.assertEqual(self.stats["failed"], 1)                       # broken.pdf logged, rest indexed
        self.assertTrue(any("broken.pdf" in p["path"] for p in self.lib.problems()))

    def test_incremental(self):
        self.assertEqual(self.lib.update()["added"], 0)
        self.assertEqual(self.lib.update()["unchanged"], len(self.lib.documents()))
        time.sleep(0.01)
        w(os.path.join(self.lib_dir, "contracte", "c3.txt"), "Al treilea contract, valoare 7 lei.")
        os.remove(os.path.join(self.lib_dir, "contracte", "c2.md"))
        w(os.path.join(self.lib_dir, "contracte", "c1.txt"), "Contract modificat complet. Valoare 99 lei.")
        s = self.lib.update()
        self.assertEqual((s["added"], s["removed"], s["updated"]), (1, 1, 1))
        self.assertTrue(self.lib.search("modificat")); self.assertFalse(self.lib.search("prestări"))

    def test_search_filters_and_diacritics(self):
        self.assertTrue(self.lib.search("scoala"))                      # ș/ş folded, also cp1250 file decoded
        self.assertTrue(self.lib.search("tara bucuresti"))
        self.assertTrue(self.lib.search("contr*"))
        self.assertTrue(self.lib.search('"45 de zile"'))
        self.assertEqual({h["ext"] for h in self.lib.search("contract", ext=["md"])}, {".md"})
        self.assertFalse(self.lib.search("contract", path_like="articole"))
        self.assertTrue(self.lib.search("valoarea OR plata NOT zbor"))
        self.assertIsInstance(self.lib.search("c++ (weird) \"unterminated"), list)   # never raises on odd input

    def test_search_fuzz_never_crashes(self):
        import random, string
        random.seed(3); alpha = string.printable + 'ăâîșț"*()-:^~\\\x00 AND OR NOT '
        for _ in range(3000):
            q = "".join(random.choice(alpha) for _ in range(random.randint(0, 30)))
            try: self.lib.search(q)
            except ValueError: pass                                  # empty query is a clean ValueError
        self.assertEqual(len(self.lib.search('"valoarea totală"')), 1)
        self.assertTrue(self.lib.search("valoarea AND"))             # dangling operator falls back to words

    def test_groups(self):
        c = os.path.join(self.lib_dir, "contracte")
        self.lib.group_set("Contracte", [c], "toate contractele")
        self.lib.group_set("Html", [os.path.join(self.lib_dir, "*.html"), os.path.join(self.lib_dir, "articole", "*.html")])
        self.assertEqual(len(self.lib.resolve_group("Contracte")), 2)
        self.assertEqual(len(self.lib.resolve_group("Html")), 1)
        self.assertEqual(len(self.lib.search("contract", group="Contracte")), 2)
        self.assertEqual(len(self.lib.documents("Contracte")), 2)
        self.lib.group_set("Contracte", [os.path.join(self.lib_dir, "note.docx")], add=True)
        self.assertEqual(len(self.lib.resolve_group("Contracte")), 3)
        # persists across instances
        self.assertIn("Contracte", ragfetch.Library(self.lib.home).groups())
        self.lib.group_remove("Html"); self.assertNotIn("Html", self.lib.groups())

    def test_tasks_mock_and_save(self):
        self.lib.group_set("G", [os.path.join(self.lib_dir, "contracte")])
        docs = self.lib.resolve_group("G")
        be = ragcarta.Backend("mock")
        for task, kw in (("summarize", {}), ("extract", {"fields": ["Suma"]}), ("synthesize", {"request": "termene de plată"})):
            out = ragcarta.run(self.lib, task, docs, backend=be, **kw)
            self.assertIn("**Sources**", out); self.assertIn("c1.txt", out); self.assertIn("[S1]", out)
        p = ragcarta.save(self.lib.home, out, "synthesize", "G", "x")
        self.assertTrue(os.path.exists(p)); self.assertIn("outputs", p)
        self.lib.update()                                               # outputs under state dir, never indexed
        self.assertFalse(any("outputs" in d["path"] for d in self.lib.documents()))

    def test_tasks_http_gateway_and_map_reduce(self):
        srv = HTTPServer(("127.0.0.1", 0), FakeGateway)
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        try:
            be = ragcarta.Backend("colibri", f"http://127.0.0.1:{srv.server_port}")
            self.assertTrue(be.alive())
            docs = self.lib.documents()
            FakeGateway.seen.clear()
            out = ragcarta.run(self.lib, "extract", [d for d in docs if d["path"].endswith("c1.txt")],
                               backend=be, fields=["Suma"], budget=40)   # tiny budget forces several pieces
            self.assertGreater(len(FakeGateway.seen), 1)
            self.assertEqual(out.count("| S1 | 42 |"), len(FakeGateway.seen))
            self.assertIn("NOT FOUND", FakeGateway.seen[0]["messages"][1]["content"])   # no-guessing instruction present
        finally:
            srv.shutdown()

    def test_gateway_down_is_a_clear_error(self):
        be = ragcarta.Backend("colibri", "http://127.0.0.1:9", timeout=2)
        with self.assertRaises(ragcarta.GatewayError):
            ragcarta.run(self.lib, "summarize", self.lib.documents()[:1], backend=be)

    def test_cli_end_to_end(self):
        home = os.path.join(self.tmp.name, "cli-state")
        import io, contextlib
        def cli(*a):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf): carta_cli.main(["--home", home, *a])
            return buf.getvalue()
        self.assertIn("added", cli("add", self.lib_dir))
        self.assertIn("c1.txt", cli("search", "prestări"))
        cli("group", "from-search", "Q", "contract")
        self.assertIn("(2 docs)", cli("group", "list"))
        r = cli("synthesize", "plata", "--group", "Q", "--mock", "--save")
        self.assertIn("saved:", r)
        self.assertIn("documents dropped", cli("remove", self.lib_dir))

    def test_upload_sanitises_names_and_indexes(self):
        r = self.lib.add_upload("../../etc/pass wd?.txt", "încărcat prin browser".encode())
        self.assertTrue(r["saved"].startswith(self.lib.upload_dir()))            # no path traversal
        self.assertEqual(r["index"]["added"], 1)
        r2 = self.lib.add_upload("../../etc/pass wd?.txt", b"second")
        self.assertNotEqual(r["saved"], r2["saved"])                              # never overwrites
        self.assertTrue(self.lib.search("incarcat"))

    def test_zip_bomb_part_refused(self):
        p = os.path.join(self.lib_dir, "bomb.docx")
        with zipfile.ZipFile(p, "w", zipfile.ZIP_DEFLATED) as z:
            z.writestr("word/document.xml", b"<a>" + b"0" * (ragfetch.MAX_ZIP_PART + 1) + b"</a>")
        self.lib.update()
        self.assertTrue(any("bomb.docx" in x["path"] for x in self.lib.problems()))

    def test_api_routes_and_errors(self):
        sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "ui"))
        import carta_api
        os.environ["CARTACOLIBRI_HOME"] = self.lib.home
        c, j = carta_api.handle("GET", "/carta/library", "", b"", "http://127.0.0.1:1"); self.assertEqual(c, 200)
        self.assertEqual(carta_api.handle("GET", "/carta/nope", "", b"", "")[0], 404)
        self.assertEqual(carta_api.handle("GET", "/carta/search", "q=", b"", "")[0], 400)   # empty query -> 400, not a crash
        c, j = carta_api.handle("POST", "/carta/run", "", json.dumps({"task": "summarize", "group": "x"}).encode(), "http://127.0.0.1:1")
        self.assertEqual(c, 400)
        ids = [d["id"] for d in self.lib.documents()][:2]
        c, j = carta_api.handle("POST", "/carta/run", "", json.dumps({"task": "summarize", "ids": ids, "mock": True}).encode(), "")
        self.assertEqual(c, 200); self.assertIn("Sources", j["text"])
        c, j = carta_api.handle("POST", "/carta/run", "", json.dumps({"task": "summarize", "ids": ids}).encode(), "http://127.0.0.1:9")
        self.assertEqual(c, 502)

    def test_epub_xlsx_and_stem_fallback(self):
        with zipfile.ZipFile(os.path.join(self.lib_dir, "carte.epub"), "w") as z:
            z.writestr("OEBPS/c1.xhtml", "<html><body><p>Capitolul despre pelicani migratori</p></body></html>")
        with zipfile.ZipFile(os.path.join(self.lib_dir, "tabel.xlsx"), "w") as z:
            z.writestr("xl/sharedStrings.xml", '<sst xmlns="x"><si><t>Furnizor Alfa</t></si></sst>')
            z.writestr("xl/worksheets/sheet1.xml", '<worksheet xmlns="x"><sheetData><row><c t="s"><v>0</v></c><c><v>1250</v></c></row></sheetData></worksheet>')
        self.lib.update()
        self.assertTrue(self.lib.search("pelicani")); self.assertTrue(self.lib.search("furnizor 1250"))
        self.assertTrue(self.lib.search("migratoare"))            # no exact word: the stem-prefix fallback finds "migratori"

    def test_watch_picks_up_changes(self):
        import threading
        stop = threading.Event(); seen = []
        t = threading.Thread(target=self.lib.watch, args=(0.05, stop, seen.append)); t.start()
        time.sleep(0.15); w(os.path.join(self.lib_dir, "nou.txt"), "document aparut in timpul supravegherii")
        time.sleep(0.6); stop.set(); t.join()
        self.assertTrue(any(st["added"] for st in seen))

    @unittest.skipUnless(ragfetch.ocr_available() and __import__("shutil").which("pdftoppm"), "tesseract/pdftoppm not installed")
    def test_scanned_pdf_ocr(self):
        from PIL import Image, ImageDraw, ImageFont
        img = Image.new("RGB", (1200, 300), "white"); d = ImageDraw.Draw(img)
        try: f = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 90)
        except OSError: f = ImageFont.load_default()
        d.text((40, 90), "HUMMINGBIRD LIBRARY", fill="black", font=f)
        p = os.path.join(self.lib_dir, "scan.pdf"); img.save(p, "PDF", resolution=150)
        self.assertFalse(ragfetch.Library(os.path.join(self.tmp.name, "noocr")).update([self.lib_dir]) and False)
        lib = ragfetch.Library(os.path.join(self.tmp.name, "ocr"), ocr=True); lib.update([self.lib_dir])
        self.assertTrue(lib.search("hummingbird")); lib.db.close()


if __name__ == "__main__":
    unittest.main()
