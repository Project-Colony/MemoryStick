// The Preferences page, as in every Colony program (design/settings-page.md
// and design/navigation.md in Project-Colony-Resources): it replaces the
// document while open, and the program's name in the toolbar opens and
// closes it. Every change applies and is saved at once; there is no Save.

const $prefs = document.getElementById('preferences');
const $identity = document.getElementById('identity');

const CATEGORIES = ['general', 'appearance', 'accessibility', 'about'];
let category = 'general';
// Sections the user opened, kept while MemoryStick runs. All start closed.
const expanded = new Set();

const icon = (codepoint) => String.fromCodePoint(codepoint);
const GLYPH = { chevronRight: 0xf054, chevronDown: 0xf078, check: 0xf00c, system: 0xf108 };

// h('tag.class', { attribute or on<event>: value }, ...children)
function h(spec, attrs = {}, ...children) {
  const [tag, ...classes] = spec.split('.');
  const el = document.createElement(tag);
  if (classes.length) el.className = classes.join(' ');
  for (const [key, value] of Object.entries(attrs)) {
    if (key.startsWith('on')) el.addEventListener(key.slice(2), value);
    else if (key === 'style') for (const [prop, v] of Object.entries(value)) el.style.setProperty(prop, v);
    else if (value !== false && value != null) el.setAttribute(key, value === true ? '' : value);
  }
  el.append(...children.flat().filter((c) => c != null && c !== false));
  return el;
}

function togglePreferences(open = $prefs.hidden) {
  $prefs.hidden = !open;
  document.body.classList.toggle('prefs-open', open);
  $identity.setAttribute('aria-pressed', String(open));
  if (open) renderPreferences();
}

// Applies one change, saves it, and redraws the page in place, focus kept.
function change(update) {
  update(prefs);
  savePreferences();
  setLocale(prefs.language);
  translate();
  updateAppearance();
  renderPreferences();
}

// ===== Controls =====

function setting(label, description, control) {
  return h('div.setting', {},
    h('div.setting-text', {}, h('div.setting-label', {}, label), h('div.setting-desc', {}, description)),
    control);
}

function toggle(id, label, description, on, flip, disabled = false) {
  return setting(label, description,
    h('button.switch', {
      role: 'switch', 'aria-checked': String(on), 'aria-label': label, 'data-id': id, disabled,
      onclick: () => change(flip),
    }, h('span.knob')));
}

// options: [[value, text]]; pick(prefs, value) stores the choice.
function choice(id, label, description, options, current, pick) {
  return setting(label, description,
    h('div.segmented', { role: 'radiogroup', 'aria-label': label },
      options.map(([value, text]) => h('button', {
        role: 'radio',
        'aria-checked': String(value === current),
        'data-id': `${id}-${value}`,
        onclick: () => change((p) => pick(p, value)),
      }, text))));
}

// A collapsible section: a flat row whose chevron points right when closed
// and down when open; the whole row toggles.
function section(id, title, description, ...content) {
  const open = expanded.has(id);
  return h('section.pref-section', {},
    h('button.section-head', {
      'aria-expanded': String(open),
      'data-id': `section-${id}`,
      onclick: () => {
        if (open) expanded.delete(id); else expanded.add(id);
        renderPreferences();
      },
    }, h('span', {}, title), h('span.icon.chevron', { 'aria-hidden': 'true' }, icon(open ? GLYPH.chevronDown : GLYPH.chevronRight))),
    open && h('div.section-body', {}, description && h('p.desc', {}, description), ...content));
}

const sizeOptions = (sizes) => sizes.map((size) => [size, t(`preferences_size_${size}`)]);

// ===== Appearance =====

