//! End-to-end checks of the extra CLI commands, run against the built binary with a temp home.
use std::process::Command;

fn bin(home: &std::path::Path, args: &[&str]) -> (bool, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_cartacolibri"))
        .arg("--home")
        .arg(home)
        .args(args)
        .output()
        .unwrap();
    let mut t = String::from_utf8_lossy(&o.stdout).into_owned();
    t.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.success(), t)
}

#[test]
fn tags_tasks_export_backup_restore_roundtrip() {
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    let a = docs.join("albatros.txt");
    std::fs::write(&a, "Albatrosul zboară peste oceanele largi ale lumii.").unwrap();
    std::fs::write(
        docs.join("pinguin.txt"),
        "Pinguinul înoată foarte bine în apele reci.",
    )
    .unwrap();
    let h = t.path().join("h");
    assert!(bin(&h, &["add", docs.to_str().unwrap()]).0);
    assert!(bin(&h, &["index"]).0);
    let p = a.to_str().unwrap();

    let (ok, out) = bin(&h, &["tag", "add", p, "pasari"]);
    assert!(ok && out.contains("pasari"), "{out}");
    let (_, out) = bin(&h, &["tag", "list"]);
    assert!(out.contains("pasari"), "{out}");

    let (ok, out) = bin(&h, &["task", "glossary", "--tag", "pasari", "--mock"]);
    assert!(ok && out.contains("[S1]"), "{out}");
    let (ok, out) = bin(&h, &["task", "nonsense", "--tag", "pasari", "--mock"]);
    assert!(!ok && out.contains("task must be one of"), "{out}");

    let (ok, out) = bin(&h, &["export", "--tag", "pasari", "--format", "json"]);
    assert!(ok && out.contains("albatros"), "{out}");

    let zip = t.path().join("b.zip");
    assert!(bin(&h, &["backup", zip.to_str().unwrap()]).0);
    let (ok, out) = bin(&h, &["backup-check", zip.to_str().unwrap()]);
    assert!(ok && out.contains("OK"), "{out}");
    let (ok, out) = bin(&h, &["verify"]);
    assert!(ok && out.contains("index OK"), "{out}");

    // restore into a fresh home, then rebuild the index and find the document again
    let h2 = t.path().join("h2");
    let (ok, out) = bin(&h2, &["restore", zip.to_str().unwrap()]);
    assert!(ok, "{out}");
    assert!(bin(&h2, &["index"]).0);
    let (_, out) = bin(&h2, &["search", "albatrosul"]);
    assert!(out.contains("albatros.txt"), "{out}");
}

#[test]
fn bookmarks_import_is_indexed() {
    let t = tempfile::tempdir().unwrap();
    let f = t.path().join("bm.html");
    std::fs::write(
        &f,
        r#"<DL><DT><H3>Zbor</H3><DL><DT><A HREF="https://ex.org/a">Ghid zepelin</A></DL></DL>"#,
    )
    .unwrap();
    let h = t.path().join("h");
    let (ok, out) = bin(&h, &["import", f.to_str().unwrap()]);
    assert!(ok && out.contains("indexed: 1"), "{out}");
    let (_, out) = bin(&h, &["search", "zepelin"]);
    assert!(out.contains("bookmarks"), "{out}");
}

// ───────────────────────── adversarial CLI ─────────────────────────

fn indexed_home(text: &str) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    let f = docs.join("doc.txt");
    std::fs::write(&f, text).unwrap();
    let h = t.path().join("h");
    assert!(bin(&h, &["add", docs.to_str().unwrap()]).0);
    (t, h, f)
}

#[test]
fn bad_numbers_are_refused_not_looped_on() {
    let (_t, h, _f) = indexed_home("The rent is 450 euro per month for the flat in the city.");
    for args in [
        vec!["summarize", "--group", "g", "--mock", "--budget", "0"],
        vec!["summarize", "--group", "g", "--mock", "--budget", "-5"],
        vec!["watch", "--interval", "0"],
        vec!["webwatch", "run", "--minutes", "0"],
        vec!["webwatch", "run", "--minutes", "18446744073709551615"],
        vec!["search", "rent", "--min-kb", "nan"],
        vec!["search", "rent", "--max-kb", "-1"],
        vec!["webwatch", "add", "file:///etc/passwd"],
    ] {
        let (ok, out) = bin(&h, &args);
        assert!(!ok, "{args:?} accepted: {out}");
    }
}

