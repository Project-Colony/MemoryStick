// MemoryStick frontend (Tauri v2)
// The Tauri bindings are exposed through withGlobalTauri: true.
// Plugins (opener) are NOT attached to window.__TAURI__ automatically, so
// they are called through invoke('plugin:NAME|COMMAND', args).

// Show any error right in the page (debugging aid)
window.addEventListener('error', (e) => {
  const el = document.getElementById('content') || document.body;
  if (el) {
    const pre = document.createElement('pre');
    pre.style.cssText = 'color:var(--colony-error);padding:20px;white-space:pre-wrap;font-size:13px;border:1px solid var(--colony-error);margin:20px;';
    pre.textContent = `JS Error: ${e.message}\nat ${e.filename}:${e.lineno}:${e.colno}`;
    el.prepend(pre);
  }
});

if (!window.__TAURI__) {
  document.body.innerHTML = '<pre style="color:var(--colony-error);padding:40px">window.__TAURI__ is not available. Check withGlobalTauri: true in tauri.conf.json.</pre>';
  throw new Error('Tauri API unavailable');
}

const invoke = window.__TAURI__.core.invoke;
const convertFileSrc = window.__TAURI__.core.convertFileSrc;
const listen = window.__TAURI__.event.listen;

// ===== State =====
let currentFilePath = null;
let currentDir = null;

const $main = document.getElementById('main');
const $content = document.getElementById('content');
const $filename = document.getElementById('filename');
const $tocPanel = document.getElementById('toc-panel');
const $tocNav = document.getElementById('toc-nav');

// ===== Mermaid =====
// Mermaid needs literal colours, so they are read from the theme (theme.js)
// and the diagrams are redrawn when it changes. 'base' is the only Mermaid
// theme that takes themeVariables. initialize() starts over from Mermaid's
// defaults, so the whole configuration is passed every time.
function mermaidConfig() {
  return {
    startOnLoad: false,
    securityLevel: 'strict',
    flowchart: { htmlLabels: true, curve: 'basis' },
    theme: 'base',
    themeVariables: {
      darkMode: isDark(),
      background: cssVar('--colony-bg-primary'),
      primaryColor: cssVar('--colony-bg-card'),
      primaryTextColor: cssVar('--colony-text-primary'),
      primaryBorderColor: cssVar('--colony-accent-blue'),
      secondaryColor: cssVar('--colony-bg-card-hover'),
      tertiaryColor: cssVar('--colony-bg-sidebar'),
      lineColor: cssVar('--colony-text-muted'),
      textColor: cssVar('--colony-text-primary'),
      noteBkgColor: cssVar('--colony-bg-sidebar'),
      noteTextColor: cssVar('--colony-text-primary'),
      noteBorderColor: cssVar('--colony-border-subtle'),
      fontFamily: getComputedStyle(document.body).fontFamily
    }
  };
}

async function renderMermaid() {
  if (!window.mermaid) return;
  // Mermaid measures its labels: with the font still loading, it would
  // size them for a fallback font.
  await document.fonts.ready;
  $content.querySelectorAll('.mermaid').forEach(async (el, i) => {
    const id = `mermaid-${Date.now()}-${i}`;
    try {
      const { svg } = await window.mermaid.render(id, el.dataset.source);
      el.innerHTML = svg;
    } catch (err) {
      const pre = document.createElement('pre');
      pre.style.color = 'var(--colony-error)';
      pre.textContent = `Mermaid error: ${String(err.message || err)}`;
      el.replaceChildren(pre);
    }
  });
}

if (window.mermaid) window.mermaid.initialize(mermaidConfig());

document.addEventListener('colony-theme-change', () => {
  if (!window.mermaid) return;
  window.mermaid.initialize(mermaidConfig());
  renderMermaid();
});

// ===== HTML post-processing =====
function dirname(p) {
  if (!p) return null;
  const idx = Math.max(p.lastIndexOf('/'), p.lastIndexOf('\\'));
  return idx >= 0 ? p.substring(0, idx) : null;
}

function joinPath(base, rel) {
  if (!base) return rel;
  const sep = base.includes('\\') ? '\\' : '/';
  return base.replace(/[\\/]+$/, '') + sep + rel.replace(/^[\\/]+/, '');
}

