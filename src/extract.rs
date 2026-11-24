//! Text extraction: every supported format -> a list of (location, text).
//! Clean-room: only public format specifications (OOXML, ODF, EPUB, RTF) were used.

use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

pub const MAX_ZIP_PART: u64 = 100 * 1024 * 1024;
pub const CHUNK_CHARS: usize = 1400;
pub const IMG_EXT: &[&str] = &[".png", ".jpg", ".jpeg", ".tif", ".tiff", ".bmp", ".webp"];

pub type Parts = Vec<(String, String)>;
pub type Res<T> = Result<T, String>;

fn re(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).unwrap())
}

/// UTF-8 (BOM ok), UTF-16 (BOM), else Windows-1250 (Romanian / Central-European legacy files).
pub fn decode(raw: &[u8]) -> String {
    if raw.starts_with(&[0xff, 0xfe]) {
        return encoding_rs::UTF_16LE.decode(&raw[2..]).0.into_owned();
    }
    if raw.starts_with(&[0xfe, 0xff]) {
        return encoding_rs::UTF_16BE.decode(&raw[2..]).0.into_owned();
    }
    let raw = raw.strip_prefix(&[0xef, 0xbb, 0xbf][..]).unwrap_or(raw);
    match std::str::from_utf8(raw) {
        Ok(s) => s.to_string(),
        Err(_) => encoding_rs::WINDOWS_1250.decode(raw).0.into_owned(),
    }
}

fn entity(name: &str) -> Option<String> {
    let c = match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => ' ',
        _ => {
            let n = name.strip_prefix('#')?;
            let code = match n.strip_prefix(['x', 'X']) {
                Some(h) => u32::from_str_radix(h, 16).ok()?,
                None => n.parse().ok()?,
            };
            char::from_u32(code)?
        }
    };
    Some(c.to_string())
}

/// A forgiving HTML -> text pass: scripts/styles dropped, block tags become newlines.
pub fn html_to_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len() / 2);
    let b = s.as_bytes();
    let (mut i, mut skip) = (0usize, 0u32);
    while i < b.len() {
        match b[i] {
            b'<' => {
                let rest = &s[i..];
                if rest.starts_with("<!--") {
                    i += rest.find("-->").map(|p| p + 3).unwrap_or(rest.len());
                    continue;
                }
                let end = rest.find('>').map(|p| p + 1).unwrap_or(rest.len());
                let tag = &rest[1..end.saturating_sub(1).max(1)];
                let closing = tag.starts_with('/');
                let name: String = tag
                    .trim_start_matches('/')
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric())
                    .collect::<String>()
                    .to_ascii_lowercase();
                match name.as_str() {
                    "script" | "style" => {
                        if closing {
                            skip = skip.saturating_sub(1)
                        } else if !tag.ends_with('/') {
                            skip += 1
                        }
                    }
                    "p" | "br" | "div" | "li" | "tr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                    | "section" | "article" | "table"
                        if skip == 0 =>
                    {
                        out.push('\n')
                    }
                    _ => {}
                }
                i += end;
            }
            b'&' if skip == 0 => {
                let rest = &s[i + 1..];
                match rest
                    .find(';')
                    .filter(|&p| p <= 10)
                    .and_then(|p| entity(&rest[..p]).map(|e| (p, e)))
                {
                    Some((p, e)) => {
                        out.push_str(&e);
                        i += p + 2;
                    }
                    None => {
                        out.push('&');
                        i += 1;
                    }
                }
            }
            _ => {
                let ch = s[i..].chars().next().unwrap();
                if skip == 0 {
                    out.push(ch)
                }
                i += ch.len_utf8();
            }
        }
    }
    out
}

fn local(name: &[u8]) -> &[u8] {
    match name.iter().rposition(|&c| c == b':') {
        Some(p) => &name[p + 1..],
        None => name,
    }
}

