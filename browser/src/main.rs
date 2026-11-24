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
//! except the page fetches you ask for. The bar and the app authenticate each other with per-webview
//! secrets that never touch page-reachable state (see the header of `overlay.js` for the threat model).

use cartacolibri::carta::{self, Backend, Options};
use cartacolibri::grab;
use cartacolibri::library::{Filter, Library, state_dir};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::{Window, WindowBuilder};
use wry::{WebView, WebViewBuilder};

const OVERLAY: &str = include_str!("overlay.js");
const START: &str = r#"<!doctype html><meta charset=utf-8><meta name=color-scheme content="dark light"><title>CartaColibri</title>
<style>body{margin:0;font:16px system-ui,sans-serif;background:#0f1115;color:#eef1f6}@media(prefers-color-scheme:light){body{background:#f6f7fa;color:#1b1f2a}}</style>
<body><noscript>CartaColibri</noscript></body>"#;

enum Ev {
    /// call the overlay's single authenticated entry point: (tab, kind, argument)
    Call(usize, &'static str, Value),
    /// the forge self-test's probe result (what the hostile hooks captured)
    Probe(String),
    /// the forge self-test page has finished loading
    ForgeGo,
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
    /// print this tab to a PDF file (tab, target file in the library's web-grab folder)
    Pdf(usize, std::path::PathBuf),
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
    selftest: Option<&'static str>,
    /// per-webview secrets, document epoch, sequence numbers and rate limits (keyed by tab id)
    auth: Mutex<BTreeMap<usize, Auth>>,
    /// documents the browser itself offered to add to the library (only those may be downloaded)
    offered: Mutex<BTreeSet<String>>,
    pages: Mutex<BTreeMap<usize, Page>>,
    /// tab id -> colour label (1..=6) for tab groups
    colors: Mutex<BTreeMap<usize, u8>>,
    /// block navigations to well-known tracking hosts (off by default)
    trackers: AtomicBool,
    /// documents the user chose to open in the webview anyway
    allow: Mutex<BTreeSet<String>>,
    /// recent page-opened tabs (popup spam limiter)
    popups: Mutex<VecDeque<Instant>>,
}

/// Cap on remembered offered/allowed document URLs (a page can make us "offer" unboundedly many).
const MAX_OFFERED: usize = 256;

fn remember(set: &Mutex<BTreeSet<String>>, u: &str) {
    let mut g = set.lock().unwrap_or_else(|e| e.into_inner());
    if g.len() >= MAX_OFFERED {
        g.clear();
    }
    g.insert(u.to_string());
}

struct Tab {
    id: usize,
    /// `None` = the tab is asleep (its webview was dropped to free memory)
    wv: Option<WebView>,
    title: String,
    url: String,
}

/// `n` random bytes from the operating system's CSPRNG, hex-encoded. Aborts if the OS has none.
fn rand_hex(n: usize) -> String {
    let mut b = vec![0u8; n];
    getrandom::fill(&mut b).expect("the operating system has no random number generator");
    hex::encode(b)
}

/// Longest URL the browser will navigate to, keep in a tab or restore.
const MAX_URL: usize = 16 * 1024;
/// Most tabs open at once (popup / `newtab` flood protection).
const MAX_TABS: usize = 64;
/// Most page-opened tabs (`window.open`, target=_blank) per `POPUP_WIN`.
const POPUP_MAX: usize = 3;
const POPUP_WIN: Duration = Duration::from_secs(10);
/// Most IPC handler threads running at once.
const MAX_INFLIGHT: usize = 16;
static INFLIGHT: AtomicUsize = AtomicUsize::new(0);

/// Decrements [`INFLIGHT`] when a handler thread ends (also on panic).
struct Inflight;
impl Drop for Inflight {
    fn drop(&mut self) {
        INFLIGHT.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Lower-cased URL scheme (`[a-z][a-z0-9+.-]*` before the first `:`), if there is one.
fn scheme_of(u: &str) -> Option<String> {
    let (s, _) = u.split_once(':')?;
    let mut ch = s.chars();
    let ok = ch.next().is_some_and(|c| c.is_ascii_alphabetic())
        && ch.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    ok.then(|| s.to_ascii_lowercase())
}

/// A real `http(s)://` URL (case-insensitive scheme; `httpfoo:` and `http:x` do not count).
fn is_http(u: &str) -> bool {
    matches!(scheme_of(u).as_deref(), Some("http" | "https")) && u.contains("://")
}

/// May the webview navigate to `u`? Web pages and the browser's own start page only: never
/// `file:`, `javascript:`, `ftp:`, app-specific schemes or absurdly long addresses.
fn nav_allowed(u: &str) -> bool {
    if u.len() > MAX_URL {
        return false;
    }
    match scheme_of(u).as_deref() {
        Some("http" | "https") => u.contains("://"),
        Some("about") => matches!(
            u.to_ascii_lowercase().as_str(),
            "about:blank" | "about:srcdoc"
        ),
        Some("data") => u.to_ascii_lowercase().starts_with("data:text/html"),
        Some("blob") => true,
        _ => false,
    }
}

/// Sliding-window limiter: true (and recorded) while fewer than `max` events happened in `win`.
fn throttle(q: &mut VecDeque<Instant>, now: Instant, max: usize, win: Duration) -> bool {
    while q.front().is_some_and(|t| now.duration_since(*t) > win) {
        q.pop_front();
    }
    if q.len() >= max {
        return false;
    }
    q.push_back(now);
    true
}

/// First `n` characters of `s`.
fn trunc(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// What another tab's page may be told about this URL: no userinfo, query or fragment, short path
/// (those parts routinely carry tokens). The tab's own page already knows its own address.
fn redact_url(u: &str) -> String {
    let Some((scheme, rest)) = u.split_once("://") else {
        return trunc(u, 40);
    };
    let end = rest.find(['?', '#']).unwrap_or(rest.len());
    let rest = &rest[..end];
    let (auth, path) = match rest.find('/') {
        Some(i) => rest.split_at(i),
        None => (rest, ""),
    };
    let host = auth.rsplit('@').next().unwrap_or("");
    format!("{scheme}://{host}{}", trunc(path, 60))
}

/// Longest IPC message we even look at (a saved page is at most ~3 M chars of html + its text).
const MAX_MSG: usize = 24 * 1024 * 1024;
/// Per tab: at most this many messages (valid or not) per second ...
const RATE_ALL: usize = 60;
/// ... and at most this many bytes per second (limits the cost of a flood of forgeries).
const RATE_BYTES: usize = 64 * 1024 * 1024;
/// Commands that start real work (disk, network, model): at most this many per 10 s per tab.
const RATE_HEAVY: usize = 12;
const HEAVY: &[&str] = &[
    "save",
    "later",
    "note",
    "highlight",
    "ask",
    "ask_tabs",
    "save_tabs",
    "sel",
    "go",
    "dl",
    "grab",
    "pdf",
];
/// Commands that may only come from a real http(s) page (they carry page content).
const WEB_ONLY: &[&str] = &[
    "save",
    "later",
    "later_done",
    "note",
    "highlight",
    "watch",
    "ask",
    "sel",
    "dl",
    "open_raw",
    "related",
    "grab",
    "pdf",
];
const KNOWN: &[&str] = &[
    "newtab",
    "reopen",
    "tour_seen",
    "close",
    "activate",
    "cycle",
    "tabcolor",
    "discard",
    "dedupe",
    "trackers",
    "open_raw",
    "dl",
    "later",
    "later_done",
    "note",
    "sel",
    "tsearch",
    "go",
    "save",
    "highlight",
    "watch",
    "history",
    "save_tabs",
    "ask",
    "ask_tabs",
    "grab",
    "pdf",
    "ready",
    "loaded",
    "pref",
    "related",
];

/// Everything we know about one webview's channel with its overlay.
struct Auth {
    /// secret the overlay puts in every message
    key_out: String,
    /// secret Rust puts in every call to the overlay's entry point
    key_in: String,
    /// document epoch of the overlay instance we are talking to, and the last accepted sequence
    doc: String,
    last: u64,
    retired: VecDeque<String>,
    /// last sequence number we sent to the overlay
    out_seq: u64,
    win: Instant,
    n: usize,
    bytes: usize,
    heavy: VecDeque<Instant>,
}

impl Auth {
    fn new() -> Auth {
        Auth {
            key_out: rand_hex(32),
            key_in: rand_hex(32),
            doc: String::new(),
            last: 0,
            retired: VecDeque::new(),
            out_seq: 0,
            win: Instant::now(),
            n: 0,
            bytes: 0,
            heavy: VecDeque::new(),
        }
    }

    /// Rate limit applied to every message, before any cryptographic work.
    fn allow(&mut self, now: Instant, len: usize) -> bool {
        if now.duration_since(self.win) >= Duration::from_secs(1) {
            (self.win, self.n, self.bytes) = (now, 0, 0);
        }
        self.n += 1;
        self.bytes = self.bytes.saturating_add(len);
        self.n <= RATE_ALL && self.bytes <= RATE_BYTES
    }

    fn allow_heavy(&mut self, now: Instant) -> bool {
        while self
            .heavy
            .front()
            .is_some_and(|t| now.duration_since(*t) > Duration::from_secs(10))
        {
            self.heavy.pop_front();
        }
        if self.heavy.len() >= RATE_HEAVY {
            return false;
        }
        self.heavy.push_back(now);
        true
    }

    /// Accept (doc, seq) only if it is strictly newer than everything seen from this document, or
    /// the very first message (seq 1) of a document we have not retired yet (a reload).
    fn accept(&mut self, doc: &str, seq: u64) -> bool {
        if doc == self.doc {
            if seq > self.last {
                self.last = seq;
                return true;
            }
            return false;
        }
        if seq == 1 && !self.retired.iter().any(|d| d == doc) {
            if !self.doc.is_empty() {
                self.retired.push_back(std::mem::take(&mut self.doc));
                if self.retired.len() > 64 {
                    self.retired.pop_front();
                }
            }
            self.doc = doc.to_string();
            self.last = 1;
            return true;
        }
        false
    }

    /// Authenticate one raw IPC body (`cc1.<tab>.<doc>.<seq>.<secret>.<json>`) for tab `id`.
    /// Returns the parsed, size-limited object with `t` forced to `id`, or why it was dropped.
    fn check(&mut self, id: usize, body: &str, now: Instant) -> Result<Value, &'static str> {
        if body.len() > MAX_MSG {
            return Err("oversize");
        }
        if !self.allow(now, body.len()) {
            return Err("rate");
        }
        let mut it = body.splitn(6, '.');
        let (Some("cc1"), Some(t), Some(doc), Some(seq), Some(key), Some(json)) = (
            it.next(),
            it.next(),
            it.next(),
            it.next(),
            it.next(),
            it.next(),
        ) else {
            return Err("format");
        };
        if t.parse::<usize>().ok() != Some(id)
            || doc.len() != 24
            || !doc.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("header");
        }
        let Ok(seq) = seq.parse::<u64>() else {
            return Err("header");
        };
        if !ct_eq(key.as_bytes(), self.key_out.as_bytes()) {
            return Err("secret");
        }
        if !self.accept(doc, seq) {
            return Err("replay");
        }
        let Ok(mut m) = serde_json::from_str::<Value>(json) else {
            return Err("json");
        };
        let Some(o) = m.as_object_mut() else {
            return Err("json");
        };
        let cmd = o.get("cmd").and_then(Value::as_str).unwrap_or("");
        if !KNOWN.contains(&cmd) {
            return Err("command");
        }
        if HEAVY.contains(&cmd) && !self.allow_heavy(now) {
            return Err("rate");
        }
        o.insert("t".into(), json!(id));
        clamp_fields(o);
        Ok(m)
    }
}

/// Constant-time byte comparison.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |d, (x, y)| d | (x ^ y)) == 0
}

/// Truncate the page-supplied string fields to sane sizes (in characters).
fn clamp_fields(o: &mut serde_json::Map<String, Value>) {
    for (k, v) in o.iter_mut() {
        let cap = match k.as_str() {
            "text" => 5_000_000,
            "html" => 3_200_000,
            "url" => 4096,
            "title" | "k" | "v" | "lang" | "task" | "mode" => 2000,
            "q" | "sel" => 20_000,
            _ => 4096,
        };
        if let Value::String(t) = v
            && t.len() > cap
        {
            *t = t.chars().take(cap).collect();
        }
    }
}

/// `scheme://host[:port]` of an http(s) URL (lower-cased), or `None` for anything else.
fn origin_of(u: &str) -> Option<String> {
    let (scheme, rest) = u.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let host = rest.split(['/', '?', '#']).next()?;
    let host = host.rsplit('@').next()?;
    (!host.is_empty()).then(|| format!("{scheme}://{}", host.to_ascii_lowercase()))
}

/// Origin policy: `uri` is what the webview reports as the page the message came from.
/// Commands that carry page content must come from an http(s) page, and the `url` they claim
/// must be on that same origin.
fn origin_ok(uri: &str, m: &Value) -> bool {
    let cmd = m["cmd"].as_str().unwrap_or("");
    let page = origin_of(uri);
    if WEB_ONLY.contains(&cmd) && page.is_none() {
        return false;
    }
    match (m["url"].as_str(), &page) {
        (Some(u), Some(pg))
            if matches!(
                cmd,
                "save" | "later" | "note" | "highlight" | "watch" | "ask" | "grab" | "pdf"
            ) =>
        {
            origin_of(u).as_deref() == Some(pg.as_str())
        }
        _ => true,
    }
}

/// Authenticate an IPC request that arrived on tab `id`'s webview. EVERY message is untrusted
/// until this returns `Some`.
fn authenticate(c: &Cfg, id: usize, uri: &str, body: &str) -> Option<Value> {
    let m = {
        let mut g = c.auth.lock().ok()?;
        let a = g.get_mut(&id)?;
        a.check(id, body, Instant::now()).ok()?
    };
    origin_ok(uri, &m).then_some(m)
}

/// The script text that calls tab `tab`'s overlay entry point (one fresh sequence number per call).
/// Must be called in the order the scripts are evaluated (the event loop does).
fn entry_js(c: &Cfg, tab: usize, kind: &str, arg: &Value) -> Option<String> {
    let mut g = c.auth.lock().ok()?;
    let a = g.get_mut(&tab)?;
    a.out_seq += 1;
    Some(format!(
        "window.__cc_entry&&window.__cc_entry({},{},{},{})",
        json!(a.key_in),
        a.out_seq,
        json!(kind),
        arg
    ))
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
    let host = if auth.starts_with('[') {
        auth.split(']').next().unwrap_or("")
    } else {
        auth.split(':').next().unwrap_or("")
    };
    // `doubleclick.net.` is the same host (a trailing dot must not bypass the blocker)
    host.trim_end_matches('.').to_ascii_lowercase()
}

fn is_tracker(host: &str) -> bool {
    TRACKER_HOSTS
        .iter()
        .any(|t| host == *t || host.ends_with(&format!(".{t}")))
}

fn doc_ext(u: &str) -> Option<String> {
    if !is_http(u) {
        return None;
    }
    let path = u.split(['?', '#']).next().unwrap_or("");
    let last = path.rsplit('/').next().unwrap_or("");
    let (_, ext) = last.rsplit_once('.')?;
    let ext = ext.to_ascii_lowercase();
    DOC_EXT.contains(&ext.as_str()).then_some(ext)
}

/// A safe file name for a downloaded document: last path segment, only letters, digits and
/// `. - _ space` (no separators, no `..`), at most 100 characters (the end, so the extension stays).
fn file_name_of(u: &str) -> String {
    let path = u.split(['?', '#']).next().unwrap_or("");
    let last = path.rsplit(['/', '\\']).next().unwrap_or("");
    let clean: Vec<char> = last
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | ' '))
        .collect();
    let tail: String = clean[clean.len().saturating_sub(100)..].iter().collect();
    let tail = tail.trim().trim_start_matches('.').trim().to_string();
    if tail.is_empty() {
        "document".into()
    } else {
        tail
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

/// Address-bar input -> a web address, or `None` (then it is a library search). Only http(s)
/// ever comes out; other schemes, whitespace/control characters and huge input are searches.
fn normalize(q: &str) -> Option<String> {
    let q = q.trim();
    if q.is_empty() || q.len() > MAX_URL || q.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return None;
    }
    if q.contains("://")
        || scheme_of(q).is_some_and(|s| {
            matches!(
                s.as_str(),
                "javascript" | "data" | "file" | "blob" | "about" | "vbscript"
            )
        })
    {
        return is_http(q).then(|| q.to_string());
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
    if kind == "dl"
        && let Some(u) = x["url"].as_str()
    {
        remember(&c.offered, u);
    }
    let msg = json!({"ok": ok, "text": text, "k": kind, "x": x});
    let _ = p.send_event(Ev::Call(tab, "msg", msg));
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
    let colors = c.colors.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let pages: Vec<Page> = c
        .pages
        .lock()
        .unwrap()
        .iter()
        .filter(|(id, _)| color.is_none_or(|k| colors.get(id) == Some(&k)))
        .map(|(_, p)| p.clone())
        .collect();
    let mut paths = vec![];
    for p in pages.iter().filter(|p| is_http(&p.url)) {
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

/// The tab id a command refers to (0 = none; ids start at 1).
fn id_of(m: &Value) -> usize {
    m["id"]
        .as_u64()
        .and_then(|i| usize::try_from(i).ok())
        .unwrap_or(0)
}

fn color_of(m: &Value) -> Option<u8> {
    m["color"]
        .as_u64()
        .and_then(|k| u8::try_from(k).ok())
        .filter(|k| (1..=6).contains(k))
}

/// Loopback, link-local, private and unspecified hosts (the browser must not be a proxy into the
/// user's own machine or LAN on behalf of a web page that offered a document there).
fn is_local_host(h: &str) -> bool {
    let h = h.trim_matches(['[', ']']);
    if h == "localhost"
        || h.ends_with(".localhost")
        || h.ends_with(".local")
        || h.ends_with(".internal")
    {
        return true;
    }
    match h.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(a)) => {
            a.is_loopback() || a.is_private() || a.is_link_local() || a.is_unspecified()
        }
        Ok(std::net::IpAddr::V6(a)) => {
            let seg = a.segments();
            a.is_loopback()
                || a.is_unspecified()
                || (seg[0] & 0xfe00) == 0xfc00
                || (seg[0] & 0xffc0) == 0xfe80
                || a.to_ipv4_mapped().is_some_and(|v| {
                    v.is_loopback() || v.is_private() || v.is_link_local() || v.is_unspecified()
                })
        }
        Err(_) => false,
    }
}

fn download(c: &Cfg, url: &str) -> Result<String, String> {
    if !is_http(url) {
        return Err("only http(s) documents can be added".into());
    }
    if is_local_host(&host_of(url)) {
        return Err("documents on this computer or private networks cannot be added".into());
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

/// Related library documents for a page (never the page's own saved copy or history entry).
fn related_for(c: &Cfg, page: &Page) -> Vec<Value> {
    let home = state_dir(c.home.as_deref()).ok();
    let skip = home
        .as_ref()
        .and_then(|h| grab::status(h, &page.url, &page.text))
        .and_then(|s| s["path"].as_str().map(String::from));
    let mut rel = Library::open(c.home.as_deref(), false)
        .map(|l| related(&l, &page.title, skip.as_deref()))
        .unwrap_or_default();
    if let Some(h) = home.as_ref() {
        let own = grab::history_path(h, &page.url)
            .to_string_lossy()
            .into_owned();
        rel.retain(|r| r["path"].as_str() != Some(own.as_str()));
    }
    rel
}

fn handle(c: &Cfg, p: &EventLoopProxy<Ev>, m: &Value) {
    let s = |k: &str| m[k].as_str().unwrap_or("").to_string();
    let tab = m["t"].as_u64().unwrap_or(0) as usize;
    let lib = || Library::open(c.home.as_deref(), false);
    match m["cmd"].as_str().unwrap_or("") {
        "newtab" => {
            let u = s("url");
            let _ = p.send_event(Ev::NewTab((is_http(&u) && u.len() <= MAX_URL).then_some(u)));
        }
        "reopen" => {
            let _ = p.send_event(Ev::Reopen);
        }
        "tour_seen" => prefs_set(c, &[("tour_done", json!(true))]),
        "close" => {
            let _ = p.send_event(Ev::Close(id_of(m)));
        }
        "activate" => {
            let _ = p.send_event(Ev::Activate(id_of(m)));
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
            let b = json!({"tour": tour, "start": start, "mock": c.mock, "prefs": {
                "rd_size": prefs_get(c).get("rd_size").cloned().unwrap_or(json!(19)),
                "rd_theme": prefs_get(c).get("rd_theme").cloned().unwrap_or(json!("")),
                "trackers": c.trackers.load(Ordering::Relaxed),
                "pdfNative": PDF_NATIVE,
            }});
            let _ = p.send_event(Ev::Call(tab, "boot", b));
            let _ = p.send_event(Ev::Push);
        }
        "loaded" => {
            let page = Page {
                url: clean_url(&s("url")),
                title: s("title"),
                text: s("text"),
            };
            c.pages
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(tab, page.clone());
            let _ = p.send_event(Ev::Url(tab, s("url")));
            let _ = p.send_event(Ev::Title(tab, page.title.clone()));
            if !is_http(&page.url) {
                return;
            }
            let home = state_dir(c.home.as_deref()).ok();
            let status = home
                .as_ref()
                .and_then(|h| grab::status(h, &page.url, &page.text));
            let history = home.as_ref().is_some_and(|h| grab::history_enabled(h));
            // Only the COUNT goes to the page's world now: the titles/snippets of related library
            // documents are shown on demand ("related" command, after a click on the badge). The
            // page controls its own title, so pushing the search hits would let it probe the library.
            let rn = related_for(c, &page).len();
            let hl = home
                .as_ref()
                .map(|h| grab::highlights(h, &page.url))
                .unwrap_or_default();
            let info = json!({"status": status, "rn": rn, "highlights": hl, "history": history});
            let _ = p.send_event(Ev::Call(tab, "info", info));
            if let (true, Ok(mut l)) = (history, lib()) {
                let _ = grab::history_add(&mut l, &page.url, &page.title, &page.text);
            }
        }
        "related" => {
            let page = c
                .pages
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get(&tab)
                .cloned();
            let rel = page.map(|pg| related_for(c, &pg)).unwrap_or_default();
            reply_k(p, c, tab, true, String::new(), "rel", json!(rel));
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
            let id = id_of(m);
            match color_of(m) {
                Some(k) => c
                    .colors
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(id, k),
                None => c
                    .colors
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .remove(&id),
            };
            let _ = p.send_event(Ev::Push);
        }
        "discard" => {
            let _ = p.send_event(Ev::Discard(
                m["id"].as_u64().and_then(|i| usize::try_from(i).ok()),
            ));
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
            if is_http(&u)
                && c.offered
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .contains(&u)
            {
                remember(&c.allow, &u);
                let _ = p.send_event(Ev::Navigate(tab, u));
            }
        }
        "dl" if !c
            .offered
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(&s("url")) =>
        {
            reply(
                p,
                c,
                tab,
                false,
                "this document was not offered by the browser".into(),
            );
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
        "grab" => {
            let url = clean_url(&s("url"));
            let fmt = s("fmt");
            if !is_http(&url) || !["html", "mhtml"].contains(&fmt.as_str()) {
                return reply(p, c, tab, false, "cannot grab this page".into());
            }
            // the page as the user sees it (scripts run, logged in) when the bar sent it, else a fresh fetch;
            // either way every sub-resource goes through the SSRF-guarded downloader
            let r = lib().and_then(|mut l| match m["html"].as_str().filter(|h| !h.is_empty()) {
                Some(h) => grab::save_page_html(&mut l, &url, h, &fmt),
                None => grab::save_page(&mut l, &url, &fmt),
            });
            match r {
                Ok(v) => reply_k(
                    p,
                    c,
                    tab,
                    true,
                    format!("✔ {}", v["path"].as_str().unwrap_or("")),
                    "gok",
                    json!({"path": v["path"], "fmt": fmt, "kb": v["bytes"].as_u64().unwrap_or(0) / 1024}),
                ),
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        "pdf" => {
            let url = clean_url(&s("url"));
            if !is_http(&url) {
                return reply(p, c, tab, false, "cannot print this page".into());
            }
            match lib().and_then(|l| grab::pdf_target(&l, &url, &s("title"))) {
                Ok(path) => {
                    let _ = p.send_event(Ev::Pdf(tab, path));
                }
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
                let _ = p.send_event(Ev::Call(tab, "info", json!({"history": on})));
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

/// A webview that could not be built must not crash the browser: forget its secrets and carry on.
fn finish(cfg: &Cfg, id: usize, r: Result<WebView, String>) -> Option<WebView> {
    match r {
        Ok(w) => Some(w),
        Err(e) => {
            eprintln!("cartacolibri-browser: cannot create tab {id}: {e}");
            cfg.auth
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&id);
            None
        }
    }
}

fn make_tab(
    window: &Window,
    cfg: &Arc<Cfg>,
    proxy: &EventLoopProxy<Ev>,
    id: usize,
    url: Option<&str>,
) -> Option<WebView> {
    let (c2, p2) = (cfg.clone(), proxy.clone());
    let c3 = cfg.clone();
    let (p3, p4, p5, p6) = (proxy.clone(), proxy.clone(), proxy.clone(), proxy.clone());
    let (c5, p7, c7) = (cfg.clone(), proxy.clone(), cfg.clone());
    // a fresh pair of secrets for every webview (also when a sleeping tab wakes up)
    let (key_out, key_in) = {
        let a = Auth::new();
        let k = (a.key_out.clone(), a.key_in.clone());
        cfg.auth
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, a);
        k
    };
    let script = OVERLAY
        .replace("__KEYOUT__", &key_out)
        .replace("__KEYIN__", &key_in)
        .replace("__TAB__", &id.to_string());
    let mut b = WebViewBuilder::new()
        .with_initialization_script_for_main_only(script, true)
        .with_new_window_req_handler(move |u, _| {
            // page-opened windows: http(s) only, and at most POPUP_MAX per POPUP_WIN (popup spam)
            let go = is_http(&u)
                && u.len() <= MAX_URL
                && throttle(
                    &mut c3.popups.lock().unwrap_or_else(|e| e.into_inner()),
                    Instant::now(),
                    POPUP_MAX,
                    POPUP_WIN,
                );
            if go {
                let _ = p3.send_event(Ev::NewTab(Some(u)));
            }
            wry::NewWindowResponse::Deny
        })
        .with_document_title_changed_handler(move |t| {
            let _ = p4.send_event(Ev::Title(id, trunc(&t, 500)));
        })
        .with_navigation_handler(move |u| {
            if !nav_allowed(&u) {
                return false; // file:, javascript:, ftp:, custom schemes, giant addresses
            }
            if is_http(&u) {
                let host = host_of(&u);
                if c5.trackers.load(Ordering::Relaxed) && is_tracker(&host) {
                    reply_k(&p5, &c5, id, true, String::new(), "blk", json!(host));
                    return false;
                }
                if doc_ext(&u).is_some()
                    && !c5
                        .allow
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .contains(&u)
                {
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
            if !is_http(&u) {
                // blob:/data: downloads are the page's own bytes; anything else (file:, ...) is refused
                return matches!(scheme_of(&u).as_deref(), Some("blob" | "data"));
            }
            if c7
                .allow
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .contains(&u)
            {
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
            // EVERY message is untrusted until it proves it knows this webview's secret
            let uri = req.uri().to_string();
            let Some(m) = authenticate(&c2, id, &uri, req.body()) else {
                return;
            };
            // bounded concurrency: a flood of valid-looking commands cannot spawn unbounded threads
            if INFLIGHT.fetch_add(1, Ordering::SeqCst) >= MAX_INFLIGHT {
                INFLIGHT.fetch_sub(1, Ordering::SeqCst);
                return;
            }
            let (c, p) = (c2.clone(), p2.clone());
            if std::thread::Builder::new()
                .spawn(move || {
                    let _g = Inflight;
                    handle(&c, &p, &m);
                })
                .is_err()
            {
                INFLIGHT.fetch_sub(1, Ordering::SeqCst);
            }
        });
    b = match url {
        Some(u) => match normalize(u).filter(|n| is_http(n)) {
            Some(n) => b.with_url(n),
            None => b.with_html(START),
        },
        None => b.with_html(START),
    };
    if let (Some(mode), 1) = (cfg.selftest, id) {
        b = b.with_on_page_load_handler(move |ev, _| {
            if matches!(ev, wry::PageLoadEvent::Finished) {
                if mode == "forge" {
                    let _ = p6.send_event(Ev::ForgeGo);
                    return;
                }
                let (args, ms) = match mode {
                    "tabs" => (json!(["ask_tabs", "colibri"]), 3500),
                    "ask" => (json!(["ask", "ce sunt colibrii?"]), 900),
                    "grab" => (json!(["grab", "mhtml"]), 900),
                    "pdf" => (json!(["pdf"]), 900),
                    _ => (json!(["save"]), 900),
                };
                let p = p6.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(ms));
                    let _ = p.send_event(Ev::Call(id, "do", args));
                });
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
            let built = window
                .default_vbox()
                .ok_or_else(|| "no gtk container".to_string())
                .and_then(|v| b.build_gtk(v).map_err(|e| e.to_string()));
            return finish(cfg, id, built);
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
        return finish(
            cfg,
            id,
            b.with_bounds(full_bounds(window))
                .build_as_child(window)
                .map_err(|e| e.to_string()),
        );
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

/// What the simulated hostile page does (run with `evaluate_script`, i.e. in the page's own JS
/// world, after the overlay is up — the strongest position a page script can have, since the
/// overlay captured everything it needs at document start).
const FORGE_JS: &str = r#"(() => {
  const P = { tried: 0, hex: [], fns: [], seen: [] };
  Object.defineProperty(window, "__probe", { value: P, configurable: true });
  const jo = JSON.stringify, ra = Reflect.apply, sl = Array.prototype.slice;
  const rec = (t, a) => { try { P.seen.push(t + ":" + ra(sl, a, []).map((x) => typeof x === "string" ? x : jo(x)).join("|").slice(0, 400)); } catch (e) {} };
  const hook = (o, k, tag) => { try { const f = o[k]; if (typeof f !== "function") return; Object.defineProperty(o, k, { configurable: true, writable: true, value: function () { rec(tag, arguments); return ra(f, this, arguments); } }); } catch (e) {} };
  // 1. a hostile page patches everything it can to read or rewrite what the overlay sends
  hook(JSON, "stringify", "JSON.stringify");
  hook(Function.prototype, "call", "call"); hook(Function.prototype, "apply", "apply"); hook(Function.prototype, "bind", "bind");
  hook(String.prototype, "concat", "concat"); hook(Object, "assign", "assign"); hook(Object, "keys", "keys");
  hook(Uint8Array.prototype, "set", "u8set");
  try { hook(window.webkit.messageHandlers.ipc, "postMessage", "native.postMessage"); } catch (e) {}
  try { hook(window.chrome.webview, "postMessage", "webview.postMessage"); } catch (e) {}
  try { Object.defineProperty(Object.prototype, "toJSON", { configurable: true, get() { rec("toJSON", []); return function () { return { cmd: "history" }; }; } }); } catch (e) {}
  for (const k of ["cmd", "url", "text", "html"]) { try { Object.defineProperty(Object.prototype, k, { configurable: true, get() { return k === "cmd" ? "history" : "POISON"; }, set() {} }); } catch (e) {} }
  // 2. enumerate window property names
  for (const n of Object.getOwnPropertyNames(window)) {
    if (/[0-9a-f]{16,}/i.test(n)) P.hex.push(n);
    try { if (/cc|nonce|colibri|ipc/i.test(n) && typeof window[n] === "function") P.fns.push(n); } catch (e) {}
  }
  const guesses = ["", "0", "1", "null", "undefined", "nonce", "secret", "0".repeat(64), "a".repeat(64), String(Date.now()), "__cc_entry", ...P.hex, ...P.fns];
  const kinds = ["msg", "do", "tabs", "info", "boot", "__proto__", "constructor", "toString"];
  const args = [["save"], ["history"], ["ask", "POISON"], { ok: true, text: "x", k: "save" }, { history: true, highlights: ["POISON"] }, null, "x"];
  // 3. call every cc-looking function with guessed secrets and sequence numbers
  for (const n of P.fns) {
    for (const g of guesses) for (const q of [0, 1, 2, 1e6, -1, NaN, "1"]) for (let i = 0; i < kinds.length; i++) {
      try { window[n](g, q, kinds[i], args[i % args.length]); window[n](); P.tried++; } catch (e) {}
    }
  }
  // 4. forged IPC messages through every door: wry's window.ipc and the native handler
  const doors = [];
  try { doors.push((m) => window.ipc.postMessage(m)); } catch (e) {}
  try { doors.push((m) => window.webkit.messageHandlers.ipc.postMessage(m)); } catch (e) {}
  try { doors.push((m) => window.chrome.webview.postMessage(m)); } catch (e) {}
  const cmds = ["save", "history", "watch", "highlight", "ask", "ask_tabs", "save_tabs", "later", "note", "dl", "open_raw", "trackers", "tour_seen", "newtab", "close", "sel", "pref", "go"];
  const keys = ["", "0", "x".repeat(64), "0".repeat(64), "null"];
  const docs = ["0".repeat(24), "a".repeat(24), "abcdef0123456789abcdef01", "short", ""];
  for (const door of doors) {
    for (const cmd of cmds) {
      const body = jo({ cmd, url: location.href, title: "POISON", text: "POISON ".repeat(50), html: "<p>POISON</p>", sel: "POISON", q: "POISON", id: 1, n: "x", t: 1 });
      const legacy = [body, jo({ n: "", t: 1, cmd }), jo({ n: "0".repeat(40), t: 1, cmd, url: location.href, text: "POISON" }), "{}", "[]", "null", "", "cc1", "cc1....."];
      for (const m of legacy) { try { door(m); P.tried++; } catch (e) {} }
      for (const k of keys) for (const d of docs) for (const seq of [0, 1, 2, 3, 1e9]) {
        try { door(["cc1", 1, d, seq, k, body].join(".")); P.tried++; } catch (e) {}
      }
      // replay: the same forged message twice
      const rp = ["cc1", 1, docs[2], 1, keys[1], body].join(".");
      try { door(rp); door(rp); P.tried += 2; } catch (e) {}
    }
  }
  // 5. script-dispatched ("untrusted") shortcuts and clicks must not trigger Save/Ask/Highlight/new tab
  for (const k of ["s", "h", "A", "r", "k"]) {
    for (const t of ["keydown", "keyup"]) { try { window.dispatchEvent(new KeyboardEvent(t, { key: k, ctrlKey: true, shiftKey: k === "A", bubbles: true })); document.dispatchEvent(new KeyboardEvent(t, { key: k, ctrlKey: true, bubbles: true })); P.tried++; } catch (e) {} }
  }
  try { const a = document.createElement("a"); a.href = "https://example.invalid/"; a.textContent = "x"; for (const t of ["click", "auxclick"]) a.addEventListener(t, (e) => e.preventDefault()); document.body.appendChild(a); a.dispatchEvent(new MouseEvent("click", { ctrlKey: true, bubbles: true, cancelable: true })); a.dispatchEvent(new MouseEvent("auxclick", { button: 1, bubbles: true, cancelable: true })); } catch (e) {}
  P.done = true;
})()"#;

/// Files (relative path + size) under the library home, ignoring database sidecar files.
fn snapshot(home: Option<&str>) -> Vec<String> {
    fn walk(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            let rel = p
                .strip_prefix(base)
                .unwrap_or(&p)
                .to_string_lossy()
                .into_owned();
            if p.is_dir() {
                walk(base, &p, out);
            } else if !(rel.ends_with("-wal")
                || rel.ends_with("-shm")
                || rel.ends_with("-journal")
                || rel.ends_with(".db")
                || rel.ends_with(".sqlite")
                || rel.ends_with(".lock"))
            {
                out.push(format!(
                    "{rel}:{}",
                    e.metadata().map(|m| m.len()).unwrap_or(0)
                ));
            }
        }
    }
    let mut v = vec![];
    if let Ok(h) = state_dir(home) {
        walk(&h, &h, &mut v);
        v.push(format!("history={}", grab::history_enabled(&h)));
    }
    v.sort();
    v
}

/// State machine of `--selftest-forge`.
#[derive(Default)]
struct Forge {
    stage: u8,
    since: Option<Instant>,
    before: Vec<String>,
    after_attack: Vec<String>,
}

impl Forge {
    fn go(&mut self) {
        if self.stage == 0 {
            self.stage = 1;
            self.since = Some(Instant::now());
        }
    }

    fn fail(&self, why: &str, flow: &mut ControlFlow) {
        println!("selftest FAILED: {why}");
        *flow = ControlFlow::ExitWithCode(1);
    }

    /// Called on every loop iteration; returns true when the event should not be processed further.
    fn tick(&mut self, cfg: &Cfg, tabs: &[Tab], flow: &mut ControlFlow) -> bool {
        let Some(t0) = self.since else {
            return false;
        };
        let age = Instant::now().duration_since(t0);
        let home = cfg.home.as_deref();
        match self.stage {
            1 if age > Duration::from_millis(2500) => {
                // the page and the overlay are up and its own start-up traffic is over
                self.before = snapshot(home);
                if let Some(wv) = tabs.first().and_then(|t| t.wv.as_ref()) {
                    let _ = wv.evaluate_script(FORGE_JS);
                }
                self.stage = 2;
                self.since = Some(Instant::now());
            }
            2 if age > Duration::from_millis(4000) => {
                self.after_attack = snapshot(home);
                if self.after_attack != self.before || tabs.len() != 1 {
                    self.fail(
                        &format!(
                            "a forged message changed the library/home\n before: {:?}\n after:  {:?}",
                            self.before, self.after_attack
                        ),
                        flow,
                    );
                    return true;
                }
                // the overlay must still work: a legitimate save
                let args = json!(["save"]);
                if let Some(wv) = tabs.first().and_then(|t| t.wv.as_ref())
                    && let Some(js) = entry_js(cfg, 1, "do", &args)
                {
                    let _ = wv.evaluate_script(&js);
                }
                self.stage = 3;
                self.since = Some(Instant::now());
            }
            3 if age > Duration::from_secs(20) => {
                self.fail("the legitimate save never answered", flow);
                return true;
            }
            _ => {}
        }
        false
    }

    /// A reply from the Rust side to the overlay (kind "msg") was produced.
    fn reply(
        &mut self,
        cfg: &Cfg,
        tabs: &[Tab],
        proxy: &EventLoopProxy<Ev>,
        arg: &Value,
        flow: &mut ControlFlow,
    ) {
        if self.stage != 3 {
            self.fail(
                &format!("a forged command produced a reply before the legit save: {arg}"),
                flow,
            );
            return;
        }
        let ok = arg["ok"] == json!(true) && arg["k"] == json!("save");
        let now = snapshot(cfg.home.as_deref());
        if !ok || now == self.after_attack {
            self.fail(&format!("the legitimate save did not work: {arg}"), flow);
            return;
        }
        self.stage = 4;
        if let Some(wv) = tabs.first().and_then(|t| t.wv.as_ref()) {
            let p = proxy.clone();
            let _ = wv.evaluate_script_with_callback(
                "(() => { try { delete Object.prototype.toJSON; delete Object.prototype.cmd; delete Object.prototype.url; delete Object.prototype.text; delete Object.prototype.html; } catch (e) {} return JSON.stringify(window.__probe || {}); })()",
                move |r| {
                    let _ = p.send_event(Ev::Probe(r));
                },
            );
        }
    }

    /// The hostile page's own notes: did it ever see a secret or find a secret-named global?
    fn probe(&mut self, cfg: &Cfg, raw: &str, flow: &mut ControlFlow) {
        let inner = serde_json::from_str::<Value>(raw).unwrap_or(Value::Null);
        let p = match &inner {
            Value::String(s) => serde_json::from_str::<Value>(s).unwrap_or(Value::Null),
            v => v.clone(),
        };
        let g = cfg.auth.lock().unwrap_or_else(|e| e.into_inner());
        let a = g.get(&1);
        let seen = p["seen"].to_string();
        let leaked = a.is_some_and(|a| {
            seen.contains(&a.key_out) || seen.contains(&a.key_in) || raw.contains(&a.key_out)
        });
        let tried = p["tried"].as_u64().unwrap_or(0);
        let hex = p["hex"].as_array().map(|v| v.len()).unwrap_or(99);
        let fns: Vec<String> = p["fns"]
            .as_array()
            .map(|v| {
                v.iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let only_entry = fns
            .iter()
            .filter(|n| n.contains("cc"))
            .all(|n| n == "__cc_entry");
        if p["done"] != json!(true) || tried < 100 {
            self.fail(
                &format!("the hostile page did not run to completion (tried={tried})"),
                flow,
            );
        } else if leaked {
            self.fail("a secret reached page-observable code", flow);
        } else if hex != 0 || !only_entry {
            self.fail(
                &format!("secret-looking globals: hex={hex} fns={fns:?}"),
                flow,
            );
        } else {
            println!(
                "selftest ok: {tried} forged calls/messages/events ignored, nothing saved or changed, secrets unseen (globals: {fns:?}), legit save still works"
            );
            *flow = ControlFlow::ExitWithCode(0);
        }
    }
}

/// True where the webview can write a PDF file by itself (WebKitGTK); elsewhere the print dialog opens.
const PDF_NATIVE: bool = cfg!(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
));

/// Tells the bar the PDF is in the library: indexes the new file, then replies.
fn pdf_done(cfg: &Arc<Cfg>, proxy: &EventLoopProxy<Ev>, tab: usize, path: std::path::PathBuf) {
    let (cfg, proxy) = (cfg.clone(), proxy.clone());
    std::thread::spawn(move || {
        let r =
            Library::open(cfg.home.as_deref(), false).and_then(|mut l| grab::index_grabs(&mut l));
        match r {
            Ok(_) => reply_k(
                &proxy,
                &cfg,
                tab,
                true,
                format!("✔ {}", path.display()),
                "pdfok",
                json!({"path": path}),
            ),
            Err(e) => reply(&proxy, &cfg, tab, false, e),
        }
    });
}

/// Print the tab to `path` as PDF. WebKitGTK: a silent print job to a file. Windows (WebView2) and
/// macOS (WKWebView): wry 0.57 only offers the print dialog, so the user picks "Save as PDF" there.
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn print_pdf(
    cfg: &Arc<Cfg>,
    proxy: &EventLoopProxy<Ev>,
    tab: usize,
    wv: &WebView,
    path: std::path::PathBuf,
) {
    use gtk::gio::prelude::FileExt;
    use webkit2gtk::PrintOperationExt;
    use wry::WebViewExtUnix;
    let op = webkit2gtk::PrintOperation::new(&wv.webview());
    let st = gtk::PrintSettings::new();
    st.set_printer("Print to File");
    st.set("output-file-format", Some("pdf"));
    st.set(
        "output-uri",
        Some(gtk::gio::File::for_path(&path).uri().as_str()),
    );
    op.set_print_settings(&st);
    let failed = std::rc::Rc::new(std::cell::Cell::new(None::<String>));
    let (f2, c2, p2, path2) = (failed.clone(), cfg.clone(), proxy.clone(), path.clone());
    op.connect_failed(move |_, e| f2.set(Some(e.to_string())));
    let finished = std::cell::Cell::new(true);
    op.connect_finished(move |_| {
        if !finished.replace(false) {
            return;
        }
        match failed.take() {
            Some(e) => {
                let _ = std::fs::remove_file(&path2);
                reply(&p2, &c2, tab, false, format!("PDF failed: {e}"));
            }
            None => pdf_done(&c2, &p2, tab, path2.clone()),
        }
    });
    op.print();
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
fn print_pdf(
    cfg: &Arc<Cfg>,
    proxy: &EventLoopProxy<Ev>,
    tab: usize,
    wv: &WebView,
    path: std::path::PathBuf,
) {
    // the empty placeholder is not a PDF: remove it, the user saves the real file from the dialog
    let dir = path.parent().map(|d| d.to_path_buf()).unwrap_or_default();
    let _ = std::fs::remove_file(&path);
    match wv.print() {
        Ok(()) => reply_k(
            proxy,
            cfg,
            tab,
            true,
            format!("print window → {}", dir.display()),
            "pdfdlg",
            json!({"dir": dir}),
        ),
        Err(e) => reply(proxy, cfg, tab, false, format!("print failed: {e}")),
    }
}

fn push_tabs(cfg: &Cfg, tabs: &[Tab], active: usize) {
    let colors = cfg.colors.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let dups: Vec<bool> = tabs
        .iter()
        .map(|t| is_http(&t.url) && seen.insert(dup_key(&t.url), t.id).is_some())
        .collect();
    for t in tabs {
        // The overlay lives in the page's own JS world, so a hostile page can read what is pushed
        // to it: other tabs get their address without query/fragment/userinfo (tokens live there).
        let list: Vec<Value> = tabs
            .iter()
            .zip(&dups)
            .map(|(o, dup)| {
                let url = if o.id == t.id { trunc(&o.url, 2048) } else { redact_url(&o.url) };
                json!({"id": o.id, "title": trunc(&o.title, 300), "url": url, "color": colors.get(&o.id).copied().unwrap_or(0),
                       "sleep": o.wv.is_none(), "dup": dup})
            })
            .collect();
        let st = json!({"tabs": list, "active": active});
        if let Some(wv) = &t.wv
            && let Some(js) = entry_js(cfg, t.id, "tabs", &st)
        {
            let _ = wv.evaluate_script(&js);
        }
    }
}

fn save_session(cfg: &Cfg, tabs: &[Tab]) {
    if let Ok(h) = state_dir(cfg.home.as_deref()) {
        let urls: Vec<&str> = tabs
            .iter()
            .map(|t| t.url.as_str())
            .filter(|u| is_http(u))
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
        t.wv = make_tab(window, cfg, proxy, id, u.as_deref());
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
            "cartacolibri-browser [URL…] [--home DIR] [--gateway URL] [--mock] [--fresh]\n  --fresh   do not restore the previous tabs\n  keys: Ctrl+S save · Ctrl+Shift+A ask · Ctrl+H highlight · Ctrl+F find · Ctrl+Shift+R reader · Ctrl+Shift+K search tabs · F1 help · Ctrl+Shift+T reopen tab · Esc close panels\n  --selftest-save | --selftest-ask | --selftest-tabs   load the first URL, save it / ask about it (use with --mock), print the result, exit (used by CI)\n  --selftest-grab | --selftest-pdf   load the first URL, grab it as .mhtml / print it to PDF (set CARTACOLIBRI_GRAB_LOCAL=1 for a loopback page), print the result, exit\n  --selftest-forge   load the first URL, let a simulated hostile page forge IPC messages and call the overlay; passes only if nothing changed and a real save still works"
        );
        return;
    }
    let selftest: Option<&'static str> = if flag("--selftest-tabs") {
        Some("tabs")
    } else if flag("--selftest-ask") {
        Some("ask")
    } else if flag("--selftest-save") {
        Some("save")
    } else if flag("--selftest-forge") {
        Some("forge")
    } else if flag("--selftest-grab") {
        Some("grab")
    } else if flag("--selftest-pdf") {
        Some("pdf")
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
        selftest,
        auth: Mutex::new(BTreeMap::new()),
        offered: Mutex::new(BTreeSet::new()),
        pages: Mutex::new(BTreeMap::new()),
        colors: Mutex::new(BTreeMap::new()),
        trackers: AtomicBool::new(false),
        allow: Mutex::new(BTreeSet::new()),
        popups: Mutex::new(VecDeque::new()),
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
    // only http(s) addresses, at most MAX_TABS (tabs.json and the command line are not trusted blindly)
    let start_urls: Vec<String> = start_urls
        .into_iter()
        .filter_map(|u| normalize(&u).filter(|n| is_http(n)))
        .take(MAX_TABS)
        .collect();

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
        let Some(wv) = make_tab(&window, &cfg, &proxy, id, u.as_deref()) else {
            continue;
        };
        tabs.push(Tab {
            id,
            wv: Some(wv),
            title: String::new(),
            url: u.unwrap_or_default(),
        });
        active = id;
    }
    if tabs.is_empty() {
        eprintln!("cartacolibri-browser: no webview could be created");
        std::process::exit(1);
    }
    show_only(&tabs, active);
    let mut closed: Vec<String> = vec![];

    let deadline = Instant::now() + Duration::from_secs(60);
    let mut forge = Forge::default();
    event_loop.run(move |event, _, flow| {
        if selftest == Some("forge") && forge.tick(&cfg, &tabs, flow) {
            return;
        }
        *flow = if selftest.is_some() {
            ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(250),
            )
        } else {
            ControlFlow::Wait
        };
        match event {
            Event::UserEvent(Ev::Call(tab, kind, arg)) => {
                if selftest == Some("forge") {
                    if kind == "msg" && arg.get("text").is_some() {
                        forge.reply(&cfg, &tabs, &proxy, &arg, flow);
                        if matches!(*flow, ControlFlow::ExitWithCode(_)) {
                            return;
                        }
                    }
                } else if selftest.is_some() && kind == "msg" && arg.get("text").is_some() {
                    let ok = arg["ok"] == json!(true)
                        && (tabs.len() < 2
                            || arg["text"].as_str().is_some_and(|t| t.contains("[S2]")));
                    println!("selftest {}: {arg}", if ok { "ok" } else { "FAILED" });
                    *flow = ControlFlow::ExitWithCode(if ok { 0 } else { 1 });
                    return;
                }
                if let Some(wv) = tabs
                    .iter()
                    .find(|t| t.id == tab)
                    .and_then(|t| t.wv.as_ref())
                    && let Some(js) = entry_js(&cfg, tab, kind, &arg)
                {
                    let _ = wv.evaluate_script(&js);
                }
            }
            Event::UserEvent(Ev::ForgeGo) => forge.go(),
            Event::UserEvent(Ev::Probe(raw)) => forge.probe(&cfg, &raw, flow),
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
            Event::UserEvent(Ev::Pdf(tab, path)) => {
                match tabs
                    .iter()
                    .find(|t| t.id == tab)
                    .and_then(|t| t.wv.as_ref())
                {
                    Some(wv) => print_pdf(&cfg, &proxy, tab, wv, path),
                    None => {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
            Event::UserEvent(Ev::Discard(which)) => {
                let mut n = 0;
                for t in tabs.iter_mut() {
                    let pick = which.is_none_or(|w| w == t.id);
                    if pick && t.id != active && t.wv.is_some() && is_http(&t.url) {
                        t.wv = None;
                        // late messages of the dropped webview are refused (a wake mints new secrets)
                        cfg.auth
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .remove(&t.id);
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
                    .filter(|t| is_http(&t.url))
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
                    cfg.auth
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .remove(id);
                    cfg.pages
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .remove(id);
                    cfg.colors
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .remove(id);
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
                let u = u.filter(|u| is_http(u) && u.len() <= MAX_URL);
                if tabs.len() >= MAX_TABS {
                    reply_k(
                        &proxy,
                        &cfg,
                        active,
                        false,
                        format!("at most {MAX_TABS} tabs"),
                        "",
                        json!(null),
                    );
                    return;
                }
                let id = next_id;
                next_id += 1;
                let Some(wv) = make_tab(&window, &cfg, &proxy, id, u.as_deref()) else {
                    return;
                };
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
                        if is_http(&gone.url) {
                            closed.push(gone.url);
                            if closed.len() > 20 {
                                closed.remove(0);
                            }
                        }
                        cfg.auth
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .remove(&id);
                        cfg.pages
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .remove(&id);
                        cfg.colors
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .remove(&id);
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
                let title = trunc(&title, 500);
                if let Some(t) = tabs.iter_mut().find(|t| t.id == id)
                    && t.title != title
                {
                    t.title = title;
                    push_tabs(&cfg, &tabs, active);
                }
            }
            Event::UserEvent(Ev::Url(id, url)) => {
                let url = trunc(&url, 4096);
                if let Some(t) = tabs.iter_mut().find(|t| t.id == id)
                    && t.url != url
                {
                    t.url = url;
                    push_tabs(&cfg, &tabs, active);
                    save_session(&cfg, &tabs);
                }
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. })
                if selftest.is_some() && Instant::now() > deadline =>
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
                tabs.iter().filter_map(|t| t.wv.as_ref()).for_each(|wv| {
                    let _ = wv.set_bounds(b);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(a: &Auth, doc: &str, seq: u64, key: &str, body: &str) -> String {
        let _ = a;
        format!("cc1.1.{doc}.{seq}.{key}.{body}")
    }
    const DOC: &str = "0123456789abcdef01234567";
    const DOC2: &str = "76543210fedcba9876543210";

    #[test]
    fn rejects_forgeries_and_replays() {
        let mut a = Auth::new();
        let k = a.key_out.clone();
        let t = Instant::now();
        let ok = msg(&a, DOC, 1, &k, r#"{"cmd":"history"}"#);
        assert!(a.check(1, &ok, t).is_ok());
        assert_eq!(a.check(1, &ok, t), Err("replay")); // replay
        assert_eq!(
            a.check(
                1,
                &msg(&a, DOC, 2, &"0".repeat(64), "{\"cmd\":\"save\"}"),
                t
            ),
            Err("secret")
        );
        assert_eq!(
            a.check(1, &msg(&a, DOC, 2, "", "{\"cmd\":\"save\"}"), t),
            Err("secret")
        );
        assert_eq!(
            a.check(1, &format!("{{\"n\":\"{k}\",\"t\":1,\"cmd\":\"save\"}}"), t),
            Err("format")
        );
        assert_eq!(
            a.check(2, &msg(&a, DOC, 3, &k, "{\"cmd\":\"save\"}"), t),
            Err("header")
        ); // other tab
        assert_eq!(
            a.check(1, &msg(&a, DOC, 3, &k, "{\"cmd\":\"rm -rf\"}"), t),
            Err("command")
        );
        assert!(
            a.check(
                1,
                &msg(&a, DOC, 4, &k, "{\"cmd\":\"save\",\"text\":\"x\"}"),
                t
            )
            .is_ok()
        );
        // an older sequence number of the same document is a replay too
        assert_eq!(
            a.check(1, &msg(&a, DOC, 3, &k, "{\"cmd\":\"save\"}"), t),
            Err("replay")
        );
    }

    #[test]
    fn reload_starts_a_new_document_and_retires_the_old() {
        let mut a = Auth::new();
        let k = a.key_out.clone();
        let t = Instant::now();
        assert!(
            a.check(1, &msg(&a, DOC, 1, &k, "{\"cmd\":\"ready\"}"), t)
                .is_ok()
        );
        assert!(
            a.check(1, &msg(&a, DOC, 2, &k, "{\"cmd\":\"ready\"}"), t)
                .is_ok()
        );
        assert!(
            a.check(1, &msg(&a, DOC2, 1, &k, "{\"cmd\":\"ready\"}"), t)
                .is_ok()
        );
        // the old document can no longer speak, nor can its seq 1 be replayed
        assert_eq!(
            a.check(1, &msg(&a, DOC, 3, &k, "{\"cmd\":\"ready\"}"), t),
            Err("replay")
        );
        assert_eq!(
            a.check(1, &msg(&a, DOC, 1, &k, "{\"cmd\":\"ready\"}"), t),
            Err("replay")
        );
        // a new document that does not start at 1 is refused
        assert_eq!(
            a.check(
                1,
                &msg(&a, "ffffffffffffffffffffffff", 7, &k, "{\"cmd\":\"ready\"}"),
                t
            ),
            Err("replay")
        );
    }

    #[test]
    fn rate_limits_and_size() {
        let mut a = Auth::new();
        let k = a.key_out.clone();
        let t = Instant::now();
        let mut n = 0;
        for i in 1..=200u64 {
            if a.check(1, &msg(&a, DOC, i, &k, "{\"cmd\":\"ready\"}"), t)
                .is_ok()
            {
                n += 1;
            }
        }
        assert_eq!(n, RATE_ALL);
        // the window slides
        let t2 = t + Duration::from_secs(2);
        assert!(
            a.check(1, &msg(&a, DOC, 500, &k, "{\"cmd\":\"ready\"}"), t2)
                .is_ok()
        );
        // heavy commands: at most RATE_HEAVY per 10 s
        let mut a = Auth::new();
        let k = a.key_out.clone();
        let mut n = 0;
        for i in 1..=30u64 {
            let t = t + Duration::from_millis(i * 100); // 10/s: under RATE_ALL
            if a.check(1, &msg(&a, DOC, i, &k, "{\"cmd\":\"ask\",\"q\":\"x\"}"), t)
                .is_ok()
            {
                n += 1;
            }
        }
        assert_eq!(n, RATE_HEAVY);
        let big = format!("cc1.1.{DOC}.1.{k}.{}", "x".repeat(MAX_MSG));
        assert_eq!(Auth::new().check(1, &big, t), Err("oversize"));
    }

    #[test]
    fn fields_are_clamped() {
        let mut a = Auth::new();
        let k = a.key_out.clone();
        let body = format!(
            "{{\"cmd\":\"save\",\"title\":\"{}\",\"url\":\"http://a/\"}}",
            "é".repeat(50_000)
        );
        let m = a
            .check(1, &msg(&a, DOC, 1, &k, &body), Instant::now())
            .unwrap();
        assert_eq!(m["title"].as_str().unwrap().chars().count(), 2000);
        assert_eq!(m["t"], json!(1));
    }

    #[test]
    fn origin_policy() {
        let save = json!({"cmd":"save","url":"https://a.example/x?y"});
        assert!(origin_ok("https://a.example/page", &save));
        assert!(origin_ok(
            "https://A.example:443/",
            &json!({"cmd":"history"})
        ));
        assert!(!origin_ok("https://evil.example/", &save)); // claims another site's url
        assert!(!origin_ok("about:blank", &save)); // content commands need a web page
        assert!(!origin_ok("file:///etc/passwd", &save));
        assert!(!origin_ok(
            "about:blank",
            &json!({"cmd":"dl","url":"http://a/"})
        ));
        assert!(origin_ok("about:blank", &json!({"cmd":"ready"})));
        assert_eq!(
            origin_of("http://user:pw@Host:81/p").as_deref(),
            Some("http://host:81")
        );
        assert_eq!(origin_of("javascript:alert(1)"), None);
    }

    #[test]
    fn secrets_are_random_and_hex() {
        let (a, b) = (Auth::new(), Auth::new());
        assert_eq!(a.key_out.len(), 64);
        assert!(a.key_out.bytes().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a.key_out, b.key_out);
        assert_ne!(a.key_out, a.key_in);
    }

    #[test]
    fn navigation_policy_blocks_dangerous_schemes() {
        for bad in [
            "file:///etc/passwd",
            "FILE:///etc/passwd",
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "ftp://x/",
            "view-source:https://a/",
            "wry://x",
            "about:config",
            "data:application/x-sh,aaa",
            "httpfoo://x",
            "http:/x",
        ] {
            assert!(!nav_allowed(bad), "{bad}");
        }
        for good in [
            "https://a.example/x",
            "HTTP://a.example",
            "about:blank",
            "data:text/html,<p>",
        ] {
            assert!(nav_allowed(good), "{good}");
        }
        assert!(!nav_allowed(&format!("https://a/{}", "x".repeat(MAX_URL))));
        assert!(!is_http("httpx://a/") && !is_http("http:a") && is_http("HtTpS://a/"));
    }

    #[test]
    fn normalize_only_yields_http() {
        assert_eq!(
            normalize("example.com").as_deref(),
            Some("https://example.com")
        );
        assert_eq!(
            normalize("localhost:8080").as_deref(),
            Some("https://localhost:8080")
        );
        assert_eq!(normalize("HTTP://a.b/").as_deref(), Some("HTTP://a.b/"));
        for bad in [
            "file:///a.txt",
            "javascript:alert(1)//a.b",
            "data:text/html,a.b",
            "a.b\nc",
            "a.b\u{0}",
            "two words",
            "",
            "   ",
        ] {
            assert_eq!(normalize(bad), None, "{bad:?}");
        }
        assert_eq!(
            normalize(&format!("https://a.b/{}", "x".repeat(MAX_URL))),
            None
        );
    }

    #[test]
    fn trackers_trailing_dot_and_ipv6() {
        assert_eq!(host_of("https://DoubleClick.net./x"), "doubleclick.net");
        assert!(is_tracker(&host_of(
            "https://user@ads.doubleclick.net.:443/p"
        )));
        assert!(is_tracker(&host_of("https://user:pw@x.hotjar.com/")));
        assert!(!is_tracker(&host_of("https://notdoubleclick.net/")));
        assert!(!is_tracker(&host_of(
            "https://example.com/?u=doubleclick.net"
        )));
        assert_eq!(host_of("http://[::1]:8080/x"), "[::1");
    }

    #[test]
    fn popups_are_throttled() {
        let mut q = VecDeque::new();
        let t = Instant::now();
        let n = (0..20)
            .filter(|_| throttle(&mut q, t, POPUP_MAX, POPUP_WIN))
            .count();
        assert_eq!(n, POPUP_MAX);
        assert!(throttle(
            &mut q,
            t + POPUP_WIN + Duration::from_secs(1),
            POPUP_MAX,
            POPUP_WIN
        ));
    }

    #[test]
    fn redaction_and_truncation_are_char_safe() {
        assert_eq!(
            redact_url("https://u:p@a.example/p/q?token=abc#frag"),
            "https://a.example/p/q"
        );
        assert_eq!(redact_url("https://a.example?x=1"), "https://a.example");
        assert_eq!(redact_url(""), "");
        let long = format!("https://a.example/{}", "é".repeat(500));
        assert!(redact_url(&long).chars().count() < 100);
        assert_eq!(trunc(&"é".repeat(10), 3), "ééé");
    }

    #[test]
    fn file_names_cannot_escape() {
        for (u, want) in [
            ("https://a/x/..", "document"),
            ("https://a/x/../../etc/passwd", "passwd"),
            ("https://a/x\\..\\..\\evil.pdf", "evil.pdf"),
            ("https://a/%2e%2e%2fx.pdf?y=1", "2e2e2fx.pdf"),
            ("https://a/", "document"),
            ("https://a/a b.pdf#f", "a b.pdf"),
        ] {
            assert_eq!(file_name_of(u), want, "{u}");
        }
        let n = file_name_of(&format!("https://a/{}.pdf", "é".repeat(300)));
        assert!(n.chars().count() <= 100 && n.ends_with(".pdf"));
    }

    #[test]
    fn colors_ids_and_local_hosts() {
        assert_eq!(color_of(&json!({"color": 3})), Some(3));
        assert_eq!(color_of(&json!({"color": 257})), None); // no wrap-around into 1..=6
        assert_eq!(color_of(&json!({"color": 0})), None);
        assert_eq!(
            id_of(&json!({"id": u64::MAX})),
            usize::try_from(u64::MAX).unwrap_or(0)
        );
        assert_eq!(id_of(&json!({"id": "x"})), 0);
        for h in [
            "localhost",
            "127.0.0.1",
            "10.0.0.5",
            "192.168.1.1",
            "169.254.169.254",
            "[::1]",
            "::1",
            "fd00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
            "0.0.0.0",
            "x.localhost",
            "printer.local",
        ] {
            assert!(is_local_host(h), "{h}");
        }
        for h in ["example.com", "8.8.8.8", "2606:4700::1111"] {
            assert!(!is_local_host(h), "{h}");
        }
        assert!(download(&test_cfg(), "http://127.0.0.1:1/a.pdf").is_err());
        assert!(download(&test_cfg(), "file:///etc/passwd").is_err());
    }

    fn test_cfg() -> Cfg {
        Cfg {
            home: None,
            gateway: String::new(),
            mock: true,
            selftest: None,
            auth: Mutex::new(BTreeMap::new()),
            offered: Mutex::new(BTreeSet::new()),
            pages: Mutex::new(BTreeMap::new()),
            colors: Mutex::new(BTreeMap::new()),
            trackers: AtomicBool::new(false),
            allow: Mutex::new(BTreeSet::new()),
            popups: Mutex::new(VecDeque::new()),
        }
    }

    #[test]
    fn offered_set_is_bounded_and_forged_ipc_for_unknown_tab_is_dropped() {
        let c = test_cfg();
        for i in 0..1000 {
            remember(&c.offered, &format!("https://a/{i}.pdf"));
        }
        assert!(c.offered.lock().unwrap().len() <= MAX_OFFERED);
        // no Auth entry (closed/asleep tab): nothing authenticates
        assert!(
            authenticate(
                &c,
                7,
                "https://a/",
                "cc1.7.0123456789abcdef01234567.1.x.{\"cmd\":\"save\"}"
            )
            .is_none()
        );
    }
}
