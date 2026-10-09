#!/usr/bin/env python3
"""Check that render() in dist/app.js still drops every element that could act
on MemoryStick's own page.

comrak keeps a document's raw HTML as written, and the inert <template> pass
in render() is the only place these elements are removed. This fails if one
of them is missing from DROPPED_ELEMENTS, or if render() stops using it.
"""

import re
import sys
from pathlib import Path

REQUIRED = {"meta", "link", "iframe", "frame", "frameset", "object", "embed", "script", "base", "style"}
# Ordinary raw HTML a document is allowed to keep, as on GitHub.
KEPT = {"kbd", "details", "summary", "sub", "sup", "svg", "img", "a"}

app = (Path(__file__).resolve().parents[2] / "dist" / "app.js").read_text(encoding="utf-8")

errors = []
match = re.search(r"^const DROPPED_ELEMENTS = '([^']*)';$", app, flags=re.M)
if not match:
    errors.append("dist/app.js: no `const DROPPED_ELEMENTS = '...';` line")
else:
    dropped = {name.strip() for name in match.group(1).split(",")}
    if missing := REQUIRED - dropped:
        errors.append(f"DROPPED_ELEMENTS no longer drops: {', '.join(sorted(missing))}")
    if wrongly := KEPT & dropped:
        errors.append(f"DROPPED_ELEMENTS drops ordinary HTML: {', '.join(sorted(wrongly))}")
if ".querySelectorAll(DROPPED_ELEMENTS).forEach((el) => el.remove());" not in app:
    errors.append("dist/app.js: render() no longer removes DROPPED_ELEMENTS from the document")

print("\n".join(errors) or f"OK: render() drops all {len(REQUIRED)} elements")
sys.exit(1 if errors else 0)
