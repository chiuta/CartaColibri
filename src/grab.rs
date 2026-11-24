//! web-grab: bring a web page (or a clipped selection) into the library as clean Markdown text,
//! keeping the source URL so RAG-CARTA citations can point back to the page.
//! 100% local: the page is fetched directly from the site, nothing goes through any third party.

use crate::extract::{decode, html_to_text};
use crate::library::Library;
use regex::Regex;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

const MAX_PAGE: u64 = 20 * 1024 * 1024;

fn title_of(html: &str) -> String {
    Regex::new(r"(?is)<title[^>]*>(.*?)</title>")
        .unwrap()
        .captures(html)
        .map(|c| {
            html_to_text(&c[1])
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

fn slug(s: &str) -> String {
    let t: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let t = Regex::new(r"-+")
        .unwrap()
        .replace_all(&t, "-")
        .trim_matches('-')
        .to_string();
    t.chars().take(60).collect()
}

static ALLOW_LOCAL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Allow grabbing loopback / private addresses (off by default; also `CARTACOLIBRI_GRAB_LOCAL=1`).
pub fn allow_local(on: bool) {
    ALLOW_LOCAL.store(on, std::sync::atomic::Ordering::Relaxed);
}

fn allowed_local() -> bool {
    ALLOW_LOCAL.load(std::sync::atomic::Ordering::Relaxed)
        || std::env::var("CARTACOLIBRI_GRAB_LOCAL").is_ok_and(|v| v == "1")
}

fn bad4(v: std::net::Ipv4Addr) -> bool {
    let o = v.octets();
    o[0] == 0                                   // 0.0.0.0/8 "this network"
        || o[0] == 10
        || (o[0] == 100 && (o[1] & 0xc0) == 64) // 100.64/10 carrier-grade NAT
        || o[0] == 127
        || (o[0] == 169 && o[1] == 254)
        || (o[0] == 172 && (o[1] & 0xf0) == 16)
        || (o[0] == 192 && o[1] == 0 && o[2] == 0)
        || (o[0] == 192 && o[1] == 0 && o[2] == 2)
        || (o[0] == 192 && o[1] == 168)
        || (o[0] == 198 && (o[1] & 0xfe) == 18) // benchmarking
        || (o[0] == 198 && o[1] == 51 && o[2] == 100)
        || (o[0] == 203 && o[1] == 0 && o[2] == 113)
        || o[0] >= 224 // multicast, reserved, broadcast
}

/// True for loopback, private, link-local, multicast, reserved and tunnelled-IPv4 addresses.
pub fn bad_ip(ip: std::net::IpAddr) -> bool {
    use std::net::{IpAddr, Ipv4Addr};
    match ip {
        IpAddr::V4(v) => bad4(v),
        IpAddr::V6(v) => {
            let s = v.segments();
            let embedded = |a: u16, b: u16| {
                bad4(Ipv4Addr::new(
                    (a >> 8) as u8,
                    a as u8,
                    (b >> 8) as u8,
                    b as u8,
                ))
            };
            v.is_unspecified()
                || v.is_loopback()
                || (s[..6].iter().all(|x| *x == 0) && embedded(s[6], s[7])) // ::a.b.c.d
                || (s[..5].iter().all(|x| *x == 0) && s[5] == 0xffff && embedded(s[6], s[7])) // ::ffff:a.b.c.d
                || (s[0] == 0x64 && s[1] == 0xff9b && s[2..6].iter().all(|x| *x == 0) && embedded(s[6], s[7])) // NAT64
                || s[0] == 0x2002 && embedded(s[1], s[2]) // 6to4
                || (s[0] == 0x2001 && s[1] == 0) // Teredo
                || (s[0] == 0x2001 && s[1] == 0xdb8) // documentation
                || (s[0] & 0xfe00) == 0xfc00 // unique local
                || (s[0] & 0xffc0) == 0xfe80 // link-local
                || (s[0] & 0xffc0) == 0xfec0 // site-local
                || (s[0] & 0xff00) == 0xff00 // multicast
        }
    }
}

/// Splits an http(s) URL with the same parser the HTTP client uses, so the address that is
/// checked is the address that is fetched. Userinfo, backslashes and control characters are refused.
fn parse_target(url: &str) -> Option<(String, u16, String)> {
    if url
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return None;
    }
    let uri: ureq::http::Uri = url.parse().ok()?;
    let scheme = uri.scheme_str()?.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let auth = uri.authority()?;
    if auth.as_str().contains('@') {
        return None;
    }
    let port = auth
        .port_u16()
        .unwrap_or(if scheme == "https" { 443 } else { 80 });
    let host = auth.host().trim_start_matches('[').trim_end_matches(']');
    if host.is_empty() || host.contains('%') {
        return None;
    }
    Some((host.to_string(), port, scheme))
}

/// True when the address behind `url` is this machine or a private network (or cannot be checked).
pub fn blocked(url: &str) -> bool {
    use std::net::ToSocketAddrs;
    if allowed_local() {
        return false;
    }
    let Some((host, port, _)) = parse_target(url) else {
        return true;
    };
    (host.as_str(), port)
        .to_socket_addrs()
        .map(|mut a| a.any(|s| bad_ip(s.ip())))
        .unwrap_or(true)
}

/// Resolver that refuses private answers at connect time, closing the DNS-rebinding gap between
/// the pre-check and the real connection.
#[derive(Debug)]
struct GuardedResolver;

impl ureq::unversioned::resolver::Resolver for GuardedResolver {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        config: &ureq::config::Config,
        timeout: ureq::unversioned::transport::NextTimeout,
    ) -> Result<ureq::unversioned::resolver::ResolvedSocketAddrs, ureq::Error> {
        use ureq::unversioned::resolver::DefaultResolver;
        let found = DefaultResolver::default().resolve(uri, config, timeout)?;
        if !allowed_local() && config.proxy().is_none() && found.iter().any(|a| bad_ip(a.ip())) {
            return Err(ureq::Error::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "address is on this machine or a private network",
            )));
        }
        Ok(found)
    }
}

/// Resolves a redirect target against the URL it came from.
fn absolute(base: &str, loc: &str) -> Option<String> {
    let loc = loc.trim();
    let lower = loc.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return Some(loc.to_string());
    }
    let (scheme, rest) = base.split_once("://")?;
    if let Some(r) = loc.strip_prefix("//") {
        return Some(format!("{scheme}://{r}"));
    }
    let auth_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (auth, path) = rest.split_at(auth_end);
    if loc.starts_with('/') {
        return Some(format!("{scheme}://{auth}{loc}"));
    }
    if loc.is_empty() || loc.contains("://") || loc.starts_with('#') {
        return None;
    }
    let path = path.split(['?', '#']).next().unwrap_or("");
    let dir = path.rsplit_once('/').map_or("/", |x| &path[..=x.0.len()]);
    Some(format!("{scheme}://{auth}{dir}{loc}"))
}

