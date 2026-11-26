//! Semantic search: the local engine's embedding model turns every passage and every question
//! into a vector (`POST /v1/embeddings`, OpenAI-compatible: Colibrì, Ollama, LM Studio, ...);
//! vectors live in the library's own SQLite file and are compared with cosine similarity.
//! Hybrid mode fuses the BM25 ranking and the vector ranking with reciprocal rank fusion, so a
//! word-for-word match still wins and a question phrased differently from the text still finds it.
//! Nothing leaves the machine; without an embedding model everything else keeps working.

use crate::carta::{Backend, redact_url};
use crate::library::{Filter, Hit, Library, parse_inline};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BinaryHeap, HashMap};
use std::time::Duration;

type Res<T> = Result<T, String>;

/// Longest text sent to the embedding model for one passage.
const MAX_INPUT_CHARS: usize = 2000;
/// Reciprocal rank fusion constant (the usual 60).
const RRF_K: f64 = 60.0;
/// Passages compared per question before the filters thin the list.
const TOP_CHUNKS: usize = 5000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Bm25,
    Semantic,
    Hybrid,
}

impl Mode {
    pub fn parse(s: &str) -> Res<Mode> {
        match s.trim().to_lowercase().as_str() {
            "" | "bm25" | "words" | "fulltext" => Ok(Mode::Bm25),
            "semantic" | "vector" | "meaning" => Ok(Mode::Semantic),
            "hybrid" | "both" => Ok(Mode::Hybrid),
            o => Err(format!(
                "unknown search mode `{o}` (bm25, semantic or hybrid)"
            )),
        }
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Searched {
    pub hits: Vec<Hit>,
    /// What actually ran: "bm25", "semantic" or "hybrid".
    pub mode: &'static str,
    /// Why the search fell back to words only, or what to do to get meaning search.
    pub note: Option<String>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct EmbedStats {
    pub model: String,
    pub embedded: usize,
    pub already: usize,
    pub pending: usize,
    pub total: usize,
}

fn normalize(mut v: Vec<f32>) -> Vec<f32> {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 && n.is_finite() {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
    v
}

/// Offline stand-in for an embedding model (`--mock`): hashed bag of words. It is lexical, not
/// semantic; it only lets the whole pipeline run (and be tested) without an engine.
fn mock_embed(t: &str) -> Vec<f32> {
    let mut v = vec![0f32; 256];
    for w in t.split(|c: char| !c.is_alphanumeric()) {
        if w.is_empty() {
            continue;
        }
        let mut h: u64 = 0xcbf29ce484222325;
        for b in w.to_lowercase().bytes() {
            h = (h ^ b as u64).wrapping_mul(0x100000001b3);
        }
        v[(h % 256) as usize] += if (h >> 40) & 1 == 0 { 1.0 } else { -1.0 };
    }
    normalize(v)
}

impl Backend {
    /// Embeddings for `inputs` from `POST {url}/v1/embeddings`, normalised to unit length.
    pub fn embed(&self, model: Option<&str>, inputs: &[String]) -> Res<Vec<Vec<f32>>> {
        if inputs.is_empty() {
            return Ok(vec![]);
        }
        if self.mock {
            return Ok(inputs.iter().map(|t| mock_embed(t)).collect());
        }
        let fail = |e: String| {
            format!(
                "no embeddings from the engine at {} ({e}). Load an embedding model (for Ollama: `ollama pull nomic-embed-text`, then --embed-model nomic-embed-text), or use --mock to try the pipeline offline.",
                redact_url(&self.url)
            )
        };
        let mut body = json!({"input": inputs});
        let m = model
            .map(str::to_string)
            .or_else(|| self.model.clone())
            .or_else(|| self.models().into_iter().next());
        if let Some(m) = &m {
            body["model"] = json!(m);
        }
        // one question must not hang a search for minutes; a batch of passages may take a while
        let agent = self.agent(
            Duration::from_secs(if inputs.len() == 1 { 30 } else { 300 }),
            true,
        );
        let mut req = agent.post(&format!("{}/v1/embeddings", self.url));
        if let Some(k) = &self.api_key {
            req = req.header("Authorization", &format!("Bearer {k}"));
        }
        let mut resp = req.send_json(&body).map_err(|e| {
            let t = e.to_string();
            fail(if t.contains("401") || t.contains("403") {
                "it needs an API key: set COLI_API_KEY".to_string()
            } else {
                t
            })
        })?;
        let v: Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| fail(e.to_string()))?;
        let data = v["data"]
            .as_array()
            .ok_or_else(|| fail("unexpected response: no `data`".into()))?;
        if data.len() != inputs.len() {
            return Err(fail(format!(
                "asked for {} vectors, got {}",
                inputs.len(),
                data.len()
            )));
        }
        let mut out: Vec<Vec<f32>> = vec![vec![]; inputs.len()];
        for (i, d) in data.iter().enumerate() {
            let idx = d["index"].as_u64().map(|x| x as usize).unwrap_or(i);
            // every entry must be a number: a skipped one would silently shorten the vector
            let emb: Vec<f32> = d["embedding"]
                .as_array()
                .and_then(|a| a.iter().map(|x| x.as_f64().map(|x| x as f32)).collect())
                .unwrap_or_default();
            if idx >= out.len() || emb.is_empty() || emb.iter().any(|x| !x.is_finite()) {
                return Err(fail("an embedding is missing or not a number".into()));
            }
            out[idx] = normalize(emb);
        }
        let dim = out[0].len();
        if out.iter().any(|e| e.len() != dim) {
            return Err(fail("vectors of different sizes".into()));
        }
        Ok(out)
    }
}

fn to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn dot_blob(q: &[f32], b: &[u8]) -> Option<f32> {
    if b.len() != q.len() * 4 {
        return None;
    }
    let (words, _) = b.as_chunks::<4>();
    Some(
        q.iter()
            .zip(words)
            .map(|(x, c)| x * f32::from_le_bytes(*c))
            .sum(),
    )
}

impl Library {
    fn vectors_table(&self) -> Res<()> {
        self.db
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS vectors(rid INTEGER PRIMARY KEY, doc_id INTEGER, model TEXT, v BLOB);",
            )
            .map_err(|e| e.to_string())
    }

    /// `(passages with a vector, passages in total)` for `model`.
    pub fn vector_counts(&self, model: &str) -> (usize, usize) {
        let _ = self.vectors_table();
        let have: i64 = self
            .db
            .query_row(
                "SELECT count(*) FROM vectors WHERE model=?1",
                [model],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let all: i64 = self
            .db
            .query_row("SELECT count(*) FROM chunks", [], |r| r.get(0))
            .unwrap_or(0);
        (have.max(0) as usize, all.max(0) as usize)
    }

    /// Embeds the passages that have no vector yet (at most `limit`, `batch` per request).
    /// Incremental: vectors of passages that were re-indexed or removed are dropped, vectors made
    /// by another model are replaced.
    pub fn embed_missing(
        &self,
        be: &Backend,
        model: &str,
        batch: usize,
        limit: usize,
        progress: &mut dyn FnMut(usize, usize),
    ) -> Res<EmbedStats> {
        self.vectors_table()?;
        let batch = batch.clamp(1, 256);
        let q = |s: &str, p: &[&dyn rusqlite::ToSql]| {
            self.db.execute(s, p).map(|_| ()).map_err(|e| e.to_string())
        };
        q("DELETE FROM vectors WHERE model<>?1", &[&model])?;
        q(
            "DELETE FROM vectors WHERE rid NOT IN (SELECT rowid FROM chunks)",
            &[],
        )?;
        let (already, total) = self.vector_counts(model);
        let mut done = 0usize;
        let mut last: i64 = 0;
        while done < limit {
            let rows: Vec<(i64, i64, String)> = {
                let mut st = self
                    .db
                    .prepare_cached(
                        "SELECT c.rowid, c.doc_id, c.text FROM chunks c LEFT JOIN vectors v ON v.rid=c.rowid \
                         WHERE c.rowid>?1 AND v.rid IS NULL ORDER BY c.rowid LIMIT ?2",
                    )
                    .map_err(|e| e.to_string())?;
                st.query_map(rusqlite::params![last, batch as i64], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?))
                })
                .map_err(|e| e.to_string())?
                .flatten()
                .collect()
            };
            if rows.is_empty() {
                break;
            }
            last = rows.last().map(|r| r.0).unwrap_or(last);
            let inputs: Vec<String> = rows
                .iter()
                .map(|r| r.2.chars().take(MAX_INPUT_CHARS).collect())
                .collect();
            let vs = be.embed(Some(model), &inputs)?;
            let tx = self.db.unchecked_transaction().map_err(|e| e.to_string())?;
            for ((rid, doc, _), v) in rows.iter().zip(&vs) {
                tx.execute(
                    "INSERT OR REPLACE INTO vectors(rid,doc_id,model,v) VALUES(?1,?2,?3,?4)",
                    rusqlite::params![rid, doc, model, to_blob(v)],
                )
                .map_err(|e| e.to_string())?;
            }
            tx.commit().map_err(|e| e.to_string())?;
            done += rows.len();
            progress(already + done, total);
        }
        let (have, total) = self.vector_counts(model);
        Ok(EmbedStats {
            model: model.to_string(),
            embedded: done,
            already,
            pending: total.saturating_sub(have),
            total,
        })
    }

