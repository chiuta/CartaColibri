//! The Colibrì engine: /v1 handling, model auto-selection, no /health, discovery file, token usage.
use cartacolibri::carta::{self, Backend};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Minimal OpenAI-style engine: only /v1/models and /v1/chat/completions; chat 404s without the right model id.
fn engine() -> (String, Arc<AtomicBool>) {
    let srv = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = srv.server_addr().to_ip().unwrap().port();
    let stop = Arc::new(AtomicBool::new(false));
    let s2 = stop.clone();
    std::thread::spawn(move || {
        while !s2.load(Ordering::Relaxed) {
            let Ok(Some(mut r)) = srv.recv_timeout(std::time::Duration::from_millis(100)) else {
                continue;
            };
            let url = r.url().to_string();
            let mut body = String::new();
            let _ = std::io::Read::read_to_string(r.as_reader(), &mut body);
            let (code, out) = if url == "/v1/models" {
                (200, r#"{"data":[{"id":"qwen-test"}]}"#.to_string())
            } else if url == "/v1/chat/completions"
                && serde_json::from_str::<serde_json::Value>(&body)
                    .is_ok_and(|v| v["model"] == "qwen-test")
            {
                (200, r#"{"choices":[{"message":{"content":"Answer [S1]"}}],"usage":{"prompt_tokens":11,"completion_tokens":3}}"#.to_string())
            } else {
                (404, "{}".to_string())
            };
            let _ = r.respond(
                tiny_http::Response::from_string(out)
                    .with_status_code(code)
                    .with_header(
                        tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap(),
                    ),
            );
        }
    });
    (format!("http://127.0.0.1:{port}"), stop)
}

#[test]
fn engine_with_v1_models_usage_and_discovery() {
    let (base, stop) = engine();
    // the address works with or without /v1 and trailing slashes
    assert_eq!(carta::normalize_base(&format!("{base}/v1/")), base);
    assert_eq!(carta::normalize_base(&format!("{base}/")), base);
    for u in [base.clone(), format!("{base}/v1")] {
        let b = Backend::gateway(&u, None);
        assert!(b.alive(), "{u}: no /health, but /v1/models answers");
        assert_eq!(b.models(), vec!["qwen-test".to_string()]);
        // no model chosen: the first one the engine reports is used, otherwise this engine answers 404
        let t = b.complete("sys", "hello").unwrap();
        assert!(t.contains("[S1]"));
        let u = carta::take_usage().expect("usage recorded");
        assert_eq!(
            (u.prompt, u.completion, u.model.as_str()),
            (11, 3, "qwen-test")
        );
    }
    // a server that owns the port but is not an engine is not reported alive
    let other = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let p = other.server_addr().to_ip().unwrap().port();
    std::thread::spawn(move || {
        for r in other.incoming_requests() {
            let _ = r.respond(tiny_http::Response::from_string("<html>hi</html>"));
        }
    });
    assert!(!Backend::gateway(&format!("http://127.0.0.1:{p}"), None).alive());
    // discovery file published by a launcher (loopback only)
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("endpoint.json");
    std::fs::write(&f, format!(r#"{{"url":"{base}/v1","model":"x"}}"#)).unwrap();
    unsafe { std::env::set_var("COLIBRI_ENDPOINT_FILE", &f) };
    let found = carta::detect();
    assert!(
        found
            .iter()
            .any(|x| x.url == base && x.models == ["qwen-test"]),
        "{found:?}"
    );
    std::fs::write(&f, r#"{"url":"http://203.0.113.9:8000"}"#).unwrap();
    assert!(
        carta::detect()
            .iter()
            .all(|x| !x.url.contains("203.0.113.9"))
    );
    // Colibri Companion's own settings.json (server_url + api_key)
    unsafe { std::env::remove_var("COLIBRI_ENDPOINT_FILE") };
    let c = tempfile::tempdir().unwrap();
    std::fs::write(
        c.path().join("settings.json"),
        format!(r#"{{"server_url":"{base}/v1/","api_key":"sk-local"}}"#),
    )
    .unwrap();
    unsafe { std::env::set_var("COLIBRI_COMPANION_HOME", c.path()) };
    assert!(carta::detect().iter().any(|x| x.url == base));
    unsafe { std::env::remove_var("COLI_API_KEY") };
    assert_eq!(
        Backend::gateway(&base, None).api_key.as_deref(),
        Some("sk-local")
    );
    // never handed to a non-loopback address
    assert!(
        Backend::gateway("http://203.0.113.9:8000", None)
            .api_key
            .is_none()
    );
    stop.store(true, Ordering::Relaxed);
}

/// An engine with `/v1/packs/complete`, or one that answers 501 there and chats normally.
fn packs_engine(with: bool) -> (String, Arc<AtomicBool>, Arc<std::sync::Mutex<Vec<String>>>) {
    let srv = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = srv.server_addr().to_ip().unwrap().port();
    let stop = Arc::new(AtomicBool::new(false));
    let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
    let (s2, seen2) = (stop.clone(), seen.clone());
    std::thread::spawn(move || {
        while !s2.load(Ordering::Relaxed) {
            let Ok(Some(mut r)) = srv.recv_timeout(std::time::Duration::from_millis(100)) else {
                continue;
            };
            let url = r.url().to_string();
            let mut body = String::new();
            let _ = std::io::Read::read_to_string(r.as_reader(), &mut body);
            seen2.lock().unwrap().push(url.clone());
            let (code, out) = match url.as_str() {
                "/v1/packs/complete" if with => {
                    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
                    let packs = v["packs"].as_array().unwrap();
                    let ok = packs.len() == 3
                        && !packs[1].as_str().unwrap().contains("[S1]")
                        && v["question"].as_str().unwrap().contains("= [S2]");
                    (
                        200,
                        format!(
                            r#"{{"text":"{}","usage":{{"prompt_tokens":5,"completion_tokens":2}}}}"#,
                            if ok { "packed [S1]" } else { "bad shape" }
                        ),
                    )
                }
                "/v1/packs/complete" => (501, "{}".to_string()),
                "/v1/models" => (200, r#"{"data":[{"id":"m"}]}"#.to_string()),
                "/v1/chat/completions" => (
                    200,
                    r#"{"choices":[{"message":{"content":"plain [S1]"}}]}"#.to_string(),
                ),
                _ => (404, "{}".to_string()),
            };
            let _ = r.respond(
                tiny_http::Response::from_string(out)
                    .with_status_code(code)
                    .with_header(
                        tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap(),
                    ),
            );
        }
    });
    (format!("http://127.0.0.1:{port}"), stop, seen)
}

const PACKED: &str =
    "Summarize.\n\n<<SOURCES>>\n### [S1] a.txt\nalpha text\n\n### [S2] b.txt\nbeta text\n<<END>>";

#[test]
fn packs_are_used_when_the_engine_has_them_and_skipped_when_it_does_not() {
    let (base, stop, seen) = packs_engine(true);
    let b = Backend::gateway(&base, None).with_packs(true);
    assert_eq!(b.complete("sys", PACKED).unwrap(), "packed [S1]");
    assert!(
        !seen
            .lock()
            .unwrap()
            .iter()
            .any(|u| u.contains("chat/completions"))
    );
    // not the shape of a sources prompt: ordinary chat even with packs on
    assert_eq!(b.complete("sys", "hello").unwrap(), "plain [S1]");
    stop.store(true, Ordering::Relaxed);

    let (base, stop, seen) = packs_engine(false);
    let b = Backend::gateway(&base, None).with_packs(true);
    assert_eq!(b.complete("sys", PACKED).unwrap(), "plain [S1]");
    assert_eq!(b.complete("sys", PACKED).unwrap(), "plain [S1]");
    let asked = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|u| u.contains("packs"))
        .count();
    assert_eq!(asked, 1, "a 501 is remembered, not asked again");
    // off by default: never asks
    let b = Backend::gateway(&base, None).with_packs(false);
    b.complete("sys", PACKED).unwrap();
    stop.store(true, Ordering::Relaxed);
}