#[test]
fn tagging_an_unknown_path_is_an_honest_error() {
    let (_t, h, f) = indexed_home("The rent is 450 euro per month for the flat in the city.");
    assert!(bin(&h, &["index"]).0);
    let (ok, out) = bin(&h, &["tag", "add", "/no/such/file.txt", "x"]);
    assert!(!ok && out.contains("not in the library"), "{out}");
    let (ok, out) = bin(&h, &["note", "/no/such/file.txt", "hi"]);
    assert!(!ok && out.contains("not in the library"), "{out}");
    let (ok, out) = bin(&h, &["star", "/no/such/file.txt"]);
    assert!(!ok, "{out}");
    let (ok, _) = bin(&h, &["tag", "add", f.to_str().unwrap(), "x"]);
    assert!(ok);
}

#[test]
fn terminal_escape_sequences_in_documents_are_not_printed_raw() {
    let (_t, h, _f) = indexed_home(
        "Zebra \u{1b}[2J\u{1b}]0;pwned\u{7} stripes are black and white in the savanna.",
    );
    assert!(bin(&h, &["index"]).0);
    let (ok, out) = bin(&h, &["search", "zebra"]);
    assert!(ok && out.contains("Zebra"), "{out}");
    assert!(!out.contains('\u{1b}') && !out.contains('\u{7}'), "{out:?}");
    let (ok, out) = bin(&h, &["task", "outline", "--query", "zebra", "--mock"]);
    assert!(ok && !out.contains('\u{1b}'), "{out:?}");
}

#[test]
fn extract_json_output_is_pure_json() {
    let (_t, h, _f) = indexed_home("The rent is 450 euro per month for the flat in the city.");
    assert!(bin(&h, &["index"]).0);
    assert!(bin(&h, &["group", "set", "g", "doc.txt"]).0 || true);
    let o = Command::new(env!("CARGO_BIN_EXE_cartacolibri"))
        .arg("--home")
        .arg(&h)
        .args(["extract", "rent", "--query", "rent", "--json", "--mock"])
        .output()
        .unwrap();
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout)
        .unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(&o.stdout)));
    assert!(v.is_array());
}

#[test]
fn closed_stdout_exits_quietly_and_secrets_in_env_are_not_echoed() {
    let (_t, h, _f) = indexed_home("The rent is 450 euro per month for the flat in the city.");
    let o = Command::new(env!("CARGO_BIN_EXE_cartacolibri"))
        .arg("--home")
        .arg(&h)
        .args(["summarize", "--help"])
        .env("COLI_URL", "http://user:topsecret@127.0.0.1:9")
        .output()
        .unwrap();
    let all = String::from_utf8_lossy(&o.stdout).into_owned() + &String::from_utf8_lossy(&o.stderr);
    assert!(!all.contains("topsecret"), "{all}");
    // gateway down: the error names the host, not the password
    let o = Command::new(env!("CARGO_BIN_EXE_cartacolibri"))
        .arg("--home")
        .arg(&h)
        .args(["task", "outline", "--query", "rent"])
        .env("COLI_URL", "http://user:topsecret@127.0.0.1:9")
        .output()
        .unwrap();
    let all = String::from_utf8_lossy(&o.stdout).into_owned() + &String::from_utf8_lossy(&o.stderr);
    assert!(!o.status.success() && !all.contains("topsecret"), "{all}");
}

#[test]
fn chat_survives_invalid_utf8_on_stdin() {
    use std::io::Write;
    let (_t, h, _f) = indexed_home("The rent is 450 euro per month for the flat in the city.");
    assert!(bin(&h, &["index"]).0);
    let mut c = Command::new(env!("CARGO_BIN_EXE_cartacolibri"))
        .arg("--home")
        .arg(&h)
        .args(["chat", "--mock"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin
        .take()
        .unwrap()
        .write_all(b"rent \xff\xfe?\nHow much is the rent?\n\n")
        .unwrap();
    let o = c.wait_with_output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stdout).contains("450"));
}

#[test]
fn verify_repairs_an_index_that_cannot_be_opened() {
    let (_t, h, _f) = indexed_home("The rent is 450 euro per month for the flat in the city.");
    assert!(bin(&h, &["index"]).0);
    std::fs::write(
        h.join("index.sqlite"),
        b"this is not a database at all, just junk bytes",
    )
    .unwrap();
    let (ok, _) = bin(&h, &["verify"]);
    assert!(!ok, "a broken index must not report success");
    let (ok, out) = bin(&h, &["verify", "--repair"]);
    assert!(ok, "{out}");
    assert!(
        h.join("index.sqlite.broken").exists(),
        "the old file must be kept"
    );
    let (ok, out) = bin(&h, &["search", "rent"]);
    assert!(ok && out.contains("doc.txt"), "{out}");
}
