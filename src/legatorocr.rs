//! OCR through LegatorOCR (github.com/chiuta/LegatorOCR, MIT), the clean-room scan post-processor
//! with built-in OCR. CartaColibri only ever *runs* it as an external command, never links it.
//!
//! This is the same small module Colibri Companion uses (`crates/core/src/legatorocr.rs` there,
//! same author), shared here so both apps find and call LegatorOCR the same way: look for the
//! program, check that `--version` answers like it, run
//! `legatorocr PAGES --ocr --lang CODE --txt FILE --format none --out-dir DIR`, read the text back.
//! Arguments are passed one by one (no shell), every path is made absolute so a file called `-x.png`
//! is never taken for an option, the language code is checked, output is capped and a run that does
//! not end is killed. The text is OCR: never a verified transcription.

use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Name of the program, without a platform extension.
pub const PROGRAM: &str = "legatorocr";
/// Longest text read back from the program.
pub const MAX_TEXT_BYTES: u64 = 32 * 1024 * 1024;

fn exe_name() -> String {
    if cfg!(windows) {
        format!("{PROGRAM}.exe")
    } else {
        PROGRAM.to_string()
    }
}

/// A relative folder (an empty `PATH` item, `.`) is read from the working folder by some shells, so a
/// program of that name planted in a folder the person merely opened would run. Absolute folders only.
fn searchable(dir: &Path) -> bool {
    dir.is_absolute()
}

/// A file that can be started: on Unix it has an execute bit.
fn startable(p: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        p.is_file() && std::fs::metadata(p).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        p.is_file()
    }
}

/// Looks in `extra` (a folder, or the program itself), then in the folder of this program and its
/// sibling `LegatorOCR` folder (portable layouts), then in every folder of `path`.
pub fn locate_in(extra: &[PathBuf], path: Option<&std::ffi::OsStr>) -> Option<PathBuf> {
    let name = exe_name();
    for x in extra {
        if searchable(x) && startable(x) {
            return Some(x.clone());
        }
    }
    let mut dirs: Vec<PathBuf> = extra.to_vec();
    if let Ok(me) = std::env::current_exe()
        && let Some(d) = me.parent()
    {
        dirs.push(d.to_path_buf());
        if let Some(up) = d.parent() {
            dirs.push(up.join("LegatorOCR"));
        }
    }
    if let Some(p) = path {
        dirs.extend(std::env::split_paths(p));
    }
    dirs.iter()
        .filter(|d| searchable(d))
        .map(|d| d.join(&name))
        .find(|c| startable(c))
}

/// The program, from `CARTACOLIBRI_LEGATOROCR` (file or folder) or the usual places, and only when
/// `--version` answers like LegatorOCR (an unrelated file of the same name is not taken for it).
pub fn locate() -> Option<PathBuf> {
    let extra: Vec<PathBuf> = std::env::var_os("CARTACOLIBRI_LEGATOROCR")
        .map(|v| vec![PathBuf::from(v)])
        .unwrap_or_default();
    let bin = locate_in(&extra, std::env::var_os("PATH").as_deref())?;
    version(&bin).map(|_| bin)
}

/// The LegatorOCR window (`legatorocr-gui`), looked for beside the `legatorocr` program that was found.
pub fn locate_gui() -> Option<PathBuf> {
    let name = if cfg!(windows) { format!("{PROGRAM}-gui.exe") } else { format!("{PROGRAM}-gui") };
    let cli = locate()?;
    let gui = cli.parent()?.join(name);
    startable(&gui).then_some(gui)
}

/// The version line (`legatorocr 0.2.0-pre.65`), or `None` when the program does not answer like LegatorOCR.
pub fn version(bin: &Path) -> Option<String> {
    let mut cmd = Command::new(bin);
    cmd.arg("--version");
    let out = crate::extract::run_cmd(cmd, Duration::from_secs(10)).ok()?;
    if !out.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();
    line.to_ascii_lowercase()
        .starts_with(PROGRAM)
        .then_some(line)
}

/// Letters, digits, `_`, `+`, `-` only, at most 40 characters, not starting with `-`.
pub fn clean_lang(s: &str) -> Option<String> {
    let ok = !s.is_empty()
        && s.len() <= 40
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '_' | '-'))
        && !s.starts_with('-');
    ok.then(|| s.to_string())
}

/// On Windows `canonicalize` gives `\\?\C:\dir\f.png`, a form many programs cannot open: back to `C:\dir\f.png`.
pub fn plain_windows_path(p: &Path) -> PathBuf {
    let Some(s) = p.to_str() else {
        return p.to_path_buf();
    };
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    match s.strip_prefix(r"\\?\") {
        Some(rest)
            if rest.len() >= 3
                && rest.as_bytes()[1] == b':'
                && rest.as_bytes()[0].is_ascii_alphabetic() =>
        {
            PathBuf::from(rest)
        }
        _ => p.to_path_buf(),
    }
}

/// An absolute form of `p` that does not need the file to exist and never starts with `-`.
fn absolute(p: &Path) -> PathBuf {
    let abs = std::fs::canonicalize(p)
        .or_else(|_| std::path::absolute(p))
        .unwrap_or_else(|_| p.to_path_buf());
    let abs = plain_windows_path(&abs);
    if abs.is_absolute() {
        abs
    } else {
        Path::new(".").join(abs)
    }
}

