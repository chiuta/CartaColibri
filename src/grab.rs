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

fn host_of(url: &str) -> Option<(String, u16)> {
    let rest = url.split_once("://")?.1;
    let auth = rest.split(['/', '?', '#']).next()?;
    let auth = auth.rsplit_once('@').map(|x| x.1).unwrap_or(auth);
    let default = if url.starts_with("https") { 443 } else { 80 };
    if let Some(v6) = auth.strip_prefix('[') {
        let (h, tail) = v6.split_once(']')?;
        let port = tail
            .strip_prefix(':')
            .and_then(|p| p.parse().ok())
            .unwrap_or(default);
        return Some((h.to_string(), port));
    }
    match auth.rsplit_once(':') {
        Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty() => {
            Some((h.to_string(), p.parse().ok()?))
        }
        _ => Some((auth.to_string(), default)),
    }
}

/// True when the address behind `url` is this machine or a private network.
fn blocked(url: &str) -> bool {
    use std::net::{IpAddr, ToSocketAddrs};
    if ALLOW_LOCAL.load(std::sync::atomic::Ordering::Relaxed)
        || std::env::var("CARTACOLIBRI_GRAB_LOCAL").is_ok_and(|v| v == "1")
    {
        return false;
    }
    let Some((host, port)) = host_of(url) else {
        return true;
    };
    let bad = |ip: IpAddr| match ip {
        IpAddr::V4(v) => {
            v.is_loopback()
                || v.is_private()
                || v.is_link_local()
                || v.is_unspecified()
                || v.is_broadcast()
        }
        IpAddr::V6(v) => {
            v.is_loopback()
                || v.is_unspecified()
                || (v.segments()[0] & 0xfe00) == 0xfc00
                || (v.segments()[0] & 0xffc0) == 0xfe80
                || v.to_ipv4_mapped().is_some_and(bad4)
        }
    };
    fn bad4(v: std::net::Ipv4Addr) -> bool {
        v.is_loopback() || v.is_private() || v.is_link_local() || v.is_unspecified()
    }
    (host.as_str(), port)
        .to_socket_addrs()
        .map(|mut a| a.any(|s| bad(s.ip())))
        .unwrap_or(true)
}

fn absolute(base: &str, loc: &str) -> Option<String> {
    if loc.starts_with("http://") || loc.starts_with("https://") {
        return Some(loc.to_string());
    }
    let (scheme, rest) = base.split_once("://")?;
    let auth = rest.split(['/', '?', '#']).next()?;
    loc.starts_with('/')
        .then(|| format!("{scheme}://{auth}{loc}"))
}

/// Fetch a page and return (title, text). Loopback and private addresses are refused (also after redirects).
pub fn fetch(url: &str) -> Result<(String, String), String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("only http:// and https:// addresses can be grabbed".into());
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .max_redirects(0)
        .http_status_as_error(false)
        .user_agent("CartaColibri-web-grab/0.2 (+local; https://github.com/chiuta/CartaColibri)")
        .build()
        .into();
    let mut cur = url.to_string();
    let mut r = None;
    for _ in 0..6 {
        if blocked(&cur) {
            return Err(format!(
                "{cur} is on this machine or a private network; web-grab only fetches public pages (set CARTACOLIBRI_GRAB_LOCAL=1 to allow it)"
            ));
        }
        let resp = agent
            .get(&cur)
            .call()
            .map_err(|e| format!("cannot fetch {cur}: {e}"))?;
        let code = resp.status().as_u16();
        if (300..400).contains(&code) {
            let loc = resp
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            cur = absolute(&cur, loc).ok_or_else(|| format!("bad redirect from {cur}"))?;
            continue;
        }
        if code >= 400 {
            return Err(format!("cannot fetch {cur}: HTTP {code}"));
        }
        r = Some(resp);
        break;
    }
    let mut r = r.ok_or_else(|| format!("too many redirects from {url}"))?;
    let ctype = r
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    let mut raw = Vec::new();
    r.body_mut()
        .as_reader()
        .take(MAX_PAGE)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
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
    let _g = LOCK.lock().unwrap();
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
    let _g = LOCK.lock().unwrap();
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
