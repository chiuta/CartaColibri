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
