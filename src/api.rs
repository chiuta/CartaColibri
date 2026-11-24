//! JSON API behind the chat UI (/carta/...): the Library page, RAG-fetch and RAG-CARTA.
//! One Library (one SQLite connection) per request — connections are never shared between threads.

use crate::{
    carta, feeds, grab, import,
    library::{Filter, Library, Progress, Stats},
    organize,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

// ---------------------------------------------------------------- background indexing job

#[derive(Default, Clone)]
struct Job {
    running: bool,
    p: Progress,
    error: Option<String>,
    stats: Option<Stats>,
    runs: u64,
}

fn lock<T>(m: &'static Mutex<T>) -> MutexGuard<'static, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn jobs() -> &'static Mutex<HashMap<String, Job>> {
    static J: OnceLock<Mutex<HashMap<String, Job>>> = OnceLock::new();
    J.get_or_init(Default::default)
}

fn begin(key: &str) -> Result<(), String> {
    let mut g = lock(jobs());
    let j = g.entry(key.to_string()).or_default();
    if j.running {
        return Err("indexing is already running, wait for it to finish".into());
    }
    j.running = true;
    j.p = Progress::default();
    j.error = None;
    j.stats = None;
    j.runs += 1;
    Ok(())
}

fn finish(key: &str, r: &Result<Stats, String>) {
    let mut g = lock(jobs());
    let j = g.entry(key.to_string()).or_default();
    j.running = false;
    match r {
        Ok(s) => j.stats = Some(s.clone()),
        Err(m) => j.error = Some(m.clone()),
    }
}

/// Runs the update with live progress (the job must have been `begin`-ed).
fn run_job(lib: &mut Library, key: &str, roots: Option<&[String]>) -> Result<Stats, String> {
    let k = key.to_string();
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        lib.update_progress(
            roots,
            &mut |p| {
                lock(jobs()).entry(k.clone()).or_default().p = p.clone();
            },
            &mut |_| {},
        )
    }))
    .unwrap_or_else(|_| Err("indexing stopped unexpectedly".into()));
    finish(key, &r);
    r
}

fn index_now(lib: &mut Library, roots: Option<&[String]>) -> Result<Stats, String> {
    let key = lib.home.to_string_lossy().into_owned();
    begin(&key)?;
    run_job(lib, &key, roots)
}

fn progress_json(key: &str) -> Value {
    let j = lock(jobs()).get(key).cloned().unwrap_or_default();
    let st = j.stats.clone().unwrap_or_default();
    json!({"running": j.running, "seen": j.p.seen, "total": j.p.total, "changed": j.p.changed,
        "unchanged": j.p.unchanged, "skipped": j.p.skipped, "failed": j.p.failed,
        "current": j.p.current, "root": j.p.root, "error": j.error, "runs": j.runs,
        "finished": !j.running && j.runs > 0,
        "added": st.added, "updated": st.updated, "removed": st.removed})
}

// ---------------------------------------------------------------- chat sessions (server side, bounded)

const MAX_CHATS: usize = 16;
static TICK: AtomicU64 = AtomicU64::new(0);

fn chats() -> &'static Mutex<HashMap<String, (carta::ChatSession, u64)>> {
    static C: OnceLock<Mutex<HashMap<String, (carta::ChatSession, u64)>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

fn valid_sid(s: &str) -> Result<&str, String> {
    if (1..=64).contains(&s.len())
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        Ok(s)
    } else {
        Err("bad session id".into())
    }
}

// ---------------------------------------------------------------- validation helpers

fn clean_text(v: &str, max: usize, what: &str) -> Result<String, String> {
    let t = v.trim();
    if t.is_empty() || t.chars().count() > max || t.chars().any(|c| c.is_control()) {
        return Err(format!("bad {what}"));
    }
    Ok(t.to_string())
}

/// A file name from the browser: no folders, no `..`, no control characters.
fn safe_name(n: &str) -> Result<String, String> {
    let t = n.trim();
    if t.is_empty()
        || t.chars().count() > 150
        || t.contains(['/', '\\'])
        || t.starts_with("..")
        || t.chars().any(|c| c.is_control())
    {
        return Err("that file name is not allowed".into());
    }
    Ok(t.to_string())
}

fn slug(s: &str, max: usize) -> String {
    let v: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .take(max)
        .collect();
    let v = v.trim_matches('_').to_string();
    if v.is_empty() { "export".into() } else { v }
}

fn http_url(u: &str) -> Result<String, String> {
    let t = u.trim();
    if t.len() > 2000
        || !(t.starts_with("http://") || t.starts_with("https://"))
        || t.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err("that is not a web address (it must start with http:// or https://)".into());
    }
    Ok(t.to_string())
}

