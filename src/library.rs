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
    ".eml",
    ".mbox",
    ".ics",
    ".vcf",
    ".tsv",
    ".srt",
    ".vtt",
    ".ipynb",
    ".tex",
    ".org",
    ".adoc",
    ".ndjson",
    ".zip",
    ".html",
    ".htm",
    ".mhtml",
    ".mht",
    ".rtf",
    ".docx",
    ".odt",
    ".pptx",
    ".epub",
    ".xlsx",
    ".ods",
    ".odp",
    ".odg",
    ".ott",
    ".tar",
    ".tgz",
    ".docm",
    ".dotx",
    ".xlsm",
    ".xltx",
    ".pptm",
    ".ppsx",
    ".potx",
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
    ".7z",
    ".rar",
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
    /// last modification, seconds since the Unix epoch
    pub mtime: i64,
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
    /// PDF page number when the location is a page ("p.12"), else None.
    pub page: Option<i64>,
}

/// Result of [`Library::search_friendly`].
#[derive(Serialize, Clone, Debug, Default)]
pub struct FriendlyResult {
    pub hits: Vec<Hit>,
    /// Set when the original query found nothing and this corrected query was searched instead.
    pub corrected: Option<String>,
    /// "Did you mean" alternatives (full queries), best first.
    pub suggestions: Vec<String>,
}

/// Live progress of an index update (see [`Library::update_progress`]).
#[derive(Serialize, Clone, Debug, Default)]
pub struct Progress {
    pub root: String,
    pub current: String,
    /// Files in the folder tree looked at so far / in total.
    pub seen: u32,
    pub total: u32,
    /// Added + updated + removed so far.
    pub changed: u32,
    pub unchanged: u32,
    pub skipped: u32,
    pub failed: u32,
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
    /// Files looked at in total (all folders).
    pub seen: u32,
}

#[derive(Default, Clone, Debug)]
pub struct Filter {
    pub ext: Vec<String>,
    pub min_size: Option<i64>,
    pub max_size: Option<i64>,
    pub path_like: Option<String>,
    pub group: Option<String>,
    pub limit: Option<usize>,
    /// only documents modified at or after this time (Unix seconds)
    pub after: Option<i64>,
    /// only documents modified before this time (Unix seconds)
    pub before: Option<i64>,
    /// keep only hits whose matching passage matches this regular expression
    pub regex: Option<String>,
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

fn init_db(home: &Path) -> Res<Connection> {
    let db = Connection::open(home.join("index.sqlite")).map_err(e)?;
    db.busy_timeout(Duration::from_secs(30)).map_err(e)?;
    // WAL: readers (the web UI) never wait for the indexer's writes. Falls back silently on
    // file systems that cannot do it.
    let _ = db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;");
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS roots(path TEXT PRIMARY KEY);
         CREATE TABLE IF NOT EXISTS docs(id INTEGER PRIMARY KEY, root TEXT, path TEXT UNIQUE, ext TEXT,
             size INTEGER, mtime_ns INTEGER, sha1 TEXT, chars INTEGER, pages INTEGER, indexed_at REAL);
         CREATE VIRTUAL TABLE IF NOT EXISTS chunks USING fts5(text, doc_id UNINDEXED, loc UNINDEXED, seq UNINDEXED,
             tokenize='unicode61 remove_diacritics 2');
         CREATE TABLE IF NOT EXISTS problems(path TEXT PRIMARY KEY, reason TEXT, at REAL);
         CREATE INDEX IF NOT EXISTS docs_sha1 ON docs(sha1);
         CREATE VIRTUAL TABLE IF NOT EXISTS chunks_vocab USING fts5vocab(chunks,'row');",
    )
    .map_err(e)?;
    Ok(db)
}

/// Longest query (bytes) and most terms that are looked at; the rest is ignored.
const MAX_QUERY_BYTES: usize = 4096;
const MAX_QUERY_TERMS: usize = 64;
const MAX_SUGGEST_WORDS: usize = 16;

/// Inline filters typed in the query box: `ext:pdf,docx`, `path:contracts`, `size:>500kb`, `size:<2mb`.
/// They are removed from the text and merged into the filter (the explicit filter keeps what it already has).
pub fn parse_inline(query: &str, f: &Filter) -> (String, Filter) {
    let mut f = f.clone();
    let mut rest: Vec<&str> = vec![];
    for tok in query.split_whitespace() {
        let low = tok.to_lowercase();
        if let Some(v) = low.strip_prefix("ext:") {
            let exts: Vec<String> = v
                .split(',')
                .map(|e| e.trim_start_matches('.').to_string())
                .filter(|e| !e.is_empty() && e.chars().all(|c| c.is_ascii_alphanumeric()))
                .collect();
            if !exts.is_empty() {
                f.ext.extend(exts);
                continue;
            }
        } else if let Some(v) = tok.get(5..).filter(|_| low.starts_with("path:")) {
            if !v.is_empty() {
                f.path_like = Some(v.trim_matches('"').to_string());
                continue;
            }
        } else if let Some(v) = low
            .strip_prefix("after:")
            .or_else(|| low.strip_prefix("before:"))
        {
            if let Some(t) = parse_date(v) {
                if low.starts_with("after:") {
                    f.after = Some(t);
                } else {
                    f.before = Some(t);
                }
                continue;
            }
        } else if let Some(v) = tok.get(3..).filter(|_| low.starts_with("re:")) {
            if !v.is_empty() {
                f.regex = Some(v.trim_matches('"').to_string());
                continue;
            }
        } else if let Some(v) = low.strip_prefix("size:") {
            let (op, num) = v.split_at(
                v.chars()
                    .next()
                    .filter(|c| matches!(c, '<' | '>'))
                    .map_or(0, |_| 1),
            );
            let digits: String = num
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            let unit = &num[digits.len()..];
            let mult = match unit {
                "" | "b" => Some(1.0),
                "k" | "kb" => Some(1024.0),
                "m" | "mb" => Some(1024.0 * 1024.0),
                "g" | "gb" => Some(1024.0 * 1024.0 * 1024.0),
                _ => None,
            };
            if let (Ok(n), Some(m), false) = (digits.parse::<f64>(), mult, op.is_empty()) {
                let bytes = (n * m) as i64;
                if op == ">" {
                    f.min_size = Some(bytes);
                } else {
                    f.max_size = Some(bytes);
                }
                continue;
            }
        }
        rest.push(tok);
    }
    (rest.join(" "), f)
}