    /// The passages closest in meaning to `qv`, as hits (filters applied, one hit per document).
    pub fn semantic_hits(&self, qv: &[f32], model: &str, f: &Filter) -> Res<Vec<Hit>> {
        self.vectors_table()?;
        let mut st = self
            .db
            .prepare("SELECT rid, doc_id, v FROM vectors WHERE model=?1")
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([model], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, Vec<u8>>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        // min-heap of the best TOP_CHUNKS passages: (similarity in millionths, rid)
        let mut heap: BinaryHeap<std::cmp::Reverse<(i64, i64)>> = BinaryHeap::new();
        for (rid, _doc, blob) in rows.flatten() {
            let Some(sim) = dot_blob(qv, &blob) else {
                continue;
            };
            // a passage with nothing in common with the question is not a match
            if !sim.is_finite() || sim <= 0.0 {
                continue;
            }
            let key = (sim as f64 * 1e6) as i64;
            if heap.len() < TOP_CHUNKS {
                heap.push(std::cmp::Reverse((key, rid)));
            } else if heap.peek().is_some_and(|m| key > m.0.0) {
                heap.pop();
                heap.push(std::cmp::Reverse((key, rid)));
            }
        }
        let mut best: Vec<(i64, i64)> = heap.into_iter().map(|r| r.0).collect();
        best.sort_by(|a, b| b.cmp(a));
        let mut text = self
            .db
            .prepare_cached("SELECT doc_id, loc, text FROM chunks WHERE rowid=?1")
            .map_err(|e| e.to_string())?;
        let mut out: Vec<(i64, String, String, f64)> = vec![];
        for (key, rid) in best {
            if let Ok(r) = text.query_row([rid], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            }) {
                // the BM25 convention is "lower is better"; the score is shown as its negative
                out.push((r.0, r.1, r.2, -(key as f64) / 1e6));
            }
        }
        self.rows_to_hits(out, &[], f)
    }

