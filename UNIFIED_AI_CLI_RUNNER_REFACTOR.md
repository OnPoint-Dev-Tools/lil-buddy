# Unified AI CLI Runner Refactor

This pass moves process management out of `commands/providers.rs` and into:

```txt
src-tauri/src/providers/unified_cli.rs
```

## What this gives Lil Buddy

- One place to spawn CLI processes.
- One place to stream stdout/stderr.
- One place to normalize events.
- One place to run safety scanning.
- One place to start/stop live workspace watching.
- One place to emit post-run diff summaries.

## Provider adapters

Each provider now has an adapter path:

```txt
OpenCodeGoAdapter → src-tauri/src/providers/opencode_go.rs
ClaudeAdapter     → src-tauri/src/providers/claude.rs
CodexAdapter      → src-tauri/src/providers/codex.rs
GeminiAdapter     → src-tauri/src/providers/gemini.rs
```

The UI no longer needs to care how each CLI wants to receive prompts.
