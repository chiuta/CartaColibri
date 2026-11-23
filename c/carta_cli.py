"""`coli carta ...` — CartaColibri's command line (RAG-fetch + RAG-CARTA).

  coli carta add DIR            register a library folder        coli carta folders
  coli carta remove DIR         forget it (files untouched)      coli carta index [DIR]
  coli carta list [--group G]   documents + location             coli carta problems
  coli carta search "query" [--ext pdf,txt] [--min-kb N] [--max-kb N] [--path TEXT] [--group G]
  coli carta group set NAME PATH_OR_GLOB... [-d TEXT] [--add]    coli carta group list|show|rm NAME
  coli carta group from-search NAME "query"                      snapshot search results as a group
  coli carta summarize|extract|synthesize --group G [REQUEST] [--fields a,b,c] [--json] [--save] [--mock]
"""
import argparse, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ragfetch, ragcarta


def _kb(v): return None if v is None else int(v * 1024)


def build_parser():
    ap = argparse.ArgumentParser(prog="coli carta", description="CartaColibri — local AI library (RAG-fetch + RAG-CARTA)")
    ap.add_argument("--home", help="state directory (default ~/.cartacolibri or $CARTACOLIBRI_HOME)")
    sub = ap.add_subparsers(dest="cmd", required=True)
    for n in ("add", "remove"): sub.add_parser(n).add_argument("folder")
    sub.add_parser("folders")
    sub.add_parser("index").add_argument("folder", nargs="?")
    sub.add_parser("problems")
    p = sub.add_parser("list"); p.add_argument("--group")
    p = sub.add_parser("search"); p.add_argument("query")
    p.add_argument("--ext", help="comma-separated, e.g. pdf,txt"); p.add_argument("--min-kb", type=float)
    p.add_argument("--max-kb", type=float); p.add_argument("--path"); p.add_argument("--group"); p.add_argument("--limit", type=int, default=20)
    g = sub.add_parser("group"); gs = g.add_subparsers(dest="gcmd", required=True)
    q = gs.add_parser("set"); q.add_argument("name"); q.add_argument("members", nargs="*"); q.add_argument("-d", "--description"); q.add_argument("--add", action="store_true")
    gs.add_parser("list"); gs.add_parser("show").add_argument("name"); gs.add_parser("rm").add_argument("name")
    q = gs.add_parser("from-search"); q.add_argument("name"); q.add_argument("query"); q.add_argument("--ext"); q.add_argument("--limit", type=int, default=20)
    for t in ragcarta.TASKS:
        p = sub.add_parser(t); p.add_argument("request", nargs="?", default="")
        p.add_argument("--group"); p.add_argument("--query", help="use the files matching this search instead of a group")
        p.add_argument("--fields"); p.add_argument("--json", action="store_true"); p.add_argument("--save", action="store_true")
        p.add_argument("--url", default=os.environ.get("COLI_URL", ragcarta.DEFAULT_URL)); p.add_argument("--model")
        p.add_argument("--mock", action="store_true", help="offline extractive fallback (no model)")
        p.add_argument("--budget", type=int, default=ragcarta.CTX_CHARS, help="source characters per model call")
    return ap


def main(argv=None):
    a = build_parser().parse_args(argv)
    lib = ragfetch.Library(a.home)
    out = print
    if a.cmd == "add":
        out("registered", lib.add_root(a.folder)); out(lib.update([a.folder]))
    elif a.cmd == "remove": out(f"forgot {a.folder} ({lib.remove_root(a.folder)} documents dropped from the index; files untouched)")
    elif a.cmd == "folders":
        for r in lib.roots(): out(r)
    elif a.cmd == "index": out(lib.update([a.folder] if a.folder else None, log=lambda m: print(m, file=sys.stderr)))
    elif a.cmd == "problems":
        for p in lib.problems(): out(f"{p['path']}\n    {p['reason']}")
    elif a.cmd == "list":
        for d in lib.documents(a.group): out(f"{d['size']//1024:>7} KB  {d['ext']:<6} {d['path']}")
    elif a.cmd == "search":
        hits = lib.search(a.query, ext=a.ext.split(",") if a.ext else None, min_size=_kb(a.min_kb), max_size=_kb(a.max_kb),
                          path_like=a.path, group=a.group, limit=a.limit)
        for h in hits: out(f"{h['score']:>7}  {h['path']} {h['loc']}\n         {h['snippet']}")
        if not hits: out("no matches")
    elif a.cmd == "group":
        if a.gcmd == "set": out(lib.group_set(a.name, a.members, a.description, a.add))
        elif a.gcmd == "list":
            for n, g in lib.groups().items(): out(f"{n}: {g['description']} ({len(lib.resolve_group(n))} docs)")
        elif a.gcmd == "show":
            for d in lib.resolve_group(a.name): out(d["path"])
        elif a.gcmd == "rm": lib.group_remove(a.name); out("removed", a.name)
        elif a.gcmd == "from-search":
            hits = lib.search(a.query, ext=a.ext.split(",") if a.ext else None, limit=a.limit)
            out(lib.group_set(a.name, [h["path"] for h in hits], f"search: {a.query}"))
    else:
        docs = (lib.resolve_group(a.group) if a.group else
                [d for d in lib.documents() if d["id"] in {h["id"] for h in lib.search(a.query, limit=50)}] if a.query else None)
        if docs is None: sys.exit("give --group NAME (or --query TEXT)")
        be = ragcarta.Backend("mock" if a.mock else "colibri", a.url, a.model)
        try:
            text = ragcarta.run(lib, a.cmd, docs, a.request, be, fields=a.fields.split(",") if a.fields else None,
                                fmt="json" if a.json else "table", budget=a.budget,
                                log=lambda m: print("·", m, file=sys.stderr))
        except ragcarta.GatewayError as e:
            sys.exit(str(e))
        out(text)
        if a.save: out("saved:", ragcarta.save(lib.home, text, a.cmd, a.group or "selection", a.request))
    return 0


if __name__ == "__main__":
    sys.exit(main())