    /// Words, meaning, or both fused (reciprocal rank fusion). Falls back to words, with a note,
    /// when no vectors exist yet or the engine has no embedding model.
    pub fn search_mode(
        &self,
        be: &Backend,
        model: &str,
        query: &str,
        f: &Filter,
        mode: Mode,
    ) -> Res<Searched> {
        let (inline, fi) = parse_inline(query, f);
        let text = inline.trim();
        if mode == Mode::Bm25 || text.is_empty() {
            return Ok(Searched {
                hits: self.search(query, f)?,
                mode: "bm25",
                note: None,
            });
        }
        let limit = fi.limit.unwrap_or(50);
        let wide = Filter {
            limit: Some(200),
            ..fi.clone()
        };
        let (have, _) = self.vector_counts(model);
        let sem: Res<Vec<Hit>> = if have == 0 {
            Err("no vectors yet: run `cartacolibri embed` first".into())
        } else {
            be.embed(Some(model), &[text.chars().take(MAX_INPUT_CHARS).collect()])
                .and_then(|q| self.semantic_hits(&q[0], model, &wide))
        };
        let sem = match sem {
            Ok(h) => h,
            Err(e) if mode == Mode::Hybrid => {
                return Ok(Searched {
                    hits: self.search(query, f)?,
                    mode: "bm25",
                    note: Some(format!(
                        "meaning search not available, showing word matches ({e})"
                    )),
                });
            }
            Err(e) => return Err(e),
        };
        if mode == Mode::Semantic {
            let mut hits = sem;
            hits.truncate(limit);
            return Ok(Searched {
                hits,
                mode: "semantic",
                note: None,
            });
        }
        let bm = self.search(query, &wide)?;
        let mut fused: HashMap<i64, (f64, Hit)> = HashMap::new();
        for list in [&bm, &sem] {
            for (rank, h) in list.iter().enumerate() {
                let add = 1.0 / (RRF_K + rank as f64 + 1.0);
                fused
                    .entry(h.id)
                    .and_modify(|e| e.0 += add)
                    .or_insert((add, h.clone()));
            }
        }
        let mut v: Vec<(f64, Hit)> = fused.into_values().collect();
        v.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.id.cmp(&b.1.id))
        });
        let top = v.first().map(|x| x.0).unwrap_or(1.0);
        let hits: Vec<Hit> = v
            .into_iter()
            .take(limit)
            .map(|(s, mut h)| {
                h.score = (100.0 * s / top).round();
                h
            })
            .collect();
        Ok(Searched {
            hits,
            mode: "hybrid",
            note: None,
        })
    }
}

/// The embedding model to use: `--embed-model`, else COLI_EMBED_MODEL, else the engine's own.
pub fn embed_model_name(be: &Backend, chosen: Option<&str>) -> String {
    if be.mock {
        return "mock".into();
    }
    chosen
        .map(str::to_string)
        .or_else(|| {
            std::env::var("COLI_EMBED_MODEL")
                .ok()
                .filter(|m| !m.trim().is_empty())
        })
        .or_else(|| be.model.clone())
        .or_else(|| be.models().into_iter().next())
        .unwrap_or_else(|| "default".into())
}

