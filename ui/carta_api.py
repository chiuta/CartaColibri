"""CartaColibri's JSON API for the chat UI (/carta/...): the Library page (folders, documents,
view), RAG-fetch (search, groups) and RAG-CARTA (summarize / extract / synthesize).
Same rules as the Models page: this page on this machine only (server.py checks).
One ragfetch.Library per request — sqlite connections are not shared between threads."""
import json, os, sys, urllib.parse
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "c"))
import ragfetch, ragcarta   # noqa: E402


def handle(method, path, query, body, backend_url, filename=None):
    q = {k: v[0] for k, v in urllib.parse.parse_qs(query).items()}
    j = json.loads(body) if body and path != "/carta/upload" else {}
    lib = ragfetch.Library(os.environ.get("CARTACOLIBRI_HOME"))
    try:
        route = path[len("/carta/"):].strip("/")
        if route == "library" and method == "GET":
            return 200, {"roots": lib.roots(), "documents": lib.documents(), "groups": lib.groups(),
                         "problems": lib.problems(), "home": lib.home}
        if route == "folders" and method == "POST":
            lib.add_root(j["path"]); return 200, lib.update([j["path"]])
        if route == "folders" and method == "DELETE":
            return 200, {"dropped": lib.remove_root(j["path"])}
        if route == "upload" and method == "POST":
            return 200, lib.add_upload(filename or "document", body, q["ocr"].split(",") if q.get("ocr") else None)
        if route == "status" and method == "GET":
            return 200, {"ocr": ragfetch.ocr_available(), "home": lib.home, "version": 1}
        if route == "index" and method == "POST":
            return 200, lib.update()
        if route == "search" and method == "GET":
            return 200, lib.search(q.get("q", ""), ext=q["ext"].split(",") if q.get("ext") else None,
                                   min_size=int(q["min"]) if q.get("min") else None,
                                   max_size=int(q["max"]) if q.get("max") else None,
                                   path_like=q.get("path"), group=q.get("group"))
        if route == "doc" and method == "GET":
            return 200, {"text": lib.text_of(int(q["id"]))}
        if route == "groups" and method == "POST":
            return 200, lib.group_set(j["name"], j.get("members"), j.get("description"), bool(j.get("add")))
        if route == "groups" and method == "DELETE":
            lib.group_remove(j["name"], j.get("members")); return 200, {"ok": True}
        if route == "run" and method == "POST":
            ids = j.get("ids")
            docs = (lib.resolve_group(j["group"]) if j.get("group")
                    else [d for d in lib.documents() if d["id"] in set(ids or [])])
            be = ragcarta.Backend("mock" if j.get("mock") else "colibri", backend_url)
            text = ragcarta.run(lib, j["task"], docs, j.get("request", ""), be, fields=j.get("fields"),
                                fmt=j.get("fmt", "table"))
            saved = ragcarta.save(lib.home, text, j["task"], j.get("group"), j.get("request", "")) if j.get("save") else None
            return 200, {"text": text, "saved": saved}
        return 404, {"error": "no such carta route"}
    except ragcarta.GatewayError as e:
        return 502, {"error": str(e)}
    except (KeyError, ValueError) as e:
        return 400, {"error": f"{type(e).__name__}: {e}"}
    finally:
        lib.db.close()
