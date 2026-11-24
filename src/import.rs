//! Importers: bring other tools' exports into the library as markdown notes.
//! Every importer writes `.md` files into a destination folder (inside a library folder, so the
//! normal indexer picks them up). Re-running an import overwrites the same notes (stable names).
//! All parsing is bounded and never panics on malformed input.

use crate::extract::{csv_rows, decode, html_to_text};
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub type Res<T> = Result<T, String>;

const MAX_INPUT: u64 = 256 * 1024 * 1024;
const MAX_NOTES: usize = 200_000;
const MAX_DEPTH: usize = 64;
const ITEMS_PER_NOTE: usize = 200;

#[derive(Debug, Default, Clone, Serialize)]
pub struct ImportStats {
    pub notes: usize,
    pub skipped: usize,
    pub dest: String,
}

fn rx(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).unwrap())
}

pub(crate) fn slug(s: &str) -> String {
    let t: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let mut out = String::new();
    for c in t.chars() {
        if c == '-' && out.ends_with('-') {
            continue;
        }
        out.push(c);
    }
    out.trim_matches('-').chars().take(60).collect()
}

pub(crate) fn short_hash(s: &str) -> String {
    use sha1::{Digest, Sha1};
    hex::encode(Sha1::digest(s.as_bytes()))[..8].to_string()
}

/// Write `<slug(title)>-<hash(key)>.md` into `dest`; returns the path.
pub(crate) fn write_note(dest: &Path, title: &str, key: &str, body: &str) -> Res<PathBuf> {
    std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    let mut name = slug(title);
    if name.is_empty() {
        name = "note".into();
    }
    let p = dest.join(format!("{name}-{}.md", short_hash(key)));
    safe_write(&p, body.as_bytes())?;
    Ok(p)
}

/// Write a file, but never through a link planted at the target (note names are predictable).
fn safe_write(p: &Path, data: &[u8]) -> Res<()> {
    if let Ok(m) = std::fs::symlink_metadata(p)
        && !m.file_type().is_file()
    {
        if m.file_type().is_symlink() {
            std::fs::remove_file(p).map_err(|e| e.to_string())?;
        } else {
            return Err(format!("{} is not an ordinary file", p.display()));
        }
    }
    std::fs::write(p, data).map_err(|e| e.to_string())
}

fn read_limited(p: &Path) -> Res<String> {
    use std::io::Read;
    let md = std::fs::metadata(p).map_err(|e| e.to_string())?;
    if !md.is_file() {
        return Err("not an ordinary file (a device, pipe or socket)".into());
    }
    if md.len() > MAX_INPUT {
        return Err("file is too large to import".into());
    }
    let mut buf = Vec::new();
    std::fs::File::open(p)
        .map_err(|e| e.to_string())?
        .take(MAX_INPUT + 1)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    if buf.len() as u64 > MAX_INPUT {
        return Err("file is too large to import".into());
    }
    Ok(decode(&buf))
}

/// Pick an importer from the file name / content. A directory is treated as a Markdown vault.
pub fn import_path(src: &Path, dest: &Path) -> Res<ImportStats> {
    if src.is_dir() {
        return vault(src, dest);
    }
    let ext = src
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let text = read_limited(src)?;
    match ext.as_str() {
        "bib" | "bibtex" => bibtex(&text, dest),
        "enex" => enex(text.as_bytes(), dest),
        "csv" => csv_bookmarks(&text, dest),
        "html" | "htm" => bookmarks(&text, dest),
        _ => Err(format!("do not know how to import .{ext} files")),
    }
}

fn finish(dest: &Path, notes: usize, skipped: usize) -> ImportStats {
    ImportStats {
        notes,
        skipped,
        dest: dest.to_string_lossy().into_owned(),
    }
}

// ------------------------------------------------------------------ browser bookmarks

