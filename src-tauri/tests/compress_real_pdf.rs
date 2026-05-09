use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures");
    p.push(name);
    p
}

fn staged_gs() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("binaries");
    let triple = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else { panic!("unsupported test target") };
    let name = if cfg!(target_os = "windows") {
        format!("gs-{triple}.exe")
    } else { format!("gs-{triple}") };
    p.push(name);
    p
}

#[test]
fn compresses_small_pdf_with_staged_gs() {
    let gs = staged_gs();
    if !gs.exists() {
        eprintln!("skip: staged gs not found at {gs:?} (run scripts/fetch-gs.sh)");
        return;
    }
    let input = fixture("english_text.pdf");
    let out = std::env::temp_dir().join("pdfshrink_test_out.pdf");
    let _ = std::fs::remove_file(&out);

    let status = Command::new(&gs)
        .args([
            "-sDEVICE=pdfwrite",
            "-dCompatibilityLevel=1.5",
            "-dNOPAUSE", "-dQUIET", "-dBATCH",
            "-dPDFSETTINGS=/ebook",
        ])
        .arg(format!("-sOutputFile={}", out.display()))
        .arg(&input)
        .status()
        .expect("spawn gs");
    assert!(status.success(), "gs exit code: {status}");
    let meta = std::fs::metadata(&out).expect("output file");
    assert!(meta.len() > 0, "output is empty");
    let header = std::fs::read(&out).unwrap();
    assert_eq!(&header[..4], b"%PDF", "not a PDF header");
}
