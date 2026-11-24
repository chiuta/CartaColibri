//! Engine-free insight tools that work on your own text: how well an answer is grounded in its
//! sources, the entities (emails, links, money, dates, phones) a library mentions, and a
//! sentence-level comparison of two documents. Nothing here calls a model or leaves the computer.
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

// ------------------------------------------------------------------ grounding of an answer

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Claim {
    pub sentence: String,
    pub cited: Vec<String>,
    /// share (0..1) of the sentence's content words found in the cited sources (all sources when uncited)
    pub support: f64,
    /// "supported" | "partial" | "weak"
    pub status: &'static str,
    /// numbers in the sentence that appear in none of the sources
    pub unmatched_numbers: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Grounding {
    /// 0..100, average support of the checked sentences, minus a penalty for unmatched numbers
    pub score: u32,
    pub claims: Vec<Claim>,
}

fn stem(w: &str) -> String {
    w.chars().take(6).collect()
}

fn words(t: &str) -> BTreeSet<String> {
    t.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 4 && !w.chars().all(|c| c.is_ascii_digit()))
        .map(stem)
        .collect()
}

fn numbers(t: &str) -> Vec<String> {
    Regex::new(r"\d[\d.,]*\d|\d")
        .unwrap()
        .find_iter(t)
        .map(|m| m.as_str().trim_end_matches(['.', ',']).to_string())
        .collect()
}

fn sentences(answer: &str) -> Vec<String> {
    let mut out = vec![];
    for line in answer.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("**Sources**") || t.starts_with("<!--") || t == "---" {
            continue;
        }
        let mut cur = String::new();
        let cs: Vec<char> = t.chars().collect();
        for (i, c) in cs.iter().enumerate() {
            cur.push(*c);
            let end =
                matches!(c, '.' | '!' | '?') && cs.get(i + 1).is_none_or(|n| n.is_whitespace());
            if end {
                out.push(std::mem::take(&mut cur).trim().to_string());
            }
        }
        if !cur.trim().is_empty() {
            out.push(cur.trim().to_string());
        }
    }
    out.retain(|s| s.chars().filter(|c| c.is_alphanumeric()).count() >= 12);
    out
}

/// `sources` are `(label, text)` pairs, labels like `S1`.
pub fn grounding(answer: &str, sources: &[(String, String)]) -> Grounding {
    let cite = Regex::new(r"\[(S\d+(?:\s*[,;]\s*S\d+)*)\]").unwrap();
    let lab = Regex::new(r"S\d+").unwrap();
    let all: BTreeSet<String> = sources.iter().flat_map(|(_, t)| words(t)).collect();
    let all_nums: BTreeSet<String> = sources.iter().flat_map(|(_, t)| numbers(t)).collect();
    let mut claims = vec![];
    for s in sentences(answer) {
        let cited: Vec<String> = cite
            .captures_iter(&s)
            .flat_map(|c| {
                lab.find_iter(&c[1])
                    .map(|m| m.as_str().to_string())
                    .collect::<Vec<_>>()
            })
            .collect();
        let bare = cite.replace_all(&s, "").to_string();
        let mine = words(&bare);
        if mine.is_empty() {
            continue;
        }
        let pool: BTreeSet<String> = if cited.is_empty() {
            all.clone()
        } else {
            sources
                .iter()
                .filter(|(l, _)| cited.contains(l))
                .flat_map(|(_, t)| words(t))
                .collect()
        };
        let hit = mine.iter().filter(|w| pool.contains(*w)).count();
        let support = hit as f64 / mine.len() as f64;
        let unmatched_numbers: Vec<String> = numbers(&bare)
            .into_iter()
            .filter(|n| !all_nums.contains(n))
            .collect();
        let status = if support >= 0.6 {
            "supported"
        } else if support >= 0.3 {
            "partial"
        } else {
            "weak"
        };
        claims.push(Claim {
            sentence: s,
            cited,
            support: (support * 100.0).round() / 100.0,
            status,
            unmatched_numbers,
        });
    }
    let score = if claims.is_empty() {
        0
    } else {
        let avg = claims.iter().map(|c| c.support).sum::<f64>() / claims.len() as f64;
        let bad = claims
            .iter()
            .filter(|c| !c.unmatched_numbers.is_empty())
            .count() as f64
            / claims.len() as f64;
        ((avg * 100.0) - bad * 25.0).clamp(0.0, 100.0).round() as u32
    };
    Grounding { score, claims }
}

// ------------------------------------------------------------------ entities

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity {
    pub kind: &'static str,
    pub value: String,
}

pub const KINDS: [&str; 5] = ["email", "url", "money", "date", "phone"];

