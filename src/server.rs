//! The local UI server: serves the single-page UI and the /carta/* JSON API, this machine only.
//! Checks: bind to 127.0.0.1, Host must be a loopback name, Origin (if sent) must be the same
//! host, and every /carta/* call must carry the `X-Colibri-UI` header (a plain cross-site form
//! or image request cannot set it). `/carta/clip` is the one exception, guarded by a secret token.

use crate::{api, library};
use percent_encoding::percent_decode_str;
use std::collections::HashMap;
use std::io::Read;
use tiny_http::{Header, Method, Request, Response, Server};

const UI: &str = include_str!("../assets/carta.html");
const LOGO: &[u8] = include_bytes!("../assets/cartacolibri-logo.png");
const MARK: &[u8] = include_bytes!("../assets/cartacolibri-mark.png");
const MAX_UPLOAD: u64 = 100 * 1024 * 1024;
const MAX_JSON: u64 = 2 * 1024 * 1024;
const MAX_CONCURRENT: usize = 32;
static ACTIVE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Reads the body, refusing (413) instead of silently truncating.
fn read_body(r: &mut Request, limit: u64) -> Result<Vec<u8>, ()> {
    if r.body_length().is_some_and(|n| n as u64 > limit) {
        return Err(());
    }
    let mut body = Vec::new();
    let _ = r.as_reader().take(limit + 1).read_to_end(&mut body);
    if body.len() as u64 > limit {
        Err(())
    } else {
        Ok(body)
    }
}

fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |d, (x, y)| d | (x ^ y)) == 0
}

fn hdr(k: &str, v: &str) -> Header {
    Header::from_bytes(k.as_bytes(), v.as_bytes()).unwrap()
}

fn host_ok(h: &str) -> bool {
    let name = h
        .rsplit_once(':')
        .filter(|(_, p)| p.chars().all(|c| c.is_ascii_digit()))
        .map(|(n, _)| n)
        .unwrap_or(h);
    matches!(name, "127.0.0.1" | "localhost" | "[::1]")
}

fn get<'a>(r: &'a Request, k: &str) -> Option<&'a str> {
    r.headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(k))
        .map(|h| h.value.as_str())
}

fn write_private(p: &std::path::Path, t: &str) {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(p)
        {
            let _ = f.write_all(t.as_bytes());
        }
        let _ = std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = std::fs::write(p, t);
}

