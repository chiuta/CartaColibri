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
