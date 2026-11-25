use cartacolibri::insight::*;

fn src(t: &str) -> Vec<(String, String)> {
    vec![("S1".into(), t.into())]
}

#[test]
fn grounding_flags_invented_claims_and_numbers() {
    let s =
        src("The contract was signed on 2024-03-01 and the penalty is 500 lei per delayed month.");
    let g = grounding(
        "The contract was signed in March and carries a penalty for every delayed month [S1]. The supplier also owes 9000 lei in damages for fraud.",
        &s,
    );
    assert_eq!(g.claims.len(), 2);
    assert_eq!(g.claims[0].status, "supported");
    assert_eq!(g.claims[0].cited, vec!["S1"]);
    assert_ne!(g.claims[1].status, "supported");
    assert!(g.claims[1].unmatched_numbers.contains(&"9000".to_string()));
    assert!(g.score < 70);
    assert_eq!(grounding("", &s).score, 0);
}

#[test]
fn entities_are_found_and_indexed() {
    let t = "Write to ana.pop@example.ro or call +40 721 234 567. Pay 1.250,50 lei or €300 by 15.03.2025 (see https://example.org/a).";
    let kinds: Vec<_> = entities(t).iter().map(|e| e.kind).collect();
    for k in KINDS {
        assert!(kinds.contains(&k), "missing {k}");
    }
    let idx = entity_index(
        &[
            ("a.txt".into(), t.into()),
            ("b.txt".into(), "ana.pop@example.ro".into()),
        ],
        Some("email"),
    );
    assert_eq!(idx.len(), 1);
    assert_eq!(idx.values().next().unwrap().len(), 2);
}

#[test]
fn compare_lists_what_changed() {
    let a = "The rent is 500 euro per month. The tenant pays utilities. Notice is thirty days.";
    let b = "The rent is 550 euro per month. The tenant pays utilities. Notice is thirty days.";
    let c = compare(a, b);
    assert_eq!(c.shared, 2);
    assert_eq!(c.only_in_a.len(), 1);
    assert!(c.only_in_b[0].contains("550"));
    assert_eq!(c.similarity, 50);
    assert_eq!(compare(a, a).similarity, 100);
}

#[test]
fn conflicts_find_same_sentence_with_different_numbers() {
    let docs = vec![
        (
            "a.txt".to_string(),
            "The monthly rent for the apartment is 500 euro. The tenant pays utilities separately."
                .to_string(),
        ),
        (
            "b.txt".to_string(),
            "The monthly rent for the apartment is 550 euro. Heating is included in the price."
                .to_string(),
        ),
        (
            "c.txt".to_string(),
            "The monthly rent for the apartment is 500 euro. Parking costs extra.".to_string(),
        ),
    ];
    let c = conflicts(&docs);
    assert!(c.iter().any(|x| x.a_path == "a.txt"
        && x.b_path == "b.txt"
        && x.a_only == vec!["500"]
        && x.b_only == vec!["550"]));
    assert!(
        c.iter()
            .all(|x| !(x.a_path == "a.txt" && x.b_path == "c.txt")),
        "same numbers is no conflict"
    );
    assert!(conflicts(&[]).is_empty());
}

#[test]
fn topics_group_documents_by_their_own_vocabulary() {
    let mk = |s: &str| s.repeat(3);
    let docs = vec![
        (
            "v1.txt".to_string(),
            mk("Vaccination schedule immunization clinic pediatric vaccination. "),
        ),
        (
            "v2.txt".to_string(),
            mk("Pediatric clinic vaccination immunization booster. "),
        ),
        (
            "t1.txt".to_string(),
            mk("Taxation invoice deduction accounting fiscal taxation. "),
        ),
        (
            "t2.txt".to_string(),
            mk("Fiscal accounting deduction taxation declaration. "),
        ),
        (
            "x.txt".to_string(),
            mk("Astronomy telescope nebula galaxy. "),
        ),
    ];
    let t = topics(&docs);
    assert_eq!(t.len(), 2, "{t:?}");
    assert!(t.iter().any(|x| x.docs == vec!["v1.txt", "v2.txt"]));
    assert!(t.iter().any(|x| x.docs == vec!["t1.txt", "t2.txt"]));
}

#[test]
fn fact_check_judges_a_claim_against_evidence() {
    let ev = "The penalty is 500 lei for every delayed month of delivery.";
    let (sup, s) = best_evidence(
        "The penalty for every delayed month of delivery is 900 lei.",
        ev,
    )
    .unwrap();
    assert!(sup >= 0.6 && s.contains("500"));
    let (st, un) = judge(
        "The penalty for every delayed month of delivery is 900 lei.",
        sup,
        &s,
    );
    assert_eq!(st, "numbers differ");
    assert_eq!(un, vec!["900"]);
    assert_eq!(judge("x", 0.1, "").0, "not found");
}

fn lib_with(files: &[(&str, &str)]) -> (cartacolibri::library::Library, tempfile::TempDir) {
    let home = tempfile::tempdir().unwrap();
    let docs = tempfile::tempdir().unwrap();
    for (n, t) in files {
        std::fs::write(docs.path().join(n), t).unwrap();
    }
    let mut lib =
        cartacolibri::library::Library::open(Some(home.path().to_str().unwrap()), false).unwrap();
    lib.add_root(docs.path().to_str().unwrap()).unwrap();
    lib.update(None, &mut |_| {}).unwrap();
    std::mem::forget(docs);
    (lib, home)
}