function isAbsolute(p) {
  return /^[a-zA-Z]:[\\/]/.test(p) || p.startsWith('/') || p.startsWith('\\');
}

function postProcess() {
  // 1) Relative images: convertFileSrc(absolute path). Other sources are
  //    left as they are; whether a web (http, https) image loads is up to
  //    img-src in the CSP (src-tauri/tauri.conf.json).
  if (currentDir) {
    $content.querySelectorAll('img').forEach(img => {
      const src = img.getAttribute('src') || '';
      if (!src) return;
      if (/^(https?:|data:|blob:|image:|file:)/i.test(src)) return;
      // Served by the app's image protocol (main.rs), which reads images on
      // this device only.
      img.src = convertFileSrc(isAbsolute(src) ? src : joinPath(currentDir, src), 'image');
    });
  }

  // 2) Syntax highlighting (highlight.js) and extraction of mermaid blocks
  $content.querySelectorAll('pre code').forEach((codeEl) => {
    const classes = codeEl.className || '';
    const langMatch = classes.match(/language-([\w-]+)/);
    const lang = langMatch ? langMatch[1].toLowerCase() : null;

    if (lang === 'mermaid') {
      // Replace <pre><code class="language-mermaid">...</code></pre> with <div class="mermaid">...</div>
      const pre = codeEl.parentElement;
      const div = document.createElement('div');
      div.className = 'mermaid';
      div.textContent = codeEl.textContent;
      div.dataset.source = codeEl.textContent; // kept for redrawing
      pre.replaceWith(div);
      return;
    }

    try {
      if (lang && window.hljs.getLanguage(lang)) {
        const res = window.hljs.highlight(codeEl.textContent, { language: lang, ignoreIllegals: true });
        codeEl.innerHTML = res.value;
      } else {
        const res = window.hljs.highlightAuto(codeEl.textContent);
        codeEl.innerHTML = res.value;
      }
      codeEl.classList.add('hljs');
    } catch (_) {}
  });

  // 3) KaTeX: comrak emits <span data-math-style="inline|display"> and
  //    <pre><code class="language-math" data-math-style="display">
  if (window.katex) {
    $content.querySelectorAll('[data-math-style]').forEach((el) => {
      const display = el.getAttribute('data-math-style') === 'display';
      const tex = el.textContent;
      try {
        const html = window.katex.renderToString(tex, {
          displayMode: display,
          throwOnError: false,
          errorColor: 'var(--colony-error)',
          strict: 'ignore'
        });
        // Replace the whole node (span or pre>code) with the KaTeX HTML
        if (el.tagName === 'CODE' && el.parentElement && el.parentElement.tagName === 'PRE') {
          el.parentElement.outerHTML = html;
        } else {
          el.outerHTML = html;
        }
      } catch (err) {
        el.textContent = '\uf071 ' + err.message;
      }
    });
  }

  // 4) Mermaid render (after extraction)
  renderMermaid();

  // 5) Build the table of contents
  buildToc();
}

function buildToc() {
  const headings = $content.querySelectorAll('h1, h2, h3, h4');
  if (headings.length === 0) {
    $tocNav.innerHTML = '<em style="color:var(--colony-text-secondary);font-size:13px">No headings in this document</em>';
    return;
  }
  const root = document.createElement('ul');
  $tocNav.innerHTML = '';
  $tocNav.appendChild(root);

  headings.forEach((h, i) => {
    if (!h.id) h.id = 'h-toc-' + i;
    const level = parseInt(h.tagName.substring(1), 10);
    const li = document.createElement('li');
    li.style.marginLeft = ((level - 1) * 12) + 'px';
    const a = document.createElement('a');
    a.textContent = h.textContent.trim();
    a.href = '#' + h.id;
    li.appendChild(a);
    root.appendChild(li);
  });
}

