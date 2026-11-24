use cartacolibri::library::{Filter, Library};
use cartacolibri::organize::*;
use std::fs;

fn setup(files: &[(&str, &str)]) -> (tempfile::TempDir, tempfile::TempDir, Library) {
    let home = tempfile::tempdir().unwrap();
    let docs = tempfile::tempdir().unwrap();
    for (n, t) in files {
        fs::write(docs.path().join(n), t).unwrap();
    }
    let mut lib = Library::open(Some(home.path().to_str().unwrap()), false).unwrap();
    lib.update(
        Some(&[docs.path().to_string_lossy().into_owned()]),
        &mut |_| {},
    )
    .unwrap();
    (home, docs, lib)
}

fn corpus() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "a.txt",
            "The hummingbird feeds on nectar and the nectar of flowers is sweet.",
        ),
        (
            "b.txt",
            "Hummingbird wings beat quickly while the bird drinks nectar from flowers.",
        ),
        (
            "c.txt",
            "Quarterly tax report about invoices and ledgers for the company.",
        ),
        (
            "d.txt",
            "Pasarea colibri este foarte mica si se hraneste cu nectar de flori.",
        ),
    ]
}

fn path_of(lib: &Library, name: &str) -> String {
    lib.documents(None)
        .unwrap()
        .into_iter()
        .find(|d| d.path.ends_with(name))
        .unwrap()
        .path
}

#[test]
fn tags_stars_notes_survive_reindex() {
    let (_h, d, mut lib) = setup(&corpus());
    let a = path_of(&lib, "a.txt");
    let b = path_of(&lib, "b.txt");
    assert_eq!(
        tag_add(&lib, &a, &["#Birds".into(), "read".into()]).unwrap(),
        vec!["birds", "read"]
    );
    tag_add(&lib, &b, &["birds".into()]).unwrap();
    set_star(&lib, &a, true).unwrap();
    set_note(&lib, &a, "check the nectar claim").unwrap();
    assert_eq!(tag_list(&lib)["birds"], 2);
    let hits = search_tagged(&lib, "nectar", &Filter::default(), Some("read"), false).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, a);
    assert_eq!(
        search_tagged(&lib, "nectar", &Filter::default(), None, true)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        search_tagged(&lib, "", &Filter::default(), Some("birds"), false)
            .unwrap()
            .len(),
        2
    );
    // reindex after a change
    fs::write(
        d.path().join("a.txt"),
        "Hummingbird nectar, rewritten text.",
    )
    .unwrap();
    lib.update(None, &mut |_| {}).unwrap();
    assert_eq!(note_of(&lib, &a), "check the nectar claim");
    assert!(starred(&lib).contains(&a));
    assert_eq!(
        tag_remove(&lib, &a, &["read".into()]).unwrap(),
        vec!["birds"]
    );
    assert!(tag_remove(&lib, &a, &[]).unwrap().is_empty());
}

