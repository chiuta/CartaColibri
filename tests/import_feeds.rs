use cartacolibri::extract::{self, extract};
use cartacolibri::feeds::{self, Subscription};
use cartacolibri::import;
use cartacolibri::library::Library;
use std::io::Write;
use std::path::Path;

fn text_of(dir: &Path, name: &str, body: &[u8]) -> String {
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    let ext = format!(".{}", name.rsplit('.').next().unwrap());
    extract(&p, &ext, false)
        .unwrap()
        .into_iter()
        .map(|(l, t)| format!("[{l}] {t}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn md_files(d: &Path) -> Vec<String> {
    let mut v = Vec::new();
    fn go(d: &Path, v: &mut Vec<String>) {
        if let Ok(rd) = std::fs::read_dir(d) {
            for e in rd.flatten() {
                if e.path().is_dir() {
                    go(&e.path(), v);
                } else {
                    v.push(std::fs::read_to_string(e.path()).unwrap_or_default());
                }
            }
        }
    }
    go(d, &mut v);
    v
}

#[test]
fn eml_plain_and_multipart() {
    let t = tempfile::tempdir().unwrap();
    let eml = "From: =?utf-8?B?SsOpcsO0bWU=?= <j@x.org>\r\nTo: a@b.c\r\nSubject: Hello =?utf-8?Q?caf=C3=A9?=\r\nContent-Type: multipart/mixed; boundary=\"BB\"\r\n\r\n--BB\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\nBody =C8=99 line\r\n--BB\r\nContent-Type: application/pdf; name=\"report.pdf\"\r\nContent-Disposition: attachment; filename=\"report.pdf\"\r\nContent-Transfer-Encoding: base64\r\n\r\nAAAA\r\n--BB--\r\n";
    let s = text_of(t.path(), "m.eml", eml.as_bytes());
    assert!(s.contains("Jérôme"), "{s}");
    assert!(s.contains("Subject: Hello café"), "{s}");
    assert!(s.contains("Body ș line"), "{s}");
    assert!(s.contains("- report.pdf"), "{s}");
    let html = "Subject: H\n\n<p>only <b>html</b></p>"
        .replace("Subject: H", "Subject: H\nContent-Type: text/html");
    assert!(text_of(t.path(), "h.eml", html.as_bytes()).contains("only html"));
}

#[test]
fn mbox_two_messages() {
    let t = tempfile::tempdir().unwrap();
    let mb = "From a@b Mon Jan 1 00:00:00 2024\nSubject: one\n\nfirst body\n\nFrom c@d Mon Jan 1 00:00:00 2024\nSubject: two\n\nsecond body\n";
    let s = text_of(t.path(), "x.mbox", mb.as_bytes());
    assert!(s.contains("[message 1]") && s.contains("first body"));
    assert!(s.contains("[message 2]") && s.contains("second body"));
}

#[test]
fn calendar_contacts_tables_json() {
    let t = tempfile::tempdir().unwrap();
    let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Team sync\r\nDTSTART:20260105T090000Z\r\nDTEND:20260105T093000Z\r\nLOCATION:Room 4\r\nDESCRIPTION:Agenda\\nline\r\n  folded\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let s = text_of(t.path(), "c.ics", ics.as_bytes());
    assert!(
        s.contains("Event: Team sync") && s.contains("2026-01-05 09:00") && s.contains("Room 4")
    );
    assert!(s.contains("Agenda\nline"));
    let vcf = "BEGIN:VCARD\nFN:Ana Pop\nTEL;TYPE=cell:+40 700\nEMAIL:ana@x.ro\nEND:VCARD\n";
    let s = text_of(t.path(), "c.vcf", vcf.as_bytes());
    assert!(
        s.contains("Name: Ana Pop")
            && s.contains("Phone: +40 700")
            && s.contains("Email: ana@x.ro")
    );
    let s = text_of(
        t.path(),
        "d.csv",
        b"name,city\n\"Pop, Ana\",Cluj\nIon,\"Ia\nsi\"\n",
    );
    assert!(s.contains("name: Pop, Ana; city: Cluj"), "{s}");
    let s = text_of(t.path(), "d.tsv", b"a\tb\n1\t2\n");
    assert!(s.contains("a: 1; b: 2"));
    let s = text_of(t.path(), "d.json", br#"{"a":{"b":[1,"x"]},"n":null}"#);
    assert!(s.contains("a.b[0]: 1") && s.contains("a.b[1]: x"), "{s}");
    let s = text_of(
        t.path(),
        "d.jsonl",
        b"{\"k\":\"v1\"}\nnot json\n{\"k\":\"v2\"}\n",
    );
    assert!(s.contains("k: v1") && s.contains("not json") && s.contains("k: v2"));
}

#[test]
fn subtitles_notebook_tex_text() {
    let t = tempfile::tempdir().unwrap();
    let s = text_of(t.path(), "a.srt", b"1\n00:00:01,000 --> 00:00:02,000\n<i>Hello</i>\n\n2\n00:00:03,000 --> 00:00:04,000\nWorld\n");
    assert!(s.contains("Hello\nWorld") && !s.contains("-->"), "{s}");
    let s = text_of(
        t.path(),
        "a.vtt",
        b"WEBVTT\n\n00:01.000 --> 00:02.000\nHi there\n",
    );
    assert!(s.contains("Hi there") && !s.contains("WEBVTT"));
    let nb = br##"{"cells":[{"cell_type":"markdown","source":["# Title\n","text"]},{"cell_type":"code","source":"print(1)"}]}"##;
    let s = text_of(t.path(), "n.ipynb", nb);
    assert!(s.contains("# Title") && s.contains("print(1)"));
    let s = text_of(t.path(), "t.tex", b"\\section{Intro} Some \\textbf{bold} text. % comment\n\\begin{itemize}\\item one\\end{itemize}\n");
    assert!(
        s.contains("# Intro")
            && s.contains("bold text.")
            && !s.contains("comment")
            && !s.contains('\\'),
        "{s}"
    );
    for n in ["a.org", "a.rst", "a.adoc"] {
        assert!(text_of(t.path(), n, b"* heading\nbody").contains("body"));
    }
}

fn make_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let o = zip::write::SimpleFileOptions::default();
        for (n, d) in entries {
            z.start_file(*n, o).unwrap();
            z.write_all(d).unwrap();
        }
        z.finish().unwrap();
    }
    buf
}

#[test]
fn zip_listing_nested_and_bomb_guard() {
    let t = tempfile::tempdir().unwrap();
    let inner = make_zip(&[("deep.txt", b"deep text")]);
    let z = make_zip(&[
        ("a/note.md", b"hello zip"),
        ("pic.bin", b"\x00\x01"),
        ("in.zip", &inner),
    ]);
    let s = text_of(t.path(), "x.zip", &z);
    assert!(s.contains("a/note.md") && s.contains("pic.bin"), "{s}");
    assert!(s.contains("hello zip") && s.contains("deep text"), "{s}");
    // a member far bigger than the per-member cap is listed but never unpacked
    let big = vec![b'a'; 25 * 1024 * 1024];
    let z = make_zip(&[("big.txt", &big), ("ok.txt", b"fine")]);
    let s = text_of(t.path(), "b.zip", &z);
    assert!(s.contains("big.txt") && s.contains("fine") && !s.contains("aaaaaaaaaa"));
    // truncated / garbage
    let _ = extract(
        &{
            let p = t.path().join("g.zip");
            std::fs::write(&p, &z[..z.len() / 2]).unwrap();
            p
        },
        ".zip",
        false,
    );
}

#[test]
fn garbage_never_panics() {
    let t = tempfile::tempdir().unwrap();
    let mut junk: Vec<u8> = (0..4000u32).map(|i| (i * 7919 % 251) as u8).collect();
    junk.extend_from_slice(b"BEGIN:VEVENT\n=?x?B?\n[[ @article{ <h3><a href=");
    for ext in [
        "eml", "mbox", "ics", "vcf", "csv", "tsv", "json", "jsonl", "srt", "vtt", "ipynb", "tex",
        "zip",
    ] {
        let p = t.path().join(format!("j.{ext}"));
        for data in [&junk[..], &junk[..100], b"", b"{", b"\"", b"\xff\xfe\x00"] {
            std::fs::write(&p, data).unwrap();
            let _ = extract::extract_safe(&p, &format!(".{ext}"), false);
        }
    }
    // deep json
    let deep = "[".repeat(10_000);
    std::fs::write(t.path().join("d.json"), deep).unwrap();
    let _ = extract::extract_safe(&t.path().join("d.json"), ".json", false);
}

#[test]
fn import_bookmarks_per_folder() {
    let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1><DL><p>
<DT><H3>Dev</H3><DL><p>
<DT><A HREF="https://rust-lang.org" ADD_DATE="1">Rust &amp; co</A>
<DT><H3>Tools</H3><DL><p><DT><A HREF="https://x.dev">X</A></DL><p>
</DL><p>
<DT><A HREF="https://top.example">Top</A>
<DT><A HREF="javascript:alert(1)">bad</A>
</DL>"#;
    let t = tempfile::tempdir().unwrap();
    let st = import::bookmarks(html, t.path()).unwrap();
    assert_eq!(st.notes, 3);
    assert_eq!(st.skipped, 1);
    let all = md_files(t.path()).join("\n");
    assert!(
        all.contains("# Bookmarks: Dev\n") && all.contains("[Rust & co](https://rust-lang.org)")
    );
    assert!(all.contains("Dev / Tools") && all.contains("[Top](https://top.example)"));
    let _ = import::bookmarks("<DL><H3>x<A href=", t.path());
}

#[test]
fn import_vault_backlinks() {
    let s = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(s.path().join("sub")).unwrap();
    std::fs::create_dir_all(s.path().join(".obsidian")).unwrap();
    std::fs::write(s.path().join(".obsidian/x.md"), "hidden").unwrap();
    std::fs::write(s.path().join("A.md"), "See [[B]] and [[sub/C|alias]].").unwrap();
    std::fs::write(s.path().join("B.md"), "Back to [[A#h]].").unwrap();
    std::fs::write(s.path().join("sub/C.md"), "leaf").unwrap();
    let d = tempfile::tempdir().unwrap();
    let st = import::vault(s.path(), d.path()).unwrap();
    assert_eq!(st.notes, 3);
    let a = std::fs::read_to_string(d.path().join("A.md")).unwrap();
    assert!(a.contains("[[B]]") && a.contains("## Backlinks") && a.contains("- [[B]]"));
    let c = std::fs::read_to_string(d.path().join("sub/C.md")).unwrap();
    assert!(c.contains("- [[A]]"));
    assert!(!d.path().join(".obsidian").exists());
}

#[test]
fn import_bibtex_csv_enex() {
    let bib = r#"@comment{x}
@article{smith2020, title={A {Great} Paper}, author = "Smith, J. and Doe, A.", year=2020, journal={Nature}, abstract={Long {nested} abstract.}}
@book(k2, title = {Second}, publisher = {P})
@article{broken, title={unterminated"#;
    let t = tempfile::tempdir().unwrap();
    let st = import::bibtex(bib, t.path()).unwrap();
    assert!(st.notes >= 2);
    let all = md_files(t.path()).join("\n");
    assert!(
        all.contains("# A Great Paper")
            && all.contains("Authors: Smith, J. and Doe, A.")
            && all.contains("Year: 2020")
    );
    assert!(all.contains("Long nested abstract.") && all.contains("# Second"));

    let csv = "title,note,excerpt,url,folder,tags,created\nRust,\"my note\",ex,https://r.dev,Dev,\"a, b\",2024\nNo url,,,,Dev,,\n";
    let d = tempfile::tempdir().unwrap();
    let st = import::csv_bookmarks(csv, d.path()).unwrap();
    assert_eq!((st.notes, st.skipped), (1, 1));
    let all = md_files(d.path()).join("\n");
    assert!(
        all.contains("[Rust](https://r.dev)")
            && all.contains("tags: a, b")
            && all.contains("# Saved links: Dev")
    );
    let pocket = "title,url,time_added,tags,status\nP,https://p.io,1,x,unread\n";
    let d2 = tempfile::tempdir().unwrap();
    assert_eq!(import::csv_bookmarks(pocket, d2.path()).unwrap().notes, 1);
    assert!(import::csv_bookmarks("a,b\n1,2", d2.path()).is_err());

    let enex = r#"<?xml version="1.0"?><en-export><note><title>Shopping</title><content><![CDATA[<en-note><div>Milk</div><div>Eggs</div></en-note>]]></content><created>20240101T000000Z</created><tag>home</tag></note><note><title>T2</title><content><![CDATA[<en-note>two</en-note>]]></content></note></en-export>"#;
    let d3 = tempfile::tempdir().unwrap();
    let st = import::enex(enex.as_bytes(), d3.path()).unwrap();
    assert_eq!(st.notes, 2);
    let all = md_files(d3.path()).join("\n");
    assert!(all.contains("# Shopping") && all.contains("Milk") && all.contains("Tags: home"));
    // garbage
    let _ = import::enex(b"<en-export><note><title>x", d3.path());
    let _ = import::bibtex("@@{{{ @a{,,= = {", d3.path());
    let _ = import::csv_bookmarks("\"unterminated,url\n\"", d3.path());
}

const RSS: &str = r#"<?xml version="1.0"?><rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/"><channel><title>T</title>
<item><title>First &amp; best</title><link>http://h/1</link><guid>g1</guid><pubDate>Mon, 01 Jan 2024 00:00:00 GMT</pubDate><description><![CDATA[<p>Hello <b>world</b></p>]]></description></item>
<item><title>Second</title><link>http://h/2</link><description>plain</description></item>
</channel></rss>"#;
const ATOM: &str = r#"<feed xmlns="http://www.w3.org/2005/Atom"><title>A</title>
<entry><title>Entry one</title><link rel="self" href="http://x/self"/><link rel="alternate" href="http://x/1"/><id>urn:1</id><updated>2024-01-01T00:00:00Z</updated><author><name>Ann</name></author><summary>Sum</summary><content type="html">&lt;p&gt;Full text&lt;/p&gt;</content></entry></feed>"#;

#[test]
fn parse_rss_atom_and_garbage() {
    let r = feeds::parse(RSS);
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].title, "First & best");
    assert_eq!(r[0].guid, "g1");
    assert!(r[0].summary.contains("Hello world"));
    let a = feeds::parse(ATOM);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].link, "http://x/1");
    assert_eq!(a[0].author, "Ann");
    assert!(a[0].summary.contains("Full text"), "{:?}", a[0]);
    for cut in [10, 80, 200, RSS.len() / 2] {
        let _ = feeds::parse(&RSS[..cut]);
    }
    let _ = feeds::parse("<<<>>> \u{0} <item><item><item>");
    let deep = "<a>".repeat(50_000) + "<item><title>x</title></item>";
    let _ = feeds::parse(&deep);
    assert!(feeds::parse("").is_empty());
}

