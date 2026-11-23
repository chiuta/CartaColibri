"""RAG-fetch — the middle layer of CartaColibri.

A local, incremental, full-text index of the user's document folders (in the spirit of
DocFetcher: index once, update only what changed, search and filter fast), plus named,
re-usable *groups* of files that RAG-CARTA (ragcarta.py) can be pointed at.

Everything is local: one SQLite file (FTS5) and one JSON manifest in the state directory.
The documents themselves are only ever read, never modified.

Pure standard library; PDF text comes from pypdf / PyMuPDF / `pdftotext` if one is present.
"""
import fnmatch, hashlib, html.parser, json, os, re, shutil, sqlite3, subprocess, time, zipfile
import xml.etree.ElementTree as ET

STATE_DIR_NAME = "_cartacolibri"          # also skipped while scanning, so outputs never re-enter the index
P1_EXT = {".txt", ".md", ".markdown", ".rst", ".log", ".pdf"}
P2_EXT = {".csv", ".json", ".jsonl", ".yaml", ".yml", ".html", ".htm", ".rtf",
          ".docx", ".odt", ".pptx"}
DEFAULT_EXT = P1_EXT | P2_EXT
IMG_EXT = {".png", ".jpg", ".jpeg", ".tif", ".tiff", ".bmp", ".webp"}   # indexed only with OCR (tesseract) available
MAX_ZIP_PART = 100 * 1024 * 1024        # a docx/odt/pptx part larger than this once unpacked is refused (zip bomb)
SKIP_DIRS = {STATE_DIR_NAME, ".cartacolibri", ".git", "__pycache__", "node_modules", ".venv", "venv", ".cache"}
HEAVY_EXT = {".zip", ".7z", ".rar", ".tar", ".gz", ".iso", ".exe", ".dll", ".so", ".bin", ".gguf",
             ".safetensors", ".pt", ".pth", ".onnx", ".mp3", ".mp4", ".mkv", ".avi", ".mov", ".wav", ".flac"}
MAX_FILE_BYTES = 80 * 1024 * 1024
CHUNK_CHARS = 1400


def state_dir(path=None):
    p = path or os.environ.get("CARTACOLIBRI_HOME") or os.path.join(os.path.expanduser("~"), ".cartacolibri")
    os.makedirs(p, exist_ok=True)
    return p


# ---------------------------------------------------------------- text extraction
def _decode(raw):
    for enc in ("utf-8-sig", "utf-16") if raw[:2] in (b"\xff\xfe", b"\xfe\xff") else ("utf-8-sig",):
        try:
            return raw.decode(enc)
        except UnicodeError:
            pass
    try:
        return raw.decode("cp1250")            # Romanian / Central-European legacy files
    except UnicodeError:
        return raw.decode("latin-1", "replace")


