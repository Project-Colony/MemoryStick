// Colony themes. This file is loaded in <head>, before the stylesheets apply,
// so the first paint already has the theme's colours and a dark theme never
// flashes light. With no saved choice MemoryStick follows the system, in
// Gruvbox, Colony's default family. The picker is set up once the page is
// parsed; app.js redraws the Mermaid diagrams on 'colony-theme-change'.

const $root = document.documentElement;
const systemDark = matchMedia('(prefers-color-scheme: dark)');
const themes = []; // { slug, family, mode }, from vendor/colony/themes.json

// What the user chose, as main.rs saved it under Colony/MemoryStick (it
// hands it over in window.__MEMORYSTICK__ before this script runs).
const prefs = (window.__MEMORYSTICK__ && window.__MEMORYSTICK__.preferences) || {};

function savePreferences() {
  window.__TAURI__.core.invoke('save_preferences', { preferences: prefs })
    .catch((err) => console.error('Preferences not saved:', err));
}

let chosenTheme = typeof prefs.theme === 'string' ? prefs.theme : ''; // '' = follow the system
const systemTheme = () => (systemDark.matches ? 'gruvbox-dark' : 'gruvbox-light');
$root.dataset.colonyTheme = chosenTheme || systemTheme();

const cssVar = (name) => getComputedStyle($root).getPropertyValue(name).trim();
const isDark = () => getComputedStyle($root).getPropertyValue('color-scheme').trim() === 'dark';

function applyTheme(slug) {
  chosenTheme = slug;
  $root.dataset.colonyTheme = slug || systemTheme();
  prefs.theme = slug || null;
  savePreferences();
  document.getElementById('theme-select').value = slug;
  keepHighlightLegible();
  document.dispatchEvent(new Event('colony-theme-change'));
}

// Ctrl+D: the family's variant in the other mode, or Gruvbox's when the
// family has only one mode.
function toggleMode() {
  const dark = isDark();
  const current = themes.find((t) => t.slug === $root.dataset.colonyTheme);
  const other = current && themes.find((t) => t.family === current.family && (t.mode === 'dark') !== dark);
  applyTheme(other ? other.slug : dark ? 'gruvbox-light' : 'gruvbox-dark');
}

// highlight.js tokens take palette colours (styles.css). In a theme where
// the first is under 3:1 on the code background, the token takes the next
// one, and text-primary (always at least 4.5:1) when none is legible.
const HIGHLIGHT = {
  keyword: ['--colony-error', '--colony-error-light'],
  title: ['--colony-accent-blue', '--colony-accent-icon'],
  number: ['--colony-accent-icon', '--colony-accent-blue'],
  string: ['--colony-success', '--colony-btn-success'],
  comment: ['--colony-text-muted', '--colony-text-secondary'],
};

// WCAG relative luminance of a #rrggbb colour.
function luminance(hex) {
  const n = parseInt(hex.slice(1), 16);
  const [r, g, b] = [n >> 16, (n >> 8) & 255, n & 255].map((c) => {
    c /= 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a, b) {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

function keepHighlightLegible() {
  const background = cssVar('--colony-bg-primary');
  for (const [token, candidates] of Object.entries(HIGHLIGHT)) {
    // An unreadable value gives NaN, which counts as illegible.
    const legible = candidates.find((name) => contrast(cssVar(name), background) >= 3);
    $root.style.setProperty('--hl-' + token, `var(${legible || '--colony-text-primary'})`);
  }
}

document.addEventListener('DOMContentLoaded', () => {
  const select = document.getElementById('theme-select');

  // A saved theme this build does not have falls back to the system, as
  // colony-ui does.
  if (!cssVar('--colony-bg-primary')) applyTheme('');
  else keepHighlightLegible();

  select.addEventListener('change', () => applyTheme(select.value));
  systemDark.addEventListener('change', () => {
    if (!chosenTheme) applyTheme('');
  });

  fetch('vendor/colony/themes.json')
    .then((response) => response.json())
    .then(({ families }) => {
      for (const family of families) {
        for (const variant of family.variants) {
          select.append(new Option(`${family.label.en} · ${variant.label.en}`, variant.slug));
          themes.push({ slug: variant.slug, family: family.key, mode: variant.mode });
        }
      }
      select.value = chosenTheme;
    })
    .catch((err) => console.error('Colony theme list unavailable:', err));
});
