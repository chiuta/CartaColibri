//! RAG-CARTA: summarize / extract / synthesize over a group of indexed documents, through the
//! Colibrì gateway (OpenAI-compatible `/v1/chat/completions`). Every result keeps [S#] citations
//! and ends with a Sources list. `Backend::mock()` is a deterministic offline extractive fallback.

use crate::library::{Doc, Library};
use regex::Regex;
use serde_json::{Value, json};
use std::time::Duration;

pub const TASKS: &[&str] = &["summarize", "extract", "synthesize"];
pub const DEFAULT_URL: &str = "http://127.0.0.1:8000";
pub const CTX_CHARS: usize = 18000;

const SYSTEM: &str = "You are CartaColibri, a careful local document assistant. Use ONLY the provided source text; never add outside knowledge. \
Cite sources as [S1], [S2] ... exactly as labelled. If something is not in the sources, say so instead of guessing. \
Answer in the language of the user's request (default: the language of the documents).";

pub struct Backend {
    pub mock: bool,
    pub url: String,
    pub model: Option<String>,
    pub timeout: Duration,
    pub max_tokens: u32,
}

impl Backend {
    pub fn gateway(url: &str, model: Option<String>) -> Backend {
        Backend {
            mock: false,
            url: url.trim_end_matches('/').to_string(),
            model,
            timeout: Duration::from_secs(1800),
            max_tokens: 1200,
        }
    }
    pub fn mock() -> Backend {
        Backend {
            mock: true,
            url: String::new(),
            model: None,
            timeout: Duration::from_secs(1),
            max_tokens: 0,
        }
    }

    pub fn complete(&self, system: &str, user: &str) -> Result<String, String> {
        if self.mock {
            return Ok(mock(user));
        }
        let mut body = json!({"messages": [{"role": "system", "content": system}, {"role": "user", "content": user}],
                              "max_tokens": self.max_tokens, "temperature": 0.2, "stream": false});
        if let Some(m) = &self.model {
            body["model"] = json!(m);
        }
        let fail = |e: String| {
            format!(
                "no answer from the Colibrì gateway at {} ({e}). Start it with `coli web --model DIR` (or `coli serve`), or use --mock.",
                self.url
            )
        };
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(self.timeout))
            .http_status_as_error(true)
            .build()
            .into();
        let mut resp = agent
            .post(&format!("{}/v1/chat/completions", self.url))
            .send_json(&body)
            .map_err(|e| fail(e.to_string()))?;
        let v: Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| fail(e.to_string()))?;
        v["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| fail("unexpected response".into()))
    }

    pub fn alive(&self) -> bool {
        if self.mock {
            return true;
        }
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(3)))
            .build()
            .into();
        agent.get(&format!("{}/health", self.url)).call().is_ok()
    }
}

/// Deterministic stand-in: the first informative sentence of each line, in order.
fn mock(user: &str) -> String {
    let body = Regex::new(r"(?s)<<SOURCES>>(.*)<<END>>")
        .unwrap()
        .captures(user)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| user.to_string());
    let sent = Regex::new(r"[.!?]\s").unwrap();
    let (mut keep, mut bullets) = (Vec::<String>::new(), 0);
    for line in body.lines() {
        let line = line.trim();
        if line.starts_with("### [S") {
            keep.push(line.to_string());
        } else if line.chars().count() > 40 && bullets < 40 {
            let l = line.trim_start_matches(['-', '*', ' ']);
            let first = sent.find(l).map(|m| &l[..m.start() + 1]).unwrap_or(l);
            keep.push(format!("- {}", first.chars().take(220).collect::<String>()));
            bullets += 1;
        }
    }
    keep.truncate(30);
    if keep.is_empty() {
        "(no text)".into()
    } else {
        keep.join("\n")
    }
}

fn base(p: &str) -> &str {
    p.rsplit(['/', '\\']).next().unwrap_or(p)
}

