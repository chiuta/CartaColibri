"""RAG-CARTA — the AI layer of CartaColibri.

Runs three local tasks over a *group* of files prepared by RAG-fetch (ragfetch.py), through
the Colibrì gateway (OpenAI-compatible `/v1/chat/completions`, e.g. `coli serve` / `coli web`):

  summarize   concise, faithful summaries per document (+ one overview for a group)
  extract     structured data (Markdown table or JSON) with the source file for every row
  synthesize  one integrated answer across several files: themes, agreements, contradictions

Every result keeps citations [S1], [S2] ... that map to the source files, and can be saved
(timestamped, labelled) under <state>/outputs/ — which is never indexed. Source documents
are never modified. When no gateway answers, `backend="mock"` gives a deterministic
extractive fallback so the whole pipeline stays testable and usable offline.
"""
import json, os, re, time, urllib.error, urllib.request
from ragfetch import STATE_DIR_NAME

TASKS = ("summarize", "extract", "synthesize")
DEFAULT_URL = "http://127.0.0.1:8000"
CTX_CHARS = 18000          # characters of source text per model call (map-reduce above this)

SYSTEM = ("You are CartaColibri, a careful local document assistant. Use ONLY the provided source text; "
          "never add outside knowledge. Cite sources as [S1], [S2] ... exactly as labelled. "
          "If something is not in the sources, say so instead of guessing. Answer in the language of the user's request "
          "(default: the language of the documents).")


class GatewayError(RuntimeError):
    pass


