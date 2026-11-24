use cartacolibri::library::{Filter, Library, Progress};
use std::fs;
use std::path::Path;

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

fn n(lib: &Library, q: &str) -> usize {
    lib.search(q, &Filter::default()).unwrap().len()
}

#[test]
fn typo_gets_corrected_and_suggested() {
    let (_h, _d, lib) = setup(&[(
        "a.txt",
        "Colibri este o pasare mica. Informatii despre colibri.",
    )]);
    let s = lib.suggest("colbri");
    assert_eq!(s.first().map(String::as_str), Some("colibri"), "{s:?}");
    let r = lib.search_friendly("colbri", &Filter::default()).unwrap();
    assert_eq!(r.corrected.as_deref(), Some("colibri"));
    assert_eq!(r.hits.len(), 1);
    // known words: nothing to correct
    let r = lib.search_friendly("colibri", &Filter::default()).unwrap();
    assert!(r.corrected.is_none() && r.suggestions.is_empty() && r.hits.len() == 1);
    // keeps the other words of the query
    let s = lib.suggest("pasare colbri");
    assert_eq!(s[0], "pasare colibri");
    // transposition
    assert_eq!(lib.suggest("clobiri")[0], "colibri");
    // total nonsense: no crash, nothing
    let r = lib
        .search_friendly("zzzqqqxxx", &Filter::default())
        .unwrap();
    assert!(r.hits.is_empty() && r.corrected.is_none());
}

#[test]
fn diacritics_every_language() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "ro",
            "Știința și țara noastră, școală, mâncăruri.",
            &[
                "stiinta",
                "tara",
                "scoala",
                "ȘTIINȚA",
                "ştiinţa",
                "ţara",
                "mancaruri",
            ],
        ),
        (
            "de",
            "Größe, Müller, schön, Mädchen, über.",
            &["mullers", "schon", "madchen", "uber", "müller", "SCHÖN"],
        ),
        (
            "fr",
            "Élève à l'école, où été, très français.",
            &["eleve", "ete", "tres", "francais", "ÉLÈVE"],
        ),
        (
            "es",
            "El niño español comió mañana piñata.",
            &["nino", "espanol", "manana", "piñata"],
        ),
        (
            "pt",
            "A coração, informação, ações, pão, mães.",
            &["coracao", "informacao", "acoes", "pao", "maes", "AÇÕES"],
        ),
        (
            "hu",
            "Erőforrás, tűz, füst, szőlő, hűtő.",
            &["eroforras", "tuz", "fust", "szolo", "hűtő", "huto"],
        ),
        (
            "it",
            "La città è perché già più virtù.",
            &["citta", "perche", "gia", "piu", "virtu"],
        ),
        (
            "en",
            "The naïve café and résumé of Zoë.",
            &["naive", "cafe", "resume", "zoe", "CAFÉ"],
        ),
    ];
    for (lang, text, queries) in cases {
        let (_h, _d, lib) = setup(&[("t.txt", text)]);
        for q in *queries {
            assert_eq!(n(&lib, q), 1, "{lang}: '{q}' should match");
        }
        // and accented query against an unaccented document
        let plain = cartacolibri::library::fold(text);
        let (_h, _d, lib2) = setup(&[("t.txt", &plain)]);
        let q0 = text
            .split_whitespace()
            .next()
            .unwrap()
            .trim_matches(|c: char| !c.is_alphanumeric());
        assert_eq!(
            n(&lib2, q0),
            1,
            "{lang}: accented query '{q0}' vs plain doc"
        );
    }
}

#[test]
fn plurals_and_suffixes_fall_back_to_stems() {
    let (_h, _d, lib) = setup(&[
        ("e.txt", "The house was old."),
        ("r.txt", "Casa veche din sat."),
        ("g.txt", "Das Haus am See."),
        ("f.txt", "La maison bleue."),
    ]);
    assert_eq!(n(&lib, "houses"), 1);
    assert_eq!(n(&lib, "casele"), 1);
    let _ = n(&lib, "Häuser Haus"); // smoke: mixed accents must not crash
    assert_eq!(n(&lib, "maisons"), 1);
}

