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
