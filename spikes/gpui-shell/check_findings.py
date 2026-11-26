#!/usr/bin/env python3
"""Check the closing 'For temet-work' section of docs/findings/gpui.md.

It must be the last section, list Build on, Avoid and Still unknown for both the
daemon and the app, and every item must end by naming earlier sections of the
document as its evidence: "(Build times, Versions)."
"""

import pathlib
import re
import sys

doc = pathlib.Path(__file__).resolve().parents[2] / "docs/findings/gpui.md"
text = doc.read_text()
errors = []

sections = re.findall(r"^## (.+)$", text, re.M)
if not sections or sections[-1] != "For temet-work":
    errors.append("'For temet-work' is not the last section")
before, _, closing = text.partition("\n## For temet-work\n")
earlier = set(re.findall(r"^##+ (.+)$", before, re.M))

parts = dict(re.findall(r"^### ([^\n]+)\n(.*?)(?=^### |\Z)", closing, re.M | re.S))
for part in ("The daemon", "The app"):
    lists = dict(
        re.findall(
            r"^(Build on|Avoid|Still unknown):\n\n(.*?)(?=^\S[^\n]*:\n\n|\Z)",
            parts.get(part, ""),
            re.M | re.S,
        )
    )
    for name in ("Build on", "Avoid", "Still unknown"):
        items = [
            " ".join(item.split())
            for item in re.split(r"^- ", lists.get(name, ""), flags=re.M)
            if item.strip()
        ]
        if not items:
            errors.append(f"{part} / {name}: no items")
        for item in items:
            cited = re.search(r"\(([^()]+)\)\.$", item)
            names = cited.group(1).split(", ") if cited else []
            unknown = [n for n in names if n not in earlier]
            if not names or unknown:
                errors.append(f"{part} / {name}: no earlier section cited: {item}")
            else:
                print(f"ok  {part} / {name}: {item[:60]}... -> {', '.join(names)}")

for error in errors:
    print("FAIL", error)
print(f"{len(errors)} failure(s)")
sys.exit(1 if errors else 0)
