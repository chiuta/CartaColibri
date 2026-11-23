//! web-grab: bring a web page (or a clipped selection) into the library as clean Markdown text,
//! keeping the source URL so RAG-CARTA citations can point back to the page.
//! 100% local: the page is fetched directly from the site, nothing goes through any third party.

use crate::extract::{decode, html_to_text};
use crate::library::Library;
use regex::Regex;
use serde_json::{Value, json};
use std::io::Read;
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

/// Fetch a page and return (title, text).
pub fn fetch(url: &str) -> Result<(String, String), String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("only http:// and https:// addresses can be grabbed".into());
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .user_agent("CartaColibri-web-grab/0.2 (+local; https://github.com/chiuta/CartaColibri)")
        .build()
        .into();
    let mut r = agent
        .get(url)
        .call()
        .map_err(|e| format!("cannot fetch {url}: {e}"))?;
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

fn norm_hash(text: &str) -> String {
    use sha1::{Digest, Sha1};
    let n = text.split_whitespace().collect::<Vec<_>>().join(" ");
    hex::encode(Sha1::digest(n.as_bytes()))
}

fn web_json(home: &std::path::Path) -> std::path::PathBuf {
    home.join("web.json")
}

fn load_web(home: &std::path::Path) -> std::collections::BTreeMap<String, Value> {
    std::fs::read_to_string(web_json(home))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Is this address already in the library, and does the page differ from the saved copy?
pub fn status(home: &std::path::Path, url: &str, text: &str) -> Option<Value> {
    let e = load_web(home).get(url).cloned()?;
    let path = e["path"].as_str()?.to_string();
    if !std::path::Path::new(&path).is_file() {
        return None;
    }
    let changed = !text.trim().is_empty() && e["hash"].as_str() != Some(norm_hash(text).as_str());
    Some(json!({"path": path, "saved_at": e["saved_at"], "changed": changed}))
}

/// Save a page/clip as Markdown inside the library uploads and index it.
/// Saving an address that is already in the library refreshes that file instead of adding a copy.
pub fn save(lib: &mut Library, url: &str, title: &str, text: &str) -> Result<Value, String> {
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
    let name = format!("{}.md", slug(if title.is_empty() { url } else { title }));
    let doc = format!(
        "# {}\n\nSource: {url}\n\n{clean}\n",
        if title.is_empty() { url } else { title }
    );
    let mut web = load_web(&lib.home);
    let known = web
        .get(url)
        .and_then(|e| e["path"].as_str())
        .map(String::from)
        .filter(|p| std::path::Path::new(p).is_file());
    let r = match known {
        Some(p) => {
            std::fs::write(&p, doc.as_bytes()).map_err(|e| e.to_string())?;
            let dir = lib.upload_dir()?.to_string_lossy().into_owned();
            let st = lib.update(Some(&[dir]), &mut |_| {})?;
            json!({"saved": p, "index": st})
        }
        None => lib.add_upload(&name, doc.as_bytes(), &[])?,
    };
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    web.insert(
        url.to_string(),
        json!({"path": r["saved"], "saved_at": secs, "hash": norm_hash(&clean)}),
    );
    let _ = std::fs::write(
        web_json(&lib.home),
        serde_json::to_string_pretty(&web).unwrap_or_default(),
    );
    Ok(
        json!({"title": title, "url": url, "chars": clean.chars().count(), "saved": r["saved"], "index": r["index"]}),
    )
}

pub fn grab(lib: &mut Library, url: &str) -> Result<Value, String> {
    let (title, text) = fetch(url)?;
    save(lib, url, &title, &text)
}