/// Text of an XML part. `text_in`: element names whose text counts (empty = all text);
/// `para`: elements that end a paragraph.
pub fn xml_text(data: &[u8], text_in: &[&str], para: &[&str]) -> Res<String> {
    let mut rd = Reader::from_reader(data);
    let mut out = String::new();
    let mut depth_in = 0u32;
    loop {
        match rd.read_event().map_err(|e| format!("xml: {e}"))? {
            Event::Start(e) => {
                if text_in
                    .iter()
                    .any(|t| t.as_bytes() == local(e.name().as_ref()))
                {
                    depth_in += 1;
                }
            }
            Event::Empty(e) => {
                if para
                    .iter()
                    .any(|t| t.as_bytes() == local(e.name().as_ref()))
                {
                    out.push('\n');
                }
            }
            Event::End(e) => {
                let n = e.name();
                let n = local(n.as_ref());
                if text_in.iter().any(|t| t.as_bytes() == n) {
                    depth_in = depth_in.saturating_sub(1);
                }
                if para.iter().any(|t| t.as_bytes() == n) {
                    out.push('\n');
                }
            }
            Event::Text(t) => {
                if text_in.is_empty() || depth_in > 0 {
                    out.push_str(&t.decode().map_err(|e| format!("xml: {e}"))?);
                }
            }
            Event::GeneralRef(r) => {
                if (text_in.is_empty() || depth_in > 0)
                    && let Ok(name) = std::str::from_utf8(&r)
                    && let Some(e) = entity(name)
                {
                    out.push_str(&e);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(out)
}

fn zip_read<R: Read + std::io::Seek>(z: &mut zip::ZipArchive<R>, name: &str) -> Res<Vec<u8>> {
    let mut f = z.by_name(name).map_err(|e| format!("{name}: {e}"))?;
    if f.size() > MAX_ZIP_PART {
        return Err(format!(
            "{name} unpacks to more than {} MB",
            MAX_ZIP_PART >> 20
        ));
    }
    let mut buf = Vec::new();
    f.by_ref()
        .take(MAX_ZIP_PART + 1)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    if buf.len() as u64 > MAX_ZIP_PART {
        return Err(format!(
            "{name} unpacks to more than {} MB",
            MAX_ZIP_PART >> 20
        ));
    }
    Ok(buf)
}

fn open_zip(path: &Path) -> Res<zip::ZipArchive<std::fs::File>> {
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    zip::ZipArchive::new(f).map_err(|e| format!("not a valid zip container: {e}"))
}

fn numbered(names: Vec<String>, prefix: &str, suffix: &str) -> Vec<(u64, String)> {
    let mut v: Vec<(u64, String)> = names
        .into_iter()
        .filter_map(|n| {
            let mid = n.strip_prefix(prefix)?.strip_suffix(suffix)?;
            Some((mid.parse().ok()?, n.clone()))
        })
        .collect();
    v.sort();
    v
}

fn xlsx_text(z: &mut zip::ZipArchive<std::fs::File>) -> Res<Parts> {
    let names: Vec<String> = z.file_names().map(String::from).collect();
    let mut shared: Vec<String> = Vec::new();
    if names.iter().any(|n| n == "xl/sharedStrings.xml") {
        let data = zip_read(z, "xl/sharedStrings.xml")?;
        let mut rd = Reader::from_reader(&data[..]);
        let (mut cur, mut in_t, mut in_si) = (String::new(), false, false);
        loop {
            match rd.read_event().map_err(|e| e.to_string())? {
                Event::Start(e) => match local(e.name().as_ref()) {
                    b"si" => {
                        in_si = true;
                        cur.clear()
                    }
                    b"t" => in_t = true,
                    _ => {}
                },
                Event::End(e) => match local(e.name().as_ref()) {
                    b"si" => {
                        in_si = false;
                        shared.push(std::mem::take(&mut cur))
                    }
                    b"t" => in_t = false,
                    _ => {}
                },
                Event::Text(t) if in_si && in_t => {
                    cur.push_str(&t.decode().map_err(|e| e.to_string())?)
                }
                Event::Eof => break,
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    for (i, (_, name)) in numbered(names, "xl/worksheets/sheet", ".xml")
        .into_iter()
        .enumerate()
    {
        let data = zip_read(z, &name)?;
        let mut rd = Reader::from_reader(&data[..]);
        let (mut rows, mut cells): (Vec<String>, Vec<String>) = (vec![], vec![]);
        let (mut ctype, mut val, mut in_c, mut in_v, mut in_t) =
            (String::new(), String::new(), false, false, false);
        loop {
            match rd.read_event().map_err(|e| e.to_string())? {
                Event::Start(e) => match local(e.name().as_ref()) {
                    b"c" => {
                        in_c = true;
                        val.clear();
                        ctype = e
                            .attributes()
                            .flatten()
                            .find(|a| a.key.as_ref() == b"t")
                            .map(|a| String::from_utf8_lossy(&a.value).into_owned())
                            .unwrap_or_default();
                    }
                    b"v" => in_v = true,
                    b"t" => in_t = true,
                    _ => {}
                },
                Event::End(e) => match local(e.name().as_ref()) {
                    b"c" => {
                        in_c = false;
                        let v = if ctype == "s" {
                            val.trim()
                                .parse::<usize>()
                                .ok()
                                .and_then(|n| shared.get(n).cloned())
                                .unwrap_or_else(|| val.clone())
                        } else {
                            val.clone()
                        };
                        if !v.is_empty() {
                            cells.push(v)
                        }
                    }
                    b"v" => in_v = false,
                    b"t" => in_t = false,
                    b"row" => {
                        if !cells.is_empty() {
                            rows.push(cells.join(" | "))
                        }
                        cells.clear()
                    }
                    _ => {}
                },
                Event::Text(t) if in_c && (in_v || in_t) => {
                    val.push_str(&t.decode().map_err(|e| e.to_string())?)
                }
                Event::Eof => break,
                _ => {}
            }
        }
        out.push((format!("sheet {}", i + 1), rows.join("\n")));
    }
    Ok(out)
}

// ------------------------------------------------------------------ OCR

pub fn which(tool: &str) -> bool {
    let exts: &[&str] = if cfg!(windows) {
        &[".exe", ".cmd", ".bat", ""]
    } else {
        &[""]
    };
    std::env::var_os("PATH")
        .map(|p| {
            std::env::split_paths(&p)
                .any(|d| exts.iter().any(|e| d.join(format!("{tool}{e}")).is_file()))
        })
        .unwrap_or(false)
}

pub fn ocr_available() -> bool {
    which("tesseract")
}

fn tesseract_lang(want: &str) -> String {
    let have = Command::new("tesseract")
        .arg("--list-langs")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let ok: Vec<&str> = want
        .split('+')
        .filter(|l| have.split_whitespace().any(|h| h == *l))
        .collect();
    if ok.is_empty() {
        "eng".into()
    } else {
        ok.join("+")
    }
}

/// tesseract on one image. `pdf` = true returns the bytes of a searchable PDF, else UTF-8 text.
pub fn ocr_image(path: &Path, pdf: bool) -> Res<Vec<u8>> {
    if !ocr_available() {
        return Err("OCR needs tesseract (apt install tesseract-ocr tesseract-ocr-ron)".into());
    }
    let mut c = Command::new("tesseract");
    c.arg(path)
        .arg("stdout")
        .arg("-l")
        .arg(tesseract_lang("ron+eng"));
    if pdf {
        c.arg("pdf");
    }
    let o = c.output().map_err(|e| e.to_string())?;
    if !o.status.success() {
        return Err(format!(
            "tesseract failed: {}",
            String::from_utf8_lossy(&o.stderr)
                .chars()
                .take(200)
                .collect::<String>()
        ));
    }
    Ok(o.stdout)
}

fn ocr_pdf_page(path: &Path, n: usize) -> String {
    if !(ocr_available() && which("pdftoppm")) {
        return String::new();
    }
    let Ok(d) = tempdir() else {
        return String::new();
    };
    let base = d.join("p");
    let _ = Command::new("pdftoppm")
        .args([
            "-r",
            "200",
            "-f",
            &n.to_string(),
            "-l",
            &n.to_string(),
            "-png",
        ])
        .arg(path)
        .arg(&base)
        .output();
    let png = std::fs::read_dir(&d).ok().and_then(|r| {
        r.flatten()
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|e| e == "png"))
    });
    let text = png
        .and_then(|p| ocr_image(&p, false).ok())
        .map(|b| decode(&b))
        .unwrap_or_default();
    let _ = std::fs::remove_dir_all(&d);
    text
}

fn tempdir() -> std::io::Result<std::path::PathBuf> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "cartacolibri-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p)?;
    Ok(p)
}

fn pdf_pages_raw(path: &Path) -> Res<Vec<String>> {
    let p = path.to_path_buf();
    let native = std::panic::catch_unwind(move || pdf_extract::extract_text_by_pages(&p));
    if let Ok(Ok(pages)) = &native
        && pages.iter().any(|t| !t.trim().is_empty())
    {
        return Ok(pages.clone());
    }
    if which("pdftotext") {
        let o = Command::new("pdftotext")
            .arg("-layout")
            .arg(path)
            .arg("-")
            .output()
            .map_err(|e| e.to_string())?;
        if o.status.success() {
            let t = decode(&o.stdout);
            return Ok(t
                .split('\x0c')
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .collect());
        }
        return Err(format!(
            "pdftotext failed: {}",
            String::from_utf8_lossy(&o.stderr)
                .chars()
                .take(200)
                .collect::<String>()
        ));
    }
    // Native reader found nothing (a scan?) and no poppler: report the pages as empty so OCR can try.
    match native {
        Ok(Ok(pages)) => Ok(pages),
        _ => Err(
            "cannot read this PDF (corrupt or unsupported; installing poppler-utils may help)"
                .into(),
        ),
    }
}

fn pdf_pages(path: &Path, ocr: bool) -> Res<Parts> {
    let pages = pdf_pages_raw(path)?;
    Ok(pages
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            let t = if ocr && t.trim().chars().count() < 20 {
                let o = ocr_pdf_page(path, i + 1);
                if o.is_empty() { t } else { o }
            } else {
                t
            };
            (format!("p.{}", i + 1), t)
        })
        .collect())
}

// ------------------------------------------------------------------ entry point

/// Turn a low-level reader error into a sentence a non-technical user can act on.
pub fn plain_reason(ext: &str, msg: &str) -> String {
    let m = msg.to_lowercase();
    let kind = ext.trim_start_matches('.').to_uppercase();
    let s = if m.contains("permission denied") || m.contains("access is denied") {
        "No permission to read this file. Check its access rights, or copy it somewhere readable."
            .to_string()
    } else if m.contains("no such file") || m.contains("cannot find the") {
        "The file vanished, or is a broken shortcut/link, while it was being indexed.".to_string()
    } else if m.contains("file name too long") || m.contains("filename too long") {
        "The file name or its folder path is too long for the system to open. Shorten the name."
            .to_string()
    } else if m.contains("looks like a binary") {
        msg.to_string()
    } else if m.contains("not a valid zip") || m.contains("zip") || m.contains("central directory")
    {
        format!(
            "This {kind} file looks damaged, or is not really a {kind} file (its container cannot be opened)."
        )
    } else if m.contains("xml") {
        format!("The text inside this {kind} file is damaged and cannot be read.")
    } else if m.contains("cannot read this pdf") || m.contains("pdftotext failed") {
        "This PDF is damaged, password-protected or unsupported (a scan? turn OCR on, or install poppler-utils)."
            .to_string()
    } else if m.contains("unpacks to more than") {
        "This file contains a part that is far too large to read safely; skipped.".to_string()
    } else if m.contains("panicked") {
        format!("This {kind} file is damaged in a way the reader could not handle; skipped.")
    } else {
        format!("Could not read this file: {msg}")
    };
    s.chars().take(300).collect()
}

/// Like [`extract`] but a reader panic on a corrupt file becomes an ordinary error.
pub fn extract_safe(path: &Path, ext: &str, ocr: bool) -> Res<Parts> {
    let p = path.to_path_buf();
    let x = ext.to_string();
    match std::panic::catch_unwind(move || extract(&p, &x, ocr)) {
        Ok(r) => r.map_err(|m| plain_reason(ext, &m)),
        Err(_) => Err(plain_reason(ext, "reader panicked")),
    }
}

pub fn extract(path: &Path, ext: &str, ocr: bool) -> Res<Parts> {
    match ext {
        ".pdf" => return pdf_pages(path, ocr),
        ".xlsx" => return xlsx_text(&mut open_zip(path)?),
        ".zip" => return zip_parts(&mut open_zip(path)?),
        ".epub" => {
            let mut z = open_zip(path)?;
            let mut names: Vec<String> = z
                .file_names()
                .filter(|n| {
                    let l = n.to_lowercase();
                    l.ends_with(".xhtml") || l.ends_with(".html") || l.ends_with(".htm")
                })
                .map(String::from)
                .collect();
            names.sort();
            let mut parts = Vec::new();
            for n in names {
                let t = html_to_text(&decode(&zip_read(&mut z, &n)?));
                let stem = Path::new(&n)
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                parts.push((stem, t));
            }
            return Ok(parts);
        }
        ".docx" => {
            let mut z = open_zip(path)?;
            return Ok(vec![(
                String::new(),
                xml_text(
                    &zip_read(&mut z, "word/document.xml")?,
                    &["t"],
                    &["p", "br"],
                )?,
            )]);
        }
        ".odt" => {
            let mut z = open_zip(path)?;
            return Ok(vec![(
                String::new(),
                xml_text(
                    &zip_read(&mut z, "content.xml")?,
                    &[],
                    &["p", "h", "line-break"],
                )?,
            )]);
        }
        ".pptx" => {
            let mut z = open_zip(path)?;
            let names: Vec<String> = z.file_names().map(String::from).collect();
            let mut out = Vec::new();
            for (i, (_, n)) in numbered(names, "ppt/slides/slide", ".xml")
                .into_iter()
                .enumerate()
            {
                out.push((
                    format!("slide {}", i + 1),
                    xml_text(&zip_read(&mut z, &n)?, &["t"], &["p"])?,
                ));
            }
            return Ok(out);
        }
        e if IMG_EXT.contains(&e) => {
            return Ok(vec![("OCR".into(), decode(&ocr_image(path, false)?))]);
        }
        _ => {}
    }
    let raw = std::fs::read(path).map_err(|e| e.to_string())?;
    let head = &raw[..raw.len().min(4096)];
    if !head.starts_with(&[0xff, 0xfe])
        && !head.starts_with(&[0xfe, 0xff])
        && head.iter().filter(|&&b| b == 0).count() * 10 > head.len().max(1)
    {
        return Err(
            "This file looks like a binary file renamed with a text extension, not text.".into(),
        );
    }
    if let Some(r) = text_parts(ext, &raw) {
        return r;
    }
    let mut text = decode(&raw);
    match ext {
        ".html" | ".htm" => text = html_to_text(&text),
        ".rtf" => {
            static A: OnceLock<Regex> = OnceLock::new();
            static B: OnceLock<Regex> = OnceLock::new();
            text = re(&A, r"\\'([0-9a-fA-F]{2})")
                .replace_all(&text, |c: &regex::Captures| {
                    let byte = u8::from_str_radix(&c[1], 16).unwrap_or(b'?');
                    encoding_rs::WINDOWS_1250.decode(&[byte]).0.into_owned()
                })
                .into_owned();
            text = re(&B, r"\\[a-z]+-?\d* ?|[{}]")
                .replace_all(&text, "")
                .into_owned();
        }
        _ => {}
    }
    Ok(vec![(String::new(), text)])
}

/// Split into ~`size`-byte chunks on paragraph / line / space boundaries (never inside a char).
pub fn chunk(text: &str, size: usize) -> Vec<String> {
    static A: OnceLock<Regex> = OnceLock::new();
    static B: OnceLock<Regex> = OnceLock::new();
    let t = text.replace('\r', "");
    let t = re(&A, r"[ \t]+").replace_all(&t, " ");
    let t = re(&B, r"\n{3,}").replace_all(&t, "\n\n");
    let (mut out, mut cur) = (Vec::<String>::new(), String::new());
    for line in t.trim().lines() {
        let mut para = line.trim();
        if para.is_empty() {
            continue;
        }
        while para.len() > size {
            let mut cut = size;
            while !para.is_char_boundary(cut) {
                cut -= 1;
            }
            let cut = para[..cut].rfind(' ').filter(|&p| p > 0).unwrap_or(cut);
            out.push(para[..cut].to_string());
            para = para[cut..].trim_start();
        }
        if !cur.is_empty() && cur.len() + para.len() + 1 > size {
            out.push(std::mem::take(&mut cur));
            cur = para.to_string();
        } else if cur.is_empty() {
            cur = para.to_string();
        } else {
            cur.push('\n');
            cur.push_str(para);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

// ------------------------------------------------------------------ wave 2: more formats

/// Hard ceiling on the text any one of the wave-2 readers produces (characters).
const MAX_OUT: usize = 32 * 1024 * 1024;
/// Zip listing guards: members listed, nesting depth, total unpacked bytes read.
const ZIP_MAX_MEMBERS: usize = 5000;
const ZIP_MAX_DEPTH: usize = 3;
const ZIP_MAX_TOTAL: u64 = 256 * 1024 * 1024;
const ZIP_MAX_MEMBER: u64 = 20 * 1024 * 1024;

/// Extensions the zip reader opens as plain text members.
const ZIP_TEXT_EXT: &[&str] = &[
    ".txt",
    ".md",
    ".markdown",
    ".log",
    ".yaml",
    ".yml",
    ".org",
    ".rst",
    ".adoc",
    ".html",
    ".htm",
    ".csv",
    ".tsv",
    ".json",
    ".jsonl",
    ".ipynb",
    ".eml",
    ".mbox",
    ".ics",
    ".vcf",
    ".srt",
    ".vtt",
    ".tex",
];

fn cap(mut s: String) -> String {
    if s.len() > MAX_OUT {
        let mut c = MAX_OUT;
        while !s.is_char_boundary(c) {
            c -= 1;
        }
        s.truncate(c);
    }
    s
}

/// Readers for the wave-2 text formats. `None` = not one of them (caller falls back to plain text).
pub fn text_parts(ext: &str, raw: &[u8]) -> Option<Res<Parts>> {
    let one = |t: String| Some(Ok(vec![(String::new(), cap(t))]));
    match ext {
        ".eml" => one(eml_text(&decode(raw))),
        ".mbox" => Some(Ok(mbox_parts(&decode(raw)))),
        ".ics" => one(ics_text(&decode(raw))),
        ".vcf" => one(vcf_text(&decode(raw))),
        ".csv" => one(table_text(&decode(raw), b',')),
        ".tsv" => one(table_text(&decode(raw), b'\t')),
        ".json" => {
            let t = decode(raw);
            match serde_json::from_str::<serde_json::Value>(&t) {
                Ok(v) => {
                    let mut out = String::new();
                    flatten_json(&v, "", 0, &mut out);
                    one(out)
                }
                Err(_) => one(t),
            }
        }
        ".jsonl" | ".ndjson" => Some(Ok(jsonl_parts(&decode(raw)))),
        ".srt" | ".vtt" => one(subtitle_text(&decode(raw))),
        ".ipynb" => {
            let t = decode(raw);
            match notebook_text(&t) {
                Some(x) => one(x),
                None => one(t),
            }
        }
        ".tex" => one(tex_text(&decode(raw))),
        _ => None,
    }
}

// ---- base64 / quoted-printable / RFC 2047

pub(crate) fn b64_decode(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let (mut acc, mut bits) = (0u32, 0u32);
    for b in s.bytes() {
        let v = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => continue,
        };
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    out
}

fn hexv(b: u8) -> Option<u8> {
    (b as char).to_digit(16).map(|d| d as u8)
}

fn qp_decode(s: &str, header: bool) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'=' if i + 1 < b.len() && (b[i + 1] == b'\n' || b[i + 1] == b'\r') => {
                i += 2;
                if b[i - 1] == b'\r' && i < b.len() && b[i] == b'\n' {
                    i += 1;
                }
                continue;
            }
            b'=' if i + 2 < b.len() && hexv(b[i + 1]).is_some() && hexv(b[i + 2]).is_some() => {
                out.push(hexv(b[i + 1]).unwrap() * 16 + hexv(b[i + 2]).unwrap());
                i += 3;
                continue;
            }
            b'_' if header => out.push(b' '),
            c => out.push(c),
        }
        i += 1;
    }
    out
}

fn charset_decode(bytes: &[u8], charset: &str) -> String {
    match encoding_rs::Encoding::for_label(charset.trim().as_bytes()) {
        Some(enc) => enc.decode(bytes).0.into_owned(),
        None => decode(bytes),
    }
}

fn mime_words(s: &str) -> String {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"=\?([^?\s]+)\?([bBqQ])\?([^?\s]*)\?=")
        .replace_all(s, |c: &regex::Captures| {
            let bytes = if c[2].eq_ignore_ascii_case("b") {
                b64_decode(&c[3])
            } else {
                qp_decode(&c[3], true)
            };
            charset_decode(&bytes, &c[1])
        })
        .into_owned()
}

// ---- e-mail

type Headers = Vec<(String, String)>;

fn split_message(s: &str) -> (Headers, String) {
    let s = s.replace("\r\n", "\n");
    let (head, body) = match s.find("\n\n") {
        Some(i) => (&s[..i], &s[i + 2..]),
        None => (&s[..], ""),
    };
    let mut hs: Headers = Vec::new();
    for line in head.lines() {
        if line.starts_with([' ', '\t']) {
            if let Some(last) = hs.last_mut() {
                last.1.push(' ');
                last.1.push_str(line.trim());
            }
        } else if let Some((k, v)) = line.split_once(':') {
            hs.push((k.trim().to_lowercase(), v.trim().to_string()));
        }
    }
    (hs, body.to_string())
}

fn hget<'a>(h: &'a Headers, k: &str) -> &'a str {
    h.iter()
        .find(|(n, _)| n == k)
        .map(|(_, v)| v.as_str())
        .unwrap_or("")
}

