//! Filters must see the whole library, not only the best passages of the whole library, and a
//! search made only of filters must list what the filters select.
use cartacolibri::library::{Filter, Library};

fn lib_with(n: usize) -> (tempfile::TempDir, Library) {
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    std::fs::create_dir_all(docs.join("big")).unwrap();
    std::fs::create_dir_all(docs.join("rare")).unwrap();
    for i in 0..n {
        // many short, strongly matching passages: they outrank the one below by far
        std::fs::write(
            docs.join("big").join(format!("d{i}.txt")),
            "alpha alpha alpha alpha alpha alpha alpha alpha",
        )
        .unwrap();
    }
    std::fs::write(
        docs.join("rare").join("needle.md"),
        "A long passage about many other things and, somewhere in the middle, alpha once. INV-2025-0042 is the invoice.",
    )
    .unwrap();
    let home = t.path().join("home").to_string_lossy().into_owned();
    let mut lib = Library::open(Some(&home), false).unwrap();
    lib.update(Some(&[docs.to_string_lossy().into_owned()]), &mut |_| {})
        .unwrap();
    (t, lib)
}

#[test]
fn a_filter_sees_documents_beyond_the_best_two_thousand_passages() {
    let (_t, lib) = lib_with(2300);
    let hits = lib.search("alpha path:rare", &Filter::default()).unwrap();
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].path.ends_with("needle.md"));
    let hits = lib.search("alpha ext:md", &Filter::default()).unwrap();
    assert_eq!(hits.len(), 1);
}

#[test]
fn a_search_of_only_filters_lists_what_they_select() {
    let (_t, lib) = lib_with(30);
    let by_ext = lib.search("ext:md", &Filter::default()).unwrap();
    assert_eq!(by_ext.len(), 1);
    let by_re = lib.search(r"re:INV-\d{4}-\d+", &Filter::default()).unwrap();
    assert_eq!(by_re.len(), 1);
    assert!(by_re[0].path.ends_with("needle.md"));
    assert!(lib.search("path:big", &Filter::default()).unwrap().len() >= 30);
    // nothing at all is still an error, and a broken regex a clean one
    assert!(lib.search("   ", &Filter::default()).is_err());
    assert!(lib.search("re:[", &Filter::default()).is_err());
}
