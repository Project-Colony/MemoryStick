# Colony themes

`colony-themes.css` and `themes.json` are copied unmodified from
[Project-Colony-Resources](https://github.com/Project-Colony/Project-Colony-Resources)
`generated/`, at commit `edc7495de15e75553b9203e85d58e4452be67b3a`: the
commit colony-ui 0.1.5 was published from, so MemoryStick shows the same
colours as Colony. They are GPL-3.0-or-later, like MemoryStick.

- `colony-themes.css` defines the `--colony-*` colours of every theme, one
  block per `[data-colony-theme="<family>-<variant>"]`.
- `themes.json` lists the families and variants, in Colony's order, with
  their names and light or dark mode. The theme picker is built from it.

Do not edit them: CI fails when they differ from upstream at that commit.
`colony-accents.css` is left out on purpose, because its accent override
variables reuse the name `--colony-accent-blue` and would repaint every theme.

## Updating

1. Pick the Project-Colony-Resources commit, normally the one the colony-ui
   release in use was published from (`.cargo_vcs_info.json` in the crate).
2. Download both files at that commit:

   ```sh
   rev=<full commit hash>
   base=https://raw.githubusercontent.com/Project-Colony/Project-Colony-Resources/$rev/generated
   curl -fsSL "$base/css/colony-themes.css" -o dist/vendor/colony/colony-themes.css
   curl -fsSL "$base/themes.json" -o dist/vendor/colony/themes.json
   ```

3. Put the same hash in this file and in `COLONY_RESOURCES_REV` in
   `.github/workflows/ci.yml`, then run
   `python3 .github/scripts/check-colony-themes.py`.
