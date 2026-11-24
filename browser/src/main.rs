//! web-grab browser — a small Rust browser on the system webview (WebKitGTK / WebView2 / WKWebView).
//!
//! Beyond tabs, it is a browser that knows your library:
//! * 📚 a badge when the page is already saved, with a warning when it differs from the saved copy
//! * 🔗 related documents from your library while you browse (RAG-fetch on the page title)
//! * 💾 save page / selection (the address stays as the source; saving again refreshes the same file)
//! * ✨ ask your local Colibrì about the page — or 🗂✨ across ALL open tabs, with [S#] citations per tab
//! * 🗂 save every open tab as one library group; words in the address bar search your library
//! * the session (open tabs) is restored next time
//!
//! Clean-room, MIT. It reuses no Falkon (GPL-3.0) or other browser code. Nothing leaves your machine
//! except the page fetches you ask for; the bar talks to the app through a per-run nonce.

use cartacolibri::carta::{self, Backend, Options};
use cartacolibri::grab;
use cartacolibri::library::{Filter, Library, state_dir};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::{Window, WindowBuilder};
use wry::{WebView, WebViewBuilder};

const OVERLAY: &str = include_str!("overlay.js");
const START: &str = r#"<!doctype html><meta charset=utf-8><meta name=color-scheme content="dark light"><title>CartaColibri</title>
<style>body{margin:0;font:16px system-ui,sans-serif;background:#0f1115;color:#eef1f6}@media(prefers-color-scheme:light){body{background:#f6f7fa;color:#1b1f2a}}</style>
<body><noscript>CartaColibri</noscript></body>"#;

enum Ev {
    Eval(usize, String),
    Navigate(usize, String),
    NewTab(Option<String>),
    Close(usize),
    Reopen,
    Activate(usize),
    Cycle(bool),
    Title(usize, String),
    Url(usize, String),
    Push,
    Discard(Option<usize>),
    Dedupe,
}

#[derive(Clone, Default)]
struct Page {
    url: String,
    title: String,
    text: String,
}

struct Cfg {
    home: Option<String>,
    gateway: String,
    mock: bool,
    nonce: String,
    selftest: Option<&'static str>,
    pages: Mutex<BTreeMap<usize, Page>>,
    /// tab id -> colour label (1..=6) for tab groups
    colors: Mutex<BTreeMap<usize, u8>>,
    /// block navigations to well-known tracking hosts (off by default)
    trackers: AtomicBool,
    /// documents the user chose to open in the webview anyway
    allow: Mutex<BTreeSet<String>>,
}

struct Tab {
    id: usize,
    /// `None` = the tab is asleep (its webview was dropped to free memory)
    wv: Option<WebView>,
    title: String,
    url: String,
}

fn nonce() -> String {
    use sha1::{Digest, Sha1};
    let mut seed = format!("{:?}{}", std::time::SystemTime::now(), std::process::id());
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        use std::io::Read;
        let mut b = [0u8; 16];
        if f.read_exact(&mut b).is_ok() {
            seed.push_str(&hex::encode(b));
        }
    }
    hex::encode(Sha1::digest(seed.as_bytes()))
}

