# Security Policy

Lil Buddy is a local-first desktop AI helper in Developer Preview.

## Supported versions

Only the latest Developer Preview is supported right now.

## Reporting a vulnerability

Please do not open public issues for serious security problems.

Instead, contact the maintainer privately. Add your contact email here before publishing:

```txt
security contact: cj@onpointwebstudio.com
```

## Sensitive areas

Please pay special attention to:

- provider command execution
- workspace folder access
- Telegram gateway tokens
- webhook secret handling
- file restore/diff features
- shell command approvals
- local settings storage

## Safety principles

- The app should not scan or diff user folders automatically.
- The app should not sync secrets by default.
- The app should make risky actions visible.
- Local-first usage should remain possible.

