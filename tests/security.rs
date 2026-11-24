//! Black-box security tests: they start the real UI server (`server::serve`) on a loopback port
//! and talk to it over raw TCP, so the Host / Origin / header checks are exercised exactly as a
//! browser (or a hostile web page) would hit them. Known gaps are `#[ignore]`d and documented in
//! SECURITY.md; run them with `cargo test --test security -- --ignored` to see where they stand.

use cartacolibri::server;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

struct Srv {
    _t: tempfile::TempDir,
    port: u16,
    home: std::path::PathBuf,
}

fn start() -> Srv {
    let t = tempfile::tempdir().unwrap();
    let home = t.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let h = home.to_string_lossy().into_owned();
    let addr = format!("127.0.0.1:{port}");
    std::thread::spawn(move || {
        let _ = server::serve(&addr, Some(h), "http://127.0.0.1:9".into());
    });
    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Srv { _t: t, port, home }
}

struct Resp {
    code: u16,
    head: String,
    body: String,
}

/// Send one raw HTTP/1.1 request. `host`: the Host header; `headers`: extra header lines.
fn req(
    s: &Srv,
    method: &str,
    path: &str,
    host: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> Resp {
    let mut c = TcpStream::connect(("127.0.0.1", s.port)).unwrap();
    c.set_read_timeout(Some(Duration::from_secs(20))).unwrap();
    let mut r = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n");
    for (k, v) in headers {
        r.push_str(&format!("{k}: {v}\r\n"));
    }
    r.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    c.write_all(r.as_bytes()).unwrap();
    let _ = c.write_all(body);
    let mut raw = Vec::new();
    let _ = c.read_to_end(&mut raw);
    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let code = head
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    Resp {
        code,
        head: head.to_string(),
        body: body.to_string(),
    }
}

fn local(s: &Srv) -> String {
    format!("127.0.0.1:{}", s.port)
}

/// A well-formed UI call: loopback Host, same-origin Origin, X-Colibri-UI header.
fn ui(s: &Srv, method: &str, path: &str, extra: &[(&str, &str)], body: &[u8]) -> Resp {
    let host = local(s);
    let origin = format!("http://{host}");
    let mut h = vec![("X-Colibri-UI", "1"), ("Origin", origin.as_str())];
    h.extend_from_slice(extra);
    req(s, method, path, &host, &h, body)
}

#[test]
fn sanity_ui_call_works() {
    let s = start();
    let r = ui(&s, "GET", "/carta/status", &[], b"");
    assert_eq!(r.code, 200, "{}", r.body);
    let r = req(&s, "GET", "/", &local(&s), &[], b"");
    assert_eq!(r.code, 200);
}

#[test]
fn bad_host_is_rejected() {
    let s = start();
    for host in [
        "evil.example",
        "evil.example:8765",
        "127.0.0.1.evil.example",
        "localhost.evil.example",
        "192.168.1.5:8765",
        "0.0.0.0:8765",
        "",
    ] {
        for path in ["/", "/health", "/carta/status"] {
            let r = req(&s, "GET", path, host, &[("X-Colibri-UI", "1")], b"");
            assert_eq!(r.code, 403, "Host {host:?} on {path} gave {}", r.code);
        }
    }
    // DNS-rebinding shape: the browser sends the attacker's name as Host
    let r = req(
        &s,
        "POST",
        "/carta/folders",
        "rebind.example",
        &[("X-Colibri-UI", "1")],
        b"{\"path\":\"/\"}",
    );
    assert_eq!(r.code, 403);
}

#[test]
fn bad_origin_is_rejected() {
    let s = start();
    let host = local(&s);
    for origin in [
        "http://evil.example",
        "https://evil.example",
        "null",
        "http://localhost:1",
        "http://127.0.0.1:1",
        "file://",
    ] {
        let r = req(
            &s,
            "POST",
            "/carta/folders",
            &host,
            &[("X-Colibri-UI", "1"), ("Origin", origin)],
            b"{\"path\":\"/tmp\"}",
        );
        assert_eq!(r.code, 403, "Origin {origin:?} gave {}", r.code);
    }
}

#[test]
fn carta_routes_need_the_ui_header() {
    let s = start();
    let host = local(&s);
    for (m, p) in [
        ("GET", "/carta/library"),
        ("GET", "/carta/search?q=a"),
        ("GET", "/carta/doc?id=1"),
        ("POST", "/carta/folders"),
        ("DELETE", "/carta/folders"),
        ("POST", "/carta/upload"),
        ("POST", "/carta/grab"),
        ("POST", "/carta/run"),
        ("POST", "/carta/index"),
    ] {
        let r = req(&s, m, p, &host, &[("Content-Type", "text/plain")], b"{}");
        assert_eq!(r.code, 403, "{m} {p} without X-Colibri-UI gave {}", r.code);
    }
    // a cross-site preflight is never granted CORS headers on /carta/*
    let r = req(
        &s,
        "OPTIONS",
        "/carta/library",
        &host,
        &[
            ("Origin", "http://evil.example"),
            ("Access-Control-Request-Headers", "x-colibri-ui"),
        ],
        b"",
    );
    assert!(
        !r.head
            .to_lowercase()
            .contains("access-control-allow-origin"),
        "{}",
        r.head
    );
}

#[test]
fn path_traversal_attempts_are_rejected() {
    let s = start();
    let host = local(&s);
    for p in [
        "/../../../../etc/passwd",
        "/%2e%2e/%2e%2e/etc/passwd",
        "/%2e%2e%2f%2e%2e%2fetc%2fpasswd",
        "//etc/passwd",
        "/etc/passwd",
        "/carta.html/../../etc/passwd",
        "/..%5c..%5cwindows/win.ini",
    ] {
        let r = req(&s, "GET", p, &host, &[], b"");
        assert!(!r.body.contains("root:"), "{p} leaked a file");
        assert!(matches!(r.code, 400 | 404), "{p} gave {}", r.code);
    }
    for p in [
        "/carta/../library",
        "/carta/..%2f..%2fetc/passwd",
        "/carta/doc?id=../../etc/passwd",
        "/carta/doc?id=%2Fetc%2Fpasswd",
        "/carta/doc?path=/etc/passwd",
    ] {
        let r = ui(&s, "GET", p, &[], b"");
        assert!(!r.body.contains("root:"), "{p} leaked a file");
        assert!(matches!(r.code, 400 | 404), "{p} gave {}", r.code);
    }
}

#[test]
fn upload_filenames_cannot_escape_the_uploads_folder() {
    let s = start();
    let uploads = s.home.join("library").join("uploads");
    for name in [
        "../../escape.txt",
        "..%2F..%2Fescape.txt",
        "%2e%2e%2f%2e%2e%2fescape.txt",
        "/tmp/cc-security-escape.txt",
        "..\\..\\escape.txt",
        "....//escape.txt",
        "..",
        ".",
        "",
    ] {
        let r = ui(
            &s,
            "POST",
            "/carta/upload",
            &[
                ("X-File-Name", name),
                ("Content-Type", "application/octet-stream"),
            ],
            b"Hello security test document with a few words.",
        );
        assert!(
            matches!(r.code, 200 | 400),
            "{name:?} gave {}: {}",
            r.code,
            r.body
        );
        if r.code == 200 {
            let v: serde_json::Value = serde_json::from_str(&r.body).unwrap();
            let saved = std::path::PathBuf::from(v["saved"].as_str().unwrap());
            assert!(saved.starts_with(&uploads), "{name:?} saved to {saved:?}");
        }
    }
    assert!(!s.home.join("escape.txt").exists());
    assert!(!s.home.parent().unwrap().join("escape.txt").exists());
    assert!(!std::path::Path::new("/tmp/cc-security-escape.txt").exists());
}

#[test]
fn clip_route_requires_the_token() {
    let s = start();
    let host = local(&s);
    let body = br#"{"url":"https://example.invalid/a","title":"t","text":"A clipped passage that is long enough to be saved."}"#;
    // no token, wrong token, token in a header instead of the query, wrong method
    for p in [
        "/carta/clip",
        "/carta/clip?token=",
        "/carta/clip?token=wrong",
        "/carta/clip?token=0000000000000000000000000000000000000000",
    ] {
        let r = req(
            &s,
            "POST",
            p,
            &host,
            &[("Origin", "http://evil.example")],
            body,
        );
        assert_eq!(r.code, 403, "{p} gave {}", r.code);
    }
    let tok = server::token(&s.home);
    assert!(tok.len() >= 16);
    let r = req(
        &s,
        "POST",
        "/carta/clip",
        &host,
        &[("X-Colibri-UI", tok.as_str())],
        body,
    );
    assert_eq!(r.code, 403, "a header is not a token");
    let r = req(
        &s,
        "GET",
        &format!("/carta/clip?token={tok}"),
        &host,
        &[],
        b"",
    );
    assert_eq!(r.code, 403, "token alone does not allow GET");
    // the right token from a foreign origin works (that is the bookmarklet's job) ...
    let r = req(
        &s,
        "POST",
        &format!("/carta/clip?token={tok}"),
        &host,
        &[("Origin", "https://some-site.example")],
        body,
    );
    assert_eq!(r.code, 200, "{}", r.body);
    // ... but a wrong Host is still refused even with the token (DNS rebinding)
    let r = req(
        &s,
        "POST",
        &format!("/carta/clip?token={tok}"),
        "evil.example",
        &[],
        body,
    );
    assert_eq!(r.code, 403);
}

#[test]
fn clip_token_is_stable_and_long_enough() {
    let s = start();
    let a = server::token(&s.home);
    let b = server::token(&s.home);
    assert_eq!(a, b);
    assert!(
        a.len() >= 32 && a.chars().all(|c| c.is_ascii_hexdigit()),
        "{a}"
    );
}

#[test]
fn grab_refuses_non_http_schemes() {
    let s = start();
    for u in [
        "file:///etc/passwd",
        "FILE:///etc/passwd",
        "ftp://127.0.0.1/x",
        "gopher://127.0.0.1:9/",
        "javascript:alert(1)",
        "data:text/html,hi",
        "/etc/passwd",
    ] {
        let body = serde_json::json!({ "url": u }).to_string();
        let r = ui(&s, "POST", "/carta/grab", &[], body.as_bytes());
        assert!(r.code >= 400, "{u} gave {}", r.code);
        assert!(!r.body.contains("root:"), "{u} leaked a file");
    }
}

#[test]
fn grab_refuses_loopback_and_private_targets() {
    let s = start();
    let target = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let tport = target.server_addr().to_ip().unwrap().port();
    std::thread::spawn(move || {
        for r in target.incoming_requests() {
            let _ = r.respond(tiny_http::Response::from_string(
                "<html><body><p>internal service secret page content</p></body></html>",
            ));
        }
    });
    for u in [
        format!("http://127.0.0.1:{tport}/"),
        format!("http://localhost:{tport}/"),
        format!("http://[::1]:{tport}/"),
        "http://169.254.169.254/latest/meta-data/".to_string(),
        "http://10.0.0.1/".to_string(),
        "http://192.168.0.1/".to_string(),
    ] {
        let body = serde_json::json!({ "url": u }).to_string();
        let r = ui(&s, "POST", "/carta/grab", &[], body.as_bytes());
        assert!(r.code >= 400, "{u} was fetched: {}", r.body);
    }
}

#[test]
fn oversized_body_is_rejected() {
    let s = start();
    let big = vec![b'a'; 201 * 1024 * 1024];
    let r = ui(
        &s,
        "POST",
        "/carta/upload",
        &[("X-File-Name", "big.txt")],
        &big,
    );
    assert!(matches!(r.code, 413 | 400), "gave {}", r.code);
}

#[test]
fn malformed_and_hostile_json_does_not_crash_the_server() {
    let s = start();
    let deep = format!("{}{}", "[".repeat(200_000), "]".repeat(200_000));
    for body in [
        deep.as_str(),
        "{",
        "\u{0}\u{0}",
        "null",
        "[1,2,3]",
        r#"{"path": 5}"#,
        r#"{"url": {"a": "b"}}"#,
    ] {
        for route in ["folders", "grab", "groups", "run"] {
            let r = ui(&s, "POST", &format!("/carta/{route}"), &[], body.as_bytes());
            assert!(
                matches!(r.code, 400 | 404 | 502),
                "{route}: {} {}",
                r.code,
                r.body
            );
        }
    }
    let r = ui(&s, "GET", "/carta/status", &[], b"");
    assert_eq!(r.code, 200, "server must still answer");
}

#[test]
fn responses_are_not_cacheable_or_sniffable() {
    let s = start();
    let r = ui(&s, "GET", "/carta/status", &[], b"");
    let h = r.head.to_lowercase();
    assert!(h.contains("x-content-type-options: nosniff"), "{h}");
    assert!(h.contains("cache-control: no-store"), "{h}");
    assert!(h.contains("application/json"), "{h}");
}

#[test]
fn ui_page_forbids_framing() {
    let s = start();
    let r = req(&s, "GET", "/", &local(&s), &[], b"");
    let h = r.head.to_lowercase();
    assert!(
        h.contains("x-frame-options") || h.contains("frame-ancestors"),
        "{h}"
    );
}

#[cfg(unix)]
#[test]
fn token_file_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let s = start();
    let _ = server::token(&s.home);
    let mode = std::fs::metadata(s.home.join("clip.token"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o077, 0, "mode {mode:o}");
}

#[test]
fn clip_token_creation_is_race_free() {
    let d = tempfile::tempdir().unwrap();
    let home = d.path().to_path_buf();
    let hs: Vec<_> = (0..16)
        .map(|_| {
            let h = home.clone();
            std::thread::spawn(move || server::token(&h))
        })
        .collect();
    let toks: Vec<String> = hs.into_iter().map(|h| h.join().unwrap()).collect();
    assert!(toks.iter().all(|t| t == &toks[0]), "{toks:?}");
}

// ---------------------------------------------------------------- second audit round

#[test]
fn ssrf_address_classifier_covers_tricky_forms() {
    use cartacolibri::grab::blocked;
    for u in [
        "http://127.0.0.1/",
        "http://127.1/",
        "http://2130706433/",
        "http://0x7f000001/",
        "http://017700000001/",
        "http://0/",
        "http://0.0.0.0/",
        "http://[::1]/",
        "http://[::]/",
        "http://[::ffff:127.0.0.1]/",
        "http://[::ffff:7f00:1]/",
        "http://[::127.0.0.1]/",
        "http://[64:ff9b::7f00:1]/",
        "http://[2002:7f00:1::]/",
        "http://[fe80::1]/",
        "http://[fc00::1]/",
        "http://[fec0::1]/",
        "http://[ff02::1]/",
        "http://100.64.0.1/",
        "http://172.16.0.1/",
        "http://172.31.255.255/",
        "http://192.0.0.1/",
        "http://198.18.0.1/",
        "http://224.0.0.1/",
        "http://255.255.255.255/",
        "http://169.254.169.254/",
        "http://user@127.0.0.1/",
        "http://evil.example@127.0.0.1/",
        "http://127.0.0.1\\@example.com/",
        "http://example.com\\@127.0.0.1/",
        "http://localhost/",
        "http://LOCALHOST./",
        "http://a b/",
        "http://\u{0}127.0.0.1/",
        "http://127.0.0.1\r\nHost: x/",
        "http://",
        "http:///x",
    ] {
        assert!(blocked(u), "{u:?} must be blocked");
    }
}

#[test]
fn ssrf_ip_classifier_unit() {
    use cartacolibri::grab::bad_ip;
    for ip in [
        "100.127.255.255",
        "172.16.5.5",
        "192.168.1.1",
        "::ffff:10.0.0.1",
        "::ffff:a00:1",
        "64:ff9b::a00:1",
        "2002:a00:1::1",
        "2001:0:4136:e378:8000:63bf:3fff:fdd2",
        "fd12::1",
        "ff00::1",
    ] {
        assert!(bad_ip(ip.parse().unwrap()), "{ip}");
    }
    for ip in [
        "8.8.8.8",
        "1.1.1.1",
        "100.128.0.1",
        "172.32.0.1",
        "2606:4700::1111",
        "::ffff:8.8.8.8",
    ] {
        assert!(!bad_ip(ip.parse().unwrap()), "{ip} is public");
    }
}

#[test]
fn redirect_targets_resolve_like_a_browser() {
    // (checked through blocked(): a redirect to an internal address must be refused)
    let s = start();
    let target = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let tport = target.server_addr().to_ip().unwrap().port();
    std::thread::spawn(move || {
        for r in target.incoming_requests() {
            let _ = r.respond(tiny_http::Response::from_string("secret"));
        }
    });
    // a "public" feed/page URL can only be tested offline through the private checks above;
    // here: feeds are guarded too
    let body =
        serde_json::json!({ "url": format!("http://127.0.0.1:{tport}/feed.xml") }).to_string();
    assert_eq!(
        ui(&s, "POST", "/carta/feeds", &[], body.as_bytes()).code,
        200
    );
    let r = ui(&s, "POST", "/carta/feeds/poll", &[], b"");
    assert_eq!(r.code, 200, "{}", r.body);
    assert!(
        r.body.contains("\"feeds_failed\":1"),
        "feed on loopback was polled: {}",
        r.body
    );
    assert!(
        !r.body.contains(&tport.to_string()) || r.body.contains("private network"),
        "{}",
        r.body
    );
}

#[test]
fn clip_refuses_non_http_source_addresses() {
    let s = start();
    let tok = std::fs::read_to_string(s.home.join("clip.token")).unwrap_or_default();
    let tok = if tok.trim().len() >= 16 {
        tok.trim().to_string()
    } else {
        server::token(&s.home)
    };
    for u in [
        "javascript:alert(1)",
        "data:text/html,x",
        "file:///etc/passwd",
        "http://a/\nSource: x",
    ] {
        let body = serde_json::json!({"url": u, "title": "t", "text": "some long enough clipped text here"}).to_string();
        let host = local(&s);
        let r = req(
            &s,
            "POST",
            &format!("/carta/clip?token={tok}"),
            &host,
            &[("Origin", "https://evil.example")],
            body.as_bytes(),
        );
        assert!(r.code >= 400, "{u:?} was saved: {}", r.body);
    }
}

#[test]
fn cross_site_fetch_metadata_is_refused() {
    let s = start();
    let r = ui(
        &s,
        "GET",
        "/carta/status",
        &[("Sec-Fetch-Site", "cross-site")],
        b"",
    );
    assert_eq!(r.code, 403);
    let r = ui(
        &s,
        "GET",
        "/carta/status",
        &[("Sec-Fetch-Site", "same-origin")],
        b"",
    );
    assert_eq!(r.code, 200);
}

#[test]
fn ui_page_has_a_locked_down_csp() {
    let s = start();
    let r = req(&s, "GET", "/", &local(&s), &[], b"");
    let h = r.head.to_lowercase();
    assert!(h.contains("default-src 'none'"), "{}", r.head);
    assert!(h.contains("connect-src 'self'"), "{}", r.head);
}

#[test]
fn zip_import_is_bounded_by_what_is_really_written() {
    use std::io::Write;
    // one 450 MB entry of zeros: tiny on the wire, huge once unpacked; its header tells the truth,
    // so the declared-size check refuses it; nothing may be left behind either way
    let s = start();
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut buf);
        let o = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        z.start_file("vault/big.md", o).unwrap();
        let chunk = vec![b'a'; 1 << 20];
        for _ in 0..450 {
            z.write_all(&chunk).unwrap();
        }
        z.finish().unwrap();
    }
    let r = ui(
        &s,
        "POST",
        "/carta/import",
        &[("X-File-Name", "bomb.zip")],
        buf.get_ref(),
    );
    assert_eq!(r.code, 400, "{}", r.body);
    assert!(r.body.contains("too large"), "{}", r.body);
    let leftovers: Vec<_> = std::fs::read_dir(&s.home)
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("web-"))
        .collect();
    assert!(leftovers.is_empty(), "staging dir left behind");
}