/// Netscape bookmark export (Chrome, Firefox, Edge, Safari): one note per bookmark folder.
pub fn bookmarks(html: &str, dest: &Path) -> Res<ImportStats> {
    static T: OnceLock<Regex> = OnceLock::new();
    static HREF: OnceLock<Regex> = OnceLock::new();
    let tok = rx(
        &T,
        r"(?is)<h3[^>]*>(.*?)</h3>|<a\s([^>]*)>(.*?)</a>|<dl\b[^>]*>|</dl\s*>",
    );
    let href = rx(&HREF, r#"(?is)\bhref\s*=\s*(?:"([^"]*)"|'([^']*)')"#);
    let mut stack: Vec<String> = Vec::new();
    let mut pending: Option<String> = None;
    let mut folders: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let mut skipped = 0;
    for c in tok.captures_iter(html) {
        let whole = c[0].to_lowercase();
        if let Some(h) = c.get(1) {
            pending = Some(html_to_text(h.as_str()).trim().to_string());
        } else if let Some(attrs) = c.get(2) {
            let url = href
                .captures(attrs.as_str())
                .and_then(|m| m.get(1).or(m.get(2)))
                .map(|m| html_to_text(m.as_str()).trim().to_string())
                .unwrap_or_default();
            let title = html_to_text(c.get(3).map_or("", |m| m.as_str()))
                .trim()
                .to_string();
            let lu: String = url
                .chars()
                .filter(|c| !c.is_whitespace() && !c.is_control())
                .collect::<String>()
                .to_lowercase();
            if url.is_empty()
                || ["javascript:", "data:", "vbscript:"]
                    .iter()
                    .any(|s| lu.starts_with(s))
            {
                skipped += 1;
                continue;
            }
            let path = if stack.is_empty() {
                "Bookmarks".to_string()
            } else {
                stack.join(" / ")
            };
            let list = folders.entry(path).or_default();
            if list.len() < 100_000 {
                list.push((title, url));
            }
        } else if whole.starts_with("<dl") {
            if stack.len() < MAX_DEPTH {
                stack.push(pending.take().unwrap_or_default());
            }
        } else if whole.starts_with("</dl") {
            stack.pop();
        }
    }
    // The outermost <DL> has no folder name: drop empty components.
    let mut notes = 0;
    for (path, items) in folders {
        let path: String = path
            .split(" / ")
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" / ");
        let path = if path.is_empty() {
            "Bookmarks".to_string()
        } else {
            path
        };
        for (n, chunk) in items.chunks(ITEMS_PER_NOTE * 5).enumerate() {
            let mut body = format!("# Bookmarks: {path}\n\n");
            for (t, u) in chunk {
                let one = |x: &str| x.replace(['\n', '\r'], " ");
                body.push_str(&format!(
                    "- [{}]({})\n",
                    one(if t.is_empty() { u.as_str() } else { t.as_str() }),
                    one(u)
                ));
            }
            write_note(
                dest,
                &format!("bookmarks {path}"),
                &format!("bm:{path}:{n}"),
                &body,
            )?;
            notes += 1;
            if notes >= MAX_NOTES {
                return Ok(finish(dest, notes, skipped));
            }
        }
    }
    Ok(finish(dest, notes, skipped))
}

// ------------------------------------------------------------------ Markdown / Obsidian vault

fn wiki_targets(text: &str) -> Vec<String> {
    static W: OnceLock<Regex> = OnceLock::new();
    rx(&W, r"\[\[([^\[\]\n]+?)\]\]")
        .captures_iter(text)
        .filter_map(|c| {
            let t = c[1].split(['|', '#']).next()?.trim();
            (!t.is_empty()).then(|| t.to_string())
        })
        .collect()
}

/// Canonical form of a path that may not exist yet: canonicalize the nearest existing ancestor.
fn canonical_lenient(p: &Path) -> Option<PathBuf> {
    let mut tail = Vec::new();
    let mut cur = p.to_path_buf();
    loop {
        if let Ok(c) = std::fs::canonicalize(&cur) {
            return Some(tail.iter().rev().fold(c, |a, t| a.join(t)));
        }
        tail.push(cur.file_name()?.to_owned());
        cur = cur.parent()?.to_path_buf();
        if cur.as_os_str().is_empty() {
            cur = PathBuf::from(".");
        }
    }
}

fn walk_md(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > 16 || out.len() >= 50_000 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut ents: Vec<_> = rd.flatten().collect();
    ents.sort_by_key(|e| e.file_name());
    for e in ents {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue; // .obsidian, .git, ...
        }
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            walk_md(&p, depth + 1, out);
        } else if name.to_lowercase().ends_with(".md") {
            out.push(p);
        }
    }
}