fn header_param(v: &str, name: &str) -> Option<String> {
    let low = v.to_lowercase();
    let key = format!("{name}=");
    let mut from = 0;
    while let Some(p) = low[from..].find(&key) {
        let at = from + p;
        let ok = at == 0 || matches!(low.as_bytes()[at - 1], b';' | b' ' | b'\t' | b'*');
        from = at + key.len();
        if !ok {
            continue;
        }
        let rest = v.get(from..)?.trim_start();
        return Some(if let Some(r) = rest.strip_prefix('"') {
            r.split('"').next().unwrap_or("").to_string()
        } else {
            rest.split(';').next().unwrap_or("").trim().to_string()
        });
    }
    None
}

#[derive(Default)]
struct Mime {
    plain: String,
    html: String,
    attachments: Vec<String>,
}

fn mime_walk(h: &Headers, body: &str, depth: usize, m: &mut Mime) {
    let ctype = hget(h, "content-type");
    let lc = ctype.to_lowercase();
    if lc.starts_with("multipart/")
        && depth < 8
        && let Some(b) = header_param(ctype, "boundary").filter(|b| !b.is_empty())
    {
        {
            let delim = format!("--{b}");
            let mut parts = body.split(&delim);
            parts.next(); // preamble
            for p in parts.take(2000) {
                if p.starts_with("--") {
                    break;
                }
                let p = p.strip_prefix('\n').unwrap_or(p);
                let (ph, pb) = split_message(p);
                mime_walk(&ph, &pb, depth + 1, m);
            }
            return;
        }
    }
    let disp = hget(h, "content-disposition");
    let fname = header_param(disp, "filename").or_else(|| header_param(ctype, "name"));
    let is_text = lc.is_empty() || lc.starts_with("text/plain") || lc.starts_with("text/html");
    if disp.to_lowercase().starts_with("attachment") || (fname.is_some() && !is_text) {
        m.attachments.push(mime_words(
            &fname.unwrap_or_else(|| "(unnamed attachment)".into()),
        ));
        return;
    }
    if !is_text {
        return;
    }
    let bytes = match hget(h, "content-transfer-encoding").to_lowercase().as_str() {
        "base64" => b64_decode(body),
        "quoted-printable" => qp_decode(body, false),
        _ => body.as_bytes().to_vec(),
    };
    let text = charset_decode(&bytes, &header_param(ctype, "charset").unwrap_or_default());
    if lc.starts_with("text/html") {
        m.html.push_str(&html_to_text(&text));
        m.html.push('\n');
    } else {
        m.plain.push_str(&text);
        m.plain.push('\n');
    }
}

