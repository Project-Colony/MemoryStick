# Third-party notices

MemoryStick's interface ships these libraries, unmodified, under
`dist/vendor/`. Everything in `dist/` is embedded in the MemoryStick
executable, so the licence file next to each library travels inside every
build.

| Component | Version | Licence | Files | Upstream |
|---|---|---|---|---|
| highlight.js | 11.10.0 | BSD-3-Clause | `dist/vendor/highlight/` ([LICENSE](../dist/vendor/highlight/LICENSE)) | https://github.com/highlightjs/highlight.js |
| KaTeX | 0.16.11 | MIT | `dist/vendor/katex/katex.min.js`, `katex.min.css` ([LICENSE](../dist/vendor/katex/LICENSE)) | https://github.com/KaTeX/KaTeX |
| KaTeX fonts | 0.16.11 | OFL-1.1 | `dist/vendor/katex/fonts/` (notice in each font file, licence text in [LICENSE](../dist/vendor/katex/LICENSE)) | https://github.com/KaTeX/KaTeX |
| Mermaid | 11.12.3 | MIT | `dist/vendor/mermaid/mermaid.min.js` ([LICENSE](../dist/vendor/mermaid/LICENSE)) | https://github.com/mermaid-js/mermaid |

`mermaid.min.js` is a single-file build that also contains 76 npm package
versions: d3 and its modules (ISC, five of them BSD-3-Clause), DOMPurify
(MPL-2.0 OR Apache-2.0), chevrotain (Apache-2.0), and cytoscape,
dagre-d3-es, langium, marked, KaTeX 0.16.22, lodash-es and others under
MIT. Most come from Mermaid's source map; five more are inside the
pre-bundled builds of cytoscape (heap) and roughjs (hachure-fill,
path-data-parser, points-on-curve, points-on-path), which the source map
does not list. The full list, with versions and licence texts, is in
[dist/vendor/mermaid/LICENSE](../dist/vendor/mermaid/LICENSE).

The Rust crates compiled into the executable are listed in
[`src-tauri/Cargo.lock`](../src-tauri/Cargo.lock).
