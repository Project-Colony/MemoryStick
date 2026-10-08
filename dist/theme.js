// Colony themes. This file is loaded in <head>, before the stylesheets apply,
// so the first paint already has the theme's colours and a dark theme never
// flashes light. With no saved choice MemoryStick follows the system, in
// Gruvbox, Colony's default family. The picker is set up once the page is
// parsed; app.js redraws the Mermaid diagrams on 'colony-theme-change'.

const THEME_KEY = 'colony-theme';
const $root = document.documentElement;
const systemDark = matchMedia('(prefers-color-scheme: dark)');
const themes = []; // { slug, family, mode }, from vendor/colony/themes.json

function savedTheme() {
  try {
    return localStorage.getItem(THEME_KEY) || '';
  } catch (_) {
    return ''; // storage unavailable: follow the system
  }
}

let chosenTheme = savedTheme(); // '' = follow the system
const systemTheme = () => (systemDark.matches ? 'gruvbox-dark' : 'gruvbox-light');
$root.dataset.colonyTheme = chosenTheme || systemTheme();

const cssVar = (name) => getComputedStyle($root).getPropertyValue(name).trim();
const isDark = () => getComputedStyle($root).getPropertyValue('color-scheme').trim() === 'dark';

function applyTheme(slug) {
  chosenTheme = slug;
  $root.dataset.colonyTheme = slug || systemTheme();
  try {
    if (slug) localStorage.setItem(THEME_KEY, slug);
    else localStorage.removeItem(THEME_KEY);
  } catch (_) {}
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

// highlight.js tokens take palette colours (styles.css). In a theme where a
// palette colour is under 3:1 on the code background, that token keeps stock
// GitHub's colour instead: [palette colour, GitHub light, GitHub dark].
const HIGHLIGHT = {
  keyword: ['--colony-error', '#d73a49', '#ff7b72'],
  title: ['--colony-accent-blue', '#6f42c1', '#d2a8ff'],
  number: ['--colony-accent-icon', '#005cc5', '#79c0ff'],
  string: ['--colony-success', '#032f62', '#a5d6ff'],
  comment: ['--colony-text-muted', '#6a737d', '#8b949e'],
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
  const dark = isDark();
  for (const [token, [palette, light, darkStock]] of Object.entries(HIGHLIGHT)) {
    // An unreadable value gives NaN, which also falls back to stock.
    if (contrast(cssVar(palette), background) >= 3) $root.style.removeProperty('--hl-' + token);
    else $root.style.setProperty('--hl-' + token, dark ? darkStock : light);
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
