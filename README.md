<div align="center">

**A Markdown viewer for Windows, macOS and Linux, for reading a Markdown file the way it was meant to look without opening an editor or a browser.**

</div>

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Colony app](https://img.shields.io/badge/Colony-office-purple)](https://github.com/Project-Colony/Colony)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20windows%20%7C%20macOS-lightgrey)](#installation)

A Markdown file opened in a text editor shows its source: tables, math and
diagrams stay raw. Rendering it usually means an editor's preview pane or a
code hosting site. MemoryStick opens the file in a window of its own and
renders it the way GitHub does, with highlighted code, math, Mermaid diagrams
and a table of contents, from a single executable.

> **Status:** released. Since v0.1.0, every MemoryStick release is built in
> CI from a release-please tag, signed by the Project-Colony organisation, and
> verified by Colony before it installs or updates it. Before joining Project
> Colony, the viewer was in use under its former name, MD Viewer. CI builds and
> tests the Rust side (Markdown rendering, which files a document may read, the
> preferences) on Linux, Windows and macOS, and on Rust 1.90, the oldest
> supported version. The release workflow checks that each executable answers
> `--version` with the release's version, and that the Intel macOS build is an
> x86_64 binary. Nothing in CI opens the window, so the interface, Preferences,
> the themes and the French translation are checked by hand.

## Why MemoryStick

- **A text editor** shows the Markdown source, not the document.
- **An editor's preview pane** renders it, but only inside that editor, and
  usually without math or diagrams unless you install extensions.
- **A code hosting site** renders it well, but only once the file is pushed
  there, and only online.
- **MemoryStick** opens a `.md` file directly, from a double-click, a drop on
  its window or the command line, and needs nothing else installed beyond the
  system's web view. It has no account and no telemetry.

## What it does

- GitHub Flavored Markdown, rendered by [comrak](https://github.com/kivikakk/comrak): tables, task lists, strikethrough, autolinks, footnotes and description lists; a YAML front matter block is left out of the page
- Syntax highlighting with highlight.js (its 36 common languages)
- Math with KaTeX: `$...$`, `$$...$$` and ```` ```math ```` blocks
- Mermaid diagrams
- A table of contents panel built from the document's headings, which marks the section you are reading
- Every Colony theme (Gruvbox, Catppuccin, Nord, Rosé Pine and the others): MemoryStick follows the system's light or dark mode in Gruvbox until you pick a theme
- Preferences, behind the MemoryStick name in the toolbar as in every Colony program: theme, accent colour, text size, animations, high contrast, the OpenDyslexic font, reduced motion, and the interface in English or French
- Images on your device shown from the document's folder and the folders inside it
- Open a file with the Open button, by dropping it on the window, or by passing its path on the command line: `.md`, `.markdown`, `.mdown`, `.mkd`, `.mkdn` and `.txt` files of up to 32 MiB
- Web and email links open in your browser or mail app; images hosted on the web are not loaded

```bash
memorystick notes.md
```

| Shortcut | Action |
|---|---|
| Ctrl+O | Open a file |
| Ctrl+D | Switch the theme between its light and dark variants |
| Ctrl+R, F5 | Reload the current file |
| Escape | Close Preferences |

On macOS, use Cmd instead of Ctrl. [docs/sample.md](docs/sample.md) shows
most of the features.

## Installation

### Via Colony (recommended)

Search for **MemoryStick** in [Colony](https://github.com/Project-Colony/Colony)
and install it. Colony checks each release's signature before installing it,
and updates arrive through the launcher.

### Direct binary download

Grab the asset for your platform from the
[latest release](../../releases/latest). Each is a single executable.

| Platform | Asset |
|---|---|
| Linux | `memorystick-linux` |
| Windows | `memorystick-windows.exe` |
| macOS (Apple Silicon) | `memorystick-macos` |
| macOS (Intel) | `memorystick-macos-x86` |

```bash
chmod +x memorystick-linux && ./memorystick-linux
```

MemoryStick uses the system's web view: WebView2 on Windows (included in
Windows 11 and current Windows 10), WebKit on macOS, and WebKitGTK 4.1 on
Linux (for example `libwebkit2gtk-4.1-0` on Debian and Ubuntu,
`webkit2gtk-4.1` on Arch Linux).

### Build from source

```bash
git clone https://github.com/Project-Colony/MemoryStick
cd MemoryStick
cargo build --release --locked --manifest-path src-tauri/Cargo.toml
```

Requires Rust 1.90 or newer and, on Linux, the WebKitGTK 4.1 development
package (`libwebkit2gtk-4.1-dev` on Debian and Ubuntu). The executable is
`src-tauri/target/release/memorystick`. No Tauri CLI or Node.js is needed:
`src-tauri/` is the Rust application, which reads a file and renders its
Markdown to HTML, and `dist/` is the interface, plain HTML, CSS and
JavaScript embedded in the executable at build time.

## Documentation

- [docs/sample.md](docs/sample.md): a document that exercises most features;
  open it in MemoryStick to see them.
- [docs/third-party-notices.md](docs/third-party-notices.md): the libraries
  MemoryStick ships and their licences.
- [SECURITY.md](SECURITY.md): how to report a vulnerability.

## Code signing policy

Every release asset is signed by the Project Colony organisation in CI, never on
a developer machine. Next to each asset on the release page:

| File | What it is |
|---|---|
| `<asset>.sig` | an ed25519 signature over the asset, made with the organisation's release key |
| `<asset>.meta` | three lines binding the asset to its file name, its sha256 and the release version |
| `<asset>.meta.sig` | an ed25519 signature over the `.meta` |

The private key is an organisation secret, used only by the shared
[sign-and-publish workflow](https://github.com/Project-Colony/Project-Colony-Resources/blob/main/.github/workflows/sign-and-publish.yml)
in a job that builds nothing; the jobs that compile MemoryStick never see it.
Colony checks all three files before it installs or updates MemoryStick, and
refuses a release older than the one installed. To check a download yourself
with OpenSSL 3 (the same commands work for every asset):

```bash
cat > colony-release.pub <<'EOF'
-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEARNjg3Nn8H6/aBg1unwGjkUTcrdTxERNefVaqU8cFu0s=
-----END PUBLIC KEY-----
EOF
a=memorystick-linux
openssl pkeyutl -verify -pubin -inkey colony-release.pub -rawin -in "$a" -sigfile "$a.sig"
openssl pkeyutl -verify -pubin -inkey colony-release.pub -rawin -in "$a.meta" -sigfile "$a.meta.sig"
cat "$a.meta"     # version=<tag>, asset=<file name>, sha256=<digest>
sha256sum "$a"    # the digest must equal the sha256 line
```

How releases are built, signed and published:
[design/releases.md](https://github.com/Project-Colony/Project-Colony-Resources/blob/main/design/releases.md#5-signing).

<!--
Add the following only once SignPath signs the Windows build, that is once
`signpath-project-slug` is set in .github/workflows/release.yml. Before that it
would promise a signature the .exe does not carry.

Windows releases are also Authenticode-signed. Free code signing provided by
[SignPath.io](https://about.signpath.io/), certificate by
[SignPath Foundation](https://signpath.org/).

- Committers and reviewers: [MotherSphere](https://github.com/MotherSphere)
- Approvers: [MotherSphere](https://github.com/MotherSphere)
-->

## Privacy

MemoryStick has no account and sends no telemetry, no analytics and no crash
reports. It has no update check: updates come through Colony.

| Data | Stored or sent | Where, and why |
|---|---|---|
| Preferences | stored | `Colony/MemoryStick/preferences/preferences.json` in this machine's config directory: `~/.config/` on Linux, `%LOCALAPPDATA%\` on Windows, `~/Library/Application Support/` on macOS. Theme, accent colour, text size, language and the other choices in Preferences, so they are back next time. |
| Web view folder | stored | `~/.cache/Colony/MemoryStick/webview/` on Linux, `%LOCALAPPDATA%\Colony\MemoryStick\cache\webview\` on Windows; on macOS WebKit keeps its own. The web view runs in private mode, so it keeps no page, cookie or history there between runs. |
| The document you open | read, never stored | The `.md`, `.markdown`, `.mdown`, `.mkd`, `.mkdn` or `.txt` file you pick in the Open dialog, drop on the window or name on the command line, up to 32 MiB. MemoryStick does not upload, copy or keep it, and keeps no list of opened files. |
| Images in that document | read, never stored | PNG, JPEG, GIF, WebP, AVIF, SVG, BMP and ICO files, only from the document's folder and the folders inside it, to show them in the page. |
| Recent files | stored by the operating system | The system's Open dialog may add the file you pick to the system's recent files list (Windows recent items, `~/.local/share/recently-used.xbel` with GTK on Linux). MemoryStick does not write or control that list. |

A document cannot make MemoryStick read any other file, or a network share by
its address, such as `\\server\share` on Windows. Documents can contain HTML,
which is displayed, apart from scripts, frames, embedded objects and style
sheets, which are left out: no script contained in a document is ever run. The
window always shows MemoryStick's own page: nothing in a document can navigate
it to another page or file.

MemoryStick connects to no server. Nothing in a document can make it reach the
network: images, style sheets, fonts, frames, audio and video hosted on the web
are never loaded, so a document shows an image only when it is on your device.
Web and email links open in your default browser or mail app, and only when you
click them; links to a place in the document scroll to it, and other links do
nothing.

MemoryStick's interface is rendered by the operating system's web view
(Microsoft Edge WebView2 on Windows, WebKit on macOS and Linux), which follows
its vendor's own privacy policy.

This program will not transfer any information to other networked systems
unless specifically requested by the user or the person installing or operating
it.

## License

GPL-3.0-or-later. You may redistribute and modify MemoryStick under the terms
of version 3 of the GNU General Public License, or (at your option) any later
version. The full text is in [LICENSE](LICENSE).

The libraries in `dist/vendor/` keep their own licences: BSD-3-Clause for
highlight.js, MIT for KaTeX and Mermaid, OFL-1.1 for the KaTeX fonts,
GPL-3.0-or-later for the Colony themes, and those of the packages Mermaid's
build contains. The fonts in `dist/fonts/` are under OFL-1.1. See
[docs/third-party-notices.md](docs/third-party-notices.md).