fn blocks<'a>(
    lib: &Library,
    docs: &'a [Doc],
    labels: &[String],
    budget: usize,
) -> Vec<Vec<(&'a Doc, String)>> {
    let mut pieces: Vec<(&Doc, String)> = vec![];
    for (i, d) in docs.iter().enumerate() {
        let text = lib.text_of(d.id);
        let chars: Vec<char> = text.trim().chars().collect();
        for part in chars.chunks(budget.max(1)) {
            let s: String = part.iter().collect();
            if !s.trim().is_empty() {
                pieces.push((d, format!("### [{}] {}\n{}", labels[i], base(&d.path), s)));
            }
        }
    }
    let (mut batches, mut cur, mut size) = (vec![], vec![], 0usize);
    for (d, p) in pieces {
        if !cur.is_empty() && size + p.chars().count() > budget {
            batches.push(std::mem::take(&mut cur));
            size = 0;
        }
        size += p.chars().count();
        cur.push((d, p));
    }
    if !cur.is_empty() {
        batches.push(cur);
    }
    batches
}

fn wrap(parts: &[String]) -> String {
    format!("<<SOURCES>>\n{}\n<<END>>", parts.join("\n\n"))
}

pub struct Options<'a> {
    pub request: &'a str,
    pub fields: Vec<String>,
    pub json: bool,
    pub budget: usize,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Options {
            request: "",
            fields: vec![],
            json: false,
            budget: CTX_CHARS,
        }
    }
}

pub fn run(
    lib: &Library,
    task: &str,
    docs: &[Doc],
    be: &Backend,
    o: &Options,
    log: &mut dyn FnMut(&str),
) -> Result<String, String> {
    if !TASKS.contains(&task) {
        return Err(format!("task must be one of {TASKS:?}"));
    }
    if docs.is_empty() {
        return Err(
            "the group has no indexed documents (run `index` first, or check its members)".into(),
        );
    }
    let labels: Vec<String> = (1..=docs.len()).map(|i| format!("S{i}")).collect();
    let mut ask = |instr: &str, batch: &[String]| -> Result<String, String> {
        log(&format!("model call ({} piece(s))", batch.len()));
        be.complete(SYSTEM, &format!("{instr}\n\n{}", wrap(batch)))
    };
    let body = match task {
        "summarize" => {
            let focus = if o.request.is_empty() {
                String::new()
            } else {
                format!(" Focus on: {}.", o.request)
            };
            let mut parts = vec![];
            for (i, d) in docs.iter().enumerate() {
                let one = std::slice::from_ref(d);
                let mut notes = vec![];
                for b in blocks(lib, one, &[labels[i].clone()], o.budget) {
                    let texts: Vec<String> = b.into_iter().map(|x| x.1).collect();
                    notes.push(ask(&format!("Summarize the source below in concise key points, faithful to the text, citing [{}].{focus}", labels[i]), &texts)?);
                }
                if notes.len() > 1 {
                    let merged = be.complete(
                        SYSTEM,
                        &format!("Merge these partial summaries into one concise summary of the document; keep the citation [{}].\n\n{}", labels[i], wrap(&notes)),
                    )?;
                    notes = vec![merged];
                }
                parts.push(format!(
                    "## [{}] {}\n\n{}",
                    labels[i],
                    base(&d.path),
                    notes.first().cloned().unwrap_or_else(|| "(no text)".into())
                ));
            }
            let mut body = parts.join("\n\n");
            if docs.len() > 1 {
                let ov = be.complete(
                    SYSTEM,
                    &format!(
                        "Write a short overview (5-8 bullet points) of what these per-document summaries cover, keeping the [S#] citations.\n\n{}",
                        wrap(&parts)
                    ),
                )?;
                body = format!("## Overview\n\n{ov}\n\n{body}");
            }
            body
        }
        "extract" => {
            let spec = if o.fields.is_empty() {
                o.request.to_string()
            } else {
                o.fields.join(", ")
            };
            if spec.is_empty() {
                return Err("extract needs --fields (comma-separated) or a request".into());
            }
            let shape = if !o.json {
                format!(
                    "a Markdown table with columns: Source, {}",
                    if o.fields.is_empty() {
                        "the requested fields".into()
                    } else {
                        o.fields.join(", ")
                    }
                )
            } else {
                "a JSON array of objects, each with a \"source\" key like \"S1\" plus the requested fields".to_string()
            };
            let mut rows = vec![];
            for (i, d) in docs.iter().enumerate() {
                for b in blocks(lib, std::slice::from_ref(d), &[labels[i].clone()], o.budget) {
                    let texts: Vec<String> = b.into_iter().map(|x| x.1).collect();
                    rows.push(ask(
                        &format!(
                            "Extract these items from the source: {spec}. Output {shape}, one row/object per finding. Use \"NOT FOUND\" for anything missing — never guess. Cite [{}]. Output only the {}.",
                            labels[i],
                            if o.json { "JSON" } else { "table" }
                        ),
                        &texts,
                    )?);
                }
            }
            merge_extract(&rows, o.json)
        }
        _ => {
            if o.request.is_empty() {
                return Err("synthesize needs a question or theme".into());
            }
            let mut notes = vec![];
            for b in blocks(lib, docs, &labels, o.budget) {
                let texts: Vec<String> = b.into_iter().map(|x| x.1).collect();
                notes.push(ask(
                    &format!(
                        "Question/theme: {}\nList the relevant facts, claims and figures from these sources as short bullet points, each with its [S#] citation. Skip irrelevant text.",
                        o.request
                    ),
                    &texts,
                )?);
            }
            be.complete(
                SYSTEM,
                &format!(
                    "Question/theme: {}\n\nUsing ONLY the notes below, write an integrated synthesis: group ideas by theme, state the points the sources share, and flag differences or contradictions between them. Cite every claim with [S#].\n\n{}",
                    o.request,
                    wrap(&notes)
                ),
            )?
        }
    };
    let sources: Vec<String> = docs
        .iter()
        .enumerate()
        .map(|(i, d)| format!("- [{}] {}", labels[i], d.path))
        .collect();
    Ok(format!(
        "{}\n\n---\n**Sources**\n{}\n",
        body.trim(),
        sources.join("\n")
    ))
}

