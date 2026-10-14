#!/usr/bin/env python3
"""Check that src/ui/ only uses Colony colours that every theme defines.

src/ui/vendor/colony/colony-themes.css sets the --colony-* variables once per
theme, and themes.json lists the themes the picker offers. A variable one
theme lacks leaves that colour unset in that theme, so this fails on it, and
on a picker entry with no CSS block or a CSS block with no picker entry.
colony-accents.css sets the accent colours once for every theme, so a
variable it defines counts as defined everywhere, and every accent the
picker offers must be one of them.

Text on an accent fill (the selected contents entry, category and choice)
takes --colony-ink or --colony-paper, whichever contrasts more, as theme.js
does. That must reach 4.0:1 on every theme's own accent and on every accent
override.
"""

import json
import re
import sys
from pathlib import Path

ui = Path(__file__).resolve().parents[2] / "src" / "ui"
vendor = ui / "vendor" / "colony"


def variables(css_file):
    """{selector: {--colony-* name: value}}, comments out first: the file
    header quotes a selector."""
    css = re.sub(r"/\*.*?\*/", "", (vendor / css_file).read_text(encoding="utf-8"), flags=re.S)
    return {
        selector.strip(): dict(re.findall(r"(--colony-[a-z0-9-]+):\s*([^;]+);", body))
        for selector, body in re.findall(r"([^{}]+)\{([^}]*)\}", css)
    }


def contrast(a, b):
    """WCAG contrast ratio of two #rrggbb colours, as theme.js computes it."""

    def luminance(color):
        r, g, b = (
            c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4
            for c in (int(color[i : i + 2], 16) / 255 for i in (1, 3, 5))
        )
        return 0.2126 * r + 0.7152 * g + 0.0722 * b

    hi, lo = sorted((luminance(a), luminance(b)), reverse=True)
    return (hi + 0.05) / (lo + 0.05)


themes = {
    re.search(r'\[data-colony-theme="([^"]+)"\]', selector).group(1): defined
    for selector, defined in variables("colony-themes.css").items()
    if "[data-colony-theme=" in selector
}
accents = variables("colony-accents.css").get(":root", {})
theme_list = json.loads((vendor / "themes.json").read_text(encoding="utf-8"))
listed = {variant["slug"] for family in theme_list["families"] for variant in family["variants"]}

errors = []
if not themes or set(themes) != listed:
    errors.append(f"themes.json and colony-themes.css disagree: {sorted(set(themes) ^ listed)}")
for accent in theme_list["accents"]:
    if accent["cssVar"] not in accents:
        errors.append(f"themes.json: accent {accent['key']} has no {accent['cssVar']} in colony-accents.css")
for path in sorted(ui.glob("*.*")):
    for var in sorted(set(re.findall(r"--colony-[a-z0-9-]+", path.read_text(encoding="utf-8"))) - accents.keys()):
        missing = [slug for slug, defined in themes.items() if var not in defined]
        if missing:
            errors.append(f"{path.name}: {var} is missing from {', '.join(missing)}")
overrides = {accent["key"]: accents.get(accent["cssVar"]) for accent in theme_list["accents"]}
for slug, defined in themes.items():
    for name, color in {"its own": defined.get("--colony-accent-blue"), **overrides}.items():
        if not color or "--colony-ink" not in accents or "--colony-paper" not in accents:
            continue  # already reported above
        best = max(contrast(accents[end], color) for end in ("--colony-ink", "--colony-paper"))
        if best < 4.0:
            errors.append(f"{slug}: text on {name} accent {color} reaches only {best:.2f}:1")

print("\n".join(errors) or f"OK: all {len(themes)} themes define every --colony-* variable in src/ui/, and text on every accent reaches 4.0:1")
sys.exit(1 if errors else 0)
