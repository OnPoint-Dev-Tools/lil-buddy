#!/usr/bin/env bash
# Generate SHA256 checksums for release artifacts
# Usage: ./scripts/generate-checksums.sh [bundle-dir]

set -euo pipefail

BUNDLE_DIR="${1:-src-tauri/target/release/bundle}"
OUTPUT="SHA256SUMS"

if [ ! -d "$BUNDLE_DIR" ]; then
  echo "Error: bundle directory not found: $BUNDLE_DIR"
  echo "Run 'npm run build:desktop' first to build release artifacts."
  exit 1
fi

cd "$(dirname "$0")/.."

echo "# SHA256 checksums for Lil Buddy v$(node -p "require('./package.json').version")" > "$OUTPUT"
echo "# Generated on $(date -u +"%Y-%m-%dT%H:%M:%SZ")" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Find all package files recursively
while IFS= read -r -d '' file; do
  filename=$(basename "$file")
  checksum=$(sha256sum "$file" | cut -d' ' -f1)
  echo "$checksum  $filename" >> "$OUTPUT"
  echo "  $filename"
done < <(find "$BUNDLE_DIR" -type f \( \
  -name "*.deb" -o \
  -name "*.rpm" -o \
  -name "*.AppImage" -o \
  -name "*.msi" -o \
  -name "*.exe" -o \
  -name "*.dmg" \
\) -print0 2>/dev/null)

if [ ! -s "$OUTPUT" ]; then
  echo "No release artifacts found in $BUNDLE_DIR"
  rm "$OUTPUT"
  exit 1
fi

echo ""
echo "Checksums written to $OUTPUT"
