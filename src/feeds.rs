//! RSS / Atom feeds: parse, poll (bounded fetch, dedupe, notes in the library) and OPML.

use crate::extract::{decode, html_to_text};
use crate::import::{short_hash, slug, write_note};
use crate::library::Library;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

pub type Res<T> = Result<T, String>;

/// Largest feed document accepted.
pub const MAX_FEED_BYTES: u64 = 5 * 1024 * 1024;
const MAX_ITEMS: usize = 2000;
const MAX_FIELD: usize = 200_000;
const MAX_DEPTH: usize = 64;
const SEEN_KEEP: usize = 5000;
const MAX_NEW_PER_FEED: usize = 200;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub title: String,
    pub link: String,
    pub guid: String,
    pub date: String,
    pub author: String,
    pub summary: String,
}

impl Item {
    /// Stable identity: guid, else link, else title+date.
    pub fn key(&self) -> String {
        let k = if !self.guid.is_empty() {
            self.guid.clone()
        } else if !self.link.is_empty() {
            self.link.clone()
        } else {
            format!("{}|{}", self.title, self.date)
        };
        use sha1::{Digest, Sha1};
        hex::encode(Sha1::digest(k.as_bytes()))
    }
}

fn lname(e: &BytesStart) -> String {
    lbytes(e.name().as_ref())
}

fn lbytes(b: &[u8]) -> String {
    let n = String::from_utf8_lossy(b).to_lowercase();
    n.rsplit(':').next().unwrap_or("").to_string()
}

/// Resolve an entity reference event (`&amp;`, `&#233;`, ...) to text.
pub(crate) fn gref(r: &quick_xml::events::BytesRef) -> String {
    if let Ok(Some(c)) = r.resolve_char_ref() {
        return c.to_string();
    }
    let name = r.decode().map(|c| c.into_owned()).unwrap_or_default();
    match name.as_str() {
        "amp" => "&".into(),
        "lt" => "<".into(),
        "gt" => ">".into(),
        "quot" => "\"".into(),
        "apos" => "'".into(),
        n => format!("&{n};"),
    }
}

fn push_capped(s: &mut String, t: &str) {
    if s.len() < MAX_FIELD {
        s.push_str(t);
    }
}

