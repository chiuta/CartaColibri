use cartacolibri::api::{self, Req};
use cartacolibri::carta::{self, Backend, Options};
use cartacolibri::extract;
use cartacolibri::library::{Filter, Library};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

fn zip_file(path: &Path, parts: &[(&str, &str)]) {
    let mut z = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    let o = zip::write::SimpleFileOptions::default();
    for (n, c) in parts {
        z.start_file(*n, o).unwrap();
        z.write_all(c.as_bytes()).unwrap();
    }
    z.finish().unwrap();
}

fn pdf(path: &Path, text: &str) {
    let content = format!("BT /F1 12 Tf 72 720 Td ({text}) Tj ET");
    let objs = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ];
    let mut out = String::from("%PDF-1.4\n");
    let mut offs = vec![];
    for (i, o) in objs.iter().enumerate() {
        offs.push(out.len());
        out.push_str(&format!("{} 0 obj\n{o}\nendobj\n", i + 1));
    }
    let xref = out.len();
    out.push_str(&format!(
        "xref\n0 {}\n0000000000 65535 f \n",
        objs.len() + 1
    ));
    for o in offs {
        out.push_str(&format!("{o:010} 00000 n \n"));
    }
    out.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        objs.len() + 1
    ));
    std::fs::write(path, out).unwrap();
}

struct Fx {
    _t: tempfile::TempDir,
    docs: std::path::PathBuf,
    home: String,
}

fn fixture() -> Fx {
    let t = tempfile::tempdir().unwrap();
    let docs = t.path().join("docs");
    std::fs::create_dir_all(docs.join("sub")).unwrap();
    std::fs::write(docs.join("contract.txt"), "Contractul de închiriere prevede o chirie lunară de 450 euro. Termenul este de doi ani.\nPlata se face până pe 5 ale lunii.").unwrap();
    std::fs::write(
        docs.join("sub/notes.md"),
        "# Migrare\nMigratoarele păsări zboară spre sud. Valoarea totală este 1200 lei.",
    )
    .unwrap();
    std::fs::write(docs.join("page.html"), "<html><head><style>p{color:red}</style><script>var secret=1;</script></head><body><p>Raport anual &amp; bilanț</p></body></html>").unwrap();
    zip_file(
        &docs.join("a.docx"),
        &[(
            "word/document.xml",
            r#"<w:document xmlns:w="x"><w:body><w:p><w:r><w:t>Directiva privind recunoașterea calificărilor</w:t></w:r></w:p></w:body></w:document>"#,
        )],
    );
    zip_file(
        &docs.join("b.odt"),
        &[(
            "content.xml",
            r#"<office:document-content xmlns:office="o" xmlns:text="t"><text:p>Paragraf ODT despre diplome echivalate</text:p></office:document-content>"#,
        )],
    );
    zip_file(
        &docs.join("c.pptx"),
        &[(
            "ppt/slides/slide1.xml",
            r#"<p:sld xmlns:p="p" xmlns:a="a"><a:p><a:r><a:t>Prezentare despre ufologie</a:t></a:r></a:p></p:sld>"#,
        )],
    );
    zip_file(
        &docs.join("d.xlsx"),
        &[
            (
                "xl/sharedStrings.xml",
                r#"<sst xmlns="x"><si><t>Buget anual</t></si><si><t>Cheltuieli</t></si></sst>"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                r#"<worksheet xmlns="x"><sheetData><row><c t="s"><v>0</v></c><c><v>500</v></c></row><row><c t="s"><v>1</v></c></row></sheetData></worksheet>"#,
            ),
        ],
    );
    zip_file(
        &docs.join("e.epub"),
        &[(
            "OEBPS/ch1.xhtml",
            "<html><body><p>Capitolul despre colibri și zborul lor</p></body></html>",
        )],
    );
    pdf(
        &docs.join("f.pdf"),
        "Documentul PDF despre arhitectura sistemului",
    );
    let home = t.path().join("home").to_string_lossy().into_owned();
    Fx { _t: t, docs, home }
}

fn open(fx: &Fx) -> Library {
    Library::open(Some(&fx.home), false).unwrap()
}

fn ing(fx: &Fx) -> Library {
    let mut lib = open(fx);
    lib.update(Some(&[fx.docs.to_string_lossy().into_owned()]), &mut |_| {})
        .unwrap();
    lib
}

