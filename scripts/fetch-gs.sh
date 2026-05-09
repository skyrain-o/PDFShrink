#!/usr/bin/env bash
# Fetch Ghostscript binary + resources for the current host platform
# and stage into src-tauri/binaries/ and src-tauri/resources/gs-lib/.
#
# Usage: bash scripts/fetch-gs.sh
#
# Requires: brew (macOS) or chocolatey (Windows via Git Bash).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN_DIR="$ROOT/src-tauri/binaries"
RES_DIR="$ROOT/src-tauri/resources/gs-lib"
mkdir -p "$BIN_DIR" "$RES_DIR"

uname_s="$(uname -s)"
uname_m="$(uname -m)"

case "$uname_s" in
  Darwin)
    case "$uname_m" in
      arm64)  TRIPLE="aarch64-apple-darwin" ;;
      x86_64) TRIPLE="x86_64-apple-darwin" ;;
      *) echo "Unsupported mac arch: $uname_m" >&2; exit 1 ;;
    esac
    if ! command -v gs >/dev/null; then
      echo "Installing ghostscript via brew..."
      brew install ghostscript
    fi
    GS_BIN="$(command -v gs)"
    GS_PREFIX="$(brew --prefix ghostscript)"
    # Some Homebrew layouts nest Resource under a version dir (share/ghostscript/X.Y.Z/Resource);
    # newer/current layouts place it directly under share/ghostscript/Resource. Try both.
    GS_VER_DIR="$(ls "$GS_PREFIX/share/ghostscript" 2>/dev/null | grep -E '^[0-9]+\.' | head -1 || true)"
    if [[ -n "$GS_VER_DIR" && -d "$GS_PREFIX/share/ghostscript/$GS_VER_DIR/Resource" ]]; then
      GS_RES="$GS_PREFIX/share/ghostscript/$GS_VER_DIR/Resource"
    else
      GS_RES="$GS_PREFIX/share/ghostscript/Resource"
    fi
    ;;
  MINGW*|MSYS*|CYGWIN*)
    TRIPLE="x86_64-pc-windows-msvc"
    if ! ls /c/Program\ Files/gs/gs*/bin/gswin64c.exe 2>/dev/null | head -1 >/dev/null; then
      if ! command -v choco >/dev/null; then
        echo "Need chocolatey: https://chocolatey.org/install" >&2; exit 1
      fi
      echo "Installing ghostscript via chocolatey..."
      choco install ghostscript --version=10.07.0 -y --no-progress
    fi
    GS_BIN="$(ls /c/Program\ Files/gs/gs*/bin/gswin64c.exe | head -1)"
    GS_RES="$(dirname "$(dirname "$GS_BIN")")/Resource"
    ;;
  *)
    echo "Unsupported OS: $uname_s" >&2; exit 1 ;;
esac

# Stage binary
TARGET_NAME="gs-$TRIPLE"
[[ "$uname_s" == MINGW* || "$uname_s" == MSYS* || "$uname_s" == CYGWIN* ]] && TARGET_NAME="${TARGET_NAME}.exe"
cp "$GS_BIN" "$BIN_DIR/$TARGET_NAME"
chmod +x "$BIN_DIR/$TARGET_NAME"
echo "Binary -> $BIN_DIR/$TARGET_NAME"

# Stage Resource/ (full, including CMap per design)
rm -rf "$RES_DIR/Resource"
cp -R "$GS_RES" "$RES_DIR/Resource"
echo "Resources -> $RES_DIR/Resource ($(du -sh "$RES_DIR/Resource" | cut -f1))"

# Smoke test: run staged binary against a fixture (if present)
FIX="$ROOT/src-tauri/tests/fixtures/english_text.pdf"
if [[ -f "$FIX" ]]; then
  OUT="$(mktemp).pdf"
  GS_LIB="$RES_DIR/Resource/Init:$RES_DIR/Resource/Font" \
    "$BIN_DIR/$TARGET_NAME" -sDEVICE=pdfwrite -dCompatibilityLevel=1.5 \
    -dNOPAUSE -dQUIET -dBATCH -dPDFSETTINGS=/ebook \
    -sOutputFile="$OUT" "$FIX"
  if [[ -s "$OUT" ]]; then
    echo "Smoke test OK ($(stat -f%z "$OUT" 2>/dev/null || stat -c%s "$OUT") bytes)"
  else
    echo "Smoke test FAILED" >&2; exit 1
  fi
fi