/// Small per-user switches kept next to the library (`browser.json`): tour seen, window size.
fn prefs_get(c: &Cfg) -> serde_json::Map<String, Value> {
    state_dir(c.home.as_deref())
        .ok()
        .and_then(|h| std::fs::read_to_string(h.join("browser.json")).ok())
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

fn prefs_set(c: &Cfg, kv: &[(&str, Value)]) {
    if let Ok(h) = state_dir(c.home.as_deref()) {
        let mut m = prefs_get(c);
        for (k, v) in kv {
            m.insert((*k).to_string(), v.clone());
        }
        let _ = std::fs::write(
            h.join("browser.json"),
            serde_json::to_string_pretty(&Value::Object(m)).unwrap_or_default(),
        );
    }
}

fn save_size(c: &Cfg, w: &Window) {
    if c.selftest.is_some() {
        return;
    }
    let s = w.inner_size().to_logical::<f64>(w.scale_factor());
    if s.width >= 400.0 && s.height >= 300.0 {
        prefs_set(
            c,
            &[
                ("w", json!(s.width.round())),
                ("h", json!(s.height.round())),
            ],
        );
    }
}

const TRACKER_HOSTS: &[&str] = &[
    "doubleclick.net",
    "googlesyndication.com",
    "google-analytics.com",
    "googletagmanager.com",
    "googleadservices.com",
    "connect.facebook.net",
    "facebook.net",
    "hotjar.com",
    "scorecardresearch.com",
    "adsrvr.org",
    "taboola.com",
    "outbrain.com",
    "criteo.com",
    "quantserve.com",
    "mixpanel.com",
    "segment.io",
    "amplitude.com",
    "adnxs.com",
    "bat.bing.com",
    "analytics.tiktok.com",
];
const DOC_EXT: &[&str] = &[
    "pdf", "doc", "docx", "odt", "ods", "odp", "epub", "rtf", "ppt", "pptx", "xls", "xlsx",
];

fn is_tracking_param(k: &str) -> bool {
    let k = k.to_ascii_lowercase();
    k.starts_with("utm_")
        || k.starts_with("oly_")
        || matches!(
            k.as_str(),
            "fbclid"
                | "gclid"
                | "dclid"
                | "msclkid"
                | "mc_eid"
                | "mc_cid"
                | "igshid"
                | "yclid"
                | "_hsenc"
                | "_hsmi"
                | "mkt_tok"
                | "ref_src"
                | "spm"
                | "vero_id"
        )
}

/// The address without tracking parameters (utm_*, fbclid, gclid, …); the rest is untouched.
fn clean_url(u: &str) -> String {
    let (base, frag) = match u.split_once('#') {
        Some((b, f)) => (b, Some(f)),
        None => (u, None),
    };
    let Some((path, q)) = base.split_once('?') else {
        return u.to_string();
    };
    let kept: Vec<&str> = q
        .split('&')
        .filter(|p| !p.is_empty() && !is_tracking_param(p.split('=').next().unwrap_or("")))
        .collect();
    let mut out = path.to_string();
    if !kept.is_empty() {
        out.push('?');
        out.push_str(&kept.join("&"));
    }
    if let Some(f) = frag {
        out.push('#');
        out.push_str(f);
    }
    out
}

fn host_of(u: &str) -> String {
    let rest = u.split_once("://").map_or(u, |x| x.1);
    let auth = rest.split(['/', '?', '#']).next().unwrap_or("");
    let auth = auth.rsplit('@').next().unwrap_or("");
    auth.split(':').next().unwrap_or("").to_ascii_lowercase()
}

fn is_tracker(host: &str) -> bool {
    TRACKER_HOSTS
        .iter()
        .any(|t| host == *t || host.ends_with(&format!(".{t}")))
}

fn doc_ext(u: &str) -> Option<String> {
    if !u.starts_with("http") {
        return None;
    }
    let path = u.split(['?', '#']).next().unwrap_or("");
    let last = path.rsplit('/').next().unwrap_or("");
    let (_, ext) = last.rsplit_once('.')?;
    let ext = ext.to_ascii_lowercase();
    DOC_EXT.contains(&ext.as_str()).then_some(ext)
}

fn file_name_of(u: &str) -> String {
    let path = u.split(['?', '#']).next().unwrap_or("");
    let last = path.rsplit('/').next().unwrap_or("");
    if last.is_empty() {
        "document".into()
    } else {
        last.to_string()
    }
}

/// Key for duplicate-tab detection: same page modulo fragment, tracking params and trailing slash.
fn dup_key(u: &str) -> String {
    let u = clean_url(u.split('#').next().unwrap_or(""));
    u.trim_end_matches('/').to_ascii_lowercase()
}

/// The read-later queue: a marker file `read-later.json` in the library home (url -> title/added).
fn later_load(h: &std::path::Path) -> BTreeMap<String, Value> {
    std::fs::read_to_string(h.join("read-later.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn later_store(h: &std::path::Path, m: &BTreeMap<String, Value>) {
    let _ = std::fs::write(
        h.join("read-later.json"),
        serde_json::to_string_pretty(m).unwrap_or_default(),
    );
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn lang_name(code: &str) -> &'static str {
    match code {
        "ro" => "Romanian",
        "fr" => "French",
        "it" => "Italian",
        "es" => "Spanish",
        "pt" => "Portuguese",
        "de" => "German",
        "hu" => "Hungarian",
        _ => "English",
    }
}

/// What the new-tab page shows: recently saved pages and watched pages (all local).
fn start_data(c: &Cfg) -> Value {
    let Ok(h) = state_dir(c.home.as_deref()) else {
        return json!({"recent": [], "watched": [], "later": []});
    };
    let web: BTreeMap<String, Value> = std::fs::read_to_string(h.join("web.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let mut v: Vec<(u64, &String, &Value)> = web
        .iter()
        .map(|(u, e)| (e["saved_at"].as_u64().unwrap_or(0), u, e))
        .collect();
    v.sort_by_key(|b| std::cmp::Reverse(b.0));
    let recent: Vec<Value> = v
        .into_iter()
        .take(6)
        .map(|(_, u, e)| {
            let path = e["path"].as_str().unwrap_or("");
            let title = std::fs::read_to_string(path)
                .ok()
                .and_then(|t| {
                    t.lines()
                        .next()
                        .map(|l| l.trim_start_matches('#').trim().to_string())
                })
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| u.clone());
            json!({"title": title, "url": u})
        })
        .collect();
    let watched: Vec<String> = grab::watch_list(&h).into_iter().take(6).collect();
    let mut later: Vec<(u64, String, String)> = later_load(&h)
        .into_iter()
        .map(|(u, e)| {
            (
                e["added"].as_u64().unwrap_or(0),
                e["title"].as_str().unwrap_or("").to_string(),
                u,
            )
        })
        .collect();
    later.sort_by_key(|b| std::cmp::Reverse(b.0));
    let later: Vec<Value> = later
        .into_iter()
        .take(8)
        .map(|(_, t, u)| json!({"title": t, "url": u}))
        .collect();
    json!({"recent": recent, "watched": watched, "later": later})
}

fn normalize(q: &str) -> Option<String> {
    let q = q.trim();
    if q.is_empty() || q.contains(' ') {
        return None;
    }
    if q.starts_with("http://") || q.starts_with("https://") {
        return Some(q.to_string());
    }
    if q.contains('.') || q.starts_with("localhost") {
        return Some(format!("https://{q}"));
    }
    None
}

fn reply(p: &EventLoopProxy<Ev>, c: &Cfg, tab: usize, ok: bool, text: String) {
    reply_k(p, c, tab, ok, text, "", json!(null));
}

/// Like [`reply`], with a `kind` (the bar shows a friendly, translated toast) and one extra value.
fn reply_k(
    p: &EventLoopProxy<Ev>,
    c: &Cfg,
    tab: usize,
    ok: bool,
    text: String,
    kind: &str,
    x: Value,
) {
    let msg = json!({"ok": ok, "text": text, "k": kind, "x": x});
    let js = format!(
        "window[\"__cc_{n}\"]&&window[\"__cc_{n}\"]({msg})",
        n = c.nonce
    );
    let _ = p.send_event(Ev::Eval(tab, js));
}

fn backend(c: &Cfg) -> Backend {
    if c.mock {
        Backend::mock()
    } else {
        Backend::gateway(&c.gateway, None)
    }
}

/// Save every open tab (that has readable text) and group them. Returns (group name, saved paths).
fn save_tabs(
    c: &Cfg,
    lib: &mut Library,
    color: Option<u8>,
) -> Result<(String, Vec<String>), String> {
    let colors = c.colors.lock().unwrap().clone();
    let pages: Vec<Page> = c
        .pages
        .lock()
        .unwrap()
        .iter()
        .filter(|(id, _)| color.is_none_or(|k| colors.get(id) == Some(&k)))
        .map(|(_, p)| p.clone())
        .collect();
    let mut paths = vec![];
    for p in pages.iter().filter(|p| p.url.starts_with("http")) {
        if let Ok(v) = grab::save(lib, &p.url, &p.title, &p.text)
            && let Some(path) = v["saved"].as_str()
        {
            paths.push(path.to_string());
        }
    }
    if paths.is_empty() {
        return Err("no open tab has readable text yet".into());
    }
    let secs = now_secs();
    let name = match color {
        Some(k) => format!("tabs-group{k}-{secs}"),
        None => format!("tabs-{secs}"),
    };
    lib.group_set(&name, &paths, Some("web-grab: open tabs"), false)?;
    Ok((name, paths))
}

fn related(lib: &Library, title: &str, skip: Option<&str>) -> Vec<Value> {
    let words: Vec<String> = title
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 4)
        .take(6)
        .map(String::from)
        .collect();
    if words.is_empty() {
        return vec![];
    }
    lib.search(
        &words.join(" OR "),
        &Filter {
            limit: Some(5),
            ..Default::default()
        },
    )
    .unwrap_or_default()
    .into_iter()
    .filter(|h| Some(h.path.as_str()) != skip)
    .take(4)
    .map(
        |h| json!({"path": h.path, "snippet": h.snippet.replace(['[', ']'], ""), "score": h.score}),
    )
    .collect()
}

fn color_of(m: &Value) -> Option<u8> {
    m["color"]
        .as_u64()
        .map(|k| k as u8)
        .filter(|k| (1..=6).contains(k))
}

fn download(c: &Cfg, url: &str) -> Result<String, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("only http(s) documents can be added".into());
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(180)))
        .user_agent("CartaColibri-browser/0.2 (+local; https://github.com/chiuta/CartaColibri)")
        .build()
        .into();
    let mut r = agent
        .get(url)
        .call()
        .map_err(|e| format!("cannot download {url}: {e}"))?;
    let mut raw = Vec::new();
    r.body_mut()
        .as_reader()
        .take(100 * 1024 * 1024)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.is_empty() {
        return Err("the document is empty".into());
    }
    let mut l = Library::open(c.home.as_deref(), false)?;
    let v = l.add_upload(&file_name_of(url), &raw, &[])?;
    Ok(v["saved"].as_str().unwrap_or("").to_string())
}

