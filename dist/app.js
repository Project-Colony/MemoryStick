// MemoryStick frontend (Tauri v2)
// The Tauri bindings are exposed through withGlobalTauri: true.
// Plugins (dialog, opener) are NOT attached to window.__TAURI__ automatically,
// so they are called through invoke('plugin:NAME|COMMAND', args).

// Show any error right in the page (debugging aid)
window.addEventListener('error', (e) => {
  const el = document.getElementById('content') || document.body;
  if (el) {
    const pre = document.createElement('pre');
    pre.style.cssText = 'color:#ff6b6b;padding:20px;white-space:pre-wrap;font-size:13px;background:#1a0000;border:1px solid #ff6b6b;margin:20px;';
    pre.textContent = `JS Error: ${e.message}\nat ${e.filename}:${e.lineno}:${e.colno}`;
    el.prepend(pre);
  }
});

if (!window.__TAURI__) {
  document.body.innerHTML = '<pre style="color:red;padding:40px">window.__TAURI__ is not available. Check withGlobalTauri: true in tauri.conf.json.</pre>';
  throw new Error('Tauri API unavailable');
}

const invoke = window.__TAURI__.core.invoke;
const convertFileSrc = window.__TAURI__.core.convertFileSrc;
const listen = window.__TAURI__.event.listen;

// dialog.open, through a direct invoke on the plugin
async function openDialog(options) {
  return await invoke('plugin:dialog|open', { options });
}

// ===== State =====
let currentFilePath = null;
let currentDir = null;

const $main = document.getElementById('main');
const $content = document.getElementById('content');
const $filename = document.getElementById('filename');
const $tocPanel = document.getElementById('toc-panel');
const $tocNav = document.getElementById('toc-nav');
const $themeBtn = document.getElementById('theme-btn');
const $hljsLight = document.getElementById('hljs-light');
const $hljsDark = document.getElementById('hljs-dark');

// ===== Mermaid init =====
if (window.mermaid) {
  window.mermaid.initialize({
    startOnLoad: false,
    theme: 'default',
    securityLevel: 'strict',
    flowchart: { htmlLabels: true, curve: 'basis' }
  });
}

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
          errorColor: '#cc0000',
          strict: 'ignore'
        });
        // Replace the whole node (span or pre>code) with the KaTeX HTML
        if (el.tagName === 'CODE' && el.parentElement && el.parentElement.tagName === 'PRE') {
          el.parentElement.outerHTML = html;
        } else {
          el.outerHTML = html;
        }
      } catch (err) {
        el.textContent = '⚠️ ' + err.message;
      }
    });
  }

  // 4) Mermaid render (after extraction)
  if (window.mermaid) {
    const blocks = $content.querySelectorAll('.mermaid');
    blocks.forEach(async (el, i) => {
      const code = el.textContent;
      const id = `mermaid-${Date.now()}-${i}`;
      try {
        const { svg } = await window.mermaid.render(id, code);
        el.innerHTML = svg;
      } catch (err) {
        const pre = document.createElement('pre');
        pre.style.color = '#cc0000';
        pre.textContent = `Mermaid error: ${String(err.message || err)}`;
        el.replaceChildren(pre);
      }
    });
  }

  // 5) Build the table of contents
  buildToc();
}

function buildToc() {
  const headings = $content.querySelectorAll('h1, h2, h3, h4');
  if (headings.length === 0) {
    $tocNav.innerHTML = '<em style="color:#8b949e;font-size:13px">No headings in this document</em>';
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
function linkAction(href, base) {
  try {
    const url = new URL(href, base);
    const here = new URL(base);
    if (url.hash && url.href.split('#')[0] === here.href.split('#')[0]) {
      return { scrollTo: decodeURIComponent(url.hash.slice(1)) };
    }
    const web = (url.protocol === 'http:' || url.protocol === 'https:') && url.origin !== here.origin;
    if (web || url.protocol === 'mailto:') return { open: url.href };
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
async function loadAndRender(filePath) {
  try {
    const result = await invoke('load_file', { path: filePath });
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
  } catch (err) {
    alert('Error: ' + err);
  }
}

// ===== UI handlers =====
document.getElementById('open-btn').addEventListener('click', async () => {
  try {
    const selected = await openDialog({
      multiple: false,
      // The same list as EXTENSIONS in main.rs, which refuses other files.
      // "All files" stays because GTK matches these patterns case-sensitively,
      // so on Linux README.MD only shows there.
      filters: [
        { name: 'Markdown', extensions: ['md', 'markdown', 'mdown', 'mkd', 'mkdn', 'txt'] },
        { name: 'All files', extensions: ['*'] }
      ]
    });
    // The Tauri dialog plugin returns a string (or null/undefined when cancelled)
    if (selected && typeof selected === 'string') {
      loadAndRender(selected);
    } else if (selected && typeof selected === 'object' && selected.path) {
      loadAndRender(selected.path);
    }
  } catch (err) {
    alert('Dialog error: ' + err);
  }
});

document.getElementById('toc-btn').addEventListener('click', () => {
  document.body.classList.toggle('toc-open');
  $tocPanel.classList.toggle('hidden');
});

function toggleTheme() {
  const isDark = document.body.classList.toggle('dark');
  $themeBtn.textContent = isDark ? '☀️' : '🌙';
  $hljsDark.disabled = !isDark;
  $hljsLight.disabled = isDark;
  if (window.mermaid) {
    window.mermaid.initialize({
      startOnLoad: false,
      theme: isDark ? 'dark' : 'default',
      securityLevel: 'strict'
    });
    if (currentFilePath) loadAndRender(currentFilePath);
  }
}

$themeBtn.addEventListener('click', toggleTheme);

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
    e.preventDefault();
    document.getElementById('open-btn').click();
  } else if ((e.ctrlKey || e.metaKey) && e.key === 'd') {
    e.preventDefault();
    toggleTheme();
  } else if ((e.ctrlKey || e.metaKey) && e.key === 'r') {
    e.preventDefault();
    if (currentFilePath) loadAndRender(currentFilePath);
  }
});

// ===== Drag and drop (native Tauri event) =====
listen('tauri://drag-enter', () => {
  document.body.classList.add('drag-over');
});
listen('tauri://drag-leave', () => {
  document.body.classList.remove('drag-over');
});
listen('tauri://drag-drop', (event) => {
  document.body.classList.remove('drag-over');
  const paths = event.payload?.paths || [];
  if (paths.length > 0) {
    loadAndRender(paths[0]);
  }
});

// Block the standard web drop, which would navigate away
window.addEventListener('dragover', (e) => e.preventDefault());
window.addEventListener('drop', (e) => e.preventDefault());

// ===== File passed as an argument (double-click) =====
listen('open-file-path', (event) => {
  if (event.payload) loadAndRender(event.payload);
});