class Backend:
    """name: 'colibri' (HTTP gateway) or 'mock' (offline, extractive)."""
    def __init__(self, name="colibri", url=DEFAULT_URL, model=None, timeout=1800, max_tokens=1200):
        self.name, self.url, self.model, self.timeout, self.max_tokens = name, url.rstrip("/"), model, timeout, max_tokens

    def complete(self, system, user):
        if self.name == "mock":
            return _mock(user)
        body = {"messages": [{"role": "system", "content": system}, {"role": "user", "content": user}],
                "max_tokens": self.max_tokens, "temperature": 0.2, "stream": False}
        if self.model: body["model"] = self.model
        req = urllib.request.Request(self.url + "/v1/chat/completions", json.dumps(body).encode(),
                                     {"Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as r:
                data = json.load(r)
            msg = data["choices"][0]["message"]
            return (msg.get("content") or "").strip()
        except (urllib.error.URLError, OSError, KeyError, ValueError) as e:
            raise GatewayError(f"no answer from the Colibrì gateway at {self.url} ({e}). "
                               f"Start it with `coli web --model DIR` (or `coli serve`), or use --mock.") from e

    def alive(self):
        if self.name == "mock": return True
        try:
            urllib.request.urlopen(self.url + "/health", timeout=3); return True
        except Exception:
            return False


def _mock(user):
    """Deterministic stand-in: the first informative sentences of the sources, in order."""
    m = re.search(r"<<SOURCES>>(.*)<<END>>", user, re.S)
    body = m.group(1) if m else user
    keep = []
    for line in body.splitlines():
        line = line.strip()
        if line.startswith("### [S"):
            keep.append(line)
        elif len(line) > 40 and sum(1 for k in keep if not k.startswith("###")) < 40:
            keep.append("- " + re.split(r"(?<=[.!?])\s", line.lstrip("-* "))[0][:220])
    return "\n".join(keep[:30]) or "(no text)"


# ---------------------------------------------------------------- source packing
def _label(docs):
    return {d["id"]: f"S{i+1}" for i, d in enumerate(docs)}


def _blocks(lib, docs, labels, budget):
    """Split every document into <=budget-char pieces labelled with its [S#]; pack pieces into batches."""
    pieces = []
    for d in docs:
        text = lib.text_of(d["id"]).strip()
        for i in range(0, max(len(text), 1), budget):
            part = text[i:i + budget]
            if part.strip():
                pieces.append((d, f"### [{labels[d['id']]}] {os.path.basename(d['path'])}\n{part}"))
    batches, cur, size = [], [], 0
    for d, p in pieces:
        if cur and size + len(p) > budget:
            batches.append(cur); cur, size = [], 0
        cur.append((d, p)); size += len(p)
    if cur: batches.append(cur)
    return batches


def _sources_block(docs, labels):
    return "\n".join(f"- [{labels[d['id']]}] {d['path']}" for d in docs)


def _wrap(batch):
    return "<<SOURCES>>\n" + "\n\n".join(p for _, p in batch) + "\n<<END>>"


# ---------------------------------------------------------------- tasks
def run(lib, task, docs, request="", backend=None, fields=None, fmt="table", budget=CTX_CHARS, log=None):
    """docs: list of document rows (Library.documents / resolve_group). Returns a Markdown/JSON string
    ending with a Sources section."""
    if task not in TASKS: raise ValueError(f"task must be one of {TASKS}")
    if not docs: raise ValueError("the group has no indexed documents (run `index` first, or check its members)")
    backend = backend or Backend()
    labels = _label(docs)
    say = log or (lambda m: None)

    def ask(instr, batch):
        say(f"model call ({len(batch)} piece(s))")
        return backend.complete(SYSTEM, f"{instr}\n\n{_wrap(batch)}")

    if task == "summarize":
        focus = f" Focus on: {request}." if request else ""
        parts = []
        for d in docs:
            batches = _blocks(lib, [d], labels, budget)
            notes = [ask(f"Summarize the source below in concise key points, faithful to the text, citing "
                         f"[{labels[d['id']]}].{focus}", b) for b in batches]
            if len(notes) > 1:
                notes = [backend.complete(SYSTEM, "Merge these partial summaries into one concise summary of the "
                             f"document; keep the citation [{labels[d['id']]}].\n\n<<SOURCES>>\n" + "\n\n".join(notes) + "\n<<END>>")]
            parts.append(f"## [{labels[d['id']]}] {os.path.basename(d['path'])}\n\n{notes[0] if notes else '(no text)'}")
        body = "\n\n".join(parts)
        if len(docs) > 1:
            body = "## Overview\n\n" + backend.complete(
                SYSTEM, "Write a short overview (5-8 bullet points) of what these per-document summaries cover, "
                        "keeping the [S#] citations.\n\n<<SOURCES>>\n" + "\n\n".join(parts) + "\n<<END>>") + "\n\n" + body
    elif task == "extract":
        spec = ", ".join(fields) if fields else request
        if not spec: raise ValueError("extract needs --fields (comma-separated) or a request")
        shape = ("a Markdown table with columns: Source, " + (", ".join(fields) if fields else "the requested fields")
                 if fmt == "table" else "a JSON array of objects, each with a \"source\" key like \"S1\" plus the requested fields")
        rows = []
        for d in docs:
            for b in _blocks(lib, [d], labels, budget):
                rows.append(ask(f"Extract these items from the source: {spec}. Output {shape}, one row/object per "
                                f"finding. Use \"NOT FOUND\" for anything missing — never guess. Cite [{labels[d['id']]}]. "
                                f"Output only the {'table' if fmt == 'table' else 'JSON'}.", b))
        body = _merge_extract(rows, fmt)
    else:   # synthesize
        if not request: raise ValueError("synthesize needs a question or theme")
        notes = []
        for b in _blocks(lib, docs, labels, budget):
            notes.append(ask(f"Question/theme: {request}\nList the relevant facts, claims and figures from these "
                             f"sources as short bullet points, each with its [S#] citation. Skip irrelevant text.", b))
        body = backend.complete(SYSTEM,
            f"Question/theme: {request}\n\nUsing ONLY the notes below, write an integrated synthesis: group ideas by "
            "theme, state the points the sources share, and flag differences or contradictions between them. "
            "Cite every claim with [S#].\n\n<<SOURCES>>\n" + "\n\n".join(notes) + "\n<<END>>")
    return f"{body.strip()}\n\n---\n**Sources**\n{_sources_block(docs, labels)}\n"


def _merge_extract(rows, fmt):
    if fmt == "json":
        out = []
        for r in rows:
            m = re.search(r"\[.*\]", r, re.S)
            try: out.extend(json.loads(m.group(0)) if m else [])
            except ValueError: out.append({"raw": r})
        return json.dumps(out, ensure_ascii=False, indent=2)
    header, lines = None, []
    for r in rows:
        for ln in r.splitlines():
            if not ln.strip().startswith("|"): continue
            if re.fullmatch(r"\|[\s:|-]+\|", ln.strip()): continue
            if header is None: header = ln.strip(); continue
            if ln.strip() != header: lines.append(ln.strip())
    if header is None: return "\n\n".join(rows)
    cols = header.count("|") - 1
    return "\n".join([header, "|" + "---|" * cols] + lines)


def save(home, text, task, group, request=""):
    """Write a result under <home>/outputs/, labelled (timestamp, group, task) — never re-indexed."""
    d = os.path.join(home, "outputs"); os.makedirs(d, exist_ok=True)
    ts = time.strftime("%Y%m%d-%H%M%S")
    safe = re.sub(r"[^\w.-]+", "_", group or "selection")
    ext = ".json" if text.lstrip().startswith(("[", "{")) else ".md"
    p = os.path.join(d, f"{ts}_{safe}_{task}{ext}")
    head = (f"<!-- cartacolibri | {time.strftime('%Y-%m-%d %H:%M:%S')} | group: {group or 'selection'} | "
            f"task: {task} | request: {request} -->\n") if ext == ".md" else ""
    with open(p, "w", encoding="utf-8") as f: f.write(head + text)
    return p
