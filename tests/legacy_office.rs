//! Old binary Office files (.doc/.xls/.ppt) are read through a converter that is already on the
//! computer; without one they are left alone instead of failing one by one.
use cartacolibri::library::{Filter, Library};
use std::os::unix::fs::PermissionsExt;

fn tool(dir: &std::path::Path, name: &str, script: &str) {
    let p = dir.join(name);
    std::fs::write(&p, format!("#!/bin/sh\n{script}\n")).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn legacy_files_need_a_converter_and_use_it_when_present() {
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("old.doc"), b"\xd0\xcf\x11\xe0 binary").unwrap();
    std::fs::write(docs.join("old.xls"), b"\xd0\xcf\x11\xe0 binary").unwrap();
    std::fs::write(docs.join("old.ppt"), b"\xd0\xcf\x11\xe0 binary").unwrap();
    std::fs::write(docs.join("new.txt"), "plain").unwrap();
    let d = docs.to_string_lossy().into_owned();
    let empty_bin = t.path().join("nobin");
    std::fs::create_dir_all(&empty_bin).unwrap();
    // 1. no converter on PATH: the old files are skipped quietly, nothing fails
    unsafe { std::env::set_var("PATH", &empty_bin) };
    let mut lib = Library::open(Some(&t.path().join("h1").to_string_lossy()), false).unwrap();
    let st = lib
        .update(Some(std::slice::from_ref(&d)), &mut |_| {})
        .unwrap();
    assert_eq!(st.failed, 0, "{st:?}");
    assert_eq!(lib.documents(None).unwrap().len(), 1);
    // 2. converters installed
    let bin = t.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    tool(
        &bin,
        "antiword",
        "echo 'Warranty terms of the old contract'",
    );
    tool(&bin, "xls2csv", "printf 'item,price\\nchair,40\\n'");
    tool(&bin, "catppt", "echo 'Slide: roadmap for next year'");
    unsafe { std::env::set_var("PATH", &bin) };
    let mut lib = Library::open(Some(&t.path().join("h2").to_string_lossy()), false).unwrap();
    let st = lib
        .update(Some(std::slice::from_ref(&d)), &mut |_| {})
        .unwrap();
    assert_eq!(st.failed, 0, "{st:?}");
    assert_eq!(lib.documents(None).unwrap().len(), 4);
    let f = Filter::default();
    assert_eq!(lib.search("warranty", &f).unwrap().len(), 1);
    assert_eq!(lib.search("chair", &f).unwrap().len(), 1);
    assert_eq!(lib.search("roadmap", &f).unwrap().len(), 1);
    // 3. a converter that fails or hangs is a clean problem, not a crash
    tool(&bin, "antiword", "echo 'cannot parse' >&2; exit 3");
    std::fs::write(docs.join("old.doc"), b"\xd0\xcf\x11\xe0 changed").unwrap();
    let st = lib.update(Some(&[d]), &mut |_| {}).unwrap();
    assert!(st.failed >= 1, "{st:?}");
}
