#!/usr/bin/env python3
"""Check that src/ui/ only uses Colony colours that every theme defines.

src/ui/vendor/colony/colony-themes.css sets the --colony-* variables once per
theme, and themes.json lists the themes the picker offers. A variable one
theme lacks leaves that colour unset in that theme, so this fails on it, and
on a picker entry with no CSS block or a CSS block with no picker entry.
colony-accents.css sets the accent colours once for every theme, so a
variable it defines counts as defined everywhere, and every accent the
picker offers must be one of them.
"""

import json
import re
import sys
from pathlib import Path

ui = Path(__file__).resolve().parents[2] / "src" / "ui"
vendor = ui / "vendor" / "colony"


def variables(css_file):
    """{selector: set of --colony-* names}, comments out first: the file
    header quotes a selector."""
    css = re.sub(r"/\*.*?\*/", "", (vendor / css_file).read_text(encoding="utf-8"), flags=re.S)
    return {
        selector.strip(): set(re.findall(r"(--colony-[a-z0-9-]+):", body))
        for selector, body in re.findall(r"([^{}]+)\{([^}]*)\}", css)
    }


themes = {
    re.search(r'\[data-colony-theme="([^"]+)"\]', selector).group(1): defined
    for selector, defined in variables("colony-themes.css").items()
    if "[data-colony-theme=" in selector
}
accents = variables("colony-accents.css").get(":root", set())
theme_list = json.loads((vendor / "themes.json").read_text(encoding="utf-8"))
listed = {variant["slug"] for family in theme_list["families"] for variant in family["variants"]}

errors = []
if not themes or set(themes) != listed:
    errors.append(f"themes.json and colony-themes.css disagree: {sorted(set(themes) ^ listed)}")
for accent in theme_list["accents"]:
    if accent["cssVar"] not in accents:
        errors.append(f"themes.json: accent {accent['key']} has no {accent['cssVar']} in colony-accents.css")
for path in sorted(ui.glob("*.*")):
    for var in sorted(set(re.findall(r"--colony-[a-z0-9-]+", path.read_text(encoding="utf-8"))) - accents):
        missing = [slug for slug, defined in themes.items() if var not in defined]
        if missing:
            errors.append(f"{path.name}: {var} is missing from {', '.join(missing)}")

print("\n".join(errors) or f"OK: all {len(themes)} themes define every --colony-* variable in src/ui/")
sys.exit(1 if errors else 0)
