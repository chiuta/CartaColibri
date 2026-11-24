//! Robustness tests for the readers: damaged / hostile inputs must come back as Ok or Err
//! quickly and never panic, a hung reader must not hang the indexer, and the walker must
//! never block on pipes or choke on huge folders. Everything is generated in code with a
//! seeded PRNG, so a failure reproduces exactly.

use cartacolibri::extract::{self, STUCK_LIMIT};
use cartacolibri::library::Library;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------- helpers

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn make_zip(entries: &[(&str, Vec<u8>)], deflate: bool) -> Vec<u8> {
    use zip::CompressionMethod::{Deflated, Stored};
    use zip::write::{SimpleFileOptions, ZipWriter};
    let mut w = ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let o =
        SimpleFileOptions::default().compression_method(if deflate { Deflated } else { Stored });
    for (n, d) in entries {
        w.start_file(*n, o).unwrap();
        w.write_all(d).unwrap();
    }
    w.finish().unwrap().into_inner()
}

const W_DOC: &str = r#"<?xml version="1.0"?><w:document xmlns:w="w"><w:body><w:p><w:r><w:t>Hello docx</w:t></w:r></w:p><w:p><w:r><w:t>Second</w:t></w:r></w:p></w:body></w:document>"#;

fn samples() -> Vec<(&'static str, Vec<u8>)> {
    let t = |s: &str| s.as_bytes().to_vec();
    let pdf = b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 200 200]/Contents 4 0 R/Resources<</Font<</F1 5 0 R>>>>>>endobj\n4 0 obj<</Length 44>>stream\nBT /F1 12 Tf 20 100 Td (Hello PDF) Tj ET\nendstream\nendobj\n5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>endobj\ntrailer<</Root 1 0 R/Size 6>>\n%%EOF\n".to_vec();
    vec![
        (".txt", t("plain text \u{103}\u{219} here\nline two")),
        (".md", t("# Title\n\nSome *markdown* text.\n")),
        (
            ".html",
            t("<html><body><p>Hi &amp; bye &#233;</p><script>x()</script></body></html>"),
        ),
        (".rtf", t(r"{\rtf1\ansi Hello \'e9 world\par}")),
        (".csv", t("name,age\n\"Ana, B\",30\nIon,\"4\"\"2\"\n")),
        (".tsv", t("a\tb\n1\t2\n")),
        (".json", t(r#"{"a":[1,2,{"b":"c"}],"d":null}"#)),
        (".jsonl", t("{\"a\":1}\n{\"b\":\"x\"}\nnot json\n")),
        (
            ".eml",
            t(
                "From: a@b.c\nSubject: =?UTF-8?B?w6k=?=\nContent-Type: multipart/mixed; boundary=XX\n\n--XX\nContent-Type: text/plain\n\nbody =C3=A9\n--XX\nContent-Disposition: attachment; filename=\"f.pdf\"\n\nzzz\n--XX--\n",
            ),
        ),
        (
            ".mbox",
            t("From a@b Mon\nSubject: one\n\nbody1\n\nFrom c@d Tue\nSubject: two\n\nbody2\n"),
        ),
        (
            ".ics",
            t(
                "BEGIN:VCALENDAR\nBEGIN:VEVENT\nSUMMARY:Meet\nDTSTART:20240101T101500Z\nDTEND:20240101T111500Z\nEND:VEVENT\nEND:VCALENDAR\n",
            ),
        ),
        (
            ".vcf",
            t("BEGIN:VCARD\nFN:Ana Pop\nTEL:123\nEMAIL:a@b.c\nEND:VCARD\n"),
        ),
        (
            ".srt",
            t(
                "1\n00:00:01,000 --> 00:00:02,000\n<i>Hello</i>\n\n2\n00:00:03,000 --> 00:00:04,000\nBye\n",
            ),
        ),
        (".vtt", t("WEBVTT\n\n00:01.000 --> 00:02.000\nHello\n")),
        (
            ".ipynb",
            t(
                r##"{"cells":[{"cell_type":"markdown","source":["# T"]},{"cell_type":"code","source":"print(1)"}]}"##,
            ),
        ),
        (
            ".tex",
            t("\\section{Intro} Hello % c\n\\begin{document}x\\\\y\\end{document}"),
        ),
        (".pdf", pdf),
        (".docx", make_zip(&[("word/document.xml", t(W_DOC))], true)),
        (
            ".odt",
            make_zip(
                &[(
                    "content.xml",
                    t(
                        "<office:document-content xmlns:office=\"o\" xmlns:text=\"t\"><text:p>Hi odt</text:p><text:h>Head</text:h></office:document-content>",
                    ),
                )],
                true,
            ),
        ),
        (
            ".pptx",
            make_zip(
                &[
                    (
                        "ppt/slides/slide1.xml",
                        t("<p:sld><a:p><a:r><a:t>Slide one</a:t></a:r></a:p></p:sld>"),
                    ),
                    (
                        "ppt/slides/slide2.xml",
                        t("<p:sld><a:p><a:r><a:t>Two</a:t></a:r></a:p></p:sld>"),
                    ),
                ],
                false,
            ),
        ),
        (
            ".xlsx",
            make_zip(
                &[
                    (
                        "xl/sharedStrings.xml",
                        t("<sst><si><t>Hello</t></si><si><t>World</t></si></sst>"),
                    ),
                    (
                        "xl/worksheets/sheet1.xml",
                        t(
                            "<worksheet><sheetData><row><c t=\"s\"><v>0</v></c><c><v>42</v></c></row></sheetData></worksheet>",
                        ),
                    ),
                ],
                true,
            ),
        ),
        (
            ".epub",
            make_zip(
                &[
                    (
                        "OEBPS/c1.xhtml",
                        t("<html><body><p>Chapter one</p></body></html>"),
                    ),
                    ("mimetype", t("application/epub+zip")),
                ],
                true,
            ),
        ),
        (
            ".zip",
            make_zip(
                &[
                    ("a.txt", t("inside a")),
                    ("d/b.csv", t("x,y\n1,2\n")),
                    ("n.zip", make_zip(&[("c.md", t("# deep"))], false)),
                ],
                true,
            ),
        ),
    ]
}

fn write_tmp(dir: &Path, ext: &str, data: &[u8]) -> PathBuf {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let p = dir.join(format!(
        "f{}{ext}",
        N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::write(&p, data).unwrap();
    p
}

/// Extract must return (Ok or Err) within 5 s and must not have panicked inside.
fn check(dir: &Path, ext: &str, data: &[u8], what: &str) {
    let p = write_tmp(dir, ext, data);
    let t = Instant::now();
    let r = extract::extract(&p, ext, false);
    let dt = t.elapsed();
    let _ = std::fs::remove_file(&p);
    assert!(dt < Duration::from_secs(5), "{ext} {what}: took {dt:?}");
    if let Err(m) = r {
        assert!(!m.contains("panicked"), "{ext} {what}: reader panicked");
    }
}

// ---------------------------------------------------------------- fuzz-ish

#[test]
fn valid_samples_read_ok() {
    let d = tempfile::tempdir().unwrap();
    for (ext, data) in samples() {
        let p = write_tmp(d.path(), ext, &data);
        let r = extract::extract(&p, ext, false);
        assert!(r.is_ok(), "{ext}: {r:?}");
    }
}

#[test]
fn truncation_at_every_7th_offset() {
    let d = tempfile::tempdir().unwrap();
    for (ext, data) in samples() {
        for cut in (0..data.len()).step_by(7) {
            check(d.path(), ext, &data[..cut], &format!("truncated at {cut}"));
        }
    }
}

#[test]
fn flipped_and_spliced_bytes() {
    let d = tempfile::tempdir().unwrap();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for (ext, data) in samples() {
        for round in 0..40 {
            let mut m = data.clone();
            for _ in 0..1 + rng.below(8) {
                let i = rng.below(m.len());
                match rng.below(4) {
                    0 => m[i] = rng.next() as u8,
                    1 => m[i] ^= 1 << rng.below(8),
                    2 => {
                        m.insert(i, rng.next() as u8);
                    }
                    _ => {
                        m.remove(i);
                    }
                }
            }
            check(d.path(), ext, &m, &format!("mutation round {round}"));
        }
    }
}

#[test]
fn random_garbage_for_every_extension() {
    let d = tempfile::tempdir().unwrap();
    let mut rng = Rng(42);
    for (ext, _) in samples() {
        for len in [0usize, 1, 2, 3, 17, 255, 4096] {
            let junk: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
            check(d.path(), ext, &junk, &format!("garbage {len}"));
        }
    }
}

#[test]
fn hostile_markup() {
    let d = tempfile::tempdir().unwrap();
    let n = 300_000;
    // html: lone '<' runs, a '<' at the very end after a multibyte char, huge attribute, repeats
    for h in [
        "<".repeat(2_000_000),
        "caf\u{e9}<\u{e9}".to_string(),
        "<\u{e9}".to_string(),
        format!("<a href=\"{}\">x</a>", "A".repeat(20_000_000)),
        "<p>".repeat(3_000_000),
        "&".repeat(1_000_000),
        "<!--".repeat(500_000),
        "<script>".repeat(500_000),
    ] {
        check(d.path(), ".html", h.as_bytes(), "hostile html");
    }
    // xml-based containers: deep nesting, repeated tags, giant attribute, entity floods
    let deep = format!("<w:document>{}{}", "<a>".repeat(n), "</a>".repeat(n));
    let rep = format!(
        "<w:document>{}</w:document>",
        "<w:p><w:t>x</w:t></w:p>".repeat(n)
    );
    let attr = format!(
        "<w:document><w:p w:v=\"{}\"><w:t>x</w:t></w:p></w:document>",
        "B".repeat(30_000_000)
    );
    let ents = format!(
        "<w:document><w:t>{}</w:t></w:document>",
        "&#x1F600;&bogus;&#99999999999;".repeat(200_000)
    );
    for (what, x) in [
        ("deep", deep),
        ("repeat", rep),
        ("attr", attr),
        ("entities", ents),
    ] {
        for (ext, member) in [
            (".docx", "word/document.xml"),
            (".odt", "content.xml"),
            (".pptx", "ppt/slides/slide1.xml"),
        ] {
            let z = make_zip(&[(member, x.clone().into_bytes())], true);
            check(d.path(), ext, &z, what);
        }
    }
    // text formats with extreme shapes
    let big_cols = vec!["a"; 3_000_000].join(",");
    let unterminated = format!("\"{}", "q,\n".repeat(10_000_000));
    check(d.path(), ".csv", big_cols.as_bytes(), "3M columns");
    check(
        d.path(),
        ".csv",
        unterminated.as_bytes(),
        "unterminated quote",
    );
    check(
        d.path(),
        ".csv",
        "a\n".repeat(3_000_000).as_bytes(),
        "3M rows",
    );
    check(
        d.path(),
        ".json",
        "[".repeat(1_000_000).as_bytes(),
        "deep json",
    );
    check(
        d.path(),
        ".json",
        format!("[{}0]", "0,".repeat(5_000_000)).as_bytes(),
        "wide json",
    );
    check(
        d.path(),
        ".jsonl",
        "{}\n".repeat(3_000_000).as_bytes(),
        "3M jsonl lines",
    );
    check(
        d.path(),
        ".ipynb",
        format!("{{\"cells\":[{}]}}", "{},".repeat(500_000) + "{}").as_bytes(),
        "many cells",
    );
    let nest = format!(
        "Content-Type: multipart/mixed; boundary=B\n\n{}",
        "--B\nContent-Type: multipart/mixed; boundary=B\n\n".repeat(50_000)
    );
    check(d.path(), ".eml", nest.as_bytes(), "nested multipart");
    check(
        d.path(),
        ".mbox",
        "From x\n\nm\n".repeat(300_000).as_bytes(),
        "many messages",
    );
    check(
        d.path(),
        ".ics",
        "BEGIN:VEVENT\nDTSTART:20240101T\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\nEND:VEVENT\n".as_bytes(),
        "non-ascii date",
    );
    check(
        d.path(),
        ".ics",
        "BEGIN:VEVENT\nX:y\n".repeat(200_000).as_bytes(),
        "unclosed events",
    );
    check(
        d.path(),
        ".tex",
        "\\section{".repeat(500_000).as_bytes(),
        "tex braces",
    );
    check(
        d.path(),
        ".tex",
        "%".repeat(3_000_000).as_bytes(),
        "tex comments",
    );
    check(
        d.path(),
        ".rtf",
        "{\\b ".repeat(1_000_000).as_bytes(),
        "rtf groups",
    );
    check(
        d.path(),
        ".srt",
        "-->\n<".repeat(500_000).as_bytes(),
        "srt tags",
    );
}

#[test]
fn zip_bombs_and_oversized_directories() {
    let d = tempfile::tempdir().unwrap();
    // ratio bomb: > 50 MB of zeros squeezed to a few tens of KB
    let bomb = vec![0u8; 56 * 1024 * 1024];
    let z = make_zip(&[("word/document.xml", bomb.clone())], true);
    assert!(z.len() < 1024 * 1024);
    for ext in [".docx", ".zip", ".epub", ".xlsx", ".pptx", ".odt"] {
        let p = write_tmp(d.path(), ext, &z);
        let t = Instant::now();
        let r = extract::extract(&p, ext, false);
        assert!(t.elapsed() < Duration::from_secs(5), "{ext} bomb slow");
        assert!(r.is_err(), "{ext}: ratio bomb should be refused");
        let plain = extract::plain_reason(ext, &r.unwrap_err());
        assert!(plain.contains("too large or complex"), "{plain}");
    }
    // nested: bomb inside zip inside zip
    let inner = make_zip(&[("big.txt", bomb)], true);
    let outer = make_zip(&[("in.zip", inner)], false);
    check(d.path(), ".zip", &outer, "nested ratio bomb");
    // 1M-entry central directory claims (EOCD says 65535 entries; zip64 says 1,000,000)
    let mut claim = make_zip(&[("a.txt", b"x".to_vec())], false);
    let eocd = claim.len() - 22;
    claim[eocd + 8..eocd + 10].copy_from_slice(&u16::MAX.to_le_bytes());
    claim[eocd + 10..eocd + 12].copy_from_slice(&u16::MAX.to_le_bytes());
    check(d.path(), ".zip", &claim, "65535-entry claim");
    let mut z64 = claim.clone();
    z64.extend_from_slice(&[0x50, 0x4b, 0x06, 0x06]); // stray zip64 record signature
    z64.extend_from_slice(&1_000_000u64.to_le_bytes().repeat(8));
    check(d.path(), ".docx", &z64, "zip64 claim");
    // a real archive with more entries than the safe limit
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let o =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for i in 0..100_500u32 {
        w.start_file(format!("f{i}"), o).unwrap();
    }
    let many = w.finish().unwrap().into_inner();
    let p = write_tmp(d.path(), ".zip", &many);
    let r = extract::extract(&p, ".zip", false);
    assert!(r.is_err(), "100k+ entries should be refused");
}

#[test]
fn huge_text_is_truncated_with_a_note() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("big.txt");
    let mut f = std::fs::File::create(&p).unwrap();
    let line = "lorem ipsum dolor sit amet ".repeat(40) + "\n";
    for _ in 0..(60 * 1024 * 1024 / line.len() + 1) {
        f.write_all(line.as_bytes()).unwrap();
    }
    drop(f);
    let parts = extract::extract_safe(&p, ".txt", false).unwrap();
    let total: usize = parts.iter().map(|(_, t)| t.len()).sum();
    assert!(total < 52 * 1024 * 1024, "kept {total} bytes");
    assert!(parts.last().unwrap().1.contains("only the first"));
}

#[test]
fn non_regular_paths_are_refused() {
    for ext in [".txt", ".pdf", ".zip", ".docx"] {
        let r = extract::extract(Path::new("/dev/zero"), ext, false);
        assert!(r.is_err(), "{ext} must not read /dev/zero");
    }
}

// ---------------------------------------------------------------- timeout path

fn slow_env() {
    // SAFETY: set once, before any reader that consults it runs; other tests only read.
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        std::env::set_var("CARTACOLIBRI_TEST_SLOW_MS", "20000");
    });
}