/// Guarded download: http(s) only, public addresses only (checked before every hop and again at
/// connect time), redirects followed by hand, at most `max` bytes (+1 so callers can detect overflow).
/// Returns the body and the lower-cased content type.
pub fn fetch_raw(
    url: &str,
    max: u64,
    secs: u64,
    agent_name: &str,
) -> Result<(Vec<u8>, String), String> {
    fetch_raw_final(url, max, secs, agent_name).map(|(b, c, _)| (b, c))
}

/// Like [`fetch_raw`], also returning the final address after redirects.
fn fetch_raw_final(
    url: &str,
    max: u64,
    secs: u64,
    agent_name: &str,
) -> Result<(Vec<u8>, String, String), String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("only http:// and https:// addresses can be grabbed".into());
    }
    let cfg = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(secs)))
        .max_redirects(0)
        .http_status_as_error(false)
        .user_agent(agent_name)
        .build();
    let agent = ureq::Agent::with_parts(
        cfg,
        ureq::unversioned::transport::DefaultConnector::default(),
        GuardedResolver,
    );
    let mut cur = url.to_string();
    let mut r = None;
    for _ in 0..6 {
        if blocked(&cur) {
            return Err("that address is on this machine or a private network; only public pages are fetched (set CARTACOLIBRI_GRAB_LOCAL=1 to allow it)".to_string());
        }
        let resp = agent.get(&cur).call().map_err(|_| {
            "cannot fetch that address (no answer, or it is not reachable)".to_string()
        })?;
        let code = resp.status().as_u16();
        if (300..400).contains(&code) {
            let loc = resp
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            cur = absolute(&cur, loc).ok_or_else(|| "bad redirect".to_string())?;
            continue;
        }
        if code >= 400 {
            return Err(format!("cannot fetch that address: HTTP {code}"));
        }
        r = Some(resp);
        break;
    }
    let mut r = r.ok_or_else(|| "too many redirects".to_string())?;
    let ctype = r
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    let mut raw = Vec::new();
    r.body_mut()
        .as_reader()
        .take(max.saturating_add(1))
        .read_to_end(&mut raw)
        .map_err(|_| "the page stopped answering".to_string())?;
    Ok((raw, ctype, cur))
}

/// Fetch a page and return (title, text). Loopback and private addresses are refused (also after redirects).
pub fn fetch(url: &str) -> Result<(String, String), String> {
    let (mut raw, ctype) = fetch_raw(
        url,
        MAX_PAGE,
        60,
        "CartaColibri-web-grab/0.2 (+local; https://github.com/chiuta/CartaColibri)",
    )?;
    raw.truncate(MAX_PAGE as usize);
    let body = decode(&raw);
    if ctype.contains("html") || body.trim_start().starts_with('<') {
        Ok((title_of(&body), html_to_text(&body)))
    } else {
        Ok((String::new(), body))
    }
}

static LOCK: Mutex<()> = Mutex::new(());
const HL_MARK: &str = "\n\n## Highlights\n";

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn norm_hash(text: &str) -> String {
    use sha1::{Digest, Sha1};
    let n = text.split_whitespace().collect::<Vec<_>>().join(" ");
    hex::encode(Sha1::digest(n.as_bytes()))
}

fn url_key(url: &str) -> String {
    use sha1::{Digest, Sha1};
    hex::encode(Sha1::digest(url.as_bytes()))[..16].to_string()
}

fn read_json<T: serde::de::DeserializeOwned + Default>(p: &Path) -> T {
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_json<T: serde::Serialize>(p: &Path, v: &T) {
    let tmp = p.with_extension("json.tmp");
    if std::fs::write(&tmp, serde_json::to_string_pretty(v).unwrap_or_default()).is_ok() {
        let _ = std::fs::rename(&tmp, p);
    }
}

fn load_web(home: &Path) -> BTreeMap<String, Value> {
    read_json(&home.join("web.json"))
}

fn saved_text(path: &str) -> String {
    let md = std::fs::read_to_string(path).unwrap_or_default();
    let body = md
        .split_once("\nSource: ")
        .and_then(|(_, r)| r.split_once("\n\n"))
        .map(|(_, b)| b)
        .unwrap_or(&md);
    body.split(HL_MARK).next().unwrap_or("").to_string()
}

/// Lines that appeared (+) or disappeared (-) between the saved copy and the live page.
pub fn diff(saved: &str, live: &str) -> Vec<String> {
    let lines = |t: &str| -> Vec<String> {
        t.lines()
            .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|l| l.chars().count() >= 4)
            .collect()
    };
    let (a, b) = (lines(saved), lines(live));
    let (sa, sb): (
        std::collections::HashSet<&String>,
        std::collections::HashSet<&String>,
    ) = (a.iter().collect(), b.iter().collect());
    let cut = |l: &String| l.chars().take(160).collect::<String>();
    let mut out: Vec<String> = b
        .iter()
        .filter(|l| !sa.contains(l))
        .take(12)
        .map(|l| format!("+ {}", cut(l)))
        .collect();
    out.extend(
        a.iter()
            .filter(|l| !sb.contains(l))
            .take(12)
            .map(|l| format!("- {}", cut(l))),
    );
    out
}

/// Is this address already in the library, and does the page differ from the saved copy (and how)?
pub fn status(home: &Path, url: &str, text: &str) -> Option<Value> {
    let e = load_web(home).get(url).cloned()?;
    let path = e["path"].as_str()?.to_string();
    if !Path::new(&path).is_file() {
        return None;
    }
    let changed = !text.trim().is_empty() && e["hash"].as_str() != Some(norm_hash(text).as_str());
    let d = if changed {
        diff(&saved_text(&path), text)
    } else {
        vec![]
    };
    Some(
        json!({"path": path, "saved_at": e["saved_at"], "changed": changed, "diff": d, "snapshot": e["snapshot"]}),
    )
}

pub fn highlights(home: &Path, url: &str) -> Vec<String> {
    let m: BTreeMap<String, Vec<String>> = read_json(&home.join("highlights.json"));
    m.get(url).cloned().unwrap_or_default()
}

