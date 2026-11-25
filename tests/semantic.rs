//! Meaning search against a fake OpenAI-compatible embeddings server: synonyms are found,
//! re-embedding is incremental, and a missing / broken engine never breaks word search.
use cartacolibri::carta::Backend;
use cartacolibri::library::{Filter, Library};
use cartacolibri::semantic::{Mode, embed_model_name};
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const DIMS: usize = 12;

/// Concept vectors: synonyms share a dimension, unknown words are spread by their first letter.
fn embed(text: &str) -> Vec<f64> {
    let mut v = vec![0.0; DIMS];
    for w in text.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        let d = match w {
            "car" | "automobile" | "vehicle" | "sedan" => 0,
            "invoice" | "bill" | "receipt" => 1,
            "dog" | "puppy" | "canine" => 2,
            "" => continue,
            o => 3 + (o.as_bytes()[0] as usize % (DIMS - 3)),
        };
        v[d] += 1.0;
    }
    v
}

struct Fake {
    url: String,
    calls: Arc<AtomicUsize>,
}

/// `mode`: "ok", "short" (one vector too few), "nan", "nodata"
fn fake(mode: &'static str) -> Fake {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let url = format!("http://{}", server.server_addr().to_ip().unwrap());
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    std::thread::spawn(move || {
        for mut rq in server.incoming_requests() {
            let path = rq.url().to_string();
            let mut body = String::new();
            let _ = std::io::Read::read_to_string(rq.as_reader(), &mut body);
            let out: Value = if path.ends_with("/v1/models") {
                json!({"data": [{"id": "fake-embed"}]})
            } else if path.ends_with("/v1/embeddings") {
                c.fetch_add(1, Ordering::SeqCst);
                let j: Value = serde_json::from_str(&body).unwrap_or(json!({}));
                let inputs: Vec<String> = j["input"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                let mut data: Vec<Value> = inputs
                    .iter()
                    .enumerate()
                    .map(|(i, t)| json!({"index": i, "embedding": embed(t)}))
                    .collect();
                match mode {
                    "short" => {
                        data.pop();
                        json!({"data": data})
                    }
                    "nan" => {
                        json!({"data": data.iter().map(|d| json!({"index": d["index"], "embedding": [null, 1.0]})).collect::<Vec<_>>()})
                    }
                    "nodata" => json!({"error": "no model"}),
                    _ => json!({"data": data}),
                }
            } else {
                json!({})
            };
            let _ = rq.respond(
                tiny_http::Response::from_string(out.to_string()).with_header(
                    "Content-Type: application/json"
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                ),
            );
        }
    });
    Fake { url, calls }
}

fn lib_with(files: &[(&str, &str)]) -> (tempfile::TempDir, Library, String) {
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    for (n, c) in files {
        std::fs::write(docs.join(n), c).unwrap();
    }
    let home = t.path().join("home").to_string_lossy().into_owned();
    let mut lib = Library::open(Some(&home), false).unwrap();
    let d = docs.to_string_lossy().into_owned();
    lib.update(Some(std::slice::from_ref(&d)), &mut |_| {})
        .unwrap();
    (t, lib, d)
}

fn names(h: &[cartacolibri::library::Hit]) -> Vec<String> {
    h.iter()
        .map(|h| h.path.rsplit(['/', '\\']).next().unwrap().to_string())
        .collect()
}

#[test]
fn a_question_finds_a_text_that_uses_other_words() {
    let fk = fake("ok");
    let (_t, lib, _) = lib_with(&[
        ("garage.txt", "The automobile was parked in the garage."),
        ("kitchen.txt", "Flour, eggs and sugar for the cake."),
        ("zoo.txt", "A canine ran across the field."),
    ]);
    let be = Backend::gateway(&fk.url, None);
    let m = embed_model_name(&be, Some("fake-embed"));
    // words alone find nothing for "vehicle"
    assert!(
        lib.search("vehicle", &Filter::default())
            .unwrap()
            .is_empty()
    );
    let st = lib
        .embed_missing(&be, &m, 2, usize::MAX, &mut |_, _| {})
        .unwrap();
    assert_eq!(st.embedded, 3);
    assert_eq!(st.pending, 0);
    let r = lib
        .search_mode(&be, &m, "vehicle", &Filter::default(), Mode::Semantic)
        .unwrap();
    assert_eq!(
        names(&r.hits).first().map(String::as_str),
        Some("garage.txt"),
        "{:?}",
        r.hits
    );
    let r = lib
        .search_mode(&be, &m, "vehicle", &Filter::default(), Mode::Hybrid)
        .unwrap();
    assert_eq!(
        names(&r.hits).first().map(String::as_str),
        Some("garage.txt")
    );
    assert_eq!(r.mode, "hybrid");
    // a word-for-word match still wins in hybrid mode
    let r = lib
        .search_mode(&be, &m, "flour", &Filter::default(), Mode::Hybrid)
        .unwrap();
    assert_eq!(
        names(&r.hits).first().map(String::as_str),
        Some("kitchen.txt")
    );
    // filters still apply to meaning search
    let f = Filter {
        ext: vec!["md".into()],
        ..Default::default()
    };
    assert!(
        lib.search_mode(&be, &m, "vehicle", &f, Mode::Semantic)
            .unwrap()
            .hits
            .is_empty()
    );
}

#[test]
fn embedding_is_incremental_and_follows_reindexing() {
    let fk = fake("ok");
    let (t, mut lib, d) = lib_with(&[("a.txt", "A puppy."), ("b.txt", "An invoice.")]);
    let be = Backend::gateway(&fk.url, None);
    let m = "fake-embed";
    assert_eq!(
        lib.embed_missing(&be, m, 8, usize::MAX, &mut |_, _| {})
            .unwrap()
            .embedded,
        2
    );
    let calls = fk.calls.load(Ordering::SeqCst);
    let again = lib
        .embed_missing(&be, m, 8, usize::MAX, &mut |_, _| {})
        .unwrap();
    assert_eq!((again.embedded, again.already), (0, 2));
    assert_eq!(
        fk.calls.load(Ordering::SeqCst),
        calls,
        "nothing new, no request"
    );
    // one new file, one changed file, one removed file
    std::fs::write(t.path().join("docs/c.txt"), "A sedan.").unwrap();
    std::fs::write(
        t.path().join("docs/a.txt"),
        "A bill and a receipt, now longer.",
    )
    .unwrap();
    std::fs::remove_file(t.path().join("docs/b.txt")).unwrap();
    lib.update(Some(&[d]), &mut |_| {}).unwrap();
    let st = lib
        .embed_missing(&be, m, 8, usize::MAX, &mut |_, _| {})
        .unwrap();
    assert_eq!(st.total, 2, "{st:?}");
    assert_eq!(st.pending, 0);
    assert_eq!(lib.vector_counts(m).0, 2, "stale vectors were dropped");
    // another model replaces the vectors instead of mixing them
    let st = lib
        .embed_missing(&be, "other", 8, usize::MAX, &mut |_, _| {})
        .unwrap();
    assert_eq!(st.embedded, 2);
    assert_eq!(lib.vector_counts(m).0, 0);
    // a limit stops early and the next run continues
    let st = lib
        .embed_missing(&be, "limited", 1, 1, &mut |_, _| {})
        .unwrap();
    assert_eq!((st.embedded, st.pending), (1, 1));
    assert_eq!(
        lib.embed_missing(&be, "limited", 1, 99, &mut |_, _| {})
            .unwrap()
            .pending,
        0
    );
}

#[test]
fn without_a_working_engine_words_still_work() {
    let (_t, lib, _) = lib_with(&[("a.txt", "The automobile is red.")]);
    // nothing listens on this port
    let dead = Backend::gateway("http://127.0.0.1:9", None);
    let f = Filter::default();
    // no vectors yet: hybrid falls back with a note, semantic says what to do
    let r = lib
        .search_mode(&dead, "m", "automobile", &f, Mode::Hybrid)
        .unwrap();
    assert_eq!(r.mode, "bm25");
    assert!(
        r.note.as_deref().unwrap_or("").contains("embed"),
        "{:?}",
        r.note
    );
    assert_eq!(r.hits.len(), 1);
    assert!(
        lib.search_mode(&dead, "m", "automobile", &f, Mode::Semantic)
            .is_err()
    );
    // embedding against a dead engine is a clean error, not a panic
    let e = lib
        .embed_missing(&dead, "m", 8, usize::MAX, &mut |_, _| {})
        .unwrap_err();
    assert!(e.contains("no embeddings"), "{e}");
    // a few ways an engine can answer wrongly
    for bad in ["short", "nan", "nodata"] {
        let fk = fake(bad);
        let be = Backend::gateway(&fk.url, None);
        let e = lib
            .embed_missing(&be, "m", 8, usize::MAX, &mut |_, _| {})
            .unwrap_err();
        assert!(e.contains("no embeddings"), "{bad}: {e}");
        assert_eq!(lib.vector_counts("m").0, 0, "{bad}: nothing half-stored");
    }
    assert!(Mode::parse("nonsense").is_err());
}

#[test]
fn the_offline_mock_runs_the_whole_pipeline() {
    let (_t, lib, _) = lib_with(&[("a.txt", "alpha beta gamma"), ("b.txt", "delta epsilon")]);
    let be = Backend::mock();
    let m = embed_model_name(&be, None);
    assert_eq!(m, "mock");
    lib.embed_missing(&be, &m, 8, usize::MAX, &mut |_, _| {})
        .unwrap();
    let r = lib
        .search_mode(&be, &m, "gamma alpha", &Filter::default(), Mode::Hybrid)
        .unwrap();
    assert_eq!(names(&r.hits).first().map(String::as_str), Some("a.txt"));
}

#[test]
fn the_evaluation_compares_modes_and_reads_both_formats() {
    let fk = fake("ok");
    let (_t, lib, _) = lib_with(&[
        ("garage.txt", "The automobile was parked in the garage."),
        ("kitchen.txt", "Flour, eggs and sugar for the cake."),
    ]);
    let be = Backend::gateway(&fk.url, None);
    let cases = cartacolibri::semantic::load_cases(
        "# comment\nvehicle\tgarage.txt\nflour cake\tkitchen.txt, other.txt\n",
    )
    .unwrap();
    assert_eq!(cases.len(), 2);
    let all = [Mode::Bm25, Mode::Semantic, Mode::Hybrid];
    // before embedding, the meaning modes are skipped with a note
    let (rows, notes) =
        cartacolibri::semantic::evaluate(&lib, &be, "fake-embed", &cases, &all).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(notes.len(), 2);
    assert_eq!(rows[0].misses, vec!["vehicle".to_string()]);
    lib.embed_missing(&be, "fake-embed", 8, usize::MAX, &mut |_, _| {})
        .unwrap();
    let (rows, _) =
        cartacolibri::semantic::evaluate(&lib, &be, "fake-embed", &cases, &all).unwrap();
    let get = |m: &str| rows.iter().find(|r| r.mode == m).unwrap();
    assert!(get("bm25").hit_at_1 < get("hybrid").hit_at_1, "{rows:?}");
    assert_eq!(get("hybrid").hit_at_1, 1.0);
    assert_eq!(get("hybrid").mrr, 1.0);
    let j = cartacolibri::semantic::load_cases(
        r#"[{"q":"a","expect":"x.txt"},{"question":"b","expect":["y","z"]}]"#,
    )
    .unwrap();
    assert_eq!(j[1].expect.len(), 2);
    for bad in [
        "",
        "[]",
        "no tab here",
        "q\t",
        "[{\"q\":\"a\"}]",
        "[1,2",
        "\tx",
    ] {
        assert!(cartacolibri::semantic::load_cases(bad).is_err(), "{bad:?}");
    }
}
