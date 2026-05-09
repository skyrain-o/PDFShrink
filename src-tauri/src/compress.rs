use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    Standard,
    HighQuality,
    Extreme,
    Custom(u32),
}

#[derive(Debug, Serialize)]
pub struct CompressionReport {
    pub input_size: u64,
    pub output_size: u64,
    pub output_path: String,
}

const COMMON: &[&str] = &[
    "-sDEVICE=pdfwrite",
    "-dCompatibilityLevel=1.5",
    "-dNOPAUSE",
    "-dQUIET",
    "-dBATCH",
];

/// Build the Ghostscript argv (without the gs binary itself) for the given preset.
pub fn build_gs_args(input: &Path, output: &Path, preset: Preset) -> Result<Vec<String>, AppError> {
    let mut args: Vec<String> = COMMON.iter().map(|s| s.to_string()).collect();

    match preset {
        Preset::Standard => {
            args.push("-dPDFSETTINGS=/ebook".into());
        }
        Preset::HighQuality => {
            push_dpi_args(&mut args, 200);
        }
        Preset::Extreme => {
            push_dpi_args(&mut args, 100);
        }
        Preset::Custom(dpi) => {
            if !(50..=600).contains(&dpi) {
                return Err(AppError::DpiOutOfRange(dpi));
            }
            push_dpi_args(&mut args, dpi);
        }
    }

    args.push(format!("-sOutputFile={}", output.display()));
    args.push(input.display().to_string());
    Ok(args)
}

fn push_dpi_args(args: &mut Vec<String>, dpi: u32) {
    args.push("-dDownsampleColorImages=true".into());
    args.push(format!("-dColorImageResolution={dpi}"));
    args.push("-dDownsampleGrayImages=true".into());
    args.push(format!("-dGrayImageResolution={dpi}"));
    args.push("-dDownsampleMonoImages=true".into());
    args.push(format!("-dMonoImageResolution={}", dpi.max(300)));
    args.push("-dColorImageDownsampleType=/Bicubic".into());
    args.push("-dGrayImageDownsampleType=/Bicubic".into());
}

use std::path::PathBuf;

/// Resolve the output path for a given input PDF.
/// `exists` is injected so tests can simulate collisions without touching the filesystem.
pub fn resolve_output_path(input: &Path, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let parent = input.parent().unwrap_or(Path::new("."));
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let primary = parent.join(format!("{stem}_compressed.pdf"));
    if !exists(&primary) {
        return primary;
    }
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    parent.join(format!("{stem}_compressed_{ts}.pdf"))
}

use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ghostscript;

const TIMEOUT_SECS: u64 = 300;

