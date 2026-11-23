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

pub fn extract(path: &Path, ext: &str, ocr: bool) -> Res<Parts> {
    match ext {
        ".pdf" => return pdf_pages(path, ocr),
        ".xlsx" => return xlsx_text(&mut open_zip(path)?),
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