pub fn entities(text: &str) -> Vec<Entity> {
    let pats: [(&str, &str); 5] = [
        (
            "email",
            r"[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}",
        ),
        ("url", r#"https?://[^\s<>"')\]]+"#),
        (
            "money",
            r"(?i)(?:[€$£]\s?\d[\d.,]*\d|\b\d[\d.,]*\d?\s?(?:€|\$|£|eur|euro|usd|ron|lei|gbp)\b)",
        ),
        (
            "date",
            r"\b(?:\d{4}-\d{2}-\d{2}|\d{1,2}[./]\d{1,2}[./]\d{4})\b",
        ),
        (
            "phone",
            r"\+\d{1,3}[\s.-]?\(?\d{1,4}\)?(?:[\s.-]?\d{2,4}){2,4}",
        ),
    ];
    let mut out = BTreeSet::new();
    for (kind, p) in pats {
        for m in Regex::new(p).unwrap().find_iter(text) {
            let v = m
                .as_str()
                .trim()
                .trim_end_matches(['.', ',', ';', ':'])
                .to_string();
            if v.len() >= 4 {
                out.insert(Entity { kind, value: v });
            }
        }
    }
    out.into_iter().collect()
}

/// Entities across many documents: `(kind, value) -> sorted document paths`.
pub fn entity_index(
    docs: &[(String, String)],
    kind: Option<&str>,
) -> BTreeMap<(String, String), BTreeSet<String>> {
    let mut m: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for (path, text) in docs {
        for e in entities(text) {
            if kind.is_none_or(|k| k == e.kind) {
                m.entry((e.kind.to_string(), e.value))
                    .or_default()
                    .insert(path.clone());
            }
        }
    }
    m
}

// ------------------------------------------------------------------ compare two documents

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Compare {
    /// 0..100 share of sentences the two documents have in common
    pub similarity: u32,
    pub only_in_a: Vec<String>,
    pub only_in_b: Vec<String>,
    pub shared: usize,
}

fn norm_sentences(t: &str) -> Vec<(String, String)> {
    let flat = t.replace('\n', " ");
    Regex::new(r"[^.!?]+[.!?]?")
        .unwrap()
        .find_iter(&flat)
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| s.chars().filter(|c| c.is_alphanumeric()).count() >= 12)
        .map(|s| {
            let key = s
                .to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| !w.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            (key, s)
        })
        .collect()
}

pub fn compare(a: &str, b: &str) -> Compare {
    let (sa, sb) = (norm_sentences(a), norm_sentences(b));
    let ka: BTreeSet<&String> = sa.iter().map(|x| &x.0).collect();
    let kb: BTreeSet<&String> = sb.iter().map(|x| &x.0).collect();
    let shared = ka.intersection(&kb).count();
    let union = ka.union(&kb).count();
    let pick = |v: &[(String, String)], other: &BTreeSet<&String>| {
        let mut seen = BTreeSet::new();
        v.iter()
            .filter(|(k, _)| !other.contains(k) && seen.insert(k.clone()))
            .map(|(_, s)| s.clone())
            .collect::<Vec<_>>()
    };
    Compare {
        similarity: if union == 0 {
            100
        } else {
            (100.0 * shared as f64 / union as f64).round() as u32
        },
        only_in_a: pick(&sa, &kb),
        only_in_b: pick(&sb, &ka),
        shared,
    }
}

// ------------------------------------------------------------------ possible conflicts

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Conflict {
    pub a_path: String,
    pub a_sentence: String,
    pub b_path: String,
    pub b_sentence: String,
    /// numbers only in A, numbers only in B
    pub a_only: Vec<String>,
    pub b_only: Vec<String>,
}

fn sentence_list(t: &str) -> Vec<String> {
    norm_sentences(t).into_iter().map(|x| x.1).collect()
}

/// Sentences of two different documents that say nearly the same thing but with different
/// numbers or dates (e.g. "rent is 500" vs "rent is 550"). A heuristic: it finds *candidates*.
pub fn conflicts(docs: &[(String, String)]) -> Vec<Conflict> {
    let mut sents: Vec<(usize, String, BTreeSet<String>, BTreeSet<String>)> = vec![];
    for (di, (_, text)) in docs.iter().enumerate().take(200) {
        for s in sentence_list(text).into_iter().take(600) {
            let w = words(&s);
            let n: BTreeSet<String> = numbers(&s).into_iter().collect();
            if w.len() >= 4 && !n.is_empty() {
                sents.push((di, s, w, n));
            }
        }
    }
    let mut index: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, (_, _, w, _)) in sents.iter().enumerate() {
        for x in w {
            index.entry(x.as_str()).or_default().push(i);
        }
    }
    let mut out: Vec<Conflict> = vec![];
    let mut done: BTreeSet<(usize, usize)> = BTreeSet::new();
    for (i, (da, sa, wa, na)) in sents.iter().enumerate() {
        let mut cand: BTreeMap<usize, usize> = BTreeMap::new();
        for x in wa {
            if let Some(v) = index.get(x.as_str()) {
                if v.len() > 300 {
                    continue; // a word that is everywhere says nothing
                }
                for &j in v {
                    if j > i && sents[j].0 != *da {
                        *cand.entry(j).or_default() += 1;
                    }
                }
            }
        }
        for (j, shared) in cand {
            let (db, sb, wb, nb) = &sents[j];
            let union = wa.union(wb).count();
            if shared < 4 || (shared as f64 / union as f64) < 0.5 || na == nb {
                continue;
            }
            if !done.insert((i, j)) {
                continue;
            }
            out.push(Conflict {
                a_path: docs[*da].0.clone(),
                a_sentence: sa.clone(),
                b_path: docs[*db].0.clone(),
                b_sentence: sb.clone(),
                a_only: na.difference(nb).cloned().collect(),
                b_only: nb.difference(na).cloned().collect(),
            });
            if out.len() >= 100 {
                return out;
            }
        }
    }
    out
}