fn s(lib: &Library, q: &str) -> Vec<String> {
    lib.search(q, &Filter::default())
        .unwrap()
        .into_iter()
        .map(|h| {
            Path::new(&h.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

#[test]
fn index_all_formats_and_search() {
    let fx = fixture();
    let lib = ing(&fx);
    assert!(lib.problems().is_empty(), "{:?}", lib.problems());
    assert_eq!(s(&lib, "chirie")[0], "contract.txt");
    assert!(
        s(&lib, "recunoasterea").contains(&"a.docx".into()),
        "diacritics must fold"
    );
    assert!(s(&lib, "diplome").contains(&"b.odt".into()));
    assert!(s(&lib, "ufologie").contains(&"c.pptx".into()));
    assert!(s(&lib, "Cheltuieli").contains(&"d.xlsx".into()));
    assert!(s(&lib, "colibri").contains(&"e.epub".into()));
    assert!(s(&lib, "arhitectura").contains(&"f.pdf".into()), "pdf text");
    assert!(s(&lib, "bilanț").contains(&"page.html".into()));
    assert!(s(&lib, "secret").is_empty(), "scripts are not text");
    assert!(s(&lib, "bilant").contains(&"page.html".into()));
}

#[test]
fn incremental_update_and_removal() {
    let fx = fixture();
    let mut lib = ing(&fx);
    let root = vec![fx.docs.to_string_lossy().into_owned()];
    let st = lib.update(Some(&root), &mut |_| {}).unwrap();
    assert_eq!((st.added, st.updated, st.removed), (0, 0, 0));
    std::fs::write(
        fx.docs.join("contract.txt"),
        "Textul nou vorbește despre tigri",
    )
    .unwrap();
    std::fs::remove_file(fx.docs.join("sub/notes.md")).unwrap();
    std::fs::write(fx.docs.join("new.txt"), "document proaspăt adăugat").unwrap();
    let st = lib.update(Some(&root), &mut |_| {}).unwrap();
    assert_eq!((st.added, st.updated, st.removed), (1, 1, 1));
    assert!(s(&lib, "tigri").contains(&"contract.txt".into()));
    assert!(s(&lib, "chirie").is_empty());
    assert!(s(&lib, "migratoare").is_empty());
}

#[test]
fn search_filters_phrases_and_fallbacks() {
    let fx = fixture();
    let lib = ing(&fx);
    let f = Filter {
        ext: vec!["md".into()],
        ..Default::default()
    };
    assert_eq!(lib.search("valoarea", &f).unwrap().len(), 1);
    assert!(lib.search("chirie", &f).unwrap().is_empty());
    assert!(!s(&lib, "\"chirie lunară\"").is_empty());
    assert!(!s(&lib, "chiri*").is_empty());
    assert!(!s(&lib, "chirie NOT zebra").is_empty());
    assert!(s(&lib, "migratoare AND sud").contains(&"notes.md".into()));
    // stem fallback: an inflected form that matches nothing literally
    assert!(
        s(&lib, "migratorii").contains(&"notes.md".into())
            || s(&lib, "migrarea").contains(&"notes.md".into())
    );
    let big = Filter {
        min_size: Some(1_000_000),
        ..Default::default()
    };
    assert!(lib.search("chirie", &big).unwrap().is_empty());
    let p = Filter {
        path_like: Some("SUB".into()),
        ..Default::default()
    };
    assert!(lib.search("valoarea", &p).unwrap().len() == 1);
    let hits = lib.search("chirie", &Filter::default()).unwrap();
    assert!(hits[0].score > 0.0 && hits[0].score <= 100.0);
}

#[test]
fn fuzz_never_panics() {
    let fx = fixture();
    let lib = ing(&fx);
    let bad = [
        "\"",
        "\"\"",
        "AND",
        "OR",
        "NOT",
        "NEAR",
        "valoarea AND",
        "(",
        ")",
        "*",
        "a\"b",
        "\"unterminated",
        "NEAR(",
        ":",
        "-",
        "^x",
        "x:y",
        "' OR 1=1 --",
        "\u{0}",
        "ß",
        "😀",
        "chirie OR",
        "NOT chirie",
    ];
    for q in bad {
        let _ = lib.search(q, &Filter::default());
    }
    assert!(!s(&lib, "valoarea AND").is_empty());
    assert!(lib.search("   ", &Filter::default()).is_err());
}

#[test]
fn groups_resolve_and_scope() {
    let fx = fixture();
    let lib = ing(&fx);
    let d = fx.docs.to_string_lossy().into_owned();
    lib.group_set(
        "juridic",
        &[format!("{d}/contract.txt")],
        Some("acte"),
        false,
    )
    .unwrap();
    lib.group_set(
        "glob",
        &[format!("{d}/*.docx"), format!("{d}/sub")],
        None,
        false,
    )
    .unwrap();
    assert_eq!(lib.resolve_group("juridic").unwrap().len(), 1);
    let g = lib.resolve_group("glob").unwrap();
    assert_eq!(g.len(), 2, "{g:?}");
    let f = Filter {
        group: Some("juridic".into()),
        ..Default::default()
    };
    assert!(lib.search("valoarea", &f).unwrap().is_empty());
    assert!(!lib.search("chirie", &f).unwrap().is_empty());
    lib.group_set("juridic", &[format!("{d}/a.docx")], None, true)
        .unwrap();
    assert_eq!(lib.resolve_group("juridic").unwrap().len(), 2);
    lib.group_remove("juridic", &[]).unwrap();
    assert!(lib.resolve_group("juridic").is_err());
}

#[test]
fn zip_bomb_and_corrupt_files_are_contained() {
    let fx = fixture();
    let t = fx.docs.join("bomb.docx");
    // a docx whose document.xml unpacks past the limit
    let mut z = zip::ZipWriter::new(std::fs::File::create(&t).unwrap());
    z.start_file(
        "word/document.xml",
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();
    let chunk = vec![b' '; 1 << 20];
    for _ in 0..(extract::MAX_ZIP_PART >> 20) + 2 {
        z.write_all(&chunk).unwrap();
    }
    z.finish().unwrap();
    std::fs::write(fx.docs.join("broken.pdf"), "not a pdf at all").unwrap();
    let lib = ing(&fx);
    let probs: Vec<String> = lib
        .problems()
        .iter()
        .map(|p| p["path"].as_str().unwrap().to_string())
        .collect();
    assert!(probs.iter().any(|p| p.ends_with("bomb.docx")), "{probs:?}");
    assert!(probs.iter().any(|p| p.ends_with("broken.pdf")));
    assert!(
        !s(&lib, "chirie").is_empty(),
        "the rest of the folder is still indexed"
    );
}

#[test]
fn legacy_encoding_is_decoded() {
    let fx = fixture();
    let (bytes, _, _) = encoding_rs::WINDOWS_1250.encode("Şcoala şi ţara noastră");
    std::fs::write(fx.docs.join("vechi.txt"), bytes).unwrap();
    let lib = ing(&fx);
    assert!(s(&lib, "scoala").contains(&"vechi.txt".into()));
}

#[test]
fn upload_is_sanitised_and_indexed() {
    let fx = fixture();
    let mut lib = open(&fx);
    let r = lib
        .add_upload(
            "../../etc/pass wd?.txt",
            "conținut încărcat despre colibri".as_bytes(),
            &[],
        )
        .unwrap();
    let saved = r["saved"].as_str().unwrap();
    assert!(
        Path::new(saved).starts_with(lib.upload_dir().unwrap()),
        "{saved}"
    );
    assert!(!saved.contains(".."));
    assert!(s(&lib, "încărcat").len() == 1);
    let r2 = lib
        .add_upload("../../etc/pass wd?.txt", b"altceva complet diferit", &[])
        .unwrap();
    assert_ne!(r["saved"], r2["saved"], "never overwrite");
}

#[test]
fn rag_carta_offline_citations_and_save() {
    let fx = fixture();
    let lib = ing(&fx);
    let d = fx.docs.to_string_lossy().into_owned();
    lib.group_set(
        "g",
        &[format!("{d}/contract.txt"), format!("{d}/sub/notes.md")],
        None,
        false,
    )
    .unwrap();
    let docs = lib.resolve_group("g").unwrap();
    let be = Backend::mock();
    let o = Options {
        request: "chirie",
        ..Default::default()
    };
    let t = carta::run(&lib, "summarize", &docs, &be, &o, &mut |_| {}).unwrap();
    assert!(
        t.contains("[S1]") && t.contains("**Sources**") && t.contains("contract.txt"),
        "{t}"
    );
    assert!(!t.contains("- -"));
    let t = carta::run(&lib, "synthesize", &docs, &be, &o, &mut |_| {}).unwrap();
    assert!(t.contains("[S2]"));
    assert!(
        carta::run(
            &lib,
            "synthesize",
            &docs,
            &be,
            &Options::default(),
            &mut |_| {}
        )
        .is_err()
    );
    let e = Options {
        fields: vec!["chirie".into()],
        ..Default::default()
    };
    let t = carta::run(&lib, "extract", &docs, &be, &e, &mut |_| {}).unwrap();
    assert!(t.contains("Sources"));
    assert!(carta::run(&lib, "nope", &docs, &be, &o, &mut |_| {}).is_err());
    assert!(carta::run(&lib, "summarize", &[], &be, &o, &mut |_| {}).is_err());
    let p = carta::save(&lib.home, &t, "extract", "g", "x").unwrap();
    assert!(Path::new(&p).is_file());
    // outputs are never indexed, even when the home is a library root
    let mut lib2 = open(&fx);
    lib2.update(
        Some(&[lib.home.to_string_lossy().into_owned()]),
        &mut |_| {},
    )
    .unwrap();
    assert!(
        lib2.documents(None)
            .unwrap()
            .iter()
            .any(|d| d.path.contains("contract.txt"))
    );
    assert!(
        !lib2
            .documents(None)
            .unwrap()
            .iter()
            .any(|d| d.path.contains("outputs"))
    );
}

#[test]
fn gateway_down_is_a_clean_error() {
    let fx = fixture();
    let lib = ing(&fx);
    let docs = lib.documents(None).unwrap();
    let mut be = Backend::gateway("http://127.0.0.1:9", None);
    be.timeout = std::time::Duration::from_secs(2);
    let err = carta::run(
        &lib,
        "summarize",
        &docs[..1],
        &be,
        &Options::default(),
        &mut |_| {},
    )
    .unwrap_err();
    assert!(err.contains("no answer from the Colibrì gateway"), "{err}");
}

fn call(
    fx: &Fx,
    method: &str,
    route: &str,
    q: &[(&str, &str)],
    body: &[u8],
    filename: Option<&str>,
) -> (u16, serde_json::Value) {
    let query: HashMap<String, String> = q
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    api::handle(
        Some(&fx.home),
        &Req {
            method,
            route,
            query,
            body,
            filename: filename.map(String::from),
            backend_url: "http://127.0.0.1:9",
        },
    )
}

#[test]
fn api_roundtrip() {
    let fx = fixture();
    let d = fx.docs.to_string_lossy().into_owned();
    let (c, v) = call(
        &fx,
        "POST",
        "folders",
        &[],
        format!(r#"{{"path":"{d}"}}"#).as_bytes(),
        None,
    );
    assert_eq!(c, 200, "{v}");
    assert!(v["added"].as_u64().unwrap() >= 7);
    let (c, v) = call(
        &fx,
        "GET",
        "search",
        &[("q", "chirie"), ("ext", "txt")],
        b"",
        None,
    );
    assert_eq!((c, v.as_array().unwrap().len()), (200, 1));
    let id = v[0]["id"].as_i64().unwrap().to_string();
    let (_, v) = call(&fx, "GET", "doc", &[("id", &id)], b"", None);
    assert!(v["text"].as_str().unwrap().contains("450 euro"));
    let (c, _) = call(
        &fx,
        "POST",
        "groups",
        &[],
        br#"{"name":"t","members":["*.txt"]}"#,
        None,
    );
    assert_eq!(c, 200);
    let (c, v) = call(
        &fx,
        "POST",
        "run",
        &[],
        br#"{"task":"summarize","group":"t","mock":true,"save":true}"#,
        None,
    );
    assert_eq!(c, 200, "{v}");
    assert!(v["text"].as_str().unwrap().contains("[S1]") && v["saved"].is_string());
    let (c, _) = call(
        &fx,
        "POST",
        "run",
        &[],
        br#"{"task":"summarize","group":"t"}"#,
        None,
    );
    assert_eq!(c, 502, "gateway down maps to 502");
    let (c, _) = call(&fx, "POST", "run", &[], b"{bad json", None);
    assert_eq!(c, 400);
    let (c, v) = call(
        &fx,
        "POST",
        "upload",
        &[],
        "document încărcat de test".as_bytes(),
        Some("up.txt"),
    );
    assert_eq!(c, 200, "{v}");
    let (c, _) = call(&fx, "GET", "nothing", &[], b"", None);
    assert_eq!(c, 404);
    let (c, v) = call(&fx, "POST", "grab", &[], r#"{"url":"https://exemplu.local/x","title":"Pagina mea","text":"Acesta este textul decupat dintr-o pagină web despre colibri."}"#.as_bytes(), None);
    assert_eq!(c, 200, "{v}");
    let (_, v) = call(&fx, "GET", "search", &[("q", "decupat")], b"", None);
    assert_eq!(v.as_array().unwrap().len(), 1);
    let (_, v) = call(
        &fx,
        "GET",
        "doc",
        &[("id", &v[0]["id"].to_string())],
        b"",
        None,
    );
    assert!(
        v["text"]
            .as_str()
            .unwrap()
            .contains("Source: https://exemplu.local/x")
    );
    let (c, _) = call(
        &fx,
        "POST",
        "grab",
        &[],
        br#"{"url":"file:///etc/passwd"}"#,
        None,
    );
    assert_eq!(c, 502, "only http(s) can be grabbed");
    let (c, v) = call(
        &fx,
        "DELETE",
        "folders",
        &[],
        format!(r#"{{"path":"{d}"}}"#).as_bytes(),
        None,
    );
    assert_eq!(c, 200);
    assert!(v["dropped"].as_u64().unwrap() >= 7);
}

#[test]
fn html_text_and_chunking() {
    assert_eq!(
        extract::html_to_text("<p>a &lt;b&gt; &#233;</p><script>x</script>").trim(),
        "a <b> é"
    );
    let long = "cuvânt ".repeat(1000);
    let ch = extract::chunk(&long, 1400);
    assert!(ch.len() > 4 && ch.iter().all(|c| c.len() <= 1400));
}
