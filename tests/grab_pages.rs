//! `grab::save_page` (self-contained .html and .mhtml) against a local fixture site, plus
//! hostile-MHTML cases for the reader. The fixture lives on loopback, so the tests that fetch it
//! turn local grabbing on; they share one lock because that switch is process-wide.

use cartacolibri::extract;
use cartacolibri::grab;
use cartacolibri::library::{Filter, Library};
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static LOCAL: Mutex<()> = Mutex::new(());

const PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0x0d, 0x49, 0x48, 0x44, 0x52, 0, 0, 0,
    1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 0x1f, 0x15, 0xc4, 0x89, 0, 0, 0, 0x0d, 0x49, 0x44, 0x41, 0x54,
    0x78, 0xda, 0x63, 0xfc, 0xff, 0xff, 0x3f, 0, 5, 0xfe, 2, 0xfe, 0xa7, 0xd5, 0xd5, 0xd0, 0, 0, 0,
    0, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

type Routes = fn(&str, u16) -> Option<(&'static str, Vec<u8>)>;

fn serve(routes: Routes) -> u16 {
    let srv = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = srv.server_addr().to_ip().unwrap().port();
    std::thread::spawn(move || {
        for r in srv.incoming_requests() {
            let url = r.url().to_string();
            let path = url.split('?').next().unwrap_or("").to_string();
            let resp = match routes(&path, port) {
                Some((ct, body)) => tiny_http::Response::from_data(body)
                    .with_header(tiny_http::Header::from_bytes("Content-Type", ct).unwrap())
                    .boxed(),
                None => tiny_http::Response::from_string("nope")
                    .with_status_code(404)
                    .boxed(),
            };
            let _ = r.respond(resp);
        }
    });
    port
}

/// Second origin on loopback: holds an image the page pulls in across origins.
fn other(path: &str, _p: u16) -> Option<(&'static str, Vec<u8>)> {
    (path == "/secret.png").then(|| ("image/png", PNG.to_vec()))
}

static OTHER_PORT: Mutex<u16> = Mutex::new(0);

fn site(path: &str, port: u16) -> Option<(&'static str, Vec<u8>)> {
    let o = *OTHER_PORT.lock().unwrap();
    match path {
        "/dir/page.html" => Some((
            "text/html; charset=utf-8",
            format!(
                r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Zebrafish Report</title>
<link rel="stylesheet" href="../style.css"><base href="http://127.0.0.1:{port}/">
<script src="/app.js"></script><script>document.write("EVILINLINE")</script></head>
<body onload="evil()"><h1>Narvalul albastru</h1><p>Un paragraf despre zebrafish si alte vietati.</p>
<img src="/img.png" alt="ok"><img src="http://127.0.0.1:{o}/secret.png" alt="cross">
<a href="/next" onclick="x()">next</a><a href="javascript:alert(1)">bad</a>
<iframe src="/frame.html"></iframe><noscript><img src="/pixel.png"></noscript>
<div style="background:url(/img.png)">x</div></body></html>"#
            )
            .into_bytes(),
        )),
        "/style.css" => Some((
            "text/css",
            b"@import url(/more.css); body{background:url('/img.png') ; color:#123} .x{color:red}"
                .to_vec(),
        )),
        "/more.css" => Some(("text/css", b".more{font-weight:bold}".to_vec())),
        "/img.png" => Some(("image/png", PNG.to_vec())),
        "/app.js" => Some(("application/javascript", b"alert('EVILFILE')".to_vec())),
        _ => None,
    }
}

fn lib() -> (tempfile::TempDir, Library) {
    let t = tempfile::tempdir().unwrap();
    let home = t.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let l = Library::open(Some(home.to_str().unwrap()), false).unwrap();
    (t, l)
}

fn hits(l: &Library, q: &str) -> usize {
    l.search(q, &Filter::default()).unwrap().len()
}

fn fixture() -> String {
    let op = serve(other);
    *OTHER_PORT.lock().unwrap() = op;
    let p = serve(site);
    format!("http://127.0.0.1:{p}/dir/page.html")
}

#[test]
fn html_is_self_contained_safe_and_searchable() {
    let _g = LOCAL.lock().unwrap_or_else(|e| e.into_inner());
    let url = fixture();
    let (_t, mut l) = lib();

    // private fixture is refused unless local grabbing is on
    grab::allow_local(false);
    assert!(grab::save_page(&mut l, &url, "html").is_err());
    grab::allow_local(true);

    assert!(grab::save_page(&mut l, &url, "pdf").is_err());
    let r = grab::save_page(&mut l, &url, "html").unwrap();
    let path = r["path"].as_str().unwrap().to_string();
    assert!(path.contains("uploads") && path.contains("web-grab") && path.ends_with(".html"));
    assert_eq!(r["title"], "Zebrafish Report");
    let out = std::fs::read_to_string(&path).unwrap();
    assert_eq!(r["bytes"].as_u64().unwrap() as usize, out.len());
    let low = out.to_lowercase();
    assert!(
        !low.contains("<script")
            && !low.contains("evil")
            && !low.contains("onload")
            && !low.contains("onclick")
    );
    assert!(!low.contains("javascript:") && !low.contains("<iframe") && !low.contains("pixel.png"));
    assert!(out.contains("data:image/png;base64,"), "image embedded");
    assert!(
        out.contains("color:#123") && out.contains(".more{font-weight:bold}"),
        "css + @import inlined"
    );
    assert!(
        !out.contains("img.png") && !out.contains("style.css"),
        "no live resource refs left"
    );
    // the cross-origin loopback image is not embedded or linked
    assert!(!out.contains("secret.png"));
    assert!(r["blocked_private"].as_u64().unwrap() >= 1);
    // links stay usable (absolute)
    assert!(out.contains(&format!(
        "href=\"{}\"",
        url.replace("/dir/page.html", "/next")
    )));
    assert!(hits(&l, "zebrafish") >= 1 && hits(&l, "Narvalul") >= 1);
    assert_eq!(hits(&l, "EVILINLINE"), 0);

    // same page again overwrites, never piles up copies
    let r2 = grab::save_page(&mut l, &url, "html").unwrap();
    assert_eq!(r2["path"], r["path"]);
    grab::allow_local(false);
}

#[test]
fn mhtml_has_parts_and_is_searchable() {
    let _g = LOCAL.lock().unwrap_or_else(|e| e.into_inner());
    let url = fixture();
    let (_t, mut l) = lib();
    grab::allow_local(true);
    let r = grab::save_page(&mut l, &url, "mhtml").unwrap();
    grab::allow_local(false);
    let path = r["path"].as_str().unwrap();
    assert!(path.ends_with(".mhtml"));
    let out = std::fs::read(path).unwrap();
    let s = String::from_utf8_lossy(&out);
    assert!(s.starts_with("From: <Saved by CartaColibri web-grab>"));
    assert!(s.contains("multipart/related") && s.contains("Content-Location: http://127.0.0.1"));
    assert!(
        s.contains("Content-Type: image/png") && s.contains("Content-Transfer-Encoding: base64")
    );
    assert!(s.contains("Content-Type: text/css") && s.contains("quoted-printable"));
    let low = s.to_lowercase();
    assert!(!low.contains("<script") && !low.contains("evil") && !low.contains("secret.png"));
    assert_eq!(
        s.matches("Content-Type: text/css").count(),
        2,
        "{}",
        s.lines()
            .filter(|l| l.starts_with("Content-Location"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(hits(&l, "zebrafish") >= 1);
    assert!(hits(&l, "Narvalul") >= 1);
    // the title travels as the subject and the reader keeps it
    let parts = extract::extract(Path::new(path), ".mhtml", false).unwrap();
    let text: String = parts.iter().map(|p| p.1.clone()).collect();
    assert!(text.contains("Zebrafish Report") && text.contains("Narvalul albastru"));
    assert!(!text.contains("font-weight") && !text.contains("EVIL"));
}

// ---------------------------------------------------------------- reader robustness

fn mht_text(raw: &[u8]) -> String {
    extract::text_parts(".mhtml", raw)
        .unwrap()
        .unwrap()
        .into_iter()
        .map(|p| p.1)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn mhtml_reader_decodes_charsets_and_encodings() {
    let m = "MIME-Version: 1.0\r\nSubject: =?utf-8?B?Q2FydGUgxIN0YQ==?=\r\nContent-Type: multipart/related; boundary=\"BB\"\r\n\r\n--BB\r\nContent-Type: text/html; charset=\"iso-8859-2\"\r\nContent-Transfer-Encoding: quoted-printable\r\nContent-Location: http://x/\r\n\r\n<html><head><style>.a{b:c}</style><script>var hidden=1</script></head><body><p>=F3rdei =C5=A3ar=\r\n=C4=83 fix</p></body></html>\r\n--BB\r\nContent-Type: text/html; charset=utf-8\r\nContent-Transfer-Encoding: base64\r\n\r\nPHA+Y3VyY3ViZXUgYmFzZTY0PC9wPg==\r\n--BB\r\nContent-Type: image/png\r\nContent-Transfer-Encoding: base64\r\n\r\niVBORw0KGgo=\r\n--BB--\r\n";
    let t = mht_text(m.as_bytes());
    assert!(t.contains("Carte"), "subject kept: {t}");
    assert!(t.contains("curcubeu base64"));
    assert!(!t.contains("hidden") && !t.contains(".a{b:c}") && !t.contains("iVBOR"));
}

#[test]
fn hostile_mhtml_is_bounded() {
    let _ = extract::text_parts(".mhtml", b"");
    let _ = extract::text_parts(".mht", b"\r\n\r\n");
    let start = Instant::now();
    // wrong / missing boundaries, truncated, boundary the same at every level
    let cases: Vec<Vec<u8>> = vec![
        b"Content-Type: multipart/related; boundary=\"\"\r\n\r\n--\r\n----\r\n".to_vec(),
        b"Content-Type: multipart/related\r\n\r\n--x\r\nhello".to_vec(),
        b"Content-Type: multipart/related; boundary=Z\r\n\r\n--Z\r\nContent-Type: text/html\r\n\r\n<p>no close".to_vec(),
        b"Content-Type: multipart/related; boundary=Z\r\n\r\n--Zextra\r\n\r\n<p>bad</p>\r\n--Z--".to_vec(),
        vec![0xff; 100_000],
        b"Content-Type: text/html; charset=nonsense\r\nContent-Transfer-Encoding: base64\r\n\r\n!!!!====".to_vec(),
    ];
    for c in &cases {
        let _ = mht_text(c);
    }
    // deep nesting with the same boundary at every level
    let mut deep = String::new();
    for _ in 0..2000 {
        deep.push_str("Content-Type: multipart/mixed; boundary=\"Q\"\r\n\r\n--Q\r\n");
    }
    deep.push_str("Content-Type: text/html\r\n\r\n<p>bottom</p>\r\n--Q--\r\n");
    let _ = mht_text(deep.as_bytes());
    // thousands of tiny parts
    let mut many = String::from("Content-Type: multipart/related; boundary=\"M\"\r\n\r\n");
    for i in 0..30_000 {
        many.push_str(&format!(
            "--M\r\nContent-Type: text/plain\r\n\r\npart {i}\r\n"
        ));
    }
    many.push_str("--M--\r\n");
    let t = mht_text(many.as_bytes());
    assert!(
        t.contains("part 0") && !t.contains("part 29999"),
        "part count is capped"
    );
    // one huge base64 body and one huge line
    let big = format!(
        "Content-Type: text/html\r\nContent-Transfer-Encoding: base64\r\n\r\n{}",
        "QUJD".repeat(5_000_000)
    );
    assert!(mht_text(big.as_bytes()).len() <= 40 * 1024 * 1024);
    let line = format!("Content-Type: text/plain\r\n\r\n{}", "a".repeat(20_000_000));
    let _ = mht_text(line.as_bytes());
    assert!(
        start.elapsed() < Duration::from_secs(60),
        "took {:?}",
        start.elapsed()
    );
}

#[test]
fn html_reader_keeps_title_and_honours_charset() {
    let mut raw = b"<html><head><meta charset=\"iso-8859-2\"><title>Cartea ".to_vec();
    raw.push(0xbf); // z with dot above in latin-2
    raw.extend_from_slice(
        b"</title><style>x{y:z}</style></head><body><script>hid()</script>vizibil</body></html>",
    );
    let parts = extract::text_parts(".html", &raw).unwrap().unwrap();
    let t = &parts[0].1;
    assert!(t.contains("Cartea \u{17c}") && t.contains("vizibil"), "{t}");
    assert!(!t.contains("hid()") && !t.contains("x{y:z}"));
}
