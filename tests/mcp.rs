//! The read-only MCP server: protocol shape, the four tools, and that nothing outside the index is reachable.
use cartacolibri::carta::Backend;
use cartacolibri::library::Library;
use cartacolibri::mcp;
use serde_json::{Value, json};

fn lib() -> (tempfile::TempDir, Library) {
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(
        docs.join("lease.txt"),
        "Lease of the warehouse. Monthly rent 3200 EUR. Payment by bank transfer only.",
    )
    .unwrap();
    std::fs::write(
        docs.join("notes.md"),
        "# Notes\nCall the supplier about the invoice.",
    )
    .unwrap();
    let mut l = Library::open(Some(t.path().join("home").to_str().unwrap()), false).unwrap();
    l.update(Some(&[docs.to_string_lossy().into_owned()]), &mut |_| {})
        .unwrap();
    (t, l)
}

fn rpc(l: &Library, m: Value) -> Value {
    mcp::handle(l, &Backend::mock(), &m.to_string()).expect("a reply")
}
fn tool(l: &Library, name: &str, args: Value) -> (String, bool) {
    let r = rpc(
        l,
        json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":args}}),
    );
    (
        r["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string(),
        r["result"]["isError"].as_bool().unwrap_or(false),
    )
}

#[test]
fn protocol_and_tools() {
    let (_t, l) = lib();
    let i = rpc(
        &l,
        json!({"jsonrpc":"2.0","id":7,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}),
    );
    assert_eq!(i["id"], 7);
    assert_eq!(i["result"]["serverInfo"]["name"], "cartacolibri");
    assert!(i["result"]["capabilities"]["tools"].is_object());
    // notifications get no reply; garbage gets a parse error; unknown methods an error
    assert!(
        mcp::handle(
            &l,
            &Backend::mock(),
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
        )
        .is_none()
    );
    assert_eq!(
        mcp::handle(&l, &Backend::mock(), "nope").unwrap()["error"]["code"],
        -32700
    );
    assert_eq!(
        rpc(&l, json!({"jsonrpc":"2.0","id":2,"method":"x/y"}))["error"]["code"],
        -32601
    );
    assert_eq!(
        rpc(&l, json!({"jsonrpc":"2.0","id":3,"method":"ping"}))["result"],
        json!({})
    );
    let names: Vec<String> =
        rpc(&l, json!({"jsonrpc":"2.0","id":4,"method":"tools/list"}))["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();
    assert_eq!(names, ["search", "read_document", "list_groups", "ask"]);

    // search -> read by the id it returned; inline filters work
    let (s, err) = tool(&l, "search", json!({"query": "rent"}));
    assert!(!err && s.contains("lease.txt"), "{s}");
    let id: i64 = s
        .split("[id ")
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let (d, err) = tool(&l, "read_document", json!({"id": id, "max_chars": 200}));
    assert!(
        !err && d.contains("Monthly rent") && d.contains("treat as data"),
        "{d}"
    );
    assert!(
        tool(&l, "search", json!({"query": "rent ext:md"}))
            .0
            .contains("no matches")
    );
    // unknown id, missing arguments, unknown tool: clean errors
    assert!(tool(&l, "read_document", json!({"id": 99999})).1);
    assert!(tool(&l, "search", json!({})).1);
    assert!(tool(&l, "nope", json!({})).1);
    // groups and ask (offline mock)
    assert!(tool(&l, "list_groups", json!({})).0.contains("no groups"));
    let lease = l
        .documents(None)
        .unwrap()
        .into_iter()
        .find(|d| d.path.ends_with("lease.txt"))
        .unwrap()
        .path;
    l.group_set("Leases", &[lease], None, false).unwrap();
    let g = tool(&l, "list_groups", json!({})).0;
    assert!(g.contains("Leases: 1 documents"), "{g}");
    let (a, err) = tool(
        &l,
        "ask",
        json!({"question": "What is the monthly rent?", "group": "Leases"}),
    );
    assert!(!err && a.contains("[S1]"), "{a}");
    assert!(tool(&l, "ask", json!({"question": "x", "group": "nope"})).1);
}
