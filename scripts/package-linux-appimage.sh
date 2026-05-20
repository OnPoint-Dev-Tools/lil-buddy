#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

APP_NAME="Lil Buddy"
ARCH="x86_64"
APPIMAGE_DIR="$ROOT_DIR/src-tauri/target/release/bundle/appimage"
APPDIR="$APPIMAGE_DIR/${APP_NAME}.AppDir"
APPIMAGE_OUTPUT="$APPIMAGE_DIR/${APP_NAME}_0.1.0_amd64.AppImage"
PLUGIN_APPIMAGE="$HOME/.cache/tauri/linuxdeploy-plugin-appimage.AppImage"

if [[ ! -f "$ROOT_DIR/src-tauri/icons/icon.icns" ]]; then
  echo "Missing src-tauri/icons/icon.icns"
  echo "Generate it once with: npx tauri icon src-tauri/icons/app-icon.png -o src-tauri/icons"
  exit 1
fi

echo "[1/3] Building frontend"
npm run build

echo "[2/3] Letting Tauri stage the AppDir"
if ! npx tauri build --bundles appimage; then
  echo "Tauri AppImage bundling failed; continuing with AppImageKit workaround"
fi

if [[ ! -d "$APPDIR" ]]; then
  echo "Expected AppDir was not created: $APPDIR"
  exit 1
fi

if [[ ! -x "$PLUGIN_APPIMAGE" ]]; then
  echo "Missing AppImage packager: $PLUGIN_APPIMAGE"
  echo "Run the Tauri build once to populate ~/.cache/tauri, then retry."
  exit 1
fi

echo "[3/3] Repacking AppDir into AppImage"
ln -sf "usr/share/icons/hicolor/256x256@2/apps/lil-buddy.png" "$APPDIR/lil-buddy.png"
ln -sf "usr/share/applications/${APP_NAME}.desktop" "$APPDIR/lil-buddy.desktop"

ARCH="$ARCH" \
LDAI_OUTPUT="$APPIMAGE_OUTPUT" \
"$PLUGIN_APPIMAGE" --appimage-extract-and-run --appdir "$APPDIR"

echo "Built AppImage: $APPIMAGE_OUTPUT"
