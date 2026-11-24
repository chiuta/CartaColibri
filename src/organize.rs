//! Organizing features on top of [`Library`]: tags, stars, notes, saved searches / smart
//! collections, recent files, related documents, export / import, backup / restore, index
//! verification, library statistics and a timeline.
//!
//! User data (tags, notes, ...) lives in `<home>/organize.json`, keyed by the document's content
//! path, so it survives re-indexing (and index rebuilds).

use crate::library::{Doc, Filter, Hit, Library, Res, fold};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn e<E: std::fmt::Display>(x: E) -> String {
    x.to_string()
}

fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// (year, month, day) from days since 1970-01-01.
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn iso(secs: f64) -> String {
    let s = secs as i64;
    let (y, m, d) = civil(s.div_euclid(86_400));
    let t = s.rem_euclid(86_400);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        t / 3600,
        t % 3600 / 60,
        t % 60
    )
}

fn ym(mtime_ns: i64) -> (i64, u32) {
    let (y, m, _) = civil((mtime_ns / 1_000_000_000).div_euclid(86_400));
    (y, m)
}

// ---------------------------------------------------------------- state

/// User data attached to one document.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Meta {
    #[serde(default)]
    pub tags: BTreeSet<String>,
    #[serde(default)]
    pub starred: bool,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub opens: u32,
    /// Unix seconds of the last open, 0 = never.
    #[serde(default)]
    pub last_opened: f64,
}

impl Meta {
    fn is_empty(&self) -> bool {
        self.tags.is_empty() && !self.starred && self.note.is_empty() && self.opens == 0
    }
}

/// A named search; with an empty `query` it selects documents by filter / tag / star only.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SavedSearch {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub ext: Vec<String>,
    #[serde(default)]
    pub min_size: Option<i64>,
    #[serde(default)]
    pub max_size: Option<i64>,
    #[serde(default)]
    pub path_like: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub starred_only: bool,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Default)]
struct OrgState {
    #[serde(default)]
    meta: BTreeMap<String, Meta>,
    #[serde(default)]
    saved: BTreeMap<String, SavedSearch>,
}

fn state_path(lib: &Library) -> PathBuf {
    lib.home.join("organize.json")
}

