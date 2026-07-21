# Changelog

## [Unreleased]

## [0.1.1] - 2026-07-21
### Fixed
- Windows：修复无系统 Ghostscript 时压缩报 `LoadLibrary error code 126`（随包内置 `gsdll64.dll` 并加入 sidecar PATH）
- 修正高质量/高 dpi 下无法压缩时误报「该 PDF 已经很小」的文案，改为诚实且可操作的提示

## [0.1.0] - 2026-05-09
### Added
- 单文件 PDF 压缩，4 个质量档位
- 内置 Ghostscript 10.x
- macOS arm64/x64 与 Windows x64 安装包
- 启动时检查 GitHub 新版本
