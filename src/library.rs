//! RAG-fetch: a local, incremental, full-text index of the user's folders (SQLite FTS5),
//! plus named groups of files that RAG-CARTA can be pointed at.
//! Documents are only ever read, never modified.

use crate::extract::{self, CHUNK_CHARS, IMG_EXT};
use regex::Regex;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const STATE_DIR_NAME: &str = "_cartacolibri";
pub const DEFAULT_EXT: &[&str] = &[
    ".txt",
    ".md",
    ".markdown",
    ".rst",
    ".log",
    ".pdf",
    ".csv",
    ".json",
    ".jsonl",
    ".yaml",
    ".yml",
    ".html",
    ".htm",
    ".rtf",
    ".docx",
    ".odt",
    ".pptx",
    ".epub",
    ".xlsx",
];
const SKIP_DIRS: &[&str] = &[
    STATE_DIR_NAME,
    ".cartacolibri",
    ".git",
    "__pycache__",
    "node_modules",
    ".venv",
    "venv",
    ".cache",
    "target",
];
const HEAVY_EXT: &[&str] = &[
    ".zip",
    ".7z",
    ".rar",
    ".tar",
    ".gz",
    ".iso",
    ".exe",
    ".dll",
    ".so",
    ".bin",
    ".gguf",
    ".safetensors",
    ".pt",
    ".pth",
    ".onnx",
    ".mp3",
    ".mp4",
    ".mkv",
    ".avi",
    ".mov",
    ".wav",
    ".flac",
];
const MAX_FILE_BYTES: u64 = 80 * 1024 * 1024;

pub type Res<T> = Result<T, String>;
fn e<E: std::fmt::Display>(x: E) -> String {
    x.to_string()
}

#[derive(Serialize, Clone, Debug)]
pub struct Doc {
    pub id: i64,
    pub root: String,
    pub path: String,
    pub ext: String,
    pub size: i64,
    pub chars: i64,
    pub pages: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct Hit {
    pub id: i64,
    pub path: String,
    pub ext: String,
    pub size: i64,
    pub loc: String,
    pub snippet: String,
    pub score: f64,
    pub matches: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Group {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub members: Vec<String>,
}

#[derive(Serialize, Default, Debug, Clone, PartialEq)]
pub struct Stats {
    pub added: u32,
    pub updated: u32,
    pub removed: u32,
    pub unchanged: u32,
    pub skipped: u32,
    pub failed: u32,
}

#[derive(Default, Clone, Debug)]
pub struct Filter {
    pub ext: Vec<String>,
    pub min_size: Option<i64>,
    pub max_size: Option<i64>,
    pub path_like: Option<String>,
    pub group: Option<String>,
    pub limit: Option<usize>,
}

pub fn state_dir(path: Option<&str>) -> Res<PathBuf> {
    let p = match path {
        Some(p) => PathBuf::from(p),
        None => match std::env::var_os("CARTACOLIBRI_HOME") {
            Some(h) => PathBuf::from(h),
            None => PathBuf::from(
                std::env::var_os("HOME")
                    .or_else(|| std::env::var_os("USERPROFILE"))
                    .unwrap_or_default(),
            )
            .join(".cartacolibri"),
        },
    };
    std::fs::create_dir_all(&p).map_err(e)?;
    Ok(p)
}

fn abs(p: &str) -> PathBuf {
    let p = if let Some(rest) = p.strip_prefix("~/") {
        PathBuf::from(
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .unwrap_or_default(),
        )
        .join(rest)
    } else {
        PathBuf::from(p)
    };
    let a = if p.is_absolute() {
        p
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    };
    // drop `.`/`..` lexically; keep symlinks as the user wrote them
    let mut out = PathBuf::new();
    for c in a.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            c => out.push(c),
        }
    }
    out
}

fn is_glob(s: &str) -> bool {
    s.contains(['*', '?', '['])
}

pub struct Library {
    pub home: PathBuf,
    pub exts: BTreeSet<String>,
    pub ocr: bool,
    db: Connection,
}