fn hl_section(items: &[String]) -> String {
    if items.is_empty() {
        String::new()
    } else {
        format!(
            "{HL_MARK}{}\n",
            items
                .iter()
                .map(|h| format!("- {}", h.replace('\n', " ")))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// Remember a highlighted passage of a saved page; it is appended to the page's library file (so it is searchable).
pub fn highlight_add(lib: &mut Library, url: &str, sel: &str) -> Result<usize, String> {
    let sel: String = sel
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(2000)
        .collect();
    if sel.chars().count() < 3 {
        return Err("select some text first".into());
    }
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = load_web(&lib.home)
        .get(url)
        .and_then(|e| e["path"].as_str().map(String::from))
        .filter(|p| Path::new(p).is_file())
        .ok_or("save the page first (💾)")?;
    let hp = lib.home.join("highlights.json");
    let mut m: BTreeMap<String, Vec<String>> = read_json(&hp);
    let list = m.entry(url.to_string()).or_default();
    if !list.contains(&sel) {
        list.push(sel);
    }
    let n = list.len();
    let items = list.clone();
    write_json(&hp, &m);
    let md = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let base = md
        .split(HL_MARK)
        .next()
        .unwrap_or("")
        .trim_end()
        .to_string();
    std::fs::write(&path, format!("{base}\n{}", hl_section(&items))).map_err(|e| e.to_string())?;
    let dir = lib.upload_dir()?.to_string_lossy().into_owned();
    lib.update(Some(&[dir]), &mut |_| {})?;
    Ok(n)
}

/// Save a page/clip as Markdown inside the library uploads and index it.
/// Saving an address that is already in the library refreshes that file instead of adding a copy
/// (its highlights are kept).
pub fn save(lib: &mut Library, url: &str, title: &str, text: &str) -> Result<Value, String> {
    save_with(lib, url, title, text, None)
}

/// Like [`save`], plus an optional HTML snapshot ("the page as you saw it") kept outside the index.
pub fn save_with(
    lib: &mut Library,
    url: &str,
    title: &str,
    text: &str,
    html: Option<&str>,
) -> Result<Value, String> {
    // one line, no control characters: the title and address become Markdown header lines
    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = title.as_str();
    if url.chars().any(|c| c.is_control()) {
        return Err("that is not a web address".into());
    }
    let clean = Regex::new(r"\n{3,}")
        .unwrap()
        .replace_all(
            &Regex::new(r"[ \t]+").unwrap().replace_all(text, " "),
            "\n\n",
        )
        .trim()
        .to_string();
    if clean.chars().count() < 20 {
        return Err("the page has no readable text".into());
    }
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let name = format!("{}.md", slug(if title.is_empty() { url } else { title }));
    let doc = format!(
        "# {}\n\nSource: {url}\n\n{clean}\n{}",
        if title.is_empty() { url } else { title },
        hl_section(&highlights(&lib.home, url))
    );
    let mut web = load_web(&lib.home);
    let known = web
        .get(url)
        .and_then(|e| e["path"].as_str())
        .map(String::from)
        .filter(|p| Path::new(p).is_file());
    let r = match known {
        Some(p) => {
            std::fs::write(&p, doc.as_bytes()).map_err(|e| e.to_string())?;
            let dir = lib.upload_dir()?.to_string_lossy().into_owned();
            let st = lib.update(Some(&[dir]), &mut |_| {})?;
            json!({"saved": p, "index": st})
        }
        None => lib.add_upload(&name, doc.as_bytes(), &[])?,
    };
    let mut entry = json!({"path": r["saved"], "saved_at": now(), "hash": norm_hash(&clean)});
    if let Some(h) = html.filter(|h| !h.is_empty() && h.len() <= 3 * 1024 * 1024) {
        let d = lib.home.join("snapshots");
        std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
        let sp = d.join(format!("{}.html", url_key(url)));
        std::fs::write(&sp, h).map_err(|e| e.to_string())?;
        entry["snapshot"] = json!(sp);
    } else if let Some(old) = web.get(url).and_then(|e| e.get("snapshot")) {
        entry["snapshot"] = old.clone();
    }
    web.insert(url.to_string(), entry);
    write_json(&lib.home.join("web.json"), &web);
    Ok(
        json!({"title": title, "url": url, "chars": clean.chars().count(), "saved": r["saved"], "index": r["index"]}),
    )
}

pub fn grab(lib: &mut Library, url: &str) -> Result<Value, String> {
    let (title, text) = fetch(url)?;
    save(lib, url, &title, &text)
}

// ------------------------------------------------------------------ searchable history (opt-in)

/// The history file of an address (it exists only when history is on).
pub fn history_path(home: &Path, url: &str) -> std::path::PathBuf {
    home.join("library")
        .join("history")
        .join(format!("{}.md", url_key(url)))
}

pub fn history_enabled(home: &Path) -> bool {
    let m: BTreeMap<String, Value> = read_json(&home.join("settings.json"));
    m.get("history").and_then(Value::as_bool).unwrap_or(false)
}

pub fn set_history(home: &Path, on: bool) {
    let p = home.join("settings.json");
    let mut m: BTreeMap<String, Value> = read_json(&p);
    m.insert("history".into(), json!(on));
    write_json(&p, &m);
}

/// Keep the text of a visited page so RAG-fetch can search your browsing history. One file per address.
pub fn history_add(lib: &mut Library, url: &str, title: &str, text: &str) -> Result<(), String> {
    if !url.starts_with("http") || text.trim().chars().count() < 20 {
        return Ok(());
    }
    let dir = lib.home.join("library").join("history");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let body: String = text.chars().take(200_000).collect();
    let doc = format!(
        "# {}\n\nSource: {url}\nVisited: {}\n\n{body}\n",
        if title.is_empty() { url } else { title },
        now()
    );
    let p = dir.join(format!("{}.md", url_key(url)));
    std::fs::write(&p, doc).map_err(|e| e.to_string())?;
    lib.update(Some(&[dir.to_string_lossy().into_owned()]), &mut |_| {})?;
    Ok(())
}

pub fn history_clear(lib: &Library) -> Result<usize, String> {
    let dir = lib.home.join("library").join("history");
    let n = lib.remove_root(&dir.to_string_lossy())?;
    let _ = std::fs::remove_dir_all(&dir);
    Ok(n)
}

// ------------------------------------------------------------------ watched pages

pub fn watch_list(home: &Path) -> Vec<String> {
    let m: BTreeMap<String, Vec<String>> = read_json(&home.join("watch.json"));
    m.get("urls").cloned().unwrap_or_default()
}

pub fn watch_set(home: &Path, urls: Vec<String>) {
    let mut m = BTreeMap::new();
    m.insert("urls".to_string(), urls);
    write_json(&home.join("watch.json"), &m);
}

/// Re-fetch every watched page and report what changed against the saved copy (`refresh` also updates the copy).
pub fn watch_check(lib: &mut Library, refresh: bool) -> Vec<Value> {
    let mut out = vec![];
    for url in watch_list(&lib.home) {
        match fetch(&url) {
            Err(e) => out.push(json!({"url": url, "state": "error", "error": e})),
            Ok((title, text)) => match status(&lib.home, &url, &text) {
                None => out.push(json!({"url": url, "state": "not-saved"})),
                Some(st) if st["changed"] == true => {
                    if refresh {
                        let _ = save(lib, &url, &title, &text);
                    }
                    out.push(json!({"url": url, "state": "changed", "diff": st["diff"], "refreshed": refresh}));
                }
                Some(_) => out.push(json!({"url": url, "state": "same"})),
            },
        }
    }
    out
}

// ------------------------------------------------------------------ save_page: a whole page as one file

macro_rules! rx {
    ($p:expr) => {{
        static R: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        R.get_or_init(|| Regex::new($p).unwrap())
    }};
}

const PAGE_AGENT: &str =
    "CartaColibri-web-grab/0.2 (+local; https://github.com/chiuta/CartaColibri)";
/// Sub-resources fetched for one page, bytes per resource, bytes in all, seconds in all.
const PAGE_MAX_RES: usize = 120;
const PAGE_MAX_ASSET: u64 = 5 * 1024 * 1024;
const PAGE_MAX_CSS: u64 = 2 * 1024 * 1024;
const PAGE_MAX_TOTAL: u64 = 30 * 1024 * 1024;
const PAGE_MAX_OUT: usize = 64 * 1024 * 1024;
const PAGE_SECS: u64 = 90;
const CSS_MAX_DEPTH: usize = 3;

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_encode(d: &[u8]) -> String {
    let mut o = String::with_capacity(d.len().div_ceil(3) * 4);
    for c in d.chunks(3) {
        let n = (c[0] as u32) << 16
            | (*c.get(1).unwrap_or(&0) as u32) << 8
            | *c.get(2).unwrap_or(&0) as u32;
        o.push(B64[(n >> 18) as usize & 63] as char);
        o.push(B64[(n >> 12) as usize & 63] as char);
        o.push(if c.len() > 1 {
            B64[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        o.push(if c.len() > 2 {
            B64[n as usize & 63] as char
        } else {
            '='
        });
    }
    o
}

fn b64_lines(d: &[u8]) -> String {
    let e = b64_encode(d);
    let mut o = String::with_capacity(e.len() + e.len() / 38 * 2 + 2);
    for l in e.as_bytes().chunks(76) {
        o.push_str(std::str::from_utf8(l).unwrap_or(""));
        o.push_str("\r\n");
    }
    o
}

/// Quoted-printable (RFC 2045): lines under 77 characters, hard breaks as CRLF.
fn qp_encode(d: &[u8]) -> String {
    let mut o = String::with_capacity(d.len() + d.len() / 20);
    let mut col = 0usize;
    let n = d.len();
    let mut i = 0;
    while i < n {
        let b = d[i];
        if b == b'\n' || (b == b'\r' && d.get(i + 1) == Some(&b'\n')) {
            if b == b'\r' {
                i += 1;
            }
            o.push_str("\r\n");
            col = 0;
            i += 1;
            continue;
        }
        let at_eol = i + 1 == n || d[i + 1] == b'\n' || d[i + 1] == b'\r';
        let plain = (33..=126).contains(&b) && b != b'=' || ((b == b' ' || b == b'\t') && !at_eol);
        let tok = if plain {
            format!("{}", b as char)
        } else {
            format!("={b:02X}")
        };
        if col + tok.len() > 75 {
            o.push_str("=\r\n");
            col = 0;
        }
        o.push_str(&tok);
        col += tok.len();
        i += 1;
    }
    o
}

fn unescape_attr(v: &str) -> String {
    if !v.contains('&') {
        return v.to_string();
    }
    rx!(r"&(#[0-9]{1,7}|#[xX][0-9a-fA-F]{1,6}|[a-zA-Z]{2,8});")
        .replace_all(v, |c: &regex::Captures| {
            let n = &c[1];
            let ch = match n {
                "amp" => Some('&'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "nbsp" => Some(' '),
                _ => n.strip_prefix('#').and_then(|d| {
                    match d.strip_prefix(['x', 'X']) {
                        Some(h) => u32::from_str_radix(h, 16).ok(),
                        None => d.parse().ok(),
                    }
                    .and_then(char::from_u32)
                }),
            };
            ch.map(String::from).unwrap_or_else(|| c[0].to_string())
        })
        .into_owned()
}

fn esc_attr(v: &str) -> String {
    v.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Resolves `loc` against `base` (RFC 3986 style, dot segments removed). Only http(s) results;
/// anything else (data:, javascript:, mailto:, ...) is `None`. The fragment is kept.
fn resolve_url(base: &str, loc: &str) -> Option<String> {
    let loc: String = loc
        .trim()
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();
    if loc.is_empty() || loc.len() > 8192 {
        return None;
    }
    let (bscheme, brest) = base.split_once("://")?;
    let lower = loc.to_ascii_lowercase();
    let has_scheme = rx!(r"^[a-z][a-z0-9+.\-]*:").is_match(&lower);
    let (scheme, auth, tail): (String, String, String);
    if has_scheme {
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return None;
        }
        let (s, r) = loc.split_once("://")?;
        let e = r.find(['/', '?', '#']).unwrap_or(r.len());
        scheme = s.to_ascii_lowercase();
        auth = r[..e].to_string();
        tail = r[e..].to_string();
    } else if let Some(r) = loc.strip_prefix("//") {
        let e = r.find(['/', '?', '#']).unwrap_or(r.len());
        scheme = bscheme.to_string();
        auth = r[..e].to_string();
        tail = r[e..].to_string();
    } else {
        let e = brest.find(['/', '?', '#']).unwrap_or(brest.len());
        scheme = bscheme.to_string();
        auth = brest[..e].to_string();
        let bpath_q = brest[e..].split('#').next().unwrap_or("");
        let (bpath, bq) = match bpath_q.split_once('?') {
            Some((p, q)) => (p, Some(q)),
            None => (bpath_q, None),
        };
        tail = if loc.starts_with('/') {
            loc.clone()
        } else if loc.starts_with('#') {
            format!("{bpath_q}{loc}")
        } else if loc.starts_with('?') {
            format!("{}{loc}", if bpath.is_empty() { "/" } else { bpath })
        } else {
            let _ = bq;
            let dir = bpath.rsplit_once('/').map_or("/", |x| &bpath[..=x.0.len()]);
            format!("{dir}{loc}")
        };
    }
    let (pq, frag) = match tail.split_once('#') {
        Some((a, b)) => (a, Some(b)),
        None => (tail.as_str(), None),
    };
    let (path, query) = match pq.split_once('?') {
        Some((a, b)) => (a, Some(b)),
        None => (pq, None),
    };
    let mut segs: Vec<&str> = Vec::new();
    let parts: Vec<&str> = path.split('/').collect();
    for (i, seg) in parts.iter().enumerate() {
        match *seg {
            "." => {}
            ".." => {
                segs.pop();
            }
            "" if i == 0 => {}
            s => segs.push(s),
        }
    }
    let last_dir = matches!(parts.last(), Some(&"" | &"." | &".."));
    let mut np = format!("/{}", segs.join("/"));
    if last_dir && !np.ends_with('/') {
        np.push('/');
    }
    let mut out = format!("{scheme}://{auth}{np}");
    if let Some(q) = query {
        out.push('?');
        out.push_str(q);
    }
    if let Some(f) = frag {
        out.push('#');
        out.push_str(f);
    }
    let mut enc = String::with_capacity(out.len());
    for b in out.bytes() {
        if b <= 0x20
            || b >= 0x7f
            || matches!(
                b,
                b'"' | b'\'' | b'<' | b'>' | b'\\' | b'`' | b'^' | b'|' | b'{' | b'}'
            )
        {
            enc.push_str(&format!("%{b:02X}"));
        } else {
            enc.push(b as char);
        }
    }
    Some(enc)
}

fn is_private_host(host: &str, port: u16) -> bool {
    use std::net::ToSocketAddrs;
    (host, port)
        .to_socket_addrs()
        .map(|mut a| a.any(|s| bad_ip(s.ip())))
        .unwrap_or(true)
}

fn sniff_mime(data: &[u8], ctype: &str, url: &str) -> String {
    let head = String::from_utf8_lossy(&data[..data.len().min(512)]).to_lowercase();
    if data.starts_with(b"\x89PNG") {
        return "image/png".into();
    }
    if data.starts_with(&[0xff, 0xd8, 0xff]) {
        return "image/jpeg".into();
    }
    if data.starts_with(b"GIF8") {
        return "image/gif".into();
    }
    if data.len() > 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return "image/webp".into();
    }
    if data.len() > 12
        && &data[4..8] == b"ftyp"
        && (&data[8..12] == b"avif" || &data[8..12] == b"avis")
    {
        return "image/avif".into();
    }
    if data.starts_with(&[0, 0, 1, 0]) {
        return "image/x-icon".into();
    }
    if data.starts_with(b"wOFF") {
        return "font/woff".into();
    }
    if data.starts_with(b"wOF2") {
        return "font/woff2".into();
    }
    if head.contains("<svg") {
        return "image/svg+xml".into();
    }
    let ct = ctype.split(';').next().unwrap_or("").trim();
    if !ct.is_empty() && ct != "application/octet-stream" && ct != "binary/octet-stream" {
        return ct.to_string();
    }
    let path = url.split(['?', '#']).next().unwrap_or("").to_lowercase();
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "css" => "text/css",
        _ => "application/octet-stream",
    }
    .into()
}

/// Bytes -> text using a BOM, an explicit charset, or a `<meta charset>` / `@charset`; else UTF-8 / Windows-1250.
fn decode_with(raw: &[u8], ctype: &str) -> String {
    let utf8 = raw.starts_with(&[0xef, 0xbb, 0xbf])
        || raw.starts_with(&[0xff, 0xfe])
        || raw.starts_with(&[0xfe, 0xff]);
    if !utf8 && std::str::from_utf8(raw).is_err() {
        let head = String::from_utf8_lossy(&raw[..raw.len().min(4096)]).into_owned();
        let label = rx!(r#"(?i)charset\s*=\s*["']?([A-Za-z0-9_\-:.]+)"#)
            .captures(ctype)
            .or_else(|| rx!(r#"(?i)charset\s*=\s*["']?([A-Za-z0-9_\-:.]+)"#).captures(&head))
            .or_else(|| rx!(r#"(?i)@charset\s+["']([A-Za-z0-9_\-:.]+)"#).captures(&head))
            .map(|c| c[1].to_string());
        if let Some(l) = label
            && let Some(enc) = encoding_rs::Encoding::for_label(l.as_bytes())
        {
            return enc.decode(raw).0.into_owned();
        }
    }
    decode(raw)
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    /// one self-contained .html (data: URIs)
    Inline,
    /// .mhtml: references stay absolute, resources travel as MIME parts
    Archive,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Image,
    Other,
}

struct Ctx {
    mode: Mode,
    origin: Option<(String, u16, String)>,
    deadline: std::time::Instant,
    count: usize,
    bytes: u64,
    emitted: usize,
    skipped: usize,
    blocked_private: usize,
    refs: BTreeMap<String, Option<String>>,
    css: BTreeMap<String, Option<String>>,
    /// archive mode: (url, mime, bytes) of every embedded resource
    parts: Vec<(String, String, Vec<u8>)>,
}

impl Ctx {
    fn left(&self) -> u64 {
        self.deadline
            .saturating_duration_since(std::time::Instant::now())
            .as_secs()
    }

    /// Guarded sub-resource download. Public hosts are fine; a private host only when local grabbing
    /// is on AND it is the same origin as the page itself.
    fn fetch(&mut self, url: &str, max: u64) -> Option<(Vec<u8>, String)> {
        let url = url.split('#').next().unwrap_or("");
        if self.count >= PAGE_MAX_RES || self.bytes >= PAGE_MAX_TOTAL || self.left() < 2 {
            self.skipped += 1;
            return None;
        }
        let Some(t) = parse_target(url) else {
            self.skipped += 1;
            return None;
        };
        if is_private_host(&t.0, t.1) && !(allowed_local() && self.origin.as_ref() == Some(&t)) {
            self.blocked_private += 1;
            return None;
        }
        self.count += 1;
        match fetch_raw(url, max, self.left().min(20), PAGE_AGENT) {
            Ok((d, c)) if d.len() as u64 <= max => {
                self.bytes += d.len() as u64;
                Some((d, c))
            }
            _ => {
                self.skipped += 1;
                None
            }
        }
    }

    /// The text to put where a binary resource was referenced: a data: URI (inline) or the absolute
    /// address (archive, where the bytes travel as a MIME part). `None` = could not be embedded.
    fn asset(&mut self, url: &str, kind: Kind) -> Option<String> {
        let url = url.split('#').next().unwrap_or("").to_string();
        if let Some(r) = self.refs.get(&url) {
            if let (Some(s), Mode::Inline) = (r, self.mode) {
                self.emitted += s.len();
            }
            return r.clone();
        }
        let got = self.fetch(&url, PAGE_MAX_ASSET).and_then(|(d, c)| {
            let mime = sniff_mime(&d, &c, &url);
            let ok = match kind {
                Kind::Image => mime.starts_with("image/"),
                Kind::Other => {
                    !(mime.starts_with("text/html")
                        || mime.contains("javascript")
                        || mime.contains("ecmascript"))
                }
            };
            ok.then_some((d, mime))
        });
        let r = match got {
            Some((d, mime)) if self.emitted + d.len() * 4 / 3 < PAGE_MAX_OUT => {
                if self.mode == Mode::Inline {
                    let s = format!("data:{mime};base64,{}", b64_encode(&d));
                    self.emitted += s.len();
                    Some(s)
                } else {
                    self.parts.push((url.clone(), mime, d));
                    Some(url.clone())
                }
            }
            _ => {
                self.skipped += 1;
                None
            }
        };
        self.refs.insert(url, r.clone());
        r
    }

    /// A stylesheet, fully rewritten (nested urls and imports embedded).
    fn stylesheet(&mut self, url: &str, depth: usize) -> Option<String> {
        let url = url.split('#').next().unwrap_or("").to_string();
        if let Some(r) = self.css.get(&url) {
            return r.clone();
        }
        self.css.insert(url.clone(), None); // import loops end here
        let r = self.fetch(&url, PAGE_MAX_CSS).and_then(|(d, c)| {
            let ct = c.split(';').next().unwrap_or("").trim().to_string();
            if ct.contains("html") || d.starts_with(b"<") {
                return None;
            }
            let text = decode_with(&d, &c);
            Some(self.css_rewrite(&text, &url, depth + 1))
        });
        if let (Some(css), Mode::Archive) = (&r, self.mode) {
            self.parts
                .push((url.clone(), "text/css".into(), css.clone().into_bytes()));
        }
        self.css.insert(url, r.clone());
        r
    }

    /// Embeds `url(...)` and `@import` targets, removes legacy script hooks, makes the text safe inside `<style>`.
    fn css_rewrite(&mut self, css: &str, base: &str, depth: usize) -> String {
        let css = rx!(r"(?s)/\*.*?\*/").replace_all(css, "").into_owned();
        let css = rx!(r#"(?is)@import\s*(?:url\(\s*(?:"([^"]*)"|'([^']*)'|([^)\s]*))\s*\)|"([^"]*)"|'([^']*)')\s*([^;{}]*);"#)
            .replace_all(&css, |c: &regex::Captures| {
                let target = (1..=5).find_map(|i| c.get(i)).map(|m| m.as_str()).unwrap_or("");
                let media = c[6].trim().to_string();
                let Some(u) = resolve_url(base, target) else {
                    return String::new();
                };
                if depth >= CSS_MAX_DEPTH {
                    return String::new();
                }
                let Some(inner) = self.stylesheet(&u, depth) else {
                    return String::new();
                };
                match self.mode {
                    Mode::Archive => format!("@import url(\"{u}\"){}{};", if media.is_empty() { "" } else { " " }, media),
                    Mode::Inline => wrap_media(&inner, &media),
                }
            })
            .into_owned();
        let css = rx!(r#"(?i)url\(\s*(?:"([^"]*)"|'([^']*)'|([^)\s"']*))\s*\)"#)
            .replace_all(&css, |c: &regex::Captures| {
                let v = (1..=3)
                    .find_map(|i| c.get(i))
                    .map(|m| m.as_str().trim())
                    .unwrap_or("");
                if v.to_ascii_lowercase().starts_with("data:")
                    && !v.to_ascii_lowercase().starts_with("data:text/html")
                {
                    return format!("url(\"{}\")", v.replace('"', "%22"));
                }
                if v.starts_with('#') {
                    return c[0].to_string();
                }
                let u = resolve_url(base, v);
                if let Some(u) = u.as_ref()
                    && self
                        .css
                        .get(u.split('#').next().unwrap_or(""))
                        .is_some_and(|x| x.is_some())
                {
                    return format!("url(\"{u}\")"); // an archived @import target
                }
                match u.and_then(|u| self.asset(&u, Kind::Other)) {
                    Some(r) => format!("url(\"{r}\")"),
                    None => "url(\"data:,\")".to_string(),
                }
            })
            .into_owned();
        let css = rx!(r"(?i)expression\s*\(|-moz-binding|behavior\s*:|javascript\s*:")
            .replace_all(&css, "x-removed:");
        css.replace('<', "\\3c ")
    }
}

/// `@media` wrapper for a stylesheet that only applies to some media.
fn wrap_media(css: &str, media: &str) -> String {
    let m: String = media
        .chars()
        .filter(|c| {
            c.is_alphanumeric()
                || matches!(
                    c,
                    ' ' | ',' | '(' | ')' | '-' | ':' | '.' | '/' | '<' | '>' | '='
                )
        })
        .collect();
    let m = m.trim();
    if m.is_empty() || m.eq_ignore_ascii_case("all") || m.eq_ignore_ascii_case("screen") {
        css.to_string()
    } else {
        format!("@media {m}{{{css}}}")
    }
}

type Attrs = Vec<(String, Option<String>)>;

fn parse_attrs(src: &str) -> Attrs {
    rx!(r#"([^\s"'<>/=]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>]+)))?"#)
        .captures_iter(src)
        .take(200)
        .map(|c| {
            let v = (2..=4)
                .find_map(|i| c.get(i))
                .map(|m| unescape_attr(m.as_str()));
            (c[1].to_ascii_lowercase(), v)
        })
        .collect()
}

fn aget<'a>(a: &'a Attrs, k: &str) -> Option<&'a str> {
    a.iter()
        .find(|(n, _)| n == k)
        .and_then(|(_, v)| v.as_deref())
}

fn aset(a: &mut Attrs, k: &str, v: String) {
    match a.iter_mut().find(|(n, _)| n == k) {
        Some(x) => x.1 = Some(v),
        None => a.push((k.to_string(), Some(v))),
    }
}

fn adel(a: &mut Attrs, k: &str) {
    a.retain(|(n, _)| n != k);
}

fn first_srcset(v: &str) -> Option<String> {
    v.split(',')
        .map(|c| c.split_whitespace().next().unwrap_or("").to_string())
        .find(|u| !u.is_empty())
}

fn script_url(v: &str) -> bool {
    let t: String = v
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect::<String>()
        .to_ascii_lowercase();
    t.starts_with("javascript:")
        || t.starts_with("vbscript:")
        || t.starts_with("data:text/html")
        || t.starts_with("data:application/xhtml")
}

fn build_tag(name: &str, a: &Attrs, selfclose: bool) -> String {
    let mut o = format!("<{name}");
    for (k, v) in a {
        match v {
            Some(v) => o.push_str(&format!(" {k}=\"{}\"", esc_attr(v))),
            None => o.push_str(&format!(" {k}")),
        }
    }
    if selfclose {
        o.push_str(" /");
    }
    o.push('>');
    o
}

impl Ctx {
    fn reference(&mut self, base: &str, v: &str, kind: Kind) -> String {
        resolve_url(base, v)
            .and_then(|u| self.asset(&u, kind))
            .unwrap_or_else(|| "data:,".to_string())
    }

    /// Rewrites one start tag; `None` removes it.
    fn tag(&mut self, name: &str, src: &str, base: &str) -> Option<String> {
        let selfclose = src.trim_end().ends_with('/');
        let mut a = parse_attrs(src);
        a.retain(|(k, v)| {
            !(k.starts_with("on")
                || matches!(
                    k.as_str(),
                    "srcdoc"
                        | "ping"
                        | "integrity"
                        | "crossorigin"
                        | "nonce"
                        | "referrerpolicy"
                        | "formaction"
                        | "manifest"
                        | "background"
                        | "dynsrc"
                        | "lowsrc"
                )
                || (matches!(
                    k.as_str(),
                    "href"
                        | "src"
                        | "xlink:href"
                        | "action"
                        | "poster"
                        | "data"
                        | "cite"
                        | "longdesc"
                ) && v.as_deref().is_some_and(script_url)))
        });
        if let Some(st) = aget(&a, "style").map(String::from) {
            let s = self.css_rewrite(&st, base, CSS_MAX_DEPTH);
            aset(&mut a, "style", s);
        }
        match name {
            "base" | "source" | "track" | "param" | "embed" | "portal" | "script" | "iframe"
            | "frame" | "object" | "applet" => None,
            "link" => {
                let rel = aget(&a, "rel").unwrap_or("").to_ascii_lowercase();
                let toks: Vec<&str> = rel.split_whitespace().collect();
                let href = aget(&a, "href").map(String::from)?;
                if toks.contains(&"stylesheet") && !toks.contains(&"alternate") {
                    let u = resolve_url(base, &href)?;
                    let css = self.stylesheet(&u, 0)?;
                    let media = aget(&a, "media").unwrap_or("").to_string();
                    return Some(match self.mode {
                        Mode::Inline => format!("<style>{}</style>", wrap_media(&css, &media)),
                        Mode::Archive => {
                            let mut l: Attrs = vec![
                                ("rel".into(), Some("stylesheet".into())),
                                ("href".into(), Some(u)),
                            ];
                            if !media.is_empty() {
                                l.push(("media".into(), Some(media)));
                            }
                            build_tag("link", &l, false)
                        }
                    });
                }
                if toks
                    .iter()
                    .any(|t| *t == "icon" || *t == "apple-touch-icon")
                {
                    let r = self.reference(base, &href, Kind::Image);
                    return Some(build_tag(
                        "link",
                        &vec![
                            ("rel".into(), Some("icon".into())),
                            ("href".into(), Some(r)),
                        ],
                        false,
                    ));
                }
                if toks.contains(&"canonical") {
                    let u = resolve_url(base, &href)?;
                    return Some(build_tag(
                        "link",
                        &vec![
                            ("rel".into(), Some("canonical".into())),
                            ("href".into(), Some(u)),
                        ],
                        false,
                    ));
                }
                None
            }
            "img" | "image" | "input" => {
                if name == "input"
                    && aget(&a, "type").is_none_or(|t| !t.eq_ignore_ascii_case("image"))
                {
                    return Some(build_tag(name, &a, selfclose));
                }
                let key = if name == "image" && aget(&a, "src").is_none() {
                    if aget(&a, "href").is_some() {
                        "href"
                    } else {
                        "xlink:href"
                    }
                } else {
                    "src"
                };
                let mut cand = aget(&a, key).map(String::from).filter(|s| {
                    !s.trim().is_empty()
                        && !s
                            .trim_start()
                            .to_ascii_lowercase()
                            .starts_with("data:image/gif")
                });
                for k in ["data-src", "data-original", "data-lazy-src"] {
                    if cand.is_none() {
                        cand = aget(&a, k)
                            .map(String::from)
                            .filter(|s| !s.trim().is_empty());
                    }
                }
                if cand.is_none() {
                    cand = aget(&a, "srcset").and_then(first_srcset);
                }
                if cand.is_none() {
                    cand = aget(&a, key).map(String::from); // keep an inline data: placeholder
                }
                for k in [
                    "srcset",
                    "sizes",
                    "data-src",
                    "data-original",
                    "data-lazy-src",
                    "data-srcset",
                    "loading",
                ] {
                    adel(&mut a, k);
                }
                let r = match cand {
                    Some(c)
                        if c.trim_start()
                            .to_ascii_lowercase()
                            .starts_with("data:image/") =>
                    {
                        c
                    }
                    Some(c) => self.reference(base, &c, Kind::Image),
                    None => "data:,".to_string(),
                };
                aset(&mut a, key, r);
                Some(build_tag(name, &a, selfclose))
            }
            "a" | "area" => {
                if let Some(h) = aget(&a, "href").map(String::from)
                    && let Some(u) = resolve_url(base, &h)
                {
                    aset(&mut a, "href", u);
                }
                Some(build_tag(name, &a, selfclose))
            }
            "form" => {
                adel(&mut a, "action");
                Some(build_tag(name, &a, selfclose))
            }
            "meta" => {
                let he = aget(&a, "http-equiv").unwrap_or("").to_ascii_lowercase();
                if a.iter().any(|(k, _)| k == "charset")
                    || matches!(
                        he.as_str(),
                        "refresh"
                            | "content-type"
                            | "content-security-policy"
                            | "default-style"
                            | "set-cookie"
                    )
                {
                    return None;
                }
                Some(build_tag(name, &a, selfclose))
            }
            _ => Some(build_tag(name, &a, selfclose)),
        }
    }

    /// The whole document: scripts and active content removed, resources embedded or archived.
    fn page(&mut self, html: &str, page_url: &str) -> String {
        let mut h = rx!(r"(?s)<!--.*?-->").replace_all(html, "").into_owned();
        for t in [
            "script", "noscript", "iframe", "object", "applet", "video", "audio", "frameset",
            "template",
        ] {
            let re = Regex::new(&format!(r"(?is)<{t}\b[^>]*>.*?</{t}\s*>")).unwrap();
            h = re.replace_all(&h, "").into_owned();
            // an unclosed one swallows the rest of the page, as a browser would
            let tail = Regex::new(&format!(r"(?is)<{t}\b.*$")).unwrap();
            h = tail.replace(&h, "").into_owned();
        }
        let base = rx!(r#"(?is)<base\b[^>]*?\bhref\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#)
            .captures(&h)
            .and_then(|c| {
                (1..=3)
                    .find_map(|i| c.get(i))
                    .map(|m| unescape_attr(m.as_str()))
            })
            .and_then(|b| resolve_url(page_url, &b))
            .unwrap_or_else(|| page_url.to_string());
        let out = rx!(r#"(?is)<style\b([^>]*)>(.*?)</style\s*>|<([a-zA-Z][a-zA-Z0-9:_-]*)((?:"[^"]*"|'[^']*'|[^'">])*)>"#)
            .replace_all(&h, |c: &regex::Captures| {
                if let Some(css) = c.get(2) {
                    let media = parse_attrs(&c[1]).iter().find(|(k, _)| k == "media").and_then(|(_, v)| v.clone()).unwrap_or_default();
                    let css = self.css_rewrite(css.as_str(), &base, 0);
                    return format!("<style>{}</style>", wrap_media(&css, &media));
                }
                let name = c[3].to_ascii_lowercase();
                self.tag(&name, &c[4], &base).unwrap_or_default()
            })
            .into_owned();
        let mut head = String::from("<meta charset=\"utf-8\">");
        if self.mode == Mode::Inline {
            head.push_str("<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; img-src data:; style-src 'unsafe-inline'; font-src data:; media-src data:\">");
        }
        head.push_str(&format!(
            "<!-- saved by CartaColibri web-grab from {} -->",
            page_url.replace("--", "-%2D")
        ));
        if let Some(m) = rx!(r"(?i)<head\b[^>]*>").find(&out) {
            format!("{}{head}{}", &out[..m.end()], &out[m.end()..])
        } else if let Some(m) = rx!(r"(?i)<html\b[^>]*>").find(&out) {
            format!("{}<head>{head}</head>{}", &out[..m.end()], &out[m.end()..])
        } else {
            format!("<!DOCTYPE html><html><head>{head}</head><body>{out}</body></html>")
        }
    }
}

fn rfc2822(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    let wd = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][((days + 4).rem_euclid(7)) as usize];
    let mon = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][(m - 1) as usize];
    format!(
        "{wd}, {d:02} {mon} {y} {:02}:{:02}:{:02} +0000",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

fn mhtml_build(url: &str, title: &str, html: &str, parts: &[(String, String, Vec<u8>)]) -> String {
    let mut rnd = [0u8; 12];
    let mut boundary = String::new();
    for _ in 0..8 {
        let _ = getrandom::fill(&mut rnd);
        boundary = format!("----=_CartaColibri_{}", hex::encode(rnd));
        if !html.contains(&boundary) {
            break;
        }
    }
    let subject: String = title.chars().take(45).collect();
    let mut o = String::new();
    o.push_str("From: <Saved by CartaColibri web-grab>\r\n");
    o.push_str(&format!(
        "Subject: =?utf-8?B?{}?=\r\n",
        b64_encode(subject.as_bytes())
    ));
    o.push_str(&format!("Date: {}\r\n", rfc2822(now())));
    o.push_str("MIME-Version: 1.0\r\n");
    o.push_str(&format!("Snapshot-Content-Location: {url}\r\n"));
    o.push_str(&format!(
        "Content-Type: multipart/related; type=\"text/html\"; boundary=\"{boundary}\"\r\n\r\n"
    ));
    o.push_str(&format!("--{boundary}\r\nContent-Type: text/html; charset=\"utf-8\"\r\nContent-Transfer-Encoding: quoted-printable\r\nContent-Location: {url}\r\n\r\n"));
    o.push_str(&qp_encode(html.as_bytes()));
    o.push_str("\r\n");
    for (u, mime, data) in parts {
        if mime == "text/css" {
            o.push_str(&format!("--{boundary}\r\nContent-Type: text/css; charset=\"utf-8\"\r\nContent-Transfer-Encoding: quoted-printable\r\nContent-Location: {u}\r\n\r\n"));
            o.push_str(&qp_encode(data));
            o.push_str("\r\n");
        } else {
            o.push_str(&format!("--{boundary}\r\nContent-Type: {mime}\r\nContent-Transfer-Encoding: base64\r\nContent-Location: {u}\r\n\r\n"));
            o.push_str(&b64_lines(data));
        }
    }
    o.push_str(&format!("--{boundary}--\r\n"));
    o
}

/// Saves a page as one file in `<home>/library/uploads/web-grab/` and indexes it.
/// `fmt` = `"html"` (self-contained: CSS inlined, images as data: URIs, scripts removed) or
/// `"mhtml"` (RFC 2557 archive: the page plus its CSS and images as MIME parts).
/// Every download goes through the guarded [`fetch_raw`] (public hosts only, redirects re-checked,
/// size / count / time caps); a sub-resource that cannot be fetched is dropped, not linked live.
pub fn save_page(lib: &mut Library, url: &str, fmt: &str) -> Result<Value, String> {
    page_mode(fmt)?;
    if url.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("that is not a web address".into());
    }
    let (raw, ctype, final_url) = fetch_raw_final(url, MAX_PAGE, 60, PAGE_AGENT)?;
    if raw.len() as u64 > MAX_PAGE {
        return Err("that page is too large (over 20 MB)".into());
    }
    let html = decode_with(&raw, &ctype);
    if !(ctype.contains("html") || html.trim_start().starts_with('<')) {
        return Err("that address is not a web page".into());
    }
    store_page(lib, &final_url, &html, fmt)
}

/// Like [`save_page`], for a page the caller already has (for instance the DOM a browser shows,
/// scripts and login included). `url` is only used to resolve and fetch sub-resources, through the
/// same guarded downloads.
pub fn save_page_html(
    lib: &mut Library,
    url: &str,
    html: &str,
    fmt: &str,
) -> Result<Value, String> {
    page_mode(fmt)?;
    if url.chars().any(|c| c.is_control() || c.is_whitespace()) || parse_target(url).is_none() {
        return Err("that is not a web address".into());
    }
    if html.len() as u64 > MAX_PAGE {
        return Err("that page is too large (over 20 MB)".into());
    }
    store_page(lib, url, html, fmt)
}

fn page_mode(fmt: &str) -> Result<(Mode, &'static str), String> {
    match fmt.trim().to_ascii_lowercase().as_str() {
        "html" | "htm" => Ok((Mode::Inline, "html")),
        "mhtml" | "mht" => Ok((Mode::Archive, "mhtml")),
        _ => Err("format must be \"html\" or \"mhtml\"".into()),
    }
}

/// The folder web-grab files go to (created): `<home>/library/uploads/web-grab`.
pub fn grab_dir(lib: &Library) -> Result<std::path::PathBuf, String> {
    let dir = lib.upload_dir()?.join("web-grab");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Reserves a fresh, collision-safe file `<stem>[-n].<ext>` in the web-grab folder (created empty).
fn reserve_file(dir: &Path, stem: &str, ext: &str) -> Result<std::path::PathBuf, String> {
    let stem = match slug(stem) {
        s if s.is_empty() => "page".to_string(),
        s => s,
    };
    for n in 0..10_000 {
        let name = if n == 0 {
            format!("{stem}.{ext}")
        } else {
            format!("{stem}-{n}.{ext}")
        };
        let p = dir.join(name);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&p)
        {
            Ok(_) => return Ok(p),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Err("cannot find a free file name".into())
}

/// A new empty `.pdf` in the web-grab folder for a page printed to PDF (a platform print job fills it).
pub fn pdf_target(lib: &Library, url: &str, title: &str) -> Result<std::path::PathBuf, String> {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = grab_dir(lib)?;
    let host = parse_target(url).map(|t| t.0).unwrap_or_default();
    reserve_file(
        &dir,
        if title.trim().is_empty() {
            &host
        } else {
            title
        },
        "pdf",
    )
}

/// Indexes whatever new files the web-grab folder holds (for example a PDF just printed there).
pub fn index_grabs(lib: &mut Library) -> Result<Value, String> {
    let up = lib.upload_dir()?.to_string_lossy().into_owned();
    let st = lib.update(Some(&[up]), &mut |_| {})?;
    Ok(json!(st))
}

fn store_page(lib: &mut Library, final_url: &str, html: &str, fmt: &str) -> Result<Value, String> {
    let (mode, ext) = page_mode(fmt)?;
    let page_url = resolve_url(final_url, final_url).ok_or("that is not a web address")?;
    let title = {
        let t = title_of(html);
        if t.is_empty() {
            parse_target(&page_url)
                .map(|t| t.0)
                .unwrap_or_else(|| "page".into())
        } else {
            t
        }
    };
    let mut ctx = Ctx {
        mode,
        origin: parse_target(&page_url),
        deadline: std::time::Instant::now() + Duration::from_secs(PAGE_SECS),
        count: 0,
        bytes: 0,
        emitted: 0,
        skipped: 0,
        blocked_private: 0,
        refs: BTreeMap::new(),
        css: BTreeMap::new(),
        parts: Vec::new(),
    };
    let doc = ctx.page(html, &page_url);
    let data = match mode {
        Mode::Inline => doc.into_bytes(),
        Mode::Archive => mhtml_build(&page_url, &title, &doc, &ctx.parts).into_bytes(),
    };
    if data.len() > PAGE_MAX_OUT {
        return Err("the saved page would be too large (over 64 MB)".into());
    }
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = grab_dir(lib)?;
    let reg_path = lib.home.join("webgrab.json");
    let mut reg: BTreeMap<String, String> = read_json(&reg_path);
    let key = format!("{ext}|{page_url}");
    let known = reg
        .get(&key)
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_file() && p.parent() == Some(dir.as_path()));
    let path = match known {
        Some(p) => p,
        None => reserve_file(&dir, &title, ext)?,
    };
    std::fs::write(&path, &data).map_err(|e| e.to_string())?;
    reg.insert(key, path.to_string_lossy().into_owned());
    write_json(&reg_path, &reg);
    let st = index_grabs(lib)?;
    Ok(json!({
        "path": path, "saved": path, "title": title, "url": page_url, "bytes": data.len(),
        "format": ext, "resources": ctx.count, "skipped": ctx.skipped,
        "blocked_private": ctx.blocked_private, "index": st,
    }))
}