fn merge_extract(rows: &[String], as_json: bool) -> String {
    if as_json {
        let arr = Regex::new(r"(?s)\[.*\]").unwrap();
        let mut out: Vec<Value> = vec![];
        for r in rows {
            match arr
                .find(r)
                .and_then(|m| serde_json::from_str::<Vec<Value>>(m.as_str()).ok())
            {
                Some(v) => out.extend(v),
                None => out.push(json!({"raw": r})),
            }
        }
        return serde_json::to_string_pretty(&out).unwrap_or_default();
    }
    let sep = Regex::new(r"^\|[\s:|-]+\|$").unwrap();
    let (mut header, mut lines) = (None::<String>, Vec::<String>::new());
    for r in rows {
        for ln in r.lines() {
            let l = ln.trim();
            if !l.starts_with('|') || sep.is_match(l) {
                continue;
            }
            match &header {
                None => header = Some(l.to_string()),
                Some(h) if h != l => lines.push(l.to_string()),
                _ => {}
            }
        }
    }
    let Some(h) = header else {
        return rows.join("\n\n");
    };
    let cols = h.matches('|').count().saturating_sub(1);
    let mut out = vec![h, format!("|{}", "---|".repeat(cols))];
    out.extend(lines);
    out.join("\n")
}

/// Write a result under <home>/outputs/ — labelled, and never re-indexed (the folder is skipped).
pub fn save(
    home: &std::path::Path,
    text: &str,
    task: &str,
    group: &str,
    request: &str,
) -> Result<String, String> {
    let d = home.join("outputs");
    std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let safe: String = group
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let is_json = text.trim_start().starts_with(['[', '{']);
    let p = d.join(format!(
        "{secs}_{safe}_{task}{}",
        if is_json { ".json" } else { ".md" }
    ));
    let head = if is_json {
        String::new()
    } else {
        format!(
            "<!-- cartacolibri | unix {secs} | group: {group} | task: {task} | request: {request} -->\n"
        )
    };
    std::fs::write(&p, format!("{head}{text}")).map_err(|e| e.to_string())?;
    Ok(p.to_string_lossy().into_owned())
}

/// Ask a question about ONE page (used by the web-grab browser): the page is source [S1].
pub fn ask_page(
    be: &Backend,
    question: &str,
    title: &str,
    url: &str,
    text: &str,
) -> Result<String, String> {
    if question.trim().is_empty() {
        return Err("empty question".into());
    }
    let body: String = text.chars().take(CTX_CHARS).collect();
    let prompt = format!(
        "Question: {question}\n\nAnswer using ONLY the page below and cite it as [S1]. If the page does not say, say so.\n\n{}",
        wrap(&[format!("### [S1] {title}\n{body}")])
    );
    let ans = be.complete(SYSTEM, &prompt)?;
    Ok(format!("{}\n\n[S1] {url}", ans.trim()))
}
