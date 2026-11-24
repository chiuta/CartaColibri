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