/// Lenient read for queries: a missing or unreadable file means "no user data yet".
fn load(lib: &Library) -> OrgState {
    std::fs::read_to_string(state_path(lib))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Strict read for edits. A corrupt `organize.json` is moved aside (never silently replaced by an
/// empty state, which would destroy the user's tags and notes) and the edit is refused once.
fn load_strict(lib: &Library) -> Res<OrgState> {
    let p = state_path(lib);
    let s = match std::fs::read_to_string(&p) {
        Ok(s) => s,
        Err(x) if x.kind() == std::io::ErrorKind::NotFound => return Ok(OrgState::default()),
        Err(x) => return Err(format!("cannot read {}: {x}", p.display())),
    };
    if s.trim().is_empty() {
        return Ok(OrgState::default());
    }
    match serde_json::from_str(&s) {
        Ok(st) => Ok(st),
        Err(x) => {
            let aside = p.with_extension(format!("json.corrupt-{}", now() as u64));
            let moved = std::fs::rename(&p, &aside).is_ok();
            Err(format!(
                "organize.json is damaged ({x}); {} Nothing was changed, try again to start with an empty state.",
                if moved {
                    format!("it was kept as {}.", aside.display())
                } else {
                    "it could not be moved aside.".to_string()
                }
            ))
        }
    }
}

fn save(lib: &Library, mut st: OrgState) -> Res<()> {
    st.meta.retain(|_, m| !m.is_empty());
    let p = state_path(lib);
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let tmp = p.with_extension(format!(
        "json.{}.{}.tmp",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let write = || -> std::io::Result<()> {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(
            serde_json::to_string_pretty(&st)
                .map_err(std::io::Error::other)?
                .as_bytes(),
        )?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, &p)
    };
    write().map_err(|x| {
        let _ = std::fs::remove_file(&tmp);
        e(x)
    })
}

/// Cross-process lock for read-modify-write of organize.json (a lock file; stale after 30 s).
struct FileLock(PathBuf);

impl FileLock {
    fn acquire(p: PathBuf) -> Res<FileLock> {
        let start = std::time::Instant::now();
        loop {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&p)
            {
                Ok(_) => return Ok(FileLock(p)),
                Err(x) if x.kind() == std::io::ErrorKind::AlreadyExists => {
                    let stale = std::fs::metadata(&p)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .is_some_and(|d| d.as_secs() > 30);
                    if stale {
                        let _ = std::fs::remove_file(&p);
                    } else if start.elapsed().as_secs() >= 10 {
                        return Err(
                            "organize.json is locked by another CartaColibri process".into()
                        );
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(15));
                    }
                }
                Err(x) => return Err(e(x)),
            }
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn edit<T>(lib: &Library, f: impl FnOnce(&mut OrgState) -> T) -> Res<T> {
    static GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _g = GATE.lock().unwrap_or_else(|p| p.into_inner());
    let _fl = FileLock::acquire(lib.home.join("organize.lock"))?;
    let mut st = load_strict(lib)?;
    let r = f(&mut st);
    save(lib, st)?;
    Ok(r)
}

/// Longest tag (characters) and note (characters) that are stored.
pub const TAG_MAX: usize = 64;
pub const NOTE_MAX: usize = 100_000;

/// Lower-case, no leading `#`, single spaces, no control characters, at most [`TAG_MAX`] chars.
fn clean_tag(t: &str) -> String {
    let t: String = t
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    t.trim_start_matches('#')
        .trim()
        .to_lowercase()
        .chars()
        .take(TAG_MAX)
        .collect::<String>()
        .trim()
        .to_string()
}

fn need_path(path: &str) -> Res<()> {
    if path.trim().is_empty() {
        Err("a document path is required".into())
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------- tags, stars, notes

/// Add tags (case-insensitive, stored lower-case). Returns the document's tags.
pub fn tag_add(lib: &Library, path: &str, tags: &[String]) -> Res<Vec<String>> {
    need_path(path)?;
    edit(lib, |st| {
        let m = st.meta.entry(path.to_string()).or_default();
        for t in tags.iter().map(|t| clean_tag(t)).filter(|t| !t.is_empty()) {
            m.tags.insert(t);
        }
        m.tags.iter().cloned().collect()
    })
}

/// Remove tags; an empty list removes all of them.
pub fn tag_remove(lib: &Library, path: &str, tags: &[String]) -> Res<Vec<String>> {
    edit(lib, |st| {
        let Some(m) = st.meta.get_mut(path) else {
            return vec![];
        };
        if tags.is_empty() {
            m.tags.clear();
        } else {
            for t in tags {
                m.tags.remove(&clean_tag(t));
            }
        }
        m.tags.iter().cloned().collect()
    })
}

/// Every tag with the number of documents carrying it.
pub fn tag_list(lib: &Library) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for m in load(lib).meta.values() {
        for t in &m.tags {
            *out.entry(t.clone()).or_insert(0) += 1;
        }
    }
    out
}

pub fn tags_of(lib: &Library, path: &str) -> Vec<String> {
    meta_of(lib, path).tags.into_iter().collect()
}

/// Paths carrying a tag.
pub fn paths_with_tag(lib: &Library, tag: &str) -> Vec<String> {
    let t = clean_tag(tag);
    load(lib)
        .meta
        .into_iter()
        .filter(|(_, m)| m.tags.contains(&t))
        .map(|(p, _)| p)
        .collect()
}

pub fn set_star(lib: &Library, path: &str, on: bool) -> Res<()> {
    need_path(path)?;
    edit(lib, |st| {
        st.meta.entry(path.to_string()).or_default().starred = on
    })
}

pub fn starred(lib: &Library) -> Vec<String> {
    load(lib)
        .meta
        .into_iter()
        .filter(|(_, m)| m.starred)
        .map(|(p, _)| p)
        .collect()
}

/// Set (or clear, with an empty string) the free-text note.
pub fn set_note(lib: &Library, path: &str, note: &str) -> Res<()> {
    need_path(path)?;
    if note.chars().count() > NOTE_MAX {
        return Err(format!("note too long (max {NOTE_MAX} characters)"));
    }
    edit(lib, |st| {
        st.meta.entry(path.to_string()).or_default().note = note.to_string()
    })
}

pub fn note_of(lib: &Library, path: &str) -> String {
    meta_of(lib, path).note
}

pub fn meta_of(lib: &Library, path: &str) -> Meta {
    load(lib).meta.remove(path).unwrap_or_default()
}

/// All user data, by path.
pub fn all_meta(lib: &Library) -> BTreeMap<String, Meta> {
    load(lib).meta
}

/// Search, then keep only hits with `tag` and/or the star.
pub fn search_tagged(
    lib: &Library,
    query: &str,
    f: &Filter,
    tag: Option<&str>,
    starred_only: bool,
) -> Res<Vec<Hit>> {
    let st = load(lib);
    let tag = tag.map(clean_tag).filter(|t| !t.is_empty());
    let ok = |p: &str| {
        let m = st.meta.get(p);
        tag.as_ref()
            .is_none_or(|t| m.is_some_and(|m| m.tags.contains(t)))
            && (!starred_only || m.is_some_and(|m| m.starred))
    };
    let limit = f.limit.unwrap_or(50);
    if query.trim().is_empty() {
        // no text query: list matching documents
        return Ok(filtered_docs(lib, f)?
            .into_iter()
            .filter(|d| ok(&d.path))
            .take(limit)
            .map(doc_hit)
            .collect());
    }
    let mut wide = f.clone();
    wide.limit = Some(100_000);
    let mut hits: Vec<Hit> = lib.search_friendly(query, &wide)?.hits;
    hits.retain(|h| ok(&h.path));
    hits.truncate(limit);
    Ok(hits)
}

fn filtered_docs(lib: &Library, f: &Filter) -> Res<Vec<Doc>> {
    let exts: Vec<String> = f
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
    Ok(lib
        .documents(f.group.as_deref())?
        .into_iter()
        .filter(|d| exts.is_empty() || exts.contains(&d.ext))
        .filter(|d| {
            f.min_size.is_none_or(|m| d.size >= m) && f.max_size.is_none_or(|m| d.size <= m)
        })
        .filter(|d| {
            f.path_like
                .as_ref()
                .is_none_or(|p| d.path.to_lowercase().contains(&p.to_lowercase()))
        })
        .collect())
}

fn doc_hit(d: Doc) -> Hit {
    Hit {
        id: d.id,
        path: d.path,
        ext: d.ext,
        size: d.size,
        loc: String::new(),
        snippet: String::new(),
        score: 100.0,
        matches: 0,
        page: None,
    }
}

// ---------------------------------------------------------------- saved searches / collections

pub fn saved_set(lib: &Library, s: SavedSearch) -> Res<()> {
    if s.name.trim().is_empty() {
        return Err("a saved search needs a name".into());
    }
    edit(lib, |st| {
        st.saved.insert(s.name.clone(), s);
    })
}

pub fn saved_list(lib: &Library) -> Vec<SavedSearch> {
    load(lib).saved.into_values().collect()
}

pub fn saved_remove(lib: &Library, name: &str) -> Res<bool> {
    edit(lib, |st| st.saved.remove(name).is_some())
}

/// Re-run a saved search / smart collection against the current index.
pub fn saved_run(lib: &Library, name: &str) -> Res<Vec<Hit>> {
    let s = load(lib)
        .saved
        .remove(name)
        .ok_or_else(|| format!("no saved search '{name}'"))?;
    let f = Filter {
        ext: s.ext.clone(),
        min_size: s.min_size,
        max_size: s.max_size,
        path_like: s.path_like.clone(),
        group: s.group.clone(),
        limit: s.limit,
    };
    search_tagged(lib, &s.query, &f, s.tag.as_deref(), s.starred_only)
}

// ---------------------------------------------------------------- recent / related

/// Count an open of a document (call when the user opens / views it).
pub fn record_open(lib: &Library, path: &str) -> Res<()> {
    edit(lib, |st| {
        let m = st.meta.entry(path.to_string()).or_default();
        m.opens = m.opens.saturating_add(1);
        m.last_opened = now();
    })
}

/// Most recently opened paths, newest first.
pub fn recent(lib: &Library, n: usize) -> Vec<String> {
    let mut v: Vec<(String, f64)> = load(lib)
        .meta
        .into_iter()
        .filter(|(_, m)| m.opens > 0)
        .map(|(p, m)| (p, m.last_opened))
        .collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.into_iter().take(n).map(|x| x.0).collect()
}

/// Most opened paths with their open counts.
pub fn most_opened(lib: &Library, n: usize) -> Vec<(String, u32)> {
    let mut v: Vec<(String, u32)> = load(lib)
        .meta
        .into_iter()
        .filter(|(_, m)| m.opens > 0)
        .map(|(p, m)| (p, m.opens))
        .collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

/// Documents sharing the rarest terms of `doc_id`, best first (at most `k`).
pub fn related(lib: &Library, doc_id: i64, k: usize) -> Res<Vec<Hit>> {
    let text = lib.text_of(doc_id);
    let n_docs: f64 = lib
        .conn()
        .query_row("SELECT count(*) FROM docs", [], |r| r.get::<_, i64>(0))
        .map_err(e)? as f64;
    let mut tf: HashMap<String, u32> = HashMap::new();
    for w in text.split(|c: char| !c.is_alphanumeric()) {
        if w.chars().count() >= 4 && !w.chars().all(|c| c.is_ascii_digit()) {
            *tf.entry(fold(w)).or_insert(0) += 1;
        }
    }
    let mut q = lib
        .conn()
        .prepare("SELECT doc FROM chunks_vocab WHERE term=?")
        .map_err(e)?;
    let mut scored: Vec<(String, f64)> = vec![];
    for (w, c) in tf {
        let df: Option<i64> = q.query_row([&w], |r| r.get(0)).ok();
        let Some(df) = df.filter(|d| *d >= 2) else {
            continue; // appears only here: cannot relate to anything
        };
        let idf = ((n_docs + 1.0) / df as f64).ln();
        scored.push((w, (1.0 + (c as f64).ln()) * idf));
    }
    scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scored.truncate(16);
    if scored.is_empty() || k == 0 {
        return Ok(vec![]);
    }
    let fts = scored
        .iter()
        .map(|(w, _)| format!("\"{w}\""))
        .collect::<Vec<_>>()
        .join(" OR ");
    let f = Filter {
        limit: Some(k.saturating_add(1)),
        ..Filter::default()
    };
    let mut hits = lib.search(&fts, &f)?;
    hits.retain(|h| h.id != doc_id);
    hits.truncate(k);
    Ok(hits)
}

// ---------------------------------------------------------------- export / import

/// One exported document with its citation data and user data.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ExportItem {
    pub path: String,
    pub title: String,
    #[serde(default)]
    pub ext: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub loc: String,
    #[serde(default)]
    pub snippet: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub starred: bool,
    #[serde(default)]
    pub note: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExportFile {
    pub format: String,
    pub version: u32,
    pub created: String,
    pub items: Vec<ExportItem>,
}

const EXPORT_FORMAT: &str = "cartacolibri-export";

fn title_of(path: &str) -> String {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    name.rsplit_once('.').map_or(name, |x| x.0).to_string()
}

fn item(path: &str, ext: &str, size: i64, loc: &str, snippet: &str, st: &OrgState) -> ExportItem {
    let m = st.meta.get(path).cloned().unwrap_or_default();
    ExportItem {
        path: path.into(),
        title: title_of(path),
        ext: ext.into(),
        size,
        loc: loc.into(),
        snippet: snippet.into(),
        tags: m.tags.into_iter().collect(),
        starred: m.starred,
        note: m.note,
    }
}

pub fn items_from_hits(lib: &Library, hits: &[Hit]) -> Vec<ExportItem> {
    let st = load(lib);
    hits.iter()
        .map(|h| item(&h.path, &h.ext, h.size, &h.loc, &h.snippet, &st))
        .collect()
}

pub fn items_from_docs(lib: &Library, docs: &[Doc]) -> Vec<ExportItem> {
    let st = load(lib);
    docs.iter()
        .map(|d| item(&d.path, &d.ext, d.size, "", "", &st))
        .collect()
}

/// A group (see `Library::groups`) as export items.
pub fn items_from_group(lib: &Library, group: &str) -> Res<Vec<ExportItem>> {
    Ok(items_from_docs(lib, &lib.resolve_group(group)?))
}

/// A smart collection / saved search as export items.
pub fn items_from_saved(lib: &Library, name: &str) -> Res<Vec<ExportItem>> {
    Ok(items_from_hits(lib, &saved_run(lib, name)?))
}

pub fn to_markdown(items: &[ExportItem]) -> String {
    let mut s = String::from("# Bibliography\n\n");
    for (i, it) in items.iter().enumerate() {
        s += &format!("{}. **{}**", i + 1, it.title);
        if !it.loc.is_empty() {
            s += &format!(", {}", it.loc);
        }
        s += &format!(" (`{}`", it.path);
        if !it.ext.is_empty() {
            s += &format!(", {}", it.ext.trim_start_matches('.'));
        }
        s += ")";
        if it.starred {
            s += " *";
        }
        if !it.tags.is_empty() {
            s += &format!(" [{}]", it.tags.join(", "));
        }
        s += "\n";
        if !it.snippet.is_empty() {
            s += &format!("   > {}\n", one_line(&it.snippet));
        }
        if !it.note.is_empty() {
            s += &format!("   Note: {}\n", one_line(&it.note));
        }
    }
    s
}

/// Collapse every run of whitespace/control characters (newlines included) into one space.
fn one_line(s: &str) -> String {
    s.split(|c: char| c.is_whitespace() || c.is_control())
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// One CSV cell. A cell that a spreadsheet would run as a formula (`=`, `+`, `-`, `@`, tab, CR)
/// gets a leading apostrophe; cells with commas, quotes or line breaks are quoted.
fn csv_field(s: &str) -> String {
    let guarded;
    let s = if s.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        guarded = format!("'{s}");
        guarded.as_str()
    } else {
        s
    };
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn to_csv(items: &[ExportItem]) -> String {
    let mut s = String::from("title,path,ext,size,loc,tags,starred,note,snippet\n");
    for it in items {
        let row = [
            it.title.clone(),
            it.path.clone(),
            it.ext.clone(),
            it.size.to_string(),
            it.loc.clone(),
            it.tags.join(";"),
            it.starred.to_string(),
            it.note.clone(),
            it.snippet.clone(),
        ];
        s += &row
            .iter()
            .enumerate()
            .map(|(i, f)| if i == 3 { f.clone() } else { csv_field(f) })
            .collect::<Vec<_>>()
            .join(",");
        s += "\n";
    }
    s
}

pub fn to_json(items: &[ExportItem]) -> String {
    let f = ExportFile {
        format: EXPORT_FORMAT.into(),
        version: 1,
        created: iso(now()),
        items: items.to_vec(),
    };
    serde_json::to_string_pretty(&f).unwrap_or_default()
}

/// Write a zip with `sources/` (the files that still exist), `citations.md`, `index.json`, `index.csv`.
/// Returns the number of source files bundled.
pub fn export_zip(items: &[ExportItem], out: &Path) -> Res<usize> {
    // never truncate one of the documents being exported (or a directory) by mistake
    if out.is_dir() {
        return Err(format!("{} is a folder, give a file name", out.display()));
    }
    if let Ok(oc) = std::fs::canonicalize(out)
        && items
            .iter()
            .any(|it| std::fs::canonicalize(&it.path).is_ok_and(|p| p == oc))
    {
        return Err(format!(
            "{} is one of the documents to export; choose another output file",
            out.display()
        ));
    }
    let r = export_zip_inner(items, out);
    if r.is_err() {
        let _ = std::fs::remove_file(out);
    }
    r
}

fn export_zip_inner(items: &[ExportItem], out: &Path) -> Res<usize> {
    let f = std::fs::File::create(out).map_err(e)?;
    let mut z = zip::ZipWriter::new(f);
    let o = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut used = BTreeSet::new();
    let mut shown = items.to_vec();
    let mut n = 0;
    for it in shown.iter_mut() {
        let p = Path::new(&it.path);
        if !p.is_file() {
            continue;
        }
        let base = p
            .file_name()
            .map_or("file".into(), |x| x.to_string_lossy().into_owned());
        let mut name = base.clone();
        let mut i = 1;
        while !used.insert(name.clone()) {
            name = format!("{i}-{base}");
            i += 1;
        }
        z.start_file(format!("sources/{name}"), o).map_err(e)?;
        z.write_all(&std::fs::read(p).map_err(e)?).map_err(e)?;
        n += 1;
    }
    for (name, body) in [
        ("citations.md", to_markdown(&shown)),
        ("index.json", to_json(&shown)),
        ("index.csv", to_csv(&shown)),
    ] {
        z.start_file(name, o).map_err(e)?;
        z.write_all(body.as_bytes()).map_err(e)?;
    }
    z.finish().map_err(e)?;
    Ok(n)
}

#[derive(Serialize, Debug, Default, PartialEq)]
pub struct ImportReport {
    pub items: usize,
    pub tagged: usize,
    pub starred: usize,
    pub notes: usize,
}

/// Re-apply tags / stars / notes from a JSON export (merging with what exists).
pub fn import_json(lib: &Library, json: &str) -> Res<ImportReport> {
    let f: ExportFile =
        serde_json::from_str(json).map_err(|x| format!("not an export file: {x}"))?;
    if f.format != EXPORT_FORMAT {
        return Err("not a CartaColibri export file".into());
    }
    let mut rep = ImportReport {
        items: f.items.len(),
        ..Default::default()
    };
    edit(lib, |st| {
        for it in f.items {
            if it.path.trim().is_empty() {
                continue;
            }
            let m = st.meta.entry(it.path).or_default();
            if !it.tags.is_empty() {
                rep.tagged += 1;
                m.tags.extend(
                    it.tags
                        .iter()
                        .take(200)
                        .map(|t| clean_tag(t))
                        .filter(|t| !t.is_empty()),
                );
            }
            if it.starred {
                rep.starred += 1;
                m.starred = true;
            }
            if !it.note.is_empty() {
                rep.notes += 1;
                let imported: String = it.note.chars().take(NOTE_MAX).collect();
                if m.note.is_empty() || m.note == imported {
                    m.note = imported;
                } else if m.note.chars().count() + imported.chars().count() < NOTE_MAX {
                    // keep the local note: the imported one is appended, not overwritten
                    m.note = format!("{}\n---\n{imported}", m.note);
                }
            }
        }
    })?;
    Ok(rep)
}

// ---------------------------------------------------------------- backup / restore

pub const BACKUP_VERSION: u32 = 1;
const MANIFEST: &str = "manifest.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupFile {
    pub name: String,
    pub size: u64,
    pub sha1: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub format: String,
    pub version: u32,
    pub app_version: String,
    pub created: String,
    pub includes_index: bool,
    pub files: Vec<BackupFile>,
}

fn sha1_hex(b: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    hex::encode(Sha1::digest(b))
}

fn collect(dir: &Path, base: &Path, skip_index: bool, out: &mut Vec<(String, PathBuf)>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut v: Vec<_> = rd.flatten().collect();
    v.sort_by_key(|x| x.file_name());
    for en in v {
        let p = en.path();
        let Ok(rel) = p.strip_prefix(base) else {
            continue;
        };
        // symlinks are neither followed (loops) nor stored (they could point anywhere)
        if en.file_type().is_ok_and(|t| t.is_symlink()) {
            continue;
        }
        let rel = rel.to_string_lossy().replace('\\', "/");
        let fname = en.file_name().to_string_lossy().into_owned();
        if en.file_type().is_ok_and(|t| t.is_dir()) {
            collect(&p, base, skip_index, out);
        } else if fname.ends_with(".tmp")
            || fname.ends_with(".lock")
            || (fname.starts_with("backup-") && fname.ends_with(".zip"))
            || (skip_index && fname.starts_with("index.sqlite"))
        {
            continue;
        } else {
            out.push((rel, p));
        }
    }
}

/// Zip the whole `<home>` state (groups, tags, saved searches, uploads, ...), with a manifest
/// carrying a format version and a SHA-1 per file. `include_index=false` leaves the (rebuildable)
/// index out. The folders list is saved as `roots.json` so a restore without index can re-register them.
pub fn backup(lib: &Library, out: &Path, include_index: bool) -> Res<Manifest> {
    if out.is_dir() {
        return Err(format!("{} is a folder, give a file name", out.display()));
    }
    let r = backup_inner(lib, out, include_index);
    if r.is_err() {
        let _ = std::fs::remove_file(out);
    }
    r
}

fn backup_inner(lib: &Library, out: &Path, include_index: bool) -> Res<Manifest> {
    let _ = lib.conn().execute_batch("PRAGMA wal_checkpoint(TRUNCATE)");
    std::fs::write(
        lib.home.join("roots.json"),
        serde_json::to_string_pretty(&lib.roots()).map_err(e)?,
    )
    .map_err(e)?;
    let mut files = vec![];
    collect(&lib.home, &lib.home, !include_index, &mut files);
    let outc = std::fs::canonicalize(out.parent().unwrap_or(Path::new(".")))
        .ok()
        .map(|d| d.join(out.file_name().unwrap_or_default()));
    files.retain(|(_, p)| std::fs::canonicalize(p).ok() != outc);
    // the clip token is a secret of this machine: never put it in a file people copy around
    files.retain(|(rel, _)| rel != "clip.token" && !rel.starts_with("clip.token."));
    let f = std::fs::File::create(out).map_err(e)?;
    let mut z = zip::ZipWriter::new(f);
    let o = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut listed = vec![];
    for (rel, p) in files {
        let data = std::fs::read(&p).map_err(e)?;
        listed.push(BackupFile {
            name: rel.clone(),
            size: data.len() as u64,
            sha1: sha1_hex(&data),
        });
        z.start_file(format!("state/{rel}"), o).map_err(e)?;
        z.write_all(&data).map_err(e)?;
    }
    let man = Manifest {
        format: "cartacolibri-backup".into(),
        version: BACKUP_VERSION,
        app_version: env!("CARGO_PKG_VERSION").into(),
        created: iso(now()),
        includes_index: include_index,
        files: listed,
    };
    z.start_file(MANIFEST, o).map_err(e)?;
    z.write_all(serde_json::to_string_pretty(&man).map_err(e)?.as_bytes())
        .map_err(e)?;
    z.finish().map_err(e)?;
    Ok(man)
}

type BackupData = (Manifest, Vec<(String, Vec<u8>)>);

fn read_backup(zip_path: &Path) -> Res<BackupData> {
    let f = std::fs::File::open(zip_path).map_err(e)?;
    let mut z = zip::ZipArchive::new(f).map_err(|x| format!("not a backup (bad zip): {x}"))?;
    let man: Manifest = {
        let mut s = String::new();
        z.by_name(MANIFEST)
            .map_err(|_| "not a CartaColibri backup: manifest missing".to_string())?
            .take(32 << 20)
            .read_to_string(&mut s)
            .map_err(e)?;
        serde_json::from_str(&s).map_err(|x| format!("manifest unreadable: {x}"))?
    };
    if man.format != "cartacolibri-backup" {
        return Err("not a CartaColibri backup".into());
    }
    if man.version > BACKUP_VERSION {
        return Err(format!(
            "backup version {} is newer than this program understands ({BACKUP_VERSION})",
            man.version
        ));
    }
    let mut data = vec![];
    for bf in &man.files {
        let safe = !bf.name.starts_with('/')
            && !bf.name.contains([':', '\\', '\0'])
            && !bf.name.chars().any(char::is_control)
            && !bf
                .name
                .split('/')
                .any(|c| matches!(c, "." | "..") || c.is_empty());
        if !safe {
            return Err(format!("unsafe path in backup: {}", bf.name));
        }
        let mut buf = vec![];
        z.by_name(&format!("state/{}", bf.name))
            .map_err(|_| format!("backup damaged: {} is missing", bf.name))?
            .take(bf.size.saturating_add(1)) // never read more than the manifest promises
            .read_to_end(&mut buf)
            .map_err(e)?;
        if buf.len() as u64 != bf.size || sha1_hex(&buf) != bf.sha1 {
            return Err(format!("backup damaged: checksum mismatch for {}", bf.name));
        }
        data.push((bf.name.clone(), buf));
    }
    Ok((man, data))
}

/// Check a backup without restoring it.
pub fn backup_verify(zip_path: &Path) -> Res<Manifest> {
    read_backup(zip_path).map(|x| x.0)
}

#[derive(Serialize, Debug)]
pub struct RestoreReport {
    pub files: usize,
    pub includes_index: bool,
    /// True when the index was not in the backup: run an update to rebuild it.
    pub needs_update: bool,
    pub roots_restored: usize,
}

/// Restore a backup into `home`. Every checksum is verified before anything is written. No
/// `Library` may be open on `home` meanwhile; open it again afterwards.
pub fn restore(zip_path: &Path, home: &Path) -> Res<RestoreReport> {
    let (man, data) = read_backup(zip_path)?;
    std::fs::create_dir_all(home).map_err(e)?;
    if man.includes_index {
        for n in [
            "index.sqlite",
            "index.sqlite-wal",
            "index.sqlite-shm",
            "index.sqlite-journal",
        ] {
            let _ = std::fs::remove_file(home.join(n));
        }
    }
    let mut roots: Vec<String> = vec![];
    for (name, buf) in &data {
        if name == "roots.json" {
            roots = serde_json::from_slice(buf).unwrap_or_default();
        }
        let p = home.join(name);
        // refuse to write through a symlink inside the home: it could lead anywhere
        let mut cur = home.to_path_buf();
        for comp in name.split('/') {
            cur.push(comp);
            if std::fs::symlink_metadata(&cur).is_ok_and(|m| m.file_type().is_symlink()) {
                return Err(format!(
                    "refusing to restore {name}: {} is a symbolic link",
                    cur.display()
                ));
            }
        }
        if let Some(d) = p.parent() {
            std::fs::create_dir_all(d).map_err(e)?;
        }
        // write next to the target, then rename: a failure never leaves a half-written file
        let tmp = p.with_extension(format!("restore-{}.tmp", std::process::id()));
        std::fs::write(&tmp, buf)
            .and_then(|_| std::fs::rename(&tmp, &p))
            .map_err(|x| {
                let _ = std::fs::remove_file(&tmp);
                format!("cannot restore {name}: {x}")
            })?;
    }
    let mut restored = 0;
    if !man.includes_index {
        let _ = std::fs::remove_file(home.join("index.sqlite"));
        let lib = Library::open(Some(&home.to_string_lossy()), false)?;
        for r in &roots {
            if lib.add_root(r).is_ok() {
                restored += 1;
            }
        }
    }
    Ok(RestoreReport {
        files: data.len(),
        includes_index: man.includes_index,
        needs_update: !man.includes_index,
        roots_restored: restored,
    })
}

// ---------------------------------------------------------------- verify

#[derive(Serialize, Debug, Default)]
pub struct VerifyReport {
    pub ok: bool,
    pub problems: Vec<String>,
    pub rebuilt: bool,
    pub docs: usize,
}

fn index_problems(lib: &Library) -> Vec<String> {
    let c = lib.conn();
    let mut p = vec![];
    match c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0)) {
        Ok(s) if s == "ok" => {}
        Ok(s) => p.push(format!("database integrity check failed: {s}")),
        Err(x) => p.push(format!("database unreadable: {x}")),
    }
    if let Err(x) = c.execute("INSERT INTO chunks(chunks) VALUES('integrity-check')", []) {
        p.push(format!("full-text index is corrupt: {x}"));
    }
    match c.query_row(
        "SELECT count(*) FROM chunks WHERE doc_id NOT IN (SELECT id FROM docs)",
        [],
        |r| r.get::<_, i64>(0),
    ) {
        Ok(0) => {}
        Ok(n) => p.push(format!("{n} orphan text chunks without a document")),
        Err(x) => p.push(format!("chunks unreadable: {x}")),
    }
    p
}

/// Save the folder list next to the index (`roots.json`) so a broken index can be rebuilt.
pub fn snapshot_roots(lib: &Library) {
    if let Ok(s) = serde_json::to_string_pretty(&lib.roots()) {
        let _ = std::fs::write(lib.home.join("roots.json"), s);
    }
}

fn rebuild(lib: &mut Library) -> Res<()> {
    snapshot_roots(lib);
    lib.reset_index()?;
    lib.update(None, &mut |_| {})?;
    if let Ok(d) = lib.upload_dir() {
        let _ = lib.update(Some(&[d.to_string_lossy().into_owned()]), &mut |_| {});
    }
    Ok(())
}

/// Detect a corrupt index; with `repair`, rebuild it from the registered folders.
pub fn verify(lib: &mut Library, repair: bool) -> Res<VerifyReport> {
    let problems = index_problems(lib);
    let mut rep = VerifyReport {
        ok: problems.is_empty(),
        problems,
        ..Default::default()
    };
    if !rep.ok && repair {
        rebuild(lib)?;
        rep.rebuilt = true;
        rep.ok = index_problems(lib).is_empty();
    } else if rep.ok {
        snapshot_roots(lib);
    }
    rep.docs = lib.documents(None).map(|d| d.len()).unwrap_or(0);
    Ok(rep)
}

/// Like [`verify`] but starts from a home folder, so it also copes with an index file so broken
/// that it cannot be opened at all. Returns the (possibly rebuilt) library.
pub fn verify_home(home: &str, repair: bool) -> Res<(Library, VerifyReport)> {
    match Library::open(Some(home), false) {
        Ok(mut lib) => {
            let r = verify(&mut lib, repair)?;
            Ok((lib, r))
        }
        Err(msg) if repair => {
            let h = PathBuf::from(home);
            // keep the unreadable index as *.broken instead of deleting it (it may only be locked)
            for n in [
                "index.sqlite",
                "index.sqlite-wal",
                "index.sqlite-shm",
                "index.sqlite-journal",
            ] {
                if h.join(n).exists() {
                    let _ = std::fs::rename(h.join(n), h.join(format!("{n}.broken")));
                }
            }
            let roots: Vec<String> = std::fs::read_to_string(h.join("roots.json"))
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let mut lib = Library::open(Some(home), false)?;
            for r in &roots {
                let _ = lib.add_root(r);
            }
            rebuild(&mut lib)?;
            let docs = lib.documents(None).map(|d| d.len()).unwrap_or(0);
            Ok((
                lib,
                VerifyReport {
                    ok: true,
                    problems: vec![format!("index could not be opened: {msg}")],
                    rebuilt: true,
                    docs,
                },
            ))
        }
        Err(msg) => Err(msg),
    }
}

// ---------------------------------------------------------------- stats / timeline

#[derive(Serialize, Debug, Default)]
pub struct LibraryStats {
    pub docs: usize,
    pub total_bytes: i64,
    pub words_total: u64,
    pub by_type: BTreeMap<String, usize>,
    /// Indexed folder (root) -> number of documents.
    pub by_folder: BTreeMap<String, usize>,
    pub by_year: BTreeMap<String, usize>,
    /// (path, bytes), largest first, at most 10.
    pub biggest: Vec<(String, i64)>,
    /// Guessed language code ("en", "ro", "fr", "de", "es", "it", "?") -> documents.
    pub languages: BTreeMap<String, usize>,
}

const STOP: &[(&str, &str)] = &[
    (
        "en",
        "the and of to in is that it was for with as on are this by be from or an at have not but they which",
    ),
    (
        "ro",
        "si de la in cu pe nu un o este care sa din pentru mai ca au se prin ce sau dar fost sunt lui al ale",
    ),
    (
        "fr",
        "le la les de des et un une est que qui dans pour pas sur au avec ce il elle sont par plus ne se",
    ),
    (
        "de",
        "der die das und ist nicht ein eine zu den von mit auf für sich dem auch es im als aber oder wird",
    ),
    (
        "es",
        "el la los las de que y en un una es por con para no se del al lo como más pero sus su fue",
    ),
    (
        "it",
        "il lo la gli le di che e un una è per con non si da del della dei come più ma sono anche al",
    ),
];

/// Guess a language from stopword frequency; "?" when too little evidence.
pub fn guess_language(text: &str) -> &'static str {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for w in text.split(|c: char| !c.is_alphanumeric()).take(4000) {
        if !w.is_empty() {
            *counts.entry(fold(w)).or_insert(0) += 1;
        }
    }
    let mut best = ("?", 0u32);
    for (lang, words) in STOP {
        let s: u32 = words
            .split(' ')
            .map(|w| counts.get(&fold(w)).copied().unwrap_or(0))
            .sum();
        if s > best.1 {
            best = (lang, s);
        }
    }
    if best.1 >= 3 { best.0 } else { "?" }
}

pub fn stats(lib: &Library) -> Res<LibraryStats> {
    let c = lib.conn();
    let mut st = LibraryStats::default();
    let mut q = c
        .prepare("SELECT id,root,path,ext,size,mtime_ns FROM docs")
        .map_err(e)?;
    let docs: Vec<(i64, String, String, String, i64, i64)> = q
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })
        .map_err(e)?
        .collect::<Result<_, _>>()
        .map_err(e)?;
    let mut big: Vec<(String, i64)> = vec![];
    for (_, root, path, ext, size, mt) in &docs {
        st.docs += 1;
        st.total_bytes += size;
        *st.by_type.entry(ext.clone()).or_insert(0) += 1;
        *st.by_folder.entry(root.clone()).or_insert(0) += 1;
        *st.by_year.entry(ym(*mt).0.to_string()).or_insert(0) += 1;
        big.push((path.clone(), *size));
    }
    big.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    big.truncate(10);
    st.biggest = big;
    let mut q = c
        .prepare("SELECT doc_id, text FROM chunks ORDER BY doc_id, CAST(seq AS INTEGER)")
        .map_err(e)?;
    let mut rows = q.query([]).map_err(e)?;
    let mut sample: HashMap<i64, String> = HashMap::new();
    while let Some(r) = rows.next().map_err(e)? {
        let id: i64 = r.get(0).map_err(e)?;
        let t: String = r.get(1).map_err(e)?;
        st.words_total += t.split_whitespace().count() as u64;
        let s = sample.entry(id).or_default();
        if s.len() < 20_000 {
            s.push_str(&t);
            s.push(' ');
        }
    }
    for (id, _, _, _, _, _) in &docs {
        let l = sample.get(id).map_or("?", |t| guess_language(t));
        *st.languages.entry(l.to_string()).or_insert(0) += 1;
    }
    Ok(st)
}