// ------------------------------------------------------------------ topic map

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Topic {
    pub terms: Vec<String>,
    pub docs: Vec<String>,
}

fn find(p: &mut [usize], x: usize) -> usize {
    let mut r = x;
    while p[r] != r {
        r = p[r];
    }
    let mut c = x;
    while p[c] != r {
        let n = p[c];
        p[c] = r;
        c = n;
    }
    r
}

/// Groups documents that share their most characteristic words (TF-IDF top terms).
pub fn topics(docs: &[(String, String)]) -> Vec<Topic> {
    let n = docs.len();
    if n < 2 {
        return vec![];
    }
    // stem -> a readable original word, per-document term frequency
    let mut shown: BTreeMap<String, String> = BTreeMap::new();
    let mut tf: Vec<BTreeMap<String, f64>> = vec![];
    let mut df: BTreeMap<String, usize> = BTreeMap::new();
    for (_, text) in docs {
        let mut m: BTreeMap<String, f64> = BTreeMap::new();
        for w in text
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.chars().count() >= 5 && !w.chars().any(|c| c.is_ascii_digit()))
        {
            let st = stem(w);
            shown.entry(st.clone()).or_insert_with(|| w.to_string());
            *m.entry(st).or_default() += 1.0;
        }
        for k in m.keys() {
            *df.entry(k.clone()).or_default() += 1;
        }
        tf.push(m);
    }
    let top: Vec<Vec<String>> = tf
        .iter()
        .map(|m| {
            let mut v: Vec<(String, f64)> = m
                .iter()
                .filter(|(k, _)| df[*k] < n.max(2) && df[*k] * 2 <= n.max(2) + 1)
                .map(|(k, c)| (k.clone(), (1.0 + c.ln()) * (n as f64 / df[k] as f64).ln()))
                .collect();
            v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            v.into_iter().take(8).map(|x| x.0).collect()
        })
        .collect();
    let mut p: Vec<usize> = (0..n).collect();
    for i in 0..n {
        for j in i + 1..n {
            if top[i].iter().filter(|t| top[j].contains(t)).count() >= 2 {
                let (a, b) = (find(&mut p, i), find(&mut p, j));
                p[a] = b;
            }
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..n {
        let r = find(&mut p, i);
        groups.entry(r).or_default().push(i);
    }
    let mut out: Vec<Topic> = groups
        .into_values()
        .filter(|g| g.len() >= 2)
        .map(|g| {
            let mut count: BTreeMap<&String, usize> = BTreeMap::new();
            for &i in &g {
                for t in &top[i] {
                    *count.entry(t).or_default() += 1;
                }
            }
            let mut c: Vec<_> = count.into_iter().collect();
            c.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
            Topic {
                terms: c
                    .into_iter()
                    .take(4)
                    .map(|(k, _)| shown[k].clone())
                    .collect(),
                docs: g.iter().map(|&i| docs[i].0.clone()).collect(),
            }
        })
        .collect();
    out.sort_by_key(|t| std::cmp::Reverse(t.docs.len()));
    out
}

// ------------------------------------------------------------------ fact-check a draft

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Checked {
    pub sentence: String,
    pub status: &'static str,
    pub support: f64,
    pub source: Option<String>,
    pub evidence: Option<String>,
    /// numbers in the sentence that the best evidence does not contain
    pub unmatched_numbers: Vec<String>,
}

pub fn claim_sentences(text: &str) -> Vec<String> {
    sentences(text).into_iter().take(60).collect()
}

/// The sentence of `text` that best supports `claim`: `(support 0..1, sentence)`.
pub fn best_evidence(claim: &str, text: &str) -> Option<(f64, String)> {
    let mine = words(claim);
    if mine.is_empty() {
        return None;
    }
    sentence_list(text)
        .into_iter()
        .map(|s| {
            let w = words(&s);
            (
                mine.iter().filter(|x| w.contains(*x)).count() as f64 / mine.len() as f64,
                s,
            )
        })
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
}

pub fn judge(claim: &str, support: f64, evidence: &str) -> (&'static str, Vec<String>) {
    let ev: BTreeSet<String> = numbers(evidence).into_iter().collect();
    let un: Vec<String> = numbers(claim)
        .into_iter()
        .filter(|n| !ev.contains(n))
        .collect();
    let st = if support >= 0.6 && un.is_empty() {
        "supported"
    } else if support >= 0.6 {
        "numbers differ"
    } else if support >= 0.3 {
        "partial"
    } else {
        "not found"
    };
    (st, un)
}

// ------------------------------------------------------------------ on top of a library

use crate::library::{Filter, Library};

fn or_query(text: &str) -> String {
    let mut w: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 4)
        .map(|w| w.to_lowercase())
        .collect();
    w.sort_by_key(|x| std::cmp::Reverse(x.chars().count()));
    w.dedup();
    w.truncate(8);
    w.join(" OR ")
}

