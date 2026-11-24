use cartacolibri::carta::{
    self, Backend, ChatSession, Options, RunExtra, Source, detect_lang, mark_unsupported,
    not_found_msg, to_docx_text, to_html, verify_citations,
};
use cartacolibri::library::Library;

struct Fx {
    _t: tempfile::TempDir,
    lib: Library,
}

fn fixture() -> Fx {
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(
        docs.join("alpha.txt"),
        "Hummingbirds are small birds that hover in flight. The species was catalogued in 1758 by Linnaeus.\nThe rent is 450 euro per month. Maria is responsible for the payment schedule. The team must review the contract before 2025.",
    )
    .unwrap();
    std::fs::write(
        docs.join("beta.txt"),
        "# Notes\nHummingbirds are tiny animals with fast wings. The rent is 600 euro per month.\nThe lease was signed in 2019 and renewed in 2023. Photosynthesis is the process plants use to make sugar.",
    )
    .unwrap();
    let home = t.path().join("home").to_string_lossy().into_owned();
    let mut lib = Library::open(Some(&home), false).unwrap();
    lib.update(Some(&[docs.to_string_lossy().into_owned()]), &mut |_| {})
        .unwrap();
    Fx { _t: t, lib }
}

fn run(fx: &Fx, task: &str, o: &Options) -> String {
    let docs = fx.lib.documents(None).unwrap();
    carta::run(&fx.lib, task, &docs, &Backend::mock(), o, &mut |_| {}).unwrap()
}

#[test]
fn every_task_is_deterministic_and_cited() {
    let fx = fixture();
    for t in carta::TASKS {
        let o = Options {
            request: if *t == "synthesize" || *t == "translate" {
                if *t == "translate" {
                    "Romanian"
                } else {
                    "rent"
                }
            } else {
                ""
            },
            fields: if *t == "extract" {
                vec!["rent".into()]
            } else {
                vec![]
            },
            ..Default::default()
        };
        let a = run(&fx, t, &o);
        let b = run(&fx, t, &o);
        assert_eq!(a, b, "{t} not deterministic");
        assert!(a.contains("**Sources**"), "{t}: {a}");
        if !["extract"].contains(t) {
            assert!(a.contains("[S1]") || a.contains("[S2]"), "{t}: {a}");
        }
    }
}

#[test]
fn task_shapes() {
    let fx = fixture();
    let d = Options::default();
    let c = run(&fx, "compare", &d);
    assert!(
        c.contains("| Source |") && c.contains("Similarities"),
        "{c}"
    );
    let t = run(&fx, "timeline", &d);
    let (i1758, i2025) = (t.find("1758").unwrap(), t.find("2025").unwrap());
    assert!(i1758 < i2025 && t.contains("2019"), "{t}");
    assert!(run(&fx, "glossary", &d).contains("**Photosynthesis**"));
    assert!(run(&fx, "outline", &d).contains("1. **alpha.txt**"));
    let a = run(&fx, "actions", &d);
    assert!(a.contains("| Type |") && a.contains("Maria"), "{a}");
    assert!(run(&fx, "faq", &d).contains("**Q1."));
    let f = run(&fx, "flashcards", &d);
    assert!(
        f.contains("**Q:**") && f.contains("_____") && f.contains("Source: [S"),
        "{f}"
    );
    let q = run(&fx, "quiz", &d);
    assert!(q.contains("Answer: ") && q.contains("A)"), "{q}");
    let e = run(&fx, "explain", &d);
    assert!(e.contains("In plain words"));
    let x = run(&fx, "contradictions", &d);
    assert!(
        x.contains("450") && x.contains("600") && x.contains("vs"),
        "{x}"
    );
    let tr = run(
        &fx,
        "translate",
        &Options {
            request: "Romanian",
            ..Default::default()
        },
    );
    assert!(tr.contains("Romanian") && tr.contains("[S1]"), "{tr}");
    let docs = fx.lib.documents(None).unwrap();
    assert!(
        carta::run(
            &fx.lib,
            "translate",
            &docs,
            &Backend::mock(),
            &d,
            &mut |_| {}
        )
        .is_err()
    );
}