#[derive(Serialize, Debug, Clone)]
pub struct TimelineDoc {
    pub id: i64,
    pub path: String,
    pub ext: String,
    pub size: i64,
    pub modified: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct TimelineMonth {
    /// "YYYY-MM"
    pub month: String,
    pub count: usize,
    pub docs: Vec<TimelineDoc>,
}

/// Documents grouped by modified month, newest month first.
pub fn timeline(lib: &Library) -> Res<Vec<TimelineMonth>> {
    let mut q = lib
        .conn()
        .prepare("SELECT id,path,ext,size,mtime_ns FROM docs ORDER BY mtime_ns DESC, path")
        .map_err(e)?;
    let rows: Vec<(i64, String, String, i64, i64)> = q
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .map_err(e)?
        .collect::<Result<_, _>>()
        .map_err(e)?;
    let mut out: Vec<TimelineMonth> = vec![];
    for (id, path, ext, size, mt) in rows {
        let (y, m) = ym(mt);
        let key = format!("{y:04}-{m:02}");
        if out.last().is_none_or(|l| l.month != key) {
            out.push(TimelineMonth {
                month: key,
                count: 0,
                docs: vec![],
            });
        }
        let l = out.last_mut().unwrap();
        l.count += 1;
        l.docs.push(TimelineDoc {
            id,
            path,
            ext,
            size,
            modified: iso((mt / 1_000_000_000) as f64),
        });
    }
    Ok(out)
}
