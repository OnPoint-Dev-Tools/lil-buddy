# Packaging Lil Buddy for Users

Lil Buddy is a Tauri v2 desktop app. The simple path is to build installers on each operating system you want to support.

## Recommended release artifacts

For a clean Developer Preview release, publish these from GitHub Releases:

```txt
Windows: Lil-Buddy_0.1.0_x64-setup.exe  (NSIS installer)
Windows: Lil-Buddy_0.1.0_x64_en-US.msi  (optional MSI)
macOS:   Lil-Buddy_0.1.0_aarch64.dmg    (Apple Silicon)
macOS:   Lil-Buddy_0.1.0_x64.dmg        (Intel)
Linux:   Lil-Buddy_0.1.0_amd64.AppImage
Linux:   Lil-Buddy_0.1.0_amd64.deb
Linux:   Lil-Buddy_0.1.0_x86_64.rpm
Checksums: SHA256SUMS.txt
```

## Local build

Install dependencies:

```bash
npm install
```

Build the desktop app:

```bash
npm run build:desktop
```

or directly:

```bash
npm run tauri build
```

The bundled installers will be under:

```txt
src-tauri/target/release/bundle/
```

## Platform notes

### Windows

Build on Windows for the smoothest result.

Building Windows installers from Linux is not the recommended path for this Tauri app. In practice, you usually want a real Windows machine or a GitHub Actions Windows runner for `.exe` / `.msi` output.

Expected output folders:

```txt
src-tauri/target/release/bundle/nsis/
src-tauri/target/release/bundle/msi/
```

For users, the easiest file is usually the NSIS setup `.exe`.

### 1. Install Rust

 In PowerShell:

 ```powershell
   winget install Rustlang.Rustup
 ```

 or use:

- [https://rustup.rs](https://rustup.rs)
 Then install the MSVC toolchain:

 ```powershell
   rustup toolchain install stable-x86_64-pc-windows-msvc
   rustup default stable-x86_64-pc-windows-msvc

 ```

### 2. Install Visual Studio C++ build tools

 Tauri on Windows also needs MSVC tools.
 Install Visual Studio Build Tools with:

- Desktop development with C++
- Windows SDK

 Quick way:

 ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools
 ```

### 3. Restart terminal / VS Code

 Important: PATH often won’t update until you reopen VS Code or the terminal.

### 4. Verify

 Run:

 ```powershell
   cargo --version
   rustc --version

 ```

 If cargo fails, PATH still isn’t loaded.

### 5. Then build

 ```powershell
   npm install
   npm run build:desktop
 ```

### macOS

Build on macOS. For public users, signed and notarized `.dmg` releases are best.

Expected output folders:

```txt
src-tauri/target/release/bundle/dmg/
src-tauri/target/release/bundle/macos/
```

### Linux

Build on Linux.

Expected output folders:

```txt
src-tauri/target/release/bundle/appimage/
src-tauri/target/release/bundle/deb/
src-tauri/target/release/bundle/rpm/
```

For broad testing, AppImage is the easiest single file. `.deb` is good for Debian/Ubuntu users. `.rpm` is good for Fedora/openSUSE users.

If AppImage packaging fails on Arch or another rolling distro because the bundled `linuxdeploy` tool cannot strip newer ELF files, use:

```bash
npm run package:linux:appimage
```

That script lets Tauri stage the AppDir, then repacks it with `linuxdeploy-plugin-appimage`, which avoids the failing strip step.

## Checksums

After building, generate checksums from the folder where you collect release files:

```bash
sha256sum * > SHA256SUMS.txt
```