#[test]
fn hung_reader_is_abandoned_and_the_next_file_still_works() {
    slow_env();
    let d = tempfile::tempdir().unwrap();
    let slow = d.path().join("__slow__a.tex");
    std::fs::write(&slow, "x").unwrap();
    let ok = d.path().join("fine.tex");
    std::fs::write(&ok, "\\section{Fine} text").unwrap();
    let t = Instant::now();
    let r = extract::extract_safe_with(&slow, ".tex", false, Duration::from_millis(300));
    assert!(t.elapsed() < Duration::from_secs(3));
    let msg = r.unwrap_err();
    assert!(msg.contains("took too long"), "{msg}");
    let r = extract::extract_safe_with(&ok, ".tex", false, Duration::from_secs(10)).unwrap();
    assert!(r[0].1.contains("Fine"));
    // more stuck readers of the same kind: after STUCK_LIMIT, new ones are refused outright
    for i in 1..STUCK_LIMIT {
        let s = d.path().join(format!("__slow__{i}.tex"));
        std::fs::write(&s, "x").unwrap();
        let r = extract::extract_safe_with(&s, ".tex", false, Duration::from_millis(200));
        assert!(r.unwrap_err().contains("took too long"));
    }
    let t = Instant::now();
    let r = extract::extract_safe_with(&ok, ".tex", false, Duration::from_secs(10));
    assert!(t.elapsed() < Duration::from_secs(1));
    assert!(r.unwrap_err().contains("got stuck"));
    // other kinds are unaffected
    let md = d.path().join("n.md");
    std::fs::write(&md, "# hi").unwrap();
    assert!(extract::extract_safe_with(&md, ".md", false, Duration::from_secs(10)).is_ok());
}

