# PDFShrink

> 拖进来，瘦下去。一个 PDF 压缩工具，仅此而已。

把 18MB 的扫描合同压到 4MB，把不能发邮件的报告变成能发的，把扫描的发票塞进只接受 10MB 上传的网银。文件不上传到任何服务器，文字与印章都还在。

## 为什么是它

- **拖进来就完事**：没有上传，没有等待，没有水印，没有"剩余次数"
- **隐私安全**：PDF 文件从不离开你的电脑。底层是开源的 Ghostscript，跑在你本地（启动时会向 GitHub 查一次新版本，仅此而已）
- **跨平台**：macOS（Apple Silicon / Intel）+ Windows 一份体验
- **不挑文件**：扫描件、合同、发票、论文、教程，体积都能砍下去
- **足够小**：单平台安装包不到 50MB，启动 1 秒

## 下载

去 [Releases](https://github.com/skyrain-o/PDFShrink/releases/latest) 挑你的系统：

| 系统 | 下载 |
|---|---|
| macOS (Apple Silicon, M1/M2/M3/M4) | `pdfshrink_<ver>_aarch64.dmg` |
| macOS (Intel) | `pdfshrink_<ver>_x64.dmg` |
| Windows 10/11 (64-bit) | `pdfshrink_<ver>_x64-setup.exe` |

### 首次打开（重要）

应用未签名（个人项目，没买开发者证书），系统第一次会拦一下：

- **macOS**：右键应用图标 → 打开 → 在弹窗里再点一次"打开"。之后双击即可。
- **Windows**：SmartScreen 蓝屏 → "更多信息" → "仍要运行"。

确认是直接从本仓库 Releases 下载的就放心用。

## 怎么用

1. 双击启动 PDFShrink
2. 把 PDF 拖进窗口（或点击选择）
3. 完成。压缩后的文件叫 `<原名>_compressed.pdf`，就在原文件旁边

### 四个档位

| 档位 | dpi | 适合 |
|---|---|---|
| **标准**（默认） | 150 | 扫描合同、发票、票据 — 90% 的场景选这个 |
| **高质量** | 200 | 含图表/插图、需要细看的资料 |
| **极致压缩** | 100 | 只要文字能看清，体积越小越好 |
| **自定义** | 50–600 | 你知道自己在干什么 |

不知道选哪个？拖进去试默认档位；不满意把高质量再试一遍，三秒钟的事。

## 它在做什么

调用本地内置的 [Ghostscript](https://www.ghostscript.com/) 重新生成 PDF：图像下采样 + 字体子集化 + 流压缩。文字和矢量图形保持原样，体积主要来自图像降码率。

PDF 数据不联网、不上传、不读你别的文件，[源码可查](./src-tauri/src/compress.rs)。

## 从源码构建

需要 Node 20+ / Rust 1.78+ / Ghostscript（系统装一份用于打包脚本）。

```bash
git clone https://github.com/skyrain-o/PDFShrink.git
cd PDFShrink
npm install
bash scripts/fetch-gs.sh   # 复制 Ghostscript 到打包目录
npm run tauri dev          # 开发模式
npm run tauri build        # 出安装包
```

CI 矩阵会同时为 macOS arm64 / x64 与 Windows x64 出包，看 `.github/workflows/build.yml`。

## 已知不做

YAGNI 原则下故意砍掉的功能（如果有人 PR 我会看）：

- 批量压缩 / 文件夹递归
- PDF 合并、拆分、加密、OCR
- 自动更新（启动时检查新版本，但不自动下载）
- 设置持久化（每次都要选档位）
- 暗色主题、英文界面

## 许可证

- 应用代码：[MIT](./LICENSE)
- 内置 Ghostscript：[AGPL v3](./LICENSE-Ghostscript.txt)（保留 Artifex 版权与许可声明，源码可在 [ghostscript.com](https://www.ghostscript.com/releases/gsdnld.html) 获取）