fn eml_text(raw: &str) -> String {
    let (h, body) = split_message(raw);
    let mut m = Mime::default();
    mime_walk(&h, &body, 0, &mut m);
    let mut out = String::new();
    for (label, key) in [
        ("From", "from"),
        ("To", "to"),
        ("Cc", "cc"),
        ("Date", "date"),
        ("Subject", "subject"),
    ] {
        let v = hget(&h, key);
        if !v.is_empty() {
            out.push_str(&format!("{label}: {}\n", mime_words(v)));
        }
    }
    out.push('\n');
    out.push_str(if m.plain.trim().is_empty() {
        m.html.trim()
    } else {
        m.plain.trim()
    });
    if !m.attachments.is_empty() {
        out.push_str("\n\nAttachments:\n");
        for a in &m.attachments {
            out.push_str(&format!("- {a}\n"));
        }
    }
    out
}

fn mbox_parts(raw: &str) -> Parts {
    let mut msgs: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut prev_blank = true;
    for line in raw.lines() {
        if prev_blank && line.starts_with("From ") {
            if !cur.trim().is_empty() {
                msgs.push(std::mem::take(&mut cur));
            }
            cur.clear();
            if msgs.len() >= 100_000 {
                break;
            }
        } else {
            cur.push_str(line);
            cur.push('\n');
        }
        prev_blank = line.trim().is_empty();
    }
    if !cur.trim().is_empty() {
        msgs.push(cur);
    }
    msgs.iter()
        .enumerate()
        .map(|(i, m)| (format!("message {}", i + 1), eml_text(m)))
        .collect()
}

