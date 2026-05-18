#!/usr/bin/env python3
from pathlib import Path
from PIL import Image, ImageChops
import sys

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "src/assets/companions"
COMPANIONS = ['blue-hoodie', 'yellow-rain', 'tan-explorer', 'red-beanie', 'pink-hood', 'lavender-bear', 'green-scout', 'autumn-vest']
TARGET_SIZE = 192
MIN_TOP_CLEARANCE = 10
MIN_BOTTOM_CLEARANCE = 10

def changed_enough(a, b):
    diff = ImageChops.difference(a.convert("RGBA"), b.convert("RGBA"))
    changed = 0
    for r, g, b, a in diff.getdata():
        if r > 10 or g > 10 or b > 10 or a > 10:
            changed += 1
    return changed > 120

def validate_folder(folder):
    files = sorted(folder.glob("*.png"))
    failures = []
    if len(files) != 8:
        return [f"{folder}: expected 8 frames, found {len(files)}"]
    images = [Image.open(file).convert("RGBA") for file in files]
    diff_count = sum(1 for image in images[1:] if changed_enough(images[0], image))
    if diff_count < 4:
        failures.append(f"{folder}: too many static frames; only {diff_count}/7 differ")
    for file, image in zip(files, images):
        if image.size != (TARGET_SIZE, TARGET_SIZE):
            failures.append(f"{file}: expected {TARGET_SIZE}x{TARGET_SIZE}, found {image.size}")
        bbox = image.getchannel("A").getbbox()
        if not bbox:
            failures.append(f"{file}: empty frame")
            continue
        if bbox[1] < MIN_TOP_CLEARANCE:
            failures.append(f"{file}: too close to top; {bbox[1]}px")
        if TARGET_SIZE - bbox[3] < MIN_BOTTOM_CLEARANCE:
            failures.append(f"{file}: too close to bottom; {TARGET_SIZE - bbox[3]}px")
    return failures

def main():
    failures = []
    for companion in COMPANIONS:
        for state in ["walk-left", "walk-right"]:
            failures.extend(validate_folder(BASE / f"{companion}-v2" / state))
    if failures:
        print("Walk pose validation failed:")
        for failure in failures:
            print(" -", failure)
        return 1
    print("Walk pose validation passed.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