class _Html(html.parser.HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.out, self.skip = [], 0

    def handle_starttag(self, tag, attrs):
        if tag in ("script", "style"): self.skip += 1
        elif tag in ("p", "br", "div", "li", "tr", "h1", "h2", "h3", "h4", "h5", "h6"): self.out.append("\n")

    def handle_endtag(self, tag):
        if tag in ("script", "style") and self.skip: self.skip -= 1

    def handle_data(self, d):
        if not self.skip: self.out.append(d)


def _xml_text(data, para_tags):
    out = []
    for ev, el in ET.iterparse(__import__("io").BytesIO(data), events=("end",)):
        t = el.tag.rsplit("}", 1)[-1]
        if t in ("t", "text-run", "span") and el.text: out.append(el.text)
        elif t in para_tags: out.append("\n")
    return "".join(out)


def ocr_available():
    return shutil.which("tesseract") is not None


def ocr_image(path, lang="ron+eng", fmt="txt"):
    """tesseract on one image. fmt 'txt' -> text; 'pdf' -> bytes of a searchable PDF."""
    if not ocr_available(): raise ValueError("OCR needs tesseract (apt install tesseract-ocr tesseract-ocr-ron)")
    langs = subprocess.run(["tesseract", "--list-langs"], capture_output=True, text=True).stdout.split()
    lang = "+".join(l for l in lang.split("+") if l in langs) or "eng"
    out = subprocess.run(["tesseract", path, "stdout", "-l", lang] + (["pdf"] if fmt == "pdf" else []),
                         capture_output=True, timeout=600)
    if out.returncode: raise ValueError("tesseract failed: " + out.stderr.decode("utf-8", "replace")[:200])
    return out.stdout if fmt == "pdf" else _decode(out.stdout)


def _zip_read(z, name):
    if z.getinfo(name).file_size > MAX_ZIP_PART: raise ValueError(f"{name} unpacks to more than {MAX_ZIP_PART >> 20} MB")
    return z.read(name)


def _pdf_pages(path):
    try:
        import pypdf
        r = pypdf.PdfReader(path)
        return [(f"p.{i+1}", (p.extract_text() or "")) for i, p in enumerate(r.pages)]
    except ImportError:
        pass
    try:
        import fitz
        with fitz.open(path) as d:
            return [(f"p.{i+1}", p.get_text()) for i, p in enumerate(d)]
    except ImportError:
        pass
    if shutil.which("pdftotext"):
        out = subprocess.run(["pdftotext", "-layout", path, "-"], capture_output=True, timeout=300)
        if out.returncode == 0:
            return [(f"p.{i+1}", t) for i, t in enumerate(_decode(out.stdout).split("\f")) if t.strip()]
        raise ValueError("pdftotext failed: " + out.stderr.decode("utf-8", "replace")[:200])
    raise ValueError("no PDF reader (pip install pypdf, or install poppler-utils)")


def extract(path, ext):
    """-> list of (location, text). Raises on a file that cannot be read."""
    if ext == ".pdf":
        return _pdf_pages(path)
    if ext in IMG_EXT:
        return [("OCR", ocr_image(path))]
    if ext in (".docx", ".odt", ".pptx"):
        with zipfile.ZipFile(path) as z:
            if ext == ".docx":
                return [("", _xml_text(_zip_read(z, "word/document.xml"), {"p", "br"}))]
            if ext == ".odt":
                return [("", _xml_text(_zip_read(z, "content.xml"), {"p", "h", "line-break"}))]
            slides = sorted((n for n in z.namelist() if re.match(r"ppt/slides/slide\d+\.xml$", n)),
                            key=lambda n: int(re.findall(r"\d+", n)[0]))
            return [(f"slide {i+1}", _xml_text(_zip_read(z, n), {"p"})) for i, n in enumerate(slides)]
    with open(path, "rb") as f:
        text = _decode(f.read())
    if ext in (".html", ".htm"):
        p = _Html(); p.feed(text); text = "".join(p.out)
    elif ext == ".rtf":
        text = re.sub(r"\\'([0-9a-f]{2})", lambda m: bytes.fromhex(m.group(1)).decode("cp1250", "replace"), text)
        text = re.sub(r"\\[a-z]+-?\d* ?|[{}]", "", text)
    return [("", text)]


def chunk(text, size=CHUNK_CHARS):
    text = re.sub(r"[ \t]+", " ", text.replace("\r", ""))
    text = re.sub(r"\n{3,}", "\n\n", text).strip()
    out, cur = [], ""
    for para in re.split(r"\n\s*\n|\n", text):
        para = para.strip()
        if not para: continue
        while len(para) > size:                # a very long line: split on a space
            cut = para.rfind(" ", 0, size) or size
            out.append(para[:cut]); para = para[cut:].lstrip()
        if len(cur) + len(para) + 1 > size and cur:
            out.append(cur); cur = para
        else:
            cur = f"{cur}\n{para}" if cur else para
    if cur: out.append(cur)
    return out


# ---------------------------------------------------------------- the library
class Library:
    def __init__(self, home=None, extensions=None, ocr=False):
        self.home = state_dir(home)
        self.exts = {e if e.startswith(".") else "." + e for e in (extensions or DEFAULT_EXT)}
        if ocr and ocr_available(): self.exts |= IMG_EXT
        self.db = sqlite3.connect(os.path.join(self.home, "index.sqlite"))
        self.db.row_factory = sqlite3.Row
        self.db.executescript("""
        CREATE TABLE IF NOT EXISTS roots(path TEXT PRIMARY KEY);
        CREATE TABLE IF NOT EXISTS docs(id INTEGER PRIMARY KEY, root TEXT, path TEXT UNIQUE, ext TEXT,
            size INTEGER, mtime_ns INTEGER, sha1 TEXT, chars INTEGER, pages INTEGER, indexed_at REAL);
        CREATE VIRTUAL TABLE IF NOT EXISTS chunks USING fts5(text, doc_id UNINDEXED, loc UNINDEXED, seq UNINDEXED,
            tokenize='unicode61 remove_diacritics 2');
        CREATE TABLE IF NOT EXISTS problems(path TEXT PRIMARY KEY, reason TEXT, at REAL);
        """)
        self._groups = os.path.join(self.home, "groups.json")

    # ---- folders (the "Library" page: list + add folder + remove folder)
    def add_root(self, path):
        path = os.path.abspath(os.path.expanduser(path))
        if not os.path.isdir(path): raise ValueError(f"not a folder: {path}")
        self.db.execute("INSERT OR IGNORE INTO roots VALUES(?)", (path,)); self.db.commit()
        return path

    def remove_root(self, path):
        path = os.path.abspath(os.path.expanduser(path))
        ids = [r[0] for r in self.db.execute("SELECT id FROM docs WHERE root=?", (path,))]
        for i in ids: self._drop(i)
        self.db.execute("DELETE FROM roots WHERE path=?", (path,)); self.db.commit()
        return len(ids)

    def roots(self):
        return [r[0] for r in self.db.execute("SELECT path FROM roots ORDER BY path")]

    # ---- uploads ("document upload to library", optionally saved as OCR text / searchable PDF)
    def upload_dir(self):
        d = os.path.join(self.home, "library", "uploads"); os.makedirs(d, exist_ok=True)
        return d

    def add_upload(self, name, data, ocr=None):
        """Save an uploaded file in the library's uploads folder and index it. ocr: None | ['txt'] | ['txt','pdf']
        — for an image (or a scanned PDF page image) also write the recognised text / searchable PDF next to it."""
        name = re.sub(r"[^\w. -]", "_", os.path.basename(name.replace("\\", "/"))).strip(" .") or "document"
        d = self.upload_dir(); path = os.path.join(d, name)
        stem, ext = os.path.splitext(name); n = 1
        while os.path.exists(path):
            path = os.path.join(d, f"{stem}-{n}{ext}"); n += 1
        with open(path, "wb") as f: f.write(data)
        made = []
        if ocr and os.path.splitext(path)[1].lower() in IMG_EXT:
            base = os.path.splitext(path)[0]
            if "txt" in ocr:
                with open(base + ".ocr.txt", "w", encoding="utf-8") as f: f.write(ocr_image(path))
                made.append(base + ".ocr.txt")
            if "pdf" in ocr:
                with open(base + ".ocr.pdf", "wb") as f: f.write(ocr_image(path, fmt="pdf"))
                made.append(base + ".ocr.pdf")
        stats = self.update([d])
        return {"saved": path, "ocr_files": made, "index": stats}

    # ---- indexing
    def _drop(self, doc_id):
        self.db.execute("DELETE FROM chunks WHERE doc_id=?", (doc_id,))
        self.db.execute("DELETE FROM docs WHERE id=?", (doc_id,))

    def _walk(self, root):
        for dp, dn, fn in os.walk(root):
            dn[:] = sorted(d for d in dn if d not in SKIP_DIRS and not d.startswith("."))
            for f in sorted(fn):
                yield os.path.join(dp, f)

    def update(self, roots=None, log=None):
        """Incremental: only new/changed files are read; vanished ones are dropped."""
        stats = dict(added=0, updated=0, removed=0, unchanged=0, skipped=0, failed=0)
        for root in (roots or self.roots()):
            root = os.path.abspath(root)
            self.add_root(root)
            seen = set()
            for p in self._walk(root):
                ext = os.path.splitext(p)[1].lower()
                name = os.path.basename(p)
                if name.startswith((".", "~$")) or name.endswith(("~", ".tmp")) or ext in HEAVY_EXT or ext not in self.exts:
                    stats["skipped"] += 1; continue
                try:
                    st = os.stat(p)
                except OSError:
                    continue
                if st.st_size > MAX_FILE_BYTES:
                    stats["skipped"] += 1; continue
                seen.add(p)
                row = self.db.execute("SELECT id,size,mtime_ns,sha1 FROM docs WHERE path=?", (p,)).fetchone()
                if row and row["size"] == st.st_size and row["mtime_ns"] == st.st_mtime_ns:
                    stats["unchanged"] += 1; continue
                try:
                    with open(p, "rb") as fh: sha = hashlib.sha1(fh.read()).hexdigest()
                    if row and row["sha1"] == sha:                     # touched, not changed
                        self.db.execute("UPDATE docs SET mtime_ns=? WHERE id=?", (st.st_mtime_ns, row["id"]))
                        stats["unchanged"] += 1; continue
                    parts = extract(p, ext)
                    if row: self._drop(row["id"]); stats["updated"] += 1
                    else: stats["added"] += 1
                    total, seq = 0, 0
                    cur = self.db.execute("INSERT INTO docs(root,path,ext,size,mtime_ns,sha1,chars,pages,indexed_at)"
                                          " VALUES(?,?,?,?,?,?,0,?,?)",
                                          (root, p, ext, st.st_size, st.st_mtime_ns, sha, len(parts), time.time()))
                    did = cur.lastrowid
                    for loc, text in parts:
                        for c in chunk(text):
                            self.db.execute("INSERT INTO chunks(text,doc_id,loc,seq) VALUES(?,?,?,?)", (c, did, loc, seq))
                            total += len(c); seq += 1
                    self.db.execute("UPDATE docs SET chars=? WHERE id=?", (total, did))
                    self.db.execute("DELETE FROM problems WHERE path=?", (p,))
                except Exception as e:                  # a corrupt file never stops the folder
                    stats["failed"] += 1
                    msg = f"{type(e).__name__}: {e}"[:300]
                    self.db.execute("INSERT OR REPLACE INTO problems VALUES(?,?,?)", (p, msg, time.time()))
                    if log: log(f"skip {p}: {msg}")
            for r in self.db.execute("SELECT id,path FROM docs WHERE root=?", (root,)).fetchall():
                if r["path"] not in seen and not os.path.exists(r["path"]):
                    self._drop(r["id"]); stats["removed"] += 1
            self.db.commit()
        return stats

    def problems(self):
        return [dict(r) for r in self.db.execute("SELECT path,reason FROM problems ORDER BY path")]

    # ---- listing / search
    def documents(self, group=None):
        rows = [dict(r) for r in self.db.execute("SELECT id,root,path,ext,size,chars,pages FROM docs ORDER BY path")]
        if group is not None:
            keep = {d["id"] for d in self.resolve_group(group)}
            rows = [r for r in rows if r["id"] in keep]
        return rows

    @staticmethod
    def _fts_query(q):
        """Words, "phrases", prefix* and AND/OR/NOT pass through; anything FTS5 would reject is quoted."""
        q = q.strip()
        if not q: raise ValueError("empty query")
        toks = re.findall(r'"[^"]+"|\S+', q)
        out = []
        for t in toks:
            if t in ("AND", "OR", "NOT"):
                out.append(t)
            elif len(t) > 2 and t.startswith('"') and t.endswith('"') and '"' not in t[1:-1]:
                phrase = " ".join(re.findall(r"\w+", t[1:-1], re.UNICODE))
                if phrase: out.append('"' + phrase + '"')
            else:
                core = t.rstrip("*")
                word = re.sub(r"[^\w]", "", core, flags=re.UNICODE)
                if word: out.append(word + ("*" if t.endswith("*") else ""))
        return " ".join(out) or '""'

    def search(self, query, ext=None, min_size=None, max_size=None, path_like=None, group=None, limit=50):
        allowed = {d["id"] for d in self.resolve_group(group)} if group else None
        exts = {e if e.startswith(".") else "." + e for e in (ext or [])}
        sql = ("SELECT doc_id, loc, snippet(chunks,0,'[',']',' … ',24) AS snip, bm25(chunks) AS score "
               "FROM chunks WHERE chunks MATCH ? ORDER BY score LIMIT 2000")
        try:
            rows = self.db.execute(sql, (self._fts_query(query),)).fetchall()
        except sqlite3.OperationalError:       # e.g. a dangling AND/NOT: fall back to plain words, never a crash
            words = [w for w in re.findall(r"\w+", query, re.UNICODE) if w not in ("AND", "OR", "NOT", "NEAR")]
            if not words: return []
            rows = self.db.execute(sql, (" ".join('"%s"' % w for w in words),)).fetchall()   # quoted: AND/OR/NEAR are words now
        best, hits = {}, {}
        for r in rows:
            hits[r["doc_id"]] = hits.get(r["doc_id"], 0) + 1
            if r["doc_id"] not in best: best[r["doc_id"]] = r
        out = []
        for did, r in best.items():
            d = self.db.execute("SELECT * FROM docs WHERE id=?", (did,)).fetchone()
            if allowed is not None and did not in allowed: continue
            if exts and d["ext"] not in exts: continue
            if min_size is not None and d["size"] < min_size: continue
            if max_size is not None and d["size"] > max_size: continue
            if path_like and path_like.lower() not in d["path"].lower(): continue
            out.append(dict(id=did, path=d["path"], ext=d["ext"], size=d["size"], loc=r["loc"],
                            snippet=r["snip"].replace("\n", " "), score=round(-r["score"], 3), matches=hits[did]))
        out.sort(key=lambda x: -x["score"])
        return out[:limit]

    def text_of(self, doc_id):
        rows = self.db.execute("SELECT loc,text FROM chunks WHERE doc_id=? ORDER BY CAST(seq AS INTEGER)", (doc_id,))
        out, last = [], None
        for r in rows:
            if r["loc"] and r["loc"] != last:
                out.append(f"\n[{r['loc']}]"); last = r["loc"]
            out.append(r["text"])
        return "\n".join(out)

    # ---- groups (configurable lists)
    def _load_groups(self):
        try:
            with open(self._groups, encoding="utf-8") as f: return json.load(f)
        except FileNotFoundError:
            return {}

    def _save_groups(self, g):
        tmp = self._groups + ".tmp"
        with open(tmp, "w", encoding="utf-8") as f: json.dump(g, f, indent=2, ensure_ascii=False)
        os.replace(tmp, self._groups)

    def groups(self):
        return self._load_groups()

    def group_set(self, name, members=None, description=None, add=False):
        """members: file paths, folders, or glob patterns (matched against the full path)."""
        g = self._load_groups()
        cur = g.get(name, {"description": "", "members": []})
        if description is not None: cur["description"] = description
        if members:
            norm = [m if any(c in m for c in "*?[") else os.path.abspath(os.path.expanduser(m)) for m in members]
            cur["members"] = sorted(set(cur["members"] + norm)) if add else norm
        g[name] = cur
        self._save_groups(g)
        return cur

    def group_remove(self, name, members=None):
        g = self._load_groups()
        if name not in g: raise KeyError(name)
        if members:
            g[name]["members"] = [m for m in g[name]["members"] if m not in members]
        else:
            del g[name]
        self._save_groups(g)

    def resolve_group(self, name):
        g = self._load_groups()
        if name not in g: raise KeyError(f"no group '{name}'")
        docs = self.documents()
        out = []
        for d in docs:
            for m in g[name]["members"]:
                if any(c in m for c in "*?["):
                    ok = fnmatch.fnmatch(d["path"], m)
                else:
                    ok = d["path"] == m or d["path"].startswith(m.rstrip(os.sep) + os.sep)
                if ok: out.append(d); break
        return out
