// Appearance: the Colony theme, the accent, high contrast, the text size,
// the dyslexia font and motion, all set on <html>. Loaded in <head> after
// the stylesheets, since it reads the palette, so the first paint already
// has the user's theme and a dark theme never flashes light. With no saved
// theme MemoryStick follows the system in Gruvbox, Colony's default family.
// app.js redraws the Mermaid diagrams on 'colony-theme-change'.

const $root = document.documentElement;
const systemDark = matchMedia('(prefers-color-scheme: dark)');
const systemReducedMotion = matchMedia('(prefers-reduced-motion: reduce)');

// What the user chose, as main.rs saved it under Colony/MemoryStick (it
// hands it over in window.__MEMORYSTICK__ before this script runs).
const prefs = (window.__MEMORYSTICK__ && window.__MEMORYSTICK__.preferences) || {};

function savePreferences() {
  window.__TAURI__.core.invoke('save_preferences', { preferences: prefs })
    .catch((err) => showToast(t('error_cannot_save', err), true));
}

// Every family, variant and accent (vendor/colony/themes.json), once loaded.
let themeList = { families: [], accents: [] };
const themeListLoaded = fetch('vendor/colony/themes.json')
  .then((response) => response.json())
  .then((list) => { themeList = list; })
  .catch((err) => console.error('Colony theme list unavailable:', err));

const cssVar = (name) => getComputedStyle($root).getPropertyValue(name).trim();
const isDark = () => cssVar('color-scheme') === 'dark';
const systemTheme = () => (systemDark.matches ? 'gruvbox-dark' : 'gruvbox-light');

// Typography size and Accessibility text size; they multiply.
const SIZES = { small: 0.85, default: 1, large: 1.2, xlarge: 1.4 };

const reducedMotion = () =>
  (typeof prefs.reducedMotion === 'boolean' ? prefs.reducedMotion : systemReducedMotion.matches);

// #rrggbb channels as 0..1, and back.
const channels = (hex) => {
  const n = parseInt(hex.slice(1), 16);
  return [n >> 16, (n >> 8) & 255, n & 255].map((c) => c / 255);
};
const toHex = (rgb) => '#' + rgb
  .map((c) => Math.round(Math.min(1, Math.max(0, c)) * 255).toString(16).padStart(2, '0'))
  .join('');

// High contrast derives a boosted palette from the active one, as
// colony_ui's with_high_contrast does: these fields move away from the
// background by [amount in a dark theme, amount in a light theme].
const HIGH_CONTRAST = {
  'text-primary': [0.12, 0.15],
  'text-secondary': [0.10, 0.12],
  'text-muted': [0.10, 0.10],
  'text-dim': [0.08, 0.10],
  'text-dimmer': [0.08, 0.08],
  'border-subtle': [0.12, 0.15],
  divider: [0.12, 0.15],
};

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
  const [r, g, b] = channels(hex).map((c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a, b) {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

function applyAppearance() {
  // Back to the theme's own palette first, then the user's changes on top.
  for (const field of Object.keys(HIGH_CONTRAST)) $root.style.removeProperty('--colony-' + field);
  $root.style.removeProperty('--colony-accent-blue');
  $root.dataset.colonyTheme = prefs.theme || systemTheme();
  // A saved theme this build does not have falls back to the system, as
  // colony-ui does.
  if (!cssVar('--colony-bg-primary')) $root.dataset.colonyTheme = systemTheme();

  // No override, or auto accent, leaves the theme's own accent.
  const accent = !prefs.autoAccent && themeList.accents.find((a) => a.key === prefs.accent);
  if (accent) $root.style.setProperty('--colony-accent-blue', accent.color);

  if (prefs.highContrast) {
    // colony_ui's is_light: YIQ luma of the background over 0.5.
    const [r, g, b] = channels(cssVar('--colony-bg-primary'));
    const light = 0.299 * r + 0.587 * g + 0.114 * b > 0.5;
    for (const [field, [dark, lighter]] of Object.entries(HIGH_CONTRAST)) {
      const shift = light ? -lighter : dark;
      const name = '--colony-' + field;
      $root.style.setProperty(name, toHex(channels(cssVar(name)).map((c) => c + shift)));
    }
  }

  const background = cssVar('--colony-bg-primary');
  for (const [token, candidates] of Object.entries(HIGHLIGHT)) {
    // An unreadable value gives NaN, which counts as illegible.
    const legible = candidates.find((name) => contrast(cssVar(name), background) >= 3);
    $root.style.setProperty('--hl-' + token, `var(${legible || '--colony-text-primary'})`);
  }

  const fontSize = ['small', 'large'].includes(prefs.fontSize) ? prefs.fontSize : 'default';
  $root.style.setProperty('--font-scale', SIZES[fontSize] * (SIZES[prefs.textSize] || 1));
  $root.toggleAttribute('data-dyslexia', prefs.dyslexiaFont === true);
  // Reduced motion silences every animation; so does turning them off.
  $root.toggleAttribute('data-still', reducedMotion() || prefs.animations === false);
}

// After a change: applies it, and lets the page redraw what depends on it.
function updateAppearance() {
  applyAppearance();
  document.dispatchEvent(new Event('colony-theme-change'));
}

// Ctrl+D: the family's variant in the other mode, or Gruvbox's when the
// family has only one mode.
function toggleMode() {
  const dark = isDark();
  const variants = themeList.families.flatMap((f) => f.variants.map((v) => ({ ...v, family: f.key })));
  const current = variants.find((v) => v.slug === $root.dataset.colonyTheme);
  const other = current && variants.find((v) => v.family === current.family && (v.mode === 'dark') !== dark);
  prefs.theme = other ? other.slug : dark ? 'gruvbox-light' : 'gruvbox-dark';
  savePreferences();
  updateAppearance();
}

applyAppearance();
// The accent override needs the theme list.
themeListLoaded.then(() => { if (prefs.accent && !prefs.autoAccent) updateAppearance(); });
systemDark.addEventListener('change', () => { if (!prefs.theme) updateAppearance(); });
systemReducedMotion.addEventListener('change', applyAppearance);
