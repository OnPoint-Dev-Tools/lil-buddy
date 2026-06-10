# Code Signing Notes

## Status: Developer Preview

This release is an **unsigned Developer Preview**. Operating systems may show security warnings when running unsigned binaries. This is expected and documented below.

---

### Unsigned build warnings

**When running an unsigned Lil Buddy on Windows:**

1. **SmartScreen**: You'll see "Windows protected your PC" with a "More info" link. Click "More info" → "Run anyway".
2. **Browser download warning**: Chrome/Edge may flag `.exe` downloads. Users should click "Keep" in the downloads bar.
3. **Antivirus**: Some AV software may quarantine the file. Users may need to add an exception or restore from quarantine.

We publish SHA256 checksums (`SHA256SUMS`) so users can verify the file hasn't been tampered with:

```powershell
certutil -hashfile .\Lil-Buddy_0.1.0_x64-setup.exe SHA256
```

---

## Linux Signing

### Status: Checksums only

Linux desktop apps typically don't use code signing certificates. Instead:

- **Checksums** are published for verification (`SHA256SUMS`)
- **Package repository signing** (apt/yum GPG keys) is the standard path for `.deb`/`.rpm` distribution

For the Developer Preview, SHA256 checksums are sufficient. GPG package signing can be added when publishing to a package repository.

---
