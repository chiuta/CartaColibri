use cartacolibri::carta::{self, Backend};
use cartacolibri::providers;
use std::io::{Read, Write};
use std::net::TcpListener;

#[test]
fn bases_keep_their_own_path() {
    assert_eq!(carta::normalize_base("http://h:1/v1/"), "http://h:1");
    assert_eq!(
        carta::endpoint("http://h:1", "/chat/completions"),
        "http://h:1/v1/chat/completions"
    );
    let g = carta::normalize_base("https://generativelanguage.googleapis.com/v1beta/openai");
    assert_eq!(
        carta::endpoint(&g, "/models"),
        "https://generativelanguage.googleapis.com/v1beta/openai/models"
    );
    let c = carta::normalize_base("https://api.cohere.ai/compatibility/v1");
    assert_eq!(
        carta::endpoint(&c, "/embeddings"),
        "https://api.cohere.ai/compatibility/v1/embeddings"
    );
    let m = carta::normalize_base("https://api.mistral.ai/v1");
    assert_eq!(
        carta::endpoint(&m, "/chat/completions"),
        "https://api.mistral.ai/v1/chat/completions"
    );
}

#[test]
fn cloud_needs_consent_key_and_model() {
    let e = providers::backend("mistral:m", None, false).err().unwrap();
    assert!(e.contains("--allow-cloud"), "{e}");
    let e = providers::backend("gemini", None, true).err().unwrap();
    assert!(e.contains("model"), "{e}");
    assert!(
        providers::backend("nope", None, true)
            .err()
            .unwrap()
            .contains("unknown provider")
    );
    assert!(providers::find("ChatGPT").is_some() && providers::find("Coral").is_some());
    let l = providers::backend("ollama:llama3.2", None, false).unwrap();
    assert_eq!(l.url, "http://127.0.0.1:11434");
    assert_eq!(l.model.as_deref(), Some("llama3.2"));
}

#[test]
fn path_base_is_really_requested() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = l.local_addr().unwrap().port();
    let h = std::thread::spawn(move || {
        let (mut s, _) = l.accept().unwrap();
        // read the whole request (headers, then the body announced by Content-Length): answering early resets the connection
        let mut raw = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = s.read(&mut buf).unwrap();
            raw.extend_from_slice(&buf[..n]);
            let t = String::from_utf8_lossy(&raw).to_string();
            if let Some(i) = t.find("\r\n\r\n") {
                let want = t[..i]
                    .lines()
                    .find_map(|l| {
                        l.to_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                    })
                    .unwrap_or(0);
                if raw.len() >= i + 4 + want {
                    break;
                }
            }
            if n == 0 {
                break;
            }
        }
        let req = String::from_utf8_lossy(&raw).to_string();
        let body = r#"{"choices":[{"message":{"content":"ok"}}]}"#;
        write!(s, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        req
    });
    let be = Backend::gateway(
        &format!("http://127.0.0.1:{port}/v1beta/openai"),
        Some("m".into()),
    );
    assert_eq!(be.complete("s", "u").unwrap(), "ok");
    let req = h.join().unwrap();
    assert!(
        req.starts_with("POST /v1beta/openai/chat/completions "),
        "{req}"
    );
}
