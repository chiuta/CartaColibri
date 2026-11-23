//! `cartacolibri doctor`: environment checks with copy-paste fixes.

use crate::library::{self, Library};
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Level {
    Ok,
    Warn,
    Fail,
}

pub struct Check {
    pub level: Level,
    pub name: &'static str,
    pub detail: String,
    pub fix: Option<String>,
}

fn ck(level: Level, name: &'static str, detail: impl Into<String>, fix: Option<String>) -> Check {
    Check {
        level,
        name,
        detail: detail.into(),
        fix,
    }
}

fn runs(cmd: &str, arg: &str) -> bool {
    Command::new(cmd)
        .arg(arg)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn fix3(linux: &str, mac: &str, win: &str) -> String {
    format!("Linux: {linux}  |  macOS: {mac}  |  Windows: {win}")
}

/// Whether anything answers HTTP at `url` (any status counts).
pub fn gateway_up(url: &str) -> bool {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(2)))
        .http_status_as_error(false)
        .build()
        .into();
    agent.get(url).call().is_ok()
}

pub fn run(home: Option<&str>, url: &str, port: u16) -> Vec<Check> {
    let mut v = vec![];

    if runs("tesseract", "--version") {
        v.push(ck(
            Level::Ok,
            "tesseract",
            "found (OCR for scans and images)",
            None,
        ));
    } else {
        v.push(ck(
            Level::Warn,
            "tesseract",
            "missing (optional: needed only to read scanned pages and images)",
            Some(fix3(
                "sudo apt install tesseract-ocr",
                "brew install tesseract",
                "winget install UB-Mannheim.TesseractOCR",
            )),
        ));
    }

    let (t, r) = (runs("pdftotext", "-v"), runs("pdftoppm", "-v"));
    if t && r {
        v.push(ck(
            Level::Ok,
            "poppler",
            "pdftotext and pdftoppm found",
            None,
        ));
    } else {
        v.push(ck(
            Level::Warn,
            "poppler",
            "pdftotext/pdftoppm missing (optional: better PDF reading, needed for scanned PDFs)",
            Some(fix3(
                "sudo apt install poppler-utils",
                "brew install poppler",
                "winget install oschwartz10612.Poppler",
            )),
        ));
    }

    if gateway_up(url) {
        v.push(ck(
            Level::Ok,
            "gateway",
            format!("Colibrì answers at {url}"),
            None,
        ));
    } else {
        v.push(ck(
            Level::Warn,
            "gateway",
            format!("Colibrì gateway not reachable at {url} (summarize/extract/synthesize will need --mock)"),
            Some("coli web --model DIR   (same command on Linux/macOS/Windows; or add --mock)".into()),
        ));
    }

    match library::state_dir(home) {
        Ok(h) => {
            let probe = h.join(".doctor-write-test");
            if std::fs::write(&probe, b"ok").is_ok() {
                let _ = std::fs::remove_file(&probe);
                v.push(ck(
                    Level::Ok,
                    "home",
                    format!("writable: {}", h.display()),
                    None,
                ));
                match Library::open(home, false) {
                    Ok(lib) => {
                        let docs = lib.documents(None).map(|d| d.len()).unwrap_or(0);
                        let kb = std::fs::metadata(h.join("index.sqlite"))
                            .map(|m| m.len() / 1024)
                            .unwrap_or(0);
                        if docs == 0 {
                            v.push(ck(
                                Level::Warn,
                                "index",
                                format!("empty ({kb} KB, {} folder(s))", lib.roots().len()),
                                Some("cartacolibri start".into()),
                            ));
                        } else {
                            v.push(ck(Level::Ok, "index", format!("{docs} document(s), {kb} KB"), None));
                        }
                    }
                    Err(e) => v.push(ck(
                        Level::Fail,
                        "index",
                        format!("cannot open the index: {e}"),
                        Some("move or delete index.sqlite in the home folder, then run: cartacolibri index".into()),
                    )),
                }
            } else {
                v.push(ck(
                    Level::Fail,
                    "home",
                    format!("not writable: {}", h.display()),
                    Some(fix3(
                        "chmod u+rwx <folder>  or use --home ~/cartacolibri",
                        "chmod u+rwx <folder>  or use --home ~/cartacolibri",
                        "pick a folder you own: --home %USERPROFILE%\\cartacolibri",
                    )),
                ));
            }
        }
        Err(e) => v.push(ck(
            Level::Fail,
            "home",
            format!("cannot create the state folder: {e}"),
            Some("use another folder: cartacolibri --home ~/cartacolibri doctor".into()),
        )),
    }

    match std::net::TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => v.push(ck(
            Level::Ok,
            "port",
            format!("{port} is free for the web UI"),
            None,
        )),
        Err(_) => v.push(ck(
            Level::Warn,
            "port",
            format!("{port} is busy (maybe CartaColibri already runs there)"),
            Some(format!(
                "cartacolibri start --port {}",
                port.saturating_add(1)
            )),
        )),
    }
    v
}
