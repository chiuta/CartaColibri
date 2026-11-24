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
    Ok((raw, ctype))
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