pub async fn run(app: &AppHandle, input: &Path, preset: Preset) -> Result<CompressionReport, AppError> {
    if !input.exists() {
        return Err(AppError::FileNotFound);
    }
    let ext = input.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext != "pdf" {
        return Err(AppError::NotAPdf(ext));
    }
    let input_size = std::fs::metadata(input).map_err(|_| AppError::ReadDenied)?.len();

    let final_path = resolve_output_path(input, |p| p.exists());
    let tmp_path = final_path.with_extension("pdf.tmp");

    let args = build_gs_args(input, &tmp_path, preset)?;

    let res_dir = app.path()
        .resolve("resources/gs-lib/Resource", tauri::path::BaseDirectory::Resource)
        .map_err(|_| AppError::GsMissing)?;
    let gs_lib = format!("{init}:{font}",
        init = res_dir.join("Init").display(),
        font = res_dir.join("Font").display());

    let sidecar = app.shell().sidecar(ghostscript::sidecar_name())
        .map_err(|_| AppError::GsMissing)?
        .args(&args)
        .env("GS_LIB", gs_lib);

    let (mut rx, _child) = sidecar.spawn().map_err(|_| AppError::GsMissing)?;

    let start = Instant::now();
    let mut stderr_tail = String::new();
    let mut exit_code: Option<i32> = None;

    while let Some(ev) = rx.recv().await {
        if start.elapsed() > Duration::from_secs(TIMEOUT_SECS) {
            return Err(AppError::GsTimeout(TIMEOUT_SECS));
        }
        match ev {
            CommandEvent::Stderr(line) => {
                let s = String::from_utf8_lossy(&line);
                stderr_tail.push_str(&s);
                if stderr_tail.len() > 200 {
                    stderr_tail = stderr_tail.chars().rev().take(200).collect::<String>().chars().rev().collect();
                }
            }
            CommandEvent::Terminated(payload) => {
                exit_code = payload.code;
                break;
            }
            _ => {}
        }
    }

    let code = exit_code.unwrap_or(-1);
    if code != 0 {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(AppError::GsFailed { code, stderr_tail });
    }

    let out_meta = std::fs::metadata(&tmp_path).map_err(|_| AppError::OutputInvalid)?;
    if out_meta.len() < 4 {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(AppError::OutputInvalid);
    }
    let mut header = [0u8; 4];
    use std::io::Read;
    let mut f = std::fs::File::open(&tmp_path).map_err(|_| AppError::OutputInvalid)?;
    f.read_exact(&mut header).map_err(|_| AppError::OutputInvalid)?;
    drop(f);
    if &header != b"%PDF" {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(AppError::OutputInvalid);
    }

    if out_meta.len() >= input_size {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(AppError::NoGain);
    }

    std::fs::rename(&tmp_path, &final_path).map_err(|_| AppError::WriteDenied(final_path.display().to_string()))?;

    Ok(CompressionReport {
        input_size,
        output_size: out_meta.len(),
        output_path: final_path.display().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn paths() -> (PathBuf, PathBuf) {
        (PathBuf::from("/in.pdf"), PathBuf::from("/out.pdf"))
    }

    #[test]
    fn standard_uses_ebook_preset() {
        let (i, o) = paths();
        let args = build_gs_args(&i, &o, Preset::Standard).unwrap();
        assert!(args.iter().any(|a| a == "-dPDFSETTINGS=/ebook"));
        assert!(!args.iter().any(|a| a.starts_with("-dColorImageResolution=")));
    }

    #[test]
    fn high_quality_uses_200_dpi() {
        let (i, o) = paths();
        let args = build_gs_args(&i, &o, Preset::HighQuality).unwrap();
        assert!(args.iter().any(|a| a == "-dColorImageResolution=200"));
        assert!(args.iter().any(|a| a == "-dGrayImageResolution=200"));
    }

    #[test]
    fn extreme_uses_100_dpi() {
        let (i, o) = paths();
        let args = build_gs_args(&i, &o, Preset::Extreme).unwrap();
        assert!(args.iter().any(|a| a == "-dColorImageResolution=100"));
    }

    #[test]
    fn custom_dpi_in_range_ok() {
        let (i, o) = paths();
        let args = build_gs_args(&i, &o, Preset::Custom(180)).unwrap();
        assert!(args.iter().any(|a| a == "-dColorImageResolution=180"));
    }

    #[test]
    fn custom_dpi_below_min_rejected() {
        let (i, o) = paths();
        let err = build_gs_args(&i, &o, Preset::Custom(49)).unwrap_err();
        assert!(matches!(err, AppError::DpiOutOfRange(49)));
    }

    #[test]
    fn custom_dpi_above_max_rejected() {
        let (i, o) = paths();
        let err = build_gs_args(&i, &o, Preset::Custom(601)).unwrap_err();
        assert!(matches!(err, AppError::DpiOutOfRange(601)));
    }

    #[test]
    fn output_path_is_last_or_second_to_last() {
        let (i, o) = paths();
        let args = build_gs_args(&i, &o, Preset::Standard).unwrap();
        assert_eq!(args[args.len() - 2], "-sOutputFile=/out.pdf");
        assert_eq!(args[args.len() - 1], "/in.pdf");
    }

    #[test]
    fn common_args_always_present() {
        let (i, o) = paths();
        for preset in [Preset::Standard, Preset::HighQuality, Preset::Extreme, Preset::Custom(150)] {
            let args = build_gs_args(&i, &o, preset).unwrap();
            for c in COMMON {
                assert!(args.iter().any(|a| a == c), "missing {c} for {preset:?}");
            }
        }
    }

    use std::fs;

    #[test]
    fn output_path_appends_compressed_suffix() {
        let p = PathBuf::from("/tmp/foo.pdf");
        assert_eq!(
            super::resolve_output_path(&p, |_| false).file_name().unwrap(),
            "foo_compressed.pdf"
        );
    }

    #[test]
    fn output_path_collision_adds_timestamp() {
        let p = PathBuf::from("/tmp/foo.pdf");
        let resolved = super::resolve_output_path(&p, |path| {
            path.file_name().unwrap().to_string_lossy() == "foo_compressed.pdf"
        });
        let name = resolved.file_name().unwrap().to_string_lossy().to_string();
        assert!(name.starts_with("foo_compressed_"), "got {name}");
        assert!(name.ends_with(".pdf"));
    }

    #[test]
    fn output_path_preserves_directory() {
        let p = PathBuf::from("/some/dir/foo.pdf");
        let resolved = super::resolve_output_path(&p, |_| false);
        assert_eq!(resolved.parent().unwrap(), Path::new("/some/dir"));
        let _ = fs::metadata("/tmp"); // silence unused import
    }
}
