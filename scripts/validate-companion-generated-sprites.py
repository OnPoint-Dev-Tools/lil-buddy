#!/usr/bin/env python3
from pathlib import Path
from PIL import Image, ImageChops
import sys

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "src/assets/companions"

def different_enough(a: Image.Image, b: Image.Image) -> bool:
    diff = ImageChops.difference(a.convert("RGBA"), b.convert("RGBA"))
    if not diff.getbbox():
        return False
    changed = 0
    for r, g, b, a in diff.getdata():
        if r > 10 or g > 10 or b > 10 or a > 10:
            changed += 1
    return changed > 140

def check_state(folder: Path, required=True):
    files = sorted(folder.glob("*.png"))
    if required and len(files) < 2:
        return [f"{folder}: missing frames"]
    if len(files) < 2:
        return []

    images = [Image.open(file) for file in files]
    first = images[0]
    diff_count = sum(1 for img in images[1:] if different_enough(first, img))

    # Some intentional static event folders may repeat a generated pose, but idle/walk must move.
    state = folder.name
    if state in {"idle", "walk-left", "walk-right"} and diff_count == 0:
        return [f"{folder}: {state} frames are visually identical"]

    return []

def main():
    targets = [
        "tan-explorer",
        "pink-hood",
        "autumn-vest",
        "green-scout",
        "blue-hoodie",
        "lavender-bear",
        "yellow-rain",
        "red-beanie",
    ]

    failures = []
    for target in targets:
        companion = BASE / f"{target}-v2"
        if not companion.exists():
            failures.append(f"{companion}: missing")
            continue
        for state in ["idle", "hello", "thinking", "working", "command", "done", "error", "walk-left", "walk-right"]:
            failures.extend(check_state(companion / state))

    if failures:
        print("Companion sprite validation failed:")
        for failure in failures:
            print(" -", failure)
        return 1

    print("Companion sprite validation passed.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
