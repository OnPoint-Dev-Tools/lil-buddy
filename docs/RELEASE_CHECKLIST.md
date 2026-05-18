# Lil Buddy v0.1 Developer Preview Release Checklist

## Product polish

- [x] Settings sections match all themes
- [x] Dark mode works across all modals/dropdowns
- [x] Default directory behavior is safe
- [x] No automatic workspace scan/diff on app boot
- [x] Telegram gateway start/stop has clear success/error messages
- [x] Reset settings / safe mode exists or is documented
- [x] First-run onboarding explains local-first behavior
- [x] No runtime events leak into normal chat messages
- [x] Experts and chat tabs persist correctly
- [x] Companion hide/show tray actions work

## Safety

- [x] Secrets are not committed
- [x] Telegram bot token is not logged
- [x] Workspace paths are not auto-scanned
- [x] Risky command approvals are visible
- [x] Privacy note is included
- [x] Security contact is set in SECURITY.md

## Packaging

- [ ] Windows build tested
- [ ] macOS build tested
- [ ] Linux build tested
- [ ] App icon and tray icon checked
- [ ] Version number set
- [ ] Changelog updated
- [ ] Checksums generated

## Signing

- [ ] macOS signing investigated
- [ ] macOS notarization investigated
- [ ] Windows signing certificate investigated
- [ ] Unsigned build warnings documented if signing is not ready

## Repository

- [x] README complete
- [x] LICENSE present
- [x] ASSET_LICENSE present
- [x] CONTRIBUTING present
- [x] SECURITY present
- [x] PRIVACY present
- [x] FUNDING placeholders updated
- [x] Issue templates added
- [ ] Screenshots added
- [ ] Demo GIF/video added