/// Checks every sentence of `text` (your own draft) against what your library says.
pub fn factcheck(lib: &Library, text: &str) -> Result<Vec<Checked>, String> {
    let mut out = vec![];
    for claim in claim_sentences(text) {
        let q = or_query(&claim);
        let hits = if q.is_empty() {
            vec![]
        } else {
            lib.search(
                &q,
                &Filter {
                    limit: Some(4),
                    ..Default::default()
                },
            )
            .unwrap_or_default()
        };
        let mut best: Option<(f64, String, String)> = None;
        for h in hits {
            if let Some((sup, ev)) = best_evidence(&claim, &lib.text_of(h.id))
                && best.as_ref().is_none_or(|b| sup > b.0)
            {
                best = Some((sup, h.path.clone(), ev));
            }
        }
        out.push(match best {
            Some((sup, path, ev)) => {
                let (status, un) = judge(&claim, sup, &ev);
                Checked {
                    sentence: claim,
                    status,
                    support: (sup * 100.0).round() / 100.0,
                    source: Some(path),
                    evidence: Some(ev),
                    unmatched_numbers: un,
                }
            }
            None => Checked {
                sentence: claim,
                status: "not found",
                support: 0.0,
                source: None,
                evidence: None,
                unmatched_numbers: vec![],
            },
        });
    }
    Ok(out)
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct DossierDoc {
    pub path: String,
    pub passages: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Dossier {
    pub term: String,
    pub docs: Vec<DossierDoc>,
    pub dates: Vec<String>,
    pub money: Vec<String>,
}

/// Everything your library says about one term, document by document, with the dates and amounts
/// found in those passages.
pub fn dossier(lib: &Library, term: &str) -> Result<Dossier, String> {
    let term = term.trim();
    if term.is_empty() {
        return Err("empty query".into());
    }
    let hits = lib.search(
        term,
        &Filter {
            limit: Some(30),
            ..Default::default()
        },
    )?;
    let needles: Vec<String> = term
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 3)
        .map(|w| w.to_lowercase())
        .collect();
    let (mut dates, mut money) = (BTreeSet::new(), BTreeSet::new());
    let mut docs = vec![];
    for h in hits {
        let text = lib.text_of(h.id);
        let passages: Vec<String> = sentence_list(&text)
            .into_iter()
            .filter(|s| {
                let l = s.to_lowercase();
                needles.iter().any(|n| l.contains(n.as_str()))
            })
            .take(3)
            .collect();
        for p in &passages {
            for e in entities(p) {
                match e.kind {
                    "date" => dates.insert(e.value),
                    "money" => money.insert(e.value),
                    _ => false,
                };
            }
        }
        if !passages.is_empty() {
            docs.push(DossierDoc {
                path: h.path,
                passages,
            });
        }
    }
    Ok(Dossier {
        term: term.to_string(),
        docs,
        dates: dates.into_iter().collect(),
        money: money.into_iter().collect(),
    })
}

/// Documents modified at or after `since` (Unix seconds), newest first.
pub fn changes(
    lib: &Library,
    since: i64,
    limit: usize,
) -> Result<Vec<crate::library::Doc>, String> {
    let mut d: Vec<_> = lib
        .documents(None)?
        .into_iter()
        .filter(|d| d.mtime >= since)
        .collect();
    d.sort_by_key(|d| std::cmp::Reverse(d.mtime));
    d.truncate(limit);
    Ok(d)
}

/// All library texts as `(path, text)`, for the whole-library analyses.
pub fn library_texts(lib: &Library, group: Option<&str>) -> Result<Vec<(String, String)>, String> {
    Ok(lib
        .documents(group)?
        .into_iter()
        .map(|d| (d.path, lib.text_of(d.id)))
        .collect())
}
