use cartacolibri::library::{Filter, Library};
use std::fs;

fn home(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let b = std::env::temp_dir().join(format!("cc-forget-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&b);
    fs::create_dir_all(b.join("docs")).unwrap();
    (b.join("home"), b.join("docs"))
}

#[test]
fn forgotten_files_stay_out_after_a_new_index_and_can_come_back() {
    let (h, d) = home("a");
    fs::write(d.join("keep.txt"), "alpha keep").unwrap();
    fs::write(d.join("drop.txt"), "alpha drop").unwrap();
    let mut lib = Library::open(Some(h.to_str().unwrap()), false).unwrap();
    lib.add_root(d.to_str().unwrap()).unwrap();
    lib.update(None, &mut |_| {}).unwrap();
    let ids = |l: &Library| l.search("alpha", &Filter::default()).unwrap().len();
    assert_eq!(ids(&lib), 2);
    let doc = lib
        .documents(None)
        .unwrap()
        .into_iter()
        .find(|x| x.path.ends_with("drop.txt"))
        .unwrap();
    assert_eq!(lib.forget_docs(&[doc.id]).unwrap(), 1);
    assert_eq!(ids(&lib), 1);
    lib.update(None, &mut |_| {}).unwrap();
    assert_eq!(ids(&lib), 1, "a re-index must not bring it back");
    assert!(
        fs::exists(d.join("drop.txt")).unwrap(),
        "the file itself is never touched"
    );
    assert_eq!(lib.excluded().len(), 1);
    assert!(lib.unforget(&doc.path).unwrap());
    lib.update(None, &mut |_| {}).unwrap();
    assert_eq!(ids(&lib), 2);
}

#[cfg(unix)]
#[test]
fn a_scan_is_read_through_legatorocr_when_it_is_installed() {
    use std::os::unix::fs::PermissionsExt;
    let (h, d) = home("b");
    let bin = d.join("legatorocr");
    fs::write(
        &bin,
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo \"legatorocr 0.0-test\"; exit 0; fi\nwhile [ $# -gt 0 ]; do if [ \"$1\" = \"--txt\" ]; then shift; out=\"$1\"; fi; shift; done\nprintf 'harbourmaster scanned page' > \"$out\"\n",
    )
    .unwrap();
    fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(d.join("page.png"), b"not really a png").unwrap();
    // one test process, one setting: this test is the only one that reads the variable
    unsafe { std::env::set_var("CARTACOLIBRI_LEGATOROCR", &bin) };
    assert_eq!(cartacolibri::extract::ocr_engine_name(), Some("legatorocr"));
    let mut lib = Library::open(Some(h.to_str().unwrap()), true).unwrap();
    lib.add_root(d.to_str().unwrap()).unwrap();
    lib.update(None, &mut |_| {}).unwrap();
    let hits = lib.search("harbourmaster", &Filter::default()).unwrap();
    assert_eq!(hits.len(), 1, "{hits:?}");
}