#[test]
fn saved_searches_rerun() {
    let (_h, d, mut lib) = setup(&corpus());
    saved_set(
        &lib,
        SavedSearch {
            name: "nect".into(),
            query: "nectar".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(saved_run(&lib, "nect").unwrap().len(), 3);
    fs::write(d.path().join("e.txt"), "more nectar here").unwrap();
    lib.update(None, &mut |_| {}).unwrap();
    assert_eq!(saved_run(&lib, "nect").unwrap().len(), 4);
    assert_eq!(saved_list(&lib).len(), 1);
    assert!(saved_remove(&lib, "nect").unwrap());
    assert!(saved_run(&lib, "nect").is_err());
}

#[test]
fn recent_and_related() {
    let (_h, _d, lib) = setup(&corpus());
    let a = path_of(&lib, "a.txt");
    let c = path_of(&lib, "c.txt");
    record_open(&lib, &a).unwrap();
    record_open(&lib, &c).unwrap();
    record_open(&lib, &a).unwrap();
    assert_eq!(recent(&lib, 5), vec![a.clone(), c.clone()]);
    assert_eq!(most_opened(&lib, 1), vec![(a.clone(), 2)]);
    let id = lib
        .documents(None)
        .unwrap()
        .into_iter()
        .find(|d| d.path == a)
        .unwrap()
        .id;
    let rel = related(&lib, id, 3).unwrap();
    assert!(rel[0].path.ends_with("b.txt"), "{rel:?}");
    assert!(rel.iter().all(|h| h.id != id));
}

#[test]
fn export_formats_and_import() {
    let (_h, d, lib) = setup(&corpus());
    let a = path_of(&lib, "a.txt");
    tag_add(&lib, &a, &["x,y".into()]).unwrap();
    set_star(&lib, &a, true).unwrap();
    set_note(&lib, &a, "has \"quotes\", commas").unwrap();
    let hits = lib.search("nectar", &Filter::default()).unwrap();
    let items = items_from_hits(&lib, &hits);
    assert!(to_markdown(&items).contains("a"));
    let csv = to_csv(&items);
    assert!(csv.contains("\"has \"\"quotes\"\", commas\""));
    let zp = d.path().join("out.zip");
    assert_eq!(export_zip(&items, &zp).unwrap(), 3);
    let z = zip::ZipArchive::new(fs::File::open(&zp).unwrap()).unwrap();
    let names: Vec<_> = z.file_names().map(|n| n.unwrap().into_owned()).collect();
    assert!(
        names.iter().any(|n| n == "citations.md")
            && names.iter().any(|n| n.starts_with("sources/a"))
    );
    // import into a fresh library
    let json = to_json(&items);
    let (_h2, _d2, lib2) = setup(&[]);
    let rep = import_json(&lib2, &json).unwrap();
    assert_eq!((rep.tagged, rep.starred, rep.notes), (1, 1, 1));
    assert!(meta_of(&lib2, &a).starred);
    assert!(import_json(&lib2, "{}").is_err());
}

#[test]
fn groups_and_collections_export() {
    let (_h, d, lib) = setup(&corpus());
    lib.group_set(
        "g",
        &[d.path().join("c.txt").to_string_lossy().into_owned()],
        None,
        false,
    )
    .unwrap();
    let it = items_from_group(&lib, "g").unwrap();
    assert_eq!(it.len(), 1);
    assert_eq!(it[0].title, "c");
}

#[test]
fn backup_restore_verified() {
    let (h, d, lib) = setup(&corpus());
    let a = path_of(&lib, "a.txt");
    tag_add(&lib, &a, &["keep".into()]).unwrap();
    lib.group_set("g", std::slice::from_ref(&a), None, false)
        .unwrap();
    let out = tempfile::tempdir().unwrap();
    let full = out.path().join("full.zip");
    let slim = out.path().join("slim.zip");
    assert!(backup(&lib, &full, true).unwrap().includes_index);
    std::fs::write(lib.home.join("clip.token"), "secret-token-value").unwrap();
    let m = backup(&lib, &slim, false).unwrap();
    assert!(
        m.files.iter().all(|f| !f.name.starts_with("clip.token")),
        "token leaked into backup"
    );
    let m2 = backup(&lib, &full, true).unwrap();
    assert!(m2.files.iter().all(|f| !f.name.starts_with("clip.token")));
    assert!(!m.includes_index && m.files.iter().all(|f| !f.name.starts_with("index.sqlite")));
    assert!(backup_verify(&full).is_ok());
    drop(lib);
    // slim restore into a new home: roots come back, index needs update
    let h2 = tempfile::tempdir().unwrap();
    let r = restore(&slim, h2.path()).unwrap();
    assert!(r.needs_update && r.roots_restored == 1);
    let mut lib2 = Library::open(Some(h2.path().to_str().unwrap()), false).unwrap();
    lib2.update(None, &mut |_| {}).unwrap();
    assert_eq!(lib2.search("nectar", &Filter::default()).unwrap().len(), 3);
    assert_eq!(tags_of(&lib2, &a), vec!["keep"]);
    assert!(lib2.groups().contains_key("g"));
    drop(lib2);
    // full restore keeps the index
    let h3 = tempfile::tempdir().unwrap();
    assert!(!restore(&full, h3.path()).unwrap().needs_update);
    let lib3 = Library::open(Some(h3.path().to_str().unwrap()), false).unwrap();
    assert_eq!(lib3.search("nectar", &Filter::default()).unwrap().len(), 3);
    drop(lib3);
    // a damaged backup is refused before anything is written
    let bad = out.path().join("bad.zip");
    {
        use std::io::{Read, Write};
        let mut src = zip::ZipArchive::new(fs::File::open(&slim).unwrap()).unwrap();
        let mut w = zip::ZipWriter::new(fs::File::create(&bad).unwrap());
        let o = zip::write::SimpleFileOptions::default();
        for i in 0..src.len() {
            let mut f = src.by_index(i).unwrap();
            let mut buf = vec![];
            f.read_to_end(&mut buf).unwrap();
            let name = f.name().unwrap().into_owned();
            if name == "state/organize.json" {
                buf.extend_from_slice(b" tampered");
            }
            w.start_file(name, o).unwrap();
            w.write_all(&buf).unwrap();
        }
        w.finish().unwrap();
    }
    let h4 = tempfile::tempdir().unwrap();
    assert!(restore(&bad, h4.path()).is_err());
    assert!(fs::read_dir(h4.path()).unwrap().next().is_none());
    drop((h, d));
}

#[test]
fn verify_rebuilds_corrupt_index() {
    let (h, _d, mut lib) = setup(&corpus());
    let rep = verify(&mut lib, true).unwrap();
    assert!(rep.ok && !rep.rebuilt && rep.docs == 4);
    drop(lib);
    let home = h.path().to_str().unwrap();
    fs::write(
        h.path().join("index.sqlite"),
        b"this is not a database at all, just junk bytes",
    )
    .unwrap();
    assert!(verify_home(home, false).is_err());
    let (lib, rep) = verify_home(home, true).unwrap();
    assert!(rep.rebuilt && rep.ok, "{rep:?}");
    assert_eq!(lib.search("nectar", &Filter::default()).unwrap().len(), 3);
}

#[test]
fn stats_and_timeline() {
    let (_h, _d, lib) = setup(&[
        (
            "en.txt",
            "The cat is on the mat and the dog is in the house with the bird of the sea.",
        ),
        (
            "ro.txt",
            "Pisica este pe covor si cainele este in casa, pentru ca nu sunt la fel de mari.",
        ),
        (
            "fr.txt",
            "Le chat est dans la maison et le chien est sur le toit avec les enfants.",
        ),
    ]);
    let s = stats(&lib).unwrap();
    assert_eq!(s.docs, 3);
    assert_eq!(s.by_type[".txt"], 3);
    assert!(s.words_total > 30);
    assert_eq!(s.biggest.len(), 3);
    assert_eq!(
        (s.languages["en"], s.languages["ro"], s.languages["fr"]),
        (1, 1, 1),
        "{:?}",
        s.languages
    );
    assert_eq!(s.by_folder.values().sum::<usize>(), 3);
    let t = timeline(&lib).unwrap();
    assert_eq!(t.len(), 1);
    assert_eq!(t[0].count, 3);
    assert_eq!(t[0].month.len(), 7);
    assert_eq!(t[0].month[..4], s.by_year.keys().next().unwrap()[..]);
}

// ───────────────────────── adversarial ─────────────────────────

fn evil_zip(path: &std::path::Path, name: &str, body: &[u8]) {
    use sha1::{Digest, Sha1};
    use std::io::Write;
    let f = fs::File::create(path).unwrap();
    let mut z = zip::ZipWriter::new(f);
    let o = zip::write::SimpleFileOptions::default();
    z.start_file(format!("state/{name}"), o).unwrap();
    z.write_all(body).unwrap();
    let man = serde_json::json!({
        "format": "cartacolibri-backup", "version": 1, "app_version": "x", "created": "now",
        "includes_index": false,
        "files": [{"name": name, "size": body.len(), "sha1": hex::encode(Sha1::digest(body))}]
    });
    z.start_file("manifest.json", o).unwrap();
    z.write_all(man.to_string().as_bytes()).unwrap();
    z.finish().unwrap();
}

#[test]
fn backup_with_traversal_names_is_refused() {
    let t = tempfile::tempdir().unwrap();
    for name in [
        "../escape.txt",
        "a/../../escape.txt",
        "..\\escape.txt",
        "a\\..\\..\\escape.txt",
        "/abs.txt",
        "C:evil.txt",
        "a//b",
        "./x",
        "bad\0name",
        "new\nline",
    ] {
        let z = t.path().join("evil.zip");
        evil_zip(&z, name, b"pwned");
        let home = t.path().join("home");
        assert!(backup_verify(&z).is_err(), "{name:?} accepted by verify");
        assert!(restore(&z, &home).is_err(), "{name:?} accepted by restore");
        assert!(!t.path().join("escape.txt").exists());
    }
}

#[cfg(unix)]
#[test]
fn restore_never_writes_through_a_symlink_and_backup_survives_symlink_loops() {
    let t = tempfile::tempdir().unwrap();
    let outside = t.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    let home = t.path().join("home");
    fs::create_dir_all(&home).unwrap();
    std::os::unix::fs::symlink(&outside, home.join("sub")).unwrap();
    let z = t.path().join("b.zip");
    evil_zip(&z, "sub/owned.txt", b"pwned");
    assert!(restore(&z, &home).is_err());
    assert!(
        !outside.join("owned.txt").exists(),
        "wrote through a symlink"
    );
    // a symlink loop inside the home must not hang or overflow the stack in backup
    let (h, _d, lib) = setup(&corpus());
    std::os::unix::fs::symlink(h.path(), h.path().join("loop")).unwrap();
    std::os::unix::fs::symlink("/etc/hostname", h.path().join("leak")).unwrap();
    let out = t.path().join("ok.zip");
    let m = backup(&lib, &out, false).unwrap();
    assert!(
        m.files
            .iter()
            .all(|f| !f.name.starts_with("loop") && f.name != "leak")
    );
}

#[test]
fn damaged_state_file_is_kept_not_silently_replaced() {
    let (h, _d, lib) = setup(&corpus());
    let a = path_of(&lib, "a.txt");
    tag_add(&lib, &a, &["keep".into()]).unwrap();
    let f = h.path().join("organize.json");
    let good = fs::read_to_string(&f).unwrap();
    fs::write(&f, &good[..good.len() / 2]).unwrap(); // truncated by a crash
    let err = tag_add(&lib, &a, &["new".into()]).unwrap_err();
    assert!(err.contains("damaged"), "{err}");
    let kept = fs::read_dir(h.path())
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().contains("corrupt"))
        .count();
    assert_eq!(kept, 1, "the damaged file must be preserved");
    assert!(tag_add(&lib, &a, &["new".into()]).is_ok());
}

#[test]
fn concurrent_writers_lose_no_update() {
    let (h, _d, lib) = setup(&corpus());
    let a = path_of(&lib, "a.txt");
    drop(lib);
    let home = h.path().to_string_lossy().into_owned();
    let mut hs = vec![];
    for i in 0..8 {
        let (home, a) = (home.clone(), a.clone());
        hs.push(std::thread::spawn(move || {
            let lib = Library::open(Some(&home), false).unwrap();
            for j in 0..10 {
                tag_add(&lib, &a, &[format!("t{i}-{j}")]).unwrap();
            }
        }));
    }
    for t in hs {
        t.join().unwrap();
    }
    let lib = Library::open(Some(&home), false).unwrap();
    assert_eq!(tags_of(&lib, &a).len(), 80, "updates were lost");
    assert!(!h.path().join("organize.lock").exists());
}

#[test]
fn tags_notes_and_paths_are_bounded_and_normalised() {
    let (_h, _d, lib) = setup(&corpus());
    let a = path_of(&lib, "a.txt");
    let t = tag_add(
        &lib,
        &a,
        &[
            "  #Mixed   CASE\ttag\n".into(),
            "x".repeat(500),
            "\u{7}\u{1b}[31m".into(),
            "   ".into(),
            "#".into(),
        ],
    )
    .unwrap();
    assert!(t.contains(&"mixed case tag".to_string()), "{t:?}");
    assert!(
        t.iter()
            .all(|s| s.chars().count() <= TAG_MAX && !s.contains(char::is_control)),
        "{t:?}"
    );
    assert!(t.iter().all(|s| !s.trim().is_empty()));
    assert!(set_note(&lib, &a, &"n".repeat(NOTE_MAX + 1)).is_err());
    assert!(set_note(&lib, &a, &"n".repeat(NOTE_MAX)).is_ok());
    assert!(tag_add(&lib, "", &["x".into()]).is_err());
    assert!(set_star(&lib, "  ", true).is_err());
    // unicode case folding for tags
    tag_add(&lib, &a, &["ȘCOALĂ".into()]).unwrap();
    assert!(paths_with_tag(&lib, "școală").contains(&a));
}

#[test]
fn csv_cells_that_look_like_formulas_are_neutralised() {
    let it = |s: &str| ExportItem {
        path: "/p.txt".into(),
        title: s.into(),
        note: s.into(),
        snippet: s.into(),
        tags: vec![s.into()],
        ..Default::default()
    };
    for bad in [
        "=1+1",
        "+cmd|' /C calc'!A0",
        "-2+3",
        "@SUM(A1)",
        "\t=1",
        "\r=1",
    ] {
        let c = to_csv(&[it(bad)]);
        let line = c.lines().nth(1).unwrap();
        assert!(
            !line.starts_with(bad.chars().next().unwrap()) || bad.starts_with('\r'),
            "{line}"
        );
        for cell in line.split(',') {
            let cell = cell.trim_matches('"');
            assert!(!cell.starts_with(['=', '+', '-', '@']), "{bad:?} -> {line}");
        }
    }
    // numbers and ordinary text are untouched
    let c = to_csv(&[ExportItem {
        path: "/a b.txt".into(),
        title: "a".into(),
        size: 42,
        ..Default::default()
    }]);
    assert!(c.contains(",42,"), "{c}");
}

#[test]
fn export_zip_refuses_to_overwrite_a_source_document() {
    let (_h, d, lib) = setup(&corpus());
    let a = path_of(&lib, "a.txt");
    let before = fs::read_to_string(&a).unwrap();
    let hits = lib.search("nectar", &Filter::default()).unwrap();
    let items = items_from_hits(&lib, &hits);
    assert!(export_zip(&items, std::path::Path::new(&a)).is_err());
    assert_eq!(fs::read_to_string(&a).unwrap(), before);
    assert!(export_zip(&items, d.path()).is_err());
}

#[test]
fn import_json_keeps_local_notes_and_bounds_tags() {
    let (_h, _d, lib) = setup(&corpus());
    let a = path_of(&lib, "a.txt");
    set_note(&lib, &a, "mine").unwrap();
    let json =
        serde_json::json!({"format":"cartacolibri-export","version":1,"created":"x","items":[
            {"path": a, "title":"t", "note":"theirs", "tags":["A".repeat(300)]},
            {"path": "", "title":"t", "note":"ignored"}
        ]})
        .to_string();
    import_json(&lib, &json).unwrap();
    let m = meta_of(&lib, &a);
    assert!(
        m.note.starts_with("mine") && m.note.contains("theirs"),
        "{:?}",
        m.note
    );
    assert!(m.tags.iter().all(|t| t.chars().count() <= TAG_MAX));
    assert!(!all_meta(&lib).contains_key(""));
}

#[test]
fn related_with_huge_k_does_not_overflow() {
    let (_h, _d, lib) = setup(&corpus());
    let id = lib.documents(None).unwrap()[0].id;
    let _ = related(&lib, id, usize::MAX).unwrap();
}