#[test]
fn factcheck_dossier_and_changes_on_a_real_library() {
    let (lib, _h) = lib_with(&[
        (
            "contract.txt",
            "The penalty is 500 lei for every delayed month of delivery. Signed on 2024-03-01 by the supplier Acme.",
        ),
        (
            "offer.txt",
            "Acme offers delivery within ten days at a price of 1200 euro.",
        ),
    ]);
    let c = factcheck(&lib, "The penalty for every delayed month of delivery is 900 lei. The moon is made of cheese and nobody knows.").unwrap();
    assert_eq!(c[0].status, "numbers differ");
    assert!(c[0].source.as_deref().unwrap().ends_with("contract.txt"));
    assert_eq!(c[1].status, "not found");
    let d = dossier(&lib, "Acme").unwrap();
    assert_eq!(d.docs.len(), 2);
    assert!(d.dates.contains(&"2024-03-01".to_string()));
    assert!(d.money.iter().any(|m| m.contains("1200")));
    assert!(dossier(&lib, "  ").is_err());
    assert_eq!(changes(&lib, 0, 10).unwrap().len(), 2);
    assert!(changes(&lib, 32503680000, 10).unwrap().is_empty());
}

#[test]
fn dupes_find_copies_and_contained_documents() {
    let a = "The supplier delivers the goods within thirty days after the order is confirmed in writing by the buyer and pays the invoice in sixty days.";
    let docs = vec![
        ("a.txt".to_string(), a.to_string()),
        ("copy.txt".to_string(), a.to_uppercase()),
        (
            "inside.txt".to_string(),
            format!("Cover note. {a} Signed."),
        ),
        (
            "other.txt".to_string(),
            "Completely different text about the migration of birds across the northern sea in autumn and spring seasons.".to_string(),
        ),
    ];
    let d = dupes(&docs, 0.6);
    assert!(
        d.iter()
            .any(|x| x.a_path == "a.txt" && x.b_path == "copy.txt" && x.similarity == 1.0)
    );
    assert!(
        d.iter()
            .any(|x| x.b_path == "inside.txt" && x.contained >= 0.9)
    );
    assert!(
        !d.iter()
            .any(|x| x.a_path == "other.txt" || x.b_path == "other.txt")
    );
    assert!(dupes(&[], 0.6).is_empty());
}

#[test]
fn chronology_orders_dated_sentences_and_rejects_bad_dates() {
    let docs = vec![
        (
            "a.txt".to_string(),
            "The deadline is 15.03.2025 for everyone. Signed on 2024-11-02 by both parties. Not a date: 45.13.2025 here.".to_string(),
        ),
        ("b.txt".to_string(), "Meeting on 01/02/2025 at noon.".to_string()),
    ];
    let t = chronology(&docs, 50);
    let dates: Vec<_> = t.iter().map(|e| e.date.as_str()).collect();
    assert_eq!(dates, vec!["2024-11-02", "2025-02-01", "2025-03-15"]);
    assert!(t[0].sentence.contains("Signed"));
    assert_eq!(chronology(&docs, 1).len(), 1);
}

#[test]
fn dupes_find_a_short_document_inside_a_long_one() {
    let words = |seed: u64, n: usize| -> String {
        let mut x = seed;
        (0..n)
            .map(|_| {
                x = x
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                format!("w{}", (x >> 33) % 5000)
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let short = words(1, 60);
    let long = format!("{} {} {}", words(2, 3000), short, words(3, 3000));
    let docs = vec![
        ("long.txt".to_string(), long),
        ("short.txt".to_string(), short),
        ("unrelated.txt".to_string(), words(9, 500)),
        (
            "tiny.txt".to_string(),
            "just five little words here".to_string(),
        ),
    ];
    let d = dupes(&docs, 0.6);
    assert_eq!(d.len(), 1, "{d:?}");
    assert!(d[0].contained >= 0.99 && d[0].similarity < 0.1);
}

#[test]
fn chronology_reads_month_names_in_eight_languages() {
    let docs = vec![(
        "m.txt".to_string(),
        [
            "Due on 12 March 2025 for all.",
            "Signed March 5, 2024 by both.",
            "Termen: 3 martie 2023 la sediu.",
            "Le 14 février 2022 il a signé.",
            "Il 9 maggio 2021 abbiamo firmato.",
            "El 1 de enero de 2020 se firmó.",
            "Assinado em 7 de março de 2019 ontem.",
            "Am 30. Oktober 2018 wurde es unterschrieben.",
            "A szerződés 2017. szeptember 21 napján kelt.",
            "Nonsense 31 Blorp 2025 is not a date.",
        ]
        .join(" "),
    )];
    let d: Vec<String> = chronology(&docs, 50).into_iter().map(|e| e.date).collect();
    assert_eq!(
        d,
        vec![
            "2017-09-21",
            "2018-10-30",
            "2019-03-07",
            "2020-01-01",
            "2021-05-09",
            "2022-02-14",
            "2023-03-03",
            "2024-03-05",
            "2025-03-12"
        ]
    );
}
