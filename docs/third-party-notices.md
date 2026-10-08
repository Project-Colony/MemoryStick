# Third-party notices

MemoryStick's interface ships these libraries and fonts, unmodified, under
`dist/vendor/` and `dist/fonts/`. Everything in `dist/` is embedded in the MemoryStick
executable, so the licence file next to each library travels inside every
build.

| Component | Version | Licence | Files | Upstream |
|---|---|---|---|---|
| highlight.js | 11.12.0 | BSD-3-Clause | `dist/vendor/highlight/` ([LICENSE](../dist/vendor/highlight/LICENSE)) | https://github.com/highlightjs/highlight.js |
| KaTeX | 0.19.0 | MIT | `dist/vendor/katex/katex.min.js`, `katex.min.css` ([LICENSE](../dist/vendor/katex/LICENSE)) | https://github.com/KaTeX/KaTeX |
| KaTeX fonts | 0.19.0 | OFL-1.1 | `dist/vendor/katex/fonts/` (notice in each font file, licence text in [LICENSE](../dist/vendor/katex/LICENSE)) | https://github.com/KaTeX/KaTeX |
| Mermaid | 11.17.2 | MIT | `dist/vendor/mermaid/mermaid.min.js` ([LICENSE](../dist/vendor/mermaid/LICENSE)) | https://github.com/mermaid-js/mermaid |
| Colony themes | Project-Colony-Resources `edc7495` (colony-ui 0.1.5) | GPL-3.0-or-later, like MemoryStick | `dist/vendor/colony/` ([README](../dist/vendor/colony/README.md)) | https://github.com/Project-Colony/Project-Colony-Resources |
| JetBrains Mono Nerd Font | 2.304, Nerd Fonts 3.4.0 | OFL-1.1 | `dist/fonts/JetBrainsMonoNerdFont-*.ttf` ([LICENSE](../dist/fonts/LICENSE)) | https://github.com/ryanoasis/nerd-fonts |
| OpenDyslexic | 0.990 | OFL-1.1 | `dist/fonts/OpenDyslexic-Regular.otf` ([LICENSE](../dist/fonts/LICENSE)) | https://opendyslexic.org |
| Font Awesome 6 Free Solid | 6.7.2 | OFL-1.1 (font), CC-BY-4.0 (icons) | `dist/fonts/fa-solid-900.ttf` ([LICENSE](../dist/fonts/LICENSE)) | https://fontawesome.com |

`mermaid.min.js` is a single-file build that also contains 79 npm package
versions: d3 and its modules (ISC, five of them BSD-3-Clause), DOMPurify
(MPL-2.0 OR Apache-2.0), chevrotain (Apache-2.0), and cytoscape,
dagre-d3-es, langium, marked, KaTeX 0.16.47, lodash-es and others under
MIT. Most come from Mermaid's source map and from those of the
@mermaid-js/parser chunks it inlines; six more are inside the pre-bundled
builds of cytoscape (heap), roughjs (hachure-fill, path-data-parser,
points-on-curve, points-on-path) and vscode-uri (path-browserify), which
the source maps do not list. The full list, with versions and licence
texts, is in
[dist/vendor/mermaid/LICENSE](../dist/vendor/mermaid/LICENSE).

The Rust crates compiled into the executable are listed in
[`src-tauri/Cargo.lock`](../src-tauri/Cargo.lock).
