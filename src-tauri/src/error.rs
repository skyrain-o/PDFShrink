use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("file not found")]
    FileNotFound,
    #[error("not a PDF (extension: {0})")]
    NotAPdf(String),
    #[error("read denied")]
    ReadDenied,
    #[error("write denied: {0}")]
    WriteDenied(String),
    #[error("ghostscript missing")]
    GsMissing,
    #[error("ghostscript failed (exit {code}): {stderr_tail}")]
    GsFailed { code: i32, stderr_tail: String },
    #[error("ghostscript timeout after {0}s")]
    GsTimeout(u64),
    #[error("output PDF invalid")]
    OutputInvalid,
    #[error("output not smaller than input")]
    NoGain,
    #[error("dpi out of range: {0}")]
    DpiOutOfRange(u32),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Serialize)]
pub struct UserError {
    pub kind: &'static str,
    pub message: String,
}

impl AppError {
    pub fn to_user(&self) -> UserError {
        let kind = match self {
            AppError::FileNotFound => "file_missing",
            AppError::NotAPdf(_) => "not_pdf",
            AppError::ReadDenied => "read_denied",
            AppError::WriteDenied(_) => "write_denied",
            AppError::GsMissing => "engine_missing",
            AppError::GsFailed { .. } => "gs_failed",
            AppError::GsTimeout(_) => "gs_timeout",
            AppError::OutputInvalid => "output_invalid",
            AppError::NoGain => "no_gain",
            AppError::DpiOutOfRange(_) => "dpi_range",
            AppError::Io(_) => "io",
        };
        UserError { kind, message: self.user_message() }
    }

    pub fn user_message(&self) -> String {
        match self {
            AppError::FileNotFound => "源文件已不存在".into(),
            AppError::NotAPdf(ext) => format!("请拖入 PDF 文件（当前文件类型：.{ext}）"),
            AppError::ReadDenied => "无法读取该文件，请检查权限".into(),
            AppError::WriteDenied(p) => format!("无法写入输出目录: {p}"),
            AppError::GsMissing => "压缩引擎缺失，请重新安装应用".into(),
            AppError::GsFailed { stderr_tail, .. } => format!("压缩失败: {stderr_tail}"),
            AppError::GsTimeout(s) => format!("压缩超时（{s}秒），文件可能损坏"),
            AppError::OutputInvalid => "压缩产物无效".into(),
            AppError::NoGain => "该 PDF 已经很小，无需进一步压缩".into(),
            AppError::DpiOutOfRange(n) => format!("dpi 必须在 50–600 之间（输入 {n}）"),
            AppError::Io(e) => format!("io 错误: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_a_pdf_message_contains_extension() {
        let e = AppError::NotAPdf("docx".into());
        assert!(e.user_message().contains(".docx"));
    }

    #[test]
    fn dpi_range_message_contains_value() {
        let e = AppError::DpiOutOfRange(9999);
        assert!(e.user_message().contains("9999"));
    }

    #[test]
    fn user_error_kind_is_stable_string() {
        assert_eq!(AppError::FileNotFound.to_user().kind, "file_missing");
        assert_eq!(AppError::GsMissing.to_user().kind, "engine_missing");
    }

    #[test]
    fn gs_failed_includes_stderr_tail() {
        let e = AppError::GsFailed { code: 1, stderr_tail: "boom".into() };
        let msg = e.user_message();
        assert!(msg.contains("boom"));
    }
}