// ---- calendar / contacts

fn unfold(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in s.replace("\r\n", "\n").split('\n') {
        if line.starts_with([' ', '\t'])
            && let Some(l) = out.last_mut()
        {
            l.push_str(&line[1..]);
            continue;
        }
        out.push(line.to_string());
    }
    out
}

fn prop(line: &str) -> Option<(String, String)> {
    let (k, v) = line.split_once(':')?;
    let name = k.split(';').next()?.trim().to_uppercase();
    let v = v
        .replace("\\n", "\n")
        .replace("\\N", "\n")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\");
    Some((name, v))
}

fn ics_date(v: &str) -> String {
    let b: Vec<char> = v.chars().collect();
    if b.len() >= 8 && b[..8].iter().all(|c| c.is_ascii_digit()) {
        let d: String = b[..8].iter().collect();
        let mut o = format!("{}-{}-{}", &d[..4], &d[4..6], &d[6..8]);
        if b.len() >= 13 && b[8] == 'T' {
            let t: String = b[9..13].iter().collect();
            o.push_str(&format!(" {}:{}", &t[..2], &t[2..]));
        }
        o
    } else {
        v.to_string()
    }
}

fn ics_text(raw: &str) -> String {
    let mut out = String::new();
    let mut block: Option<Vec<(String, String)>> = None;
    for l in unfold(raw) {
        let up = l.trim().to_uppercase();
        if up == "BEGIN:VEVENT" || up == "BEGIN:VTODO" || up == "BEGIN:VJOURNAL" {
            block = Some(Vec::new());
        } else if up == "END:VEVENT" || up == "END:VTODO" || up == "END:VJOURNAL" {
            if let Some(b) = block.take() {
                let g = |k: &str| {
                    b.iter()
                        .find(|(n, _)| n == k)
                        .map(|(_, v)| v.trim().to_string())
                        .unwrap_or_default()
                };
                out.push_str(&format!("Event: {}\n", g("SUMMARY")));
                let (s, e) = (ics_date(&g("DTSTART")), ics_date(&g("DTEND")));
                if !s.is_empty() {
                    out.push_str(&format!(
                        "When: {s}{}\n",
                        if e.is_empty() {
                            String::new()
                        } else {
                            format!(" - {e}")
                        }
                    ));
                }
                for (lbl, k) in [
                    ("Where", "LOCATION"),
                    ("Organizer", "ORGANIZER"),
                    ("Description", "DESCRIPTION"),
                ] {
                    if !g(k).is_empty() {
                        out.push_str(&format!("{lbl}: {}\n", g(k)));
                    }
                }
                out.push('\n');
            }
        } else if let (Some(b), Some(p)) = (block.as_mut(), prop(&l)) {
            if b.len() < 200 {
                b.push(p);
            }
        } else if let Some((n, v)) = prop(&l)
            && n == "X-WR-CALNAME"
        {
            out.push_str(&format!("Calendar: {v}\n\n"));
        }
    }
    out
}