function themeCard(slug, name, swatch, ink) {
  const selected = (prefs.theme || null) === slug;
  return h('button.theme-card', {
    'aria-pressed': String(selected),
    'data-id': `theme-${slug}`,
    style: { '--card-bg': swatch.bg, '--card-accent': swatch.accent, '--card-ink': ink },
    onclick: () => {
      change((p) => { p.theme = slug; });
      showToast(t('theme_applied'));
    },
  },
  h('span.card-bar'),
  h('span.card-name', {}, name),
  selected && h('span.icon.card-check', { 'aria-hidden': 'true' }, icon(GLYPH.check)));
}

function themePicker() {
  const label = (item) => item.label[locale] || item.label.en;
  const gruvbox = themeList.families.find((f) => f.key === 'gruvbox');
  const system = gruvbox && gruvbox.variants.find((v) => v.slug === systemTheme());
  return h('div.theme-picker', {},
    // No choice: the system's light or dark mode, in Gruvbox.
    system && h('div.theme-family', {},
      h('div.family-name', {}, h('span.icon', { 'aria-hidden': 'true' }, icon(GLYPH.system)), t('preferences_theme_system')),
      h('div.theme-cards', {}, themeCard(null, t('preferences_theme_system_variant'), system.swatch, system.palette.text_primary))),
    themeList.families.map((family) => h('div.theme-family', {},
      h('div.family-name', {},
        family.icon && h('span.icon', { 'aria-hidden': 'true' }, icon(parseInt(family.icon, 16))),
        label(family)),
      h('div.theme-cards', {}, family.variants.map((v) => themeCard(v.slug, label(v), v.swatch, v.palette.text_primary))))));
}

function accentPicker() {
  // The check takes whichever of the theme's text and background reads
  // better on the swatch.
  const ink = (color) => [cssVar('--colony-text-primary'), cssVar('--colony-bg-primary')]
    .sort((a, b) => contrast(b, color) - contrast(a, color))[0];
  return h('div.accents', { role: 'radiogroup', 'aria-label': t('preferences_section_colors') },
    themeList.accents.map((accent) => {
      const selected = !prefs.autoAccent && prefs.accent === accent.key;
      const name = accent.label[locale] || accent.label.en;
      return h('button.accent', {
        role: 'radio',
        'aria-checked': String(selected),
        'aria-label': name,
        title: name,
        'data-id': `accent-${accent.key}`,
        style: { '--swatch': accent.color, color: ink(accent.color) },
        onclick: () => change((p) => { p.accent = accent.key; p.autoAccent = false; }),
      }, selected ? icon(GLYPH.check) : '');
    }));
}

function preview() {
  return h('div.preview.markdown-body', {},
    h('h3', {}, t('preferences_preview_heading')),
    h('p', {}, t('preferences_preview_body'), ' ', h('a', { href: '#' }, t('preferences_preview_link')), ' ', h('code', {}, t('preferences_preview_code'))),
    h('pre', {}, h('code.hljs', {},
      h('span.hljs-keyword', {}, 'fn'), ' ', h('span.hljs-title.function_', {}, 'main'), '() {\n    ',
      h('span.hljs-comment', {}, '// ' + t('preferences_section_preview')), '\n    println!(', h('span.hljs-string', {}, '"MemoryStick"'), ');\n}')));
}

// ===== Categories =====

