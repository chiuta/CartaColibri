use std::process::{Command, Output};

fn run(home: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cartacolibri"))
        .args(args)
        .env("CARTACOLIBRI_HOME", home)
        .env("NO_COLOR", "1")
        .env_remove("CARTA_LANG")
        .env("LANG", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn no_args_shows_guided_start() {
    let h = tempfile::tempdir().unwrap();
    let o = run(h.path(), &[]);
    assert!(o.status.success());
    let s = out(&o);
    assert!(s.contains("cartacolibri start"), "{s}");
    assert!(!s.contains('\x1b'), "no colour when piped");
    let o = run(h.path(), &["--lang", "ro"]);
    assert!(out(&o).contains("Bun venit"));
}

#[test]
fn no_args_with_library_shows_status() {
    let h = tempfile::tempdir().unwrap();
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("a.txt"), "hello colibri world").unwrap();
    let o = run(
        h.path(),
        &[
            "start",
            "--folder",
            d.path().to_str().unwrap(),
            "--no-serve",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let s = out(&run(h.path(), &[]));
    assert!(s.contains("1 document(s) in 1 folder(s)"), "{s}");
    assert!(s.contains("cartacolibri doctor"));
    let j = out(&run(h.path(), &["search", "colibri", "--json"]));
    assert!(j.trim_start().starts_with('['), "{j}");
    assert!(j.contains("a.txt"));
}

#[test]
fn doctor_reports_and_exits_zero() {
    let h = tempfile::tempdir().unwrap();
    // an unreachable gateway is only a warning
    let o = run(
        h.path(),
        &["doctor", "--url", "http://127.0.0.1:9", "--port", "0"],
    );
    let s = out(&o);
    assert!(o.status.success(), "{s}");
    assert!(s.contains("OK") && s.contains("WARN"), "{s}");
    assert!(s.contains("gateway") && s.contains("fix:"), "{s}");
}

#[test]
fn friendly_error_for_missing_folder() {
    let h = tempfile::tempdir().unwrap();
    let o = run(h.path(), &["add", "/definitely/not/here"]);
    assert!(!o.status.success());
    let e = err(&o);
    assert!(e.contains("can't find the folder"), "{e}");
    assert!(e.contains("Try this:"), "{e}");
}

#[test]
fn no_results_hint_and_completions() {
    let h = tempfile::tempdir().unwrap();
    let o = run(h.path(), &["search", "zzzz"]);
    let s = out(&o);
    assert!(s.contains("No matches") && s.contains("Try this:"), "{s}");
    let o = run(h.path(), &["completions", "bash"]);
    assert!(out(&o).contains("cartacolibri"));
}
