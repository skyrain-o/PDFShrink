# PDFShrink

> 公司内部 PDF 压缩工具。拖一个 PDF 进窗口，输出到原目录的 `<名称>_compressed.pdf`。

## 下载与首次打开

### macOS
1. 下载对应芯片的 `.dmg` 文件（Apple Silicon → `aarch64`，Intel → `x64`）
2. 拖入 Applications
3. **首次打开**：右键应用图标 → 打开 → 在弹窗中再次"打开"
4. 此后双击即可

### Windows
1. 下载 `PDFShrink_<version>_x64-setup.exe`
2. 双击安装
3. **首次打开** SmartScreen 蓝屏：点 "更多信息" → "仍要运行"

## 使用

把 PDF 拖到窗口里。可选档位：
- **标准 (150dpi)**：默认，体积/质量平衡，扫描件首选
- **高质量 (200dpi)**：保留更多细节
- **极致压缩 (100dpi)**：最大限度缩小
- **自定义**：50–600 dpi

## 从源码构建

需要 Node 20+、Rust 1.78+、Ghostscript（开发依赖，会被打包脚本取代）。

```bash
git clone <repo>
cd pdfshrink
npm install
bash scripts/fetch-gs.sh
npm run tauri dev      # 开发
npm run tauri build    # 出安装包
```

## 许可证

应用代码：MIT
Ghostscript（内置）：AGPL v3，详见 [LICENSE-Ghostscript.txt](./LICENSE-Ghostscript.txt)