#[test]
fn zip_slip_entries_never_leave_the_staging_area() {
    use std::io::Write;
    let s = start();
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut buf);
        let o = zip::write::SimpleFileOptions::default();
        for n in [
            "../../evil.md",
            "/abs/evil.md",
            "a/../../evil2.md",
            "..\\..\\evil3.md",
            "ok.md",
        ] {
            z.start_file(n, o).unwrap();
            z.write_all(b"# note\n\nsome text").unwrap();
        }
        z.finish().unwrap();
    }
    let parent = s.home.parent().unwrap().to_path_buf();
    ui(
        &s,
        "POST",
        "/carta/import",
        &[("X-File-Name", "slip.zip")],
        buf.get_ref(),
    );
    for n in ["evil.md", "evil2.md", "evil3.md"] {
        assert!(
            !parent.join(n).exists() && !s.home.join(n).exists(),
            "{n} escaped"
        );
        assert!(
            !parent.parent().unwrap().join(n).exists(),
            "{n} escaped further"
        );
    }
}

#[test]
fn import_and_export_names_are_strict() {
    let s = start();
    for n in [
        "a:b.csv",
        "x<y>.csv",
        "con?.csv",
        "dots.csv.",
        "..hidden.csv",
        "a/b.csv",
        "a\\b.csv",
        "",
    ] {
        let r = ui(
            &s,
            "POST",
            "/carta/import",
            &[("X-File-Name", n)],
            b"a,b\n1,2\n",
        );
        assert_eq!(r.code, 400, "{n:?} gave {}", r.code);
    }
}

