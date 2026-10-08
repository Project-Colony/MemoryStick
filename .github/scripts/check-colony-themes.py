#!/usr/bin/env python3
"""Check that dist/ only uses Colony colours that every theme defines.

dist/vendor/colony/colony-themes.css sets the --colony-* variables once per
theme, and themes.json lists the themes the picker offers. A variable one
theme lacks leaves that colour unset in that theme, so this fails on it, and
on a picker entry with no CSS block or a CSS block with no picker entry.
"""

import json
import re
import sys
from pathlib import Path

dist = Path(__file__).resolve().parents[2] / "dist"
vendor = dist / "vendor" / "colony"

# Comments out first: the file header quotes a selector.
css = re.sub(r"/\*.*?\*/", "", (vendor / "colony-themes.css").read_text(encoding="utf-8"), flags=re.S)
themes = {
    slug: set(re.findall(r"(--colony-[a-z0-9-]+):", body))
    for slug, body in re.findall(r'\[data-colony-theme="([^"]+)"\][^{]*\{([^}]*)\}', css)
}
listed = {
    variant["slug"]
    for family in json.loads((vendor / "themes.json").read_text(encoding="utf-8"))["families"]
    for variant in family["variants"]
}

errors = []
if not themes or set(themes) != listed:
    errors.append(f"themes.json and colony-themes.css disagree: {sorted(set(themes) ^ listed)}")
for path in sorted(dist.glob("*.*")):
    for var in sorted(set(re.findall(r"--colony-[a-z0-9-]+", path.read_text(encoding="utf-8")))):
        missing = [slug for slug, defined in themes.items() if var not in defined]
        if missing:
            errors.append(f"{path.name}: {var} is missing from {', '.join(missing)}")

print("\n".join(errors) or f"OK: all {len(themes)} themes define every --colony-* variable in dist/")
sys.exit(1 if errors else 0)