#[test]
fn indexer_continues_past_a_hung_file_and_reports_it() {
    slow_env();
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("__slow__b.md"), "never read").unwrap();
    std::fs::write(docs.join("good.md"), "# Good\n\nzebra content").unwrap();
    std::fs::write(docs.join("zzz.md"), "last file").unwrap();
    // SAFETY: only this test sets the timeout, before the indexer starts.
    unsafe { std::env::set_var("CARTACOLIBRI_EXTRACT_TIMEOUT", "1") };
    let home = t.path().join("home");
    let mut lib = Library::open(Some(&home.to_string_lossy()), false).unwrap();
    let start = Instant::now();
    let st = lib
        .update(Some(&[docs.to_string_lossy().into_owned()]), &mut |_| {})
        .unwrap();
    unsafe { std::env::remove_var("CARTACOLIBRI_EXTRACT_TIMEOUT") };
    assert!(start.elapsed() < Duration::from_secs(15), "indexer hung");
    assert_eq!(st.added, 2, "{st:?}");
    assert_eq!(st.failed, 1, "{st:?}");
    let probs = lib.problems();
    assert!(
        probs
            .iter()
            .any(|p| p["reason"].as_str().unwrap_or("").contains("took too long")),
        "{probs:?}"
    );
}

// ---------------------------------------------------------------- library walker

