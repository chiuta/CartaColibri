use cartacolibri::digest::run_once;
use cartacolibri::library::Library;
use cartacolibri::organize::{self, SavedSearch};

#[test]
fn digest_reports_only_what_is_new() {
    let home = tempfile::tempdir().unwrap();
    let docs = tempfile::tempdir().unwrap();
    std::fs::write(docs.path().join("a.txt"), "the budget was approved").unwrap();
    let mut lib = Library::open(Some(home.path().to_str().unwrap()), false).unwrap();
    lib.add_root(docs.path().to_str().unwrap()).unwrap();
    lib.update(None, &mut |_| {}).unwrap();
    organize::saved_set(
        &lib,
        SavedSearch {
            name: "money".into(),
            query: "budget".into(),
            ..Default::default()
        },
    )
    .unwrap();
    // first run only records what already matches
    assert!(run_once(&lib, "money", None, false).unwrap().is_none());
    assert!(run_once(&lib, "money", None, false).unwrap().is_none());
    std::fs::write(docs.path().join("b.txt"), "a new budget draft").unwrap();
    lib.update(None, &mut |_| {}).unwrap();
    let md = run_once(&lib, "money", None, false).unwrap().unwrap();
    assert!(md.contains("b.txt") && !md.contains("a.txt") && md.contains("1 new"));
    assert!(run_once(&lib, "money", None, false).unwrap().is_none());
    assert!(run_once(&lib, "nope", None, false).is_err());
}
