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

/// A plain-text e-mail (RFC 5322 headers, UTF-8 body) as bytes. The address is checked so it can
/// never smuggle extra headers.
pub fn eml(to: &str, subject: &str, body: &str) -> Res<String> {
    let ok = to.contains('@')
        && to.len() <= 200
        && !to
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "<>,;\"".contains(c));
    if !ok {
        return Err(format!("'{to}' is not a plain e-mail address"));
    }
    let subj: String = subject
        .chars()
        .filter(|c| !c.is_control())
        .take(120)
        .collect();
    let date = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    Ok(format!(
        "From: CartaColibri <cartacolibri@localhost>\r\nTo: {to}\r\nSubject: =?UTF-8?B?{}?=\r\nX-CartaColibri-Time: {date}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: 8bit\r\n\r\n{}\r\n",
        b64(subj.as_bytes()),
        body.replace("\r\n", "\n").replace('\n', "\r\n")
    ))
}

fn b64(d: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = String::new();
    for c in d.chunks(3) {
        let n = (c[0] as u32) << 16
            | (*c.get(1).unwrap_or(&0) as u32) << 8
            | *c.get(2).unwrap_or(&0) as u32;
        o.push(T[(n >> 18) as usize & 63] as char);
        o.push(T[(n >> 12) as usize & 63] as char);
        o.push(if c.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        o.push(if c.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    o
}

/// Saves the e-mail in `<library>/outbox/` and, only when `send` is true, hands it to the
/// computer's own `sendmail`. Returns the saved file.
pub fn deliver(lib: &Library, name: &str, to: &str, body: &str, send: bool) -> Res<PathBuf> {
    let msg = eml(to, &format!("CartaColibri: {name}"), body)?;
    let dir = lib.home.join("outbox");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
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
    let f = dir.join(format!("{safe}-{stamp}.eml"));
    std::fs::write(&f, &msg).map_err(|e| e.to_string())?;
    if send {
        use std::io::Write;
        let mut ch = std::process::Command::new("sendmail")
            .arg("-t")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                format!(
                    "no local sendmail to hand it to ({e}); the e-mail is saved in {}",
                    f.display()
                )
            })?;
        ch.stdin
            .take()
            .ok_or("sendmail has no input")?
            .write_all(msg.as_bytes())
            .map_err(|e| e.to_string())?;
        ch.wait().map_err(|e| e.to_string())?;
    }
    Ok(f)
}