fn civil(secs: u64) -> String {
    let z = (secs / 86400) as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

fn today() -> String {
    civil(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    )
}

fn tmp_path(home: &Path, tail: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    home.join(format!(
        "web-{}-{}{tail}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ))
}

fn feeds_file(lib: &Library) -> PathBuf {
    lib.home.join("feed-urls.json")
}
fn feed_urls(lib: &Library) -> Vec<String> {
    std::fs::read_to_string(feeds_file(lib))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}
fn set_feed_urls(lib: &Library, v: &[String]) -> Result<(), String> {
    std::fs::write(feeds_file(lib), serde_json::to_string_pretty(v).unwrap())
        .map_err(|e| e.to_string())
}

fn doc_of(lib: &Library, j: &Value) -> Result<crate::library::Doc, (u16, String)> {
    let id = j["id"].as_i64().ok_or((400, "missing 'id'".to_string()))?;
    lib.docs_by_id(&[id])
        .into_iter()
        .next()
        .ok_or((404, "that document is not in the library".to_string()))
}

fn filter_of(q: &HashMap<String, String>) -> Filter {
    Filter {
        ext: csv(q, "ext"),
        min_size: q.get("min").and_then(|v| v.parse().ok()),
        max_size: q.get("max").and_then(|v| v.parse().ok()),
        path_like: q.get("path").cloned().filter(|p| !p.is_empty()),
        group: q.get("group").cloned().filter(|p| !p.is_empty()),
        limit: None,
        after: q.get("after").and_then(|v| crate::library::parse_date(v)),
        before: q.get("before").and_then(|v| crate::library::parse_date(v)),
        regex: q.get("re").cloned().filter(|p| !p.is_empty()),
    }
}

/// Unpacks an Obsidian-style zip into `dest`, refusing anything outside it and bounding the size.
fn unzip_safe(data: &[u8], dest: &Path) -> Result<(), String> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(data))
        .map_err(|_| "that zip file is damaged or not a zip".to_string())?;
    if z.len() > 20_000 {
        return Err("that zip has too many files".into());
    }
    let (mut total, cap) = (0u64, 400u64 * 1024 * 1024);
    for i in 0..z.len() {
        let mut f = z.by_index(i).map_err(|e| e.to_string())?;
        let Some(rel) = f.enclosed_name() else {
            continue;
        };
        if f.is_dir() {
            continue;
        }
        // the sizes in a zip header are only claims: count what is really written
        if total.saturating_add(f.size()) > cap {
            return Err("that zip is too large once unpacked".into());
        }
        let out = dest.join(&rel);
        if let Some(p) = out.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let mut o = std::fs::File::create(&out).map_err(|e| e.to_string())?;
        let room = cap - total;
        let n = std::io::copy(&mut std::io::Read::take(&mut f, room + 1), &mut o)
            .map_err(|_| "that zip file is damaged".to_string())?;
        total += n;
        if total > cap {
            return Err("that zip is too large once unpacked".into());
        }
    }
    Ok(())
}

pub struct Download {
    pub name: String,
    pub ctype: &'static str,
    pub bytes: Vec<u8>,
}

/// Routes that answer with a file instead of JSON.
pub fn is_download(method: &str, route: &str) -> bool {
    method == "GET" && matches!(route, "export" | "backup" | "feeds/opml")
}

pub fn download(home: Option<&str>, r: &Req) -> Result<Download, (u16, String)> {
    let bad = |m: String| (400u16, m);
    let lib = Library::open(home, false).map_err(|m| (500, m))?;
    let q = &r.query;
    match r.route {
        "backup" => {
            let out = tmp_path(&lib.home, "-backup.zip.tmp");
            let res = organize::backup(&lib, &out, q.get("index").is_some_and(|v| v == "1"));
            let bytes = std::fs::read(&out);
            let _ = std::fs::remove_file(&out);
            res.map_err(bad)?;
            Ok(Download {
                name: format!("cartacolibri-backup-{}.zip", today()),
                ctype: "application/zip",
                bytes: bytes.map_err(|e| (500, e.to_string()))?,
            })
        }
        "feeds/opml" => {
            let subs: Vec<feeds::Subscription> = feed_urls(&lib)
                .into_iter()
                .map(|u| feeds::Subscription {
                    title: u.clone(),
                    url: u,
                })
                .collect();
            Ok(Download {
                name: "cartacolibri-feeds.opml".into(),
                ctype: "text/x-opml; charset=utf-8",
                bytes: feeds::opml_export(&subs).into_bytes(),
            })
        }
        _ => {
            let fmt = q.get("format").map(String::as_str).unwrap_or("md");
            let (ext, ctype) = match fmt {
                "md" => ("md", "text/markdown; charset=utf-8"),
                "csv" => ("csv", "text/csv; charset=utf-8"),
                "json" => ("json", "application/json; charset=utf-8"),
                "zip" => ("zip", "application/zip"),
                _ => return Err(bad("format must be md, csv, json or zip".into())),
            };
            if let Some(n) = q.get("name") {
                safe_name(n).map_err(bad)?;
            }
            let (items, label) = if let Some(n) = q.get("saved").filter(|s| !s.is_empty()) {
                (organize::items_from_saved(&lib, n).map_err(bad)?, n.clone())
            } else if let Some(n) = q.get("group").filter(|s| !s.is_empty()) {
                (organize::items_from_group(&lib, n).map_err(bad)?, n.clone())
            } else if let Some(ids) = q.get("ids").filter(|s| !s.is_empty()) {
                let ids: Vec<i64> = ids
                    .split(',')
                    .take(5000)
                    .map(|x| x.trim().parse::<i64>())
                    .collect::<Result<_, _>>()
                    .map_err(|_| bad("bad ids".into()))?;
                (
                    organize::items_from_docs(&lib, &lib.docs_by_id(&ids)),
                    "selection".into(),
                )
            } else if q.get("q").is_some_and(|s| !s.trim().is_empty())
                || q.get("tag").is_some_and(|s| !s.is_empty())
                || q.get("starred").is_some_and(|s| s == "1")
            {
                let mut f = filter_of(q);
                f.limit = Some(1000);
                let hits = organize::search_tagged(
                    &lib,
                    q.get("q").map(String::as_str).unwrap_or(""),
                    &f,
                    q.get("tag").map(String::as_str).filter(|t| !t.is_empty()),
                    q.get("starred").is_some_and(|s| s == "1"),
                )
                .map_err(bad)?;
                (organize::items_from_hits(&lib, &hits), "results".into())
            } else {
                (
                    organize::items_from_docs(&lib, &lib.documents(None).map_err(bad)?),
                    "library".into(),
                )
            };
            let bytes = match fmt {
                "csv" => organize::to_csv(&items).into_bytes(),
                "json" => organize::to_json(&items).into_bytes(),
                "zip" => {
                    let out = tmp_path(&lib.home, "-export.zip.tmp");
                    let res = organize::export_zip(&items, &out);
                    let b = std::fs::read(&out);
                    let _ = std::fs::remove_file(&out);
                    res.map_err(bad)?;
                    b.map_err(|e| (500, e.to_string()))?
                }
                _ => organize::to_markdown(&items).into_bytes(),
            };
            let stem = q
                .get("name")
                .map(|n| slug(n.rsplit_once('.').map_or(n, |x| x.0), 60))
                .unwrap_or_else(|| format!("cartacolibri-{}", slug(&label, 40)));
            Ok(Download {
                name: format!("{stem}.{ext}"),
                ctype,
                bytes,
            })
        }
    }
}

/// The AI-engine address saved from the web UI (`<home>/settings.json`), if any.
pub fn saved_gateway(home: Option<&str>) -> Option<String> {
    let h = crate::library::state_dir(home).ok()?;
    let j: Value = serde_json::from_slice(&std::fs::read(h.join("settings.json")).ok()?).ok()?;
    let u = j["gateway_url"].as_str()?.trim();
    (!u.is_empty() && valid_gateway(u).is_ok()).then(|| u.to_string())
}

/// The address to use: the one saved in the UI wins over the command line / `COLI_URL` default.
pub fn effective_gateway(home: Option<&str>, default: &str) -> String {
    saved_gateway(home)
        .or_else(|| AUTO.lock().unwrap_or_else(|e| e.into_inner()).clone())
        .unwrap_or_else(|| default.to_string())
}

/// An engine found by itself on this machine (loopback only); lives only as long as the program runs
/// and never overrides an address the person saved.
static AUTO: Mutex<Option<String>> = Mutex::new(None);
static LAST_SCAN: Mutex<Option<std::time::Instant>> = Mutex::new(None);

/// Health of the engine at `current`, finding one by itself when nothing was configured.
pub fn engine_health(home: Option<&str>, current: &str) -> (bool, Value) {
    let b = carta::Backend::gateway(current, None);
    if b.alive() {
        let auto = AUTO.lock().unwrap_or_else(|e| e.into_inner()).as_deref() == Some(current);
        return (
            true,
            json!({"gateway": true, "auto": auto, "url": carta::redact_url(current), "model": b.models().into_iter().next()}),
        );
    }
    {
        let mut a = AUTO.lock().unwrap_or_else(|e| e.into_inner());
        if a.as_deref() == Some(current) {
            *a = None;
        }
    }
    if saved_gateway(home).is_none() {
        let due = {
            let mut l = LAST_SCAN.lock().unwrap_or_else(|e| e.into_inner());
            let due = l.is_none_or(|t| t.elapsed() > std::time::Duration::from_secs(10));
            if due {
                *l = Some(std::time::Instant::now());
            }
            due
        };
        if due && let Some(f) = carta::detect().into_iter().next() {
            *AUTO.lock().unwrap_or_else(|e| e.into_inner()) = Some(f.url.clone());
            return (
                true,
                json!({"gateway": true, "auto": true, "url": f.url, "model": f.models.first()}),
            );
        }
    }
    (false, json!({"gateway": false}))
}

/// http(s) only, a host, no credentials, no control characters. Loopback and LAN are allowed on purpose:
/// the AI engine is a program the person runs themselves.
pub fn valid_gateway(u: &str) -> Result<(), String> {
    if u.len() > 200
        || u.chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return Err("not a valid address".into());
    }
    let uri: ureq::http::Uri = u.parse().map_err(|_| "not a valid address".to_string())?;
    let ok_scheme = matches!(uri.scheme_str(), Some("http" | "https"));
    let auth = uri.authority().map(|a| a.as_str()).unwrap_or("");
    if !ok_scheme || auth.is_empty() || auth.contains('@') {
        return Err("use an address like http://127.0.0.1:8000".into());
    }
    Ok(())
}

