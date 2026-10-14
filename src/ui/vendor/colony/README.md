# Colony themes

`colony-themes.css`, `colony-accents.css` and `themes.json` are copied
unmodified from
[Project-Colony-Resources](https://github.com/Project-Colony/Project-Colony-Resources)
`generated/`, at commit `8931f2b38422fe75757db6218a6bcbc8caa06a85`. Its
theme and accent colours are the ones colony-ui 0.1.6 draws, so MemoryStick
shows the same colours as Colony. They are GPL-3.0-or-later, like
MemoryStick.

- `colony-themes.css` defines the `--colony-*` colours of every theme, one
  block per `[data-colony-theme="<family>-<variant>"]`.
- `colony-accents.css` defines the shared accent colours, the same in every
  theme, as `--colony-accent-<key>` on `:root`, and `--colony-ink` and
  `--colony-paper`, the two label colours for text on an accent.
- `themes.json` lists the families and variants, in Colony's order, with
  their names and light or dark mode, and the accents with their names. The
  theme picker and the accent swatches are built from it.

`index.html` loads `colony-accents.css` before `colony-themes.css`. Both set
`--colony-accent-blue` on the root element with the same specificity, so the
later file wins and each theme keeps its own accent. When the user picks an
accent, `theme.js` reads its colour from the `:root` rule of
`colony-accents.css` and sets it as an inline `--colony-accent-blue`, which
wins over both; the swatches in Preferences read the same rule. It then sets
`--on-accent` to whichever of `--colony-ink` and `--colony-paper` contrasts
more with the accent in use, as colony-ui's `contrast_on()` does, for the
text of a selected entry or choice. `check-colony-themes.py` fails if that
falls under 4.0:1 for any theme and accent.

Do not edit them: CI fails when they differ from upstream at that commit.

## Updating

1. Pick the Project-Colony-Resources commit, normally its latest `main`.
2. Download the three files at that commit:

   ```sh
   rev=<full commit hash>
   base=https://raw.githubusercontent.com/Project-Colony/Project-Colony-Resources/$rev/generated
   curl -fsSL "$base/css/colony-themes.css" -o src/ui/vendor/colony/colony-themes.css
   curl -fsSL "$base/css/colony-accents.css" -o src/ui/vendor/colony/colony-accents.css
   curl -fsSL "$base/themes.json" -o src/ui/vendor/colony/themes.json
   ```

3. Put the same hash in this file and in `COLONY_RESOURCES_REV` in
   `.github/workflows/ci.yml`, then run
   `python3 .github/scripts/check-colony-themes.py`.
