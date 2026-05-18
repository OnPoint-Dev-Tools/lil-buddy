# Unified AI CLI Framework Direction

Yes — this is the right direction.

Lil Buddy should not treat Claude, Codex, Gemini, and OpenCode as unrelated one-off process calls. They should sit behind one Rust adapter layer.

## Target architecture

```txt
React UI
  ↓
Tauri command
  ↓
Unified AI CLI runner
  ↓
Provider adapter
  ├─ opencode-go provider identity → opencode binary
  ├─ claude provider identity      → claude binary
  ├─ codex provider identity       → codex binary
  └─ gemini provider identity      → gemini binary
  ↓
Normalized Lil Buddy events
```

## Why this is better

- One process runner.
- One event model.
- One safety classifier.
- One workspace/diff watcher.
- Provider-specific command differences stay isolated.
- Lil Buddy can add providers later without rewriting the UI.

## Added in this pass

```txt
src-tauri/src/providers/unified_cli.rs
```

This is the seed of the unified framework. The existing adapters still work, but the next refactor should move each provider behind the `AiCliAdapter` trait.