#[test]
fn opml_roundtrip() {
    let subs = vec![
        Subscription {
            title: "A & B".into(),
            url: "http://a/feed?x=1&y=2".into(),
        },
        Subscription {
            title: "C".into(),
            url: "https://c/rss".into(),
        },
    ];
    let xml = feeds::opml_export(&subs);
    assert_eq!(feeds::opml_parse(&xml), subs);
    let nested = r#"<opml><body><outline text="Folder"><outline text="Z" xmlUrl="http://z/f"/></outline></body></opml>"#;
    assert_eq!(feeds::opml_parse(nested)[0].url, "http://z/f");
    assert!(feeds::opml_parse("<opml><body><outline").is_empty());
}

#[test]
fn poll_local_server_dedupes() {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    std::thread::spawn(move || {
        for rq in server.incoming_requests() {
            let (code, body) = match rq.url() {
                "/rss" => (200, RSS.to_string()),
                "/big" => (200, "x".repeat(6 * 1024 * 1024)),
                _ => (404, "nope".to_string()),
            };
            let _ = rq.respond(tiny_http::Response::from_string(body).with_status_code(code));
        }
    });
    // SAFETY: only this test touches the variable, and it is set before any reader looks at it.
    unsafe { std::env::set_var("CARTACOLIBRI_GRAB_LOCAL", "1") };
    let home = tempfile::tempdir().unwrap();
    let mut lib = Library::open(Some(home.path().to_str().unwrap()), false).unwrap();
    let urls: Vec<String> = ["rss", "missing", "big"]
        .iter()
        .map(|p| format!("http://127.0.0.1:{port}/{p}"))
        .chain(["file:///etc/passwd".to_string(), "ftp://x/y".to_string()])
        .collect();
    let s = feeds::poll(home.path(), &urls, &mut lib);
    assert_eq!(
        (s.feeds_ok, s.feeds_failed, s.new_items),
        (1, 4, 2),
        "{s:?}"
    );
    assert!(home.path().join("feeds.json").is_file());
    let notes = md_files(&lib.upload_dir().unwrap().join("feeds")).join("\n");
    assert!(notes.contains("# First & best") && notes.contains("Source: http://h/1"));
    let again = feeds::poll(home.path(), &urls[..1], &mut lib);
    assert_eq!(again.new_items, 0);
    assert_eq!(again.feeds[0].items, 2);
}