const PAGES = {
  general: () => [
    section('language', t('preferences_section_language'), t('preferences_language_desc'),
      choice('language', t('preferences_section_language'), '', [
        [null, t('preferences_language_system')],
        ['en', t('preferences_language_en')],
        ['fr', t('preferences_language_fr')],
      ], ['en', 'fr'].includes(prefs.language) ? prefs.language : null, (p, v) => { p.language = v; })),
  ],
  appearance: () => [
    section('theme', t('preferences_section_theme'), t('preferences_theme_desc'), themePicker()),
    section('colors', t('preferences_section_colors'), t('preferences_colors_desc'),
      accentPicker(),
      toggle('auto-accent', t('preferences_auto_accent'), t('preferences_auto_accent_desc'),
        prefs.autoAccent === true, (p) => { p.autoAccent = !p.autoAccent; })),
    section('typography', t('preferences_section_typography'), t('preferences_typography_desc'),
      choice('font-size', t('preferences_font_size'), t('preferences_font_size_desc'),
        sizeOptions(['small', 'default', 'large']),
        ['small', 'large'].includes(prefs.fontSize) ? prefs.fontSize : 'default', (p, v) => { p.fontSize = v; })),
    section('effects', t('preferences_section_effects'), t('preferences_effects_desc'),
      // Off, and left alone, while Reduce motion is on.
      toggle('animations', t('preferences_animations'), t('preferences_animations_desc'),
        prefs.animations !== false && !reducedMotion(), (p) => { p.animations = p.animations === false; },
        reducedMotion())),
    section('preview', t('preferences_section_preview'), '', preview()),
  ],
  accessibility: () => [
    section('vision', t('preferences_section_vision'), t('preferences_vision_desc'),
      toggle('high-contrast', t('preferences_high_contrast'), t('preferences_high_contrast_desc'),
        prefs.highContrast === true, (p) => { p.highContrast = !p.highContrast; }),
      toggle('dyslexia-font', t('preferences_dyslexia_font'), t('preferences_dyslexia_font_desc'),
        prefs.dyslexiaFont === true, (p) => { p.dyslexiaFont = !p.dyslexiaFont; })),
    section('motion', t('preferences_section_motion'), t('preferences_motion_desc'),
      toggle('reduce-motion', t('preferences_reduce_motion'), t('preferences_reduce_motion_desc'),
        reducedMotion(), (p) => { p.reducedMotion = !reducedMotion(); })),
    section('reading', t('preferences_section_reading'), t('preferences_reading_desc'),
      choice('text-size', t('preferences_text_size'), t('preferences_text_size_desc'),
        sizeOptions(['small', 'default', 'large', 'xlarge']),
        SIZES[prefs.textSize] ? prefs.textSize : 'default', (p, v) => { p.textSize = v; })),
  ],
  about: () => [
    h('p.about', {}, t('preferences_version', (window.__MEMORYSTICK__ && window.__MEMORYSTICK__.version) || '')),
    h('p.about', {}, t('preferences_license')),
    h('button.link-button', {
      'data-id': 'source',
      onclick: () => invoke('plugin:opener|open_url', { url: 'https://github.com/Project-Colony/MemoryStick' })
        .catch((err) => showToast(t('error_cannot_open_link', err), true)),
    }, t('preferences_source')),
  ],
};

const DESCRIPTIONS = {
  general: 'preferences_general_desc',
  appearance: 'preferences_appearance_desc',
  accessibility: 'preferences_accessibility_desc',
  about: 'preferences_about_desc',
};

function renderPreferences() {
  const focused = document.activeElement && document.activeElement.dataset.id;
  $prefs.replaceChildren(
    h('header.prefs-header', {},
      h('h1', {}, t('preferences_title')),
      h('button.prefs-close', { 'data-id': 'close', onclick: () => togglePreferences(false) }, t('preferences_close'))),
    h('div.prefs-body', {},
      h('nav.prefs-categories', { 'aria-label': t('preferences_title') },
        CATEGORIES.map((key) => h('button.category', {
          'aria-current': key === category ? 'page' : false,
          'data-id': `category-${key}`,
          onclick: () => { category = key; renderPreferences(); },
        }, t(`preferences_cat_${key}`)))),
      h('div.prefs-content', {},
        h('h2', {}, t(`preferences_cat_${category}`)),
        h('p.desc', {}, t(DESCRIPTIONS[category])),
        PAGES[category]())));
  const again = focused && $prefs.querySelector(`[data-id="${focused}"]`);
  if (again) again.focus();
}

$identity.addEventListener('click', () => togglePreferences());
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && !$prefs.hidden) togglePreferences(false);
});
// The picker needs the theme list; draw it again once it is there.
themeListLoaded.then(() => { if (!$prefs.hidden) renderPreferences(); });