fn vcf_text(raw: &str) -> String {
    let mut out = String::new();
    let mut card: Option<Vec<(String, String)>> = None;
    for l in unfold(raw) {
        let up = l.trim().to_uppercase();
        if up == "BEGIN:VCARD" {
            card = Some(Vec::new());
        } else if up == "END:VCARD" {
            if let Some(c) = card.take() {
                for (n, v) in &c {
                    let label = match n.as_str() {
                        "FN" => "Name",
                        "ORG" => "Organization",
                        "TITLE" => "Title",
                        "TEL" => "Phone",
                        "EMAIL" => "Email",
                        "ADR" => "Address",
                        "URL" => "Web",
                        "NOTE" => "Note",
                        "BDAY" => "Birthday",
                        _ => continue,
                    };
                    let v = v.replace(';', " ");
                    out.push_str(&format!("{label}: {}\n", v.trim()));
                }
                out.push('\n');
            }
        } else if let (Some(c), Some(p)) = (card.as_mut(), prop(&l))
            && c.len() < 200
        {
            c.push(p);
        }
    }
    out
}

// ---- tables / json

/// Minimal RFC 4180 reader (quotes, doubled quotes, newlines inside quotes).
pub fn csv_rows(text: &str, delim: u8) -> Vec<Vec<String>> {
    let d = delim as char;
    let mut rows = Vec::new();
    let (mut row, mut field) = (Vec::<String>::new(), String::new());
    let (mut inq, mut had) = (false, false);
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        if inq {
            if c == '"' {
                if it.peek() == Some(&'"') {
                    field.push('"');
                    it.next();
                } else {
                    inq = false;
                }
            } else {
                field.push(c);
            }
        } else if c == '"' && field.is_empty() {
            inq = true;
            had = true;
        } else if c == d {
            row.push(std::mem::take(&mut field));
            had = true;
        } else if c == '\n' || c == '\r' {
            if c == '\r' && it.peek() == Some(&'\n') {
                it.next();
            }
            if had || !field.is_empty() || !row.is_empty() {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            had = false;
            if rows.len() >= 2_000_000 {
                return rows;
            }
        } else {
            field.push(c);
            had = true;
        }
    }
    if had || !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

fn table_text(text: &str, delim: u8) -> String {
    let rows = csv_rows(text, delim);
    let Some((head, body)) = rows.split_first() else {
        return String::new();
    };
    // Header-aware only when the first row looks like labels (non-empty, no pure numbers).
    let header_ok = !head.is_empty()
        && head
            .iter()
            .all(|h| !h.trim().is_empty() && h.trim().parse::<f64>().is_err());
    let mut out = String::new();
    let mut emit = |cells: &[String], names: Option<&Vec<String>>| {
        let line: Vec<String> = cells
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.trim().is_empty())
            .map(|(i, v)| match names.and_then(|n| n.get(i)) {
                Some(n) => format!("{}: {}", n.trim(), v.trim()),
                None => v.trim().to_string(),
            })
            .collect();
        if !line.is_empty() {
            out.push_str(&line.join("; "));
            out.push('\n');
        }
    };
    if header_ok {
        for r in body {
            emit(r, Some(head));
        }
    } else {
        for r in &rows {
            emit(r, None);
        }
    }
    out
}

