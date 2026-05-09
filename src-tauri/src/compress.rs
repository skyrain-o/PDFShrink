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
}