// ------------------------------------------------------------------ retrieval evaluation

/// One question with the document(s) that should come back for it.
#[derive(Serialize, Clone, Debug)]
pub struct EvalCase {
    pub q: String,
    pub expect: Vec<String>,
}

/// Cases from JSON (`[{"q": "...", "expect": ["lease.txt"]}]`) or from lines of
/// `question<TAB>file1,file2` (`#` starts a comment).
pub fn load_cases(text: &str) -> Res<Vec<EvalCase>> {
    let t = text.trim_start_matches('\u{feff}').trim();
    let mut out = vec![];
    if t.starts_with('[') {
        let v: Value = serde_json::from_str(t).map_err(|e| format!("not valid JSON: {e}"))?;
        for (i, c) in v.as_array().into_iter().flatten().enumerate() {
            let q = c["q"]
                .as_str()
                .or(c["question"].as_str())
                .unwrap_or("")
                .trim();
            let expect: Vec<String> = match &c["expect"] {
                Value::String(s) => vec![s.clone()],
                Value::Array(a) => a
                    .iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect(),
                _ => vec![],
            };
            if q.is_empty() || expect.is_empty() {
                return Err(format!("case {}: needs \"q\" and \"expect\"", i + 1));
            }
            out.push(EvalCase {
                q: q.into(),
                expect,
            });
        }
    } else {
        for (i, l) in t.lines().enumerate() {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            let Some((q, e)) = l.split_once('\t') else {
                return Err(format!(
                    "line {}: expected `question<TAB>file1,file2`",
                    i + 1
                ));
            };
            let expect: Vec<String> = e
                .split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect();
            if q.trim().is_empty() || expect.is_empty() {
                return Err(format!(
                    "line {}: question or expected file is empty",
                    i + 1
                ));
            }
            out.push(EvalCase {
                q: q.trim().into(),
                expect,
            });
        }
    }
    if out.is_empty() {
        return Err("no questions found".into());
    }
    Ok(out)
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct EvalRow {
    pub mode: &'static str,
    pub questions: usize,
    /// share of questions whose expected document is the first result / in the first 5 / 10
    pub hit_at_1: f64,
    pub hit_at_5: f64,
    pub hit_at_10: f64,
    /// mean reciprocal rank of the first expected document (0 when it is not in the first 10)
    pub mrr: f64,
    pub misses: Vec<String>,
}

/// How well each search mode brings back the expected documents. Semantic modes are skipped (with
/// a note in `notes`) when the library has no vectors for `model`.
pub fn evaluate(
    lib: &Library,
    be: &Backend,
    model: &str,
    cases: &[EvalCase],
    modes: &[Mode],
) -> Res<(Vec<EvalRow>, Vec<String>)> {
    let mut rows = vec![];
    let mut notes = vec![];
    let f = Filter {
        limit: Some(10),
        ..Default::default()
    };
    for &mode in modes {
        if mode != Mode::Bm25 && lib.vector_counts(model).0 == 0 {
            notes.push(format!(
                "{} skipped: no vectors yet (run `cartacolibri embed`)",
                if mode == Mode::Semantic {
                    "semantic"
                } else {
                    "hybrid"
                }
            ));
            continue;
        }
        let mut row = EvalRow {
            mode: match mode {
                Mode::Bm25 => "bm25",
                Mode::Semantic => "semantic",
                Mode::Hybrid => "hybrid",
            },
            questions: cases.len(),
            ..Default::default()
        };
        for c in cases {
            let hits = lib.search_mode(be, model, &c.q, &f, mode)?.hits;
            let rank = hits.iter().position(|h| {
                // Windows paths use backslashes: compare with forward slashes on both sides
                let p = h.path.to_lowercase().replace('\\', "/");
                c.expect.iter().any(|e| {
                    let e = e.to_lowercase().replace('\\', "/");
                    p == e || p.ends_with(&format!("/{e}"))
                })
            });
            match rank {
                Some(r) => {
                    row.mrr += 1.0 / (r as f64 + 1.0);
                    row.hit_at_1 += (r < 1) as u8 as f64;
                    row.hit_at_5 += (r < 5) as u8 as f64;
                    row.hit_at_10 += 1.0;
                }
                None => row.misses.push(c.q.clone()),
            }
        }
        let n = cases.len().max(1) as f64;
        for x in [
            &mut row.hit_at_1,
            &mut row.hit_at_5,
            &mut row.hit_at_10,
            &mut row.mrr,
        ] {
            *x = (*x / n * 1000.0).round() / 1000.0;
        }
        rows.push(row);
    }
    Ok((rows, notes))
}
