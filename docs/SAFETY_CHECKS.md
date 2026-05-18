# Release safety checks

The following release safety items are enforced or documented in code.

## Secrets are not committed

`.gitignore` excludes local environment files, private keys, logs, local secret folders, Telegram token exports, build artifacts, and editor/OS noise.

Important: still review `git status` before publishing.

## Telegram bot token is not logged

Telegram gateway runtime messages do not print the bot token. Gateway error paths redact the bot token, webhook secret, and webhook path secret before surfacing startup/polling errors.

## Workspace paths are not auto-scanned

Workspace detection/diff is intentionally explicit. `refreshWorkspace()` is not called on app boot or expert switch. Users must select/refresh a workspace intentionally.

The workspace-selected runtime event no longer prints the full local path.

## Risky command approvals are visible

`ApprovalConfirm` remains mounted in the chat window and must display pending risky command/directory approvals before execution.