fn handle(c: &Cfg, p: &EventLoopProxy<Ev>, m: &Value) {
    let s = |k: &str| m[k].as_str().unwrap_or("").to_string();
    let tab = m["t"].as_u64().unwrap_or(0) as usize;
    let lib = || Library::open(c.home.as_deref(), false);
    match m["cmd"].as_str().unwrap_or("") {
        "newtab" => {
            let u = s("url");
            let _ = p.send_event(Ev::NewTab(
                (u.starts_with("http://") || u.starts_with("https://")).then_some(u),
            ));
        }
        "reopen" => {
            let _ = p.send_event(Ev::Reopen);
        }
        "tour_seen" => prefs_set(c, &[("tour_done", json!(true))]),
        "close" => {
            let _ = p.send_event(Ev::Close(m["id"].as_u64().unwrap_or(0) as usize));
        }
        "activate" => {
            let _ = p.send_event(Ev::Activate(m["id"].as_u64().unwrap_or(0) as usize));
        }
        "cycle" => {
            let _ = p.send_event(Ev::Cycle(m["back"].as_bool().unwrap_or(false)));
        }
        "ready" => {
            let tour = tab == 1 && c.selftest.is_none() && prefs_get(c).get("tour_done").is_none();
            if tour {
                prefs_set(c, &[("tour_done", json!(true))]);
            }
            let start = if m["start"].as_bool().unwrap_or(false) {
                start_data(c)
            } else {
                json!(null)
            };
            let js = format!(
                "window[\"__cc_boot_{n}\"]&&window[\"__cc_boot_{n}\"]({b})",
                n = c.nonce,
                b = json!({"tour": tour, "start": start, "mock": c.mock, "prefs": {
                    "rd_size": prefs_get(c).get("rd_size").cloned().unwrap_or(json!(19)),
                    "rd_theme": prefs_get(c).get("rd_theme").cloned().unwrap_or(json!("")),
                    "trackers": c.trackers.load(Ordering::Relaxed),
                }})
            );
            let _ = p.send_event(Ev::Eval(tab, js));
            let _ = p.send_event(Ev::Push);
        }
        "loaded" => {
            let page = Page {
                url: clean_url(&s("url")),
                title: s("title"),
                text: s("text"),
            };
            c.pages.lock().unwrap().insert(tab, page.clone());
            let _ = p.send_event(Ev::Url(tab, s("url")));
            let _ = p.send_event(Ev::Title(tab, page.title.clone()));
            if !page.url.starts_with("http") {
                return;
            }
            let home = state_dir(c.home.as_deref()).ok();
            let status = home
                .as_ref()
                .and_then(|h| grab::status(h, &page.url, &page.text));
            let skip = status
                .as_ref()
                .and_then(|s| s["path"].as_str().map(String::from));
            let history = home.as_ref().is_some_and(|h| grab::history_enabled(h));
            let mut rel = lib()
                .map(|l| related(&l, &page.title, skip.as_deref()))
                .unwrap_or_default();
            if let Some(h) = home.as_ref() {
                let own = grab::history_path(h, &page.url)
                    .to_string_lossy()
                    .into_owned();
                rel.retain(|r| r["path"].as_str() != Some(own.as_str()));
            }
            let hl = home
                .as_ref()
                .map(|h| grab::highlights(h, &page.url))
                .unwrap_or_default();
            let info =
                json!({"status": status, "related": rel, "highlights": hl, "history": history});
            let js = format!(
                "window[\"__cc_info_{n}\"]&&window[\"__cc_info_{n}\"]({info})",
                n = c.nonce
            );
            let _ = p.send_event(Ev::Eval(tab, js));
            if let (true, Ok(mut l)) = (history, lib()) {
                let _ = grab::history_add(&mut l, &page.url, &page.title, &page.text);
            }
        }
        "pref" => {
            let k = s("k");
            match k.as_str() {
                "rd_size" => {
                    if let Some(v) = m["v"].as_f64().filter(|v| (12.0..=40.0).contains(v)) {
                        prefs_set(c, &[("rd_size", json!(v))]);
                    }
                }
                "rd_theme" if ["dark", "light", "sepia"].contains(&s("v").as_str()) => {
                    prefs_set(c, &[("rd_theme", json!(s("v")))]);
                }
                _ => {}
            }
        }
        "tabcolor" => {
            let id = m["id"].as_u64().unwrap_or(0) as usize;
            match color_of(m) {
                Some(k) => c.colors.lock().unwrap().insert(id, k),
                None => c.colors.lock().unwrap().remove(&id),
            };
            let _ = p.send_event(Ev::Push);
        }
        "discard" => {
            let _ = p.send_event(Ev::Discard(m["id"].as_u64().map(|i| i as usize)));
        }
        "dedupe" => {
            let _ = p.send_event(Ev::Dedupe);
        }
        "trackers" => {
            let on = !c.trackers.load(Ordering::Relaxed);
            c.trackers.store(on, Ordering::Relaxed);
            prefs_set(c, &[("trackers", json!(on))]);
            reply_k(p, c, tab, true, String::new(), "trk", json!(on));
        }
        "open_raw" => {
            let u = s("url");
            if u.starts_with("http") {
                c.allow.lock().unwrap().insert(u.clone());
                let _ = p.send_event(Ev::Navigate(tab, u));
            }
        }
        "dl" => match download(c, &s("url")) {
            Ok(path) => reply_k(p, c, tab, true, path.clone(), "dlok", json!({"path": path})),
            Err(e) => reply(p, c, tab, false, e),
        },
        "later" => {
            let url = clean_url(&s("url"));
            let Ok(h) = state_dir(c.home.as_deref()) else {
                return;
            };
            let mut q = later_load(&h);
            if q.remove(&url).is_some() {
                later_store(&h, &q);
                reply_k(
                    p,
                    c,
                    tab,
                    true,
                    String::new(),
                    "later",
                    json!({"on": false}),
                );
                return;
            }
            let r = lib().and_then(|mut l| {
                grab::save_with(&mut l, &url, &s("title"), &s("text"), m["html"].as_str())
            });
            match r {
                Ok(v) => {
                    q.insert(
                        url,
                        json!({"title": s("title"), "added": now_secs(), "path": v["saved"]}),
                    );
                    later_store(&h, &q);
                    reply_k(p, c, tab, true, String::new(), "later", json!({"on": true}));
                }
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        "later_done" => {
            if let Ok(h) = state_dir(c.home.as_deref()) {
                let mut q = later_load(&h);
                q.remove(&s("url"));
                later_store(&h, &q);
            }
        }
        "note" => {
            let sel: String = s("sel").chars().take(8000).collect();
            if sel.trim().is_empty() {
                return;
            }
            let quote: String = sel.lines().map(|l| format!("> {l}\n")).collect();
            let title = s("title");
            let doc = format!(
                "# Note — {}\n\nSource: {}\n\n{quote}",
                if title.is_empty() { "web" } else { &title },
                clean_url(&s("url"))
            );
            match lib().and_then(|mut l| {
                l.add_upload(&format!("note-{}.md", now_secs()), doc.as_bytes(), &[])
            }) {
                Ok(v) => reply_k(
                    p,
                    c,
                    tab,
                    true,
                    String::new(),
                    "note",
                    json!({"path": v["saved"]}),
                ),
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        "sel" => {
            let sel: String = s("sel").chars().take(4000).collect();
            let to = lang_name(&s("lang"));
            let (sys, user) = if s("mode") == "translate" {
                (
                    format!(
                        "You are a precise translator. Translate the user's text into {to}. Output only the translation."
                    ),
                    sel,
                )
            } else {
                (
                    format!(
                        "Explain the user's passage in simple words, in {to}, in at most four short sentences. It comes from a web page titled: {}.",
                        s("title")
                    ),
                    sel,
                )
            };
            match backend(c).complete(&sys, &user) {
                Ok(t) => reply(p, c, tab, true, t),
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        "tsearch" => {
            let q = s("q");
            let hits = lib().and_then(|l| {
                l.search(
                    &q,
                    &Filter {
                        limit: Some(6),
                        ..Default::default()
                    },
                )
            });
            let x: Vec<Value> = hits
                .unwrap_or_default()
                .into_iter()
                .map(|h| json!({"path": h.path, "snippet": h.snippet.replace(['[', ']'], ""), "score": h.score}))
                .collect();
            reply_k(p, c, tab, true, String::new(), "ts", json!(x));
        }
        "go" if s("q").trim_start().starts_with('?') => {
            let q = s("q")
                .trim_start()
                .trim_start_matches('?')
                .trim()
                .to_string();
            let r = lib().and_then(|l| {
                let hits = l.search(
                    &q,
                    &Filter {
                        limit: Some(8),
                        ..Default::default()
                    },
                )?;
                let ids: Vec<i64> = hits.iter().map(|h| h.id).collect();
                let docs = l.docs_by_id(&ids);
                let o = Options {
                    request: &q,
                    ..Default::default()
                };
                carta::run(&l, "synthesize", &docs, &backend(c), &o, &mut |_| {})
            });
            match r {
                Ok(t) => reply(p, c, tab, true, t),
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        "go" => match normalize(&s("q")) {
            Some(u) => {
                let _ = p.send_event(Ev::Navigate(tab, u));
            }
            None => {
                let out = lib()
                    .and_then(|l| {
                        l.search(
                            &s("q"),
                            &Filter {
                                limit: Some(10),
                                ..Default::default()
                            },
                        )
                    })
                    .map(|hits| {
                        if hits.is_empty() {
                            "—".to_string()
                        } else {
                            hits.iter()
                                .map(|h| {
                                    format!(
                                        "{:>3}%  {}\n      {}",
                                        h.score,
                                        h.path,
                                        h.snippet.replace(['[', ']'], "")
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        }
                    });
                match out {
                    Ok(t) => reply_k(p, c, tab, true, t, "search", json!(null)),
                    Err(e) => reply(p, c, tab, false, e),
                }
            }
        },
        "save" => {
            match lib().and_then(|mut l| {
                grab::save_with(
                    &mut l,
                    &clean_url(&s("url")),
                    &s("title"),
                    &s("text"),
                    m["html"].as_str(),
                )
            }) {
                Ok(v) => reply_k(
                    p,
                    c,
                    tab,
                    true,
                    format!(
                        "✔ {} characters → {}",
                        v["chars"],
                        v["saved"].as_str().unwrap_or("")
                    ),
                    "save",
                    json!({"chars": v["chars"], "path": v["saved"]}),
                ),
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        "highlight" => {
            let url = clean_url(&s("url"));
            let r = lib().and_then(|mut l| {
                if grab::status(&l.home, &url, "").is_none() {
                    grab::save(&mut l, &url, &s("title"), &s("text"))?;
                }
                grab::highlight_add(&mut l, &url, &s("sel"))
            });
            match r {
                Ok(n) => reply_k(
                    p,
                    c,
                    tab,
                    true,
                    format!("🖍 {n} highlight(s) saved with this page"),
                    "hl",
                    json!(n),
                ),
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        "watch" => {
            let url = clean_url(&s("url"));
            if let Ok(h) = state_dir(c.home.as_deref()) {
                let mut l = grab::watch_list(&h);
                if !l.contains(&url) {
                    l.push(url.clone());
                }
                grab::watch_set(&h, l);
                reply_k(
                    p,
                    c,
                    tab,
                    true,
                    format!(
                        "👁 watching {url}\ncheck with: cartacolibri webwatch check   (or run, to keep checking)"
                    ),
                    "watch",
                    json!(url),
                );
            }
        }
        "history" => {
            if let Ok(h) = state_dir(c.home.as_deref()) {
                let on = !grab::history_enabled(&h);
                grab::set_history(&h, on);
                reply_k(
                    p,
                    c,
                    tab,
                    true,
                    format!(
                        "🕘 history {} — {}",
                        if on { "on" } else { "off" },
                        if on {
                            "pages you visit are kept locally and searchable from the address bar"
                        } else {
                            "nothing new is recorded"
                        }
                    ),
                    "hist",
                    json!(on),
                );
                let js = format!(
                    "window[\"__cc_info_{n}\"]&&window[\"__cc_info_{n}\"]({{\"history\":{on}}})",
                    n = c.nonce
                );
                let _ = p.send_event(Ev::Eval(tab, js));
            }
        }
        "save_tabs" => match lib().and_then(|mut l| save_tabs(c, &mut l, color_of(m))) {
            Ok((name, paths)) => reply_k(
                p,
                c,
                tab,
                true,
                format!(
                    "✔ {} tabs → group “{name}”\n{}",
                    paths.len(),
                    paths.join("\n")
                ),
                "tabs",
                json!(paths.len()),
            ),
            Err(e) => reply(p, c, tab, false, e),
        },
        "ask" => match carta::ask_page(&backend(c), &s("q"), &s("title"), &s("url"), &s("text")) {
            Ok(t) => reply(p, c, tab, true, t),
            Err(e) => reply(p, c, tab, false, e),
        },
        "ask_tabs" => {
            let r = lib().and_then(|mut l| {
                let (name, _) = save_tabs(c, &mut l, color_of(m))?;
                let docs = l.resolve_group(&name)?;
                let o = Options {
                    request: &s("q"),
                    ..Default::default()
                };
                let task = match m["task"].as_str() {
                    Some("summarize") => "summarize",
                    Some("extract") => "extract",
                    _ => "synthesize",
                };
                let o = Options {
                    fields: if task == "extract" {
                        s("q")
                            .split(',')
                            .map(|f| f.trim().to_string())
                            .filter(|f| !f.is_empty())
                            .collect()
                    } else {
                        vec![]
                    },
                    ..o
                };
                carta::run(&l, task, &docs, &backend(c), &o, &mut |_| {})
            });
            match r {
                Ok(t) => reply(p, c, tab, true, t),
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        _ => {}
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
const GTK: bool = true;
#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
const GTK: bool = false;

fn make_tab(
    window: &Window,
    cfg: &Arc<Cfg>,
    proxy: &EventLoopProxy<Ev>,
    id: usize,
    url: Option<&str>,
) -> WebView {
    let (c2, p2) = (cfg.clone(), proxy.clone());
    let (p3, p4, p5, p6) = (proxy.clone(), proxy.clone(), proxy.clone(), proxy.clone());
    let (c5, p7, c7) = (cfg.clone(), proxy.clone(), cfg.clone());
    let script = OVERLAY
        .replace("__NONCE__", &cfg.nonce)
        .replace("__TAB__", &id.to_string());
    let mut b = WebViewBuilder::new()
        .with_initialization_script(script)
        .with_new_window_req_handler(move |u, _| {
            let _ = p3.send_event(Ev::NewTab(Some(u)));
            wry::NewWindowResponse::Deny
        })
        .with_document_title_changed_handler(move |t| {
            let _ = p4.send_event(Ev::Title(id, t));
        })
        .with_navigation_handler(move |u| {
            if u.starts_with("http") {
                let host = host_of(&u);
                if c5.trackers.load(Ordering::Relaxed) && is_tracker(&host) {
                    reply_k(&p5, &c5, id, true, String::new(), "blk", json!(host));
                    return false;
                }
                if doc_ext(&u).is_some() && !c5.allow.lock().unwrap().contains(&u) {
                    // a document: offer to add it to the library instead of a blank/failed view
                    reply_k(
                        &p5,
                        &c5,
                        id,
                        true,
                        String::new(),
                        "dl",
                        json!({"url": u, "open": true}),
                    );
                    return false;
                }
            }
            let _ = p5.send_event(Ev::Url(id, u));
            true
        })
        .with_download_started_handler(move |u, _dest| {
            if !u.starts_with("http") || c7.allow.lock().unwrap().contains(&u) {
                return true;
            }
            reply_k(
                &p7,
                &c7,
                id,
                true,
                String::new(),
                "dl",
                json!({"url": u, "open": false}),
            );
            false
        })
        .with_ipc_handler(move |req| {
            let Ok(m) = serde_json::from_str::<Value>(req.body()) else {
                return;
            };
            if m["n"].as_str() != Some(c2.nonce.as_str()) {
                return; // a page script cannot know the per-run nonce
            }
            let (c, p) = (c2.clone(), p2.clone());
            std::thread::spawn(move || handle(&c, &p, &m));
        });
    b = match url {
        Some(u) => b.with_url(normalize(u).unwrap_or_else(|| u.to_string())),
        None => b.with_html(START),
    };
    if let (Some(js), 1) = (cfg.selftest, id) {
        let nonce = cfg.nonce.clone();
        b = b.with_on_page_load_handler(move |ev, _| {
            if matches!(ev, wry::PageLoadEvent::Finished) {
                let _ = p6.send_event(Ev::Eval(
                    id,
                    format!(
                        "setTimeout(()=>window[\"__cc_do_{nonce}\"]{js},{})",
                        if js.contains("ask_tabs") { 3500 } else { 900 }
                    ),
                ));
            }
        });
    }
    if GTK {
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::WindowExtUnix;
            use wry::WebViewBuilderExtUnix;
            return b
                .build_gtk(window.default_vbox().expect("gtk vbox"))
                .expect("webview");
        }
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    {
        return b
            .with_bounds(full_bounds(window))
            .build_as_child(window)
            .expect("webview");
    }
    #[allow(unreachable_code)]
    {
        unreachable!()
    }
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
fn full_bounds(window: &Window) -> wry::Rect {
    let size = window.inner_size().to_logical::<f64>(window.scale_factor());
    wry::Rect {
        position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
        size: wry::dpi::LogicalSize::new(size.width, size.height).into(),
    }
}

fn push_tabs(cfg: &Cfg, tabs: &[Tab], active: usize) {
    let colors = cfg.colors.lock().unwrap().clone();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let list: Vec<Value> = tabs
        .iter()
        .map(|t| {
            let dup = t.url.starts_with("http")
                && seen.insert(dup_key(&t.url), t.id).is_some();
            json!({"id": t.id, "title": t.title, "url": t.url, "color": colors.get(&t.id).copied().unwrap_or(0),
                   "sleep": t.wv.is_none(), "dup": dup})
        })
        .collect();
    let st = json!({"tabs": list, "active": active});
    let js = format!(
        "window[\"__cc_tabs_{n}\"]&&window[\"__cc_tabs_{n}\"]({st})",
        n = cfg.nonce
    );
    for t in tabs {
        if let Some(wv) = &t.wv {
            let _ = wv.evaluate_script(&js);
        }
    }
}

fn save_session(cfg: &Cfg, tabs: &[Tab]) {
    if let Ok(h) = state_dir(cfg.home.as_deref()) {
        let urls: Vec<&str> = tabs
            .iter()
            .map(|t| t.url.as_str())
            .filter(|u| u.starts_with("http"))
            .collect();
        let _ = std::fs::write(
            h.join("tabs.json"),
            serde_json::to_string(&urls).unwrap_or_default(),
        );
    }
}

/// Wake a sleeping tab: rebuild its webview at the address it had.
fn wake(window: &Window, cfg: &Arc<Cfg>, proxy: &EventLoopProxy<Ev>, tabs: &mut [Tab], id: usize) {
    if let Some(t) = tabs.iter_mut().find(|t| t.id == id)
        && t.wv.is_none()
    {
        let u = (!t.url.is_empty()).then(|| t.url.clone());
        t.wv = Some(make_tab(window, cfg, proxy, id, u.as_deref()));
    }
}

fn show_only(tabs: &[Tab], active: usize) {
    for t in tabs {
        if let Some(wv) = &t.wv {
            let _ = wv.set_visible(t.id == active);
        }
    }
    if let Some(wv) = tabs
        .iter()
        .find(|t| t.id == active)
        .and_then(|t| t.wv.as_ref())
    {
        let _ = wv.focus();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |f: &str| args.iter().any(|a| a == f);
    let val = |f: &str| {
        args.iter()
            .position(|a| a == f)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    if flag("--help") || flag("-h") {
        println!(
            "cartacolibri-browser [URL…] [--home DIR] [--gateway URL] [--mock] [--fresh]\n  --fresh   do not restore the previous tabs\n  keys: Ctrl+S save · Ctrl+Shift+A ask · Ctrl+H highlight · Ctrl+F find · Ctrl+Shift+R reader · Ctrl+Shift+K search tabs · F1 help · Ctrl+Shift+T reopen tab · Esc close panels\n  --selftest-save | --selftest-ask | --selftest-tabs   load the first URL, save it / ask about it (use with --mock), print the result, exit (used by CI)"
        );
        return;
    }
    let selftest: Option<&'static str> = if flag("--selftest-tabs") {
        Some("('ask_tabs','colibri')")
    } else if flag("--selftest-ask") {
        Some("('ask','ce sunt colibrii?')")
    } else if flag("--selftest-save") {
        Some("('save')")
    } else {
        None
    };
    let (hv, gv) = (val("--home"), val("--gateway"));
    let urls: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with("--") && Some(*a) != hv.as_ref() && Some(*a) != gv.as_ref())
        .cloned()
        .collect();
    let cfg = Arc::new(Cfg {
        home: hv,
        gateway: gv
            .or_else(|| std::env::var("COLI_URL").ok())
            .unwrap_or_else(|| carta::DEFAULT_URL.into()),
        mock: flag("--mock"),
        nonce: nonce(),
        selftest,
        pages: Mutex::new(BTreeMap::new()),
        colors: Mutex::new(BTreeMap::new()),
        trackers: AtomicBool::new(false),
        allow: Mutex::new(BTreeSet::new()),
    });
    if selftest.is_none() {
        let on = prefs_get(&cfg)
            .get("trackers")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        cfg.trackers.store(on, Ordering::Relaxed);
    }
    let mut start_urls = urls;
    if start_urls.is_empty()
        && !flag("--fresh")
        && selftest.is_none()
        && let Ok(h) = state_dir(cfg.home.as_deref())
        && let Some(v) = std::fs::read_to_string(h.join("tabs.json"))
            .ok()
            .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
    {
        start_urls = v;
    }

    let event_loop = EventLoopBuilder::<Ev>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let size = prefs_get(&cfg);
    let dim = |k: &str, d: f64, hi: f64| {
        size.get(k)
            .and_then(Value::as_f64)
            .filter(|v| (400.0..=hi).contains(v))
            .unwrap_or(d)
    };
    let window = WindowBuilder::new()
        .with_title("CartaColibri · web-grab")
        .with_inner_size(tao::dpi::LogicalSize::new(
            dim("w", 1280.0, 10000.0),
            dim("h", 860.0, 10000.0),
        ))
        .build(&event_loop)
        .expect("window");

    let mut tabs: Vec<Tab> = vec![];
    let mut next_id = 1usize;
    let mut active = 1usize;
    let first: Vec<Option<String>> = if start_urls.is_empty() {
        vec![None]
    } else {
        start_urls.into_iter().map(Some).collect()
    };
    for u in first {
        let id = next_id;
        next_id += 1;
        let wv = make_tab(&window, &cfg, &proxy, id, u.as_deref());
        tabs.push(Tab {
            id,
            wv: Some(wv),
            title: String::new(),
            url: u.unwrap_or_default(),
        });
        active = id;
    }
    show_only(&tabs, active);
    let mut closed: Vec<String> = vec![];

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    event_loop.run(move |event, _, flow| {
        *flow = if selftest.is_some() {
            ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(250),
            )
        } else {
            ControlFlow::Wait
        };
        match event {
            Event::UserEvent(Ev::Eval(tab, js)) => {
                if selftest.is_some() && js.contains("\"text\":") {
                    let ok = js.contains("\"ok\":true") && (tabs.len() < 2 || js.contains("[S2]"));
                    println!(
                        "selftest {}: {}",
                        if ok { "ok" } else { "FAILED" },
                        js.split("(").last().unwrap_or("")
                    );
                    *flow = ControlFlow::ExitWithCode(if ok { 0 } else { 1 });
                    return;
                }
                if let Some(wv) = tabs
                    .iter()
                    .find(|t| t.id == tab)
                    .and_then(|t| t.wv.as_ref())
                {
                    let _ = wv.evaluate_script(&js);
                }
            }
            Event::UserEvent(Ev::Navigate(tab, u)) => {
                if let Some(wv) = tabs
                    .iter()
                    .find(|t| t.id == tab)
                    .and_then(|t| t.wv.as_ref())
                {
                    let _ = wv.load_url(&u);
                }
            }
            Event::UserEvent(Ev::Push) => push_tabs(&cfg, &tabs, active),
            Event::UserEvent(Ev::Discard(which)) => {
                let mut n = 0;
                for t in tabs.iter_mut() {
                    let pick = which.is_none_or(|w| w == t.id);
                    if pick && t.id != active && t.wv.is_some() && t.url.starts_with("http") {
                        t.wv = None;
                        n += 1;
                    }
                }
                push_tabs(&cfg, &tabs, active);
                reply_k(&proxy, &cfg, active, true, String::new(), "slept", json!(n));
            }
            Event::UserEvent(Ev::Dedupe) => {
                let mut seen: BTreeMap<String, usize> = BTreeMap::new();
                // keep the active tab of a group of duplicates, else the first
                let order: Vec<(usize, String)> = tabs
                    .iter()
                    .filter(|t| t.url.starts_with("http"))
                    .map(|t| (t.id, dup_key(&t.url)))
                    .collect();
                for (id, k) in &order {
                    if *id == active {
                        seen.insert(k.clone(), *id);
                    }
                }
                let mut drop_ids = vec![];
                for (id, k) in &order {
                    match seen.get(k) {
                        Some(keep) if keep != id => drop_ids.push(*id),
                        Some(_) => {}
                        None => {
                            seen.insert(k.clone(), *id);
                        }
                    }
                }
                tabs.retain(|t| !drop_ids.contains(&t.id));
                for id in &drop_ids {
                    cfg.pages.lock().unwrap().remove(id);
                    cfg.colors.lock().unwrap().remove(id);
                }
                show_only(&tabs, active);
                push_tabs(&cfg, &tabs, active);
                save_session(&cfg, &tabs);
                reply_k(
                    &proxy,
                    &cfg,
                    active,
                    true,
                    String::new(),
                    "dedup",
                    json!(drop_ids.len()),
                );
            }
            Event::UserEvent(Ev::NewTab(u)) => {
                let id = next_id;
                next_id += 1;
                let wv = make_tab(&window, &cfg, &proxy, id, u.as_deref());
                tabs.push(Tab {
                    id,
                    wv: Some(wv),
                    title: String::new(),
                    url: u.unwrap_or_default(),
                });
                active = id;
                show_only(&tabs, active);
                push_tabs(&cfg, &tabs, active);
                save_session(&cfg, &tabs);
            }
            Event::UserEvent(Ev::Close(id)) => {
                if tabs.len() > 1 {
                    if let Some(i) = tabs.iter().position(|t| t.id == id) {
                        let gone = tabs.remove(i);
                        if gone.url.starts_with("http") {
                            closed.push(gone.url);
                            if closed.len() > 20 {
                                closed.remove(0);
                            }
                        }
                        cfg.pages.lock().unwrap().remove(&id);
                        cfg.colors.lock().unwrap().remove(&id);
                        if active == id {
                            active = tabs[i.min(tabs.len() - 1)].id;
                        }
                        wake(&window, &cfg, &proxy, &mut tabs, active);
                        show_only(&tabs, active);
                        push_tabs(&cfg, &tabs, active);
                        save_session(&cfg, &tabs);
                    }
                } else {
                    save_size(&cfg, &window);
                    *flow = ControlFlow::Exit;
                }
            }
            Event::UserEvent(Ev::Reopen) => {
                if let Some(u) = closed.pop() {
                    let _ = proxy.send_event(Ev::NewTab(Some(u)));
                }
            }
            Event::UserEvent(Ev::Activate(id)) => {
                if tabs.iter().any(|t| t.id == id) {
                    active = id;
                    wake(&window, &cfg, &proxy, &mut tabs, active);
                    show_only(&tabs, active);
                    push_tabs(&cfg, &tabs, active);
                }
            }
            Event::UserEvent(Ev::Cycle(back)) => {
                if let Some(i) = tabs.iter().position(|t| t.id == active) {
                    let n = tabs.len();
                    active = tabs[if back { (i + n - 1) % n } else { (i + 1) % n }].id;
                    wake(&window, &cfg, &proxy, &mut tabs, active);
                    show_only(&tabs, active);
                    push_tabs(&cfg, &tabs, active);
                }
            }
            Event::UserEvent(Ev::Title(id, title)) => {
                if let Some(t) = tabs.iter_mut().find(|t| t.id == id)
                    && t.title != title
                {
                    t.title = title;
                    push_tabs(&cfg, &tabs, active);
                }
            }
            Event::UserEvent(Ev::Url(id, url)) => {
                if let Some(t) = tabs.iter_mut().find(|t| t.id == id)
                    && t.url != url
                {
                    t.url = url;
                    push_tabs(&cfg, &tabs, active);
                    save_session(&cfg, &tabs);
                }
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. })
                if selftest.is_some() && std::time::Instant::now() > deadline =>
            {
                eprintln!("selftest FAILED: timeout");
                *flow = ControlFlow::ExitWithCode(2);
            }
            #[cfg(not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )))]
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                let b = full_bounds(&window);
                tabs.iter().for_each(|t| {
                    let _ = t.wv.set_bounds(b);
                });
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                save_size(&cfg, &window);
                *flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