fn write_settings(home: &Path, url: &str) -> Result<(), String> {
    let tmp = home.join(format!("settings.json.{}.tmp", std::process::id()));
    std::fs::write(
        &tmp,
        serde_json::to_vec_pretty(&json!({"gateway_url": url})).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, home.join("settings.json")).map_err(|e| e.to_string())
}

pub struct Req<'a> {
    pub method: &'a str,
    pub route: &'a str,
    pub query: HashMap<String, String>,
    pub body: &'a [u8],
    pub filename: Option<String>,
    pub backend_url: &'a str,
}

fn jbody(b: &[u8]) -> Result<Value, String> {
    if b.is_empty() {
        Ok(json!({}))
    } else {
        serde_json::from_slice(b).map_err(|e| format!("bad JSON: {e}"))
    }
}

fn s<'a>(j: &'a Value, k: &str) -> Result<&'a str, String> {
    j[k].as_str().ok_or_else(|| format!("missing '{k}'"))
}

fn strs(j: &Value, k: &str) -> Vec<String> {
    j[k].as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn csv(q: &HashMap<String, String>, k: &str) -> Vec<String> {
    q.get(k)
        .map(|v| {
            v.split(',')
                .filter(|x| !x.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

pub fn handle(home: Option<&str>, r: &Req) -> (u16, Value) {
    match inner(home, r) {
        Ok(v) => (200, v),
        Err((code, m)) => (code, json!({"error": m})),
    }
}

fn inner(home: Option<&str>, r: &Req) -> Result<Value, (u16, String)> {
    let bad = |m: String| (400u16, m);
    let mut lib = Library::open(home, false).map_err(|m| (500, m))?;
    let (m, route) = (r.method, r.route);
    Ok(match (m, route) {
        ("GET", "library") => {
            json!({"roots": lib.roots(), "documents": lib.documents(None).map_err(bad)?, "groups": lib.groups(),
                                     "problems": lib.problems(), "home": lib.home})
        }
        ("POST", "folders") => {
            let j = jbody(r.body).map_err(bad)?;
            let p = s(&j, "path").map_err(bad)?.to_string();
            lib.add_root(&p).map_err(bad)?;
            serde_json::to_value(index_now(&mut lib, Some(&[p])).map_err(bad)?).unwrap()
        }
        ("DELETE", "folders") => {
            let j = jbody(r.body).map_err(bad)?;
            json!({"dropped": lib.remove_root(s(&j, "path").map_err(bad)?).map_err(bad)?})
        }
        ("POST", "upload") => lib
            .add_upload(
                r.filename.as_deref().unwrap_or("document"),
                r.body,
                &csv(&r.query, "ocr"),
            )
            .map_err(bad)?,
        ("POST", "grab") => {
            let j = jbody(r.body).map_err(bad)?;
            let url = http_url(s(&j, "url").map_err(bad)?).map_err(bad)?;
            let url = url.as_str();
            if j["title"].as_str().is_some_and(|t| t.chars().count() > 500) {
                return Err((400, "that title is too long".into()));
            }
            if let Some(fmt) = j["fmt"].as_str() {
                if !matches!(fmt, "html" | "mhtml") {
                    return Err((400, "fmt must be html or mhtml".into()));
                }
                return grab::save_page(&mut lib, url, fmt).map_err(|m| (502, m));
            }
            match j["text"].as_str() {
                Some(t) => {
                    grab::save(&mut lib, url, j["title"].as_str().unwrap_or(""), t).map_err(bad)?
                }
                None => grab::grab(&mut lib, url).map_err(|m| (502, m))?,
            }
        }
        ("GET", "status") => {
            json!({"ocr": crate::extract::ocr_available(), "home": lib.home, "version": 2})
        }
        ("POST", "index") => {
            let j = jbody(r.body).map_err(bad)?;
            if j["background"].as_bool().unwrap_or(false) {
                // runs on its own thread; the page polls GET progress and keeps working
                let key = lib.home.to_string_lossy().into_owned();
                if begin(&key).is_err() {
                    return Ok(json!({"started": false, "running": true}));
                }
                let h = home.map(String::from);
                std::thread::spawn(move || match Library::open(h.as_deref(), false) {
                    Ok(mut l) => {
                        let _ = run_job(&mut l, &key, None);
                    }
                    Err(m) => finish(&key, &Err(m)),
                });
                json!({"started": true, "running": true})
            } else {
                serde_json::to_value(index_now(&mut lib, None).map_err(bad)?).unwrap()
            }
        }
        ("GET", "progress") => progress_json(&lib.home.to_string_lossy()),
        ("GET", "search") => {
            let f = Filter {
                ext: csv(&r.query, "ext"),
                min_size: r.query.get("min").and_then(|v| v.parse().ok()),
                max_size: r.query.get("max").and_then(|v| v.parse().ok()),
                path_like: r.query.get("path").cloned().filter(|p| !p.is_empty()),
                group: r.query.get("group").cloned().filter(|p| !p.is_empty()),
                limit: None,
                after: r
                    .query
                    .get("after")
                    .and_then(|v| crate::library::parse_date(v)),
                before: r
                    .query
                    .get("before")
                    .and_then(|v| crate::library::parse_date(v)),
                regex: r.query.get("re").cloned().filter(|p| !p.is_empty()),
            };
            serde_json::to_value(
                lib.search(r.query.get("q").map(String::as_str).unwrap_or(""), &f)
                    .map_err(bad)?,
            )
            .unwrap()
        }
        ("GET", "doc") => {
            let id: i64 = r
                .query
                .get("id")
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| bad("missing 'id'".into()))?;
            json!({"text": lib.text_of(id)})
        }
        ("POST", "groups") => {
            let j = jbody(r.body).map_err(bad)?;
            serde_json::to_value(
                lib.group_set(
                    s(&j, "name").map_err(bad)?,
                    &strs(&j, "members"),
                    j["description"].as_str(),
                    j["add"].as_bool().unwrap_or(false),
                )
                .map_err(bad)?,
            )
            .unwrap()
        }
        ("DELETE", "groups") => {
            let j = jbody(r.body).map_err(bad)?;
            lib.group_remove(s(&j, "name").map_err(bad)?, &strs(&j, "members"))
                .map_err(bad)?;
            json!({"ok": true})
        }
        ("POST", "run") => {
            let j = jbody(r.body).map_err(bad)?;
            let task = s(&j, "task").map_err(bad)?;
            let docs = match j["group"].as_str().filter(|g| !g.is_empty()) {
                Some(g) => lib.resolve_group(g).map_err(bad)?,
                None => lib.docs_by_id(
                    &j["ids"]
                        .as_array()
                        .map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
                        .unwrap_or_default(),
                ),
            };
            let be = if j["mock"].as_bool().unwrap_or(false) {
                carta::Backend::mock()
            } else {
                carta::Backend::gateway(r.backend_url, None)
            };
            let request = j["request"].as_str().unwrap_or("");
            let o = carta::Options {
                request,
                fields: strs(&j, "fields"),
                json: j["fmt"].as_str() == Some("json"),
                budget: carta::CTX_CHARS,
            };
            let lang = |k: &str| {
                j[k].as_str()
                    .map(str::trim)
                    .filter(|x| !x.is_empty() && x.chars().count() <= 40)
                    .map(String::from)
            };
            let x = carta::RunExtra {
                answer_lang: lang("lang"),
                target_lang: if task == "translate" {
                    lang("target")
                } else {
                    None
                },
                verify: j["verify"].as_bool().unwrap_or(false),
                ..Default::default()
            };
            let text = carta::run_ex(&lib, task, &docs, &be, &o, &x, &mut |_| {}).map_err(|m| {
                if m.starts_with("no answer from the Colibr") {
                    (502, m)
                } else {
                    (400, m)
                }
            })?;
            let saved = if j["save"].as_bool().unwrap_or(false) {
                Some(
                    carta::save_as(
                        &lib.home,
                        &text,
                        task,
                        j["group"].as_str().unwrap_or("selection"),
                        request,
                        match j["save_fmt"].as_str().unwrap_or("md") {
                            f @ ("md" | "html" | "txt") => f,
                            _ => return Err((400, "save as md, html or txt".into())),
                        },
                    )
                    .map_err(bad)?,
                )
            } else {
                None
            };
            json!({"text": text, "saved": saved, "usage": carta::take_usage().as_ref().map(carta::usage_json)})
        }
        ("GET", "tasks") => json!({"tasks": carta::TASKS}),
        ("GET", "find") => {
            let f = filter_of(&r.query);
            let qs = r.query.get("q").map(String::as_str).unwrap_or("");
            let tag = r
                .query
                .get("tag")
                .map(String::as_str)
                .filter(|t| !t.is_empty());
            let star = r.query.get("starred").is_some_and(|v| v == "1");
            if tag.is_some() || star {
                let hits = organize::search_tagged(&lib, qs, &f, tag, star).map_err(bad)?;
                let sg = if hits.is_empty() && !qs.trim().is_empty() {
                    lib.suggest(qs)
                } else {
                    vec![]
                };
                json!({"query": qs, "hits": hits, "corrected": null, "suggestions": sg})
            } else {
                // smart search: the engine adds synonyms (a plain OR, so exact matches still rank first)
                let smart = r.query.get("smart").is_some_and(|v| v == "1");
                let extra = if smart {
                    carta::expand_query(&carta::Backend::gateway(r.backend_url, None), qs)
                } else {
                    vec![]
                };
                let wide = if extra.is_empty() {
                    qs.to_string()
                } else {
                    format!("{qs} OR {}", extra.join(" OR "))
                };
                let fr = lib.search_friendly(&wide, &f).map_err(bad)?;
                let mut by_ext: std::collections::BTreeMap<&str, usize> = Default::default();
                for h in &fr.hits {
                    *by_ext.entry(h.ext.as_str()).or_default() += 1;
                }
                json!({"query": qs, "hits": fr.hits, "corrected": fr.corrected, "suggestions": fr.suggestions, "expanded": extra, "facets": {"ext": by_ext}})
            }
        }
        ("GET", "entities") => {
            let kind = r
                .query
                .get("kind")
                .map(String::as_str)
                .filter(|k| !k.is_empty());
            if kind.is_some_and(|k| !crate::insight::KINDS.contains(&k)) {
                return Err((400, "kind must be email, url, money, date or phone".into()));
            }
            let docs: Vec<(String, String)> = lib
                .documents(
                    r.query
                        .get("group")
                        .map(String::as_str)
                        .filter(|g| !g.is_empty()),
                )
                .map_err(bad)?
                .into_iter()
                .map(|d| (d.path, lib.text_of(d.id)))
                .collect();
            let idx = crate::insight::entity_index(&docs, kind);
            let mut rows: Vec<Value> = idx
                .into_iter()
                .map(|((k, v), p)| json!({"kind": k, "value": v, "count": p.len(), "docs": p}))
                .collect();
            rows.sort_by_key(|x| std::cmp::Reverse(x["count"].as_u64().unwrap_or(0)));
            rows.truncate(500);
            json!({"entities": rows})
        }
        ("GET", "compare") => {
            let get = |k: &str| -> Result<(String, String), (u16, String)> {
                let id: i64 = r
                    .query
                    .get(k)
                    .and_then(|v| v.parse().ok())
                    .ok_or((400, format!("missing document id '{k}'")))?;
                let d = lib
                    .docs_by_id(&[id])
                    .into_iter()
                    .next()
                    .ok_or((404, "that document is not in the library".to_string()))?;
                Ok((d.path, lib.text_of(id)))
            };
            let (pa, ta) = get("a")?;
            let (pb, tb) = get("b")?;
            json!({"a": pa, "b": pb, "compare": crate::insight::compare(&ta, &tb)})
        }
        ("GET", "duplicates") => {
            let by: HashMap<String, (i64, i64)> = lib
                .documents(None)
                .map_err(bad)?
                .into_iter()
                .map(|d| (d.path, (d.id, d.size)))
                .collect();
            let groups: Vec<Vec<Value>> = lib
                .duplicates()
                .into_iter()
                .map(|g| {
                    g.into_iter()
                        .map(|p| {
                            let (id, size) = by.get(&p).copied().unwrap_or((0, 0));
                            json!({"id": id, "path": p, "size": size})
                        })
                        .collect()
                })
                .collect();
            json!({"groups": groups})
        }
        ("GET", "meta") => {
            let meta: serde_json::Map<String, Value> = organize::all_meta(&lib)
                .into_iter()
                .map(|(p, m)| {
                    (
                        p,
                        json!({"tags": m.tags, "starred": m.starred, "has_note": !m.note.is_empty(), "opens": m.opens}),
                    )
                })
                .collect();
            json!({"meta": meta, "tags": organize::tag_list(&lib)})
        }
        ("GET", "tags") => json!({"tags": organize::tag_list(&lib)}),
        ("GET", "docmeta") => {
            let id: i64 = r
                .query
                .get("id")
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| bad("missing 'id'".into()))?;
            let d = lib
                .docs_by_id(&[id])
                .into_iter()
                .next()
                .ok_or((404, "that document is not in the library".to_string()))?;
            let m = organize::meta_of(&lib, &d.path);
            let rel: Vec<Value> = organize::related(&lib, id, 6)
                .unwrap_or_default()
                .into_iter()
                .map(|h| json!({"id": h.id, "path": h.path, "score": h.score}))
                .collect();
            json!({"id": id, "path": d.path, "tags": m.tags, "starred": m.starred, "note": m.note,
                "opens": m.opens, "related": rel, "all_tags": organize::tag_list(&lib)})
        }
        ("POST", "open") => {
            let d = doc_of(&lib, &jbody(r.body).map_err(bad)?)?;
            organize::record_open(&lib, &d.path).map_err(bad)?;
            json!({"ok": true})
        }
        ("POST", "tags") => {
            let j = jbody(r.body).map_err(bad)?;
            let d = doc_of(&lib, &j)?;
            let list = |k: &str| -> Result<Vec<String>, (u16, String)> {
                let v = strs(&j, k);
                if v.len() > 20 {
                    return Err((400, "too many tags at once".into()));
                }
                v.iter()
                    .map(|t| clean_text(t.trim_start_matches('#'), 40, "tag").map_err(bad))
                    .collect()
            };
            let (add, rm) = (list("add")?, list("remove")?);
            if !add.is_empty() {
                organize::tag_add(&lib, &d.path, &add).map_err(bad)?;
            }
            if !rm.is_empty() {
                organize::tag_remove(&lib, &d.path, &rm).map_err(bad)?;
            }
            json!({"tags": organize::tags_of(&lib, &d.path), "all_tags": organize::tag_list(&lib)})
        }
        ("POST", "star") => {
            let j = jbody(r.body).map_err(bad)?;
            let d = doc_of(&lib, &j)?;
            let on = j["on"].as_bool().unwrap_or(true);
            organize::set_star(&lib, &d.path, on).map_err(bad)?;
            json!({"starred": on})
        }
        ("POST", "note") => {
            let j = jbody(r.body).map_err(bad)?;
            let d = doc_of(&lib, &j)?;
            let n = j["note"].as_str().unwrap_or("");
            if n.chars().count() > 50_000 {
                return Err((400, "that note is too long".into()));
            }
            organize::set_note(&lib, &d.path, n).map_err(bad)?;
            json!({"ok": true})
        }
        ("GET", "recent") => {
            let n = r
                .query
                .get("n")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(8)
                .clamp(1, 50);
            let by: HashMap<String, i64> = lib
                .documents(None)
                .map_err(bad)?
                .into_iter()
                .map(|d| (d.path, d.id))
                .collect();
            let recent: Vec<Value> = organize::recent(&lib, n * 3)
                .into_iter()
                .filter_map(|p| by.get(&p).map(|id| json!({"id": id, "path": p})))
                .take(n)
                .collect();
            let most: Vec<Value> = organize::most_opened(&lib, n * 3)
                .into_iter()
                .filter_map(|(p, c)| {
                    by.get(&p)
                        .map(|id| json!({"id": id, "path": p, "opens": c}))
                })
                .take(n)
                .collect();
            json!({"recent": recent, "most": most})
        }
        ("GET", "engine") => {
            let found: Vec<Value> = carta::detect()
                .into_iter()
                .map(|f| json!({"url": f.url, "models": f.models}))
                .collect();
            json!({"found": found, "effective": carta::redact_url(r.backend_url)})
        }
        ("POST", "engine/start") => {
            if carta::detect().is_empty() {
                let n = carta::start_companion().map_err(|e| (404, e))?;
                json!({"started": n})
            } else {
                json!({"started": null, "already": true})
            }
        }
        ("GET", "settings") => json!({
            "gateway_url": saved_gateway(Some(&lib.home.to_string_lossy())).unwrap_or_default(),
            "effective": r.backend_url,
        }),
        ("POST", "settings") => {
            let j = jbody(r.body).map_err(bad)?;
            let u = j["gateway_url"].as_str().unwrap_or("").trim().to_string();
            if !u.is_empty() {
                valid_gateway(&u).map_err(bad)?;
            }
            write_settings(&lib.home, &u).map_err(|e| (500, e))?;
            json!({"ok": true, "gateway_url": u})
        }
        ("GET", "saved") => json!({"saved": organize::saved_list(&lib)}),
        ("POST", "saved") => {
            let j = jbody(r.body).map_err(bad)?;
            let opt = |k: &str, max: usize| -> Result<Option<String>, (u16, String)> {
                match j[k].as_str().map(str::trim).filter(|x| !x.is_empty()) {
                    Some(v) => clean_text(v, max, k).map(Some).map_err(bad),
                    None => Ok(None),
                }
            };
            let s = organize::SavedSearch {
                name: clean_text(s(&j, "name").map_err(bad)?, 80, "name").map_err(bad)?,
                query: j["query"]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .chars()
                    .take(500)
                    .collect(),
                ext: strs(&j, "ext").into_iter().take(30).collect(),
                min_size: j["min"].as_i64(),
                max_size: j["max"].as_i64(),
                path_like: opt("path", 300)?,
                group: opt("group", 120)?,
                tag: opt("tag", 40)?,
                starred_only: j["starred"].as_bool().unwrap_or(false),
                limit: None,
            };
            let name = s.name.clone();
            organize::saved_set(&lib, s).map_err(bad)?;
            json!({"ok": true, "name": name})
        }
        ("DELETE", "saved") => {
            let j = jbody(r.body).map_err(bad)?;
            json!({"removed": organize::saved_remove(&lib, s(&j, "name").map_err(bad)?).map_err(bad)?})
        }
        ("GET", "saved/run") => {
            let name = r.query.get("name").map(String::as_str).unwrap_or("");
            json!({"hits": organize::saved_run(&lib, name).map_err(|m| (404, m))?})
        }
        ("GET", "stats") => serde_json::to_value(organize::stats(&lib).map_err(bad)?).unwrap(),
        ("GET", "timeline") => {
            let months: Vec<Value> = organize::timeline(&lib)
                .map_err(bad)?
                .into_iter()
                .map(|m| json!({"month": m.month, "count": m.count}))
                .collect();
            json!({"months": months})
        }
        ("POST", "verify") => {
            let j = jbody(r.body).map_err(bad)?;
            serde_json::to_value(
                organize::verify(&mut lib, j["repair"].as_bool().unwrap_or(false)).map_err(bad)?,
            )
            .unwrap()
        }
        ("POST", "import") => {
            let name = safe_name(r.filename.as_deref().unwrap_or("")).map_err(bad)?;
            // the name becomes a real file in a staging folder: keep it valid on every OS
            if name.contains([':', '<', '>', '"', '|', '?', '*']) || name.ends_with('.') {
                return Err((400, "that file name is not allowed".into()));
            }
            let ext = name
                .rsplit_once('.')
                .map(|x| x.1.to_lowercase())
                .unwrap_or_default();
            if !matches!(
                ext.as_str(),
                "html" | "htm" | "bib" | "bibtex" | "enex" | "csv" | "zip"
            ) {
                return Err((
                    400,
                    "I can bring in bookmarks (.html), .bib, .enex, Pocket/Raindrop .csv and Obsidian .zip files".into(),
                ));
            }
            if r.body.is_empty() {
                return Err((400, "that file is empty".into()));
            }
            let stage = tmp_path(&lib.home, "-import");
            std::fs::create_dir_all(&stage).map_err(|e| (500, e.to_string()))?;
            let dest = lib.home.join("library").join("uploads").join("imported");
            let res = (|| -> Result<import::ImportStats, String> {
                let src = if ext == "zip" {
                    let v = stage.join("vault");
                    std::fs::create_dir_all(&v).map_err(|e| e.to_string())?;
                    unzip_safe(r.body, &v)?;
                    v
                } else {
                    let f = stage.join(&name);
                    std::fs::write(&f, r.body).map_err(|e| e.to_string())?;
                    f
                };
                import::import_path(&src, &dest)
            })();
            let _ = std::fs::remove_dir_all(&stage);
            let st = res.map_err(bad)?;
            let mut indexed = 0;
            if Path::new(&dest).is_dir() && st.notes > 0 {
                let d = dest.to_string_lossy().into_owned();
                lib.add_root(&d).map_err(bad)?;
                let s2 = index_now(&mut lib, Some(&[d])).map_err(bad)?;
                indexed = s2.added + s2.updated;
            }
            json!({"notes": st.notes, "skipped": st.skipped, "indexed": indexed})
        }
        ("GET", "feeds") => json!({"urls": feed_urls(&lib)}),
        ("POST", "feeds") => {
            let j = jbody(r.body).map_err(bad)?;
            let u = http_url(s(&j, "url").map_err(bad)?).map_err(bad)?;
            let mut v = feed_urls(&lib);
            if v.len() >= 200 {
                return Err((400, "too many subscriptions".into()));
            }
            if !v.contains(&u) {
                v.push(u);
            }
            set_feed_urls(&lib, &v).map_err(bad)?;
            json!({"urls": v})
        }
        ("DELETE", "feeds") => {
            let j = jbody(r.body).map_err(bad)?;
            let u = s(&j, "url").map_err(bad)?;
            let v: Vec<String> = feed_urls(&lib).into_iter().filter(|x| x != u).collect();
            set_feed_urls(&lib, &v).map_err(bad)?;
            json!({"urls": v})
        }
        ("POST", "feeds/opml") => {
            let t = String::from_utf8_lossy(r.body).into_owned();
            let mut v = feed_urls(&lib);
            let mut added = 0;
            for sub in feeds::opml_parse(&t) {
                if let Ok(u) = http_url(&sub.url)
                    && !v.contains(&u)
                    && v.len() < 200
                {
                    v.push(u);
                    added += 1;
                }
            }
            set_feed_urls(&lib, &v).map_err(bad)?;
            json!({"urls": v, "added": added})
        }
        ("POST", "feeds/poll") => {
            let urls = feed_urls(&lib);
            let home_dir = lib.home.clone();
            let sum = feeds::poll(&home_dir, &urls, &mut lib);
            json!({"feeds_ok": sum.feeds_ok, "feeds_failed": sum.feeds_failed, "new_items": sum.new_items,
                "feeds": sum.feeds.iter().map(|f| json!({"url": f.url, "items": f.items, "new": f.new, "error": f.error})).collect::<Vec<_>>()})
        }
        ("POST", "chat") => {
            let j = jbody(r.body).map_err(bad)?;
            let sid = valid_sid(s(&j, "sid").map_err(bad)?).map_err(bad)?;
            let q = s(&j, "q").map_err(bad)?.trim().to_string();
            if q.is_empty() || q.chars().count() > 4000 {
                return Err((400, "ask a question of up to 4000 characters".into()));
            }
            let group = match j["group"].as_str().filter(|g| !g.is_empty()) {
                Some(g) if lib.groups().contains_key(g) => Some(g.to_string()),
                Some(_) => return Err((400, "no such group".into())),
                None => None,
            };
            let be = if j["mock"].as_bool().unwrap_or(false) {
                carta::Backend::mock()
            } else {
                carta::Backend::gateway(r.backend_url, None)
            };
            let key = format!("{}\0{sid}", lib.home.display());
            // "query" mode (AnythingLLM-style): every question stands alone, nothing is remembered
            let single = j["mode"].as_str() == Some("query");
            let taken = if single {
                None
            } else {
                lock(chats()).remove(&key)
            };
            let mut cs = taken
                .map(|x| x.0)
                .unwrap_or_else(|| carta::ChatSession::new(be.clone()));
            cs.be = be;
            cs.group = group;
            cs.answer_lang = j["lang"]
                .as_str()
                .map(str::trim)
                .filter(|x| !x.is_empty() && x.chars().count() <= 40)
                .map(String::from);
            let reply = cs.ask(&lib, &q);
            if !single {
                let mut g = lock(chats());
                g.insert(key, (cs, TICK.fetch_add(1, Ordering::SeqCst)));
                while g.len() > MAX_CHATS {
                    if let Some(old) = g.iter().min_by_key(|(_, v)| v.1).map(|(k, _)| k.clone()) {
                        g.remove(&old);
                    } else {
                        break;
                    }
                }
            }
            let rep = reply.map_err(|m| {
                if m.starts_with("no answer from the Colibr") {
                    (502, m)
                } else {
                    (400, m)
                }
            })?;
            let gsrc: Vec<(String, String)> = rep
                .sources
                .iter()
                .map(|x| (x.label.clone(), x.passage.clone()))
                .collect();
            let grounding = crate::insight::grounding(&rep.answer, &gsrc);
            json!({"answer": rep.answer, "found": rep.found, "note": rep.note, "grounding": grounding, "usage": carta::take_usage().as_ref().map(carta::usage_json),
                "sources": rep.sources.iter().map(|x| json!({"label": x.label, "path": x.path, "passage": x.passage})).collect::<Vec<_>>()})
        }
        ("POST", "chat/reset") => {
            let j = jbody(r.body).map_err(bad)?;
            let sid = valid_sid(s(&j, "sid").map_err(bad)?).map_err(bad)?;
            lock(chats()).remove(&format!("{}\0{sid}", lib.home.display()));
            json!({"ok": true})
        }
        _ => return Err((404, "no such carta route".into())),
    })
}
