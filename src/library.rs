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
            let prefix = format!("{}{}", root.trim_end_matches('/'), "/");
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
                let res: Res<bool> = (|| {
                    let raw = {
                        use std::io::Read;
                        let f = std::fs::File::open(&p)
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
                if !seen.contains(&path) && !Path::new(&path).exists() {
                    tx.execute("DELETE FROM chunks WHERE doc_id=?", [id])
                        .map_err(e)?;
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
        let mut q = self
            .db
            .prepare(
                "SELECT sha1, path FROM docs WHERE size>0 AND sha1 IN
                 (SELECT sha1 FROM docs WHERE size>0 GROUP BY sha1 HAVING count(*)>1)
                 ORDER BY sha1, path",
            )
            .unwrap();
        let rows: Vec<(String, String)> = q
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .flatten()
            .collect();
        let mut by: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (h, p) in rows {
            by.entry(h).or_default().push(p);
        }
        let mut out: Vec<Vec<String>> = by.into_values().collect();
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
        let (hits, _) = self.search_core(query, f, true)?;
        Ok(hits)
    }

    fn fetch(&self, m: &str) -> rusqlite::Result<Vec<(i64, String, String, f64)>> {
        let mut q = self.db.prepare_cached(
            "SELECT doc_id, loc, text, bm25(chunks) FROM chunks WHERE chunks MATCH ?1 ORDER BY bm25(chunks) LIMIT 2000",
        )?;
        let rows = q.query_map([m], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
        rows.collect()
    }

    /// Returns the hits and the FTS match string that actually produced them.
    fn search_core(&self, query: &str, f: &Filter, fallback: bool) -> Res<(Vec<Hit>, String)> {
        let word = Regex::new(r"\w+").unwrap();
        let plain: Vec<&str> = word
            .find_iter(query)
            .map(|m| m.as_str())
            .filter(|w| !matches!(*w, "AND" | "OR" | "NOT" | "NEAR"))
            .collect();
        let mut used = Self::fts_query(query)?;
        let mut rows = match self.fetch(&used) {
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
                self.fetch(&used).map_err(e)?
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
                rows = self.fetch(&m).unwrap_or_default();
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

    fn rows_to_hits(
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
        let word = Regex::new(r"\w+").unwrap();
        let spans: Vec<(usize, usize, String)> = word
            .find_iter(query)
            .filter(|m| !matches!(m.as_str(), "AND" | "OR" | "NOT" | "NEAR"))
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
