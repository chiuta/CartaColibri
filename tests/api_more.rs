//! Black-box tests of the newer /carta/* routes (progress, tags/stars/notes, saved searches, tasks,
//! chat, export, stats, backup, import, feeds). Same raw-HTTP approach as tests/security.rs.

use cartacolibri::server;
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

struct Srv {
    _t: tempfile::TempDir,
    port: u16,
    docs: std::path::PathBuf,
}

fn start() -> Srv {
    let t = tempfile::tempdir().unwrap();
    let home = t.path().join("home");
    let docs = t.path().join("docs");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(
        docs.join("contract.txt"),
        "The payment terms are thirty days after the invoice date. Late payment costs interest.\n\nThe supplier delivers the goods within ten days.",
    )
    .unwrap();
    std::fs::write(
        docs.join("offer.txt"),
        "Our offer covers delivery of goods. The payment terms are sixty days. Delivery takes two weeks.",
    )
    .unwrap();
    std::fs::write(
        docs.join("notes.md"),
        "# Meeting\nWe discussed delivery of the goods and the invoice.",
    )
    .unwrap();
    let h = home.to_string_lossy().into_owned();
    // a free port is picked and released before the server binds it, so another test (or process) can take
    // it in between: when the server stops at once, or never answers, try again on a new port
    let mut port = 0;
    for _attempt in 0..8 {
        port = {
            let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().port()
        };
        let (addr, h) = (format!("127.0.0.1:{port}"), h.clone());
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let r = server::serve(&addr, Some(h), "http://127.0.0.1:9".into());
            let _ = tx.send(r.is_ok());
        });
        let mut up = false;
        for _ in 0..100 {
            if rx.try_recv().is_ok() {
                break; // the server returned: it could not start on this port
            }
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                up = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if up {
            break;
        }
    }
    Srv { _t: t, port, docs }
}

struct Resp {
    code: u16,
    head: String,
    body: Vec<u8>,
}

impl Resp {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body)
            .unwrap_or_else(|_| panic!("not JSON: {}", String::from_utf8_lossy(&self.body)))
    }
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

fn raw(s: &Srv, method: &str, path: &str, headers: &[(&str, &str)], body: &[u8]) -> Resp {
    let mut c = TcpStream::connect(("127.0.0.1", s.port)).unwrap();
    c.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
    let mut r = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
        s.port
    );
    for (k, v) in headers {
        r.push_str(&format!("{k}: {v}\r\n"));
    }
    r.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    c.write_all(r.as_bytes()).unwrap();
    let _ = c.write_all(body);
    let mut buf = Vec::new();
    let _ = c.read_to_end(&mut buf);
    let at = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .unwrap_or(buf.len());
    let head = String::from_utf8_lossy(&buf[..at]).into_owned();
    let code = head
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    Resp {
        code,
        head,
        body: buf.get(at + 4..).unwrap_or(&[]).to_vec(),
    }
}

fn ui(s: &Srv, method: &str, path: &str, body: &[u8]) -> Resp {
    raw(s, method, path, &[("X-Colibri-UI", "1")], body)
}

fn get(s: &Srv, path: &str) -> Resp {
    ui(s, "GET", path, b"")
}

fn post(s: &Srv, path: &str, v: Value) -> Resp {
    ui(s, "POST", path, v.to_string().as_bytes())
}

/// Starts the server with the sample documents indexed; returns their ids by file name.
fn indexed() -> (Srv, std::collections::BTreeMap<String, i64>) {
    let s = start();
    let r = post(
        &s,
        "/carta/folders",
        json!({"path": s.docs.to_string_lossy()}),
    );
    assert_eq!(r.code, 200, "{}", r.text());
    let lib = get(&s, "/carta/library").json();
    let ids = lib["documents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            let p = d["path"].as_str().unwrap();
            (
                p.rsplit(['/', '\\']).next().unwrap().to_string(),
                d["id"].as_i64().unwrap(),
            )
        })
        .collect();
    (s, ids)
}

