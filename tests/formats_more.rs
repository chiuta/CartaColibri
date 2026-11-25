//! tar / tar.gz / tgz archives, OpenDocument sheets and slides, macro-enabled Office variants.
use cartacolibri::extract::extract;
use std::io::Write;
use std::path::Path;

fn tar_header(name: &str, size: usize, kind: u8) -> [u8; 512] {
    let mut h = [0u8; 512];
    h[..name.len()].copy_from_slice(name.as_bytes());
    h[100..107].copy_from_slice(b"0000644");
    h[124..135].copy_from_slice(format!("{size:011o}").as_bytes());
    h[156] = kind;
    h[257..262].copy_from_slice(b"ustar");
    h[148..156].copy_from_slice(b"        ");
    let sum: u32 = h.iter().map(|b| *b as u32).sum();
    h[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
    h
}

fn tar(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = vec![];
    for (n, d) in files {
        out.extend_from_slice(&tar_header(n, d.len(), b'0'));
        out.extend_from_slice(d);
        out.extend(std::iter::repeat_n(0u8, (512 - d.len() % 512) % 512));
    }
    out.extend(std::iter::repeat_n(0u8, 1024));
    out
}

fn text(p: &Path, ext: &str) -> Result<String, String> {
    extract(p, ext, false).map(|parts| {
        parts
            .into_iter()
            .map(|(l, t)| format!("[{l}] {t}"))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

#[test]
fn tar_and_tgz_list_and_read_their_text_members() {
    let t = tempfile::tempdir().unwrap();
    let data = tar(&[
        ("notes/plan.txt", b"The quarterly budget needs approval."),
        ("img/logo.png", &[0x89, b'P', b'N', b'G', 0, 1, 2]),
        ("docs/readme.md", b"# Handover\nKeys are in the safe."),
    ]);
    let p = t.path().join("a.tar");
    std::fs::write(&p, &data).unwrap();
    let s = text(&p, ".tar").unwrap();
    assert!(s.contains("notes/plan.txt (36 bytes)"), "{s}");
    assert!(s.contains("quarterly budget"), "{s}");
    assert!(s.contains("Keys are in the safe"), "{s}");
    assert!(
        !s.contains("PNG"),
        "binary members are listed, never read as text"
    );
    // gzip
    let mut gz = flate2::write::GzEncoder::new(vec![], flate2::Compression::fast());
    gz.write_all(&data).unwrap();
    let p2 = t.path().join("a.tgz");
    std::fs::write(&p2, gz.finish().unwrap()).unwrap();
    assert!(text(&p2, ".tgz").unwrap().contains("quarterly budget"));
}

#[test]
fn damaged_and_hostile_tar_files_fail_cleanly() {
    let t = tempfile::tempdir().unwrap();
    let good = tar(&[("a.txt", b"hello world")]);
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("empty", vec![]),
        (
            "garbage",
            b"this is not a tar file at all, only text".repeat(40),
        ),
        ("truncated header", good[..300].to_vec()),
        ("truncated body", good[..520].to_vec()),
        ("bad checksum", {
            let mut b = good.clone();
            b[0] ^= 0xff;
            b
        }),
    ];
    for (name, bytes) in cases {
        let p = t.path().join("x.tar");
        std::fs::write(&p, &bytes).unwrap();
        // an error or a partial listing are both fine; a panic or a hang is not
        let _ = text(&p, ".tar");
        eprintln!("{name}: ok");
    }
    // a header that announces a 1 TB member must not allocate it
    let mut huge = tar_header("big.txt", 0, b'0');
    huge[124..135].copy_from_slice(b"10000000000");
    huge[148..156].copy_from_slice(b"        ");
    let sum: u32 = huge.iter().map(|b| *b as u32).sum();
    huge[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
    std::fs::write(t.path().join("h.tar"), huge).unwrap();
    let _ = text(&t.path().join("h.tar"), ".tar");
    // a gzip bomb of zeros is cut by the stream limit rather than exhausting memory
    let mut gz = flate2::write::GzEncoder::new(vec![], flate2::Compression::best());
    gz.write_all(&vec![0u8; 8 * 1024 * 1024]).unwrap();
    std::fs::write(t.path().join("z.tgz"), gz.finish().unwrap()).unwrap();
    let _ = text(&t.path().join("z.tgz"), ".tgz");
    // gzip that is not a tar
    std::fs::write(t.path().join("n.tgz"), b"\x1f\x8b not really").unwrap();
    assert!(text(&t.path().join("n.tgz"), ".tgz").is_err());
}

fn zip_of(path: &Path, files: &[(&str, &str)]) {
    let f = std::fs::File::create(path).unwrap();
    let mut z = zip::ZipWriter::new(f);
    for (n, c) in files {
        z.start_file(*n, zip::write::SimpleFileOptions::default())
            .unwrap();
        z.write_all(c.as_bytes()).unwrap();
    }
    z.finish().unwrap();
}

#[test]
fn opendocument_sheets_slides_and_office_variants_are_read() {
    let t = tempfile::tempdir().unwrap();
    let ods = t.path().join("a.ods");
    zip_of(
        &ods,
        &[(
            "content.xml",
            r#"<office:document-content xmlns:office="o" xmlns:table="t" xmlns:text="x"><office:body><office:spreadsheet><table:table><table:table-row><table:table-cell><text:p>Rent</text:p></table:table-cell><table:table-cell><text:p>1200</text:p></table:table-cell></table:table-row></table:table></office:spreadsheet></office:body></office:document-content>"#,
        )],
    );
    let s = text(&ods, ".ods").unwrap();
    assert!(s.contains("Rent") && s.contains("1200"), "{s}");
    let odp = t.path().join("a.odp");
    zip_of(
        &odp,
        &[(
            "content.xml",
            r#"<office:document-content xmlns:office="o" xmlns:text="x" xmlns:draw="d"><office:body><office:presentation><draw:page><text:p>Launch plan</text:p></draw:page></office:presentation></office:body></office:document-content>"#,
        )],
    );
    assert!(text(&odp, ".odp").unwrap().contains("Launch plan"));
    // a macro-enabled Word file is a Word file
    let docm = t.path().join("m.docm");
    zip_of(
        &docm,
        &[(
            "word/document.xml",
            r#"<w:document xmlns:w="w"><w:body><w:p><w:r><w:t>Macro letter</w:t></w:r></w:p></w:body></w:document>"#,
        )],
    );
    assert!(text(&docm, ".docm").unwrap().contains("Macro letter"));
    // a damaged one is an error, not a panic
    std::fs::write(t.path().join("bad.ods"), b"PK nothing").unwrap();
    assert!(text(&t.path().join("bad.ods"), ".ods").is_err());
}