/// `YYYY`, `YYYY-MM` or `YYYY-MM-DD` -> Unix seconds at 00:00 UTC.
pub fn parse_date(v: &str) -> Option<i64> {
    let mut it = v.split('-');
    let y: i64 = it.next()?.parse().ok()?;
    let m: i64 = it.next().map_or(Some(1), |x| x.parse().ok())?;
    let d: i64 = it.next().map_or(Some(1), |x| x.parse().ok())?;
    if it.next().is_some()
        || !(1..=9999).contains(&y)
        || !(1..=12).contains(&m)
        || !(1..=31).contains(&d)
    {
        return None;
    }
    // days from civil (Howard Hinnant)
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some((era * 146097 + doe - 719468) * 86400)
}

fn clip(q: &str) -> &str {
    if q.len() <= MAX_QUERY_BYTES {
        return q;
    }
    let mut c = MAX_QUERY_BYTES;
    while !q.is_char_boundary(c) {
        c -= 1;
    }
    &q[..c]
}

pub struct Library {
    pub home: PathBuf,
    pub exts: BTreeSet<String>,
    pub ocr: bool,
    pub(crate) db: Connection,
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
        // old binary Office files, only when a converter is installed (else they would all fail)
        exts.extend(extract::legacy_readable().iter().map(|s| s.to_string()));
        let db = init_db(&home)?;
        Ok(Library {
            home,
            exts,
            ocr,
            db,
        })
    }

    /// The index connection, for sibling modules (organize).
    pub(crate) fn conn(&self) -> &Connection {
        &self.db
    }

    /// Throw the index away and start an empty one (same folders registered; the caller re-runs
    /// `update`). Used to recover from a corrupt index.
    pub fn reset_index(&mut self) -> Res<()> {
        let roots = self.roots();
        self.db = Connection::open_in_memory().map_err(e)?;
        for n in [
            "index.sqlite",
            "index.sqlite-wal",
            "index.sqlite-shm",
            "index.sqlite-journal",
        ] {
            let _ = std::fs::remove_file(self.home.join(n));
        }
        self.db = init_db(&self.home)?;
        for r in roots {
            self.db
                .execute("INSERT OR IGNORE INTO roots VALUES(?)", [&r])
                .map_err(e)?;
        }
        Ok(())
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
        let Ok(mut q) = self.db.prepare("SELECT path FROM roots ORDER BY path") else {
            return vec![];
        };
        let Ok(rows) = q.query_map([], |r| r.get(0)) else {
            return vec![];
        };
        rows.flatten().collect()
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
        // vectors are keyed by passage rowid, which SQLite may hand out again: never keep them
        let _ = self.db.execute("DELETE FROM vectors WHERE doc_id=?", [id]);
        self.db
            .execute("DELETE FROM docs WHERE id=?", [id])
            .map_err(e)?;
        Ok(())
    }

    fn walk(
        dir: &Path,
        out: &mut Vec<PathBuf>,
        probs: &mut Vec<(String, String)>,
        seen_dirs: &mut HashSet<PathBuf>,
        depth: usize,
    ) {
        if depth > 64 {
            probs.push((
                dir.to_string_lossy().into_owned(),
                "Folders are nested too deeply; the rest of this branch was skipped.".into(),
            ));
            return;
        }
        let rd = match std::fs::read_dir(dir) {
            Ok(r) => r,
            Err(er) => {
                probs.push((
                    dir.to_string_lossy().into_owned(),
                    format!(
                        "This folder cannot be opened: {}",
                        extract::plain_reason("", &er.to_string())
                    ),
                ));
                return;
            }
        };
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|x| x.file_name());
        for en in entries {
            let name = en.file_name().to_string_lossy().into_owned();
            let Ok(ft) = en.file_type() else { continue };
            let path = en.path();
            let is_dir = if ft.is_symlink() {
                // follow links to folders, but never walk the same real folder twice (loops)
                match std::fs::metadata(&path) {
                    Ok(m) if m.is_dir() => match std::fs::canonicalize(&path) {
                        Ok(c) if seen_dirs.insert(c.clone()) => true,
                        _ => continue,
                    },
                    _ => false,
                }
            } else {
                ft.is_dir()
            };
            if is_dir {
                if !SKIP_DIRS.contains(&name.as_str()) && !name.starts_with('.') {
                    if !ft.is_symlink()
                        && let Ok(c) = std::fs::canonicalize(&path)
                    {
                        seen_dirs.insert(c);
                    }
                    Self::walk(&path, out, probs, seen_dirs, depth + 1);
                }
            } else if ft.is_file() {
                out.push(path);
            } else {
                // Pipes, sockets and device nodes are never opened (reading a FIFO would block
                // forever); a link is kept only if it points at an ordinary file.
                let regular =
                    ft.is_symlink() && std::fs::metadata(&path).is_ok_and(|m| m.is_file());
                if regular {
                    out.push(path);
                } else if !ft.is_symlink() {
                    probs.push((
                        path.to_string_lossy().into_owned(),
                        "This is not an ordinary file (a device, pipe or socket), so it was skipped."
                            .into(),
                    ));
                }
            }
        }
    }

    /// Incremental: only new/changed files are read; vanished ones are dropped.
    pub fn update(&mut self, roots: Option<&[String]>, log: &mut dyn FnMut(&str)) -> Res<Stats> {
        self.update_progress(roots, &mut |_| {}, log)
    }

    /// Like [`update`](Self::update) with a progress callback fired after every file.
    pub fn update_progress(
        &mut self,
        roots: Option<&[String]>,
        progress: &mut dyn FnMut(&Progress),
        log: &mut dyn FnMut(&str),
    ) -> Res<Stats> {
        let mut st = Stats::default();
        let list: Vec<String> = match roots {
            Some(r) => r.to_vec(),
            None => self.roots(),
        };
        let now_f = || {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0)
        };
        for root in list {
            let root = self.add_root(&root)?;
            let mut files = vec![];
            let mut walk_probs = vec![];
            let mut seen_dirs = HashSet::new();
            if let Ok(c) = std::fs::canonicalize(&root) {
                seen_dirs.insert(c);
            }
            Self::walk(
                Path::new(&root),
                &mut files,
                &mut walk_probs,
                &mut seen_dirs,
                0,
            );
            // everything the walk found, before any filtering: an indexed file that is still
            // here but no longer qualifies (renamed hidden, grown too big, ...) must leave the index
            let walked: HashSet<String> = files
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
            let outputs = self.home.join("outputs"); // RAG-CARTA results are never re-indexed
            files.retain(|p| !p.starts_with(&outputs));
            // a link planted inside the folder must not pull in files from elsewhere (e.g. ~/.ssh)
            if let Ok(croot) = std::fs::canonicalize(&root) {
                files.retain(|p| match std::fs::symlink_metadata(p) {
                    Ok(m) if m.file_type().is_symlink() => match std::fs::canonicalize(p) {
                        Ok(c) if c.starts_with(&croot) => true,
                        _ => {
                            walk_probs.push((
                                p.to_string_lossy().into_owned(),
                                "This link points outside the folder, so it was skipped.".into(),
                            ));
                            false
                        }
                    },
                    _ => true,
                });
            }
            let mut seen: HashSet<String> = HashSet::new();
            let tx = self.db.unchecked_transaction().map_err(e)?;
            let prefix = format!(
                "{}{}",
                root.trim_end_matches(['/', '\\']),
                std::path::MAIN_SEPARATOR
            );
            tx.execute(
                "DELETE FROM problems WHERE substr(path,1,length(?1))=?1",
                [&prefix],
            )
            .map_err(e)?;
            let note = |tx: &rusqlite::Transaction, path: &str, msg: &str| -> Res<()> {
                tx.execute(
                    "INSERT OR REPLACE INTO problems VALUES(?,?,?)",
                    params![path, msg, now_f()],
                )
                .map_err(e)?;
                Ok(())
            };
            for (p, m) in &walk_probs {
                note(&tx, p, m)?;
                log(&format!("skip {p}: {m}"));
            }
            let mut pr = Progress {
                root: root.clone(),
                total: files.len() as u32,
                ..Default::default()
            };
            for p in files {
                let ps = p.to_string_lossy().into_owned();
                st.seen += 1;
                pr.seen += 1;
                pr.current = ps.clone();
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let mut ext = p
                    .extension()
                    .map(|x| format!(".{}", x.to_string_lossy().to_lowercase()))
                    .unwrap_or_default();
                if name.to_lowercase().ends_with(".tar.gz") {
                    ext = ".tgz".into();
                }
                if name.starts_with('.')
                    || name.starts_with("~$")
                    || name.ends_with('~')
                    || name.ends_with(".tmp")
                    || HEAVY_EXT.contains(&ext.as_str())
                    || !self.exts.contains(&ext)
                {
                    st.skipped += 1;
                    pr.skipped += 1;
                    progress(&pr);
                    continue;
                }
                let md = match std::fs::metadata(&p) {
                    Ok(m) => m,
                    Err(er) => {
                        st.failed += 1;
                        pr.failed += 1;
                        let msg = extract::plain_reason(&ext, &er.to_string());
                        note(&tx, &ps, &msg)?;
                        log(&format!("skip {ps}: {msg}"));
                        progress(&pr);
                        continue;
                    }
                };
                if !md.is_file() {
                    st.skipped += 1;
                    pr.skipped += 1;
                    note(
                        &tx,
                        &ps,
                        "This is not an ordinary file (a device, pipe or socket), so it was skipped.",
                    )?;
                    progress(&pr);
                    continue;
                }
                if md.len() > MAX_FILE_BYTES {
                    st.skipped += 1;
                    pr.skipped += 1;
                    let msg = format!(
                        "Skipped: the file is very large ({} MB; the limit is {} MB).",
                        md.len() >> 20,
                        MAX_FILE_BYTES >> 20
                    );
                    note(&tx, &ps, &msg)?;
                    progress(&pr);
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
                    .prepare_cached("SELECT id,size,mtime_ns,sha1 FROM docs WHERE path=?")
                    .map_err(e)?
                    .query_row([&ps], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
                    .optional()
                    .map_err(e)?;
                if let Some((_, s, m, _)) = &row
                    && *s == size
                    && *m == mtime
                {
                    st.unchanged += 1;
                    pr.unchanged += 1;
                    progress(&pr);
                    continue;
                }
                let mut empty = false;
                tx.execute_batch("SAVEPOINT file_sp").map_err(e)?;
                let res: Res<bool> = (|| {
                    let raw = {
                        use std::io::Read;
                        let f = extract::open_regular(&p)
                            .map_err(|er| extract::plain_reason(&ext, &er.to_string()))?;
                        let mut buf = Vec::new();
                        f.take(MAX_FILE_BYTES + 1)
                            .read_to_end(&mut buf)
                            .map_err(|er| extract::plain_reason(&ext, &er.to_string()))?;
                        if buf.len() as u64 > MAX_FILE_BYTES {
                            return Err(format!(
                                "Skipped: the file is very large (the limit is {} MB).",
                                MAX_FILE_BYTES >> 20
                            ));
                        }
                        buf
                    };
                    let sha = hex::encode(<sha1::Sha1 as sha1::Digest>::digest(&raw));
                    if let Some((id, _, _, old)) = &row
                        && *old == sha
                    {
                        tx.execute("UPDATE docs SET mtime_ns=? WHERE id=?", params![mtime, id])
                            .map_err(e)?;
                        st.unchanged += 1;
                        pr.unchanged += 1;
                        return Ok(false);
                    }
                    drop(raw);
                    let parts = extract::extract_safe(&p, &ext, self.ocr)?;
                    if let Some((id, ..)) = &row {
                        tx.execute("DELETE FROM chunks WHERE doc_id=?", [id])
                            .map_err(e)?;
                        let _ = tx.execute("DELETE FROM vectors WHERE doc_id=?", [id]);
                        tx.execute("DELETE FROM docs WHERE id=?", [id]).map_err(e)?;
                        st.updated += 1;
                    } else {
                        st.added += 1;
                    }
                    pr.changed += 1;
                    tx.execute(
                        "INSERT INTO docs(root,path,ext,size,mtime_ns,sha1,chars,pages,indexed_at) VALUES(?,?,?,?,?,?,0,?,?)",
                        params![root, ps, ext, size, mtime, sha, parts.len() as i64, now_f()],
                    )
                    .map_err(e)?;
                    let did = tx.last_insert_rowid();
                    let (mut total, mut seq) = (0usize, 0i64);
                    {
                        let mut ins = tx
                            .prepare_cached(
                                "INSERT INTO chunks(text,doc_id,loc,seq) VALUES(?,?,?,?)",
                            )
                            .map_err(e)?;
                        for (loc, text) in &parts {
                            for c in extract::chunk(text, CHUNK_CHARS) {
                                ins.execute(params![c, did, loc, seq]).map_err(e)?;
                                total += c.chars().count();
                                seq += 1;
                            }
                        }
                    }
                    tx.execute(
                        "UPDATE docs SET chars=? WHERE id=?",
                        params![total as i64, did],
                    )
                    .map_err(e)?;
                    tx.execute("DELETE FROM problems WHERE path=?", [&ps])
                        .map_err(e)?;
                    empty = total == 0;
                    Ok(true)
                })();
                // a file either lands completely or not at all (never a doc with half its chunks)
                tx.execute_batch(if res.is_err() {
                    "ROLLBACK TO file_sp; RELEASE file_sp"
                } else {
                    "RELEASE file_sp"
                })
                .map_err(e)?;
                if let Err(msg) = res {
                    st.failed += 1;
                    pr.failed += 1;
                    let msg: String = msg.chars().take(300).collect();
                    note(&tx, &ps, &msg)?;
                    log(&format!("skip {ps}: {msg}"));
                } else if empty {
                    note(
                        &tx,
                        &ps,
                        "No readable text was found in this file (empty, or a scan: try OCR).",
                    )?;
                }
                progress(&pr);
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
                if !seen.contains(&path) && (walked.contains(&path) || !Path::new(&path).exists()) {
                    tx.execute("DELETE FROM chunks WHERE doc_id=?", [id])
                        .map_err(e)?;
                    let _ = tx.execute("DELETE FROM vectors WHERE doc_id=?", [id]);
                    tx.execute("DELETE FROM docs WHERE id=?", [id]).map_err(e)?;
                    st.removed += 1;
                    pr.changed += 1;
                    progress(&pr);
                }
            }
            tx.commit().map_err(e)?;
        }
        Ok(st)
    }

    /// Groups of indexed files with identical content (same SHA-1), each group sorted by path.
    pub fn duplicates(&self) -> Vec<Vec<String>> {
        let Ok(mut q) = self.db.prepare(
            "SELECT sha1, size, path FROM docs WHERE size>0 AND (sha1,size) IN
             (SELECT sha1,size FROM docs WHERE size>0 GROUP BY sha1,size HAVING count(*)>1)
             ORDER BY sha1, size, path",
        ) else {
            return vec![];
        };
        let Ok(it) = q.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
            ))
        }) else {
            return vec![];
        };
        let mut by: BTreeMap<(String, i64), Vec<String>> = BTreeMap::new();
        for (h, sz, p) in it.flatten() {
            by.entry((h, sz)).or_default().push(p);
        }
        // SHA-1 is broken (crafted collisions exist): confirm byte for byte before telling the
        // user two files are the same, and drop files that vanished since indexing.
        let mut out: Vec<Vec<String>> = vec![];
        for (_, paths) in by {
            let mut classes: Vec<Vec<String>> = vec![];
            'next: for p in paths {
                if !Path::new(&p).is_file() {
                    continue;
                }
                for c in classes.iter_mut() {
                    if same_bytes(Path::new(&c[0]), Path::new(&p)) == Some(true) {
                        c.push(p);
                        continue 'next;
                    }
                }
                classes.push(vec![p]);
            }
            out.extend(classes.into_iter().filter(|c| c.len() > 1));
        }
        out.sort();
        out
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
        let Ok(mut q) = self
            .db
            .prepare("SELECT path,reason FROM problems ORDER BY path")
        else {
            return vec![];
        };
        let Ok(rows) = q.query_map([], |r| {
            Ok(serde_json::json!({"path": r.get::<_, String>(0)?, "reason": r.get::<_, String>(1)?}))
        }) else {
            return vec![];
        };
        rows.flatten().collect()
    }

    // ---- listing / search
    fn all_docs(&self) -> Vec<Doc> {
        let Ok(mut q) = self
            .db
            .prepare("SELECT id,root,path,ext,size,chars,pages,COALESCE(mtime_ns,0)/1000000000 FROM docs ORDER BY path")
        else {
            return vec![];
        };
        let Ok(rows) = q.query_map([], |r| {
            Ok(Doc {
                id: r.get(0)?,
                root: r.get(1)?,
                path: r.get(2)?,
                ext: r.get(3)?,
                size: r.get(4)?,
                chars: r.get(5)?,
                pages: r.get(6)?,
                mtime: r.get(7)?,
            })
        }) else {
            return vec![];
        };
        rows.flatten().collect()
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
        let q = clip(q.trim());
        if q.is_empty() {
            return Err("empty query".into());
        }
        let tok = Regex::new(r#""[^"]+"|\S+"#).unwrap();
        let word = Regex::new(r"\w+").unwrap();
        let mut out: Vec<String> = vec![];
        for m in tok.find_iter(q).take(MAX_QUERY_TERMS) {
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
                // Split like the index tokenizer does (e-mail -> "e mail"); combining marks stay.
                let core = t.trim_end_matches('*');
                let pieces: Vec<&str> = word.find_iter(core).map(|m| m.as_str()).collect();
                let star = if t.ends_with('*') { "*" } else { "" };
                match pieces.len() {
                    0 => {}
                    1 => out.push(format!("{}{star}", pieces[0])),
                    _ => out.push(format!(
                        "\"{}\"{}",
                        pieces.join(" "),
                        if star.is_empty() { "" } else { " *" }
                    )),
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
        let (hits, _) = self.search_core(query, f, true)?;
        Ok(hits)
    }

    /// `limit` is 2000 passages normally, 50000 when a filter will thin the list afterwards (so a
    /// filter never only sees the best 2000 passages of the whole library).
    fn fetch(&self, m: &str, limit: usize) -> rusqlite::Result<Vec<(i64, String, String, f64)>> {
        let mut q = self.db.prepare_cached(
            "SELECT doc_id, loc, text, bm25(chunks) FROM chunks WHERE chunks MATCH ?1 ORDER BY bm25(chunks) LIMIT ?2",
        )?;
        let rows = q.query_map(rusqlite::params![m, limit as i64], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?;
        rows.collect()
    }

    /// Passages in index order, for searches made only of filters (`ext:pdf`, `re:INV-\d+`, ...).
    fn scan(&self, limit: usize) -> rusqlite::Result<Vec<(i64, String, String, f64)>> {
        let mut q = self
            .db
            .prepare_cached("SELECT doc_id, loc, text, 0.0 FROM chunks LIMIT ?1")?;
        let rows = q.query_map([limit as i64], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?;
        rows.collect()
    }

    /// Returns the hits and the FTS match string that actually produced them.
    fn search_core(&self, query: &str, f: &Filter, fallback: bool) -> Res<(Vec<Hit>, String)> {
        let (inline, fi) = parse_inline(query, f);
        let f = &fi;
        let query = clip(&inline);
        let filtered = !f.ext.is_empty()
            || f.min_size.is_some()
            || f.max_size.is_some()
            || f.path_like.is_some()
            || f.group.is_some()
            || f.after.is_some()
            || f.before.is_some()
            || f.regex.is_some();
        if query.trim().is_empty() && filtered {
            // only filters, no words: list what they select (a scan, capped)
            let rows = self.scan(300_000).map_err(e)?;
            return Ok((self.rows_to_hits(rows, &[], f)?, String::new()));
        }
        let limit = if filtered { 50_000 } else { 2_000 };
        let word = Regex::new(r"\w+").unwrap();
        let plain: Vec<&str> = word
            .find_iter(query)
            .map(|m| m.as_str())
            .filter(|w| !matches!(*w, "AND" | "OR" | "NOT" | "NEAR"))
            .take(MAX_QUERY_TERMS)
            .collect();
        let mut used = Self::fts_query(query)?;
        let mut rows = match self.fetch(&used, limit) {
            Ok(r) => r,
            Err(_) => {
                // e.g. a dangling AND/NOT: fall back to plain quoted words, never a crash
                if plain.is_empty() {
                    return Ok((vec![], used));
                }
                used = plain
                    .iter()
                    .map(|w| format!("\"{w}\""))
                    .collect::<Vec<_>>()
                    .join(" ");
                self.fetch(&used, limit).map_err(e)?
            }
        };
        if rows.is_empty() && fallback && !plain.is_empty() {
            // inflection tolerant fallback: every word cut to a stem and matched as a prefix,
            // from the gentlest cut (plural / one-letter endings) to the roughest.
            let mut tried: Vec<String> = vec![];
            for level in 0..4 {
                let stems: Vec<String> = plain
                    .iter()
                    .map(|w| {
                        let n = w.chars().count();
                        let keep = match level {
                            0 => n.saturating_sub(1).max(3),
                            1 => n.saturating_sub(2).max(3),
                            2 => n.saturating_sub(3).max(3),
                            _ => 4.max((n as f64 * 0.7) as usize),
                        }
                        .min(n);
                        format!("{}*", w.chars().take(keep).collect::<String>())
                    })
                    .collect();
                let m = stems.join(" ");
                if tried.contains(&m) {
                    continue;
                }
                rows = self.fetch(&m, limit).unwrap_or_default();
                tried.push(m.clone());
                if !rows.is_empty() {
                    used = m;
                    break;
                }
            }
        }
        let terms = match_terms(&used);
        Ok((self.rows_to_hits(rows, &terms, f)?, used))
    }

    pub(crate) fn rows_to_hits(
        &self,
        rows: Vec<(i64, String, String, f64)>,
        terms: &[(String, bool)],
        f: &Filter,
    ) -> Res<Vec<Hit>> {
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
        let mut hits: HashMap<i64, u32> = HashMap::new();
        let mut best: Vec<(i64, String, String, f64)> = vec![];
        for r in rows {
            let c = hits.entry(r.0).or_insert(0);
            *c += 1;
            if *c == 1 {
                best.push(r);
            }
        }
        let re = match f.regex.as_deref() {
            Some(p) => Some(
                regex::RegexBuilder::new(p)
                    .size_limit(1 << 20)
                    .case_insensitive(true)
                    .build()
                    .map_err(|x| format!("bad regular expression: {x}"))?,
            ),
            None => None,
        };
        let docs: HashMap<i64, Doc> = self.all_docs().into_iter().map(|d| (d.id, d)).collect();
        let mut out: Vec<Hit> = vec![];
        for (did, loc, text, score) in best {
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
            if f.after.is_some_and(|t| d.mtime < t) || f.before.is_some_and(|t| d.mtime >= t) {
                continue;
            }
            if re.as_ref().is_some_and(|r| !r.is_match(&text)) {
                continue;
            }
            if f.path_like
                .as_ref()
                .is_some_and(|p| !d.path.to_lowercase().contains(&p.to_lowercase()))
            {
                continue;
            }
            let page = loc.strip_prefix("p.").and_then(|n| n.trim().parse().ok());
            out.push(Hit {
                id: did,
                path: d.path.clone(),
                ext: d.ext.clone(),
                size: d.size,
                loc,
                snippet: make_snippet(&text, terms),
                score: -score,
                matches: hits[&did],
                page,
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

    /// Search that forgives typos: exact -> corrected spelling -> word stems; if still nothing,
    /// "did you mean" suggestions are returned.
    pub fn search_friendly(&self, query: &str, f: &Filter) -> Res<FriendlyResult> {
        let (inline, fi) = parse_inline(query, f);
        let (query, f) = (inline.as_str(), &fi);
        let (hits, _) = self.search_core(query, f, false)?;
        if !hits.is_empty() {
            return Ok(FriendlyResult {
                hits,
                ..Default::default()
            });
        }
        let suggestions = self.suggest(query);
        if let Some(best) = suggestions.first()
            && let Ok((hits, _)) = self.search_core(best, f, false)
            && !hits.is_empty()
        {
            return Ok(FriendlyResult {
                hits,
                corrected: Some(best.clone()),
                suggestions,
            });
        }
        let (hits, _) = self.search_core(query, f, true)?;
        Ok(FriendlyResult {
            hits,
            corrected: None,
            suggestions,
        })
    }

    /// "Did you mean": full queries with unknown words replaced by close words from the
    /// index vocabulary (edit distance, bounded). Empty when every word is already known.
    pub fn suggest(&self, query: &str) -> Vec<String> {
        let query = clip(query);
        let word = Regex::new(r"\w+").unwrap();
        let spans: Vec<(usize, usize, String)> = word
            .find_iter(query)
            .filter(|m| !matches!(m.as_str(), "AND" | "OR" | "NOT" | "NEAR"))
            .take(MAX_SUGGEST_WORDS)
            .map(|m| (m.start(), m.end(), fold(m.as_str())))
            .collect();
        if spans.is_empty() {
            return vec![];
        }
        let vocab: Vec<(String, String, i64)> = (|| -> rusqlite::Result<_> {
            let mut q = self.db.prepare("SELECT term, doc FROM chunks_vocab")?;
            let v: Vec<(String, String, i64)> = q
                .query_map([], |r| {
                    let t: String = r.get(0)?;
                    Ok((fold(&t), t, r.get(1)?))
                })?
                .flatten()
                .collect();
            Ok(v)
        })()
        .unwrap_or_default();
        if vocab.is_empty() {
            return vec![];
        }
        // per word: ranked candidates (empty = word is known or nothing is close)
        let mut cands: Vec<Vec<String>> = vec![];
        for (_, _, w) in &spans {
            let n = w.chars().count();
            let max_d = match n {
                0..=3 => 0,
                4..=6 => 1,
                _ => 2,
            };
            if max_d == 0 || vocab.iter().any(|(f, _, _)| f == w) {
                cands.push(vec![]);
                continue;
            }
            let wc: Vec<char> = w.chars().collect();
            let mut found: Vec<(usize, i64, &str)> = vec![];
            for (f, t, docs) in &vocab {
                let fl = f.chars().count();
                if fl + max_d < n || n + max_d < fl {
                    continue;
                }
                let fc: Vec<char> = f.chars().collect();
                if let Some(d) = edit_distance(&wc, &fc, max_d) {
                    found.push((d, -*docs, t));
                }
            }
            found.sort();
            found.dedup_by(|a, b| a.2 == b.2);
            cands.push(found.into_iter().take(4).map(|x| x.2.to_string()).collect());
        }
        let first_bad = cands.iter().position(|c| !c.is_empty());
        let Some(fb) = first_bad else {
            return vec![];
        };
        let build = |pick: &dyn Fn(usize) -> Option<String>| -> String {
            let mut out = String::new();
            let mut last = 0;
            for (i, (s, en, _)) in spans.iter().enumerate() {
                out.push_str(&query[last..*s]);
                out.push_str(&pick(i).unwrap_or_else(|| query[*s..*en].to_string()));
                last = *en;
            }
            out.push_str(&query[last..]);
            out
        };
        let mut out: Vec<String> = vec![build(&|i| cands[i].first().cloned())];
        for alt in cands[fb].iter().skip(1) {
            out.push(build(&|i| {
                if i == fb {
                    Some(alt.clone())
                } else {
                    cands[i].first().cloned()
                }
            }));
        }
        out.retain(|q| q.trim() != query.trim());
        out.dedup();
        out
    }

    pub fn text_of(&self, doc_id: i64) -> String {
        let Ok(mut q) = self
            .db
            .prepare("SELECT loc,text FROM chunks WHERE doc_id=? ORDER BY CAST(seq AS INTEGER)")
        else {
            return String::new();
        };
        let rows: Vec<(String, String)> =
            match q.query_map([doc_id], |r| Ok((r.get(0)?, r.get(1)?))) {
                Ok(it) => it.flatten().collect(),
                Err(_) => return String::new(),
            };
        join_chunks(rows)
    }

    /// The text of many documents in ONE pass over the passages. `chunks` is an FTS5 table whose
    /// `doc_id` column is not indexed, so calling `text_of` once per document scans the whole
    /// table every time (quadratic); whole-library analyses must use this instead.
    pub fn texts_of(&self, ids: &[i64]) -> std::collections::HashMap<i64, String> {
        use std::collections::{HashMap, HashSet};
        let want: HashSet<i64> = ids.iter().copied().collect();
        let Ok(mut q) = self
            .db
            .prepare("SELECT doc_id,CAST(seq AS INTEGER),loc,text FROM chunks")
        else {
            return HashMap::new();
        };
        let Ok(rows) = q.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        }) else {
            return HashMap::new();
        };
        let mut by: HashMap<i64, Vec<(i64, String, String)>> = HashMap::new();
        for (id, seq, loc, text) in rows.flatten() {
            if want.contains(&id) {
                by.entry(id).or_default().push((seq, loc, text));
            }
        }
        by.into_iter()
            .map(|(id, mut v)| {
                v.sort_by_key(|x| x.0);
                (id, join_chunks(v.into_iter().map(|x| (x.1, x.2)).collect()))
            })
            .collect()
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

/// Whether two files hold the same bytes (None when either cannot be read).
fn same_bytes(a: &Path, b: &Path) -> Option<bool> {
    use std::io::Read;
    let (mut fa, mut fb) = (
        extract::open_regular(a).ok()?,
        extract::open_regular(b).ok()?,
    );
    if fa.metadata().ok()?.len() != fb.metadata().ok()?.len() {
        return Some(false);
    }
    let (mut x, mut y) = (vec![0u8; 64 * 1024], vec![0u8; 64 * 1024]);
    loop {
        let n = read_full(&mut fa, &mut x)?;
        let m = read_full(&mut fb, &mut y)?;
        if n != m || x[..n] != y[..m] {
            return Some(false);
        }
        if n == 0 {
            return Some(true);
        }
    }
    fn read_full(f: &mut std::fs::File, buf: &mut [u8]) -> Option<usize> {
        let mut got = 0;
        while got < buf.len() {
            match f.read(&mut buf[got..]) {
                Ok(0) => break,
                Ok(k) => got += k,
                Err(_) => return None,
            }
        }
        Some(got)
    }
}

/// Lower-case and strip diacritics, one char to one char (ș ş ț ţ ä ö ü é è ñ ã õ ő ű ...).
pub fn fold_char(c: char) -> char {
    let c = c.to_lowercase().next().unwrap_or(c);
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
        'ç' | 'ć' | 'č' | 'ĉ' => 'c',
        'ď' | 'đ' => 'd',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ě' | 'ę' | 'ė' => 'e',
        'ğ' => 'g',
        'ì' | 'í' | 'î' | 'ï' | 'ī' | 'ı' => 'i',
        'ł' => 'l',
        'ñ' | 'ń' | 'ň' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ő' | 'ō' => 'o',
        'ř' => 'r',
        'ș' | 'ş' | 'ś' | 'š' => 's',
        'ț' | 'ţ' | 'ť' => 't',
        'ù' | 'ú' | 'û' | 'ü' | 'ű' | 'ū' | 'ů' => 'u',
        'ý' | 'ÿ' => 'y',
        'ź' | 'ż' | 'ž' => 'z',
        c => c,
    }
}

pub fn fold(s: &str) -> String {
    s.chars().map(fold_char).collect()
}

/// Optimal-string-alignment distance (adjacent swaps count 1); None when above `max`.
fn edit_distance(a: &[char], b: &[char], max: usize) -> Option<usize> {
    if a.len().abs_diff(b.len()) > max {
        return None;
    }
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, x) in d[0].iter_mut().enumerate() {
        *x = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let c = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + c);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    (d[n][m] <= max).then_some(d[n][m])
}

/// Words of an FTS match string as (folded word, is_prefix); words after NOT are ignored.
fn match_terms(m: &str) -> Vec<(String, bool)> {
    let mut out = vec![];
    let mut after_not = false;
    let tok = Regex::new(r#""[^"]*"|\S+"#).unwrap();
    let word = Regex::new(r"\w+").unwrap();
    for t in tok.find_iter(m) {
        let t = t.as_str();
        if t == "NOT" {
            after_not = true;
            continue;
        }
        if matches!(t, "AND" | "OR") {
            continue;
        }
        if after_not {
            after_not = false;
            continue;
        }
        let prefix = t.ends_with('*');
        let ws: Vec<&str> = word.find_iter(t).map(|w| w.as_str()).collect();
        for (i, w) in ws.iter().enumerate() {
            out.push((fold(w), prefix && i + 1 == ws.len()));
        }
    }
    out
}

/// A sentence-trimmed excerpt of `text` around the first match, every matched word in [brackets].
fn make_snippet(text: &str, terms: &[(String, bool)]) -> String {
    let chars: Vec<char> = text
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();
    let n = chars.len();
    // matched word spans
    let mut spans: Vec<(usize, usize)> = vec![];
    let mut i = 0;
    while i < n {
        if chars[i].is_alphanumeric() || chars[i] == '_' {
            let s = i;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let w: String = chars[s..i].iter().map(|&c| fold_char(c)).collect();
            if terms
                .iter()
                .any(|(t, pre)| !t.is_empty() && (w == *t || (*pre && w.starts_with(t.as_str()))))
            {
                spans.push((s, i));
            }
        } else {
            i += 1;
        }
    }
    let is_end = |k: usize| matches!(chars[k], '.' | '!' | '?' | '…' | ';');
    let first = spans.first().map(|s| s.0).unwrap_or(0);
    // start: back to a sentence boundary within 140 chars
    let mut start = first.saturating_sub(140);
    let mut k = first;
    let mut at_boundary = first == 0;
    while k > start {
        if is_end(k - 1) && chars.get(k).is_some_and(|c| *c == ' ') {
            start = k + 1;
            at_boundary = true;
            break;
        }
        k -= 1;
    }
    if !at_boundary && start > 0 {
        // cut at a word boundary
        while start < first && chars[start] != ' ' {
            start += 1;
        }
        start += 1;
    }
    let start = start.min(first);
    // end: sentence end after the first match, extended to ~120 chars, capped at 320
    let mut end = n.min(first + 1);
    let mut j = first;
    let mut closed = false;
    while j < n && j - start < 320 {
        if is_end(j) && j - start >= 120.min(n) {
            end = j + 1;
            closed = true;
            break;
        }
        j += 1;
    }
    if !closed {
        end = if j >= n {
            n
        } else {
            let mut e = j;
            while e > first && chars[e] != ' ' {
                e -= 1;
            }
            e.max(first + 1)
        };
    }
    let mut out = String::new();
    if start > 0 {
        out.push_str("… ");
    }
    let mut p = start;
    for &(s, e2) in spans.iter().filter(|s| s.0 >= start && s.1 <= end) {
        out.extend(&chars[p..s]);
        out.push('[');
        out.extend(&chars[s..e2]);
        out.push(']');
        p = e2;
    }
    out.extend(&chars[p..end.max(p)]);
    let mut out = out.trim().to_string();
    if end < n {
        out.push_str(" …");
    }
    out
}

fn join_chunks(rows: Vec<(String, String)>) -> String {
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
