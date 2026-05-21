# Lil Buddy Agent Notes

## Stack And Entrypoints
- This repo is a single-package Tauri v2 desktop app, not a monorepo.
- Frontend entrypoint: `src/main.tsx` -> `src/App.tsx` -> `src/components/chat/ChatWindow.tsx`.
- Rust entrypoint: `src-tauri/src/main.rs`. It sets up the tray, starts the native companion manager, optionally auto-starts Telegram, and registers all Tauri commands.
- Runtime UI updates flow through the single Tauri event channel `runtime://stream` (`src/lib/tauri/events.ts`). If you change backend event shapes or names, update the frontend listeners and message handling too.

## Commands
- Install deps: `npm install`
- Frontend-only dev server: `npm run dev` (Vite on port `1437`)
- Full desktop dev app: `npm run tauri dev` or `npm run dev:desktop`
- Frontend build/typecheck: `npm run build`
- Desktop production build: `npm run build:desktop`
- Linux Wayland fallbacks are explicit scripts, not autodetected by npm:
  - `npm run tauri:wayland-safe`
  - `npm run tauri:x11`
- There are currently no repo-level `test`, `lint`, `format`, or dedicated `typecheck` scripts in `package.json`. Do not claim you ran them unless you added them.

## Config Coupling
- Vite port `1437` is duplicated in `package.json`, `vite.config.ts`, and `src-tauri/tauri.conf.json` (`devUrl`). Keep them in sync.
- `src-tauri/tauri.conf.json` uses `beforeDevCommand: "npm run dev"`; breaking the Vite dev command also breaks Tauri dev startup.

## Settings And Workspace Behavior
- Persisted app settings live in Tauri's app config dir as `settings.json`, not in the repo (`src-tauri/src/settings/mod.rs`).
- `.env` is only a fallback source for Telegram fields. Saved settings take precedence, and env-backed values are stripped before settings are persisted.
- `save_workspace_path` and `save_default_directory` normalize paths to the Git repo root when possible (`src-tauri/src/commands/settings.rs`). If a path behaves unexpectedly, check the normalized stored value first.
- Workspace fallback order matters across the Rust side: explicit workspace -> default directory -> OS home/current dir fallback.
- Git detection and diffs intentionally do not run when no explicit workspace is selected; the app may still show a fallback path while reporting `workspace not selected` (`src-tauri/src/workspace/mod.rs`).

## Provider Rules
- The only supported provider IDs are `opencode-go` and `claude`. Unknown values are coerced back to `opencode-go` in Rust.
- Provider command defaults come from persisted settings: `opencode_command` defaults to `opencode`, `claude_command` defaults to `claude`.
- OpenCode model listing is shell-driven from `opencode models` and `opencode models --refresh`; fallback models are hardcoded in Rust if parsing fails.

## UI State Gotchas
- `ChatWindow.tsx` is the real integration hub: provider detection, workspace selection, approvals, runtime stream hookup, expert sessions, and Telegram expert syncing all meet there. For behavior changes, inspect this file before editing smaller leaf components.
- The Zustand store in `src/stores/appStore.ts` deliberately strips protocol-style JSON noise and deduplicates/merges recent stream messages. If streamed output looks duplicated or missing, check store normalization before changing backend emitters.

## Packaging
- `npm run package:linux:appimage` is a fallback path for Linux packaging failures after Tauri stages the AppDir.
- That script requires `src-tauri/icons/icon.icns` to exist and expects `~/.cache/tauri/linuxdeploy-plugin-appimage.AppImage` to already be populated by a prior Tauri build.