fn flatten_json(v: &serde_json::Value, path: &str, depth: usize, out: &mut String) {
    use serde_json::Value;
    if out.len() > MAX_OUT || depth > 64 {
        return;
    }
    let leaf = |out: &mut String, s: String| {
        if path.is_empty() {
            out.push_str(&s);
        } else {
            out.push_str(&format!("{path}: {s}"));
        }
        out.push('\n');
    };
    match v {
        Value::Null => {}
        Value::Bool(b) => leaf(out, b.to_string()),
        Value::Number(n) => leaf(out, n.to_string()),
        Value::String(s) => leaf(out, s.clone()),
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                flatten_json(x, &format!("{path}[{i}]"), depth + 1, out);
            }
        }
        Value::Object(m) => {
            for (k, x) in m {
                let p = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                flatten_json(x, &p, depth + 1, out);
            }
        }
    }
}

fn jsonl_parts(text: &str) -> Parts {
    let mut parts = Vec::new();
    for (i, line) in text.lines().enumerate().take(2_000_000) {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        let t = match serde_json::from_str::<serde_json::Value>(l) {
            Ok(v) => {
                let mut o = String::new();
                flatten_json(&v, "", 0, &mut o);
                o
            }
            Err(_) => l.to_string(),
        };
        parts.push((format!("line {}", i + 1), t));
    }
    // Merge to bounded-size parts so the index is not flooded with tiny rows.
    let mut merged: Parts = Vec::new();
    for (loc, t) in parts {
        match merged.last_mut() {
            Some((_, cur)) if cur.len() + t.len() < 4000 => {
                cur.push('\n');
                cur.push_str(&t);
            }
            _ => merged.push((loc, t)),
        }
    }
    merged
}