// ---------------------------------------------------------------- adversarial audit

#[test]
fn vault_into_itself_never_modifies_sources() {
    let s = tempfile::tempdir().unwrap();
    std::fs::write(s.path().join("A.md"), "see [[B]]").unwrap();
    std::fs::write(s.path().join("B.md"), "see [[A]]").unwrap();
    let before = std::fs::read(s.path().join("A.md")).unwrap();
    let _ = import::vault(s.path(), s.path());
    let _ = import::vault(s.path(), &s.path().join("out"));
    let _ = import::vault(s.path(), &s.path().join("out"));
    assert_eq!(
        std::fs::read(s.path().join("A.md")).unwrap(),
        before,
        "source note was rewritten"
    );
    assert!(!s.path().join("out/out").exists());
}

#[cfg(unix)]
#[test]
fn import_never_writes_through_planted_links() {
    let d = tempfile::tempdir().unwrap();
    let victim = tempfile::tempdir().unwrap();
    let target = victim.path().join("precious.txt");
    std::fs::write(&target, "keep me").unwrap();
    // first run learns the (deterministic) note name; then plant a link under that name
    let bib = "@book{k1, title={Same}, author={X}}";
    import::bibtex(bib, d.path()).unwrap();
    let name = std::fs::read_dir(d.path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::remove_file(&name).unwrap();
    std::os::unix::fs::symlink(&target, &name).unwrap();
    let _ = import::bibtex(bib, d.path());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "keep me");
}