// ===== Links =====
// What a click on a link does. A web or mail link opens in the default
// browser or mail app, a link to a place in this document scrolls to it,
// and any other link does nothing: the window itself never navigates.
// Returns { open: url }, { scrollTo: id } or null.
// A mailto link may only fill in these fields: some mail apps have honoured
// others, such as attach=, by attaching a file from the disk.
const MAIL_FIELDS = ['to', 'cc', 'bcc', 'subject', 'body'];
function linkAction(href, base) {
  try {
    const url = new URL(href, base);
    const here = new URL(base);
    if (url.hash && url.href.split('#')[0] === here.href.split('#')[0]) {
      return { scrollTo: decodeURIComponent(url.hash.slice(1)) };
    }
    const web = (url.protocol === 'http:' || url.protocol === 'https:') && url.origin !== here.origin;
    const mail = url.protocol === 'mailto:'
      && [...url.searchParams.keys()].every((key) => MAIL_FIELDS.includes(key.toLowerCase()));
    if (web || mail) return { open: url.href };
  } catch (_) {}
  return null;
}

// One handler for every link, including those added after rendering
// (Mermaid diagrams, the table of contents) and <area> elements. It runs in
// the capture phase, before anything inside the page, and it calls
// Element.prototype.closest rather than e.target.closest: a <form> in a
// document can shadow its own closest with a field named "closest", and a
// handler that throws would let the browser follow the link.
document.addEventListener('click', (e) => {
  const link = e.target instanceof Element && Element.prototype.closest.call(e.target, 'a, area');
  if (!link) return;
  e.preventDefault();
  const href = link.getAttribute('href') ?? link.getAttribute('xlink:href') ?? '';
  const action = linkAction(href, location.href);
  if (action && action.open) {
    invoke('plugin:opener|open_url', { url: action.open })
      .catch((err) => alert('Cannot open the link: ' + err));
  } else if (action) {
    // comrak gives a heading's anchor the id "h-name", while links to the
    // heading, as on GitHub, are written "#name"
    const target = document.getElementById(action.scrollTo)
      || document.getElementById('h-' + action.scrollTo);
    if (target) target.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }
}, true);

// ===== Rendering =====
// The page never names a file: main.rs reads only the document the user
// opened, through the Open dialog, a drop or the command line.
function render(result) {
  currentFilePath = result.file_path;
  currentDir = dirname(currentFilePath);

  // Parsed in an inert <template> first, so that the document's <meta>
  // elements (a refresh tag navigates the window) and <link> elements
  // (preconnect and prefetch hints reach the network) are dropped before
  // the page ever sees them.
  const doc = document.createElement('template');
  doc.innerHTML = result.html;
  doc.content.querySelectorAll('meta, link').forEach((el) => el.remove());
  $content.replaceChildren(doc.content);
  $main.classList.add('has-content');
  $filename.textContent = result.file_name;
  document.title = `${result.file_name} - MemoryStick`;

  postProcess();
  window.scrollTo(0, 0);
}

// Loads the opened document, if there is one.
async function loadAndRender() {
  try {
    const result = await invoke('load_file');
    if (result) render(result);
  } catch (err) {
    alert('Error: ' + err);
  }
}

// ===== UI handlers =====
document.getElementById('open-btn').addEventListener('click', async () => {
  try {
    const result = await invoke('open_document', { allFiles: 'All files' });
    if (result) render(result);
  } catch (err) {
    alert('Error: ' + err);
  }
});

document.getElementById('toc-btn').addEventListener('click', () => {
  document.body.classList.toggle('toc-open');
  $tocPanel.classList.toggle('hidden');
});

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
    e.preventDefault();
    document.getElementById('open-btn').click();
  } else if ((e.ctrlKey || e.metaKey) && e.key === 'd') {
    e.preventDefault();
    toggleMode();
  } else if (((e.ctrlKey || e.metaKey) && e.key === 'r') || e.key === 'F5') {
    // F5 too: reloading the page would bring back the preferences as they
    // were when MemoryStick started.
    e.preventDefault();
    if (currentFilePath) loadAndRender();
  }
});

// ===== Drag and drop (native Tauri event) =====
listen('tauri://drag-enter', () => {
  document.body.classList.add('drag-over');
});
listen('tauri://drag-leave', () => {
  document.body.classList.remove('drag-over');
});
listen('tauri://drag-drop', () => {
  document.body.classList.remove('drag-over');
});
// main.rs takes the dropped file as the opened document, then says so.
listen('document-opened', loadAndRender);

// Block the standard web drop, which would navigate away
window.addEventListener('dragover', (e) => e.preventDefault());
window.addEventListener('drop', (e) => e.preventDefault());

// ===== A file named on the command line (double-click) =====
loadAndRender();
