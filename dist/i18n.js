// The interface's text, in English and French (i18n/en.json, i18n/fr.json).
// main.rs hands both over in window.__MEMORYSTICK__ before any script runs.
// English is canonical: a key missing from French shows the English text,
// though the Rust test fr_and_en_have_identical_key_sets forbids that.

const STRINGS = (window.__MEMORYSTICK__ && window.__MEMORYSTICK__.strings) || { en: {}, fr: {} };
let locale = 'en';

// 'en', 'fr', or anything else (null) for the system's language.
function setLocale(choice) {
  const system = (navigator.language || 'en').toLowerCase().startsWith('fr') ? 'fr' : 'en';
  locale = choice === 'en' || choice === 'fr' ? choice : system;
  document.documentElement.lang = locale;
}

// The text of a key, with {detail} filled in when given.
function t(key, detail) {
  const text = STRINGS[locale][key] ?? STRINGS.en[key] ?? key;
  // A function, so that a $ in the detail is not a replacement pattern.
  return detail === undefined ? text : text.replace('{detail}', () => detail);
}

// Fills every data-i18n (text), data-i18n-title and data-i18n-label
// (aria-label) under root.
function translate(root = document) {
  root.querySelectorAll('[data-i18n]').forEach((el) => { el.textContent = t(el.dataset.i18n); });
  root.querySelectorAll('[data-i18n-title]').forEach((el) => { el.title = t(el.dataset.i18nTitle); });
  root.querySelectorAll('[data-i18n-label]').forEach((el) => el.setAttribute('aria-label', t(el.dataset.i18nLabel)));
}

setLocale(prefs.language);