impl Library {
    pub fn open(home: Option<&str>, ocr: bool) -> Res<Library> {
        let home = state_dir(home)?;
        let ocr = (ocr || std::env::var("CARTACOLIBRI_OCR").as_deref() == Ok("1"))
            && extract::ocr_available();
        let mut exts: BTreeSet<String> = DEFAULT_EXT.iter().map(|s| s.to_string()).collect();
        if ocr {
            exts.extend(IMG_EXT.iter().map(|s| s.to_string()));
        }
        let db = Connection::open(home.join("index.sqlite")).map_err(e)?;
        db.busy_timeout(Duration::from_secs(30)).map_err(e)?;
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS roots(path TEXT PRIMARY KEY);
             CREATE TABLE IF NOT EXISTS docs(id INTEGER PRIMARY KEY, root TEXT, path TEXT UNIQUE, ext TEXT,
                 size INTEGER, mtime_ns INTEGER, sha1 TEXT, chars INTEGER, pages INTEGER, indexed_at REAL);
             CREATE VIRTUAL TABLE IF NOT EXISTS chunks USING fts5(text, doc_id UNINDEXED, loc UNINDEXED, seq UNINDEXED,
                 tokenize='unicode61 remove_diacritics 2');
             CREATE TABLE IF NOT EXISTS problems(path TEXT PRIMARY KEY, reason TEXT, at REAL);",
        )
        .map_err(e)?;
        Ok(Library {
            home,
            exts,
            ocr,
            db,
        })
    }

    // ---- folders
    pub fn add_root(&self, path: &str) -> Res<String> {
        let p = abs(path);
        if !p.is_dir() {
            return Err(format!("not a folder: {}", p.display()));
        }
        let s = p.to_string_lossy().into_owned();
        self.db
            .execute("INSERT OR IGNORE INTO roots VALUES(?)", [&s])
            .map_err(e)?;
        Ok(s)
    }

    pub fn remove_root(&self, path: &str) -> Res<usize> {
        let s = abs(path).to_string_lossy().into_owned();
        let ids: Vec<i64> = {
            let mut q = self
                .db
                .prepare("SELECT id FROM docs WHERE root=?")
                .map_err(e)?;
            q.query_map([&s], |r| r.get(0))
                .map_err(e)?
                .flatten()
                .collect()
        };
        for i in &ids {
            self.drop_doc(*i)?;
        }
        self.db
            .execute("DELETE FROM roots WHERE path=?", [&s])
            .map_err(e)?;
        Ok(ids.len())
    }

    pub fn roots(&self) -> Vec<String> {
        let mut q = self
            .db
            .prepare("SELECT path FROM roots ORDER BY path")
            .unwrap();
        q.query_map([], |r| r.get(0)).unwrap().flatten().collect()
    }

    // ---- uploads
    pub fn upload_dir(&self) -> Res<PathBuf> {
        let d = self.home.join("library").join("uploads");
        std::fs::create_dir_all(&d).map_err(e)?;
        Ok(d)
    }

    /// Save an uploaded file in the library and index it. `ocr`: "txt" and/or "pdf" for images.
    pub fn add_upload(
        &mut self,
        name: &str,
        data: &[u8],
        ocr: &[String],
    ) -> Res<serde_json::Value> {
        let base = name.replace('\\', "/");
        let base = base.rsplit('/').next().unwrap_or("document");
        let safe: String = base
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || matches!(c, '_' | '.' | ' ' | '-') {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let safe = safe.trim_matches(|c| c == ' ' || c == '.');
        let safe = if safe.is_empty() { "document" } else { safe };
        let dir = self.upload_dir()?;
        let mut path = dir.join(safe);
        let (stem, ext) = match safe.rsplit_once('.') {
            Some((s, x)) => (s.to_string(), format!(".{x}")),
            None => (safe.to_string(), String::new()),
        };
        let mut n = 1;
        while path.exists() {
            path = dir.join(format!("{stem}-{n}{ext}"));
            n += 1;
        }
        std::fs::write(&path, data).map_err(e)?;
        let mut made = vec![];
        if !ocr.is_empty() && IMG_EXT.contains(&ext.to_lowercase().as_str()) {
            let stem_path = path.with_extension("");
            if ocr.iter().any(|o| o == "txt") {
                let t = extract::ocr_image(&path, false)?;
                let p = format!("{}.ocr.txt", stem_path.display());
                std::fs::write(&p, t).map_err(e)?;
                made.push(p);
            }
            if ocr.iter().any(|o| o == "pdf") {
                let t = extract::ocr_image(&path, true)?;
                let p = format!("{}.ocr.pdf", stem_path.display());
                std::fs::write(&p, t).map_err(e)?;
                made.push(p);
            }
        }
        let stats = self.update(Some(&[dir.to_string_lossy().into_owned()]), &mut |_| {})?;
        Ok(serde_json::json!({"saved": path, "ocr_files": made, "index": stats}))
    }

    // ---- indexing
    fn drop_doc(&self, id: i64) -> Res<()> {
        self.db
            .execute("DELETE FROM chunks WHERE doc_id=?", [id])
            .map_err(e)?;
        self.db
            .execute("DELETE FROM docs WHERE id=?", [id])
            .map_err(e)?;
        Ok(())
    }

    fn walk(root: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(root) else {
            return;
        };
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|x| x.file_name());
        for en in entries {
            let name = en.file_name().to_string_lossy().into_owned();
            let Ok(ft) = en.file_type() else { continue };
            if ft.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) && !name.starts_with('.') {
                    Self::walk(&en.path(), out);
                }
            } else {
                out.push(en.path());
            }
        }
    }

    /// Incremental: only new/changed files are read; vanished ones are dropped.
    pub fn update(&mut self, roots: Option<&[String]>, log: &mut dyn FnMut(&str)) -> Res<Stats> {
        let mut st = Stats::default();
        let list: Vec<String> = match roots {
            Some(r) => r.to_vec(),
            None => self.roots(),
        };
        for root in list {
            let root = self.add_root(&root)?;
            let mut files = vec![];
            Self::walk(Path::new(&root), &mut files);
            let outputs = self.home.join("outputs"); // RAG-CARTA results are never re-indexed
            files.retain(|p| !p.starts_with(&outputs));
            let mut seen: HashSet<String> = HashSet::new();
            let tx = self.db.unchecked_transaction().map_err(e)?;
            for p in files {
                let ps = p.to_string_lossy().into_owned();
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let ext = p
                    .extension()
                    .map(|x| format!(".{}", x.to_string_lossy().to_lowercase()))
                    .unwrap_or_default();
                if name.starts_with('.')
                    || name.starts_with("~$")
                    || name.ends_with('~')
                    || name.ends_with(".tmp")
                    || HEAVY_EXT.contains(&ext.as_str())
                    || !self.exts.contains(&ext)
                {
                    st.skipped += 1;
                    continue;
                }
                let Ok(md) = std::fs::metadata(&p) else {
                    continue;
                };
                if md.len() > MAX_FILE_BYTES {
                    st.skipped += 1;
                    continue;
                }
                seen.insert(ps.clone());
                let mtime = md
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as i64)
                    .unwrap_or(0);
                let size = md.len() as i64;
                let row: Option<(i64, i64, i64, String)> = tx
                    .query_row(
                        "SELECT id,size,mtime_ns,sha1 FROM docs WHERE path=?",
                        [&ps],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                    )
                    .optional()
                    .map_err(e)?;
                if let Some((_, s, m, _)) = &row
                    && *s == size
                    && *m == mtime
                {
                    st.unchanged += 1;
                    continue;
                }
                let res: Res<bool> = (|| {
                    let raw = std::fs::read(&p).map_err(e)?;
                    let sha = hex::encode(<sha1::Sha1 as sha1::Digest>::digest(&raw));
                    if let Some((id, _, _, old)) = &row
                        && *old == sha
                    {
                        tx.execute("UPDATE docs SET mtime_ns=? WHERE id=?", params![mtime, id])
                            .map_err(e)?;
                        st.unchanged += 1;
                        return Ok(false);
                    }
                    drop(raw);
                    let parts = extract::extract(&p, &ext, self.ocr)?;
                    if let Some((id, ..)) = &row {
                        tx.execute("DELETE FROM chunks WHERE doc_id=?", [id])
                            .map_err(e)?;
                        tx.execute("DELETE FROM docs WHERE id=?", [id]).map_err(e)?;
                        st.updated += 1;
                    } else {
                        st.added += 1;
                    }
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs_f64())
                        .unwrap_or(0.0);
                    tx.execute(
                        "INSERT INTO docs(root,path,ext,size,mtime_ns,sha1,chars,pages,indexed_at) VALUES(?,?,?,?,?,?,0,?,?)",
                        params![root, ps, ext, size, mtime, sha, parts.len() as i64, now],
                    )
                    .map_err(e)?;
                    let did = tx.last_insert_rowid();
                    let (mut total, mut seq) = (0usize, 0i64);
                    for (loc, text) in &parts {
                        for c in extract::chunk(text, CHUNK_CHARS) {
                            tx.execute(
                                "INSERT INTO chunks(text,doc_id,loc,seq) VALUES(?,?,?,?)",
                                params![c, did, loc, seq],
                            )
                            .map_err(e)?;
                            total += c.chars().count();
                            seq += 1;
                        }
                    }
                    tx.execute(
                        "UPDATE docs SET chars=? WHERE id=?",
                        params![total as i64, did],
                    )
                    .map_err(e)?;
                    tx.execute("DELETE FROM problems WHERE path=?", [&ps])
                        .map_err(e)?;
                    Ok(true)
                })();
                if let Err(msg) = res {
                    st.failed += 1;
                    let msg: String = msg.chars().take(300).collect();
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs_f64())
                        .unwrap_or(0.0);
                    tx.execute(
                        "INSERT OR REPLACE INTO problems VALUES(?,?,?)",
                        params![ps, msg, now],
                    )
                    .map_err(e)?;
                    log(&format!("skip {ps}: {msg}"));
                }
            }
            let gone: Vec<(i64, String)> = {
                let mut q = tx
                    .prepare("SELECT id,path FROM docs WHERE root=?")
                    .map_err(e)?;
                q.query_map([&root], |r| Ok((r.get(0)?, r.get(1)?)))
                    .map_err(e)?
                    .flatten()
                    .collect()
            };
            for (id, path) in gone {
                if !seen.contains(&path) && !Path::new(&path).exists() {
                    tx.execute("DELETE FROM chunks WHERE doc_id=?", [id])
                        .map_err(e)?;
                    tx.execute("DELETE FROM docs WHERE id=?", [id]).map_err(e)?;
                    st.removed += 1;
                }
            }
            tx.commit().map_err(e)?;
        }
        Ok(st)
    }

    /// Folder watching by polling: re-run the incremental update every `interval` seconds.
    pub fn watch(&mut self, interval: Duration, stop: &AtomicBool, log: &mut dyn FnMut(&str)) {
        let mut waited = Duration::ZERO;
        while !stop.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(200));
            waited += Duration::from_millis(200);
            if waited < interval {
                continue;
            }
            waited = Duration::ZERO;
            if let Ok(st) = self.update(None, &mut |_| {})
                && st.added + st.updated + st.removed > 0
            {
                log(&format!("{st:?}"));
            }
        }
    }

    pub fn problems(&self) -> Vec<serde_json::Value> {
        let mut q = self
            .db
            .prepare("SELECT path,reason FROM problems ORDER BY path")
            .unwrap();
        q.query_map([], |r| Ok(serde_json::json!({"path": r.get::<_, String>(0)?, "reason": r.get::<_, String>(1)?}))).unwrap().flatten().collect()
    }

    // ---- listing / search
    fn all_docs(&self) -> Vec<Doc> {
        let mut q = self
            .db
            .prepare("SELECT id,root,path,ext,size,chars,pages FROM docs ORDER BY path")
            .unwrap();
        q.query_map([], |r| {
            Ok(Doc {
                id: r.get(0)?,
                root: r.get(1)?,
                path: r.get(2)?,
                ext: r.get(3)?,
                size: r.get(4)?,
                chars: r.get(5)?,
                pages: r.get(6)?,
            })
        })
        .unwrap()
        .flatten()
        .collect()
    }

    pub fn documents(&self, group: Option<&str>) -> Res<Vec<Doc>> {
        match group {
            None => Ok(self.all_docs()),
            Some(g) => self.resolve_group(g),
        }
    }

    pub fn docs_by_id(&self, ids: &[i64]) -> Vec<Doc> {
        let want: HashSet<i64> = ids.iter().copied().collect();
        self.all_docs()
            .into_iter()
            .filter(|d| want.contains(&d.id))
            .collect()
    }

    /// Words, "phrases", prefix* and AND/OR/NOT pass through; anything FTS5 would reject is quoted.
    pub fn fts_query(q: &str) -> Res<String> {
        let q = q.trim();
        if q.is_empty() {
            return Err("empty query".into());
        }
        let tok = Regex::new(r#""[^"]+"|\S+"#).unwrap();
        let word = Regex::new(r"\w+").unwrap();
        let mut out: Vec<String> = vec![];
        for m in tok.find_iter(q) {
            let t = m.as_str();
            if matches!(t, "AND" | "OR" | "NOT") {
                out.push(t.to_string());
            } else if t.len() > 2
                && t.starts_with('"')
                && t.ends_with('"')
                && !t[1..t.len() - 1].contains('"')
            {
                let phrase: Vec<&str> = word
                    .find_iter(&t[1..t.len() - 1])
                    .map(|m| m.as_str())
                    .collect();
                if !phrase.is_empty() {
                    out.push(format!("\"{}\"", phrase.join(" ")));
                }
            } else {
                let core = t.trim_end_matches('*');
                let w: String = core
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !w.is_empty() {
                    out.push(if t.ends_with('*') { format!("{w}*") } else { w });
                }
            }
        }
        Ok(if out.is_empty() {
            "\"\"".into()
        } else {
            out.join(" ")
        })
    }

    pub fn search(&self, query: &str, f: &Filter) -> Res<Vec<Hit>> {
        let allowed: Option<HashSet<i64>> = match &f.group {
            Some(g) => Some(self.resolve_group(g)?.into_iter().map(|d| d.id).collect()),
            None => None,
        };
        let exts: HashSet<String> = f
            .ext
            .iter()
            .map(|x| {
                if x.starts_with('.') {
                    x.to_lowercase()
                } else {
                    format!(".{}", x.to_lowercase())
                }
            })
            .collect();
        let sql = "SELECT doc_id, loc, snippet(chunks,0,'[',']',' … ',24), bm25(chunks) FROM chunks WHERE chunks MATCH ?1 ORDER BY bm25(chunks) LIMIT 2000";
        let run = |m: &str| -> rusqlite::Result<Vec<(i64, String, String, f64)>> {
            let mut q = self.db.prepare(sql)?;
            let rows = q.query_map([m], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
            rows.collect()
        };
        let word = Regex::new(r"\w+").unwrap();
        let plain: Vec<&str> = word
            .find_iter(query)
            .map(|m| m.as_str())
            .filter(|w| !matches!(*w, "AND" | "OR" | "NOT" | "NEAR"))
            .collect();
        let mut rows = match run(&Self::fts_query(query)?) {
            Ok(r) => r,
            Err(_) => {
                // e.g. a dangling AND/NOT: fall back to plain quoted words, never a crash
                if plain.is_empty() {
                    return Ok(vec![]);
                }
                run(&plain
                    .iter()
                    .map(|w| format!("\"{w}\""))
                    .collect::<Vec<_>>()
                    .join(" "))
                .map_err(e)?
            }
        };
        if rows.is_empty() && !plain.is_empty() {
            // typo/inflection tolerant fallback: every word cut to its stem, matched as a prefix
            let stems: Vec<String> = plain
                .iter()
                .map(|w| {
                    let n = w.chars().count();
                    let keep = 4.max((n as f64 * 0.7) as usize).min(n);
                    format!("{}*", w.chars().take(keep).collect::<String>())
                })
                .collect();
            rows = run(&stems.join(" ")).unwrap_or_default();
        }
        let mut hits: HashMap<i64, u32> = HashMap::new();
        let mut best: Vec<(i64, String, String, f64)> = vec![];
        for r in rows {
            let c = hits.entry(r.0).or_insert(0);
            *c += 1;
            if *c == 1 {
                best.push(r);
            }
        }
        let docs: HashMap<i64, Doc> = self.all_docs().into_iter().map(|d| (d.id, d)).collect();
        let mut out: Vec<Hit> = vec![];
        for (did, loc, snip, score) in best {
            let Some(d) = docs.get(&did) else { continue };
            if allowed.as_ref().is_some_and(|a| !a.contains(&did)) {
                continue;
            }
            if !exts.is_empty() && !exts.contains(&d.ext) {
                continue;
            }
            if f.min_size.is_some_and(|m| d.size < m) || f.max_size.is_some_and(|m| d.size > m) {
                continue;
            }
            if f.path_like
                .as_ref()
                .is_some_and(|p| !d.path.to_lowercase().contains(&p.to_lowercase()))
            {
                continue;
            }
            out.push(Hit {
                id: did,
                path: d.path.clone(),
                ext: d.ext.clone(),
                size: d.size,
                loc,
                snippet: snip.replace('\n', " "),
                score: -score,
                matches: hits[&did],
            });
        }
        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let top = out.first().map(|h| h.score).unwrap_or(0.0);
        let maxm = out.iter().map(|h| h.matches).max().unwrap_or(1).max(1) as f64;
        for h in out.iter_mut() {
            // DocFetcher-style "Score [%]"; on a tiny corpus bm25 is ~0, so fall back to the match count
            h.score = if top > 1e-9 {
                (100.0 * h.score / top).round()
            } else {
                (100.0 * h.matches as f64 / maxm).round()
            };
        }
        out.truncate(f.limit.unwrap_or(50));
        Ok(out)
    }

    pub fn text_of(&self, doc_id: i64) -> String {
        let mut q = self
            .db
            .prepare("SELECT loc,text FROM chunks WHERE doc_id=? ORDER BY CAST(seq AS INTEGER)")
            .unwrap();
        let rows: Vec<(String, String)> = q
            .query_map([doc_id], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .flatten()
            .collect();
        let (mut out, mut last): (Vec<String>, Option<String>) = (vec![], None);
        for (loc, text) in rows {
            if !loc.is_empty() && last.as_deref() != Some(loc.as_str()) {
                out.push(format!("\n[{loc}]"));
                last = Some(loc);
            }
            out.push(text);
        }
        out.join("\n")
    }

    // ---- groups
    fn groups_path(&self) -> PathBuf {
        self.home.join("groups.json")
    }

    pub fn groups(&self) -> BTreeMap<String, Group> {
        std::fs::read_to_string(self.groups_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_groups(&self, g: &BTreeMap<String, Group>) -> Res<()> {
        let tmp = self.groups_path().with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(g).map_err(e)?).map_err(e)?;
        std::fs::rename(&tmp, self.groups_path()).map_err(e)
    }

    /// members: file paths, folders, or glob patterns (matched against the full path).
    pub fn group_set(
        &self,
        name: &str,
        members: &[String],
        description: Option<&str>,
        add: bool,
    ) -> Res<Group> {
        let mut g = self.groups();
        let cur = g.entry(name.to_string()).or_default();
        if let Some(d) = description {
            cur.description = d.to_string();
        }
        if !members.is_empty() {
            let norm: Vec<String> = members
                .iter()
                .map(|m| {
                    if is_glob(m) {
                        m.clone()
                    } else {
                        abs(m).to_string_lossy().into_owned()
                    }
                })
                .collect();
            cur.members = if add {
                cur.members
                    .iter()
                    .cloned()
                    .chain(norm)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            } else {
                norm
            };
        }
        let out = cur.clone();
        self.save_groups(&g)?;
        Ok(out)
    }

    pub fn group_remove(&self, name: &str, members: &[String]) -> Res<()> {
        let mut g = self.groups();
        if !g.contains_key(name) {
            return Err(format!("no group '{name}'"));
        }
        if members.is_empty() {
            g.remove(name);
        } else {
            g.get_mut(name)
                .unwrap()
                .members
                .retain(|m| !members.contains(m));
        }
        self.save_groups(&g)
    }

    pub fn resolve_group(&self, name: &str) -> Res<Vec<Doc>> {
        let g = self.groups();
        let grp = g.get(name).ok_or_else(|| format!("no group '{name}'"))?;
        let pats: Vec<(bool, Option<glob::Pattern>, &String)> = grp
            .members
            .iter()
            .map(|m| (is_glob(m), glob::Pattern::new(m).ok(), m))
            .collect();
        let sep = std::path::MAIN_SEPARATOR;
        Ok(self
            .all_docs()
            .into_iter()
            .filter(|d| {
                pats.iter().any(|(glob_, pat, m)| {
                    if *glob_ {
                        pat.as_ref().is_some_and(|p| p.matches(&d.path))
                    } else {
                        d.path == **m
                            || d.path
                                .starts_with(&format!("{}{}", m.trim_end_matches(sep), sep))
                    }
                })
            })
            .collect())
    }
}