#[test]
fn truncation_note_and_fallback() {
    let fx = fixture();
    let docs = fx.lib.documents(None).unwrap();
    let x = RunExtra {
        max_chars: Some(10),
        ..Default::default()
    };
    let t = carta::run_ex(
        &fx.lib,
        "outline",
        &docs,
        &Backend::mock(),
        &Options::default(),
        &x,
        &mut |_| {},
    )
    .unwrap();
    assert!(t.contains("truncated: 1 docs skipped"), "{t}");
    let mut be = Backend::gateway("http://127.0.0.1:9", None).with_fallback(true);
    be.timeout = std::time::Duration::from_secs(2);
    be.retries = 0;
    let t = carta::run(
        &fx.lib,
        "outline",
        &docs,
        &be,
        &Options::default(),
        &mut |_| {},
    )
    .unwrap();
    assert!(
        t.contains("gateway unavailable") && t.contains("[S1]"),
        "{t}"
    );
    be.fallback_mock = false;
    assert!(
        carta::run(
            &fx.lib,
            "outline",
            &docs,
            &be,
            &Options::default(),
            &mut |_| {}
        )
        .is_err()
    );
}

#[test]
fn verify_flags_missing_and_unsupported() {
    let src = vec![Source {
        label: "S1".into(),
        path: "a.txt".into(),
        text: "The rent is 450 euro per month.".into(),
    }];
    let ans = "The rent is 450 euro monthly [S1]. Dolphins swim in oceans [S1]. Rent is due [S9].\n\n---\n**Sources**\n- [S1] a.txt\n";
    let c = verify_citations(ans, &src);
    assert_eq!(c.len(), 3, "{c:?}");
    assert!(c[0].exists && c[0].supported);
    assert!(c[1].exists && !c[1].supported);
    assert!(!c[2].exists && !c[2].supported);
    let m = mark_unsupported(ans, &src);
    assert!(
        m.contains("oceans [S1]. [unsupported]") || m.contains("[unsupported]"),
        "{m}"
    );
    assert_eq!(m.matches("[unsupported]").count(), 2, "{m}");
    assert!(m.contains("- [S1] a.txt"));
    // citation after the full stop stays attached to its sentence
    let c = verify_citations("The rent is 450 euro. [S1]", &src);
    assert_eq!(c.len(), 1);
    assert!(c[0].supported);
}

#[test]
fn chat_multi_turn_and_not_found() {
    let fx = fixture();
    let mut s = ChatSession::new(Backend::mock());
    s.max_turns = 2;
    let r = s.ask(&fx.lib, "What is photosynthesis?").unwrap();
    assert!(
        r.found && r.answer.contains("[S1]") && r.answer.contains("sugar"),
        "{r:?}"
    );
    assert!(r.render().contains("**Sources**") && r.render().contains("beta.txt"));
    assert!(r.checks.iter().all(|c| c.supported));
    let r = s.ask(&fx.lib, "How much is the rent?").unwrap();
    assert!(r.found && r.answer.contains("euro"));
    let r = s.ask(&fx.lib, "zebra quantum").unwrap();
    assert!(
        !r.found
            && r.answer
                .to_lowercase()
                .contains("not found in your documents"),
        "{r:?}"
    );
    assert_eq!(s.history.len(), 2, "history is bounded");
    let r = s.ask(&fx.lib, "Care este chiria pentru contract?").unwrap();
    assert!(r.found || r.answer.contains("documentele tale"), "{r:?}");
    let r = s.ask(&fx.lib, "Ce este zebră cuantică?").unwrap();
    assert!(
        r.answer
            .to_lowercase()
            .contains("nu am găsit în documentele tale"),
        "{r:?}"
    );
    assert!(s.ask(&fx.lib, "  ").is_err());
}

#[test]
fn language_and_formats() {
    assert_eq!(detect_lang("Care este chiria și când se plătește?"), "ro");
    assert_eq!(detect_lang("What is the rent and where is it paid?"), "en");
    assert_eq!(detect_lang("Quelle est la date dans les documents ?"), "fr");
    assert_eq!(not_found_msg("en"), "not found in your documents");
    let md = "# Title\n\n- one **bold** [S1]\n- two\n\n| A | B |\n|---|---|\n| 1 | <x> |\n\nText & more.\n";
    let h = to_html(md, "T");
    assert!(
        h.contains("<h1>Title</h1>") && h.contains("<ul>") && h.contains("<strong>bold</strong>")
    );
    assert!(h.contains("<th>A</th>") && h.contains("&lt;x&gt;") && h.contains("&amp;"));
    let t = to_docx_text(md);
    assert!(
        t.contains("• one bold [S1]")
            && t.contains("1\t<x>")
            && !t.contains("**")
            && !t.contains("---"),
        "{t}"
    );
    let fx = fixture();
    let p = carta::save_as(&fx.lib.home, md, "outline", "g", "", "html").unwrap();
    assert!(p.ends_with(".html") && std::fs::read_to_string(&p).unwrap().contains("<h1>"));
    let p = carta::save_as(&fx.lib.home, md, "outline", "g", "", "txt").unwrap();
    assert!(p.ends_with(".txt"));
    assert!(carta::save_as(&fx.lib.home, md, "o", "g", "", "pdf").is_err());
}