fn subtitle_text(s: &str) -> String {
    static TAG: OnceLock<Regex> = OnceLock::new();
    let tag = re(&TAG, r"<[^>]*>|\{\\[^}]*\}");
    let mut out: Vec<String> = Vec::new();
    for l in s.replace('\r', "").lines() {
        let t = l.trim().trim_start_matches('\u{feff}');
        if t.is_empty()
            || t.contains("-->")
            || t.starts_with("WEBVTT")
            || t.starts_with("NOTE")
            || t.starts_with("STYLE")
            || t.chars().all(|c| c.is_ascii_digit())
        {
            continue;
        }
        let t = tag.replace_all(t, "").trim().to_string();
        if !t.is_empty() && out.last() != Some(&t) {
            out.push(t);
        }
    }
    out.join("\n")
}

fn notebook_text(s: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(s).ok()?;
    let src = |c: &serde_json::Value| match &c["source"] {
        serde_json::Value::String(t) => t.clone(),
        serde_json::Value::Array(a) => a.iter().filter_map(|x| x.as_str()).collect::<String>(),
        _ => String::new(),
    };
    let cells = v
        .get("cells")
        .or_else(|| v.pointer("/worksheets/0/cells"))?
        .as_array()?;
    let mut out = String::new();
    for c in cells.iter().take(100_000) {
        let t = src(c);
        match c["cell_type"].as_str() {
            Some("markdown") => out.push_str(&format!("{t}\n\n")),
            Some("code") => out.push_str(&format!("```\n{t}\n```\n\n")),
            _ => {}
        }
        if out.len() > MAX_OUT {
            break;
        }
    }
    Some(out)
}

fn tex_text(s: &str) -> String {
    static CMT: OnceLock<Regex> = OnceLock::new();
    static SEC: OnceLock<Regex> = OnceLock::new();
    static ENV: OnceLock<Regex> = OnceLock::new();
    static CMD: OnceLock<Regex> = OnceLock::new();
    let t = re(&CMT, r"(?m)(^|[^\\])%.*$").replace_all(s, "$1");
    let t = re(
        &SEC,
        r"\\(?:part|chapter|section|subsection|subsubsection|paragraph|title)\*?\{([^{}]*)\}",
    )
    .replace_all(&t, "\n# $1\n");
    let t = re(&ENV, r"\\(?:begin|end)\{[^{}]*\}").replace_all(&t, "\n");
    let t = re(&CMD, r"\\[a-zA-Z@]+\*?(\[[^\]]*\])?").replace_all(&t, "");
    t.replace("\\\\", "\n")
        .replace(['{', '}'], "")
        .replace('~', " ")
}

// ---- zip listing

fn zip_parts(z: &mut zip::ZipArchive<std::fs::File>) -> Res<Parts> {
    let mut parts = Parts::new();
    let mut budget = ZIP_MAX_TOTAL;
    zip_walk(z, "", 0, &mut budget, &mut parts)?;
    Ok(parts)
}

fn zip_walk<R: Read + std::io::Seek>(
    z: &mut zip::ZipArchive<R>,
    prefix: &str,
    depth: usize,
    budget: &mut u64,
    parts: &mut Parts,
) -> Res<()> {
    let n = z.len().min(ZIP_MAX_MEMBERS);
    let mut listing = String::new();
    let mut todo: Vec<(usize, String, u64)> = Vec::new();
    for i in 0..n {
        let Ok(f) = z.by_index(i) else { continue };
        if f.is_dir() {
            continue;
        }
        let name = f.name().to_string();
        listing.push_str(&format!("{prefix}{name} ({} bytes)\n", f.size()));
        todo.push((i, name, f.size()));
    }
    if z.len() > n {
        listing.push_str(&format!("... and {} more entries\n", z.len() - n));
    }
    parts.push((
        format!("{}listing", prefix),
        format!("Files in archive:\n{listing}"),
    ));
    for (i, name, size) in todo {
        let lower = name.to_lowercase();
        let ext = lower.rfind('.').map(|p| &lower[p..]).unwrap_or("");
        let nested = ext == ".zip" && depth + 1 < ZIP_MAX_DEPTH;
        if !(nested || ZIP_TEXT_EXT.contains(&ext)) || size > ZIP_MAX_MEMBER || size > *budget {
            continue;
        }
        let mut buf = Vec::new();
        let Ok(f) = z.by_index(i) else { continue };
        if f.take(ZIP_MAX_MEMBER + 1).read_to_end(&mut buf).is_err()
            || buf.len() as u64 > ZIP_MAX_MEMBER
        {
            continue;
        }
        *budget = budget.saturating_sub(buf.len() as u64);
        let p = format!("{prefix}{name}");
        if nested {
            if let Ok(mut inner) = zip::ZipArchive::new(std::io::Cursor::new(buf)) {
                let _ = zip_walk(&mut inner, &format!("{p}!/"), depth + 1, budget, parts);
            }
            continue;
        }
        let r = match text_parts(ext, &buf) {
            Some(r) => r.ok(),
            None => {
                let t = decode(&buf);
                Some(vec![(
                    String::new(),
                    if ext == ".html" || ext == ".htm" {
                        html_to_text(&t)
                    } else {
                        t
                    },
                )])
            }
        };
        for (loc, t) in r.unwrap_or_default() {
            if !t.trim().is_empty() {
                parts.push((
                    if loc.is_empty() {
                        p.clone()
                    } else {
                        format!("{p} {loc}")
                    },
                    t,
                ));
            }
        }
    }
    Ok(())
}