/// Copy a Markdown vault into `dest`, keeping `[[wiki-links]]` intact and appending a
/// "Backlinks" section to every note that is linked from other notes.
pub fn vault(src: &Path, dest: &Path) -> Res<ImportStats> {
    if !src.is_dir() {
        return Err(format!("not a folder: {}", src.display()));
    }
    // Writing into the vault itself would re-import our own notes and append "Backlinks"
    // sections to the user's originals on every run.
    if let (Ok(cs), Some(cd)) = (std::fs::canonicalize(src), canonical_lenient(dest))
        && cd.starts_with(&cs)
    {
        return Err("the destination must be outside the vault being imported".into());
    }
    let mut files = Vec::new();
    walk_md(src, 0, &mut files);
    let mut budget = MAX_INPUT;
    let mut notes: Vec<(String, String, String)> = Vec::new(); // (rel path, stem, text)
    let mut skipped = 0;
    for f in &files {
        match std::fs::metadata(f) {
            Ok(m) if m.is_file() && m.len() <= 5 * 1024 * 1024 && m.len() <= budget => {
                budget -= m.len();
            }
            _ => {
                skipped += 1;
                continue;
            }
        }
        let Ok(raw) = std::fs::read(f) else {
            skipped += 1;
            continue;
        };
        let rel = f
            .strip_prefix(src)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        let stem = f
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        notes.push((rel, stem, decode(&raw)));
    }
    let by_stem: BTreeMap<String, usize> = notes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.1.to_lowercase(), i))
        .collect();
    let mut back: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    for (i, (_, stem, text)) in notes.iter().enumerate() {
        for t in wiki_targets(text) {
            let key = t.rsplit('/').next().unwrap_or(&t).to_lowercase();
            if let Some(&j) = by_stem.get(&key)
                && j != i
            {
                back.entry(j).or_default().insert(stem.clone());
            }
        }
    }
    let mut count = 0;
    for (i, (rel, _, text)) in notes.iter().enumerate() {
        let mut body = text.clone();
        if let Some(b) = back.get(&i) {
            body.push_str("\n\n## Backlinks\n\n");
            for s in b {
                body.push_str(&format!("- [[{s}]]\n"));
            }
        }
        // keep the folder structure; neutralise any `..` components
        let safe: PathBuf = rel
            .split('/')
            .filter(|c| !c.is_empty() && *c != "." && *c != ".." && !c.contains(':'))
            .collect();
        let target = dest.join(safe);
        let written = target
            .parent()
            .map_or(Ok(()), |p| {
                std::fs::create_dir_all(p).map_err(|e| e.to_string())
            })
            .and_then(|_| safe_write(&target, body.as_bytes()));
        if written.is_ok() {
            count += 1;
        } else {
            skipped += 1; // one unwritable note (reserved name, clash with a folder) is not fatal
        }
    }
    Ok(finish(dest, count, skipped))
}

// ------------------------------------------------------------------ BibTeX / Zotero

