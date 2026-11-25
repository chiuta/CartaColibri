//! The way an OpenAI-compatible engine (Colibrì, Ollama, LM Studio) can answer wrongly must give
//! a clear error, never a panic or a hang, and a good answer in the Ollama/LM Studio shape works.
use cartacolibri::carta::Backend;
use std::time::Duration;

fn serve(status: u16, body: &'static str, delay_ms: u64) -> String {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let url = format!("http://{}", server.server_addr().to_ip().unwrap());
    std::thread::spawn(move || {
        for mut rq in server.incoming_requests() {
            let mut sink = String::new();
            let _ = std::io::Read::read_to_string(rq.as_reader(), &mut sink);
            let models = rq.url().ends_with("/v1/models");
            std::thread::sleep(Duration::from_millis(delay_ms));
            let (st, b) = if models {
                (
                    200,
                    r#"{"object":"list","data":[{"id":"llama3.2:3b","object":"model"}]}"#,
                )
            } else {
                (status, body)
            };
            let _ = rq.respond(
                tiny_http::Response::from_string(b)
                    .with_status_code(st)
                    .with_header(
                        "Content-Type: application/json"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    ),
            );
        }
    });
    url
}

fn ask(url: &str) -> Result<String, String> {
    let mut be = Backend::gateway(url, None);
    be.retries = 0;
    be.timeout = Duration::from_secs(2);
    be.complete("system", "user")
}

#[test]
fn an_ollama_shaped_answer_works() {
    let url = serve(
        200,
        r#"{"id":"x","object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":"  Hello [S1].  "},"finish_reason":"stop"}],"usage":{"prompt_tokens":3,"completion_tokens":2}}"#,
        0,
    );
    assert_eq!(ask(&url).unwrap(), "Hello [S1].");
}

#[test]
fn every_wrong_answer_is_a_clear_error() {
    for (st, body, want) in [
        (500, r#"{"error":"boom"}"#, "Colibrì engine"),
        (401, r#"{"error":"no key"}"#, "COLI_API_KEY"),
        (403, "{}", "COLI_API_KEY"),
        (200, "this is not json", "Colibrì engine"),
        (200, r#"{"choices":[]}"#, "unexpected response"),
        (
            200,
            r#"{"choices":[{"message":{"content":"   "}}]}"#,
            "empty answer",
        ),
        (
            200,
            r#"{"choices":[{"message":{"content":null}}]}"#,
            "unexpected response",
        ),
        (
            200,
            r#"{"choices":[{"message":{"content":42}}]}"#,
            "unexpected response",
        ),
    ] {
        let url = serve(st, body, 0);
        let e = ask(&url).unwrap_err();
        assert!(e.contains(want), "{st} {body}: {e}");
    }
}

#[test]
fn a_slow_engine_times_out_and_a_dead_one_is_reported() {
    let url = serve(200, r#"{"choices":[{"message":{"content":"late"}}]}"#, 3000);
    let mut be = Backend::gateway(&url, None);
    be.retries = 0;
    be.timeout = Duration::from_millis(600);
    let t = std::time::Instant::now();
    assert!(be.complete("s", "u").is_err());
    assert!(t.elapsed() < Duration::from_secs(10));
    let e = ask("http://127.0.0.1:9").unwrap_err();
    assert!(e.contains("no answer from the Colibrì engine"), "{e}");
}