#[test]
fn progress_shape_and_background_index() {
    let (s, _) = indexed();
    let p = get(&s, "/carta/progress");
    assert_eq!(p.code, 200);
    let j = p.json();
    for k in [
        "running", "seen", "total", "changed", "skipped", "failed", "current",
    ] {
        assert!(j.get(k).is_some(), "progress lacks {k}: {j}");
    }
    assert!(j["running"].is_boolean() && j["seen"].is_number() && j["current"].is_string());
    let r = post(&s, "/carta/index", json!({"background": true}));
    assert_eq!(r.code, 200, "{}", r.text());
    let mut done = false;
    for _ in 0..200 {
        let j = get(&s, "/carta/progress").json();
        if j["running"] == false && j["finished"] == true {
            assert!(j["error"].is_null(), "{j}");
            assert!(j["total"].as_u64().unwrap() >= 3, "{j}");
            done = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(done, "background index never finished");
}

#[test]
fn tags_star_note_roundtrip() {
    let (s, ids) = indexed();
    let id = ids["contract.txt"];
    let r = post(
        &s,
        "/carta/tags",
        json!({"id": id, "add": ["#Legal", "urgent"]}),
    );
    assert_eq!(r.code, 200, "{}", r.text());
    let tags = r.json()["tags"].clone();
    assert_eq!(tags, json!(["legal", "urgent"]));
    assert_eq!(
        post(&s, "/carta/star", json!({"id": id, "on": true})).code,
        200
    );
    assert_eq!(
        post(
            &s,
            "/carta/note",
            json!({"id": id, "note": "check clause 4"})
        )
        .code,
        200
    );
    let m = get(&s, &format!("/carta/docmeta?id={id}")).json();
    assert_eq!(m["tags"], json!(["legal", "urgent"]));
    assert_eq!(m["starred"], true);
    assert_eq!(m["note"], "check clause 4");
    assert!(m["related"].is_array());
    let meta = get(&s, "/carta/meta").json();
    assert_eq!(meta["tags"]["legal"], 1);
    // filtered search
    let f = get(&s, "/carta/find?q=payment&tag=legal").json();
    assert_eq!(f["hits"].as_array().unwrap().len(), 1, "{f}");
    let f = get(&s, "/carta/find?q=payment&starred=1").json();
    assert_eq!(f["hits"].as_array().unwrap().len(), 1);
    // remove + unstar
    let r = post(&s, "/carta/tags", json!({"id": id, "remove": ["urgent"]})).json();
    assert_eq!(r["tags"], json!(["legal"]));
    post(&s, "/carta/star", json!({"id": id, "on": false}));
    assert_eq!(
        get(&s, &format!("/carta/docmeta?id={id}")).json()["starred"],
        false
    );
    // open counting feeds the Recent strip
    assert_eq!(post(&s, "/carta/open", json!({"id": id})).code, 200);
    let rc = get(&s, "/carta/recent").json();
    assert_eq!(rc["recent"][0]["id"], id);
    // bad input is refused, not stored
    assert_eq!(
        post(&s, "/carta/tags", json!({"id": id, "add": ["a\nb"]})).code,
        400
    );
    assert_eq!(
        post(&s, "/carta/tags", json!({"id": 999999, "add": ["x"]})).code,
        404
    );
}

#[test]
fn saved_search_roundtrip() {
    let (s, _) = indexed();
    let r = post(
        &s,
        "/carta/saved",
        json!({"name": "pay", "query": "payment"}),
    );
    assert_eq!(r.code, 200, "{}", r.text());
    let l = get(&s, "/carta/saved").json();
    assert_eq!(l["saved"][0]["name"], "pay");
    let run = get(&s, "/carta/saved/run?name=pay").json();
    assert_eq!(run["hits"].as_array().unwrap().len(), 2);
    assert_eq!(
        post(&s, "/carta/saved", json!({"name": "", "query": "x"})).code,
        400
    );
    assert_eq!(get(&s, "/carta/saved/run?name=nope").code, 404);
    let d = ui(
        &s,
        "DELETE",
        "/carta/saved",
        json!({"name": "pay"}).to_string().as_bytes(),
    );
    assert_eq!(d.json()["removed"], true);
    assert!(
        get(&s, "/carta/saved").json()["saved"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn tasks_and_mock_run_and_chat() {
    let (s, ids) = indexed();
    let t = get(&s, "/carta/tasks").json();
    let names: Vec<&str> = t["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    assert_eq!(names.len(), 14);
    for k in [
        "compare",
        "contradictions",
        "flashcards",
        "translate",
        "quiz",
    ] {
        assert!(names.contains(&k), "missing task {k}");
    }
    let docs = [ids["contract.txt"], ids["offer.txt"]];
    let r = post(
        &s,
        "/carta/run",
        json!({"task": "compare", "ids": docs, "mock": true, "request": "payment terms", "verify": true,
               "lang": "en", "save": true, "save_fmt": "html"}),
    );
    assert_eq!(r.code, 200, "{}", r.text());
    let j = r.json();
    assert!(j["text"].as_str().unwrap().contains("[S1]"), "{j}");
    assert!(j["saved"].as_str().unwrap().ends_with(".html"));
    let r = post(
        &s,
        "/carta/run",
        json!({"task": "translate", "ids": docs, "mock": true, "target": "French"}),
    );
    assert_eq!(r.code, 200, "{}", r.text());
    assert_eq!(
        post(
            &s,
            "/carta/run",
            json!({"task": "compare", "ids": docs, "mock": true, "save": true, "save_fmt": "exe"})
        )
        .code,
        400
    );
    // chat: kept per session id, cited sources, honest "not found"
    let c = post(
        &s,
        "/carta/chat",
        json!({"sid": "page-1", "q": "What are the payment terms?", "mock": true}),
    );
    assert_eq!(c.code, 200, "{}", c.text());
    let j = c.json();
    assert_eq!(j["found"], true, "{j}");
    assert!(j["sources"][0]["label"] == "S1" && j["sources"][0]["path"].is_string());
    let n = post(
        &s,
        "/carta/chat",
        json!({"sid": "page-1", "q": "zzqxyvw plugh", "mock": true}),
    );
    assert_eq!(n.code, 200);
    assert_eq!(n.json()["found"], false);
    assert_eq!(
        post(
            &s,
            "/carta/chat",
            json!({"sid": "../x", "q": "hi", "mock": true})
        )
        .code,
        400
    );
    assert_eq!(
        post(&s, "/carta/chat/reset", json!({"sid": "page-1"})).code,
        200
    );
    // a saved conversation can be put back; a bad id or malformed pairs are handled
    assert_eq!(
        post(
            &s,
            "/carta/chat/restore",
            json!({"sid": "page-2", "history": [["What are the payment terms?", "Net 30. [S1]"], [1, 2], "x"]})
        )
        .code,
        200
    );
    assert_eq!(
        post(
            &s,
            "/carta/chat/restore",
            json!({"sid": "../x", "history": []})
        )
        .code,
        400
    );
    // the web-grab browser is not built next to the test binary: an honest 404, never a panic
    let b = post(&s, "/carta/browser/open", json!({}));
    assert_eq!(b.code, 404, "{}", b.text());
}

#[test]
fn find_has_did_you_mean_shape() {
    let (s, _) = indexed();
    let j = get(&s, "/carta/find?q=paymnet").json();
    assert!(j["hits"].is_array() && j["suggestions"].is_array() && j.get("corrected").is_some());
    assert!(
        j["corrected"].is_string() || !j["suggestions"].as_array().unwrap().is_empty(),
        "{j}"
    );
    let d = get(&s, "/carta/duplicates").json();
    assert!(d["groups"].is_array());
}

#[test]
fn export_formats() {
    let (s, ids) = indexed();
    post(
        &s,
        "/carta/tags",
        json!({"id": ids["offer.txt"], "add": ["offer"]}),
    );
    let md = get(&s, "/carta/export?format=md&q=payment");
    assert_eq!(md.code, 200);
    assert!(
        md.head
            .to_lowercase()
            .contains("content-disposition: attachment; filename=\"")
    );
    assert!(md.text().contains("contract"));
    let csv = get(&s, "/carta/export?format=csv&q=payment");
    assert!(csv.head.contains(".csv\""));
    assert!(csv.text().lines().count() >= 3);
    let js = get(&s, "/carta/export?format=json&group=");
    let j = js.json();
    assert_eq!(j["format"], "cartacolibri-export");
    assert_eq!(j["items"].as_array().unwrap().len(), 3);
    let by = get(
        &s,
        &format!("/carta/export?format=json&ids={}", ids["offer.txt"]),
    )
    .json();
    assert_eq!(by["items"][0]["tags"], json!(["offer"]));
    let z = get(&s, "/carta/export?format=zip&q=payment");
    assert_eq!(z.code, 200);
    assert!(z.head.contains(".zip\""));
    let mut zr = zip::ZipArchive::new(std::io::Cursor::new(z.body)).unwrap();
    let names: Vec<String> = (0..zr.len())
        .map(|i| zr.by_index(i).unwrap().name().unwrap().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n == "citations.md")
            && names.iter().any(|n| n.starts_with("sources/"))
    );
    // saved search + group sources
    post(
        &s,
        "/carta/saved",
        json!({"name": "pay", "query": "payment"}),
    );
    assert_eq!(get(&s, "/carta/export?format=md&saved=pay").code, 200);
    assert_ne!(get(&s, "/carta/export?format=md&saved=missing").code, 200);
}

#[test]
fn stats_timeline_and_backup() {
    let (s, _) = indexed();
    let st = get(&s, "/carta/stats").json();
    assert_eq!(st["docs"], 3);
    for k in [
        "total_bytes",
        "by_type",
        "by_folder",
        "by_year",
        "biggest",
        "languages",
    ] {
        assert!(st.get(k).is_some(), "stats lacks {k}");
    }
    let tl = get(&s, "/carta/timeline").json();
    assert!(
        tl["months"][0]["month"].as_str().unwrap().len() == 7
            && tl["months"][0]["count"].is_number()
    );
    let v = post(&s, "/carta/verify", json!({"repair": false})).json();
    assert_eq!(v["ok"], true, "{v}");
    let b = get(&s, "/carta/backup");
    assert_eq!(b.code, 200);
    assert!(b.head.contains("application/zip") && b.head.contains("cartacolibri-backup-"));
    assert_eq!(&b.body[..2], b"PK");
    let z = zip::ZipArchive::new(std::io::Cursor::new(b.body.clone())).unwrap();
    assert!(!z.is_empty());
    // the downloaded zip passes the library's own check
    let f = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(f.path(), &b.body).unwrap();
    assert!(cartacolibri::organize::backup_verify(f.path()).is_ok());
}

#[test]
fn import_and_feeds() {
    let (s, _) = indexed();
    let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1><DL><p><DT><H3>Reading</H3><DL><p>
        <DT><A HREF="https://example.org/a">Alpha page</A><DT><A HREF="https://example.org/b">Beta page</A></DL></DL>"#;
    let r = ui_h(
        &s,
        "/carta/import",
        &[("X-File-Name", "bookmarks.html")],
        html.as_bytes(),
    );
    assert_eq!(r.code, 200, "{}", r.text());
    assert!(r.json()["notes"].as_u64().unwrap() >= 1);
    assert!(
        !get(&s, "/carta/find?q=Alpha").json()["hits"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    // an Obsidian-style zip
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut zw = zip::ZipWriter::new(&mut buf);
        zw.start_file("vault/idea.md", zip::write::SimpleFileOptions::default())
            .unwrap();
        zw.write_all(b"# Idea\nA zeppelin note").unwrap();
        zw.start_file("../../evil.md", zip::write::SimpleFileOptions::default())
            .unwrap();
        zw.write_all(b"nope").unwrap();
        zw.finish().unwrap();
    }
    let r = ui_h(
        &s,
        "/carta/import",
        &[("X-File-Name", "vault.zip")],
        buf.get_ref(),
    );
    assert_eq!(r.code, 200, "{}", r.text());
    assert!(
        !get(&s, "/carta/find?q=zeppelin").json()["hits"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    // feeds: add / list / OPML out+in / remove (no network)
    let r = post(
        &s,
        "/carta/feeds",
        json!({"url": "https://example.org/feed.xml"}),
    );
    assert_eq!(r.json()["urls"], json!(["https://example.org/feed.xml"]));
    assert_eq!(
        post(&s, "/carta/feeds", json!({"url": "file:///etc/passwd"})).code,
        400
    );
    assert_eq!(
        post(&s, "/carta/feeds", json!({"url": "javascript:alert(1)"})).code,
        400
    );
    let o = get(&s, "/carta/feeds/opml");
    assert!(o.head.contains(".opml\"") && o.text().contains("example.org/feed.xml"));
    let o2 = ui_h(
        &s,
        "/carta/feeds/opml",
        &[("Content-Type", "text/xml")],
        br#"<opml version="2.0"><body><outline type="rss" text="X" xmlUrl="https://example.net/rss"/></body></opml>"#,
    );
    assert_eq!(o2.json()["added"], 1, "{}", o2.text());
    let d = ui(
        &s,
        "DELETE",
        "/carta/feeds",
        json!({"url": "https://example.net/rss"})
            .to_string()
            .as_bytes(),
    );
    assert_eq!(d.json()["urls"], json!(["https://example.org/feed.xml"]));
}

fn ui_h(s: &Srv, path: &str, extra: &[(&str, &str)], body: &[u8]) -> Resp {
    let mut h = vec![("X-Colibri-UI", "1")];
    h.extend_from_slice(extra);
    raw(s, "POST", path, &h, body)
}

#[test]
fn path_traversal_names_are_rejected() {
    let (s, _) = indexed();
    for n in [
        "../../evil.html",
        "..\\evil.html",
        "a/b.html",
        "..",
        "x\u{0}.html",
    ] {
        let enc: String = n.bytes().map(|b| format!("%{b:02X}")).collect();
        let r = ui_h(
            &s,
            "/carta/import",
            &[("X-File-Name", &enc)],
            b"<a href='http://x'>x</a>",
        );
        assert_eq!(r.code, 400, "import name {n:?} gave {}", r.code);
    }
    let r = ui_h(&s, "/carta/import", &[("X-File-Name", "tool.exe")], b"MZ");
    assert_eq!(r.code, 400);
    assert_eq!(
        get(&s, "/carta/export?format=md&name=..%2F..%2Fevil").code,
        400
    );
    assert_eq!(get(&s, "/carta/export?format=..%2Fx").code, 400);
    assert_ne!(
        get(&s, "/carta/export?format=md&saved=..%2F..%2Fetc%2Fpasswd").code,
        200
    );
    assert_ne!(
        get(&s, "/carta/export?format=md&group=..%2F..%2Fetc").code,
        200
    );
    // nothing was written outside the home
    let outside = s.docs.parent().unwrap();
    assert!(!outside.join("evil.html").exists() && !outside.join("evil.md").exists());
    // download names are always plain
    let ok = get(&s, "/carta/export?format=csv&name=my%20report%3F.csv");
    assert_eq!(ok.code, 200);
    assert!(
        ok.head.contains("filename=\"my_report.csv\""),
        "{}",
        ok.head
    );
}

#[test]
fn every_new_route_demands_the_ui_header() {
    let (s, _) = indexed();
    let routes: &[(&str, &str)] = &[
        ("GET", "progress"),
        ("GET", "tasks"),
        ("GET", "find?q=a"),
        ("GET", "duplicates"),
        ("GET", "meta"),
        ("GET", "tags"),
        ("GET", "docmeta?id=1"),
        ("POST", "open"),
        ("POST", "tags"),
        ("POST", "star"),
        ("POST", "note"),
        ("GET", "recent"),
        ("GET", "saved"),
        ("POST", "saved"),
        ("DELETE", "saved"),
        ("GET", "saved/run?name=x"),
        ("GET", "export?format=md"),
        ("GET", "stats"),
        ("GET", "timeline"),
        ("POST", "verify"),
        ("GET", "backup"),
        ("POST", "import"),
        ("GET", "feeds"),
        ("POST", "feeds"),
        ("DELETE", "feeds"),
        ("POST", "feeds/poll"),
        ("GET", "feeds/opml"),
        ("POST", "feeds/opml"),
        ("POST", "chat"),
        ("POST", "chat/reset"),
        ("POST", "chat/restore"),
        ("POST", "browser/open"),
        ("POST", "legatorocr/open"),
        ("POST", "index"),
    ];
    for (m, r) in routes {
        let res = raw(&s, m, &format!("/carta/{r}"), &[], b"{}");
        assert_eq!(
            res.code, 403,
            "{m} /carta/{r} without X-Colibri-UI gave {}",
            res.code
        );
        // and a foreign Origin is refused even with the header
        let res = raw(
            &s,
            m,
            &format!("/carta/{r}"),
            &[("X-Colibri-UI", "1"), ("Origin", "http://evil.example")],
            b"{}",
        );
        assert_eq!(
            res.code, 403,
            "{m} /carta/{r} with a foreign Origin gave {}",
            res.code
        );
    }
}

#[test]
fn json_body_limit_still_applies_to_new_routes() {
    let (s, _) = indexed();
    let big = vec![b' '; 2 * 1024 * 1024 + 10];
    let r = ui(&s, "POST", "/carta/note", &big);
    assert_eq!(r.code, 413);
    let r = ui(&s, "POST", "/carta/chat", &big);
    assert_eq!(r.code, 413);
}

#[test]
fn engine_address_is_saved_validated_and_reset() {
    let (s, _ids) = indexed();
    let val = |r: Resp| -> Value { serde_json::from_slice(&r.body).unwrap_or(Value::Null) };
    assert_eq!(val(get(&s, "/carta/settings"))["gateway_url"], "");
    for bad in [
        "ftp://x",
        "javascript:alert(1)",
        "http://u:p@127.0.0.1:8000",
        "http://a b",
        "not a url",
    ] {
        let r = post(&s, "/carta/settings", json!({"gateway_url": bad}));
        assert_eq!(r.code, 400, "{bad}");
    }
    let r = post(
        &s,
        "/carta/settings",
        json!({"gateway_url": "http://127.0.0.1:9"}),
    );
    assert_eq!(r.code, 200);
    let r = val(get(&s, "/carta/settings"));
    assert_eq!(r["gateway_url"], "http://127.0.0.1:9");
    assert_eq!(r["effective"], "http://127.0.0.1:9");
    post(&s, "/carta/settings", json!({"gateway_url": ""}));
    assert_eq!(val(get(&s, "/carta/settings"))["gateway_url"], "");
}

#[test]
fn start_companion_never_runs_anything_the_page_chose() {
    let (s, _ids) = indexed();
    // the page sends no program and no arguments; with none installed it is a clean 404
    let r = post(
        &s,
        "/carta/engine/start",
        json!({"cmd": "touch /tmp/pwned"}),
    );
    assert!(matches!(r.code, 200 | 404), "{}", r.code);
    assert!(!std::path::Path::new("/tmp/pwned").exists());
}

#[test]
fn query_mode_chat_inline_filters_and_smart_flag() {
    let (s, _ids) = indexed();
    let val = |r: Resp| -> Value { serde_json::from_slice(&r.body).unwrap_or(Value::Null) };
    // single-question mode answers with sources and keeps no session
    for _ in 0..2 {
        let c = post(
            &s,
            "/carta/chat",
            json!({"sid": "q-1", "q": "What are the payment terms?", "mock": true, "mode": "query"}),
        );
        assert_eq!(c.code, 200, "{}", c.text());
        assert_eq!(c.json()["found"], true);
    }
    // inline filters through the web search route
    let all = val(get(&s, "/carta/find?q=payment"));
    let n_all = all["hits"].as_array().map_or(0, Vec::len);
    assert!(n_all > 0);
    let none = val(get(&s, "/carta/find?q=payment%20ext%3Anope"));
    assert_eq!(none["hits"].as_array().map_or(99, Vec::len), 0);
    // smart search never fails: with no engine it is the plain search plus an empty expansion
    let sm = val(get(&s, "/carta/find?q=payment&smart=1"));
    assert_eq!(sm["hits"].as_array().map_or(0, Vec::len), n_all);
    assert_eq!(sm["expanded"].as_array().map_or(99, Vec::len), 0);
}

#[test]
fn grounding_entities_and_compare_routes() {
    let (s, ids) = indexed();
    let val = |r: Resp| -> Value { serde_json::from_slice(&r.body).unwrap_or(Value::Null) };
    // every chat answer now carries a grounding report
    let c = post(
        &s,
        "/carta/chat",
        json!({"sid": "g-1", "q": "What are the payment terms?", "mock": true, "mode": "query"}),
    );
    assert_eq!(c.code, 200, "{}", c.text());
    let g = &c.json()["grounding"];
    assert!(g["score"].as_u64().is_some() && g["claims"].is_array());
    // entities: a clean error for an unknown kind, a list otherwise
    assert_eq!(get(&s, "/carta/entities?kind=nope").code, 400);
    let e = val(get(&s, "/carta/entities?kind=email"));
    assert!(e["entities"].is_array());
    // compare two indexed documents
    let idv: Vec<i64> = ids.values().copied().collect();
    assert!(idv.len() >= 2);
    let cmp = val(get(
        &s,
        &format!("/carta/compare?a={}&b={}", idv[0], idv[1]),
    ));
    assert!(cmp["compare"]["similarity"].as_u64().is_some());
    assert_eq!(get(&s, "/carta/compare?a=1").code, 400);
    assert_eq!(get(&s, "/carta/compare?a=1&b=999999").code, 404);
}

#[test]
fn conflicts_topics_dossier_changes_factcheck_routes() {
    let (s, _ids) = indexed();
    assert!(get(&s, "/carta/conflicts").json()["conflicts"].is_array());
    assert!(get(&s, "/carta/dupes?min=0.5").json()["dupes"].is_array());
    assert!(get(&s, "/carta/chronology").json()["chronology"].is_array());
    // the monthly count keeps its own route
    assert!(get(&s, "/carta/timeline").json()["months"].is_array());
    assert!(get(&s, "/carta/topics").json()["topics"].is_array());
    let d = get(&s, "/carta/dossier?q=payment").json();
    assert!(d["docs"].as_array().is_some_and(|a| !a.is_empty()));
    assert_eq!(get(&s, "/carta/dossier?q=").code, 400);
    assert_eq!(get(&s, "/carta/changes").code, 400);
    assert!(
        get(&s, "/carta/changes?since=2000").json()["changes"]
            .as_array()
            .is_some_and(|a| a.len() >= 3)
    );
    let f = post(
        &s,
        "/carta/factcheck",
        json!({"text": "The payment terms are thirty days after the invoice date."}),
    );
    assert_eq!(f.code, 200, "{}", f.text());
    assert_eq!(f.json()["claims"][0]["status"], "supported");
    assert_eq!(
        post(&s, "/carta/factcheck", json!({"text": "  "})).code,
        400
    );
}

#[test]
fn chat_can_be_limited_to_a_time_range() {
    let (s, _ids) = indexed();
    let ask = |extra: Value| {
        let mut b = json!({"sid": "t-1", "q": "What are the payment terms?", "mock": true, "mode": "query"});
        b.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        post(&s, "/carta/chat", b)
    };
    assert_eq!(ask(json!({"after": "2000"})).json()["found"], true);
    assert_eq!(ask(json!({"before": "2000"})).json()["found"], false);
    assert_eq!(ask(json!({"after": "2999"})).json()["found"], false);
}

#[cfg(unix)]
#[test]
fn legatorocr_window_opens_or_says_why_not() {
    use std::os::unix::fs::PermissionsExt;
    let (s, _) = indexed();
    let dir = std::env::temp_dir().join(format!("cc-lgr-gui-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let put = |name: &str, body: &str| {
        let p = dir.join(name);
        std::fs::write(&p, body).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p
    };
    let cli = put("legatorocr", "#!/bin/sh\necho \"legatorocr 0.0-test\"\n");
    // the only test of this file that touches the variable; it is removed at the end
    unsafe { std::env::set_var("CARTACOLIBRI_LEGATOROCR", &cli) };
    // the program is there but not its window: an honest 404
    let none = post(&s, "/carta/legatorocr/open", json!({}));
    assert_eq!(none.code, 404, "{}", none.text());
    // the window program ends at once with an error: the reason is passed on, not "opening"
    put(
        "legatorocr-gui",
        "#!/bin/sh\necho 'no display found' >&2\nexit 3\n",
    );
    let bad = post(&s, "/carta/legatorocr/open", json!({}));
    assert_eq!(bad.code, 500, "{}", bad.text());
    assert!(bad.text().contains("no display found"), "{}", bad.text());
    // a launcher that hands over to the real window and ends with success is fine
    put("legatorocr-gui", "#!/bin/sh\nexit 0\n");
    let hand = post(&s, "/carta/legatorocr/open", json!({}));
    assert_eq!(hand.code, 200, "{}", hand.text());
    // one that stays open is a success
    put("legatorocr-gui", "#!/bin/sh\nsleep 4\n");
    let ok = post(&s, "/carta/legatorocr/open", json!({}));
    assert_eq!(ok.code, 200, "{}", ok.text());
    unsafe { std::env::remove_var("CARTACOLIBRI_LEGATOROCR") };
    let _ = std::fs::remove_dir_all(&dir);
}