#[test]
fn backup_does_not_contain_the_clip_token() {
    let s = start();
    let tok = server::token(&s.home);
    let r = ui(&s, "GET", "/carta/backup", &[], b"");
    assert_eq!(r.code, 200);
    assert!(!r.body.contains(&tok), "backup leaks the clipper token");
    let r = ui(&s, "GET", "/carta/export?format=zip", &[], b"");
    assert_eq!(r.code, 200);
    assert!(!r.body.contains(&tok));
}

#[test]
fn download_headers_are_safe_and_not_inline() {
    let s = start();
    let r = ui(
        &s,
        "GET",
        "/carta/export?format=md&name=a%0d%0aX-Evil%3a%201%22.md",
        &[],
        b"",
    );
    assert!(r.code == 200 || r.code == 400, "{}", r.code);
    assert!(!r.head.to_lowercase().contains("x-evil"), "{}", r.head);
    if r.code == 200 {
        assert!(
            r.head
                .to_lowercase()
                .contains("content-disposition: attachment"),
            "{}",
            r.head
        );
        assert!(
            r.head
                .to_lowercase()
                .contains("x-content-type-options: nosniff")
        );
    }
}

#[test]
fn a_panicking_or_odd_request_does_not_leak_capacity() {
    let s = start();
    // 40 hostile requests (more than MAX_CONCURRENT) must all still be answered, one after another
    for i in 0..40 {
        let r = ui(
            &s,
            "GET",
            &format!("/carta/doc?id={}", "9".repeat(i + 1)),
            &[],
            b"",
        );
        assert!(r.code != 0 && r.code != 503, "request {i} gave {}", r.code);
    }
    let r = ui(&s, "GET", "/carta/status", &[], b"");
    assert_eq!(r.code, 200);
}

#[test]
fn truncated_json_bodies_are_not_processed() {
    let s = start();
    let host = local(&s);
    let mut c = TcpStream::connect(("127.0.0.1", s.port)).unwrap();
    c.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let head = format!(
        "POST /carta/folders HTTP/1.1\r\nHost: {host}\r\nX-Colibri-UI: 1\r\nConnection: close\r\nContent-Length: 5000\r\n\r\n{{\"path\":\"/tmp\"}}"
    );
    c.write_all(head.as_bytes()).unwrap();
    c.shutdown(std::net::Shutdown::Write).unwrap();
    let mut out = Vec::new();
    let _ = c.read_to_end(&mut out);
    let t = String::from_utf8_lossy(&out);
    assert!(!t.contains("\"added\""), "half a body was acted on: {t}");
}
