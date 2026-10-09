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

> **Status:** the viewer has been in use under its former name, MD Viewer.
> Nothing has been released under the MemoryStick name yet; the
> first release, signed by the Project-Colony organisation, is pending. CI
> builds and tests the Rust side on Linux, Windows and macOS, but nothing in CI
> opens the window, so the interface is only checked by hand.

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

Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

Windows builds are signed this way once the SignPath Foundation has accepted
the project; until then they ship without Authenticode. Every release asset,
on every platform, is always signed with the Project-Colony organisation's
ed25519 key, which Colony verifies before installing it. Releases are cut by
release-please: merging its release pull request tags the version, and the
release workflow builds the four executables from that tag, signs them and
publishes them.

- Committers and reviewers: [MotherSphere](https://github.com/MotherSphere)
- Approvers: [MotherSphere](https://github.com/MotherSphere)

### Privacy policy

MemoryStick has no account, telemetry, analytics, crash reporting or update
check, and its own code makes no network requests. It keeps no history. Its
preferences, such as the theme you pick, are a file on your device:
`~/.config/Colony/MemoryStick/preferences/preferences.json` on Linux,
`%LOCALAPPDATA%\Colony\MemoryStick\preferences\preferences.json` on
Windows and
`~/Library/Application Support/Colony/MemoryStick/preferences/preferences.json`
on macOS. The web view that shows documents runs in private mode, so it
keeps nothing between runs.

MemoryStick reads only the files you open yourself, through its Open
dialog, by dropping them on its window or by naming them on the command
line, and only Markdown and text files (`.md`, `.markdown`, `.mdown`,
`.mkd`, `.mkdn` and `.txt`). It also reads the images a document shows
(PNG, JPEG, GIF, WebP, AVIF, SVG, BMP and ICO files), but only from that
document's folder and the folders inside it. A document cannot make it open
any other file, or a network share by its address, such as
`\\server\share` on Windows. It does not upload, copy or keep these
files.

Documents can contain HTML, which is displayed, apart from scripts, frames,
embedded objects and style sheets, which are left out: no script contained in
a document is ever run. The window always shows MemoryStick's own page: nothing
in a document can navigate it to another page or file. Web and email links
open in your default browser or mail app, and only when you click them;
links to a place in the document scroll to it, and other links do nothing.

Nothing in a document can make MemoryStick reach the network: images,
style sheets, fonts, frames, audio and video hosted on the web are never
loaded, so a document shows an image only when it is on your device. A web
link reaches the network only through your browser, once you click it.

MemoryStick's interface is rendered by the operating system's web view
(Microsoft Edge WebView2 on Windows, WebKit on macOS and Linux), which follows
its vendor's own privacy policy.

## License

GPL-3.0-or-later. You may redistribute and modify MemoryStick under the terms
of version 3 of the GNU General Public License, or (at your option) any later
version. The full text is in [LICENSE](LICENSE).

The libraries in `dist/vendor/` keep their own licences: BSD-3-Clause for
highlight.js, MIT for KaTeX and Mermaid, OFL-1.1 for the KaTeX fonts,
GPL-3.0-or-later for the Colony themes, and those of the packages Mermaid's
build contains. The fonts in `dist/fonts/` are under OFL-1.1. See
[docs/third-party-notices.md](docs/third-party-notices.md).
