//! Scheduled digests: re-run a saved search and report only the documents that are new since the
//! last run, optionally with a summary written by the local engine. Nothing is ever modified.
use crate::carta::{self, Backend, Options};
use crate::library::{Library, Res};
use crate::organize;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn seen_file(lib: &Library, name: &str) -> PathBuf {
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    lib.home.join(format!("digest-{safe}.json"))
}

fn load_seen(p: &PathBuf) -> BTreeSet<String> {
    std::fs::read_to_string(p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

/// One run. Returns `None` when nothing new matched, otherwise the Markdown report.
/// The very first run only records what already matches (no flood of old documents) unless `first_all`.
pub fn run_once(
    lib: &Library,
    name: &str,
    be: Option<&Backend>,
    first_all: bool,
) -> Res<Option<String>> {
    let hits = organize::saved_run(lib, name)?;
    let file = seen_file(lib, name);
    let first = !file.exists();
    let mut seen = load_seen(&file);
    let new: Vec<_> = hits.iter().filter(|h| !seen.contains(&h.path)).collect();
    seen.extend(hits.iter().map(|h| h.path.clone()));
    std::fs::write(&file, serde_json::to_string(&seen).unwrap()).map_err(|e| e.to_string())?;
    if new.is_empty() || (first && !first_all) {
        return Ok(None);
    }
    let mut out = format!("## {name} — {} new\n\n", new.len());
    for h in &new {
        out.push_str(&format!(
            "- `{}` ({}) — {}\n",
            h.path,
            h.loc,
            h.snippet.replace('\n', " ")
        ));
    }
    if let Some(be) = be {
        let docs: Vec<_> = lib
            .documents(None)?
            .into_iter()
            .filter(|d| new.iter().any(|h| h.id == d.id))
            .collect();
        let o = Options {
            request: "what is new and what matters",
            ..Options::default()
        };
        match carta::run(lib, "summarize", &docs, be, &o, &mut |_| {}) {
            Ok(s) => out.push_str(&format!("\n{s}\n")),
            Err(e) => out.push_str(&format!("\n(summary unavailable: {e})\n")),
        }
    }
    Ok(Some(out))
}