/// Parse RSS 2.0 / RSS 1.0 / Atom into items. Malformed input yields whatever parsed so far.
pub fn parse(xml: &str) -> Vec<Item> {
    let mut r = Reader::from_str(xml);
    r.config_mut().check_end_names = false;
    let mut items = Vec::new();
    let mut cur: Option<Item> = None;
    let mut path: Vec<String> = Vec::new();
    let mut content = String::new();
    loop {
        let ev = r.read_event();
        match ev {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let n = lname(e);
                if (n == "item" || n == "entry") && cur.is_none() {
                    cur = Some(Item::default());
                    content.clear();
                }
                if n == "link"
                    && let Some(it) = cur.as_mut()
                {
                    {
                        let (mut href, mut rel) = (String::new(), String::new());
                        for a in e.attributes().flatten() {
                            let v = a
                                .unescape_value()
                                .map(|v| v.into_owned())
                                .unwrap_or_default();
                            match a.key.as_ref() {
                                b"href" => href = v,
                                b"rel" => rel = v,
                                _ => {}
                            }
                        }
                        if !href.is_empty()
                            && (rel.is_empty() || rel == "alternate")
                            && it.link.is_empty()
                        {
                            it.link = href;
                        }
                    }
                }
                if matches!(ev, Ok(Event::Start(_))) && path.len() < MAX_DEPTH {
                    path.push(n);
                }
            }
            Ok(Event::Text(ref t)) => {
                let s = t.decode().map(|c| c.into_owned()).unwrap_or_default();
                field(&mut cur, &path, &s, &mut content);
            }
            Ok(Event::GeneralRef(ref t)) => {
                let s = gref(t);
                field(&mut cur, &path, &s, &mut content);
            }
            Ok(Event::CData(ref t)) => {
                let s = decode(t.as_ref());
                field(&mut cur, &path, &s, &mut content);
            }
            Ok(Event::End(ref e)) => {
                let n = lbytes(e.name().as_ref());
                if (n == "item" || n == "entry")
                    && path.len() <= MAX_DEPTH
                    && let Some(mut it) = cur.take()
                {
                    {
                        if !content.is_empty() {
                            it.summary = std::mem::take(&mut content);
                        }
                        it.summary = html_to_text(&it.summary);
                        it.title = html_to_text(&it.title).trim().to_string();
                        it.link = it.link.trim().to_string();
                        it.guid = it.guid.trim().to_string();
                        if !(it.title.is_empty() && it.link.is_empty() && it.summary.is_empty()) {
                            items.push(it);
                        }
                        if items.len() >= MAX_ITEMS {
                            break;
                        }
                    }
                }
                path.pop();
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    items
}

fn field(cur: &mut Option<Item>, path: &[String], s: &str, content: &mut String) {
    let Some(it) = cur.as_mut() else { return };
    match path.last().map(String::as_str).unwrap_or("") {
        "title" => push_capped(&mut it.title, s),
        "link" => {
            if it.link.is_empty() {
                it.link = s.trim().to_string();
            }
        }
        "guid" | "id" => push_capped(&mut it.guid, s),
        "pubdate" | "published" | "updated" | "date" => {
            if it.date.is_empty() || path.last().map(String::as_str) == Some("pubdate") {
                it.date = s.trim().to_string();
            }
        }
        "author" | "creator" | "name" => {
            if it.author.is_empty() {
                it.author = s.trim().to_string();
            }
        }
        "encoded" | "content" => push_capped(content, s),
        "description" | "summary" if content.is_empty() => push_capped(&mut it.summary, s),
        _ => {}
    }
}

// ------------------------------------------------------------------ polling

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    /// feed url -> item keys already saved (oldest first)
    seen: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct FeedResult {
    pub url: String,
    pub items: usize,
    pub new: usize,
    pub error: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct PollSummary {
    pub feeds_ok: usize,
    pub feeds_failed: usize,
    pub new_items: usize,
    pub feeds: Vec<FeedResult>,
}

/// Fetch a feed document: http/https only, timeout, size limit.
pub fn fetch(url: &str) -> Res<String> {
    let lower = url.to_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err("only http:// and https:// feeds are supported".into());
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .user_agent("CartaColibri-feeds/0.2 (+local; https://github.com/chiuta/CartaColibri)")
        .build()
        .into();
    let mut resp = agent
        .get(url)
        .call()
        .map_err(|e| format!("cannot fetch {url}: {e}"))?;
    let mut raw = Vec::new();
    resp.body_mut()
        .as_reader()
        .take(MAX_FEED_BYTES + 1)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() as u64 > MAX_FEED_BYTES {
        return Err(format!(
            "feed larger than {} MB; skipped",
            MAX_FEED_BYTES >> 20
        ));
    }
    Ok(decode(&raw))
}

fn item_note(feed: &str, it: &Item) -> String {
    let title = if it.title.is_empty() {
        &it.link
    } else {
        &it.title
    };
    let mut s = format!("# {title}\n\n");
    if !it.link.is_empty() {
        s.push_str(&format!("Source: {}\n", it.link));
    }
    s.push_str(&format!("Feed: {feed}\n"));
    if !it.date.is_empty() {
        s.push_str(&format!("Date: {}\n", it.date));
    }
    if !it.author.is_empty() {
        s.push_str(&format!("Author: {}\n", it.author));
    }
    s.push_str(&format!("\n{}\n", it.summary.trim()));
    s
}

/// Poll every feed; new items become markdown notes under `<uploads>/feeds/<feed>/`, then the
/// notes are indexed. State lives in `<home>/feeds.json`.
pub fn poll(home: &Path, urls: &[String], lib: &mut Library) -> PollSummary {
    let state_path = home.join("feeds.json");
    let mut state: State = std::fs::read_to_string(&state_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let mut sum = PollSummary::default();
    let mut dirs = Vec::new();
    for url in urls {
        let mut fr = FeedResult {
            url: url.clone(),
            ..Default::default()
        };
        match fetch(url) {
            Err(e) => fr.error = Some(e),
            Ok(xml) => {
                let items = parse(&xml);
                fr.items = items.len();
                let host = url
                    .split("//")
                    .nth(1)
                    .unwrap_or(url)
                    .split(['/', '?'])
                    .next()
                    .unwrap_or("feed");
                let fname = format!("{}-{}", slug(host), short_hash(url));
                let seen = state.seen.entry(url.clone()).or_default();
                let dest = lib
                    .upload_dir()
                    .map(|d| d.join("feeds").join(&fname))
                    .unwrap_or_else(|_| home.join("feeds-notes").join(&fname));
                for it in items.iter().take(MAX_ITEMS) {
                    let k = it.key();
                    if seen.contains(&k) || fr.new >= MAX_NEW_PER_FEED {
                        continue;
                    }
                    let title = if it.title.is_empty() {
                        &it.link
                    } else {
                        &it.title
                    };
                    match write_note(&dest, title, &k, &item_note(url, it)) {
                        Ok(_) => {
                            seen.push(k);
                            fr.new += 1;
                        }
                        Err(e) => fr.error = Some(e),
                    }
                }
                if seen.len() > SEEN_KEEP {
                    let drop = seen.len() - SEEN_KEEP;
                    seen.drain(..drop);
                }
                if fr.new > 0 {
                    dirs.push(dest.to_string_lossy().into_owned());
                }
            }
        }
        if fr.error.is_some() && fr.new == 0 {
            sum.feeds_failed += 1;
        } else {
            sum.feeds_ok += 1;
        }
        sum.new_items += fr.new;
        sum.feeds.push(fr);
    }
    if let Ok(s) = serde_json::to_string_pretty(&state) {
        let tmp = state_path.with_extension("json.tmp");
        if std::fs::write(&tmp, s).is_ok() {
            let _ = std::fs::rename(&tmp, &state_path);
        }
    }
    if !dirs.is_empty() {
        let _ = lib.update(Some(&dirs), &mut |_| {});
    }
    sum
}

// ------------------------------------------------------------------ OPML

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subscription {
    pub title: String,
    pub url: String,
}

/// Feed subscriptions from an OPML document (nested outlines are flattened).
pub fn opml_parse(xml: &str) -> Vec<Subscription> {
    let mut r = Reader::from_str(xml);
    r.config_mut().check_end_names = false;
    let mut out = Vec::new();
    loop {
        match r.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) if lname(e) == "outline" => {
                let (mut url, mut title) = (String::new(), String::new());
                for a in e.attributes().flatten() {
                    let v = a
                        .unescape_value()
                        .map(|v| v.into_owned())
                        .unwrap_or_default();
                    match a.key.as_ref().to_ascii_lowercase().as_slice() {
                        b"xmlurl" => url = v,
                        b"title" => title = v,
                        b"text" if title.is_empty() => title = v,
                        _ => {}
                    }
                }
                if !url.is_empty() && out.len() < 10_000 {
                    out.push(Subscription { title, url });
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    out
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Write subscriptions as OPML 2.0.
pub fn opml_export(subs: &[Subscription]) -> String {
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<opml version=\"2.0\">\n  <head><title>CartaColibri subscriptions</title></head>\n  <body>\n",
    );
    for x in subs {
        s.push_str(&format!(
            "    <outline type=\"rss\" text=\"{0}\" title=\"{0}\" xmlUrl=\"{1}\"/>\n",
            esc(&x.title),
            esc(&x.url)
        ));
    }
    s.push_str("  </body>\n</opml>\n");
    s
}
