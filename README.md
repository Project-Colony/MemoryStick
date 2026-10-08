# MemoryStick

MemoryStick is a Markdown viewer for Windows, macOS and Linux, written in Rust with Tauri. It is software from the Colony project (https://github.com/Project-Colony/Colony). It was formerly called MD Viewer.

## Features

- GitHub Flavored Markdown, rendered by [comrak](https://github.com/kivikakk/comrak): tables, task lists, strikethrough, autolinks, footnotes and description lists; a YAML front matter block is left out of the page
- Syntax highlighting with highlight.js (its 36 common languages), in light and dark themes
- Math with KaTeX: `$...$`, `$$...$$` and ```` ```math ```` blocks
- Mermaid diagrams
- A table of contents panel built from the document's headings
- Dark mode
- Relative images resolved from the document's folder
- Open a file with the Open button, by dropping it on the window, or by passing its path on the command line

[sample.md](sample.md) shows most of these.

## Install

- **With Colony:** install MemoryStick from [Colony](https://github.com/Project-Colony/Colony), which checks each release's signature before installing it.
- **From a release:** download the file for your system from [Releases](https://github.com/Project-Colony/MemoryStick/releases): `memorystick-linux`, `memorystick-windows.exe`, `memorystick-macos` (Apple silicon) or `memorystick-macos-x86` (Intel). Each is a single executable; on Linux and macOS, make it executable with `chmod +x`.

MemoryStick uses the system's web view: WebView2 on Windows (included in Windows 11 and current Windows 10), WebKit on macOS, and WebKitGTK 4.1 on Linux (for example `libwebkit2gtk-4.1-0` on Debian and Ubuntu, `webkit2gtk-4.1` on Arch Linux).

## Usage

```bash
memorystick notes.md
```

or start MemoryStick and open a file from the window.

| Shortcut | Action |
|---|---|
| Ctrl+O | Open a file |
| Ctrl+D | Toggle dark mode |
| Ctrl+R | Reload the current file |

On macOS, use Cmd instead of Ctrl.

## Build from source

Prerequisites: Rust (stable toolchain) and, on Linux, the WebKitGTK 4.1 development package (`libwebkit2gtk-4.1-dev` on Debian and Ubuntu).

```bash
cargo build --release --locked --manifest-path src-tauri/Cargo.toml
```

The executable is `src-tauri/target/release/memorystick`. No Tauri CLI or Node.js is needed: the interface in `dist/` is plain HTML, CSS and JavaScript, embedded in the executable at build time.

## Structure

- `src-tauri/`: the Rust application, which reads a file and renders its Markdown to HTML with comrak
- `dist/`: the interface, which adds syntax highlighting, math, diagrams and the table of contents
- `dist/vendor/`: highlight.js, KaTeX and Mermaid, unmodified

## Releases

Releases are cut by release-please. Merging its release pull request tags the version, and the release workflow builds the four executables from that tag, signs them and publishes them.

## Code signing policy

Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

Windows builds are signed this way once the SignPath Foundation has accepted
the project; until then they ship without Authenticode. Every release asset,
on every platform, is always signed with the Project-Colony organisation's
ed25519 key, which Colony verifies before installing it.

- Committers and reviewers: [MotherSphere](https://github.com/MotherSphere)
- Approvers: [MotherSphere](https://github.com/MotherSphere)

### Privacy policy

MemoryStick has no account, telemetry, analytics, crash reporting or update
check, and its own code makes no network requests. It keeps no history or
settings of its own.

A document can point to content on the web, and MemoryStick displays it the
way a browser would: when a document you open contains images hosted on the
web, they load as soon as the document is shown, and the servers hosting them
receive your IP address and can tell that the document was opened. Documents
can also contain HTML, which is displayed without filtering, so only open
documents from sources you trust.

MemoryStick's interface is rendered by the operating system's web view
(Microsoft Edge WebView2 on Windows, WebKit on macOS and Linux), which follows
its vendor's own privacy policy.

## License

GPL-3.0-or-later. You may redistribute and modify MemoryStick under the terms
of version 3 of the GNU General Public License, or (at your option) any later
version. The full text is in [LICENSE](LICENSE).

The libraries in `dist/vendor/` keep their own licences: BSD-3-Clause for
highlight.js, MIT for KaTeX and Mermaid, OFL-1.1 for the KaTeX fonts, and
those of the packages Mermaid's build contains. See
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