/// The command line, as separate arguments.
pub fn build_command(bin: &Path, input: &Path, lang: &str, txt: &Path, out_dir: &Path) -> Command {
    let mut cmd = Command::new(bin);
    let mut args: Vec<OsString> = vec![absolute(input).into_os_string()];
    args.extend(["--ocr".into(), "--lang".into(), lang.into(), "--txt".into()]);
    args.push(absolute(txt).into_os_string());
    args.extend(["--format".into(), "none".into(), "--out-dir".into()]);
    args.push(absolute(out_dir).into_os_string());
    if lang.split('+').any(|l| l == "ron") {
        // comma-below ș ț, words split at a line end joined
        args.push("--clean-text".into());
    }
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // no console window
    }
    cmd
}

/// UTF-8 (a byte-order mark is dropped) or UTF-16 with a byte-order mark; invalid bytes are replaced; NULs dropped.
pub fn decode_text(bytes: &[u8]) -> String {
    let utf16 = |b: &[u8], le: bool| {
        let units = b.as_chunks::<2>().0.iter().map(|c| {
            if le {
                u16::from_le_bytes([c[0], c[1]])
            } else {
                u16::from_be_bytes([c[0], c[1]])
            }
        });
        char::decode_utf16(units)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect::<String>()
    };
    let text = match bytes {
        [0xFF, 0xFE, rest @ ..] => utf16(rest, true),
        [0xFE, 0xFF, rest @ ..] => utf16(rest, false),
        [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8_lossy(rest).into_owned(),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    };
    text.replace('\0', "")
}

fn tail_lines(stderr: &[u8]) -> String {
    let end = &stderr[stderr.len().saturating_sub(8 * 1024)..];
    let s = String::from_utf8_lossy(end);
    let lines: Vec<&str> = s.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    lines[lines.len().saturating_sub(4)..]
        .join("\n")
        .chars()
        .take(600)
        .collect()
}

/// Runs LegatorOCR on one image or PDF and returns the recognised text. `work` is a private folder
/// for its output (created here, the caller removes it).
pub fn read_text(
    bin: &Path,
    input: &Path,
    lang: &str,
    work: &Path,
    timeout: Duration,
) -> Result<String, String> {
    let lang = clean_lang(lang).ok_or("bad language code")?;
    std::fs::create_dir_all(work).map_err(|e| e.to_string())?;
    let txt = work.join("scan.txt");
    let _ = std::fs::remove_file(&txt);
    let cmd = build_command(bin, input, &lang, &txt, &work.join("pages"));
    let out = crate::extract::run_cmd(cmd, timeout)?;
    if !out.status.success() {
        return Err(format!("legatorocr failed: {}", tail_lines(&out.stderr)));
    }
    let mut buf = Vec::new();
    std::fs::File::open(&txt)
        .map_err(|_| "legatorocr wrote no text (blank pages, or no OCR language data: run `legatorocr --doctor`)".to_string())?
        .take(MAX_TEXT_BYTES)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    let mut text = decode_text(&buf);
    if buf.len() as u64 >= MAX_TEXT_BYTES {
        while text.ends_with(char::REPLACEMENT_CHARACTER) {
            text.pop();
        }
    }
    if text.trim().is_empty() {
        return Err("legatorocr found no text on the page".into());
    }
    Ok(text)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn temp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("cc-lgr-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
    fn fake(dir: &Path, body: &str) -> PathBuf {
        let p = dir.join(PROGRAM);
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p
    }
    const OK: &str = r#"
if [ "$1" = "--version" ]; then echo "legatorocr 0.0-test"; exit 0; fi
while [ $# -gt 0 ]; do if [ "$1" = "--txt" ]; then shift; out="$1"; fi; shift; done
printf 'The harbour office opened at nine.\n' > "$out"
"#;

    #[test]
    fn language_codes_are_checked() {
        assert_eq!(clean_lang("ron+eng").as_deref(), Some("ron+eng"));
        assert!(
            clean_lang("--version").is_none()
                && clean_lang("a b").is_none()
                && clean_lang("").is_none()
        );
    }

    #[test]
    fn finds_only_a_program_that_answers_like_legatorocr() {
        let d = temp("find");
        let good = fake(&d, OK);
        assert_eq!(
            locate_in(std::slice::from_ref(&d), Some(std::ffi::OsStr::new(""))),
            Some(good.clone())
        );
        assert!(version(&good).unwrap().starts_with("legatorocr"));
        let e = temp("other");
        let other = fake(&e, "echo hello");
        assert!(version(&other).is_none());
        assert!(
            locate_in(
                &[PathBuf::from("relative")],
                Some(std::ffi::OsStr::new("."))
            )
            .is_none()
        );
    }

    #[test]
    fn reads_the_text_back_and_reports_failures() {
        let d = temp("run");
        let bin = fake(&d, OK);
        let png = d.join("-x.png");
        std::fs::write(&png, b"x").unwrap();
        let t = read_text(&bin, &png, "ron+eng", &d.join("w"), Duration::from_secs(20)).unwrap();
        assert!(t.contains("harbour"));
        let bad = fake(&temp("bad"), "echo boom >&2; exit 3");
        let e = read_text(&bad, &png, "eng", &d.join("w2"), Duration::from_secs(20)).unwrap_err();
        assert!(e.contains("boom"), "{e}");
        let none = fake(&temp("none"), "exit 0");
        assert!(read_text(&none, &png, "eng", &d.join("w3"), Duration::from_secs(20)).is_err());
        assert!(read_text(&bin, &png, "-bad", &d.join("w4"), Duration::from_secs(20)).is_err());
    }

    #[test]
    fn utf16_and_bom_are_decoded() {
        assert_eq!(decode_text(&[0xFF, 0xFE, b'h', 0, b'i', 0]), "hi");
        assert_eq!(decode_text(&[0xEF, 0xBB, 0xBF, b'o', b'k']), "ok");
    }
}
