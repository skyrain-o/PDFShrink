import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

type Preset =
  | "standard"
  | "high_quality"
  | "extreme"
  | { custom: number };

type Report = { input_size: number; output_size: number; output_path: string };
type UserError = { kind: string; message: string };

const dropzone = document.getElementById("dropzone")!;
const statusEl = document.getElementById("status")!;
const customDpi = document.getElementById("custom-dpi") as HTMLInputElement;

function selectedPreset(): Preset {
  const r = document.querySelector<HTMLInputElement>('input[name="preset"]:checked')!;
  if (r.value === "custom") {
    return { custom: parseInt(customDpi.value || "150", 10) };
  }
  return r.value as Preset;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

function setStatus(html: string, kind: "info" | "ok" | "err" = "info") {
  statusEl.className = `status status-${kind}`;
  statusEl.innerHTML = html;
}

async function handlePath(path: string) {
  if (!path.toLowerCase().endsWith(".pdf")) {
    setStatus(`请拖入 PDF 文件（当前文件：${path.split(/[\/\\]/).pop()}）`, "err");
    return;
  }
  dropzone.classList.add("busy");
  setStatus("处理中…", "info");
  try {
    const r = await invoke<Report>("compress_pdf", { path, preset: selectedPreset() });
    const saved = (1 - r.output_size / r.input_size) * 100;
    setStatus(
      `<strong>✓ 完成</strong> ${formatSize(r.input_size)} → ${formatSize(r.output_size)}（节省 ${saved.toFixed(0)}%）
       <button id="reveal">在 Finder 中显示</button>`,
      "ok"
    );
    document.getElementById("reveal")?.addEventListener("click", () => {
      revealItemInDir(r.output_path);
    });
  } catch (raw) {
    const e = raw as UserError;
    setStatus(e.message ?? "未知错误", "err");
  } finally {
    dropzone.classList.remove("busy");
  }
}

dropzone.addEventListener("click", async () => {
  const picked = await openDialog({
    multiple: false,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (typeof picked === "string") handlePath(picked);
});

getCurrentWebview().onDragDropEvent((event) => {
  if (event.payload.type === "drop") {
    const paths = event.payload.paths;
    if (paths.length === 0) return;
    if (paths.length > 1) {
      setStatus(`当前版本只支持单个文件，已忽略其余 ${paths.length - 1} 个`, "info");
    }
    const sorted = [...paths].sort();
    handlePath(sorted[0]);
  }
});

customDpi.addEventListener("input", () => {
  const v = parseInt(customDpi.value, 10);
  if (isNaN(v) || v < 50 || v > 600) {
    customDpi.setCustomValidity("dpi 必须在 50–600 之间");
  } else {
    customDpi.setCustomValidity("");
  }
});

setStatus("拖入一个 PDF 开始", "info");