#[cfg(unix)]
#[test]
fn import_of_a_fifo_does_not_hang() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("x.bib");
    let c = std::ffi::CString::new(f.to_str().unwrap()).unwrap();
    unsafe extern "C" {
        fn mkfifo(p: *const std::ffi::c_char, m: u32) -> i32;
    }
    if unsafe { mkfifo(c.as_ptr(), 0o600) } != 0 {
        return;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    let out = d.path().join("out");
    std::thread::spawn(move || {
        let _ = tx.send(import::import_path(&f, &out).is_err());
    });
    assert_eq!(rx.recv_timeout(std::time::Duration::from_secs(5)), Ok(true));
}

#[test]
fn vault_unwritable_note_is_skipped_not_fatal() {
    let s = tempfile::tempdir().unwrap();
    std::fs::write(s.path().join("a.md"), "one").unwrap();
    std::fs::write(s.path().join("b.md"), "two").unwrap();
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir(d.path().join("a.md")).unwrap(); // a directory sits where a note goes
    let st = import::vault(s.path(), d.path()).unwrap();
    assert_eq!((st.notes, st.skipped), (1, 1));
}

#[test]
fn hostile_import_inputs() {
    let d = tempfile::tempdir().unwrap();
    let o = d.path();
    let _ = import::bibtex(&"@a{".repeat(300_000), o);
    let _ = import::bibtex(&format!("@a{{k, t = {}", "{".repeat(500_000)), o);
    let _ = import::bibtex("@a{k, t = \"unterminated", o);
    let _ = import::bookmarks(&"<h3>".repeat(200_000), o);
    let _ = import::bookmarks(&"<dl>".repeat(200_000), o);
    let _ = import::bookmarks(&"</dl>".repeat(200_000), o);
    let _ = import::csv_bookmarks("url\n\"", o);
    let _ = import::csv_bookmarks(&format!("url,t\n{}", "x,\"\n".repeat(100_000)), o);
    let _ = import::enex(&"<note>".repeat(100_000).into_bytes(), o);
    let _ = import::enex(
        b"<note><title>\xff\xfe</title><content>&bogus;&#0;</content></note>",
        o,
    );
    let deep = format!(
        "{}<note><title>t</title><content>hello</content></note>{}",
        "<a>".repeat(200),
        "</a>".repeat(200)
    );
    let st = import::enex(deep.as_bytes(), o).unwrap();
    assert_eq!(st.notes, 1, "deep nesting must not lose the note");
    // dangerous link schemes are not carried into notes
    let st = import::bookmarks(
        "<DL><DT><A HREF=\"data:text/html,<script>\">x</A><DT><A HREF=\" VBScript:x\">y</A></DL>",
        o,
    )
    .unwrap();
    assert_eq!(st.skipped, 2);
}
