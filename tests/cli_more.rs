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