fn run_with_deadline<T: Send + 'static>(secs: u64, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(Duration::from_secs(secs))
        .expect("operation hung")
}

#[cfg(unix)]
#[test]
fn fifos_and_devices_never_block_the_indexer() {
    use std::os::unix::fs::symlink;
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("real.txt"), "mango text").unwrap();
    let made = std::process::Command::new("mkfifo")
        .arg(docs.join("pipe.txt"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if made {
        symlink(docs.join("pipe.txt"), docs.join("linkpipe.md")).unwrap();
    }
    let _ = symlink("/dev/zero", docs.join("zero.txt"));
    let _ = std::os::unix::net::UnixListener::bind(docs.join("sock.txt"));
    let home = t.path().join("home").to_string_lossy().into_owned();
    let root = docs.to_string_lossy().into_owned();
    let (st, _t) = run_with_deadline(20, move || {
        let mut lib = Library::open(Some(&home), false).unwrap();
        (lib.update(Some(&[root]), &mut |_| {}).unwrap(), t)
    });
    assert_eq!(st.added, 1, "{st:?}");
}

#[test]
fn oversized_single_file_is_skipped_with_a_reason() {
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    let f = std::fs::File::create(docs.join("huge.txt")).unwrap();
    f.set_len(200 * 1024 * 1024).unwrap(); // sparse
    std::fs::write(docs.join("ok.txt"), "small").unwrap();
    let mut lib = Library::open(Some(&t.path().join("h").to_string_lossy()), false).unwrap();
    let st = lib
        .update(Some(&[docs.to_string_lossy().into_owned()]), &mut |_| {})
        .unwrap();
    assert_eq!(st.added, 1, "{st:?}");
    assert!(
        lib.problems()
            .iter()
            .any(|p| p["reason"].as_str().unwrap_or("").contains("very large"))
    );
}

#[test]
fn twenty_thousand_files_walk_quickly() {
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    for s in 0..20 {
        let sub = docs.join(format!("d{s}"));
        std::fs::create_dir_all(&sub).unwrap();
        for i in 0..1000 {
            std::fs::File::create(sub.join(format!("e{i}.xyz"))).unwrap();
        }
    }
    let mut lib = Library::open(Some(&t.path().join("h").to_string_lossy()), false).unwrap();
    let start = Instant::now();
    let st = lib
        .update(Some(&[docs.to_string_lossy().into_owned()]), &mut |_| {})
        .unwrap();
    let dt = start.elapsed();
    assert_eq!(st.seen, 20_000, "{st:?}");
    assert!(dt < Duration::from_secs(20), "walk took {dt:?}");
}
