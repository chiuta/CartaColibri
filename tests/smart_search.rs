//! Smart search: the engine proposes synonyms; only plain words are ever accepted back.
use cartacolibri::carta::{self, Backend};

fn engine(reply: &'static str) -> String {
    let srv = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = srv.server_addr().to_ip().unwrap().port();
    std::thread::spawn(move || {
        for mut r in srv.incoming_requests() {
            let url = r.url().to_string();
            let mut body = String::new();
            let _ = std::io::Read::read_to_string(r.as_reader(), &mut body);
            let out = if url == "/v1/models" {
                r#"{"data":[{"id":"m"}]}"#.to_string()
            } else {
                serde_json::json!({"choices":[{"message":{"content":reply}}]}).to_string()
            };
            let _ = r.respond(tiny_http::Response::from_string(out).with_header(
                tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap(),
            ));
        }
    });
    format!("http://127.0.0.1:{port}")
}

#[test]
fn only_plain_new_words_come_back() {
    let reply = "- chirie\n1. închiriere\nAND\nrent\n\"; DROP TABLE\nlocuință*\nx\nchirie\nevil OR (hack)\nabcdefghijklmnopqrstuvwxyzabcdefghij";
    let be = Backend::gateway(&engine(reply), None);
    let got = carta::expand_query(&be, "contract rent");
    // "rent" is already in the query, "chirie" is a duplicate, operators/punctuation/too short/too long are dropped
    assert_eq!(got, vec!["chirie", "închiriere", "locuință"]);
}

#[test]
fn smart_search_fails_quietly() {
    // no engine: no expansion, no panic; mock backend: no expansion
    let dead = Backend::gateway("http://127.0.0.1:9", None);
    assert!(carta::expand_query(&dead, "contract").is_empty());
    assert!(carta::expand_query(&Backend::mock(), "contract").is_empty());
    assert!(carta::expand_query(&dead, "   ").is_empty());
}

#[test]
fn date_and_regex_filters_in_the_query_box() {
    use cartacolibri::library::{Filter, parse_date, parse_inline};
    assert_eq!(parse_date("1970-01-02"), Some(86400));
    assert_eq!(parse_date("2000-03-01"), Some(951868800));
    assert_eq!(parse_date("2025"), parse_date("2025-01-01"));
    assert_eq!(parse_date("2025-13-01"), None);
    let (rest, f) = parse_inline(
        "invoice after:2024-06 before:2025 re:INV-\\d+",
        &Filter::default(),
    );
    assert_eq!(rest, "invoice");
    assert_eq!(f.after, parse_date("2024-06-01"));
    assert_eq!(f.before, parse_date("2025-01-01"));
    assert_eq!(f.regex.as_deref(), Some("INV-\\d+"));
    // an unparsable date stays in the query as a plain word
    let (rest, f) = parse_inline("after:soon", &Filter::default());
    assert_eq!(rest, "after:soon");
    assert!(f.after.is_none());
}
