# Third-party notices

MemoryStick's interface ships these libraries and fonts, unmodified, under
`src/ui/vendor/` and `src/ui/fonts/`. Everything in `src/ui/` is embedded in the
MemoryStick executable, so the licence file next to each library travels inside
every build.

| Component | Version | Licence | Files | Upstream |
|---|---|---|---|---|
| highlight.js | 11.12.0 | BSD-3-Clause | `src/ui/vendor/highlight/` ([LICENSE](../src/ui/vendor/highlight/LICENSE)) | https://github.com/highlightjs/highlight.js |
| KaTeX | 0.19.0 | MIT | `src/ui/vendor/katex/katex.min.js`, `katex.min.css` ([LICENSE](../src/ui/vendor/katex/LICENSE)) | https://github.com/KaTeX/KaTeX |
| KaTeX fonts | 0.19.0 | OFL-1.1 | `src/ui/vendor/katex/fonts/` (notice in each font file, licence text in [LICENSE](../src/ui/vendor/katex/LICENSE)) | https://github.com/KaTeX/KaTeX |
| Mermaid | 11.17.2 | MIT | `src/ui/vendor/mermaid/mermaid.min.js` ([LICENSE](../src/ui/vendor/mermaid/LICENSE)) | https://github.com/mermaid-js/mermaid |
| Colony themes and accents | Project-Colony-Resources `7247b81` (colony-ui 0.1.6) | GPL-3.0-or-later, like MemoryStick | `src/ui/vendor/colony/` ([README](../src/ui/vendor/colony/README.md)) | https://github.com/Project-Colony/Project-Colony-Resources |
| JetBrains Mono Nerd Font | 2.304, Nerd Fonts 3.4.0 | OFL-1.1 | `src/ui/fonts/JetBrainsMonoNerdFont-*.ttf` ([LICENSE](../src/ui/fonts/LICENSE)) | https://github.com/ryanoasis/nerd-fonts |
| OpenDyslexic | 0.990 | OFL-1.1 | `src/ui/fonts/OpenDyslexic-Regular.otf` ([LICENSE](../src/ui/fonts/LICENSE)) | https://opendyslexic.org |
| Font Awesome 6 Free Solid | 6.7.2 | OFL-1.1 (font), CC-BY-4.0 (icons) | `src/ui/fonts/fa-solid-900.ttf` ([LICENSE](../src/ui/fonts/LICENSE)) | https://fontawesome.com |

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
[src/ui/vendor/mermaid/LICENSE](../src/ui/vendor/mermaid/LICENSE).

Mermaid is held on 11.x on purpose. Mermaid 12 depends on elkjs `^0.9.3`, and
elkjs before 0.12 is licensed under EPL-2.0 only, which is not compatible with
MemoryStick's GPL-3.0-or-later. elkjs 0.12 and later are available under
EPL-2.0 OR GPL-3.0-or-later, so the hold can be lifted once a Mermaid release
depends on elkjs 0.12 or later.

## Rust libraries

The executable also links the Rust crates its code is built on: Tauri,
comrak, serde and the libraries they depend on, about 300 of them, mostly
under MIT, Apache-2.0, BSD or ISC. Their licence texts travel inside every
build, and Preferences > About > Third-party licences shows them.

`build.rs` collects them at build time. It asks
`cargo metadata --locked --offline --filter-platform <target>` for the
dependency graph, follows the normal dependencies of `memorystick` (build
and dev dependencies run on the build machine and are not linked in), and
reads every `LICENSE*`, `LICENCE*`, `UNLICENSE*`, `COPYING*`, `COPYRIGHT*`
and `NOTICE*` file, plus any declared `license-file`, at the top of each
crate Cargo downloaded to build it. Identical texts are kept once, under
the list of crates that ship them. A crate that ships no licence file is
listed with the SPDX expression from its manifest, whose standard text is
at https://spdx.org/licenses/. The result is embedded in the executable,
so each platform's build carries the notices of exactly the crates it
links, and the build needs no network beyond what Cargo already fetched.