#[test]
fn snippets_mark_words_trim_sentences_and_give_page() {
    let filler = "Aceasta este o propozitie lunga fara legatura cu subiectul nostru principal. ";
    let text = format!(
        "{}{}Colibrii bat din aripi de cincizeci de ori pe secunda. Urmeaza alta propozitie fara importanta pentru cautare.",
        filler.repeat(3),
        filler.repeat(2)
    );
    let (_h, _d, lib) = setup(&[("c.txt", &text)]);
    let hits = lib.search("colibrii aripi", &Filter::default()).unwrap();
    let s = &hits[0].snippet;
    assert!(s.contains("[Colibrii]") && s.contains("[aripi]"), "{s}");
    assert!(
        s.starts_with("Colibrii") || s.starts_with("… Colibrii") || s.contains("[Colibrii] bat"),
        "{s}"
    );
    assert!(s.chars().count() < 400, "{s}");
    assert_eq!(hits[0].page, None);
    // accent-insensitive marking
    let (_h, _d, lib) = setup(&[("d.txt", "Școala noastră e frumoasă.")]);
    let h = lib.search("scoala", &Filter::default()).unwrap();
    assert!(h[0].snippet.contains("[Școala]"), "{}", h[0].snippet);
}

#[test]
fn resilient_indexing_with_plain_reasons() {
    let home = tempfile::tempdir().unwrap();
    let docs = tempfile::tempdir().unwrap();
    let d = docs.path();
    fs::write(d.join("ok.txt"), "continut bun despre colibri").unwrap();
    fs::write(d.join("broken.docx"), b"not a zip at all").unwrap();
    fs::write(d.join("broken.xlsx"), b"PK\x03\x04garbage").unwrap();
    fs::write(d.join("broken.pdf"), b"%PDF-1.4 garbage garbage").unwrap();
    fs::write(d.join("fake.txt"), vec![0u8; 5000]).unwrap();
    fs::write(d.join("empty.md"), b"").unwrap();
    let long = format!("{}.txt", "x".repeat(300));
    let _ = fs::write(d.join(&long), "nume prea lung"); // may be refused by the OS: fine
    fs::create_dir(d.join("sub")).unwrap();
    fs::write(d.join("sub/in.txt"), "inauntru").unwrap();
    #[cfg(unix)]
    {
        let _ = std::os::unix::fs::symlink(d, d.join("sub/loop"));
        let _ = std::os::unix::fs::symlink(d.join("sub"), d.join("sub/self"));
        let _ = std::os::unix::fs::symlink(d.join("nowhere.txt"), d.join("dangling.txt"));
    }
    let mut lib = Library::open(Some(home.path().to_str().unwrap()), false).unwrap();
    let mut last = Progress::default();
    let mut calls = 0;
    let st = lib
        .update_progress(
            Some(&[d.to_string_lossy().into_owned()]),
            &mut |p| {
                last = p.clone();
                calls += 1;
            },
            &mut |_| {},
        )
        .unwrap();
    assert!(st.failed >= 4, "{st:?}");
    assert!(st.added >= 2);
    assert!(calls >= 6 && last.seen == last.total, "{last:?}");
    assert_eq!(st.seen, last.seen);
    assert_eq!(n(&lib, "colibri"), 1);
    let probs = lib.problems();
    let why = |name: &str| -> String {
        probs
            .iter()
            .find(|p| p["path"].as_str().unwrap().ends_with(name))
            .unwrap_or_else(|| panic!("no problem for {name}: {probs:?}"))["reason"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert!(
        why("broken.docx").contains("damaged"),
        "{}",
        why("broken.docx")
    );
    assert!(why("broken.xlsx").contains("damaged"));
    assert!(why("broken.pdf").contains("PDF"), "{}", why("broken.pdf"));
    assert!(why("fake.txt").contains("binary"));
    assert!(why("empty.md").contains("No readable text"));
    // second run: same problems, nothing crashes, nothing re-added
    let st2 = lib.update(None, &mut |_| {}).unwrap();
    assert_eq!(st2.added, 0);
}

#[cfg(unix)]
#[test]
fn unreadable_file_and_folder_do_not_stop_indexing() {
    use std::os::unix::fs::PermissionsExt;
    let home = tempfile::tempdir().unwrap();
    let docs = tempfile::tempdir().unwrap();
    let d = docs.path();
    fs::write(d.join("a.txt"), "alfa").unwrap();
    fs::write(d.join("secret.txt"), "beta").unwrap();
    fs::create_dir(d.join("locked")).unwrap();
    fs::write(d.join("locked/x.txt"), "gama").unwrap();
    fs::set_permissions(d.join("secret.txt"), fs::Permissions::from_mode(0o000)).unwrap();
    fs::set_permissions(d.join("locked"), fs::Permissions::from_mode(0o000)).unwrap();
    let mut lib = Library::open(Some(home.path().to_str().unwrap()), false).unwrap();
    let st = lib
        .update(Some(&[d.to_string_lossy().into_owned()]), &mut |_| {})
        .unwrap();
    assert!(st.added >= 1);
    // (running as root can read everything: then there is simply nothing to report)
    for p in lib.problems() {
        let r = p["reason"].as_str().unwrap();
        assert!(
            r.contains("permission") || r.contains("No permission"),
            "{r}"
        );
    }
    fs::set_permissions(d.join("secret.txt"), fs::Permissions::from_mode(0o644)).unwrap();
    fs::set_permissions(d.join("locked"), fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn duplicates_by_content() {
    let (_h, _d, lib) = setup(&[
        ("a.txt", "acelasi continut"),
        ("b.txt", "acelasi continut"),
        ("c.txt", "acelasi continut"),
        ("d.txt", "altceva"),
        ("e.txt", "alta pereche"),
        ("f.txt", "alta pereche"),
    ]);
    let dups = lib.duplicates();
    assert_eq!(dups.len(), 2, "{dups:?}");
    assert!(dups.iter().any(|g| g.len() == 3));
    assert!(dups.iter().any(|g| g.len() == 2));
    assert!(dups.iter().flatten().all(|p| !p.ends_with("d.txt")));
}

#[test]
fn reindex_of_unchanged_2000_files_is_fast() {
    let home = tempfile::tempdir().unwrap();
    let docs = tempfile::tempdir().unwrap();
    for i in 0..2000 {
        let sub = docs.path().join(format!("d{}", i % 20));
        fs::create_dir_all(&sub).unwrap();
        fs::write(
            sub.join(format!("f{i}.txt")),
            format!("document numarul {i} cu text"),
        )
        .unwrap();
    }
    let mut lib = Library::open(Some(home.path().to_str().unwrap()), false).unwrap();
    let root = [docs.path().to_string_lossy().into_owned()];
    let st = lib.update(Some(&root), &mut |_| {}).unwrap();
    assert_eq!(st.added, 2000);
    let t = std::time::Instant::now();
    let st = lib.update(Some(&root), &mut |_| {}).unwrap();
    let el = t.elapsed();
    assert_eq!((st.unchanged, st.added, st.updated), (2000, 0, 0));
    assert!(el.as_secs_f64() < 5.0, "took {el:?}");
    assert!(Path::new(&root[0]).exists());
}

// ---------------------------------------------------------------- adversarial audit

fn root_of(d: &tempfile::TempDir) -> [String; 1] {
    [d.path().to_string_lossy().into_owned()]
}

#[test]
fn hyphenated_and_indic_words_are_found() {
    let (_h, _d, lib) = setup(&[
        ("a.txt", "Send the e-mail to the user-agent today."),
        ("b.txt", "\u{939}\u{93f}\u{928}\u{94d}\u{926}\u{940} bhasha"),
    ]);
    assert_eq!(n(&lib, "user-agent"), 1, "hyphen must act as a word break");
    assert_eq!(n(&lib, "e-mail"), 1);
    assert_eq!(
        n(&lib, "\u{939}\u{93f}\u{928}\u{94d}\u{926}\u{940}"),
        1,
        "combining marks"
    );
}

#[test]
fn hostile_queries_never_panic_or_error() {
    let (_h, _d, lib) = setup(&[("a.txt", "alpha beta gamma delta [x] \"quoted\" text")]);
    let big = "word ".repeat(200_000);
    let qs: Vec<String> = [
        "\"",
        "\"\"\"\"",
        "NEAR(",
        "alpha NEAR/2 beta",
        "NEAR(alpha beta, 2)",
        "text:alpha",
        "{text}: alpha",
        "*",
        "**",
        "(",
        ")",
        "((alpha)",
        "-",
        "^alpha",
        "alpha AND",
        "NOT",
        "OR OR OR",
        "alpha NOT",
        "'; DROP TABLE docs;--",
        "alpha\0beta",
        "\u{202e}alpha",
        "\"alpha",
        "alpha\"",
        "col:*",
        "alpha*beta",
        "AND alpha",
        "a b c d e f g h i j k l",
    ]
    .iter()
    .map(|s| s.to_string())
    .chain([big])
    .collect();
    for q in qs {
        let t = std::time::Instant::now();
        let f = Filter::default();
        let r = lib.search_friendly(&q, &f);
        assert!(
            r.is_ok(),
            "search_friendly({:?}) failed: {:?}",
            &q[..q.len().min(30)],
            r.err()
        );
        assert!(
            lib.search(&q, &f).is_ok(),
            "search({:?})",
            &q[..q.len().min(30)]
        );
        let _ = lib.suggest(&q);
        assert!(
            t.elapsed().as_secs() < 10,
            "query of {} bytes took {:?}",
            q.len(),
            t.elapsed()
        );
    }
    assert_eq!(lib.documents(None).unwrap().len(), 1, "no SQL got through");
}

#[test]
fn files_that_stop_qualifying_leave_the_index() {
    let home = tempfile::tempdir().unwrap();
    let docs = tempfile::tempdir().unwrap();
    for f in ["a.txt", "b.txt", "c.txt"] {
        fs::write(docs.path().join(f), format!("unique{f} content")).unwrap();
    }
    let mut lib = Library::open(Some(home.path().to_str().unwrap()), false).unwrap();
    let root = root_of(&docs);
    lib.update(Some(&root), &mut |_| {}).unwrap();
    assert_eq!(lib.documents(None).unwrap().len(), 3);
    fs::rename(docs.path().join("a.txt"), docs.path().join(".a.txt")).unwrap(); // hidden now
    fs::rename(docs.path().join("b.txt"), docs.path().join("b.xyz")).unwrap(); // unsupported
    fs::rename(docs.path().join("c.txt"), docs.path().join("c.txt~")).unwrap(); // backup file
    lib.update(Some(&root), &mut |_| {}).unwrap();
    let left: Vec<String> = lib
        .documents(None)
        .unwrap()
        .into_iter()
        .map(|d| d.path)
        .collect();
    assert!(left.is_empty(), "stale entries kept: {left:?}");
}

#[test]
fn sha1_collision_is_not_reported_as_duplicate() {
    let (h, d, mut lib) = setup(&[
        ("a.txt", "first body"),
        ("b.txt", "second body!"),
        ("c.txt", "first body"),
    ]);
    // simulate a SHA-1 collision (e.g. the SHAttered PDFs): same hash recorded for different bytes
    {
        let db = rusqlite::Connection::open(h.path().join("index.sqlite")).unwrap();
        db.execute("UPDATE docs SET sha1='deadbeef'", []).unwrap();
    }
    let _ = &mut lib;
    let dups = lib.duplicates();
    let flat: Vec<&String> = dups.iter().flatten().collect();
    assert!(
        dups.len() == 1 && flat.len() == 2 && flat.iter().all(|p| !p.ends_with("b.txt")),
        "different content reported as duplicate: {dups:?} in {:?}",
        d.path()
    );
}

#[test]
fn corrupt_index_does_not_panic() {
    let home = tempfile::tempdir().unwrap();
    fs::write(
        home.path().join("index.sqlite"),
        b"this is not a database at all, really",
    )
    .unwrap();
    let r = std::panic::catch_unwind(|| {
        let h = home.path().to_str().unwrap().to_string();
        if let Ok(lib) = Library::open(Some(&h), false) {
            let _ = lib.roots();
            let _ = lib.problems();
            let _ = lib.duplicates();
            let _ = lib.documents(None);
            let _ = lib.text_of(1);
            let _ = lib.search("x", &Filter::default());
        }
    });
    assert!(r.is_ok(), "panicked on a corrupt index");
}

#[test]
fn file_that_becomes_unreadable_keeps_no_stale_text() {
    let home = tempfile::tempdir().unwrap();
    let docs = tempfile::tempdir().unwrap();
    fs::write(docs.path().join("a.docx"), b"PK not really").unwrap();
    let mut lib = Library::open(Some(home.path().to_str().unwrap()), false).unwrap();
    let st = lib.update(Some(&root_of(&docs)), &mut |_| {}).unwrap();
    assert_eq!(st.failed, 1);
    assert!(lib.documents(None).unwrap().is_empty());
}

#[test]
fn odd_file_names_are_indexed_and_searchable() {
    let home = tempfile::tempdir().unwrap();
    let docs = tempfile::tempdir().unwrap();
    let long = format!("{}.txt", "\u{e9}".repeat(120)); // 240 bytes
    for name in [
        "new\nline.txt",
        "quote\"'; DROP TABLE docs;--.txt",
        "con.txt",
        "aux.md",
        "\u{202e}rtl.txt",
        "emoji \u{1f600} \u{4e2d}\u{6587}.txt",
        "percent%20and%00.txt",
        long.as_str(),
    ] {
        let _ = fs::write(docs.path().join(name), "needleword inside");
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let bad = std::ffi::OsStr::from_bytes(b"bad\xff\xfename.txt");
        let _ = fs::write(docs.path().join(bad), "needleword invalid utf8 name");
        std::os::unix::fs::symlink(docs.path(), docs.path().join("loop")).unwrap();
        std::os::unix::fs::symlink("nowhere", docs.path().join("dangling.txt")).unwrap();
    }
    let mut lib = Library::open(Some(home.path().to_str().unwrap()), false).unwrap();
    let root = root_of(&docs);
    for _ in 0..2 {
        lib.update(Some(&root), &mut |_| {}).unwrap();
    }
    assert!(n(&lib, "needleword") >= 6, "got {}", n(&lib, "needleword"));
    let hits = lib.search("needleword", &Filter::default()).unwrap();
    for h in hits {
        assert!(!h.snippet.is_empty());
    }
}

#[test]
fn edit_is_seen_even_if_the_clock_went_backwards() {
    let home = tempfile::tempdir().unwrap();
    let docs = tempfile::tempdir().unwrap();
    let f = docs.path().join("a.txt");
    fs::write(&f, "alpha one").unwrap();
    let mut lib = Library::open(Some(home.path().to_str().unwrap()), false).unwrap();
    let root = root_of(&docs);
    lib.update(Some(&root), &mut |_| {}).unwrap();
    // a rewrite with another size is always seen, whatever the clock says
    let t0 = fs::metadata(&f).unwrap().modified().unwrap();
    fs::write(&f, "beta one two").unwrap();
    let past = t0 - std::time::Duration::from_secs(86_400 * 400); // clock went backwards
    fs::File::options()
        .write(true)
        .open(&f)
        .unwrap()
        .set_modified(past)
        .unwrap();
    let st = lib.update(Some(&root), &mut |_| {}).unwrap();
    assert_eq!(st.updated, 1, "{st:?}");
    assert_eq!(n(&lib, "beta"), 1);
    assert_eq!(n(&lib, "alpha"), 0);
}