pub fn token(home: &std::path::Path) -> String {
    let p = home.join("clip.token");
    if let Ok(t) = std::fs::read_to_string(&p)
        && t.trim().len() >= 16
    {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600));
        }
        return t.trim().to_string();
    }
    let mut b = [0u8; 20];
    let t = if getrandom::fill(&mut b).is_ok() {
        hex::encode(b)
    } else {
        use sha1::{Digest, Sha1};
        hex::encode(Sha1::digest(
            format!("{:?}{}", std::time::SystemTime::now(), std::process::id()).as_bytes(),
        ))
    };
    // Publish atomically: write a private temp file, then hard-link it into place.
    // The link fails if another caller got there first, so every caller ends up
    // returning the same, winning token.
    let tmp = home.join(format!("clip.token.{}.{:x}", std::process::id(), {
        let mut n = [0u8; 4];
        let _ = getrandom::fill(&mut n);
        u32::from_le_bytes(n)
    }));
    write_private(&tmp, &t);
    let linked = std::fs::hard_link(&tmp, &p).is_ok();
    let _ = std::fs::remove_file(&tmp);
    if !linked {
        for _ in 0..50 {
            if let Ok(w) = std::fs::read_to_string(&p)
                && w.trim().len() >= 16
            {
                return w.trim().to_string();
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        write_private(&p, &t);
    }
    t
}

fn respond(r: Request, code: u16, ctype: &str, body: Vec<u8>, extra: &[Header]) {
    let mut resp = Response::from_data(body)
        .with_status_code(code)
        .with_header(hdr("Content-Type", ctype))
        .with_header(hdr("Cache-Control", "no-store"));
    resp = resp
        .with_header(hdr("X-Content-Type-Options", "nosniff"))
        .with_header(hdr("X-Frame-Options", "DENY"))
        .with_header(hdr("Referrer-Policy", "no-referrer"))
        .with_header(hdr(
            "Content-Security-Policy",
            "frame-ancestors 'none'; base-uri 'none'; form-action 'self'; object-src 'none'",
        ));
    for h in extra {
        resp = resp.with_header(h.clone());
    }
    let _ = r.respond(resp);
}

fn json(r: Request, code: u16, v: serde_json::Value, extra: &[Header]) {
    respond(
        r,
        code,
        "application/json; charset=utf-8",
        serde_json::to_vec(&v).unwrap(),
        extra,
    )
}

fn handle(mut r: Request, home: Option<String>, backend: String) {
    let host = get(&r, "Host").unwrap_or("").to_string();
    if !host_ok(&host) {
        return json(
            r,
            403,
            serde_json::json!({"error": "this page works on this machine only"}),
            &[],
        );
    }
    if let Some(o) = get(&r, "Origin")
        && o != format!("http://{host}")
        && r.url().split('?').next() != Some("/carta/clip")
    {
        return json(
            r,
            403,
            serde_json::json!({"error": "cross-site request refused"}),
            &[],
        );
    }
    let url = r.url().to_string();
    let (path, qs) = url.split_once('?').unwrap_or((&url, ""));
    let method = r.method().clone();
    let home_path = library::state_dir(home.as_deref()).ok();

    if path == "/carta/clip" {
        let cors = [
            hdr("Access-Control-Allow-Origin", "*"),
            hdr("Access-Control-Allow-Headers", "Content-Type"),
            hdr("Access-Control-Allow-Methods", "POST, OPTIONS"),
        ];
        if method == Method::Options {
            return respond(r, 204, "text/plain", vec![], &cors);
        }
        let q = parse_query(qs);
        let ok = home_path
            .as_ref()
            .is_some_and(|h| q.get("token").is_some_and(|t| same(t, &token(h))));
        if !ok || method != Method::Post {
            return json(
                r,
                403,
                serde_json::json!({"error": "bad or missing token"}),
                &cors,
            );
        }
        let Ok(body) = read_body(&mut r, MAX_JSON) else {
            return json(
                r,
                413,
                serde_json::json!({"error": "request too large"}),
                &cors,
            );
        };
        let req = api::Req {
            method: "POST",
            route: "grab",
            query: q,
            body: &body,
            filename: None,
            backend_url: &backend,
        };
        let (code, v) = api::handle(home.as_deref(), &req);
        return json(r, code, v, &cors);
    }

    if !path.starts_with("/carta/") {
        return match (method.clone(), path) {
            (Method::Get, "/" | "/carta" | "/carta.html") => respond(
                r,
                200,
                "text/html; charset=utf-8",
                UI.as_bytes().to_vec(),
                &[],
            ),
            (Method::Get, "/carta-logo.png") => respond(r, 200, "image/png", LOGO.to_vec(), &[]),
            (Method::Get, "/carta-mark.png") => respond(r, 200, "image/png", MARK.to_vec(), &[]),
            (Method::Get, "/health") => {
                let up = crate::carta::Backend::gateway(&backend, None).alive();
                json(
                    r,
                    if up { 200 } else { 503 },
                    serde_json::json!({"gateway": up}),
                    &[],
                )
            }
            _ => json(r, 404, serde_json::json!({"error": "not found"}), &[]),
        };
    }
    if get(&r, "X-Colibri-UI").is_none() {
        return json(
            r,
            403,
            serde_json::json!({"error": "missing X-Colibri-UI header"}),
            &[],
        );
    }
    let route = path.trim_start_matches("/carta/").trim_end_matches('/');
    let limit = if matches!(route, "upload" | "import") {
        MAX_UPLOAD
    } else {
        MAX_JSON
    };
    let Ok(body) = read_body(&mut r, limit) else {
        return json(
            r,
            413,
            serde_json::json!({"error": "request too large"}),
            &[],
        );
    };
    let filename =
        get(&r, "X-File-Name").map(|f| percent_decode_str(f).decode_utf8_lossy().into_owned());
    let req = api::Req {
        method: method.as_str(),
        route,
        query: parse_query(qs),
        body: &body,
        filename,
        backend_url: &backend,
    };
    if api::is_download(req.method, route) {
        return match api::download(home.as_deref(), &req) {
            Ok(d) => {
                let cd = format!("attachment; filename=\"{}\"", d.name);
                respond(r, 200, d.ctype, d.bytes, &[hdr("Content-Disposition", &cd)]);
            }
            Err((code, m)) => json(r, code, serde_json::json!({"error": m}), &[]),
        };
    }
    let (code, v) = api::handle(home.as_deref(), &req);
    json(r, code, v, &[])
}

fn parse_query(qs: &str) -> HashMap<String, String> {
    qs.split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            let dec = |s: &str| {
                percent_decode_str(&s.replace('+', " "))
                    .decode_utf8_lossy()
                    .into_owned()
            };
            (dec(k), dec(v))
        })
        .collect()
}

/// Blocks serving forever. `addr` like "127.0.0.1:8765".
pub fn serve(addr: &str, home: Option<String>, backend: String) -> Result<(), String> {
    let server = Server::http(addr).map_err(|e| format!("cannot listen on {addr}: {e}"))?;
    if let Ok(h) = library::state_dir(home.as_deref()) {
        token(&h);
    }
    for req in server.incoming_requests() {
        let (h, b) = (home.clone(), backend.clone());
        if ACTIVE.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= MAX_CONCURRENT {
            ACTIVE.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            json(
                req,
                503,
                serde_json::json!({"error": "busy, try again"}),
                &[],
            );
            continue;
        }
        let dbg = std::env::var_os("CARTACOLIBRI_DEBUG_REQ").map(|_| {
            let line = format!("{} {}", req.method(), req.url());
            eprintln!("[req] recv  {line}");
            (line, std::time::Instant::now())
        });
        std::thread::spawn(move || {
            handle(req, h, b);
            if let Some((line, t0)) = dbg {
                eprintln!("[req] done  {line}  {} ms", t0.elapsed().as_millis());
            }
            ACTIVE.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
    }
    Ok(())
}
