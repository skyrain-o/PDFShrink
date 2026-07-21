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

/// Directory holding the bundled Ghostscript resources (and, on Windows,
/// `gsdll64.dll`). Mirrors what compress.rs resolves via BaseDirectory::Resource.
fn gs_lib_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("resources");
    p.push("gs-lib");
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

    let mut cmd = Command::new(&gs);
    cmd.args([
        "-sDEVICE=pdfwrite",
        "-dCompatibilityLevel=1.5",
        "-dNOPAUSE", "-dQUIET", "-dBATCH",
        "-dPDFSETTINGS=/ebook",
    ])
    .arg(format!("-sOutputFile={}", out.display()))
    .arg(&input);

    // Reproduce the end-user machine: `gswin64c.exe` is a launcher that loads
    // gsdll64.dll at runtime. A clean machine has NO system Ghostscript on PATH,
    // so the app must ship the DLL and put its dir on PATH itself. We simulate
    // that here by stripping any system Ghostscript dir from PATH and adding
    // only the bundled resource dir. Before the fix this failed with
    // "Can't load Ghostscript DLL ... LoadLibrary error code 126".
    #[cfg(target_os = "windows")]
    {
        let dll = gs_lib_dir().join("gsdll64.dll");
        assert!(dll.exists(), "gsdll64.dll not staged at {dll:?} (run scripts/fetch-gs.sh)");
        let clean: Vec<PathBuf> = std::env::split_paths(&std::env::var("PATH").unwrap_or_default())
            .filter(|p| {
                let s = p.to_string_lossy().to_lowercase();
                !(s.contains("\\gs\\") || s.contains("ghostscript"))
            })
            .collect();
        let path = std::env::join_paths(std::iter::once(gs_lib_dir()).chain(clean))
            .expect("join PATH");
        cmd.env("PATH", path);
    }
    let _ = gs_lib_dir; // used only on Windows; silence unused warning elsewhere

    let status = cmd.status().expect("spawn gs");
    assert!(status.success(), "gs exit code: {status}");
    let meta = std::fs::metadata(&out).expect("output file");
    assert!(meta.len() > 0, "output is empty");
    let header = std::fs::read(&out).unwrap();
    assert_eq!(&header[..4], b"%PDF", "not a PDF header");
}
