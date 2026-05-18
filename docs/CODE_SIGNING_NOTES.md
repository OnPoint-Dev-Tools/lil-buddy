# Code signing notes

For a private test build, unsigned apps are acceptable.

For public users, signing is strongly recommended.

## macOS

You generally need:

- Apple Developer account
- Developer ID Application certificate
- app signing
- notarization
- stapling

Unsigned macOS apps may trigger Gatekeeper warnings.

## Windows

You generally need:

- code signing certificate
- signed installer/executable
- time stamping

Unsigned Windows apps may trigger SmartScreen warnings.

## Linux

Linux distribution is less centralized. Consider:

- AppImage / deb / rpm
- checksums
- signed package repositories later
- clear install docs

## Developer Preview note

If signing is not ready, clearly label downloads as experimental unsigned Developer Preview builds and explain expected OS warnings.