fn bib_clean(v: &str) -> String {
    let t: String = v.chars().filter(|c| *c != '{' && *c != '}').collect();
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// (entry type, citation key, fields)
pub type BibEntry = (String, String, Vec<(String, String)>);

/// Parse a `.bib` file into (type, key, fields) tuples.
pub fn parse_bib(text: &str) -> Vec<BibEntry> {
    let b: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() && out.len() < MAX_NOTES {
        if b[i] != '@' {
            i += 1;
            continue;
        }
        i += 1;
        let s = i;
        while i < b.len() && b[i].is_alphanumeric() {
            i += 1;
        }
        let kind: String = b[s..i].iter().collect::<String>().to_lowercase();
        while i < b.len() && b[i].is_whitespace() {
            i += 1;
        }
        if i >= b.len() || !(b[i] == '{' || b[i] == '(') {
            continue;
        }
        let close = if b[i] == '{' { '}' } else { ')' };
        i += 1;
        // body up to the matching close at depth 0
        let start = i;
        let (mut depth, mut inq) = (0usize, false);
        while i < b.len() {
            match b[i] {
                '{' => depth += 1,
                '}' if depth > 0 => depth -= 1,
                '"' if depth == 0 => inq = !inq,
                c if c == close && depth == 0 && !inq => break,
                _ => {}
            }
            i += 1;
        }
        let body: Vec<char> = b[start..i.min(b.len())].to_vec();
        i += 1;
        if matches!(kind.as_str(), "comment" | "string" | "preamble" | "") {
            continue;
        }
        // key up to first comma
        let Some(comma) = body.iter().position(|&c| c == ',') else {
            continue;
        };
        let key: String = body[..comma].iter().collect::<String>().trim().to_string();
        let mut fields = Vec::new();
        let mut j = comma + 1;
        while j < body.len() && fields.len() < 200 {
            while j < body.len() && (body[j].is_whitespace() || body[j] == ',') {
                j += 1;
            }
            let ns = j;
            while j < body.len() && body[j] != '=' && body[j] != ',' {
                j += 1;
            }
            if j >= body.len() || body[j] != '=' {
                break;
            }
            let name: String = body[ns..j].iter().collect::<String>().trim().to_lowercase();
            j += 1;
            while j < body.len() && body[j].is_whitespace() {
                j += 1;
            }
            let mut val = String::new();
            if j < body.len() && body[j] == '{' {
                let (mut d, vs) = (0usize, j + 1);
                while j < body.len() {
                    if body[j] == '{' {
                        d += 1;
                    } else if body[j] == '}' {
                        d -= 1;
                        if d == 0 {
                            break;
                        }
                    }
                    j += 1;
                }
                val = body[vs..j.min(body.len())].iter().collect();
                j += 1;
            } else if j < body.len() && body[j] == '"' {
                let vs = j + 1;
                j += 1;
                let mut d = 0usize;
                while j < body.len() && !(body[j] == '"' && d == 0) {
                    match body[j] {
                        '{' => d += 1,
                        '}' => d = d.saturating_sub(1),
                        _ => {}
                    }
                    j += 1;
                }
                val = body[vs..j.min(body.len())].iter().collect();
                j += 1;
            } else {
                while j < body.len() && body[j] != ',' {
                    val.push(body[j]);
                    j += 1;
                }
            }
            if !name.is_empty() {
                fields.push((name, bib_clean(&val)));
            }
        }
        out.push((kind, key, fields));
    }
    out
}

/// BibTeX / Zotero export: one note per entry.
pub fn bibtex(text: &str, dest: &Path) -> Res<ImportStats> {
    let mut n = 0;
    for (kind, key, fields) in parse_bib(text) {
        let get = |k: &str| {
            fields
                .iter()
                .find(|(f, _)| f == k)
                .map(|(_, v)| v.as_str())
                .unwrap_or("")
        };
        let title = if get("title").is_empty() {
            key.as_str()
        } else {
            get("title")
        };
        let mut body = format!("# {title}\n\n");
        body.push_str(&format!("Citation key: {key}\nType: {kind}\n"));
        for (lbl, k) in [
            ("Authors", "author"),
            ("Editors", "editor"),
            ("Year", "year"),
            ("Journal", "journal"),
            ("Book", "booktitle"),
            ("Publisher", "publisher"),
            ("DOI", "doi"),
            ("URL", "url"),
            ("Keywords", "keywords"),
        ] {
            if !get(k).is_empty() {
                body.push_str(&format!("{lbl}: {}\n", get(k)));
            }
        }
        let abs = if get("abstract").is_empty() {
            get("note")
        } else {
            get("abstract")
        };
        if !abs.is_empty() {
            body.push_str(&format!("\n{abs}\n"));
        }
        write_note(dest, title, &format!("bib:{key}"), &body)?;
        n += 1;
    }
    Ok(finish(dest, n, 0))
}

// ------------------------------------------------------------------ Pocket / Raindrop CSV

/// Pocket (`title,url,time_added,tags,status`) and Raindrop
/// (`title,note,excerpt,url,folder,tags,created,...`) CSV exports: one note per folder / status.
pub fn csv_bookmarks(text: &str, dest: &Path) -> Res<ImportStats> {
    let rows = csv_rows(text, b',');
    let Some((head, body)) = rows.split_first() else {
        return Ok(finish(dest, 0, 0));
    };
    let col = |n: &str| head.iter().position(|h| h.trim().to_lowercase() == n);
    let url_i = col("url").ok_or("no \"url\" column: not a Pocket/Raindrop export")?;
    let (title_i, note_i, ex_i, tags_i) = (col("title"), col("note"), col("excerpt"), col("tags"));
    let group_i = col("folder").or(col("status"));
    let (mut groups, mut skipped): (BTreeMap<String, Vec<String>>, usize) = Default::default();
    for r in body {
        let g = |i: Option<usize>| {
            i.and_then(|i| r.get(i))
                .map(|s| s.trim())
                .unwrap_or("")
                .to_string()
        };
        let url = r
            .get(url_i)
            .map(|s| s.trim())
            .unwrap_or("")
            .replace(['\n', '\r'], " ");
        let lu: String = url
            .chars()
            .filter(|c| !c.is_whitespace() && !c.is_control())
            .collect::<String>()
            .to_lowercase();
        if url.is_empty()
            || ["javascript:", "data:", "vbscript:"]
                .iter()
                .any(|s| lu.starts_with(s))
        {
            skipped += 1;
            continue;
        }
        let title = g(title_i).replace(['\n', '\r'], " ");
        let mut line = format!(
            "- [{}]({url})",
            if title.is_empty() { &url } else { &title }
        );
        for (lbl, v) in [
            ("tags", g(tags_i)),
            ("note", g(note_i)),
            ("excerpt", g(ex_i)),
        ] {
            if !v.is_empty() {
                line.push_str(&format!(" -- {lbl}: {}", v.replace(['\n', '\r'], " ")));
            }
        }
        let group = g(group_i);
        groups
            .entry(if group.is_empty() {
                "Unsorted".into()
            } else {
                group
            })
            .or_default()
            .push(line);
    }
    let mut notes = 0;
    for (g, lines) in groups {
        for (n, chunk) in lines.chunks(ITEMS_PER_NOTE).enumerate() {
            let body = format!("# Saved links: {g}\n\n{}\n", chunk.join("\n"));
            write_note(
                dest,
                &format!("saved links {g}"),
                &format!("csv:{g}:{n}"),
                &body,
            )?;
            notes += 1;
        }
    }
    Ok(finish(dest, notes, skipped))
}

// ------------------------------------------------------------------ Evernote ENEX

/// Evernote `.enex` export: one note per `<note>`.
pub fn enex(xml: &[u8], dest: &Path) -> Res<ImportStats> {
    let mut r = Reader::from_reader(xml);
    r.config_mut().check_end_names = false;
    // one byte per open element (0 = other, 1 = note, 2 = title, 3 = content, 4 = created, 5 = tag)
    let mut path: Vec<u8> = Vec::new();
    let mut in_note_flag = false;
    let (mut title, mut content, mut created) = (String::new(), String::new(), String::new());
    let mut tags: Vec<String> = Vec::new();
    let (mut notes, mut skipped) = (0usize, 0usize);
    let mut buf = Vec::new();
    loop {
        let ev = r.read_event_into(&mut buf);
        let (text, is_text) = match &ev {
            Ok(Event::Text(t)) => (t.xml10_content().into_owned(), true),
            Ok(Event::CData(t)) => (t.as_ref().to_string(), true),
            Ok(Event::GeneralRef(t)) => (crate::feeds::gref(t), true),
            _ => (String::new(), false),
        };
        if is_text {
            let cur = path.last().copied().unwrap_or(0);
            let in_note = in_note_flag;
            if in_note && content.len() < 32 * 1024 * 1024 {
                match cur {
                    2 => title.push_str(&text),
                    3 => content.push_str(&text),
                    4 => created.push_str(&text),
                    5 => tags.push(text.trim().to_string()),
                    _ => {}
                }
            }
        }
        match ev {
            Ok(Event::Start(e)) => {
                let n = e.name().as_ref().to_lowercase();
                if n == "note" {
                    in_note_flag = true;
                    title.clear();
                    content.clear();
                    created.clear();
                    tags.clear();
                }
                path.push(match n.as_str() {
                    "note" => 1,
                    "title" => 2,
                    "content" => 3,
                    "created" => 4,
                    "tag" => 5,
                    _ => 0,
                });
            }
            Ok(Event::End(e)) => {
                let n = e.name().as_ref().to_lowercase();
                if n == "note" && in_note_flag {
                    in_note_flag = false;
                    let t = if title.trim().is_empty() {
                        "Untitled"
                    } else {
                        title.trim()
                    };
                    let text = html_to_text(&content);
                    if text.trim().is_empty() && tags.is_empty() {
                        skipped += 1;
                    } else {
                        let mut body = format!("# {t}\n\n");
                        if !created.trim().is_empty() {
                            body.push_str(&format!("Created: {}\n", created.trim()));
                        }
                        if !tags.is_empty() {
                            body.push_str(&format!("Tags: {}\n", tags.join(", ")));
                        }
                        body.push_str(&format!("\n{}\n", text.trim()));
                        write_note(dest, t, &format!("enex:{t}:{created}:{notes}"), &body)?;
                        notes += 1;
                    }
                }
                path.pop();
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
        if notes >= MAX_NOTES {
            break;
        }
    }
    Ok(finish(dest, notes, skipped))
}
