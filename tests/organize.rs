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
    let m = backup(&lib, &slim, false).unwrap();
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
